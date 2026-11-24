//! Owner Task A: same-phase per-row smooth-twin comparison in inches.
use vstone_core::rectified::smooth_twin_margins;
#[test]
fn archived_bad_cylinder_fails_both_relative_margins() {
    let margins = smooth_twin_margins(
        [-0.7662377537559593, 0.723921466100764],
        [-0.07788261772250138, 0.1578330245263988],
        0.25,
    );
    assert!((margins[0] + 0.23835513603345787).abs() < 1e-12);
    assert!((margins[1] + 0.46608844157436524).abs() < 1e-12);
}
#[test]
fn twin_relative_boundary_is_inclusive_without_absolute_outward_cap() {
    assert_eq!(smooth_twin_margins([-0.45, 0.6], [0., 0.5], 0.25), [0., 0.]);
    assert!(smooth_twin_margins([-0.450001, 0.6], [0., 0.5], 0.25)[0] < 0.);
    assert!(smooth_twin_margins([-0.45, 0.600001], [0., 0.5], 0.25)[1] < 0.);
}
