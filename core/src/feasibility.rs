//! Deterministic, advisory feasibility estimates for open surfaces of revolution.
use crate::{
    model::{Feasibility, Params},
    surface::Surface,
};
use std::f64::consts::TAU;

const STEP: f64 = 0.05;
const NOISE: f64 = 1e-6;

/// Estimate feasibility from inch-valued geometry and degree-valued bevel limits.
/// Requires positive finite size/bevel inputs and a nonempty side set in 4..=9.
/// Counts are estimates, not a guarantee of optimizer convergence.
pub fn check(surface: &Surface, params: &Params) -> Feasibility {
    let mut result = estimate(surface, params);
    if !result.feasible {
        let mut candidates = Vec::new();
        for mask in 1u32..64 {
            let set: Vec<u32> = (4..=9).filter(|k| mask & (1 << (k - 4)) != 0).collect();
            if !params.allowed_sides.iter().all(|k| set.contains(k)) {
                continue;
            }
            let mut candidate = params.clone();
            candidate.allowed_sides = set.clone();
            // Suggestions contain only sets that work at the requested variation.
            if estimate(surface, &candidate).feasible {
                let added: Vec<_> = set
                    .iter()
                    .filter(|k| !params.allowed_sides.contains(k))
                    .collect();
                candidates.push((
                    added.len(),
                    added.iter().map(|k| k.abs_diff(6)).sum::<u32>(),
                    set,
                ));
            }
        }
        candidates.sort();
        result.suggestions = candidates
            .into_iter()
            .take(4)
            .map(|(_, _, set)| set)
            .collect();
    }
    result
}

fn estimate(surface: &Surface, p: &Params) -> Feasibility {
    let steps = (surface.length() / STEP).ceil() as usize;
    let delta = p.size_variation;
    let min_radius = p.min_bounding_square * (180.0 / p.bevel_cap_deg).ceil() / TAU;
    let mut f = Feasibility {
        feasible: false,
        pentagons_needed: 0,
        heptagons_needed: 0,
        column_changes: 0,
        columns: vec![],
        small_radius_ranges: vec![],
        waist_ranges: vec![],
        suggestions: vec![],
    };
    let mut positive = 0.0;
    let mut negative = 0.0;
    let mut area = 0.0;
    let mut intervals = Vec::new();
    let mut possible = true;
    for i in 0..=steps {
        let s = surface.length() * i as f64 / steps as f64;
        let r = surface.r(s);
        // Even wraps avoid a helical seam. Units below are pairs of columns.
        let low = (TAU * r / (p.target_size * (1.0 + delta)) / 2.0 - 1e-12)
            .ceil()
            .max(1.0) as u32;
        let high = (TAU * r / (p.target_size * (1.0 - delta)) / 2.0 + 1e-12).floor() as u32;
        possible &= low <= high;
        intervals.push((low, high.max(low)));
        if i == 0 {
            continue;
        }
        let prev = surface.length() * (i - 1) as f64 / steps as f64;
        let mid = (s + prev) / 2.0;
        let dr = surface.dr(s) - surface.dr(prev);
        if dr < 0.0 {
            positive -= 6.0 * dr;
        } else {
            negative += 6.0 * dr;
        }
        area += TAU * (surface.r(prev) + r) * 0.5 * (s - prev);
        let heights = [surface.point(0.0, prev)[1], surface.point(0.0, s)[1]];
        if surface.d2r(mid) > NOISE {
            append_range(&mut f.waist_ranges, heights);
        }
        if surface.r(mid) < min_radius {
            append_range(&mut f.small_radius_ranges, heights);
        }
    }
    f.pentagons_needed = positive.round() as u32;
    f.heptagons_needed = negative.round() as u32;
    // Interval projection minimizes total variation, with deterministic low-count ties.
    let mut optimum = intervals[0];
    for &(lo, hi) in &intervals[1..] {
        optimum = if lo > optimum.1 {
            (lo, lo)
        } else if hi < optimum.0 {
            (hi, hi)
        } else {
            (lo.max(optimum.0), hi.min(optimum.1))
        };
    }
    let mut n = optimum.0;
    let mut sequence = Vec::new();
    for &(lo, hi) in intervals.iter().rev() {
        n = n.clamp(lo, hi);
        sequence.push(2 * n);
    }
    sequence.reverse();
    sequence.dedup();
    f.column_changes = sequence
        .windows(2)
        .map(|pair| pair[0].abs_diff(pair[1]))
        .sum();
    f.columns = sequence;
    let lower = p.allowed_sides.iter().copied().min().unwrap_or(9) as f64;
    let upper = p.allowed_sides.iter().copied().max().unwrap_or(4) as f64;
    let average =
        6.0 - (positive.round() - negative.round()) / (area / p.target_size.powi(2)).max(1.0);
    let below = p.allowed_sides.iter().any(|&k| k < 6);
    let above = p.allowed_sides.iter().any(|&k| k > 6);
    f.feasible = possible
        && (f.pentagons_needed == 0 || below)
        && (f.heptagons_needed == 0 || above)
        && (f.column_changes == 0 || (below && above))
        && average >= lower
        && average <= upper
        && (p.allowed_sides.contains(&6) || (lower < average && upper > average))
        && (p.allow_waists || f.waist_ranges.is_empty())
        && f.small_radius_ranges.is_empty();
    f
}

