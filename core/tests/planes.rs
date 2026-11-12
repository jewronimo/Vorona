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
    for depth in [1., 3.] {
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
