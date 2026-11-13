#![allow(dead_code)]
//! Fixed-budget deterministic repair of the periodic Delaunay sites.
use crate::{
    feasibility,
    model::Params,
    seeds::{self, Seed},
    surface::Surface,
    topology::Topology,
};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

const PASSES: usize = 512;

/// Build topology from inch-valued geometry and numeric contract parameters.
/// Returns a named-tile diagnostic on infeasibility, invalid input, or repair cap.
/// Does not produce pieces or claim downstream geometric validation.
pub fn build(surface: &Surface, p: &Params) -> Result<Topology, String> {
    build_with_jitter(surface, p, 0.1)
}

/// Diagnostic topology build with explicit local positional jitter cap (fraction).
/// Returns the same infeasibility/repair errors as `build`.
pub fn build_with_jitter(surface: &Surface, p: &Params, cap: f64) -> Result<Topology, String> {
    // Repair has stochastic proposal ordering even when placement is symmetric.
    // Canonicalize the entire stream, not only the initial displacement.
    let mut rng = seeds::rng(if p.randomness == 0. { 1 } else { p.seed });
    let sites = seeds::place_with_jitter(surface, p, &mut rng, cap)?;
    repair(surface, p, sites, &mut rng)
}

/// True only for ordinary rows strictly between the two physical rims.
/// Auxiliary and physical rim rows are excluded, not their neighboring rows.
/// A triangulation hull flag is not a physical-rim exemption: an ordinary
/// interior site must still satisfy degrees, averages and edge bands.
pub fn interior(t: &Topology, i: usize, surface: &Surface) -> bool {
    !t.tiles[i].auxiliary && t.tiles[i].s > 1e-9 && t.tiles[i].s < surface.length() - 1e-9
}

/// Permitted edge variation: exactly five percentage points extra only when
/// an endpoint has degree five or seven. No allowance for other degrees.
pub fn edge_variation(p: &Params, t: &Topology, i: usize, j: usize) -> f64 {
    p.size_variation
        + if matches!(t.degree(i), 5 | 7) || matches!(t.degree(j), 5 | 7) {
            0.05
        } else {
            0.0
        }
}

fn average_ratio(surface: &Surface, p: &Params, t: &Topology, i: usize) -> f64 {
    // All incident spacings contribute to an interior tile's average, including
    // its neighbor on a physical rim. Only the tile itself is exempt at a rim.
    t.neighbors[i]
        .iter()
        .map(|&j| seeds::spacing(surface, &t.tiles[i], &t.tiles[j]) / p.target_size)
        .sum::<f64>()
        / t.degree(i) as f64
}

fn violations(surface: &Surface, p: &Params, t: &Topology) -> Vec<usize> {
    (0..t.tiles.len())
        .filter(|&i| {
            interior(t, i, surface)
                && (!p.allowed_sides.contains(&(t.degree(i) as u32))
                    || (average_ratio(surface, p, t, i) - 1.0).abs() > p.size_variation + 1e-9
                    || t.neighbors[i].iter().any(|&j| {
                        interior(t, j, surface)
                            && (seeds::spacing(surface, &t.tiles[i], &t.tiles[j]) / p.target_size
                                - 1.0)
                                .abs()
                                > edge_variation(p, t, i, j) + 1e-9
                    }))
        })
        .collect()
}

/// Squared dimensionless edge AND per-tile average band residual.
/// Edge extrema are reported separately; rim exclusion is unchanged.
pub fn spacing_residual(surface: &Surface, p: &Params, t: &Topology) -> (f64, f64, f64) {
    let mut residual = 0.0;
    let mut minimum = f64::INFINITY;
    let mut maximum: f64 = 0.0;
    for i in 0..t.tiles.len() {
        if !interior(t, i, surface) {
            continue;
        }
        residual += ((average_ratio(surface, p, t, i) - 1.0).abs() - p.size_variation)
            .max(0.0)
            .powi(2);
        for &j in &t.neighbors[i] {
            if j <= i || !interior(t, j, surface) {
                continue;
            }
            let ratio = seeds::spacing(surface, &t.tiles[i], &t.tiles[j]) / p.target_size;
            minimum = minimum.min(ratio);
            maximum = maximum.max(ratio);
            residual += ((ratio - 1.0).abs() - edge_variation(p, t, i, j))
                .max(0.0)
                .powi(2);
        }
    }
    (
        residual,
        if minimum.is_finite() { minimum } else { 0.0 },
        maximum,
    )
}

