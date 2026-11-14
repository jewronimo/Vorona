use vstone_core::{
    model::CoreOutput,
    pieces::{self, Side},
    rims::{self, LevelCut},
};
fn fixture() -> CoreOutput {
    serde_json::from_str(include_str!("../../tests/fixtures/core_output_ring12.json")).unwrap()
}
fn sides(f: &CoreOutput, i: usize) -> Vec<Side> {
    f.pieces[i]
        .edges
        .iter()
        .map(|e| match &e.neighbor {
            Some(id) => {
                let j = f.pieces.iter().position(|p| &p.id == id).unwrap();
                pieces::joint(&f.pieces[i].plane, &f.pieces[j].plane, j).unwrap()
            }
            None => Side {
                plane: vstone_core::model::Plane {
                    n: e.side_normal,
                    d: e.side_normal[1] * f.pieces[i].outer[e.from][1],
                },
                kind: "rim_level",
                neighbor: None,
            },
        })
        .collect()
}
#[test]
fn analytic_ring12_offsets_bevels_and_joint_quads() {
    let f = fixture();
    let mut solids = Vec::new();
    for (i, p) in f.pieces.iter().enumerate() {
        let solid = pieces::build(&p.plane, p.outer.clone(), sides(&f, i), 0.375).unwrap();
        for (a, b) in solid.inner.iter().zip(&p.inner) {
            for k in 0..3 {
                assert!((a[k] - b[k]).abs() < 1e-9);
            }
        }
        for (a, b) in solid.bevel_deg.iter().zip(&p.edges) {
            assert!((a - b.bevel_deg).abs() < 1e-9);
        }
        for (side, quad) in solid.sides.iter().zip(&solid.quads) {
            for point in quad {
                assert!(
                    ((0..3).map(|k| side.plane.n[k] * point[k]).sum::<f64>() - side.plane.d).abs()
                        < 1e-9
                );
            }
        }
        solids.push(solid);
    }
    for (i, s) in solids.iter().enumerate() {
        for (edge, side) in s.sides.iter().enumerate() {
            if let Some(j) = side.neighbor {
                let other = solids[j]
                    .sides
                    .iter()
                    .position(|s| s.neighbor == Some(i))
                    .unwrap();
                for a in s.quads[edge] {
                    assert!(solids[j].quads[other]
                        .iter()
                        .any(|b| (0..3).all(|k| (a[k] - b[k]).abs() < 1e-9)));
                }
            }
        }
    }
}
#[test]
fn level_saw_and_sliver_are_explicit() {
    let f = fixture();
    let p = &f.pieces[0];
    let LevelCut::Kept {
        outer,
        sides,
        partial,
    } = rims::level(&p.outer, &sides(&f, 0), 3., true, 1.).unwrap()
    else {
        panic!()
    };
    assert!(partial);
    assert!(outer.iter().all(|p| p[1] <= 3.));
    let solid = pieces::build(&p.plane, outer, sides, 0.375).unwrap();
    for (s, q) in solid.sides.iter().zip(solid.quads) {
        if s.kind == "rim_level" && s.plane.n[1] > 0. {
            assert!(q.iter().all(|p| (p[1] - 3.).abs() < 1e-9));
        }
    }
    assert!(matches!(
        rims::level(&p.outer, &self::sides(&f, 0), 3., true, 20.).unwrap(),
        LevelCut::Sliver
    ));
}
#[test]
fn collapsed_thick_offset_returns_failure() {
    let f = fixture();
    let p = &f.pieces[0];
    assert!(pieces::build(&p.plane, p.outer.clone(), sides(&f, 0), 10.).is_err());
}

