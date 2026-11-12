use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use vstone_core::{model::CoreInput, seeds, sidecount, surface::Surface, topology::Topology};
fn input(name: &str) -> CoreInput {
    let mut i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    i.surface.profile = serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/../tests/fixtures/profiles/{name}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap();
    i.params.target_size = 6.0;
    i
}
fn surface(i: &CoreInput) -> Surface {
    Surface::from_profile(&i.surface.profile).unwrap()
}
fn assert_manifold(t: &Topology) {
    let mut edges = BTreeMap::new();
    for &[a, b, c] in &t.corners {
        assert!(a < b && b < c);
        for e in [(a, b), (a, c), (b, c)] {
            *edges.entry(e).or_insert(0) += 1;
        }
    }
    for ((a, b), count) in edges {
        assert!((1..=2).contains(&count));
        if !t.boundary[a] && !t.boundary[b] {
            assert_eq!(count, 2);
        }
        assert!(t.neighbors[a].contains(&b) && t.neighbors[b].contains(&a));
    }
}
#[test]
fn regular_cylinder_matches_c2_columns() {
    let mut i = input("cylinder");
    i.params.randomness = 0.0;
    let s = surface(&i);
    let f = vstone_core::feasibility::check(&s, &i.params);
    let t = sidecount::build(&s, &i.params).unwrap();
    let circumference = std::f64::consts::TAU * s.r(0.);
    let lo =
        (circumference / (i.params.target_size * (1. + i.params.size_variation))).ceil() as usize;
    let hi =
        (circumference / (i.params.target_size * (1. - i.params.size_variation))).floor() as usize;
    let nearest = (circumference / i.params.target_size).round() as usize;
    let expected = nearest.clamp(lo, hi);
    assert_eq!(f.columns, vec![expected as u32]);
    // Exact lattice spacing describes placement, before bounded repair is
    // allowed to move positions. Final repaired geometry is checked below
    // against the independent physical spacing oracle, not frozen angles.
    let initial = seeds::place(&s, &i.params, &mut seeds::rng(i.params.seed)).unwrap();
    for row in 0..=initial.iter().map(|x| x.row).max().unwrap() {
        let mut sites: Vec<_> = initial.iter().filter(|x| x.row == row).collect();
        sites.sort_by(|a, b| a.u.total_cmp(&b.u));
        assert_eq!(sites.len(), expected);
        for j in 0..sites.len() {
            let d = (sites[(j + 1) % sites.len()].u - sites[j].u).rem_euclid(std::f64::consts::TAU);
            assert!((d - std::f64::consts::TAU / sites.len() as f64).abs() < 1e-9);
        }
    }
    for j in 0..t.tiles.len() {
        if sidecount::interior(&t, j, &s) {
            assert_eq!(t.degree(j), 6);
        }
    }
    for row in 0..=t.tiles.iter().map(|x| x.row).max().unwrap() {
        assert_eq!(t.tiles.iter().filter(|x| x.row == row).count(), expected);
    }
    assert_spacing(&s, &i.params, &t);
    assert_manifold(&t);
}
#[test]
fn twenty_organic_cylinders_are_allowed_and_manifold() {
    for seed in 1..=20 {
        let mut i = input("cylinder");
        i.params.allowed_sides = vec![5, 6, 7];
        i.params.randomness = 1.0;
        i.params.seed = seed;
        let s = surface(&i);
        let t = sidecount::build(&s, &i.params).unwrap();
        assert_manifold(&t);
        for j in 0..t.tiles.len() {
            if sidecount::interior(&t, j, &s) {
                assert!((5..=7).contains(&t.degree(j)));
            }
        }
    }
}
#[test]
fn seeded_topology_is_byte_deterministic() {
    let mut i = input("cylinder");
    i.params.randomness = 1.0;
    i.params.allowed_sides = vec![5, 6, 7];
    let s = surface(&i);
    let run = |p: &vstone_core::model::Params| {
        serde_json::to_vec(&sidecount::build(&s, p).unwrap()).unwrap()
    };
    let a = run(&i.params);
    assert_eq!(Sha256::digest(&a), Sha256::digest(run(&i.params)));
    i.params.seed += 1;
    assert_ne!(a, run(&i.params));
}
fn assert_spacing(s: &Surface, p: &vstone_core::model::Params, t: &Topology) {
    let mut edges = 0;
    for a in 0..t.tiles.len() {
        // Independent exclusion by physical row, not the solver's predicate.
        if t.tiles[a].auxiliary || t.tiles[a].s <= 1e-9 || t.tiles[a].s >= s.length() - 1e-9 {
            continue;
        }
        let mean = t.neighbors[a]
            .iter()
            .map(|&b| seeds::spacing(s, &t.tiles[a], &t.tiles[b]) / p.target_size)
            .sum::<f64>()
            / t.degree(a) as f64;
        assert!(
            (mean - 1.0).abs() <= p.size_variation + 1e-9,
            "tile {a}: mean ratio {mean}"
        );
        for &b in &t.neighbors[a] {
            if b <= a
                || t.tiles[b].auxiliary
                || t.tiles[b].s <= 1e-9
                || t.tiles[b].s >= s.length() - 1e-9
            {
                continue;
            }
            edges += 1;
            let ratio = seeds::spacing(s, &t.tiles[a], &t.tiles[b]) / p.target_size;
            let variation = p.size_variation
                + if matches!(t.degree(a), 5 | 7) || matches!(t.degree(b), 5 | 7) {
                    0.05
                } else {
                    0.0
                };
            assert!(
                ratio >= 1.0 - variation - 1e-9 && ratio <= 1.0 + variation + 1e-9,
                "edge {a}-{b}: ratio={ratio:.12}, requested band={}..{}",
                1.0 - p.size_variation,
                1.0 + p.size_variation
            );
        }
    }
    assert!(edges > 0);
}
#[test]
fn every_interior_edge_on_cylinder_cone_and_vase_is_in_band() {
    for name in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        let mut i = input(name);
        i.params.allowed_sides = vec![5, 6, 7];
        let s = surface(&i);
        let t = sidecount::build(&s, &i.params).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_spacing(&s, &i.params, &t);
    }
}
#[test]
fn seeds_one_to_five_change_positions_with_valid_geometry() {
    for randomness in [0.5, 1.0] {
        let mut positions = Vec::new();
        for seed in 1..=5 {
            let mut i = input("cylinder");
            i.params.allowed_sides = vec![5, 6, 7];
            i.params.randomness = randomness;
            i.params.seed = seed;
            let s = surface(&i);
            let t = sidecount::build(&s, &i.params).unwrap();
            assert_spacing(&s, &i.params, &t);
            assert!(
                !positions.contains(&t.tiles),
                "seed {seed}: repeated positions"
            );
            positions.push(t.tiles.clone());
            // Bounded positional jitter need not cross a Delaunay flip boundary.
            // Graph uniqueness is not a position-only randomness requirement.
            assert_manifold(&t);
        }
    }
}
#[test]
fn initial_position_jitter_respects_local_component_caps_and_symmetric_depths() {
    let mut i = input("cylinder");
    i.params.allowed_sides = vec![5, 6, 7];
    i.params.randomness = 0.;
    let s = surface(&i);
    let baseline = seeds::place(&s, &i.params, &mut seeds::rng(1)).unwrap();
    let mut rows = BTreeMap::new();
    for site in &baseline {
        let entry = rows.entry(site.row).or_insert((site.s, 0usize));
        entry.1 += 1;
    }
    for randomness in [0., 0.5, 1.] {
        i.params.randomness = randomness;
        for seed in 1..=5 {
            let sites = seeds::place(&s, &i.params, &mut seeds::rng(seed)).unwrap();
            assert_eq!(sites.len(), baseline.len());
            for (site, base) in sites.iter().zip(&baseline) {
                assert_eq!(site.row, base.row);
                assert_eq!(site.auxiliary, base.auxiliary);
                let depth = match site.row % 3 {
                    0 => 0.,
                    1 => i.params.depth / 2.,
                    _ => i.params.depth,
                };
                assert_eq!(site.depth, depth);
                let angular_pitch = std::f64::consts::TAU / rows[&base.row].1 as f64;
                let row_pitch = if let Some(next) = rows.get(&(base.row + 1)) {
                    next.0 - base.s
                } else {
                    base.s - rows[&(base.row - 1)].0
                };
                let du = (site.u - base.u + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI;
                assert!(du.abs() <= randomness * 0.1 * angular_pitch + 1e-12);
                assert!((site.s - base.s).abs() <= randomness * 0.1 * row_pitch + 1e-12);
                if base.auxiliary || base.s <= 0. || base.s >= s.length() {
                    assert_eq!(site.s, base.s);
                }
            }
            if randomness == 0. {
                assert_eq!(sites, baseline);
            } else {
                assert_ne!(sites, baseline);
            }
        }
    }
}

#[test]
fn zero_randomness_repaired_topology_is_seed_independent() {
    let mut i = input("cylinder");
    i.params.randomness = 0.;
    let s = surface(&i);
    let mut baseline = None;
    for seed in 1..=5 {
        i.params.seed = seed;
        let t = sidecount::build(&s, &i.params).unwrap();
        assert_spacing(&s, &i.params, &t);
        assert_manifold(&t);
        let bytes = serde_json::to_vec(&t).unwrap();
        if let Some(expected) = &baseline {
            assert_eq!(&bytes, expected);
        } else {
            baseline = Some(bytes);
        }
    }
}

#[test]
fn cone_and_vase_repair_reduce_nonzero_residual() {
    for name in ["cone_9_to_6", "vase_bulge"] {
        let mut i = input(name);
        i.params.allowed_sides = vec![5, 6, 7];
        let s = surface(&i);
        let mut rng = seeds::rng(i.params.seed);
        let mut sites = seeds::place(&s, &i.params, &mut rng).unwrap();
        // Deliberate, reproducible spacing violation ensures this exercises repair
        // rather than succeeding solely because the initial defect count matches.
        let j = sites
            .iter()
            .position(|x| {
                !x.auxiliary
                    && x.s > i.params.target_size
                    && x.s < s.length() - i.params.target_size
            })
            .unwrap();
        sites[j].u = (sites[j].u + 0.35 * i.params.target_size / s.r(sites[j].s))
            .rem_euclid(std::f64::consts::TAU);
        let (t, report) = sidecount::repair_with_report(&s, &i.params, sites, &mut rng)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        eprintln!("{name}: {report:?}");
        assert!(
            report.initial_residual > 1e-8,
            "must begin outside strict band"
        );
        assert!(report.accepted_steps > 0 && report.iterations > 0);
        assert!(report.final_residual < report.initial_residual);
        assert!(report.final_score < report.initial_score);
        assert_spacing(&s, &i.params, &t);
    }
}
#[test]
fn periodic_seam_and_bad_sites() {
    let i = input("cylinder");
    let s = surface(&i);
    let mut rng = seeds::rng(1);
    let mut sites = seeds::place(&s, &i.params, &mut rng).unwrap();
    let t = Topology::build(sites.clone()).unwrap();
    assert_manifold(&t);
    assert!(t.neighbors.iter().enumerate().any(|(a, ns)| ns
        .iter()
        .any(|&b| (t.tiles[a].u - t.tiles[b].u).abs() > std::f64::consts::PI)));
    sites[1] = sites[0].clone();
    assert!(Topology::build(sites).is_err());
}

#[test]
fn cone_and_bulge_defect_budgets() {
    for name in ["cone_9_to_6", "vase_bulge"] {
        let mut i = input(name);
        i.params.allowed_sides = vec![5, 6, 7];
        let s = surface(&i);
        let f = vstone_core::feasibility::check(&s, &i.params);
        let t = sidecount::build(&s, &i.params).unwrap();
        assert_manifold(&t);
        let count = |degree| {
            (0..t.tiles.len())
                .filter(|&j| sidecount::interior(&t, j, &s) && t.degree(j) == degree)
                .count()
        };
        if name == "vase_bulge" {
            assert!(count(5).abs_diff(f.pentagons_needed as usize) <= 1);
        } else {
            assert!(count(5) >= f.column_changes as usize);
            assert!(count(7) >= f.column_changes as usize);
            let mut rng = seeds::rng(i.params.seed);
            let initial = seeds::place(&s, &i.params, &mut rng).unwrap();
            let mut rows = BTreeMap::<usize, usize>::new();
            for site in initial {
                if !site.auxiliary {
                    *rows.entry(site.row).or_default() += 1;
                }
            }
            let rows: Vec<_> = rows.into_iter().collect();
            let transitions: Vec<_> = rows
                .windows(2)
                .filter(|w| w[0].1 != w[1].1)
                .map(|w| w[1].0)
                .collect();
            assert!(!transitions.is_empty());
            for j in 0..t.tiles.len() {
                if !sidecount::interior(&t, j, &s) || !matches!(t.degree(j), 5 | 7) {
                    continue;
                }
                assert!(
                    transitions
                        .iter()
                        .any(|&row| t.tiles[j].row.abs_diff(row) <= 1),
                    "defect tile {j}, row {} outside C2 transition rows {transitions:?}",
                    t.tiles[j].row
                );
                assert!(t.neighbors[j].iter().any(|&k| sidecount::interior(&t,k,&s) && t.degree(k)==12-t.degree(j)),
                    "defect {j} has no adjacent complementary 5/7");
            }
        }
    }
}

#[test]
fn cone_pairs_are_constructed_before_repair() {
    let mut i = input("cone_9_to_6");
    i.params.allowed_sides = vec![5, 6, 7];
    i.params.randomness = 0.0;
    let s = surface(&i);
    let f = vstone_core::feasibility::check(&s, &i.params);
    let mut rng = seeds::rng(i.params.seed);
    let t = Topology::build(seeds::place(&s, &i.params, &mut rng).unwrap()).unwrap();
    let mut rows = BTreeMap::<usize, usize>::new();
    for site in &t.tiles {
        if !site.auxiliary {
            *rows.entry(site.row).or_default() += 1;
        }
    }
    let rows: Vec<_> = rows.into_iter().collect();
    let mut sequence = rows.iter().map(|x| x.1 as u32).collect::<Vec<_>>();
    sequence.dedup();
    assert_eq!(sequence, f.columns);
    // Odd counts and unit transitions are legal: independently verify every
    // actual row's circumference width, rather than an obsolete even wrap.
    for &(row, count) in &rows {
        let site = t.tiles.iter().find(|s| s.row == row).unwrap();
        let width = std::f64::consts::TAU * s.r(site.s) / count as f64;
        assert!(width >= i.params.target_size * (1. - i.params.size_variation) - 1e-9);
        assert!(width <= i.params.target_size * (1. + i.params.size_variation) + 1e-9);
    }
    for pair in rows.windows(2).filter(|w| w[0].1 != w[1].1) {
        let row = pair[1].0;
        let needed = pair[0].1.abs_diff(pair[1].1);
        for degree in [5, 7] {
            let defects: Vec<_> = (0..t.tiles.len())
                .filter(|&j| {
                    sidecount::interior(&t, j, &s)
                        && t.tiles[j].row.abs_diff(row) <= 1
                        && t.degree(j) == degree
                })
                .collect();
            assert_eq!(
                defects.len(),
                needed,
                "transition row {row}, degree {degree}"
            );
            for j in defects {
                assert!(t.neighbors[j]
                    .iter()
                    .any(|&k| sidecount::interior(&t, k, &s) && t.degree(k) == 12 - degree));
            }
        }
    }
}

#[test]
fn edge_allowance_is_exactly_five_percentage_points_only_at_five_or_seven() {
    let i = input("cylinder");
    let s = surface(&i);
    let mut rng = seeds::rng(1);
    let mut t = Topology::build(seeds::place(&s, &i.params, &mut rng).unwrap()).unwrap();
    // This policy test deliberately sets degree vectors; geometric acceptance
    // is independently exercised by assert_spacing on real Delaunay outputs.
    for (a, b, expected) in [
        (6, 6, 0.15),
        (4, 8, 0.15),
        (5, 6, 0.20),
        (6, 7, 0.20),
        (5, 7, 0.20),
    ] {
        t.neighbors[0] = vec![1; a];
        t.neighbors[1] = vec![0; b];
        assert!((sidecount::edge_variation(&i.params, &t, 0, 1) - expected).abs() < 1e-12);
    }
}

#[test]
fn conformal_path_spacing_integrates_surface_metric_exactly() {
    for name in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        let i = input(name);
        let s = surface(&i);
        let a = seeds::Seed {
            u: 0.1,
            v: s.v_of_s(3.0),
            s: 3.0,
            row: 0,
            auxiliary: false,
            depth: 0.0,
        };
        let mut b = seeds::Seed {
            u: 0.1,
            v: s.v_of_s(17.0),
            s: 17.0,
            row: 1,
            auxiliary: false,
            depth: 0.0,
        };
        assert!((seeds::spacing(&s, &a, &b) - 14.0).abs() < 1e-10);
        b.u += 0.4;
        let dv = b.v - a.v;
        let expected = 14.0 * (1.0 + (0.4 / dv).powi(2)).sqrt();
        assert!((seeds::spacing(&s, &a, &b) - expected).abs() < 1e-10);
    }
}