/// Internal convergence evidence, not a CoreOutput/JSON contract extension.
#[derive(Clone, Debug)]
pub struct RepairReport {
    pub initial_residual: f64,
    pub final_residual: f64,
    pub initial_score: f64,
    pub final_score: f64,
    pub accepted_steps: usize,
    pub iterations: usize,
}

fn defect_penalty(surface: &Surface, f: &crate::model::Feasibility, t: &Topology) -> usize {
    let mut rows = std::collections::BTreeMap::<usize, usize>::new();
    for site in &t.tiles {
        if !site.auxiliary {
            *rows.entry(site.row).or_default() += 1;
        }
    }
    let rows: Vec<_> = rows.into_iter().collect();
    let transitions: Vec<_> = rows
        .windows(2)
        .filter(|pair| pair[0].1 != pair[1].1)
        .map(|pair| pair[1].0)
        .collect();
    let misplaced = if f.column_changes > 0 && f.pentagons_needed == 0 {
        (0..t.tiles.len())
            .filter(|&i| interior(t, i, surface) && matches!(t.degree(i), 5 | 7))
            .filter(|&i| {
                !transitions
                    .iter()
                    .any(|&row| t.tiles[i].row.abs_diff(row) <= 1)
                    || !t.neighbors[i]
                        .iter()
                        .any(|&j| interior(t, j, surface) && t.degree(j) == 12 - t.degree(i))
            })
            .count()
    } else {
        0
    };

    let count = |degree| {
        (0..t.tiles.len())
            .filter(|&i| interior(t, i, surface) && t.degree(i) == degree)
            .count()
    };
    if f.pentagons_needed > 0 {
        count(5)
            .abs_diff(f.pentagons_needed as usize)
            .saturating_sub(1)
    } else {
        misplaced
            + (f.column_changes as usize).saturating_sub(count(5))
            + (f.column_changes.max(f.heptagons_needed) as usize).saturating_sub(count(7))
    }
}

fn score(surface: &Surface, p: &Params, f: &crate::model::Feasibility, t: &Topology) -> f64 {
    let side_distance: usize = (0..t.tiles.len())
        .filter(|&i| interior(t, i, surface))
        .map(|i| {
            p.allowed_sides
                .iter()
                .map(|&k| t.degree(i).abs_diff(k as usize))
                .min()
                .unwrap_or(100)
        })
        .sum();
    side_distance as f64
        + defect_penalty(surface, f, t) as f64
        + 100.0 * optimization_residual(surface, p, t)
}

// Optimize toward a genuine interior band, not a tolerance-sized inset.
// For each checked constraint with nominal excess e, the optimized residual
// contains max(e + m, 0)^2. Thus optimized residual < m^2 certifies EVERY
// nominal spacing constraint (including incident averages) is strictly legal.
// The previous m=1e-7 required a global squared residual below 1e-14 merely
// to certify acceptance: cap512 can approach that boundary asymptotically.
// A 1% share of the available variation gives a finite feasibility gap without
// changing ANY terminal bound, degree allowance, or acceptance epsilon.
fn optimization_margin(p: &Params) -> f64 {
    0.01 * p.size_variation
}

fn optimization_residual(surface: &Surface, p: &Params, t: &Topology) -> f64 {
    let mut tightened = p.clone();
    tightened.size_variation -= optimization_margin(p);
    spacing_residual(surface, &tightened, t).0
}

// Only incident edge hinges and averages at this site and its neighbors can
// change under a one-site probe. Unaffected terms cancel in the derivative.
// Evaluating them formerly made each gradient O(N*E) rather than O(E*degree).
fn local_optimization_residual(surface: &Surface, p: &Params, t: &Topology, moved: usize) -> f64 {
    let mut tight = p.clone();
    tight.size_variation -= optimization_margin(p);
    let mut residual = 0.;
    for i in std::iter::once(moved).chain(t.neighbors[moved].iter().copied()) {
        if interior(t, i, surface) {
            residual += ((average_ratio(surface, &tight, t, i) - 1.).abs() - tight.size_variation)
                .max(0.)
                .powi(2);
        }
    }
    if interior(t, moved, surface) {
        for &j in &t.neighbors[moved] {
            if interior(t, j, surface) {
                let ratio = seeds::spacing(surface, &t.tiles[moved], &t.tiles[j]) / p.target_size;
                residual += ((ratio - 1.).abs() - edge_variation(&tight, t, moved, j))
                    .max(0.)
                    .powi(2);
            }
        }
    }
    residual
}

