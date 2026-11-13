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
    input
}
fn check(input: &CoreInput) -> vstone_core::model::Feasibility {
    feasibility::check(
        &Surface::from_profile(&input.surface.profile).unwrap(),
        &input.params,
    )
}
fn message(f: &vstone_core::model::Feasibility, p: &vstone_core::model::Params) -> String {
    feasibility::message_with_depth(
        &Surface::from_profile(&input("cylinder").surface.profile).unwrap(),
        f,
        p,
        0.25 * p.target_size,
    )
}
#[test]
fn cylinder_nearest_integer_wrap() {
    let f = check(&input("cylinder"));
    assert!(f.feasible);
    assert_eq!(f.columns.len(), 1);
    assert!((8..=12).contains(&f.columns[0]));
    assert_eq!(f.columns[0], 9);
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
        "The allowed side set is feasible for this profile and size range. At height 0.00 in, size-range candidates (width = circumference/n): 9 columns: ~6.3 in piece width; 10 columns: ~5.7 in piece width; 11 columns: ~5.1 in piece width. These local estimates do not certify whole-profile feasibility."
    );
    let mut f = check(&i);
    f.feasible = false;
    f.pentagons_needed = 9;
    f.suggestions = vec![vec![5, 6], vec![5, 6, 7]];
    assert_eq!(message(&f,&i.params), "6 only is not possible at ±15 %: this profile bulges, so it needs about 9 five-sided pieces. With size variation of at least ±20 %, [6] works. Try [5,6] or [5,6,7]. At height 0.00 in, size-range candidates (width = circumference/n): 9 columns: ~6.3 in piece width; 10 columns: ~5.7 in piece width; 11 columns: ~5.1 in piece width. These local estimates do not certify whole-profile feasibility.");
    let mut i = i;
    i.params.allowed_sides = vec![5];
    let mut f = check(&i);
    f.suggestions = vec![vec![5, 6]];
    assert_eq!(message(&f,&i.params), "5 only is not possible at ±15 %: the allowed side counts cannot balance the required average or fit a column wrap. Try [5,6]. At height 0.00 in, size-range candidates (width = circumference/n): 9 columns: ~6.3 in piece width; 10 columns: ~5.7 in piece width; 11 columns: ~5.1 in piece width. These local estimates do not certify whole-profile feasibility.");
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
    assert_eq!(feasibility::message_with_depth(&surface, &f, &i.params, 1.5),
        "6 only is not possible at ±15 %: the circumference changes, so it needs about 2 five-and-seven-sided pairs. With size variation of at least ±25 %, [6] works. Native C2 supports [6] with size variation ≥ 22% (whole-percent grid); C3/C4 verification still required. Try [5,6,7] or [4,6,7] or [5,6,8] or [4,6,8]. At height 0.00 in, size-range candidates (width = circumference/n): 9 columns: ~6.3 in piece width; 10 columns: ~5.7 in piece width; 11 columns: ~5.1 in piece width. These local estimates do not certify whole-profile feasibility.");
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
fn reference_cylinder_uses_size_band_only_and_keeps_widths() {
    let mut i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let expected = check(&i);
    assert!(expected.feasible);
    assert_eq!(expected.columns, [9]);
    for depth in [0.001, 0.5, 0.75, 2.0] {
        i.params.depth = depth;
        assert_eq!(check(&i), expected);
        let msg = message(&expected, &i.params);
        assert!(msg.contains("8 columns: ~7.1 in piece width"));
        assert!(msg.contains("10 columns: ~5.7 in piece width"));
        assert!(!msg.contains("dip"));
        assert!(!msg.contains("tolerance"));
    }
}