#[test]
fn a05_symmetric_depths_differ_across_every_vertical_neighbor() {
    let mut i = input("cylinder");
    i.params.randomness = 0.0;
    i.params.depth = 2.0;
    i.params.rims.bottom = "level".into();
    i.params.rims.top = "level".into();
    let s = surface(&i);
    let t = sidecount::build(&s, &i.params).unwrap();
    assert!(t.tiles.iter().any(|x| x.auxiliary));
    let mut vertical = 0;
    for (index, site) in t.tiles.iter().enumerate() {
        assert_eq!(site.depth, (site.row % 3) as f64);
        for &j in &t.neighbors[index] {
            if site.row.abs_diff(t.tiles[j].row) == 1 {
                vertical += 1;
                assert_ne!(site.depth, t.tiles[j].depth);
            }
        }
    }
    assert!(vertical > 0);
}

#[test]
fn a05_depth_bounds_determinism_and_depth_independent_topology() {
    use rand::RngCore;
    for randomness in [0.0, 0.5, 1.0] {
        for seed in 1..=3 {
            let mut i = input("cylinder");
            i.params.allowed_sides = vec![5, 6, 7];
            i.params.randomness = randomness;
            i.params.seed = seed;
            let s = surface(&i);
            let mut previous: Option<Topology> = None;
            for depth in [1.0, 2.0, 3.0] {
                i.params.depth = depth;
                let mut rng = seeds::rng(seed);
                let sites = seeds::place(&s, &i.params, &mut rng).unwrap();
                // Placement MUST consume exactly the old positional stream,
                // independent of depth, and carry symmetric provisional offsets.
                for site in &sites {
                    assert_eq!(site.depth, seeds::symmetric_depth(site.row, depth));
                }
                let mut expected_rng = rng.clone();
                let (t, _) = sidecount::repair_with_report(&s, &i.params, sites, &mut rng).unwrap();
                let repeat = sidecount::build(&s, &i.params).unwrap();
                assert_eq!(
                    serde_json::to_vec(&t).unwrap(),
                    serde_json::to_vec(&repeat).unwrap()
                );
                assert!(t.tiles.iter().all(|x| (0.0..=depth).contains(&x.depth)));
                if randomness == 0.0 {
                    // Regular cylinder needs no repair draws; relief at zero
                    // randomness must not advance the stream either.
                    assert_eq!(rng.next_u64(), expected_rng.next_u64());
                }
                if let Some(mut prior) = previous.take() {
                    // Exact equality of ALL old fields, not only adjacency.
                    for (a, b) in prior.tiles.iter_mut().zip(&t.tiles) {
                        assert!((a.depth / (depth - 1.0) - b.depth / depth).abs() < 1e-14);
                        a.depth = b.depth;
                    }
                    assert_eq!(prior, t);
                }
                previous = Some(t);
            }
        }
    }
}