// Differentiate the ACTUAL band objective with the current Delaunay chamber
// held fixed. In particular every neighbor contributes to the average term;
// endpoint-only springs omitted those derivatives and stalled near tight bands.
// Trial endpoints are always retriangulated before acceptance.
fn metric_step(
    surface: &Surface,
    p: &Params,
    f: &crate::model::Feasibility,
    t: &Topology,
) -> Option<Topology> {
    let mut gradient = vec![[0.; 2]; t.tiles.len()];
    // Stay well below the optimization inset. Larger probes straddle
    // inactive hinges and bias the gradient precisely where cleanup is needed.
    // A fixed probe also smooths the squared hinge into a different objective
    // near convergence. Shrink with the remaining residual, staying well below
    // active excesses; the floor is below the unchanged acceptance tolerance.
    let h = p.target_size * (optimization_residual(surface, p, t).sqrt() * 1e-3).clamp(1e-12, 1e-8);
    let mut probe = t.clone();
    for (i, g) in gradient.iter_mut().enumerate() {
        if t.tiles[i].auxiliary {
            continue;
        }
        for (axis, derivative) in g.iter_mut().enumerate() {
            // Ordinary sites can move in arc and azimuth. Physical rim rows
            // remain fixed in arc; their azimuth still influences averages.
            if axis == 1 && !interior(t, i, surface) {
                continue;
            }
            let mut values = [0.; 2];
            for (k, sign) in [-1., 1.].into_iter().enumerate() {
                probe.tiles[i] = t.tiles[i].clone();
                if axis == 0 {
                    probe.tiles[i].u = (t.tiles[i].u + sign * h / surface.r(t.tiles[i].s))
                        .rem_euclid(std::f64::consts::TAU);
                } else {
                    let arc = (t.tiles[i].s + sign * h).clamp(1e-8, surface.length() - 1e-8);
                    probe.tiles[i].s = arc;
                    probe.tiles[i].v = seeds::extended_v(surface, arc);
                }
                values[k] = local_optimization_residual(surface, p, &probe, i);
            }
            *derivative = (values[1] - values[0]) / (2. * h);
            probe.tiles[i] = t.tiles[i].clone();
        }
    }
    let largest = gradient.iter().map(|g| g[0].hypot(g[1])).fold(0., f64::max);
    if largest < 1e-18 {
        return None;
    }
    let before = score(surface, p, f, t);
    // The old eight-entry search had a minimum displacement of target/640
    // (0.01015625 inches at the reference target). Every proposal could then
    // overshoot a small, nonzero violation and falsely report a plateau.
    // Fixed-budget backtracking resolves the inset without widening acceptance.
    let norm_sq: f64 = gradient.iter().map(|g| g[0] * g[0] + g[1] * g[1]).sum();
    for backtrack in 0..40 {
        let fraction = 0.2 * 0.5_f64.powi(backtrack);
        let mut sites = t.tiles.clone();
        for (i, site) in sites.iter_mut().enumerate() {
            let step = fraction * p.target_size / largest;
            site.u = (site.u - step * gradient[i][0] / surface.r(site.s))
                .rem_euclid(std::f64::consts::TAU);
            if interior(t, i, surface) {
                site.s = (site.s - step * gradient[i][1]).clamp(1e-8, surface.length() - 1e-8);
                site.v = seeds::extended_v(surface, site.s);
            }
        }
        if let Ok(candidate) = Topology::build(sites) {
            let step = fraction * p.target_size / largest;
            // Sufficient decrease avoids accepting numerically negligible
            // progress while preserving the complete retriangulated objective.
            if score(surface, p, f, &candidate) < before - 1e-4 * step * 100. * norm_sq {
                return Some(candidate);
            }
        }
    }
    None
}

fn discrete_penalty(
    surface: &Surface,
    p: &Params,
    f: &crate::model::Feasibility,
    t: &Topology,
) -> f64 {
    score(surface, p, f, t) - 100. * optimization_residual(surface, p, t)
}