fn append_range(ranges: &mut Vec<[f64; 2]>, range: [f64; 2]) {
    let range = [range[0].min(range[1]), range[0].max(range[1])];
    if let Some(last) = ranges.last_mut() {
        if range[0] <= last[1] + 1e-9 {
            last[1] = last[1].max(range[1]);
            return;
        }
    }
    ranges.push(range);
}

/// Plain-language inch-valued feasibility summary; does not change the selected side set.
pub fn message(surface: &Surface, f: &Feasibility, p: &Params) -> String {
    if f.feasible {
        return "The allowed side set is feasible for this profile and size range.".into();
    }
    let label = if p.allowed_sides.len() == 1 {
        format!("{} only", p.allowed_sides[0])
    } else {
        format_set(&p.allowed_sides)
    };
    let mut reasons = Vec::new();
    if !p.allow_waists && !f.waist_ranges.is_empty() {
        reasons.push(format!(
            "this profile has a waist at heights {} in",
            format_ranges(&f.waist_ranges)
        ));
    }
    if !f.small_radius_ranges.is_empty() {
        reasons.push(format!(
            "the radius is too small for the size and bevel limits at heights {} in",
            format_ranges(&f.small_radius_ranges)
        ));
    }
    if f.pentagons_needed > 0 {
        reasons.push(format!(
            "this profile bulges, so it needs about {} five-sided pieces",
            f.pentagons_needed
        ));
    }
    if f.heptagons_needed > 0 {
        reasons.push(format!(
            "this profile has negative curvature, so it needs about {} seven-sided pieces",
            f.heptagons_needed
        ));
    }
    if f.column_changes > 0 {
        reasons.push(format!(
            "the circumference changes, so it needs about {} five-and-seven-sided pairs",
            f.column_changes
        ));
    }
    if reasons.is_empty() {
        reasons.push("the allowed side counts cannot balance the required average or fit an even column wrap".into());
    }
    let mut text = format!(
        "{label} is not possible at ±{} %: {}.",
        p.size_variation * 100.0,
        reasons.join("; ")
    );
    // Feasibility is monotone in variation: wrap intervals only expand; all
    // other blockers are unchanged. The first successful 5%-grid point is
    // therefore the true threshold rounded upward, not a preset fallback.
    let mut candidate = p.clone();
    for percent in (5..=50).step_by(5) {
        candidate.size_variation = f64::from(percent) / 100.0;
        if candidate.size_variation <= p.size_variation {
            continue;
        }
        if estimate(surface, &candidate).feasible {
            text.push_str(&format!(
                " With size variation of at least ±{percent} %, {} works.",
                format_set(&p.allowed_sides)
            ));
            break;
        }
    }
    if !f.suggestions.is_empty() {
        text.push_str(&format!(
            " Try {}.",
            f.suggestions
                .iter()
                .map(|s| format_set(s))
                .collect::<Vec<_>>()
                .join(" or ")
        ));
    }
    text
}
fn format_set(set: &[u32]) -> String {
    format!(
        "[{}]",
        set.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    )
}
fn format_ranges(ranges: &[[f64; 2]]) -> String {
    ranges
        .iter()
        .map(|r| format!("{:.2}–{:.2}", r[0], r[1]))
        .collect::<Vec<_>>()
        .join(", ")
}
