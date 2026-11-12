use vstone_core::{
    model::CoreInput,
    planes::{self, Planes, SurfaceQueries},
    seeds, sidecount,
    surface::Surface,
};
fn input() -> CoreInput {
    serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .map(|mut i: CoreInput| {
        i.params.depth = 1.5;
        i
    })
    .unwrap()
}
fn run(i: &CoreInput) -> Planes {
    let s = Surface::from_profile(&i.surface.profile).unwrap();
    let t = sidecount::build(&s, &i.params).unwrap();
    let result = planes::optimize(&s, &t, &i.params, &mut seeds::rng(i.params.seed));
    let geometry = match result {
        Ok(v) => v,
        Err(e) => *e.best.expect("must preserve best report"),
    };
    for (seed, plane) in t.tiles.iter().zip(&geometry.planes) {
        assert!((0.0..=i.params.depth).contains(&seed.depth));
        let point = s.offset([seed.u, seed.v], seed.depth);
        assert!(
            (plane.n.iter().zip(point).map(|(a, b)| a * b).sum::<f64>() - plane.d).abs() < 1e-9
        );
    }
    assert!(geometry.report.corner_distance.min.is_finite());
    assert!(geometry.report.corner_distance.max.is_finite());
    geometry
}
#[test]
fn enclosing_square_rotates_and_handles_reflex_points() {
    let q = [[0., 0.], [4., 0.], [4., 4.], [2., 2.], [0., 4.]];
    assert!((planes::bounding_square(&q) - 4.).abs() < 1e-10);
    let a = 0.39f64;
    let rotated: Vec<_> = q
        .iter()
        .map(|p| {
            [
                p[0] * a.cos() - p[1] * a.sin(),
                p[0] * a.sin() + p[1] * a.cos(),
            ]
        })
        .collect();
    assert!((planes::bounding_square(&rotated) - 4.).abs() < 1e-9);
}
#[test]
fn symmetric_seed_plane_band_and_deterministic_planes() {
    let i = input();
    let a = run(&i);
    let b = run(&i);
    assert_eq!(a, b);
}
// Strict acceptance tests are ignored by default: parent runs in release, records
// best reports on failure and stops for Jerry rather than weakening assertions.
#[test]
#[ignore = "release A05 acceptance experiment"]
fn symmetric_cylinder_all_limits() {
    assert!(run(&input()).report.violated.is_empty());
}
#[test]
#[ignore = "release A05 acceptance experiment: 20 solves"]
fn organic_half_and_full_seeds_one_to_ten() {
    for randomness in [0.5, 1.] {
        for seed in 1..=10 {
            let mut i = input();
            i.params.randomness = randomness;
            i.params.seed = seed;
            let g = run(&i);
            assert!(
                g.report.violated.is_empty(),
                "randomness {randomness} seed {seed}: {:?}",
                g.report
            );
        }
    }
}
#[test]
#[ignore = "release A05 acceptance experiment"]
fn shallow_and_deep_clear_reports() {
    for depth in [1., 2.] {
        let mut i = input();
        i.params.depth = depth;
        let g = run(&i);
        assert!(g.report.violated.iter().all(|(_, name)| !name.is_empty()));
        println!("depth {depth}: {:?}", g.report);
    }
}
#[test]
#[ignore = "release A05 acceptance experiment"]
fn cone_and_vase_all_limits() {
    for name in ["cone_9_to_6", "vase_bulge"] {
        let mut i = input();
        i.params.allowed_sides = vec![5, 6, 7];
        i.surface.profile = serde_json::from_str(
            &std::fs::read_to_string(format!(
                "{}/../tests/fixtures/profiles/{name}.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
        )
        .unwrap();
        let g = run(&i);
        assert!(g.report.violated.is_empty(), "{name}: {:?}", g.report);
    }
}

#[test]
fn production_polishing_never_worse_than_native_fit() {
    let i = input();
    let s = Surface::from_profile(&i.surface.profile).unwrap();
    let t = sidecount::build(&s, &i.params).unwrap();
    let fit = planes::fitted(&s, &t, &i.params).unwrap();
    let result = planes::optimize(&s, &t, &i.params, &mut seeds::rng(i.params.seed));
    let polished = match result {
        Ok(g) => g,
        Err(e) => *e.best.unwrap(),
    };
    assert!(polished.report.limit_violation_sum_sq <= fit.report.limit_violation_sum_sq);
    assert!(polished.report.violated.len() <= fit.report.violated.len());
    assert!(polished
        .report
        .violated
        .iter()
        .all(|v| fit.report.violated.contains(v)));
    if fit.report.violated.is_empty() {
        assert_eq!(
            serde_json::to_vec(&fit).unwrap(),
            serde_json::to_vec(&polished).unwrap()
        );
    }
}

/// Frozen identical-input SciPy results guard the solver, independently of C3.
#[test]
#[ignore = "release reference parity gate; two dense SVD fits"]
fn identical_input_reference_solver_parity() {
    for fixture in [
        include_str!("../../tests/fixtures/c4-reference-nine.json"),
        include_str!("../../tests/fixtures/c4-reference-legacy-eight.json"),
    ] {
        let reference: planes::ReferenceInput = serde_json::from_str(fixture).unwrap();
        let value: serde_json::Value = serde_json::from_str(fixture).unwrap();
        let i = input();
        let surf = Surface::from_profile(&i.surface.profile).unwrap();
        let native = planes::reference_stage(&surf, &i.params, &reference, "native-fit").unwrap();
        let target_cost: f64 = native
            .corners
            .iter()
            .zip(&reference.targets)
            .map(|(a, b)| (0..3).map(|j| (a[j] - b[j]).powi(2)).sum::<f64>())
            .sum();
        let tilt: f64 = native
            .planes
            .iter()
            .enumerate()
            .map(|(k, p)| {
                let dot =
                    |a: [f64; 3], b: [f64; 3]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
                let radial = dot(p.n, reference.radial[k]);
                (dot(p.n, reference.etheta[k]) / radial).powi(2)
                    + (dot(p.n, reference.axis_b[k]) / radial).powi(2)
            })
            .sum();
        let objective = target_cost + 0.0025 * tilt;
        let expected = value["expected_objective"].as_f64().unwrap();
        let error = native
            .corners
            .iter()
            .zip(&reference.reference_corners)
            .flat_map(|(a, b)| (0..3).map(move |j| (a[j] - b[j]).abs()))
            .fold(0., f64::max);
        // Different SVD backends need not be bitwise equal; these tolerances
        // expose a different basin (legacy LM's 643 vs 520), not roundoff.
        assert!(
            (objective - expected).abs() < 0.001,
            "objective {objective} vs {expected}"
        );
        assert!(error < 0.02, "corner max coordinate error {error}");
    }
}
#[test]
#[ignore = "release fitted-cylinder reflex polish gate"]
fn fitted_cylinder_polish_removes_reflex_excess_without_new_failures() {
    let mut i = input();
    i.params.target_size = 6.5;
    i.params.randomness = 0.;
    i.params.seed = 1;
    let surf = Surface::from_profile(&i.surface.profile).unwrap();
    let t = sidecount::build(&surf, &i.params).unwrap();
    let fit = planes::fitted(&surf, &t, &i.params).unwrap();
    let result = planes::optimize(&surf, &t, &i.params, &mut seeds::rng(1));
    let polished = match result {
        Ok(g) => g,
        Err(e) => *e.best.unwrap(),
    };
    assert_eq!(polished.whole_tiles, fit.whole_tiles);
    assert!(polished.report.limit_violation_sum_sq <= fit.report.limit_violation_sum_sq);
    assert!(polished
        .report
        .violated
        .iter()
        .all(|v| fit.report.violated.contains(v)));
    assert!(
        !polished
            .report
            .violated
            .iter()
            .any(|(_, v)| v == "max_corner_angle_deg"),
        "{:?}",
        polished.report
    );
}

/// Exercises stochastic repair AND plane fitting, not merely initial placement.
#[test]
#[ignore = "release end-to-end cross-seed regression: 30 solves"]
fn symmetric_geometry_is_seed_independent_end_to_end() {
    for profile in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        let mut i = input();
        i.params.randomness = 0.;
        i.params.target_size = 6.5;
        i.params.size_variation = 0.15;
        if profile != "cylinder" {
            i.params.allowed_sides = vec![5, 6, 7];
            i.surface.profile = serde_json::from_str(
                &std::fs::read_to_string(format!(
                    "{}/../tests/fixtures/profiles/{profile}.json",
                    env!("CARGO_MANIFEST_DIR")
                ))
                .unwrap(),
            )
            .unwrap();
        }
        let mut expected = None;
        for seed in 1..=10 {
            i.params.seed = seed;
            let geometry = run(&i);
            let bytes = serde_json::to_vec(&geometry).unwrap();
            if let Some(ref previous) = expected {
                assert_eq!(previous, &bytes, "{profile} seed {seed}");
            } else {
                expected = Some(bytes);
            }
        }
    }
}

/// Release diagnostic gate: same start/fit/polish, no second fit hidden in timings.
#[test]
#[ignore = "three full reference fits; parent release profiling gate"]
fn single_fit_diagnostic_matches_production_stages() {
    let mut i = input();
    i.params.target_size = 6.5;
    i.params.randomness = 0.;
    i.params.seed = 1;
    i.params.max_corner_angle_deg = 230.;
    let s = Surface::from_profile(&i.surface.profile).unwrap();
    let t = sidecount::build(&s, &i.params).unwrap();
    let diagnostic = planes::fit_and_polish(&s, &t, &i.params).unwrap();
    let fit = planes::fitted(&s, &t, &i.params).unwrap();
    let result = planes::optimize(&s, &t, &i.params, &mut seeds::rng(i.params.seed));
    assert_eq!(diagnostic.fit, fit);
    // Includes Ok/Err, error text, best planes, frozen whole tiles and all report fields.
    assert_eq!(
        serde_json::to_vec(&diagnostic.result).unwrap(),
        serde_json::to_vec(&result).unwrap()
    );
    assert_eq!(
        diagnostic.fit_profile.residual_calls,
        diagnostic.fit_profile.nfev + diagnostic.fit_profile.jacobians * 2 * t.tiles.len()
    );
    assert!(diagnostic.fit_profile.jacobians > 0);
    assert!(diagnostic.fit_profile.svds <= diagnostic.fit_profile.jacobians);
}

// Frozen formerly-passing matrix cases: no changes to caps, starts, budgets or
// acceptance. Keep separate tests so the parent sees all three outcomes.
fn assert_historical_cylinder_pass(randomness: f64, seed: u64) {
    let mut i = input();
    i.params.target_size = 6.5;
    i.params.size_variation = 0.15;
    i.params.max_corner_angle_deg = 230.;
    i.params.randomness = randomness;
    i.params.seed = seed;
    let surface = Surface::from_profile(&i.surface.profile).unwrap();
    let topology = sidecount::build_with_jitter(&surface, &i.params, 0.1).unwrap();
    let diagnostic = planes::fit_and_polish(&surface, &topology, &i.params).unwrap();
    assert!(diagnostic.result.is_ok(), "{diagnostic:?}");
    assert!(diagnostic.result.unwrap().report.violated.is_empty());
}
#[test]
#[ignore = "release historical compact-SVD acceptance regression"]
fn historical_cylinder_half_seed7() {
    assert_historical_cylinder_pass(0.5, 7);
}
#[test]
#[ignore = "release historical compact-SVD acceptance regression"]
fn historical_cylinder_full_seed4() {
    assert_historical_cylinder_pass(1., 4);
}
#[test]
#[ignore = "release historical compact-SVD acceptance regression"]
fn historical_cylinder_full_seed8() {
    assert_historical_cylinder_pass(1., 8);
}
