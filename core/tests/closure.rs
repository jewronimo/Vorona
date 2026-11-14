use vstone_core::{closure, model::CoreOutput};
fn ring() -> CoreOutput {
    serde_json::from_str(include_str!("../../tests/fixtures/core_output_ring12.json")).unwrap()
}
#[test]
fn native_closure_rejects_holes_and_null_in_band_edges() {
    let o = ring();
    let h = [
        o.pieces
            .iter()
            .flat_map(|p| &p.outer)
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min),
        o.pieces
            .iter()
            .flat_map(|p| &p.outer)
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max),
    ];
    let c = closure::certify(&o.pieces, h).unwrap();
    assert!(c
        .rim_coverage_radians
        .iter()
        .all(|v| (v - std::f64::consts::TAU).abs() < closure::ANGLE_TOL));
    let mut missing = o.pieces.clone();
    missing.remove(0);
    assert!(closure::certify(&missing, h).is_err());
    let mut null = o.pieces.clone();
    let e = null[0]
        .edges
        .iter_mut()
        .find(|e| e.neighbor.is_some())
        .unwrap();
    e.neighbor = None;
    e.kind = "rim_whole".into();
    assert!(closure::certify(&null, h)
        .unwrap_err()
        .contains("null neighbor inside band"));
    let mut shifted = o.pieces.clone();
    shifted[0].outer[0][0] += 1e-3;
    assert!(closure::certify(&shifted, h).is_err());
    let mut duplicate = o.pieces.clone();
    duplicate.push(o.pieces[0].clone());
    assert!(closure::certify(&duplicate, h).is_err());
    let mut fold = o.pieces.clone();
    fold[0].plane.n[0] *= -1.;
    fold[0].plane.n[2] *= -1.;
    assert!(closure::certify(&fold, h).is_err());
}
