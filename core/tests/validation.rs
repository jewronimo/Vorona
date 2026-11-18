use vstone_core::{
    closure,
    model::{CoreInput, CoreOutput},
    numbering,
    rectified_assembly::Assembly,
    surface::Surface,
    validate,
};
fn fixture() -> (Assembly, CoreInput) {
    let o: CoreOutput =
        serde_json::from_str(include_str!("../../tests/fixtures/core_output_ring12.json")).unwrap();
    let p: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_a06_cylinder.json"
    ))
    .unwrap();
    let c = closure::certify(&o.pieces, [0., 6.]);
    (
        Assembly {
            pieces: o.pieces,
            rim_heights: [0., 6.],
            pattern_phase: 0.,
            symmetry_rows: vec![],
            closure: c,
            back_gap: [0.; 3],
            per_piece_back_gap: vec![],
            violations: vec![],
            solid_errors: vec![],
            c6_pending: false,
        },
        p,
    )
}
#[test]
fn ring_checks_and_numbering() {
    let (mut a, p) = fixture();
    assert!(
        validate::triangles(&a, &p.params).passed,
        "{:?}",
        validate::triangles(&a, &p.params)
    );
    let s = Surface::from_profile(&p.surface.profile).unwrap();
    a.pieces.reverse();
    numbering::number(&mut a.pieces, &s, 6.5);
    assert_eq!(a.pieces[0].id, "P001");
    assert!(a.pieces[0].plane.n[0] > 0.99);
    for q in &a.pieces {
        for e in &q.edges {
            if let Some(n) = &e.neighbor {
                assert!(a
                    .pieces
                    .iter()
                    .any(|o| &o.id == n
                        && o.edges.iter().any(|f| f.neighbor.as_ref() == Some(&q.id))));
            }
        }
        let area: f64 = (0..q.map_uv.len())
            .map(|i| {
                let x = q.map_uv[i];
                let y = q.map_uv[(i + 1) % q.map_uv.len()];
                x[0] * y[1] - y[0] * x[1]
            })
            .sum();
        assert!(area > 0.);
    }
}
#[test]
fn each_check_names_its_broken_piece() {
    for index in 0..8 {
        let (mut a, p) = fixture();
        match index {
            0 => a.pieces[0].sides = 2,
            1 => a.pieces[0].bounding_square = 100.,
            2 => a.violations.push(("P001".into(), "min_edge".into(), 1.)),
            3 => a.pieces[0].corner_angles_deg[0] = 1.,
            4 => a
                .violations
                .push(("P001".into(), "bevel_cap_deg".into(), 1.)),
            5 => a.pieces[0].plane.d += 0.01,
            6 => a
                .violations
                .push(("P001".into(), "corner_band_high".into(), 1.)),
            7 => a.pieces[0].edges[0].neighbor = Some("missing".into()),
            _ => unreachable!(),
        }
        let v = validate::triangles(&a, &p.params);
        let failed: Vec<_> = v.checks.iter().filter(|c| !c.passed).collect();
        assert_eq!(failed.len(), 1, "{v:?}");
        assert_eq!(failed[0].id, format!("R-VA-{}", index + 1));
        assert_eq!(
            failed[0].failures,
            if index == 7 {
                vec!["P001", "P012"]
            } else {
                vec!["P001"]
            }
        );
    }
}
