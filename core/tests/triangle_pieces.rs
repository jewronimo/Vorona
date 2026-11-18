//! Native A06 joint and clipping regressions; legacy kernels are not substituted.
use vstone_core::{model::Plane, pieces, rims};

#[test]
fn near_coplanar_joint_is_square_and_regular_joint_is_bisected() {
    let face = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    for angle in [0_f64, 0.01, 1.99] {
        let r = angle.to_radians();
        let other = Plane {
            n: [0., r.sin(), r.cos()],
            d: 0.,
        };
        let side = pieces::triangle_joint(&face, &other, [0.; 3], [2., 0., 0.], 7).unwrap();
        assert_eq!(side.neighbor, Some(7));
        assert_eq!(side.kind, "joint");
        let reverse = pieces::triangle_joint(&other, &face, [2., 0., 0.], [0.; 3], 8).unwrap();
        for k in 0..3 {
            assert!((side.plane.n[k] + reverse.plane.n[k]).abs() < 1e-14);
        }
        assert!((side.plane.d + reverse.plane.d).abs() < 1e-14);
        assert!((side.plane.n[2].abs().asin().to_degrees() - angle / 2.).abs() < 1e-10);
        assert_eq!(side.plane.d, 0.);
    }
    let r = 3_f64.to_radians();
    let other = Plane {
        n: [0., r.sin(), r.cos()],
        d: 0.,
    };
    let side = pieces::triangle_joint(&face, &other, [0.; 3], [2., 0., 0.], 7).unwrap();
    let bevel = side.plane.n[2].abs().asin().to_degrees();
    assert!((bevel - 1.5).abs() < 1e-10);
    assert!(pieces::joint(&face, &face, 7).is_err()); // A05 unchanged
}

#[test]
fn exact_coplanar_square_solid_has_zero_bevel() {
    let face = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    let outer = vec![[0., 0., 0.], [2., 0., 0.], [2., 2., 0.], [0., 2., 0.]];
    let sides = (0..4)
        .map(|i| pieces::triangle_joint(&face, &face, outer[i], outer[(i + 1) % 4], i).unwrap())
        .collect();
    let solid = pieces::build(&face, outer, sides, 0.375).unwrap();
    assert!(solid.bevel_deg.iter().all(|&x| x == 0.));
    assert!(solid.inner.iter().all(|x| x[2] == -0.375));
}

#[test]
fn clipping_near_existing_vertex_preserves_side_incidence() {
    let face = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    // A crossing only a few ulps beyond an existing corner reproduces a
    // duplicate clipped vertex; its zero-length side must not replace a joint.
    let outer = vec![[0., -1e-15, 0.], [2., 1., 0.], [0., 2., 0.]];
    let sides = (0..3)
        .map(|i| pieces::triangle_joint(&face, &face, outer[i], outer[(i + 1) % 3], i).unwrap())
        .collect::<Vec<_>>();
    let rims::LevelCut::Kept { outer, sides, .. } =
        rims::triangle_level(&outer, &sides, 0., false, 0.1).unwrap()
    else {
        panic!("valid partial")
    };
    pieces::build(&face, outer, sides, 0.375).unwrap();
}

#[test]
#[ignore = "parent-owned release numerical gate"]
fn smooth_cylinder_square_joints_have_no_invalid_input_solids() {
    use vstone_core::{
        model::{CoreInput, Pattern},
        rectified_assembly,
        surface::Surface,
    };
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_a06_cylinder.json"
    ))
    .unwrap();
    input.params.pattern = Pattern::VoronoiTriangles;
    input.params.min_triangle_edge = 1.5;
    input.params.max_polygon_edge = 3.;
    input.params.count_step = 2;
    input.params.depth = 0.;
    input.params.randomness = 0.;
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let a = rectified_assembly::solve_assembly(&surface, &input.params).unwrap();
    assert_eq!(a.rim_heights, surface.y_range());
    assert!(a.pieces.iter().any(|p| p.partial));
    assert!(a
        .pieces
        .iter()
        .filter(|p| p.partial)
        .all(|p| p.bounding_square >= 1.5 - 1e-7));
    assert!(a
        .pieces
        .iter()
        .filter(|p| p.partial)
        .flat_map(|p| &p.edges)
        .any(|e| e.length < input.params.min_triangle_edge));
    let ids: std::collections::BTreeSet<_> = a.pieces.iter().map(|p| &p.id).collect();
    assert!(a
        .pieces
        .iter()
        .flat_map(|p| &p.edges)
        .all(|e| e.neighbor.as_ref().is_none_or(|id| ids.contains(id))));
    assert!(
        !a.solid_errors.iter().any(|s| s.contains("in-band solid")),
        "{:?}",
        a.solid_errors
    );
    assert!(a.solid_errors.is_empty(), "{:?}", a.solid_errors);
    assert!(a.violations.is_empty(), "{:?}", a.violations);
    assert!(a.closure.is_ok(), "{:?}", a.closure);
}