// A flip can improve the integer defect budget while temporarily stretching
// its new diagonal. Evaluate that flip together with bounded metric cleanup;
// rejecting the unrelaxed intermediate state traps the search in one chamber.
fn relax_flip(
    surface: &Surface,
    p: &Params,
    f: &crate::model::Feasibility,
    mut current: Topology,
) -> Topology {
    for _ in 0..96 {
        if let Some(candidate) = metric_step(surface, p, f, &current) {
            current = candidate;
            continue;
        }
        let before = score(surface, p, f, &current);
        let mut best = None;
        let mut best_score = before;
        for gain in [0.25, 0.6] {
            let mut sites = current.tiles.clone();
            for (i, site) in current.tiles.iter().enumerate() {
                if !interior(&current, i, surface) {
                    continue;
                }
                let mut force = [0.0_f64; 2];
                let average = average_ratio(surface, p, &current, i);
                let excess = average
                    - average.clamp(
                        1.0 - p.size_variation + optimization_margin(p),
                        1.0 + p.size_variation - optimization_margin(p),
                    );
                for &j in &current.neighbors[i] {
                    let other = &current.tiles[j];
                    let du = (other.u - site.u + std::f64::consts::PI)
                        .rem_euclid(std::f64::consts::TAU)
                        - std::f64::consts::PI;
                    let length = seeds::spacing(surface, site, other);
                    let edge_excess = if interior(&current, j, surface) {
                        let band = edge_variation(p, &current, i, j) - optimization_margin(p);
                        length
                            - length
                                .clamp(p.target_size * (1.0 - band), p.target_size * (1.0 + band))
                    } else {
                        0.0
                    };
                    let scale = edge_excess / length.max(1e-9) + excess / average.max(1e-9);
                    force[0] += scale * du * surface.r((site.s + other.s) * 0.5);
                    force[1] += scale * (other.s - site.s);
                }
                let step =
                    gain * (0.1 * p.target_size / force[0].hypot(force[1]).max(1e-9)).min(1.0);
                let arc = (site.s + step * force[1]).clamp(1e-8, surface.length() - 1e-8);
                sites[i].s = arc;
                sites[i].v = seeds::extended_v(surface, arc);
                sites[i].u =
                    (site.u + step * force[0] / surface.r(arc)).rem_euclid(std::f64::consts::TAU);
            }
            if let Ok(candidate) = Topology::build(sites) {
                let value = score(surface, p, f, &candidate);
                if value < best_score {
                    best_score = value;
                    best = Some(candidate);
                }
            }
        }
        match best {
            Some(candidate) => current = candidate,
            None => break,
        }
    }
    current
}

/// Repair inch-valued sites with a single caller-owned random stream.
/// Returns an explicit failure rather than a partially valid topology at the cap.
pub fn repair(
    surface: &Surface,
    p: &Params,
    sites: Vec<Seed>,
    rng: &mut ChaCha8Rng,
) -> Result<Topology, String> {
    repair_with_report(surface, p, sites, rng).map(|(topology, _)| topology)
}

