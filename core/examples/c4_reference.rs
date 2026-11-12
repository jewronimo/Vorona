//! Exact-reference input gate: reconstruct SciPy geometry before any barrier continuation.
use vstone_core::{
    model::CoreInput,
    planes::{self, ReferenceInput},
    surface::Surface,
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let raw = std::fs::read_to_string(&args[1]).expect("reference export JSON");
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let reference: ReferenceInput = serde_json::from_str(&raw).unwrap();
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.params.depth = value["identity"]["depth"].as_f64().unwrap();
    let stage = args.get(2).map(String::as_str).unwrap_or("fit");
    assert!([
        "start",
        "fit",
        "barriers",
        "native-start",
        "native-fit",
        "native-barriers"
    ]
    .contains(&stage));
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let result = planes::reference_stage(&surface, &input.params, &reference, stage);
    if stage == "fit" {
        let g = result.as_ref().unwrap();
        let max_error = g
            .corners
            .iter()
            .zip(&reference.reference_corners)
            .flat_map(|(a, b)| (0..3).map(move |i| (a[i] - b[i]).abs()))
            .fold(0f64, f64::max);
        assert!(max_error < 1e-8, "reference corner mismatch {max_error}");
    }
    let geometry = match &result {
        Ok(g) => Some(g),
        Err(e) => e.best.as_deref(),
    };
    let metrics = geometry.map(|g| {
        let mut edges=Vec::new(); let mut angles=Vec::new(); let mut signed=Vec::new();
        for &i in &reference.keep {
            let ids=&g.polygons[i];
            for k in 0..ids.len() {
                let a=g.corners[ids[(k+ids.len()-1)%ids.len()]];
                let b=g.corners[ids[k]];
                let c=g.corners[ids[(k+1)%ids.len()]];
                let u: [f64;3]=std::array::from_fn(|j| a[j]-b[j]);
                let v: [f64;3]=std::array::from_fn(|j| c[j]-b[j]);
                let norm=|q:[f64;3]| q.iter().map(|a| a*a).sum::<f64>().sqrt();
                edges.push(norm(v));
                angles.push((u.iter().zip(v).map(|(a,b)| a*b).sum::<f64>()/norm(u)/norm(v)).clamp(-1.,1.).acos().to_degrees());
                signed.push(b[0].hypot(b[2])-9.);
            }
        }
        let range=|mut q:Vec<f64>| {q.sort_by(f64::total_cmp); serde_json::json!({"min":q.first(),"max":q.last()})};
        let rms=(g.corners.iter().zip(&reference.targets).map(|(a,b)| (0..3).map(|j| (a[j]-b[j]).powi(2)).sum::<f64>()).sum::<f64>()/g.corners.len() as f64).sqrt();
        let tilt_norm_sq = g.planes.iter().enumerate().map(|(i,p)| {
            let dot = |a:[f64;3],b:[f64;3]| a.iter().zip(b).map(|(a,b)|a*b).sum::<f64>();
            let radial = dot(p.n,reference.radial[i]);
            (dot(p.n,reference.etheta[i])/radial).powi(2)+(p.n[1]/radial).powi(2)
        }).sum::<f64>();
        serde_json::json!({"prototype_objective_sum_sq":rms*rms*g.corners.len() as f64+0.0025*tilt_norm_sq,"whole_tiles":reference.keep.len(),"edges":range(edges),"unsigned_angles":range(angles),"signed_corners":range(signed),"target_rms":rms,"reference_corner_max_error":g.corners.iter().zip(&reference.reference_corners).flat_map(|(a,b)|(0..3).map(move |j|(a[j]-b[j]).abs())).fold(0f64,f64::max)})
    });
    println!(
        "{}",
        serde_json::json!({"identity":value["identity"],"stage":stage,"metrics":metrics,"result":result})
    );
}