fn planar_sides(points: &[[f64; 3]], normal: [f64; 3]) -> Vec<Side> {
    use nalgebra::Vector3;
    (0..points.len())
        .map(|k| {
            let a = Vector3::from(points[k]);
            let b = Vector3::from(points[(k + 1) % points.len()]);
            let n = (b - a).cross(&Vector3::from(normal)).normalize();
            Side {
                plane: vstone_core::model::Plane {
                    n: n.into(),
                    d: n.dot(&a),
                },
                kind: "rim_whole",
                neighbor: None,
            }
        })
        .collect()
}
#[test]
fn thickness_setting_and_reflex_offset() {
    let points = vec![
        [0., 0., 0.],
        [3., 0., 0.],
        [3., 3., 0.],
        [1.5, 2.6, 0.],
        [0., 3., 0.],
    ];
    let face = vstone_core::model::Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    for thickness in [0.375, 0.75] {
        let s = pieces::build(
            &face,
            points.clone(),
            planar_sides(&points, face.n),
            thickness,
        )
        .unwrap();
        for (a, b) in s.outer.iter().zip(s.inner) {
            assert!((a[2] - b[2] - thickness).abs() < 1e-12);
            assert!((a[0] - b[0]).abs() < 1e-12);
            assert!((a[1] - b[1]).abs() < 1e-12);
        }
    }
}
#[test]
fn placement_precedes_rejection() {
    let p = vec![[0., 0., 0.], [1., 0., 0.], [1., 4., 0.], [0., 4., 0.]];
    let q: Vec<_> = p.iter().map(|a| [a[0], a[1] + 4., a[2]]).collect();
    let sides = planar_sides(&p, [0., 0., 1.]);
    let qs = planar_sides(&q, [0., 0., 1.]);
    assert!(matches!(
        rims::level(&p, &sides, 2., false, 2.5).unwrap(),
        LevelCut::Sliver
    ));
    let (height, failures) = rims::place(
        &[(p.clone(), sides.clone()), (q.clone(), qs.clone())],
        2.,
        false,
        4.,
        2.5,
    );
    assert!(height > 2.);
    assert!(failures.is_empty());
    assert!(matches!(
        rims::level(&p, &sides, height, false, 2.5).unwrap(),
        LevelCut::Outside
    ));
    assert!(matches!(
        rims::level(&q, &qs, height, false, 2.5).unwrap(),
        LevelCut::Kept { .. }
    ));
    let tall = vec![[0., 0., 0.], [1., 0., 0.], [1., 20., 0.], [0., 20., 0.]];
    let (_, failures) = rims::place(
        &[(tall.clone(), planar_sides(&tall, [0., 0., 1.]))],
        10.,
        false,
        1.,
        15.,
    );
    assert_eq!(
        failures,
        vec![0],
        "bounded placement must expose fallback rejection"
    );
    let square = vec![[0., 0., 0.], [4., 0., 0.], [4., 4., 0.], [0., 4., 0.]];
    assert!((rims::enclosing_square(&square) - 4.).abs() < 1e-12);
    // Diameter sqrt(32)>5 would incorrectly pass this actual square-side limit.
    assert!(matches!(
        rims::level(
            &square,
            &planar_sides(&square, [0., 0., 1.]),
            3.99,
            true,
            5.
        )
        .unwrap(),
        LevelCut::Sliver
    ));
}
#[test]
fn disconnected_reflex_clip_rejected() {
    let p = vec![
        [0., 0., 0.],
        [4., 0., 0.],
        [4., 4., 0.],
        [3., 4., 0.],
        [3., 1., 0.],
        [1., 1., 0.],
        [1., 4., 0.],
        [0., 4., 0.],
    ];
    assert_eq!(
        rims::level(&p, &planar_sides(&p, [0., 0., 1.]), 2., false, 0.1).unwrap_err(),
        "disconnected reflex level clip"
    );
}
#[test]
fn topology_adapter_rejects_noncovering_cube() {
    use vstone_core::{
        assembly,
        model::{CoreInput, Plane},
        planes::{DistanceStats, OptReport, Planes},
        seeds::Seed,
        surface::Surface,
        topology::Topology,
    };
    let input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let mut p = input.params;
    p.thickness = 0.001;
    p.min_bounding_square = 0.1;
    let planes: Vec<_> = (0..3)
        .flat_map(|axis| {
            [-1., 1.].map(move |sign| {
                let mut n = [0.; 3];
                n[axis] = sign;
                Plane { n, d: 1. }
            })
        })
        .collect();
    let mut corners = Vec::new();
    let mut triples = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                corners.push([
                    if x == 0 { -1. } else { 1. },
                    if y == 0 { -1. } else { 1. },
                    if z == 0 { -1. } else { 1. },
                ]);
                triples.push([x, 2 + y, 4 + z]);
            }
        }
    }
    let polygons: Vec<Vec<usize>> = (0..6)
        .map(|i| {
            let n = nalgebra::Vector3::from(planes[i].n);
            let axis = if n[0].abs() < 0.9 {
                nalgebra::Vector3::x()
            } else {
                nalgebra::Vector3::y()
            };
            let u = n.cross(&axis).normalize();
            let w = n.cross(&u);
            let mut fan: Vec<_> = (0..8).filter(|&k| triples[k].contains(&i)).collect();
            fan.sort_by(|&a, &b| {
                let angle = |k| {
                    let v = nalgebra::Vector3::from(corners[k]);
                    v.dot(&w).atan2(v.dot(&u))
                };
                angle(a).total_cmp(&angle(b))
            });
            fan
        })
        .collect();
    let t = Topology {
        tiles: (0..6)
            .map(|_| Seed {
                u: 0.,
                v: 0.,
                s: 0.,
                row: 0,
                auxiliary: false,
                depth: 0.5,
            })
            .collect(),
        neighbors: vec![vec![]; 6],
        corners: triples,
        boundary: vec![false; 6],
    };
    let g = Planes {
        planes,
        corners,
        polygons,
        whole_tiles: vec![true; 6],
        report: OptReport {
            limit_violation_sum_sq: 0.,
            iterations: 0,
            corner_distance: DistanceStats {
                min: 0.,
                max: 0.,
                median: 0.,
            },
            reflex_corners: 0,
            violated: vec![],
        },
    };
    let surface = Surface::from_profile(&[[10., -2.], [10., 2.]]).unwrap();
    // A closed cube is NOT a once-covering revolution band: its horizontal
    // faces project degenerately. The former synthetic adapter test silently
    // certified this unrelated shape; production now rejects it explicitly.
    let error = assembly::assemble(&t, &g, &surface, &p).unwrap_err();
    assert!(error.contains("closure:"), "{error}");
    p.rims.top = "whole".into();
    let surface = Surface::from_profile(&[[10., -2.], [10., 0.5]]).unwrap();
    assert!(assembly::assemble(&t, &g, &surface, &p).is_err());
}

