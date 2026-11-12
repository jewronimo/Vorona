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
    for row in 0..=t.tiles.iter().map(|x| x.row).max().unwrap() {
        let sites: Vec<_> = t.tiles.iter().filter(|x| x.row == row).collect();
        assert_eq!(sites.len(), f.columns[0] as usize);
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
fn seeds_one_to_five_change_adjacency_not_just_positions() {
    for randomness in [0.5, 1.0] {
        let mut graphs = Vec::new();
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
            assert!(
                randomness < 1.0 || !graphs.contains(&t.neighbors),
                "seed {seed}, randomness {randomness}: repeated adjacency"
            );
            graphs.push(t.neighbors);
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
