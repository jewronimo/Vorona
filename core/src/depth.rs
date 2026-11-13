//! Configured-first fit/polish with bounded ranked fallback; internal diagnostics, not a CoreOutput contract.
use crate::{
    feasibility,
    model::{CoreInput, Feasibility},
    planes::{self, OptError, Planes},
    seeds, sidecount,
    surface::Surface,
    topology::Topology,
};
use serde::Serialize;

/// Exact search spacing in inches (no cumulative floating-point addition).
pub const STEP: f64 = 0.25;
/// Fixed screening window; native solves never use wall-clock stopping.
pub const SCREEN_CANDIDATES: u64 = 4;
/// At most two fits receive the unchanged guarded penalty solver.
pub const POLISH_CANDIDATES: usize = 2;

#[derive(Debug, Serialize)]
pub struct Attempt {
    pub depth: f64,
    pub feasibility: Feasibility,
    pub topology_error: Option<String>,
    /// Exact fit-only native geometry used in deterministic ranking.
    pub screened: Option<Planes>,
    pub polished: bool,
    /// Full guarded polish success or failure, with best geometry and limit report.
    pub optimizer: Option<Result<Planes, OptError>>,
}

#[derive(Debug, Serialize)]
pub struct Selection {
    pub configured_depth: f64,
    pub selected_depth: Option<f64>,
    pub search_bound: f64,
    pub message: String,
    pub attempts: Vec<Attempt>,
    /// The C3 topology at selected_depth; no C5 geometry is implied.
    pub topology: Option<Topology>,
}

/// Screen up to four configured_depth + k*0.25 inch candidates in ascending order,
/// bounded by the physical bounding-box diagonal; first polish configured depth.
/// The configured fit is checked by native polish immediately; guarded
/// success cannot be improved in depth by screening deeper candidates.
/// On failure, rank the other fits by native squared limit violation, violation
/// count, then shallower depth; polish the best one (two total polish calls).
/// No global-smallest feasible polished depth guarantee: unscreened/unpolished depths
/// may succeed. Limits, fixed target size and the never-worse guard are unchanged.
/// Invalid input and depth-independent C2/C3 failures retain explicit diagnostics.
pub fn select(input: &CoreInput) -> Selection {
    select_ranked(input, planes::screen_depth, planes::polish_depth)
}