#[test]
fn topology_adapter_metadata_and_cache() {
    use vstone_core::{assembly, closure, model::CoreInput, planes, sidecount, surface::Surface};
    let input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let t = sidecount::build(&surface, &input.params).unwrap();
    let g = planes::fitted(&surface, &t, &input.params).unwrap();
    assert!(g.report.violated.is_empty(), "{:?}", g.report.violated);
    let a = assembly::assemble(&t, &g, &surface, &input.params).unwrap();
    let b = assembly::assemble(&t, &g, &surface, &input.params).unwrap();
    assert_eq!(a.pieces, b.pieces);
    assert_eq!(a.pieces.len(), 108);
    assert_eq!(a.metadata.len(), a.pieces.len());
    closure::certify(&a.pieces, a.rim_heights).unwrap();
    for piece in &a.pieces {
        assert!(piece.seed_depth.is_some());
        assert!(piece.max_corner_distance.is_some());
        assert!(piece.reflex_corners.is_some());
        assert!(piece.inner.len() == piece.outer.len());
        for edge in &piece.edges {
            if let Some(j) = &edge.neighbor {
                let other = a.pieces.iter().find(|p| &p.id == j).unwrap();
                let reverse = other
                    .edges
                    .iter()
                    .find(|e| e.neighbor.as_ref() == Some(&piece.id))
                    .unwrap();
                assert_eq!(piece.outer[edge.from], other.outer[reverse.to]);
                assert_eq!(piece.inner[edge.from], other.inner[reverse.to]);
            }
        }
    }
}

#[test]
fn inner_edge_minimum_is_positive_and_independent_of_outer_min_edge() {
    let face = vstone_core::model::Plane {
        n: [0., 0., 1.],
        d: 0.,
    };
    for (width, valid) in [(0.0005, false), (0.002, true)] {
        let points = vec![[0., 0., 0.], [width, 0., 0.], [width, 2., 0.], [0., 2., 0.]];
        let result = pieces::build(&face, points.clone(), planar_sides(&points, face.n), 0.375);
        assert_eq!(result.is_ok(), valid, "width={width}");
    }
}
