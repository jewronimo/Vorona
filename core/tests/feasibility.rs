use sha2::{Digest, Sha256};
use vstone_core::{feasibility, model::CoreInput, surface::Surface};
fn input(name: &str) -> CoreInput {
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.surface.profile = serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/../tests/fixtures/profiles/{name}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap();
    input.params.target_size = 6.0;
    // Legacy defect/variation tests isolate those constraints from the dip gate.
    input.params.surface_tolerance = 2.0;
    input
}
fn check(input: &CoreInput) -> vstone_core::model::Feasibility {
    feasibility::check(
        &Surface::from_profile(&input.surface.profile).unwrap(),
        &input.params,
    )
}
fn message(f: &vstone_core::model::Feasibility, p: &vstone_core::model::Params) -> String {
    feasibility::message(
        &Surface::from_profile(&input("cylinder").surface.profile).unwrap(),
        f,
        p,
    )
}
#[test]
fn cylinder_even_wrap() {
    let f = check(&input("cylinder"));
    assert!(f.feasible);
    assert_eq!(f.columns.len(), 1);
    assert!((8..=12).contains(&f.columns[0]));
    assert_eq!(f.columns[0] % 2, 0);
    assert!(f.suggestions.is_empty());
}
#[test]
fn cone_dislocations_and_wide() {
    let mut i = input("cone_9_to_6");
    let f = check(&i);
    assert!(!f.feasible);
    assert!(f.column_changes > 0);
    assert_eq!(f.suggestions[0], [5, 6, 7]);
    i.params.size_variation = 40.0 / 100.0;
    assert!(check(&i).feasible);
    i.params.size_variation = 15.0 / 100.0;
    i.params.allowed_sides = vec![5, 6, 7];
    assert!(check(&i).feasible);
}
#[test]
fn bulge_requires_positive_defects() {
    let i = input("vase_bulge");
    let f = check(&i);
    assert!(!f.feasible);
    assert!(f.pentagons_needed >= 1);
    assert!(f.suggestions[0].contains(&5));
}
#[test]
fn waist_blocks_all_sets_and_names_heights() {
    for mask in 1u32..64 {
        let mut i = input("hourglass");
        i.params.allowed_sides = (4..=9).filter(|k| mask & (1 << (k - 4)) != 0).collect();
        let f = check(&i);
        assert!(!f.feasible);
        assert!(!f.waist_ranges.is_empty());
        assert!(f.suggestions.is_empty());
        let msg = feasibility::message(
            &Surface::from_profile(&i.surface.profile).unwrap(),
            &f,
            &i.params,
        );
        for range in f.waist_ranges {
            assert!(msg.contains(&format!("{:.2}–{:.2}", range[0], range[1])));
        }
    }
}
#[test]
fn small_radius_covers_full_height() {
    let mut i = input("cylinder");
    i.surface.profile = vec![[1.5, 0.0], [1.5, 60.0]];
    i.params.bevel_cap_deg = 60.0;
    i.params.min_bounding_square = 4.0;
    let f = check(&i);
    assert!(!f.feasible);
    assert_eq!(f.small_radius_ranges, vec![[0.0, 60.0]]);
}
#[test]
fn pentagons_only_cannot_balance_cylinder() {
    let mut i = input("cylinder");
    i.params.allowed_sides = vec![5];
    let f = check(&i);
    assert!(!f.feasible);
    assert!(f.suggestions[0].contains(&6));
}
#[test]
fn exact_message_snapshots() {
    let i = input("cylinder");
    assert_eq!(
        message(&check(&i), &i.params),
        "The allowed side set is feasible for this profile and size range."
    );
    let mut f = check(&i);
    f.feasible = false;
    f.pentagons_needed = 9;
    f.suggestions = vec![vec![5, 6], vec![5, 6, 7]];
    assert_eq!(message(&f,&i.params), "6 only is not possible at ±15 %: this profile bulges, so it needs about 9 five-sided pieces. With size variation of at least ±20 %, [6] works. Try [5,6] or [5,6,7].");
    let mut i = i;
    i.params.allowed_sides = vec![5];
    let mut f = check(&i);
    f.suggestions = vec![vec![5, 6]];
    assert_eq!(message(&f,&i.params), "5 only is not possible at ±15 %: the allowed side counts cannot balance the required average or fit an even column wrap. Try [5,6].");
}
#[test]
fn generation_gate_cli_and_determinism() {
    for name in ["cylinder", "cone_9_to_6", "vase_bulge", "hourglass"] {
        let i = input(name);
        let json = serde_json::to_string(&i).unwrap();
        for only in [false, true] {
            let run = || {
                if only {
                    vstone_core::feasibility_only_json(&json)
                } else {
                    vstone_core::generate_json(&json)
                }
            };
            assert_eq!(Sha256::digest(run()), Sha256::digest(run()));
        }
        let out = vstone_core::feasibility_only(&i);
        assert_eq!(out.feasibility, check(&i));
        assert_eq!(
            out.status,
            if out.feasibility.feasible {
                "ok"
            } else {
                "infeasible"
            }
        );
        if !out.feasibility.feasible {
            assert_eq!(vstone_core::generate(&i).status, "infeasible");
        }
        assert!(out.pieces.is_empty());
        assert!(out.blanks.is_empty());
    }
    let path = format!(
        "{}/../tests/fixtures/core_input_cylinder.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_vstone-core"))
        .args(["--feasibility-only", &path])
        .output()
        .unwrap();
    assert!(output.status.success());
    let out: vstone_core::model::CoreOutput = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(out.status, "ok");
}

