use std::collections::BTreeMap;
use vstone_core::{
    model::{CoreInput, Pattern, ReliefBase},
    rectified,
    surface::Surface,
    topology::Topology,
};
fn input() -> CoreInput {
    let mut i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    i.params.pattern = Pattern::VoronoiTriangles;
    // Switching an already deserialized A05 fixture preserves its explicit 70°
    // minimum. A06 §9 uses 30°; requiring every triangle angle >= 70° is
    // infeasible. Override only this pattern-specific setting, not A05 defaults.
    i.params.min_corner_angle_deg = 30.0;
    i.params.min_triangle_edge = 2.0;
    i.params.max_polygon_edge = None;
    i.params.depth = 1.0;
    i.params.relief_base = serde_json::from_str("\"auto\"").unwrap();
    i
}
fn incidence(t: &rectified::Rectified) {
    let n = t.polygons.len();
    assert_eq!(t.triangles.len(), t.delaunay.corners.len());
    let edge_count: usize = t.delaunay.neighbors.iter().map(Vec::len).sum::<usize>() / 2;
    assert_eq!(t.corners.len(), edge_count);
    // The periodic strip is a cylinder, Euler characteristic zero.
    assert_eq!(n + t.triangles.len(), edge_count);
    let mut uses = BTreeMap::<usize, Vec<usize>>::new();
    for (p, cs) in t.polygons.iter().enumerate() {
        assert_eq!(cs.len(), t.delaunay.degree(p));
        for &c in cs {
            uses.entry(c).or_default().push(p);
        }
    }
    for (p, cs) in t.triangles.iter().enumerate() {
        for &c in cs {
            uses.entry(c).or_default().push(n + p);
        }
    }
    for (c, corner) in t.corners.iter().enumerate() {
        assert_eq!(uses[&c], corner.pieces);
        assert_eq!(corner.pieces.len(), if corner.open { 3 } else { 4 });
        if corner.open {
            assert!(corner.seeds.iter().all(|&s| t.delaunay.tiles[s].auxiliary));
        }
    }
}
#[test]
fn counts_four_valence_and_determinism_across_profiles() {
    for profile in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        let mut i = input();
        i.surface.profile = serde_json::from_str(
            &std::fs::read_to_string(format!(
                "{}/../tests/fixtures/profiles/{profile}.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
        )
        .unwrap();
        let s = Surface::from_profile(&i.surface.profile)
            .unwrap()
            .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
        for r in [0.0, 0.5, 1.0] {
            for seed in 1..=3 {
                i.params.randomness = r;
                i.params.seed = seed;
                let t = rectified::build(&s, &i.params).unwrap();
                incidence(&t);
                if r == 0.0 {
                    let mut symmetric = i.params.clone();
                    symmetric.seed = 999;
                    assert_eq!(t, rectified::build(&s, &symmetric).unwrap());
                }
                for (row, data) in t.rows.iter().enumerate() {
                    let auxiliary = data.s < 0.0 || data.s > s.length();
                    let sites: Vec<_> = t.delaunay.tiles.iter().filter(|x| x.row == row).collect();
                    for site in &sites {
                        assert_eq!(site.auxiliary, auxiliary);
                        if auxiliary || data.s == 0.0 {
                            assert_eq!(site.s, data.s);
                        } else {
                            assert!(site.s > 0.0 && site.s < s.length());
                        }
                    }
                    if r == 0.0 {
                        for w in sites.windows(2) {
                            assert_eq!(w[0].depth, w[1].depth);
                            assert_eq!(w[0].s, w[1].s);
                            assert!(
                                (w[1].u - w[0].u - std::f64::consts::TAU / data.count as f64).abs()
                                    < 1e-12
                            );
                        }
                    }
                }
                assert_eq!(
                    serde_json::to_vec(&t).unwrap(),
                    serde_json::to_vec(&rectified::build(&s, &i.params).unwrap()).unwrap()
                );
                let count = t.rows[0].count;
                for row in &t.rows {
                    assert_eq!(row.count, count);
                    assert!(row.height > 0.0);
                    if (0.0..=s.length()).contains(&row.s) {
                        assert!(row.triangle_base >= i.params.min_triangle_edge - 1e-9);
                        assert!(
                            row.triangle_base
                                <= i.params.max_polygon_edge.unwrap_or(f64::INFINITY) + 1e-9
                        );
                        assert!(row.triangle_leg >= i.params.min_triangle_edge - 1e-9);
                        assert!(
                            row.triangle_leg
                                <= i.params.max_polygon_edge.unwrap_or(f64::INFINITY) + 1e-9
                        );
                    }
                }
                for (k, site) in t.delaunay.tiles.iter().enumerate() {
                    assert!(
                        site.depth >= t.relief_bases[k]
                            && site.depth <= t.relief_bases[k] + i.params.depth
                    );
                    if r == 0.0 {
                        assert_eq!(site.depth, t.relief_bases[k] + (site.row % 3) as f64 / 2.0);
                    }
                }
            }
        }
    }
}
#[test]
fn cylinder_curvature_base_smooth_numeric_and_legacy_knobs() {
    let mut i = input();
    let s = Surface::from_profile(&i.surface.profile)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    assert_eq!(s.principal_curvatures(30.0), [1.0 / 9.0, 0.0]);
    i.params.randomness = 0.0;
    i.params.depth = 0.0;
    let a = rectified::build(&s, &i.params).unwrap();
    for b in &a.relief_bases {
        assert!((*b - a.seed_spacing.powi(2) / 144.0).abs() < 1e-12);
    }
    assert!(a
        .delaunay
        .tiles
        .iter()
        .zip(&a.relief_bases)
        .all(|(t, b)| t.depth == *b));
    i.params.target_size = 100.0;
    i.params.allowed_sides = vec![3];
    i.params.size_variation = 0.05;
    assert_eq!(a, rectified::build(&s, &i.params).unwrap());
    i.params.relief_base = ReliefBase::Inches(0.25);
    let t = rectified::build(&s, &i.params).unwrap();
    assert_eq!(t.relief_base_stats, [0.25; 3]);
    i.params.min_triangle_edge = 0.0;
    assert!(rectified::build(&s, &i.params).is_err());
}
#[test]
fn periodic_cut_does_not_change_rectified_incidence() {
    let mut i = input();
    for profile in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        i.surface.profile = serde_json::from_str(
            &std::fs::read_to_string(format!(
                "{}/../tests/fixtures/profiles/{profile}.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
        )
        .unwrap();
        for r in [0.0, 0.5, 1.0] {
            i.params.randomness = r;
            let s = Surface::from_profile(&i.surface.profile)
                .unwrap()
                .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
            let a = rectified::build(&s, &i.params).unwrap();
            for shift in [0.731, 0.123456789, std::f64::consts::PI, 5.9] {
                let mut tiles = a.delaunay.tiles.clone();
                for t in &mut tiles {
                    t.u = (t.u + shift).rem_euclid(std::f64::consts::TAU);
                }
                let d = Topology::build_a06(tiles).unwrap();
                let b =
                    rectified::rectify(d, a.seed_spacing, a.rows.clone(), a.relief_bases.clone())
                        .unwrap();
                assert_eq!(a.corners, b.corners);
                assert_eq!(a.triangles, b.triangles);
                for (a, b) in a.polygons.iter().zip(&b.polygons) {
                    assert_eq!(a.len(), b.len());
                    assert!(
                        (0..a.len()).any(|k| (0..a.len()).all(|j| a[j] == b[(j + k) % b.len()]))
                    );
                }
            }
        }
    }
}

#[test]
fn signed_curvature_and_row_symmetry() {
    let points: Vec<_> = (0..=80)
        .map(|j| {
            let a = (j as f64).to_radians();
            [10.0 * a.cos(), 10.0 * a.sin()]
        })
        .collect();
    let sphere = Surface::from_profile(&points)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    for k in sphere.principal_curvatures(5.0) {
        assert!((k - 0.1).abs() < 0.003);
    }
    let waist = Surface::from_profile(&[[9.0, 0.0], [5.0, 20.0], [9.0, 40.0]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let k = waist.principal_curvatures(waist.length() / 2.0);
    assert!(k[0] > 0.0 && k[1] < 0.0);
    let mut i = input();
    i.params.randomness = 0.0;
    let s = Surface::from_profile(&i.surface.profile)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let t = rectified::build(&s, &i.params).unwrap();
    for (row, r) in t.rows.iter().enumerate() {
        let sites: Vec<_> = t.delaunay.tiles.iter().filter(|x| x.row == row).collect();
        for w in sites.windows(2) {
            assert_eq!(w[0].depth, w[1].depth);
            assert_eq!(w[0].s, w[1].s);
            assert!((w[1].u - w[0].u - std::f64::consts::TAU / r.count as f64).abs() < 1e-12);
        }
    }
}

#[test]
fn horizontal_tangent_between_vertices_and_clamped_support_curvature() {
    // Circle arc has a horizontal tangent at pi/2, deliberately between samples.
    let points: Vec<_> = (0..=80)
        .map(|j| {
            let a = 0.5 + j as f64 * 0.0251;
            [20.0 - 10.0 * a.cos(), 10.0 * a.sin()]
        })
        .collect();
    let s = Surface::from_profile(&points)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    for j in 0..=2000 {
        let k = s.principal_curvatures(s.length() * j as f64 / 2000.0);
        assert!(k.iter().all(|x| x.is_finite()));
        assert!((k[1] + 0.1).abs() < 0.004, "{j}: {k:?}");
    }
    assert_eq!(s.principal_curvatures(-0.05), s.principal_curvatures(0.0));
    assert_eq!(
        s.principal_curvatures(s.length() + 0.05),
        s.principal_curvatures(s.length())
    );
}

#[test]
fn invalid_settings_are_rejected() {
    let i = input();
    let s = Surface::from_profile(&i.surface.profile)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let mut bad = Vec::new();
    let mut p = i.params.clone();
    p.pattern = Pattern::Voronoi;
    bad.push(p);
    let mut p = i.params.clone();
    p.depth = -1.0;
    bad.push(p);
    let mut p = i.params.clone();
    p.randomness = f64::NAN;
    bad.push(p);
    let mut p = i.params.clone();
    p.rims.top = "bad".into();
    bad.push(p);
    for p in bad {
        assert!(rectified::build(&s, &p).is_err());
    }
}

#[test]
fn section9_defaults_and_explicit_zero() {
    let mut v: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    v["params"]["pattern"] = serde_json::json!("voronoi_triangles");
    for key in [
        "depth",
        "randomness",
        "min_triangle_edge",
        "max_polygon_edge",
        "count_step",
    ] {
        v["params"].as_object_mut().unwrap().remove(key);
    }
    let i: CoreInput = serde_json::from_value(v.clone()).unwrap();
    assert!((i.params.depth - 0.25).abs() < 1e-15);
    assert_eq!(i.params.randomness, 0.);
    assert_eq!(i.params.max_polygon_edge, None);
    assert_eq!(i.params.min_triangle_edge, 2.0);
    // The relief default is fixed inches, independent of the stone-size knob.
    v["params"]["max_polygon_edge"] = serde_json::json!(4.);
    let resized: CoreInput = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(resized.params.depth, 0.25);
    v["params"]["depth"] = serde_json::json!(0);
    let i: CoreInput = serde_json::from_value(v).unwrap();
    assert_eq!(i.params.depth, 0.);
}

#[test]
fn section9_empty_triangle_interval_is_infeasible() {
    let i = input();
    let s = Surface::from_profile(&i.surface.profile)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let mut p = i.params;
    p.min_triangle_edge = 4.;
    p.max_polygon_edge = Some(3.);
    let error = rectified::row_layout::choose(&s, &p).unwrap_err();
    assert!(error.contains("max_polygon_edge") && error.contains("s="));
}