/// Exact owner reference inputs: do not substitute the historical target 6.0.
#[test]
fn exact_reference_full_topology_matrix() {
    let mut failures = Vec::new();
    let mut attempted = 0;
    for name in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        for randomness in [0., 0.5, 1.] {
            for seed in 1..=10 {
                let mut i = if name == "cylinder" {
                    // Match c4_matrix's sampled reference profile exactly.
                    serde_json::from_str(include_str!(
                        "../../tests/fixtures/core_input_cylinder.json"
                    ))
                    .unwrap()
                } else {
                    input(name)
                };
                i.params.target_size = 6.5;
                i.params.size_variation = 0.15;
                i.params.depth = 1.5;
                i.params.max_corner_angle_deg = 230.;
                i.params.allowed_sides = if name == "cylinder" {
                    vec![6]
                } else {
                    vec![5, 6, 7]
                };
                i.params.randomness = randomness;
                i.params.seed = seed;
                let s = surface(&i);
                // Verify classification before repair, not just the final graph.
                for site in seeds::place(&s, &i.params, &mut seeds::rng(seed)).unwrap() {
                    assert_eq!(site.auxiliary, site.s < 0. || site.s > s.length());
                }
                attempted += 1;
                let t = match sidecount::build(&s, &i.params) {
                    Ok(t) => t,
                    Err(e) => {
                        failures.push(format!("{name} r={randomness} seed={seed}: {e}"));
                        continue;
                    }
                };
                assert_manifold(&t);
                assert_spacing(&s, &i.params, &t);
                for j in 0..t.tiles.len() {
                    if sidecount::interior(&t, j, &s) {
                        assert!(i.params.allowed_sides.contains(&(t.degree(j) as u32)));
                    }
                }
                for above in [false, true] {
                    let rows: std::collections::BTreeSet<_> = t
                        .tiles
                        .iter()
                        .filter(|x| if above { x.s > s.length() } else { x.s < 0. })
                        .map(|x| {
                            assert!(x.auxiliary);
                            x.row
                        })
                        .collect();
                    assert!(rows.len() >= 2);
                }
                assert_eq!(t, sidecount::build(&s, &i.params).unwrap());
            }
        }
    }
    assert_eq!(attempted, 90);
    assert!(
        failures.is_empty(),
        "90-case topology failures:\n{}",
        failures.join("\n")
    );
}