#[test]
fn depth_warning_is_inclusive_advisory_and_deterministic() {
    let mut i = input("cylinder");
    i.params.target_size = 10.0;
    i.params.size_variation = 0.4;
    let surface = Surface::from_profile(&i.surface.profile).unwrap();
    let f = check(&i);
    assert!(f.feasible);
    for depth in [1.5, 2.0, 3.0] {
        let msg = feasibility::message_with_depth(&surface, &f, &i.params, depth);
        assert!(!msg.contains("Warning:"), "{msg}");
    }
    for depth in [1.5 - 1e-10, 3.0 + 1e-10] {
        let run = || feasibility::message_with_depth(&surface, &f, &i.params, depth);
        let msg = run();
        assert!(msg.contains("is outside the recommended range 1.50–3.00 in for target size 10 in; expect slivers (too shallow) or distorted pieces (too deep)."));
        assert!(msg.starts_with("The allowed side set is feasible"));
        assert_eq!(msg, run());
        assert_eq!(check(&i), f);
    }
    assert_eq!(
        feasibility::message(&surface, &f, &i.params),
        feasibility::message_with_depth(&surface, &f, &i.params, 2.0)
    );
    i.params.allowed_sides = vec![5];
    let f = check(&i);
    assert!(!f.feasible);
    let baseline = feasibility::message_with_depth(&surface, &f, &i.params, 2.0);
    let warned = feasibility::message_with_depth(&surface, &f, &i.params, 0.1);
    assert_eq!(warned, format!("{baseline} Warning: depth 0.1 in is outside the recommended range 1.50–3.00 in for target size 10 in; expect slivers (too shallow) or distorted pieces (too deep)."));
    assert_eq!(check(&i), f);
}

#[test]
fn out_of_range_depth_warning_reaches_generation_messages_without_refusal() {
    let mut i = input("cylinder");
    // The shared reference depth is in range; make the advisory case explicit
    // rather than depending on a historically out-of-range fixture value.
    let quiet = vstone_core::feasibility_only(&i);
    assert_eq!(quiet.status, "ok");
    assert!(!quiet.message.contains("Warning:"));
    i.params.depth = 2.0;
    let surface = Surface::from_profile(&i.surface.profile).unwrap();
    let f = check(&i);
    assert!(f.feasible);
    let msg = feasibility::message(&surface, &f, &i.params);
    assert!(msg.contains("Warning: depth 2 in is outside the recommended range"));
    let out = vstone_core::feasibility_only(&i);
    assert_eq!(out.status, "ok");
    assert_eq!(out.feasibility, f);
    assert!(out.message.contains("Warning: depth 2 in"));
    assert_eq!(out.feasibility, quiet.feasibility);
}

#[test]
fn size_band_empty_intervals_still_reject_and_advise_variation() {
    let mut i = input("cylinder");
    i.params.target_size = 6.65;
    i.params.size_variation = 0.05;
    let f = check(&i);
    assert!(!f.feasible);
    assert!(f.columns.is_empty());
    assert!(f.suggestions.is_empty());
    assert!(message(&f, &i.params).contains("With size variation of at least ±10 %, [6] works."));
    assert_eq!(vstone_core::feasibility_only(&i).status, "infeasible");
    assert_eq!(vstone_core::generate(&i).status, "infeasible");
    i.params.size_variation = 0.10;
    assert_eq!(check(&i).columns, [9]);
    assert!(check(&i).feasible);
}

#[test]
fn configured_depth_reaches_public_feasibility_message() {
    let mut i = input("cylinder");
    i.params.depth = 0.1;
    let out = vstone_core::feasibility_only(&i);
    assert!(out.message.contains("Warning: depth 0.1 in"));
    i.params.depth = 0.2 * i.params.target_size;
    let quiet = vstone_core::feasibility_only(&i);
    assert!(!quiet.message.contains("Warning:"));
    assert_eq!(out.feasibility, quiet.feasibility);
}

#[test]
fn constant_wrap_absorbs_curvature_and_hex_advice_is_native_verified() {
    for (profile, variation) in [("cone_9_to_6", 0.25), ("vase_bulge", 0.30)] {
        let mut i = input(profile);
        i.params.target_size = 6.5;
        i.params.allowed_sides = vec![6];
        i.params.size_variation = variation;
        let surface = Surface::from_profile(&i.surface.profile).unwrap();
        let f = feasibility::check(&surface, &i.params);
        assert!(f.feasible, "{profile}: {f:?}");
        assert_eq!(f.column_changes, 0);
        assert_eq!(f.pentagons_needed, 0);
        assert_eq!(f.heptagons_needed, 0);
        i.params.size_variation = 0.05;
        let failed = feasibility::check(&surface, &i.params);
        assert!(!failed.feasible);
        let text = feasibility::message(&surface, &failed, &i.params);
        let advice = text.split("[6] with size variation ≥ ").nth(1).unwrap();
        let percent: f64 = advice.split('%').next().unwrap().parse().unwrap();
        i.params.size_variation = percent / 100.;
        assert!(feasibility::check(&surface, &i.params).feasible);
        i.params.size_variation -= 0.01;
        assert!(!feasibility::check(&surface, &i.params).feasible);
    }
}
