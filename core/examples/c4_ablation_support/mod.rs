use std::collections::BTreeSet;
use vstone_core::{
    model::Params,
    planes::{self, FitAndPolish, OptError},
    seeds, sidecount,
    surface::Surface,
    topology::Topology,
};
fn conflicts(t: &Topology, c: &[usize]) -> usize {
    t.neighbors
        .iter()
        .enumerate()
        .map(|(i, ns)| ns.iter().filter(|&&j| j > i && c[i] == c[j]).count())
        .sum()
}
fn rank(d: &FitAndPolish) -> (usize, usize) {
    let g = match &d.result {
        Ok(g) => g,
        Err(e) => e.best.as_deref().unwrap(),
    };
    (
        g.report
            .violated
            .iter()
            .map(|(i, _)| *i)
            .collect::<BTreeSet<_>>()
            .len(),
        g.report.violated.len(),
    )
}
pub fn intervene(
    s: &Surface,
    p: &Params,
    t: &mut Topology,
    arm: &str,
) -> (serde_json::Value, Option<Result<FitAndPolish, OptError>>) {
    let original = t.clone();
    let row: Vec<_> = t.tiles.iter().map(|x| x.row % 3).collect();
    let before = conflicts(t, &row);
    if arm == "color" {
        let mut c = vec![usize::MAX; t.tiles.len()];
        // Deterministic DSATUR; saturation then degree, lowest id wins ties.
        for _ in 0..c.len() {
            let i = (0..c.len())
                .filter(|&i| c[i] == usize::MAX)
                .max_by_key(|&i| {
                    (
                        t.neighbors[i]
                            .iter()
                            .filter_map(|&j| (c[j] != usize::MAX).then_some(c[j]))
                            .collect::<BTreeSet<_>>()
                            .len(),
                        t.degree(i),
                        usize::MAX - i,
                    )
                })
                .unwrap();
            c[i] = (0..3)
                .min_by_key(|&k| {
                    (
                        t.neighbors[i].iter().filter(|&&j| c[j] == k).count(),
                        usize::from(k != row[i]),
                        k,
                    )
                })
                .unwrap();
        }
        let greedy = conflicts(t, &c);
        let mut sweeps = 0;
        for _ in 0..64 {
            let mut changed = false;
            for i in 0..c.len() {
                let count = |k| t.neighbors[i].iter().filter(|&&j| c[j] == k).count();
                let k = (0..3).min_by_key(|&k| (count(k), k)).unwrap();
                if count(k) < count(c[i]) {
                    c[i] = k;
                    changed = true;
                }
            }
            sweeps += 1;
            if !changed {
                break;
            }
        }
        for (i, x) in t.tiles.iter_mut().enumerate() {
            x.depth = c[i] as f64 * p.depth / 2.;
        }
        return (
            serde_json::json!({"equal_edges_before":before,"equal_edges_greedy":greedy,"equal_edges_after":conflicts(t,&c),"fixup_sweeps":sweeps,"colors":c,"topology_unchanged":t.neighbors==original.neighbors,"spacing":sidecount::spacing_residual(s,p,t)}),
            None,
        );
    }
    if arm == "baseline" {
        return (
            serde_json::json!({"equal_edges_before":before,"equal_edges_after":before,"spacing":sidecount::spacing_residual(s,p,t)}),
            None,
        );
    }
    assert_eq!(arm, "motion");
    let defects: BTreeSet<_> = (0..t.tiles.len())
        .filter(|&i| !t.boundary[i] && matches!(t.degree(i), 5 | 7))
        .collect();
    let eligible: Vec<_> = (0..t.tiles.len())
        .filter(|&i| {
            sidecount::interior(t, i, s)
                && (defects.contains(&i) || t.neighbors[i].iter().any(|j| defects.contains(j)))
        })
        .collect();
    let mut best = planes::fit_and_polish(s, t, p).unwrap();
    let mut trials = vec![serde_json::json!({"fraction":0,"rank":rank(&best)})];
    let mut selected = 0.;
    // A bounded one-dimensional spacing-relaxation field, sampled both signs.
    // Candidate selection uses the unchanged native final-limit report, not surrogate acceptance.
    for initial_fraction in [0.02, -0.02, 0.05, -0.05] {
        for halvings in 0..13 {
            let fraction = initial_fraction / 2f64.powi(halvings);
            let mut sites = original.tiles.clone();
            for &i in &eligible {
                let x = &original.tiles[i];
                let mut du = 0.;
                let mut ds = 0.;
                for &j in &original.neighbors[i] {
                    let y = &original.tiles[j];
                    let a = (y.u - x.u + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                        - std::f64::consts::PI;
                    let b = y.s - x.s;
                    let distance = seeds::spacing(s, x, y);
                    let force = (distance - p.target_size) / distance;
                    du += a * s.r(x.s) * force;
                    ds += b * force;
                }
                let norm = du.hypot(ds);
                if norm > 1e-12 {
                    sites[i].u = (x.u + fraction * p.target_size * du / norm / s.r(x.s))
                        .rem_euclid(std::f64::consts::TAU);
                    sites[i].s = x.s + fraction * p.target_size * ds / norm;
                    sites[i].v = seeds::extended_v(s, sites[i].s);
                }
            }
            let candidate = Topology::build(sites).unwrap();
            let spacing = sidecount::spacing_residual(s, p, &candidate);
            let valid = candidate.neighbors == original.neighbors
                && candidate.corners == original.corners
                && spacing.0 <= 1e-18
                && eligible
                    .iter()
                    .all(|&i| sidecount::interior(&candidate, i, s));
            if !valid {
                trials
                    .push(serde_json::json!({"fraction":fraction,"valid":false,"spacing":spacing}));
                continue;
            }
            let d = planes::fit_and_polish(s, &candidate, p).unwrap();
            let score = rank(&d);
            trials.push(serde_json::json!({"fraction":fraction,"valid":true,"rank":score,"fit_seconds":d.fit_seconds,"polish_seconds":d.polish_seconds,"spacing":spacing}));
            if score < rank(&best) {
                *t = candidate;
                best = d;
                selected = fraction;
            }
            break;
        }
    }
    let movements:Vec<_>=eligible.iter().map(|&i|serde_json::json!({"tile":i,"surface_spacing_inches":seeds::spacing(s,&original.tiles[i],&t.tiles[i]),"delta_s_inches":t.tiles[i].s-original.tiles[i].s,"delta_u_r_inches":((t.tiles[i].u-original.tiles[i].u+std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)-std::f64::consts::PI)*s.r(original.tiles[i].s)})).collect();
    (
        serde_json::json!({"equal_edges_before":before,"equal_edges_after":before,"eligible":eligible,"component_vector_bound_inches":0.05*p.target_size,"selected_fraction":selected,"trials":trials,"movements":movements,"topology_unchanged":t.neighbors==original.neighbors,"spacing":sidecount::spacing_residual(s,p,t)}),
        Some(Ok(best)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_is_deterministic_and_preserves_original_sites_and_graph() {
        for profile in ["vase_bulge", "cone_9_to_6"] {
            let mut input: vstone_core::model::CoreInput = serde_json::from_str(include_str!(
                "../../../tests/fixtures/core_input_cylinder.json"
            ))
            .unwrap();
            input.surface.profile = serde_json::from_str(
                &std::fs::read_to_string(format!(
                    "{}/../tests/fixtures/profiles/{profile}.json",
                    env!("CARGO_MANIFEST_DIR")
                ))
                .unwrap(),
            )
            .unwrap();
            input.params.allowed_sides = vec![5, 6, 7];
            input.params.seed = 1;
            input.params.randomness = 0.;
            let s = Surface::from_profile(&input.surface.profile).unwrap();
            let original = sidecount::build(&s, &input.params).unwrap();
            let mut a = original.clone();
            let mut b = original.clone();
            let (ma, _) = intervene(&s, &input.params, &mut a, "color");
            let (mb, _) = intervene(&s, &input.params, &mut b, "color");
            assert_eq!(ma, mb);
            assert_eq!(a, b);
            assert_eq!(a.neighbors, original.neighbors);
            assert_eq!(a.corners, original.corners);
            assert!(
                ma["equal_edges_after"].as_u64().unwrap()
                    <= ma["equal_edges_greedy"].as_u64().unwrap()
            );
            for (x, y) in a.tiles.iter().zip(&original.tiles) {
                assert_eq!(
                    (x.u, x.v, x.s, x.row, x.auxiliary),
                    (y.u, y.v, y.s, y.row, y.auxiliary)
                );
                assert!([0., input.params.depth / 2., input.params.depth].contains(&x.depth));
            }
        }
    }
}