#[test]
fn rim_partial_minimum_square_is_not_waived_by_edge_exemption() {
    let face = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    let outer = vec![[0., -1., 0.], [2., 1., 0.], [0., 2., 0.]];
    let sides = (0..3)
        .map(|i| pieces::triangle_joint(&face, &face, outer[i], outer[(i + 1) % 3], i).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        rims::triangle_level(&outer, &sides, 0., false, 4.).unwrap(),
        rims::LevelCut::Sliver
    ));
    assert!(matches!(
        rims::triangle_level(&outer, &sides, 0., false, 1.).unwrap(),
        rims::LevelCut::Kept { partial: true, .. }
    ));
}

#[test]
fn near_vertex_cut_provenance_survives_every_cyclic_origin_and_cut_direction() {
    let face = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    for top in [false, true] {
        let mut outer = if top {
            vec![[0., 1e-15, 0.], [0., -2., 0.], [2., -1., 0.]]
        } else {
            vec![[0., -1e-15, 0.], [2., 1., 0.], [0., 2., 0.]]
        };
        for _ in 0..3 {
            let sides = (0..3)
                .map(|i| {
                    pieces::triangle_joint(&face, &face, outer[i], outer[(i + 1) % 3], i).unwrap()
                })
                .collect::<Vec<_>>();
            let rims::LevelCut::Kept {
                outer: clipped,
                sides,
                ..
            } = rims::triangle_level(&outer, &sides, 0., top, 0.1).unwrap()
            else {
                panic!("non-sliver partial");
            };
            pieces::build(&face, clipped, sides, 0.375).unwrap();
            outer.rotate_left(1);
        }
    }
}

#[test]
fn historical_cut_placement_helper_remains_separate_from_fixed_design_rims() {
    let face = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    // At y=0 the retained strip is .25in tall and 1in wide: undersized.
    // The historical helper can move the saw. A06 production deliberately does
    // NOT use it: the full assembly regression above gates fixed design rims.
    let outer = vec![[0., -1., 0.], [1., -1., 0.], [1., 0.25, 0.], [0., 0.25, 0.]];
    let sides = (0..4)
        .map(|k| pieces::triangle_joint(&face, &face, outer[k], outer[(k + 1) % 4], k).unwrap())
        .collect();
    let raw = vec![(outer, sides)];
    let (height, rejected) = rims::triangle_place(&raw, 0., false, 1., 1.5);
    assert!(height > 0.);
    assert!(rejected.is_empty());
    assert!(matches!(
        rims::triangle_level(&raw[0].0, &raw[0].1, height, false, 1.5).unwrap(),
        rims::LevelCut::Outside
    ));
}

#[test]
fn row_phase_rebuilds_on_surface_and_preserves_seeded_determinism() {
    use vstone_core::{model::CoreInput, rectified, surface::Surface};
    let input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_a06_cylinder.json"
    ))
    .unwrap();
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let zero = rectified::build(&surface, &input.params).unwrap();
    let phase = zero.rows[0].height / 3.;
    let shifted = rectified::build_with_phase(&surface, &input.params, phase).unwrap();
    assert_eq!(
        shifted,
        rectified::build_with_phase(&surface, &input.params, phase).unwrap()
    );
    assert!(zero
        .rows
        .iter()
        .zip(&shifted.rows)
        .all(|(a, b)| (b.s - a.s - phase).abs() < 1e-12));
    assert_eq!(surface.y_range(), [0., 60.]);
    assert!(rectified::build_with_phase(&surface, &input.params, -phase).is_err());
    assert!(rectified::build_with_phase(&surface, &input.params, zero.rows[0].height).is_err());
}

#[test]
fn shared_near_coplanar_plane_contains_both_inner_edges() {
    use nalgebra::Vector3;
    let angle = 1.9_f64.to_radians();
    let a = Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    let b = Plane {
        n: [0., angle.sin(), angle.cos()],
        d: 0.,
    };
    let x = [3., 0., 0.];
    let y = [7., 0., 0.];
    let first = pieces::triangle_joint(&a, &b, x, y, 1).unwrap();
    let second = pieces::triangle_joint(&b, &a, y, x, 0).unwrap();
    let n = Vector3::from(first.plane.n);
    // Intersect each inward face with this single shared side plane.
    for face in [&a, &b] {
        let f = Vector3::from(face.n);
        let inward = -0.375 * (f - n * f.dot(&n)) / (1. - f.dot(&n).powi(2));
        for endpoint in [x, y] {
            let inner = Vector3::from(endpoint) + inward;
            assert!((n.dot(&inner) - first.plane.d).abs() < 1e-12);
            assert!((Vector3::from(second.plane.n).dot(&inner) - second.plane.d).abs() < 1e-12);
            assert!((f.dot(&inner) + 0.375).abs() < 1e-12);
        }
    }
}