/// Internal initial/final residual and iteration evidence; units are inches.
/// Returns named tiles and edge ratios on capped failure, never silent success.
pub fn repair_with_report(
    surface: &Surface,
    p: &Params,
    sites: Vec<Seed>,
    rng: &mut ChaCha8Rng,
) -> Result<(Topology, RepairReport), String> {
    if !p.depth.is_finite() || p.depth <= 0.0 {
        return Err("depth must be positive and finite".into());
    }
    if p.allowed_sides.is_empty() {
        return Err("empty allowed side set".into());
    }
    let f = feasibility::check(surface, p);
    let mut current = Topology::build(sites)?;
    let mut report = RepairReport {
        initial_residual: spacing_residual(surface, p, &current).0,
        final_residual: 0.0,
        initial_score: score(surface, p, &f, &current),
        final_score: 0.0,
        accepted_steps: 0,
        iterations: 0,
    };
    for pass in 0..PASSES {
        report.iterations = pass;
        report.final_residual = spacing_residual(surface, p, &current).0;
        report.final_score = score(surface, p, &f, &current);
        let mut bad = violations(surface, p, &current);
        if bad.is_empty() && defect_penalty(surface, &f, &current) == 0 {
            seeds::assign_depths(&mut current.tiles, p, rng);
            return Ok((current, report));
        }
        if bad.is_empty() {
            // Rim angular moves can transfer one unit of defect to the open
            // boundary, as required by the annulus Euler balance. Keep rim s
            // fixed and retain exactly the same physical-rim exclusions.
            bad = (0..current.tiles.len())
                .filter(|&i| !current.tiles[i].auxiliary)
                .collect();
        }
        // Rotate even when bad sites exist: a fixed prefix must not starve other rows.
        if !bad.is_empty() {
            let offset = (pass * 8) % bad.len();
            bad.rotate_left(offset);
        }
        let mut best_score = score(surface, p, &f, &current);
        let mut best = metric_step(surface, p, &f, &current);
        if let Some(candidate) = &best {
            best_score = score(surface, p, &f, candidate);
        }
        // Continuous spring descent across ALL invalid sites, not a binary count.
        // Every proposal is retriangulated to retain the Delaunay contract.
        for gain in [0.15, 0.35, 0.65] {
            let mut moved = current.tiles.clone();
            for &i in &bad {
                let site = &current.tiles[i];
                let mut force = [0.0, 0.0];
                for &j in &current.neighbors[i] {
                    if !interior(&current, j, surface) {
                        continue;
                    }
                    let other = &current.tiles[j];
                    let du = (other.u - site.u + std::f64::consts::PI)
                        .rem_euclid(std::f64::consts::TAU)
                        - std::f64::consts::PI;
                    let length = seeds::spacing(surface, site, other);
                    let target = length.clamp(
                        p.target_size
                            * (1.0 - edge_variation(p, &current, i, j) + optimization_margin(p)),
                        p.target_size
                            * (1.0 + edge_variation(p, &current, i, j) - optimization_margin(p)),
                    );
                    let average = average_ratio(surface, p, &current, i);
                    let average_target = average.clamp(
                        1.0 - p.size_variation + optimization_margin(p),
                        1.0 + p.size_variation - optimization_margin(p),
                    );
                    let scale = (length - target) / length.max(1e-9)
                        + (average - average_target) / average.max(1e-9);
                    force[0] += scale * du * surface.r((other.s + site.s) * 0.5);
                    force[1] += scale * (other.s - site.s);
                }
                let norm = force[0].hypot(force[1]);
                let step = gain * (p.target_size * 0.15 / norm.max(1e-9)).min(1.0);
                // Defect-only proposals also visit physical rim sites. Their
                // angular motion is legal, but spring descent must not pull
                // them off the rim and turn them into ordinary interior sites.
                let s = if site.s <= 1e-9 || site.s >= surface.length() - 1e-9 {
                    site.s
                } else {
                    (site.s + step * force[1]).clamp(1e-8, surface.length() - 1e-8)
                };
                moved[i].s = s;
                moved[i].v = seeds::extended_v(surface, s);
                moved[i].u =
                    (site.u + step * force[0] / surface.r(s)).rem_euclid(std::f64::consts::TAU);
            }
            if let Ok(candidate) = Topology::build(moved) {
                let value = score(surface, p, &f, &candidate);
                if value < best_score {
                    best_score = value;
                    best = Some(candidate);
                }
            }
        }
        // A single-site walk cannot undo coherent row rotations, and can be
        // trapped on the wrong side of a count-change dislocation. Move a whole
        // row through fixed phase samples, then clean up the resulting chamber.
        let rows: std::collections::BTreeSet<_> =
            bad.iter().map(|&i| current.tiles[i].row).collect();
        for row in rows.into_iter().take(4) {
            let count = current.tiles.iter().filter(|s| s.row == row).count();
            for fraction in [-0.5, -0.25, -0.125, 0.125, 0.25, 0.5] {
                let mut moved = current.tiles.clone();
                for site in moved.iter_mut().filter(|s| s.row == row) {
                    site.u = (site.u + fraction * std::f64::consts::TAU / count as f64)
                        .rem_euclid(std::f64::consts::TAU);
                }
                if let Ok(mut candidate) = Topology::build(moved) {
                    if discrete_penalty(surface, p, &f, &candidate)
                        < discrete_penalty(surface, p, &f, &current)
                    {
                        candidate = relax_flip(surface, p, &f, candidate);
                    }
                    let value = score(surface, p, &f, &candidate);
                    if value < best_score {
                        best_score = value;
                        best = Some(candidate);
                    }
                }
            }
        }
        for &i in bad.iter().take(8) {
            let site = &current.tiles[i];
            // A defect-only plateau needs a legal retriangulation crossing,
            // not more zero-force spring passes. Expand the geometric search
            // only in that case; every accepted result still lowers the same
            // band-and-defect objective.
            let defect_only = violations(surface, p, &current).is_empty();
            for direction in 0..24 {
                let angle = std::f64::consts::TAU
                    * ((direction % 8) as f64 + rng.gen_range(0.0..0.125))
                    / 8.0;
                let scale = if defect_only {
                    [0.10, 0.25, 0.45][direction / 8]
                } else {
                    [0.025, 0.10, 0.25][direction / 8]
                };
                let step = p.target_size * scale;
                let mut moved = current.tiles.clone();
                let s = if site.s <= 1e-9 || site.s >= surface.length() - 1e-9 {
                    site.s
                } else {
                    (site.s + step * angle.sin()).clamp(1e-8, surface.length() - 1e-8)
                };
                moved[i].u =
                    (site.u + step * angle.cos() / surface.r(s)).rem_euclid(std::f64::consts::TAU);
                moved[i].s = s;
                moved[i].v = seeds::extended_v(surface, s);
                if let Ok(mut candidate) = Topology::build(moved) {
                    // Composite proposals may cross a spacing barrier, but
                    // only a fully cleaned-up, lower-score endpoint is accepted.
                    if discrete_penalty(surface, p, &f, &candidate)
                        < discrete_penalty(surface, p, &f, &current)
                        && score(surface, p, &f, &candidate) >= best_score
                    {
                        candidate = relax_flip(surface, p, &f, candidate);
                    }
                    let value = score(surface, p, &f, &candidate);
                    if value < best_score {
                        best_score = value;
                        best = Some(candidate);
                    }
                }
            }
            if f.column_changes == 0
                && current.degree(i) > *p.allowed_sides.iter().max().unwrap() as usize
            {
                let mut deleted = current.tiles.clone();
                deleted.remove(i);
                if let Ok(candidate) = Topology::build(deleted) {
                    let value = score(surface, p, &f, &candidate);
                    if value < best_score {
                        best_score = value;
                        best = Some(candidate);
                    }
                }
            }
            if f.column_changes == 0
                && current.degree(i) < *p.allowed_sides.iter().min().unwrap() as usize
            {
                for &j in current.neighbors[i].iter().take(4) {
                    let other = &current.tiles[j];
                    let du = (other.u - site.u + std::f64::consts::PI)
                        .rem_euclid(std::f64::consts::TAU)
                        - std::f64::consts::PI;
                    let s = (site.s + other.s) * 0.5;
                    let mut inserted = current.tiles.clone();
                    inserted.push(Seed {
                        u: (site.u + du * 0.5).rem_euclid(std::f64::consts::TAU),
                        v: seeds::extended_v(surface, s),
                        s,
                        row: site.row,
                        auxiliary: false,
                        depth: seeds::symmetric_depth(site.row, p.depth),
                    });
                    if let Ok(candidate) = Topology::build(inserted) {
                        let value = score(surface, p, &f, &candidate);
                        if value < best_score {
                            best_score = value;
                            best = Some(candidate);
                        }
                    }
                }
            }
        }
        if let Some(candidate) = best {
            current = candidate;
            report.accepted_steps += 1;
        }
    }
    // The last permitted proposal must receive the same terminal check as all
    // earlier proposals; this evaluates pass512, it does not add a repair pass.
    report.iterations = PASSES;
    report.final_residual = spacing_residual(surface, p, &current).0;
    report.final_score = score(surface, p, &f, &current);
    if violations(surface, p, &current).is_empty() && defect_penalty(surface, &f, &current) == 0 {
        seeds::assign_depths(&mut current.tiles, p, rng);
        return Ok((current, report));
    }
    let (residual, min, max) = spacing_residual(surface, p, &current);
    let max_average_excess = (0..current.tiles.len())
        .filter(|&i| interior(&current, i, surface))
        .map(|i| (average_ratio(surface, p, &current, i) - 1.).abs() - p.size_variation)
        .fold(0., f64::max);
    let optimized = optimization_residual(surface, p, &current);
    Err(format!("topology repair cap ({PASSES} passes); initial residual={:.12e}, final residual={residual:.12e}, min/max edge ratios={min:.9}/{max:.9}, accepted steps={}; max average excess={max_average_excess:.12e}, optimized residual={optimized:.12e}; unresolved defect budget {}; tiles: {}",
        report.initial_residual,report.accepted_steps,defect_penalty(surface,&f,&current),
        violations(surface,p,&current).iter().map(|&i|format!("{i}(degree={})",current.degree(i)))
            .collect::<Vec<_>>().join(", ")))
}

