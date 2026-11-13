use vstone_core::{depth, model::CoreInput};
fn input() -> CoreInput {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap()
}
#[test]
fn independent_feasibility_failure_preserves_evidence_and_size() {
    let mut i = input();
    i.params.allowed_sides = vec![5];
    i.params.depth = 0.25; // outside recommended range must not be refused
    let before = serde_json::to_string(&i).unwrap();
    let a = depth::select(&i);
    assert_eq!(a.attempts.len(), 1);
    assert!(!a.attempts[0].feasibility.feasible);
    assert!(a.attempts[0].optimizer.is_none());
    assert!(a.message.contains("C2 failure"));
    assert_eq!(a.selected_depth, None);
    assert_eq!(serde_json::to_string(&i).unwrap(), before);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&depth::select(&i)).unwrap()
    );
}
#[test]
fn invalid_depth_never_enters_search() {
    for d in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        let mut i = input();
        i.params.depth = d;
        let a = depth::select(&i);
        assert!(a.attempts.is_empty());
        assert!(a.selected_depth.is_none());
        assert!(a.message.contains("invalid configured depth"));
    }
}
#[test]
fn physical_bound_is_not_recommended_depth_range() {
    let mut i = input();
    i.params.allowed_sides = vec![5];
    i.params.depth = 100.;
    let a = depth::select(&i);
    assert_eq!(a.search_bound, 100.);
    assert_eq!(a.attempts[0].depth, 100.);
    assert!(a.message.contains("C2 failure"));
}
