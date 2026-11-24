use vstone_core::{model::CoreInput, rectified::row_layout, surface::Surface};
fn params() -> vstone_core::model::Params {
    serde_json::from_str::<CoreInput>(include_str!(
        "../../tests/fixtures/core_input_a06_cylinder.json"
    ))
    .unwrap()
    .params
}
#[test]
fn owner_four_layouts_and_report() {
    let p = params();
    for (name, profile, expected) in [
        ("cylinder", vec![[9., 0.], [9., 60.]], 12),
        ("cone", vec![[9., 0.], [6., 60.]], 8),
        ("vase", vec![[6., 0.], [10., 30.], [6., 60.]], 8),
        ("flare", vec![[6., 0.], [15., 30.]], 8),
    ] {
        let s = Surface::from_profile(&profile)
            .unwrap()
            .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
        let rows = row_layout::choose(&s, &p).unwrap();
        let h = rows[0].height;
        assert!(rows.iter().all(|r| r.count == expected && r.height == h));
        let rmin = profile.iter().map(|q| q[0]).fold(f64::INFINITY, f64::min);
        let h0 = 3_f64.sqrt() / 2. * std::f64::consts::TAU * rmin / expected as f64;
        assert!((h - h0).abs() < 1e-12);
        let mut report = Vec::new();
        let mut positions: Vec<f64> = rows
            .iter()
            .filter(|r| r.s >= 0. && r.s <= s.length())
            .map(|r| r.s)
            .collect();
        positions.extend([0., s.length()]);
        positions.extend(
            s.layout_knots()
                .into_iter()
                .filter(|&at| (s.r(at) - 10.).abs() < 1e-12),
        );
        positions.sort_by(f64::total_cmp);
        positions.dedup();
        for position in positions {
            let base = std::f64::consts::PI * s.r(position) / expected as f64;
            let leg = 0.5 * (base * base + h * h).sqrt();
            let apex = (2. * (base / h).atan()).to_degrees();
            assert!(base.min(leg) >= 2. - 1e-9);
            assert!(apex.min((180. - apex) / 2.) >= 30. - 1e-9);
            report.push(serde_json::json!({"s":position,"base":base,"leg":leg,"apex":apex,"base_angle":(180.-apex)/2.}));
        }
        println!(
            "{}",
            serde_json::json!({"profile":name,"n":expected,"h":h,"h_equilateral":h0,"rows":report})
        );
    }
}
#[test]
fn hard_caps_do_not_trigger_count_fallback() {
    let mut p = params();
    let s = Surface::from_profile(&[[6., 0.], [15., 30.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    p.max_polygon_edge = Some(3.);
    let e = row_layout::choose(&s, &p).unwrap_err();
    assert!(
        e.contains("max_polygon_edge") && e.contains("s=") && e.contains("n=8"),
        "{e}"
    );
    p.max_polygon_edge = None;
    p.blank_usable = 8.;
    // Nominal pitch cannot certify (or reject) the actual fitted blank.
    assert!(row_layout::choose(&s, &p).is_ok());
}
#[test]
fn wider_row_raises_height_only_to_angle_limit() {
    let mut p = params();
    p.blank_usable = 100.;
    let s = Surface::from_profile(&[[6., 0.], [21., 30.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let rows = row_layout::choose(&s, &p).unwrap();
    let expected = std::f64::consts::PI * 21. / 8. * 30_f64.to_radians().tan();
    assert!((rows[0].height - expected).abs() < 1e-12);
    assert!(rows.iter().all(|r| r.count == 8));
    p.max_polygon_edge = Some(0.);
    assert!(row_layout::choose(&s, &p).is_err());
}
#[test]
fn fractional_settings_validation_and_zero_headroom() {
    let mut p = params();
    let s = Surface::from_profile(&[[9., 0.], [9., 60.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    p.layout_headroom = 0.;
    assert_eq!(row_layout::choose(&s, &p).unwrap()[0].count, 14);
    p.edge_shortfall = 0.10;
    assert_eq!(row_layout::choose(&s, &p).unwrap()[0].count, 14);
    for bad in [-0.01, 0.5, f64::NAN, f64::INFINITY] {
        p.edge_shortfall = bad;
        assert!(row_layout::choose(&s, &p).is_err());
    }
    p.edge_shortfall = 0.;
    for bad in [-0.01, f64::NAN, f64::INFINITY] {
        p.layout_headroom = bad;
        assert!(row_layout::choose(&s, &p).is_err());
    }
}

#[test]
fn proportional_pitch_tracks_radius_without_count_changes() {
    use vstone_core::model::RowHeight;
    let mut p = params();
    p.row_height = RowHeight::Proportional;
    // Sizing-only test; do not claim the flare fits production blanks.
    p.blank_usable = 100.;
    let s = Surface::from_profile(&[[6., 0.], [15., 30.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let rows = row_layout::choose(&s, &p).unwrap();
    for row in &rows {
        assert_eq!(row.count, 8);
        assert!(row.height > 0.);
    }
    for pair in rows.windows(2) {
        assert!((pair[1].s - pair[0].s - pair[0].height).abs() < 1e-12);
    }
    p.row_height = RowHeight::Constant;
    let c = row_layout::choose(&s, &p).unwrap();
    assert!(rows.len() < c.len());
}
#[test]
fn row_mode_defaults_and_invalid_values() {
    use vstone_core::model::RowHeight;
    assert_eq!(params().row_height, RowHeight::Constant);
    let mut wire: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_a06_cylinder.json"
    ))
    .unwrap();
    wire["params"]["row_height"] = "other".into();
    assert!(serde_json::from_value::<CoreInput>(wire).is_err());
}

#[test]
fn integrated_pitch_phase_gap_and_flare_angles() {
    use vstone_core::model::RowHeight;
    let mut p = params();
    p.blank_usable = 100.;
    p.row_height = RowHeight::Proportional;
    // Check both slope signs and a profile-knot crossing.
    for profile in [
        vec![[6., 0.], [15., 30.]],
        vec![[15., 0.], [6., 30.]],
        vec![[6., 0.], [10., 15.], [6., 30.]],
    ] {
        let s = Surface::from_profile(&profile)
            .unwrap()
            .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
        let h0 = 3_f64.sqrt() / 2. * std::f64::consts::TAU * 6. / 8.;
        for phase in [0., 0.125, 0.5, 0.9375, 1. - f64::EPSILON] {
            let rows = row_layout::choose_with_phase(&s, &p, phase).unwrap();
            for (i, row) in rows.iter().enumerate() {
                let t = if row.s < 0. {
                    row.s / (h0 * s.r(0.) / 6.)
                } else if row.s > s.length() {
                    s.v_of_s(s.length()) * 6. / h0
                        + (row.s - s.length()) / (h0 * s.r(s.length()) / 6.)
                } else {
                    s.v_of_s(row.s) * 6. / h0
                };
                assert!((t - (i as f64 - 2. + phase)).abs() < 1e-9);
            }
            for pair in rows.windows(2) {
                let gap = pair[1].s - pair[0].s;
                assert!((gap - pair[0].height).abs() < 1e-9);
                if profile.len() == 2 && pair[0].s >= 0. && pair[1].s <= s.length() {
                    let width = std::f64::consts::TAU * s.r((pair[0].s + pair[1].s) / 2.) / 8.;
                    let apex = (2. * (width / (2. * gap)).atan()).to_degrees();
                    assert!((apex - 60.).abs() <= 0.5, "apex {apex}");
                }
            }
            for pair in rows
                .windows(2)
                .filter(|pair| pair[1].s <= 0. || pair[0].s >= s.length())
            {
                let endpoint = if pair[1].s <= 0. { 0. } else { s.length() };
                assert!((pair[0].height - h0 * s.r(endpoint) / 6.).abs() < 1e-9);
            }
        }
    }
}
#[test]
fn cylinder_modes_are_byte_identical_at_every_phase() {
    use vstone_core::model::RowHeight;
    let mut p = params();
    let s = Surface::from_profile(&[[9., 0.], [9., 60.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    for phase in (0..128).map(|i| i as f64 / 128.).chain([1. - f64::EPSILON]) {
        p.row_height = RowHeight::Constant;
        let constant = row_layout::choose_with_phase(&s, &p, phase).unwrap();
        p.row_height = RowHeight::Proportional;
        let proportional = row_layout::choose_with_phase(&s, &p, phase).unwrap();
        assert_eq!(
            serde_json::to_vec(&constant).unwrap(),
            serde_json::to_vec(&proportional).unwrap()
        );
        for pair in proportional.windows(2) {
            assert!((pair[1].s - pair[0].s - pair[0].height).abs() < 1e-9);
        }
    }
    for phase in [-0.01, 1., f64::NAN, f64::INFINITY] {
        assert!(row_layout::choose_with_phase(&s, &p, phase).is_err());
    }
}

#[test]
fn cylinder_rectified_modes_are_identical_for_search_phases() {
    use vstone_core::{model::RowHeight, rectified};
    let mut p = params();
    let s = Surface::from_profile(&[[9., 0.], [9., 60.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    for phase in (0..16).map(|i| i as f64 / 16.) {
        p.row_height = RowHeight::Constant;
        let constant = rectified::build_with_phase(&s, &p, phase).unwrap();
        p.row_height = RowHeight::Proportional;
        let proportional = rectified::build_with_phase(&s, &p, phase).unwrap();
        assert_eq!(constant, proportional);
    }
}

#[test]
fn proportional_strip_report_uses_midpoint_width() {
    use vstone_core::{model::RowHeight, rectified};
    let mut p = params();
    p.row_height = RowHeight::Proportional;
    p.blank_usable = 100.;
    let surface = Surface::from_profile(&[[6., 0.], [15., 30.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let topology = rectified::build(&surface, &p).unwrap();
    for row in topology
        .rows
        .iter()
        .filter(|r| r.s >= 0. && r.s + r.height <= surface.length())
    {
        let width = std::f64::consts::TAU * surface.r(row.s + row.height / 2.) / row.count as f64;
        let apex = (2. * (width / (2. * row.height)).atan()).to_degrees();
        assert!((row.triangle_base - width / 2.).abs() < 1e-12);
        assert!((row.triangle_apex_angle_deg - apex).abs() < 1e-12);
        assert!((apex - 60.).abs() < 0.5);
        assert!(
            (row.triangle_apex_angle_deg + 2. * row.triangle_base_angle_deg - 180.).abs() < 1e-12
        );
    }
}

#[test]
fn smooth_flare_fits_actual_blanks_in_both_row_modes() {
    let surface = Surface::from_profile(&[[6., 0.], [15., 30.]])
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    for mode in [
        vstone_core::model::RowHeight::Constant,
        vstone_core::model::RowHeight::Proportional,
    ] {
        let mut p = params();
        p.depth = 0.;
        p.row_height = mode;
        let assembly = vstone_core::rectified_assembly::solve_assembly(&surface, &p).unwrap();
        assert!(!assembly.pieces.is_empty());
        assert!(assembly.closure.is_ok());
        assert!(assembly.violations.is_empty(), "{:?}", assembly.violations);
        assert!(assembly.solid_errors.is_empty());
        assert!(assembly
            .symmetry_rows
            .iter()
            .all(|r| !r.required || r.passed == Some(true)));
        assert!(assembly
            .pieces
            .iter()
            .all(|piece| piece.bounding_square <= p.blank_usable + 1e-7));
    }
}