/// Reproduce the matrix's dense cylinder profile and complete reference params,
/// not input("cylinder"), which substitutes a two-point profile and target 6.0.
#[test]
fn reference_cylinder_half_randomness_cleans_small_spacing_residuals() {
    let base: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    for seed in 1..=10 {
        let mut i = base.clone();
        i.params.target_size = 6.5;
        i.params.size_variation = 0.15;
        i.params.depth = 1.5;
        i.params.max_corner_angle_deg = 230.;
        i.params.allowed_sides = vec![6];
        i.params.randomness = 0.5;
        i.params.seed = seed;
        let s = surface(&i);
        for perturb in [false, true] {
            let mut rng = seeds::rng(seed);
            let mut sites = seeds::place(&s, &i.params, &mut rng).unwrap();
            if perturb {
                // Exact nine-column pitch may already satisfy the band at .5.
                // Preserve natural-input coverage AND force nonzero repair.
                let j = sites
                    .iter()
                    .position(|x| {
                        !x.auxiliary
                            && x.s > i.params.target_size
                            && x.s < s.length() - i.params.target_size
                    })
                    .unwrap();
                sites[j].u = (sites[j].u + 0.35 * i.params.target_size / s.r(sites[j].s))
                    .rem_euclid(std::f64::consts::TAU);
            }
            let (t, report) = sidecount::repair_with_report(&s, &i.params, sites, &mut rng)
                .unwrap_or_else(|e| {
                    panic!("matrix cylinder r=.5 seed={seed} perturb={perturb}: {e}")
                });
            assert!(report.final_residual <= report.initial_residual);
            if perturb {
                assert!(
                    report.initial_residual > 1e-8,
                    "must exercise nonzero repair"
                );
                assert!(report.final_residual < report.initial_residual);
                assert!(report.accepted_steps > 0 && report.iterations > 0);
            }
            assert_manifold(&t);
            assert_spacing(&s, &i.params, &t);
            for j in 0..t.tiles.len() {
                if sidecount::interior(&t, j, &s) {
                    assert_eq!(t.degree(j), 6);
                }
            }
        }
    }
}