#[test]
fn cone_numeric_variation_message_snapshot() {
    let mut i = input("cone_9_to_6");
    let surface = Surface::from_profile(&i.surface.profile).unwrap();
    let f = check(&i);
    assert_eq!(feasibility::message(&surface, &f, &i.params),
        "6 only is not possible at ±15 %: the circumference changes, so it needs about 4 five-and-seven-sided pairs. With size variation of at least ±25 %, [6] works. Try [5,6,7] or [4,6,7] or [5,6,8] or [4,6,8].");
    i.params.size_variation = 20.0 / 100.0;
    assert!(!check(&i).feasible);
    i.params.size_variation = 25.0 / 100.0;
    assert!(check(&i).feasible);
    // The limiting lower circumference at eight columns gives 21.46%,
    // rounded up to 25%, not the wide shortcut.
    let threshold = 1.0 - std::f64::consts::TAU * 6.0 / (8.0 * 6.0);
    i.params.size_variation = threshold - 1e-8;
    assert!(!check(&i).feasible);
    i.params.size_variation = threshold + 1e-8;
    assert!(check(&i).feasible);
}

#[test]
fn reference_cylinder_dip_selects_ten() {
    let i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let f = check(&i);
    assert!(f.feasible);
    assert_eq!(f.columns, [10]);
    assert!(9.0 * (1.0 - (std::f64::consts::PI / 8.0).cos()) > i.params.surface_tolerance);
    assert!(9.0 * (1.0 - (std::f64::consts::PI / 10.0).cos()) <= i.params.surface_tolerance);
}

#[test]
fn dip_boundary_is_inclusive_and_respects_even_counts() {
    let mut i = input("cylinder");
    i.params.size_variation = 0.4;
    let boundary = 9.0 * (1.0 - (std::f64::consts::PI / 10.0).cos());
    i.params.surface_tolerance = boundary;
    assert_eq!(check(&i).columns, [10]);
    i.params.surface_tolerance = boundary - 1e-10;
    assert_eq!(check(&i).columns, [12]);
    i.params.surface_tolerance = boundary + 1e-10;
    assert_eq!(check(&i).columns, [10]);
}

#[test]
fn no_dip_compatible_count_reports_limit_without_side_suggestions() {
    let mut i = input("cylinder");
    i.params.surface_tolerance = 0.1;
    let f = check(&i);
    assert!(!f.feasible);
    assert!(f.columns.is_empty());
    assert!(f.suggestions.is_empty());
    let msg = message(&f, &i.params);
    assert!(msg
        .contains("no even column count satisfies r*(1-cos(pi/n)) <= surface_tolerance (0.1 in)"));
    assert!(msg.contains("heights 0.00–60.00 in"));
    assert!(!msg.contains("works."));
    assert_eq!(vstone_core::feasibility_only(&i).status, "infeasible");
    assert_eq!(vstone_core::generate(&i).status, "infeasible");
}

#[test]
fn dip_variation_advice_uses_first_working_grid_point() {
    let mut i = input("cylinder");
    i.params.target_size = 6.5;
    i.params.size_variation = 0.05;
    i.params.surface_tolerance = 0.5;
    let f = check(&i);
    assert!(!f.feasible);
    assert!(message(&f, &i.params).contains("With size variation of at least ±15 %, [6] works."));
    i.params.size_variation = 0.10;
    assert!(!check(&i).feasible);
    i.params.size_variation = 0.15;
    assert_eq!(check(&i).columns, [10]);
}
