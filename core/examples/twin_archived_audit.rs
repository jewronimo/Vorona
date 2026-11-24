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
        if input.params.depth == 0. {
            continue;
        }
        let phase = (0..16)
            .find(|&k| {
                rectified::build_with_phase(&surface, &input.params, k as f64 / 16.).unwrap() == t
            })
            .unwrap() as f64
            / 16.;
        let mut smooth_params = input.params.clone();
        smooth_params.depth = 0.;
        let smooth_t = rectified::build_with_phase(&surface, &smooth_params, phase).unwrap();
        assert_eq!(t.rows, smooth_t.rows);
        for (a, b) in t.delaunay.tiles.iter().zip(&smooth_t.delaunay.tiles) {
            assert_eq!((a.u, a.v), (b.u, b.v));
        }
        let start = std::time::Instant::now();
        let result = vstone_core::rectified_planes::solve(&surface, &smooth_t, &smooth_params);
        let seconds = start.elapsed().as_secs_f64();
        let mut rows = BTreeMap::<usize, Vec<f64>>::new();
        for p in pieces.iter().filter(|p| p["partial"] == false) {
            let seeds: Vec<usize> = if let Some(&i) = mapping.get(p["id"].as_str().unwrap()) {
                vec![i]
            } else {
                p["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|e| mapping.get(e["neighbor"].as_str().unwrap_or("")).copied())
                    .collect()
            };
            assert!(!seeds.is_empty());
            let incident: std::collections::BTreeSet<_> =
                seeds.iter().map(|&i| t.delaunay.tiles[i].row).collect();
            for row in incident {
                for q in p["outer"].as_array().unwrap() {
                    rows.entry(row)
                        .or_default()
                        .push(surface.signed_distance(vec3(q)));
                }
            }
        }
        match result {
            Err(e) => {
                any_failed = true;
                println!(
                    "{}",
                    json!({"case":name,"phase":phase,"seconds":seconds,"twin_failure":e,"relief_rows":rows})
                );
            }
            Ok(g) => {
                let assembly = vstone_core::rectified_assembly::assemble(
                    &smooth_t,
                    &g,
                    &surface,
                    &smooth_params,
                )
                .unwrap();
                let mut reports = Vec::new();
                let mut passed = assembly.violations.is_empty()
                    && assembly.solid_errors.is_empty()
                    && assembly.closure.is_ok();
                for (row, values) in rows {
                    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
                    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let sr = assembly
                        .outer_envelope_rows
                        .iter()
                        .find(|r| r.row == row)
                        .unwrap();
                    let outward = sr.signed_outer_corners[2] + 0.10 - hi;
                    let inward = lo - sr.signed_outer_corners[0] + input.params.depth + 0.20;
                    passed &= outward >= 0. && inward >= 0.;
                    reports.push(json!({"row":row,"relief":[lo,hi],"smooth":[sr.signed_outer_corners[0],sr.signed_outer_corners[2]],"outward_margin":outward,"inward_margin":inward}));
                }
                println!(
                    "{}",
                    json!({"case":name,"phase":phase,"phase_incidence_squared_error":score,"seconds":seconds,"passed":passed,"rows":reports,"smooth_violations":assembly.violations,"smooth_solid_errors":assembly.solid_errors,"smooth_closure":assembly.closure})
                );
            }
        }
    }
    if any_failed {
        std::process::exit(1);
    }
}