/// On a cylinder the exact chart-path distance is hypot(r*du, ds), even
/// for nearly same-height sites far from the origin or across a sample knot.
/// Absolute-v subtraction previously made such distances noisy by ~1e-7.
#[test]
fn near_horizontal_spacing_has_no_absolute_chart_cancellation() {
    let i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let s = surface(&i);
    for arc in [30., 49.999999999, 50., 50.000000001, 60., 61.] {
        for delta in [0., 1e-12, 1e-10, 1e-8, 1e-6, 0.01] {
            let a = seeds::Seed {
                u: 0.3,
                v: seeds::extended_v(&s, arc),
                s: arc,
                row: 0,
                auxiliary: arc > s.length(),
                depth: 0.,
            };
            let b = seeds::Seed {
                u: 1.0,
                v: seeds::extended_v(&s, arc + delta),
                s: arc + delta,
                ..a.clone()
            };
            let expected = (0.7 * s.r(arc)).hypot(b.s - a.s);
            for (x, y) in [(&a, &b), (&b, &a)] {
                assert!(
                    (seeds::spacing(&s, x, y) - expected).abs() < 1e-12,
                    "arc={arc} delta={delta}"
                );
            }
        }
    }
}

/// Owner lattice rule: odd columns and exact hex pitch, with two support rows per end.
#[test]
fn reference_lattice_has_nine_columns_without_row_stretching() {
    let mut i = input("cylinder");
    i.params.target_size = 6.5;
    i.params.randomness = 0.;
    let surf = surface(&i);
    let sites = seeds::place(&surf, &i.params, &mut seeds::rng(1)).unwrap();
    let mut rows = BTreeMap::<usize, Vec<&seeds::Seed>>::new();
    for site in &sites {
        rows.entry(site.row).or_default().push(site);
    }
    assert!(rows.values().all(|row| row.len() == 9));
    let pitch = std::f64::consts::TAU * 9. / 9. * 3f64.sqrt() / 2.;
    let ys: Vec<_> = rows.values().map(|row| row[0].s).collect();
    for pair in ys.windows(2) {
        assert!((pair[1] - pair[0] - pitch).abs() < 1e-9);
    }
    assert!(ys.iter().filter(|&&s| s < 0.).count() >= 2);
    assert!(ys.iter().filter(|&&s| s > surf.length()).count() >= 2);
}
