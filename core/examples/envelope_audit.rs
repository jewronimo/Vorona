//! Audit saved production outer corners without interpreting face_depth as a signed distance.
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};
use vstone_core::{model::CoreInput, planes::SurfaceQueries, rectified, surface::Surface};
fn vec3(v: &Value) -> [f64; 3] {
    serde_json::from_value(v.clone()).unwrap()
}
fn center(p: &Value) -> [f64; 3] {
    let a = p["outer"].as_array().unwrap();
    std::array::from_fn(|k| a.iter().map(|v| v[k].as_f64().unwrap()).sum::<f64>() / a.len() as f64)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let dir = Path::new(&args[1]);
    let mut paths: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with("-input.json"))
        .collect();
    paths.sort();
    let mut any_failed = false;
    for input_path in paths {
        let name = input_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches("-input.json");
        let input: CoreInput =
            serde_json::from_str(&fs::read_to_string(&input_path).unwrap()).unwrap();
        let surface = Surface::from_profile(&input.surface.profile)
            .unwrap()
            .for_pattern(input.params.pattern);
        let output_path =
            Path::new(args.get(2).unwrap_or(&args[1])).join(format!("{name}-output-0.json"));
        let out: Value = match fs::read_to_string(output_path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
        {
            Some(out) => out,
            None => {
                any_failed = true;
                println!(
                    "{}",
                    json!({"case":name,"error":"missing/incomplete output; not a geometric pass"})
                );
                continue;
            }
        };
        let Some(pieces) = out["pieces"].as_array().filter(|p| !p.is_empty()) else {
            any_failed = true;
            println!(
                "{}",
                json!({"case":name,"error":"no pieces", "status":out["status"],"message":out["message"]})
            );
            continue;
        };
        // Recover the exact saved phase by through-seed plane incidence, not by
        // assuming geometric centers lie on the seeds. IDs were renumbered by C6.
        let mut best = None;
        for phase in 0..16 {
            let t =
                rectified::build_with_phase(&surface, &input.params, phase as f64 / 16.).unwrap();
            let mut score = 0.;
            let mut mapping = BTreeMap::new();
            for p in pieces.iter().filter(|p| p["seed_depth"].is_number()) {
                let normal = vec3(&p["plane"]["n"]);
                let d = p["plane"]["d"].as_f64().unwrap();
                let c = center(p);
                let (i, e) = t
                    .delaunay
                    .tiles
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let q = surface.offset([s.u, s.v], s.depth);
                        let plane = (0..3).map(|k| normal[k] * q[k]).sum::<f64>() - d;
                        let depth = s.depth - p["seed_depth"].as_f64().unwrap();
                        let dist = (0..3).map(|k| (q[k] - c[k]).powi(2)).sum::<f64>();
                        (i, plane.powi(2) + depth.powi(2) + 1e-14 * dist)
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .unwrap();
                score += e;
                mapping.insert(p["id"].as_str().unwrap().to_string(), i);
            }
            if best.as_ref().is_none_or(|(v, _, _)| score < *v) {
                best = Some((score, t, mapping));
            }
        }
        let (score, t, mapping) = best.unwrap();
        let mut rows = BTreeMap::<usize, Vec<Value>>::new();
        let mut failures = 0;
        for p in pieces {
            let seeds: Vec<usize> = if let Some(&i) = mapping.get(p["id"].as_str().unwrap()) {
                vec![i]
            } else {
                let mut a: Vec<_> = p["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|e| mapping.get(e["neighbor"].as_str().unwrap_or("")).copied())
                    .collect();
                a.sort();
                a.dedup();
                a
            };
            if seeds.is_empty() {
                panic!("missing seed provenance {name} {}", p["id"]);
            }
            let base = seeds.iter().map(|&i| t.relief_bases[i]).sum::<f64>() / seeds.len() as f64;
            let row = t.delaunay.tiles[seeds[0]].row;
            let corners: Vec<_> = p["outer"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| surface.signed_distance(vec3(v)))
                .collect();
            let outer_points: Vec<_> = p["outer"].as_array().unwrap().iter().map(vec3).collect();
            let mut diameter_squared: f64 = 0.;
            for a in &outer_points {
                for b in &outer_points {
                    diameter_squared =
                        diameter_squared.max((0..3).map(|k| (a[k] - b[k]).powi(2)).sum());
                }
            }
            let kmax = seeds
                .iter()
                .map(|&i| {
                    surface
                        .principal_curvatures(t.delaunay.tiles[i].s)
                        .into_iter()
                        .fold(0., f64::max)
                })
                .sum::<f64>()
                / seeds.len() as f64;
            let sag = diameter_squared * kmax / 8.;
            let allowance = sag - base + 0.10;
            let lo = -(base + input.params.depth + 0.10);
            let failed = p["partial"] == false && corners.iter().any(|&d| d < lo || d > allowance);
            failures += usize::from(failed);
            let c = center(p);
            let outer = p["outer"].as_array().unwrap();
            let mut samples = vec![surface.signed_distance(c)];
            for k in 0..outer.len() {
                let a = vec3(&outer[k]);
                let b = vec3(&outer[(k + 1) % outer.len()]);
                samples.push(surface.signed_distance(std::array::from_fn(|j| (a[j] + b[j]) / 2.)));
            }
            rows.entry(row).or_default().push(json!({"id":p["id"],"partial":p["partial"],"seed_depth":p["seed_depth"],"base":base,"diameter":diameter_squared.sqrt(),"kmax":kmax,"sag":sag,"outward_allowance":allowance,"band":[lo,allowance],"corners":p["outer"],"incident_rows":seeds.iter().map(|&i|t.delaunay.tiles[i].row).collect::<Vec<_>>(),"corners_signed":corners,"center_midpoints_signed":samples,"failed":failed,"stored_face_depth":p["face_depth"]}));
        }
        any_failed |= failures > 0;
        println!(
            "{}",
            json!({"case":name,"phase_incidence_squared_error":score,"failed_whole_stones":failures,"rows":rows})
        );
    }
    if any_failed {
        std::process::exit(1);
    }
}