fn select_ranked(
    input: &CoreInput,
    mut screen: impl FnMut(
        &Surface,
        &Topology,
        &crate::model::Params,
    ) -> Result<(Planes, Vec<f64>), OptError>,
    mut polish: impl FnMut(
        &Surface,
        &Topology,
        &crate::model::Params,
        Planes,
        Vec<f64>,
    ) -> Result<Planes, OptError>,
) -> Selection {
    let configured = input.params.depth;
    let mut out = Selection {
        configured_depth: configured,
        selected_depth: None,
        search_bound: configured,
        message: String::new(),
        attempts: vec![],
        topology: None,
    };
    if !configured.is_finite() || configured <= 0.0 {
        out.message = "invalid configured depth: expected positive finite inches".into();
        return out;
    }
    let p = &input.params;
    if input.surface.kind != "revolve"
        || p.allowed_sides.is_empty()
        || p.allowed_sides.iter().any(|k| !(4..=9).contains(k))
        || !p.target_size.is_finite()
        || p.target_size <= 0.
        || !p.bevel_cap_deg.is_finite()
        || p.bevel_cap_deg <= 0.
        || p.bevel_cap_deg > 90.
        || !p.min_bounding_square.is_finite()
        || p.min_bounding_square <= 0.
        || !p.size_variation.is_finite()
        || !(0.05..=0.50).contains(&p.size_variation)
    {
        out.message = format!(
            "invalid feasibility parameters; configured depth {configured}, selected depth none"
        );
        return out;
    }
    let surface = match Surface::from_profile(&input.surface.profile) {
        Ok(s) => s,
        Err(e) => {
            out.message =
                format!("invalid surface: {e}; configured depth {configured}, selected depth none");
            return out;
        }
    };
    let radius = input
        .surface
        .profile
        .iter()
        .map(|p| p[0])
        .fold(0.0, f64::max);
    let y = surface.y_range();
    let diagonal = (2.0 * radius).hypot(2.0 * radius).hypot(y[1] - y[0]);
    out.search_bound = configured.max(diagonal);
    let steps = ((out.search_bound - configured) / STEP).floor();
    // Beyond this range f64 cannot preserve a distinct quarter-inch grid.
    if !out.search_bound.is_finite()
        || steps >= (1_u64 << 50) as f64
        || configured + STEP == configured
        || out.search_bound + STEP == out.search_bound
    {
        out.message = format!("unrepresentable quarter-inch search grid; configured depth {configured}, selected depth none");
        return out;
    }
    // C3 chart placement/repair is depth-independent. Preserve its exact native
    // graph and retarget only deterministic row offsets for later candidates.
    let mut chart: Option<Topology> = None;
    let mut candidates: Vec<(usize, Topology, Vec<f64>)> = vec![];
    let mut early_polishes = 0;
    for k in 0..=(steps as u64).min(SCREEN_CANDIDATES - 1) {
        let mut p = input.params.clone();
        p.depth = configured + k as f64 * STEP;
        let f = feasibility::check(&surface, &p);
        let mut attempt = Attempt {
            depth: p.depth,
            feasibility: f,
            topology_error: None,
            screened: None,
            polished: false,
            optimizer: None,
        };
        if !attempt.feasibility.feasible {
            out.message = format!("depth-independent C2 failure: {}; configured depth {configured}, selected depth none", feasibility::message(&surface, &attempt.feasibility, &p));
            out.attempts.push(attempt);
            return out;
        }
        let topology_result = if let Some(base) = &chart {
            let mut t = base.clone();
            seeds::assign_depths(&mut t.tiles, &p, &mut seeds::rng(p.seed));
            Ok(t)
        } else {
            sidecount::build(&surface, &p)
        };
        let topology = match topology_result {
            Ok(t) => t,
            Err(e) => {
                // C3 uses depth only to assign seed offsets, never to repair its chart graph.
                out.message = format!("depth-independent C3 failure: {e}; configured depth {configured}, selected depth none");
                attempt.topology_error = Some(e);
                out.attempts.push(attempt);
                return out;
            }
        };
        if chart.is_none() {
            chart = Some(topology.clone());
        }
        match screen(&surface, &topology, &p) {
            Ok((fit, x)) => {
                // No depth smaller than configured belongs to this search. Only
                // short-circuit on an actual native guarded success, never a
                // ranking score alone. Retain both fit and polish evidence.
                if k == 0 {
                    let result = polish(&surface, &topology, &p, fit.clone(), x);
                    let success = result.is_ok();
                    early_polishes += 1;
                    attempt.screened = Some(fit);
                    attempt.polished = true;
                    attempt.optimizer = Some(result);
                    out.attempts.push(attempt);
                    if success {
                        out.selected_depth = Some(p.depth);
                        out.topology = Some(topology);
                        out.message = format!("configured depth {configured} in; selected depth {configured} in; configured fit passed native guarded polish; deeper screening unnecessary. No depths below configured searched; no global-minimum claim");
                        return out;
                    }
                    continue;
                }
                candidates.push((out.attempts.len(), topology, x));
                attempt.screened = Some(fit);
            }
            Err(e) => attempt.optimizer = Some(Err(e)),
        }
        out.attempts.push(attempt);
    }
    candidates.sort_by(|a, b| {
        let ar = &out.attempts[a.0].screened.as_ref().unwrap().report;
        let br = &out.attempts[b.0].screened.as_ref().unwrap().report;
        ar.limit_violation_sum_sq
            .total_cmp(&br.limit_violation_sum_sq)
            .then(ar.violated.len().cmp(&br.violated.len()))
            .then(a.0.cmp(&b.0))
    });
    for (index, topology, x) in candidates
        .into_iter()
        .take(POLISH_CANDIDATES - early_polishes)
    {
        let mut p = input.params.clone();
        p.depth = out.attempts[index].depth;
        let fit = out.attempts[index].screened.as_ref().unwrap().clone();
        let result = polish(&surface, &topology, &p, fit, x);
        let success = result.is_ok();
        out.attempts[index].polished = true;
        out.attempts[index].optimizer = Some(result);
        if success && out.selected_depth.is_none_or(|d| p.depth < d) {
            out.selected_depth = Some(p.depth);
            out.topology = Some(topology);
        }
    }
    let polished = out.attempts.iter().filter(|a| a.polished).count();
    out.message = format!("configured depth {configured} in; selected depth {:?} in; fit-screened {} ascending quarter-inch candidates; polished {polished} candidates (configured first, then best ranked fallback). Heuristic bounded shortlist: NOT a guarantee of globally smallest feasible polished depth; unsuccessful search is not geometric impossibility", out.selected_depth, out.attempts.len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> CoreInput {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap()
    }
    fn geometry(score: f64) -> Planes {
        Planes {
            planes: vec![],
            corners: vec![],
            polygons: vec![],
            whole_tiles: vec![],
            report: planes::OptReport {
                limit_violation_sum_sq: score,
                iterations: 0,
                corner_distance: planes::DistanceStats {
                    min: 0.,
                    max: 0.,
                    median: 0.,
                },
                reflex_corners: 0,
                violated: vec![],
            },
        }
    }
    #[test]
    fn configured_fit_requires_guarded_success_before_stopping() {
        for accept in [true, false] {
            let i = input();
            let mut screens = 0;
            let mut polishes = 0;
            let out = select_ranked(
                &i,
                |_, _, _| {
                    screens += 1;
                    Ok((geometry(1.), vec![]))
                },
                |_, _, _, g, _| {
                    polishes += 1;
                    if accept {
                        Ok(g)
                    } else {
                        Err(OptError {
                            message: "guard rejected".into(),
                            best: Some(Box::new(g)),
                        })
                    }
                },
            );
            assert_eq!(screens, if accept { 1 } else { 4 });
            assert_eq!(polishes, if accept { 1 } else { 2 });
            assert_eq!(
                out.selected_depth,
                if accept { Some(i.params.depth) } else { None }
            );
            assert!(out.attempts[0].polished);
            assert!(out.attempts[0].screened.is_some());
        }
    }
    #[test]
    fn configured_failure_ranks_remaining_screen_without_refitting_or_minimality_claim() {
        let i = input();
        let mut screened = vec![];
        let mut polished = vec![];
        let out = select_ranked(
            &i,
            |surface, t, p| {
                assert_eq!(*t, sidecount::build(surface, p).unwrap());
                assert_eq!(p.target_size, i.params.target_size);
                assert_eq!(p.size_variation, i.params.size_variation);
                screened.push(p.depth);
                Ok((geometry(10. - p.depth), vec![p.depth]))
            },
            |_, _, p, baseline, x| {
                assert_eq!(x, vec![p.depth]);
                assert_eq!(baseline.report.limit_violation_sum_sq, 10. - p.depth);
                polished.push(p.depth);
                if polished.len() == 1 {
                    Err(OptError {
                        message: "synthetic failure".into(),
                        best: Some(Box::new(baseline)),
                    })
                } else {
                    Ok(baseline)
                }
            },
        );
        assert_eq!(
            screened,
            (0..4).map(|k| 1.5 + k as f64 * STEP).collect::<Vec<_>>()
        );
        assert_eq!(polished, vec![1.5, 2.25]);
        assert_eq!(out.selected_depth, Some(2.25));
        assert!(out.message.contains("NOT a guarantee"));
        assert_eq!(out.attempts.iter().filter(|a| a.polished).count(), 2);
        assert!(out.attempts.iter().all(|a| a.screened.is_some()));
    }
    #[test]
    #[ignore = "native nonlinear solver equivalence control"]
    fn screened_fit_reuse_is_identical_to_direct_production_optimize() {
        let i = input();
        let surface = Surface::from_profile(&i.surface.profile).unwrap();
        let t = sidecount::build(&surface, &i.params).unwrap();
        let (fit, x) = planes::screen_depth(&surface, &t, &i.params).unwrap();
        let reused = planes::polish_depth(&surface, &t, &i.params, fit, x);
        let direct = planes::optimize(&surface, &t, &i.params, &mut seeds::rng(i.params.seed));
        assert_eq!(
            serde_json::to_string(&reused).unwrap(),
            serde_json::to_string(&direct).unwrap()
        );
    }
    #[test]
    fn rank_ties_choose_shallower_and_physical_bound_clips_window() {
        let mut i = input();
        i.params.depth = 100.;
        let out = select_ranked(
            &i,
            |_, _, _| Ok((geometry(1.), vec![])),
            |_, _, _, g, _| Ok(g),
        );
        assert_eq!(out.attempts.len(), 1);
        assert_eq!(out.selected_depth, Some(100.));
        i.params.depth = 1.5;
        let mut chosen = vec![];
        let out = select_ranked(
            &i,
            |_, _, _| Ok((geometry(1.), vec![])),
            |_, _, p, g, _| {
                chosen.push(p.depth);
                Ok(g)
            },
        );
        assert_eq!(chosen, vec![1.5]);
        assert_eq!(out.selected_depth, Some(1.5));
    }
}
