//! Regression for A06 endpoint arithmetic perturbing A05 production closure.
use vstone_core::{assembly, closure, depth, model::CoreInput, surface::Surface};

#[test]
#[ignore = "focused release production solve; run explicitly"]
fn cone_depth_1_5_randomness_1_seed_9_closes() {
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.surface.profile = serde_json::from_str(include_str!(
        "../../tests/fixtures/profiles/cone_9_to_6.json"
    ))
    .unwrap();
    input.params.allowed_sides = vec![6];
    input.params.size_variation = 0.25;
    input.params.depth = 1.5;
    input.params.randomness = 1.;
    input.params.seed = 9;
    let selected = depth::select(&input);
    let d = selected
        .selected_depth
        .expect("A05 production selection failed");
    assert_eq!(
        d, 1.5,
        "configured-first cone should not require a fallback"
    );
    let topology = selected.topology.as_ref().unwrap();
    let geometry = selected
        .attempts
        .iter()
        .find(|a| a.depth == d)
        .and_then(|a| a.optimizer.as_ref())
        .and_then(|g| g.as_ref().ok())
        .unwrap();
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    input.params.depth = d;
    let assembled = assembly::assemble(topology, geometry, &surface, &input.params)
        .expect("A05 cone closure regressed (formerly S000025 CollapsedOffset)");
    let certificate = closure::certify(&assembled.pieces, assembled.rim_heights).unwrap();
    assert_eq!(certificate.paired_edges, 216);
}