#[cfg(test)]
mod metric_margin_tests {
    use super::*;

    // A local six-neighbor star isolates the average-only constraint: its six
    // neighbors are on a physical rim, so no interior-interior edge is checked.
    // This is an objective unit test, not a fabricated manifold acceptance test.
    #[test]
    fn average_only_boundary_has_finite_inward_descent_and_certification_gap() {
        let input: crate::model::CoreInput = serde_json::from_str(include_str!(
            "../../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        let surface = Surface::from_profile(&input.surface.profile).unwrap();
        let mut p = input.params;
        p.target_size = 6.5;
        p.size_variation = 0.15;
        p.allowed_sides = vec![6];
        let center = Seed {
            u: std::f64::consts::PI,
            s: 5.,
            v: surface.v_of_s(5.),
            row: 1,
            auxiliary: false,
            depth: 0.,
        };
        let mut tiles = vec![center.clone()];
        for offset in [-0.003, -0.002, -0.001, 0.001, 0.002, 0.003] {
            let length = p.target_size * (1. + p.size_variation + 5e-8 + offset);
            tiles.push(Seed {
                u: center.u + (length * length - 25.).sqrt() / surface.r(0.),
                s: 0.,
                v: 0.,
                row: 0,
                ..center.clone()
            });
        }
        let t = Topology {
            tiles,
            neighbors: std::iter::once((1..=6).collect())
                .chain((1..=6).map(|_| vec![0]))
                .collect(),
            corners: vec![],
            boundary: vec![false; 7],
        };
        let (residual, min, max) = spacing_residual(&surface, &p, &t);
        assert!(residual > 1e-15);
        assert_eq!((min, max), (0., 0.));
        assert_eq!(violations(&surface, &p, &t), vec![0]);
        let margin_sq = optimization_margin(&p).powi(2);
        let before = optimization_residual(&surface, &p, &t);
        assert!(before > margin_sq);
        let mut moved = t.clone();
        moved.tiles[0].s -= 0.001;
        moved.tiles[0].v = surface.v_of_s(moved.tiles[0].s);
        let after = optimization_residual(&surface, &p, &moved);
        assert!(after < before - 1e-8);
        assert!(after < margin_sq);
        assert!(violations(&surface, &p, &moved).is_empty());
        assert_eq!(spacing_residual(&surface, &p, &moved).0, 0.);
    }
    #[test]
    fn local_probe_matches_global_objective_difference() {
        let input: crate::model::CoreInput = serde_json::from_str(include_str!(
            "../../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        let surface = Surface::from_profile(&input.surface.profile).unwrap();
        let p = input.params;
        let sites = seeds::place(&surface, &p, &mut seeds::rng(1)).unwrap();
        let t = Topology::build(sites).unwrap();
        for i in (0..t.tiles.len()).step_by(7) {
            for ds in [-0.7, 0.4] {
                let mut probe = t.clone();
                probe.tiles[i].u += 0.08;
                probe.tiles[i].s += ds;
                probe.tiles[i].v = seeds::extended_v(&surface, probe.tiles[i].s);
                let global = optimization_residual(&surface, &p, &probe)
                    - optimization_residual(&surface, &p, &t);
                let local = local_optimization_residual(&surface, &p, &probe, i)
                    - local_optimization_residual(&surface, &p, &t, i);
                // Only compare probes that keep the physical-rim membership fixed.
                if interior(&t, i, &surface) == interior(&probe, i, &surface) {
                    assert!((global - local).abs() < 1e-12, "{i}: {global} vs {local}");
                }
            }
        }
    }
}
