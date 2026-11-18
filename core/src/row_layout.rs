//! A06 §9 inch-valued constant-height/count stretches, independent of A05.
use crate::{model::Params, surface::Surface};
use serde::Serialize;
use std::f64::consts::{PI, TAU};
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlannedRow {
    pub s: f64,
    pub count: usize,
    pub height: f64,
    pub width: f64,
}
fn range(radii: &[f64], n: usize, p: &Params) -> Option<(f64, f64, f64)> {
    let mut lo = 0_f64;
    let mut hi = f64::INFINITY;
    for &r in radii {
        let w = TAU * r / n as f64;
        let b = w / 2.;
        if b < p.min_triangle_edge || b > p.max_polygon_edge || w > p.blank_usable {
            return None;
        }
        lo = lo.max(
            (4. * p.min_triangle_edge.powi(2) - w * w / 4.)
                .max(0.)
                .sqrt(),
        );
        hi = hi.min(
            (4. * p.max_polygon_edge.powi(2) - w * w / 4.)
                .max(0.)
                .sqrt(),
        );
        hi = hi.min(w / (2. * (p.min_corner_angle_deg * PI / 360.).tan()));
        // The leg/base corner also respects the same angle minimum.
        lo = lo.max(w / 2. * (p.min_corner_angle_deg.to_radians()).tan());
        hi = hi.min(p.blank_usable);
    }
    if lo > hi || hi <= 0. {
        return None;
    }
    let w = TAU * radii[radii.len() / 2] / n as f64;
    let h = (3_f64.sqrt() / 2. * w).clamp(lo.max(1e-6), hi);
    let score = (0.5 * (w * w / 4. + h * h).sqrt() - w / 2.).abs();
    Some((h, score, w))
}
/// Choose the fewest feasible constant-count stretches at exact profile knots.
/// The fallback considers only count differences ±g and g-fold-compatible counts.
/// Rejects infeasible layouts and resource caps; never changes physical limits.
pub fn choose(surface: &Surface, p: &Params) -> Result<Vec<PlannedRow>, String> {
    if !p.max_polygon_edge.is_finite()
        || p.max_polygon_edge <= 0.
        || p.min_triangle_edge <= 0.
        || !p.min_triangle_edge.is_finite()
        || p.count_step == 0
        || p.count_step > 128
        || p.blank_usable <= 0.
        || !p.min_corner_angle_deg.is_finite()
        || !(0.0..90.0).contains(&p.min_corner_angle_deg)
    {
        return Err("invalid A06 row-layout settings".into());
    }
    // Every triangle edge must satisfy both bounds. No count/stretch can
    // rescue an empty interval: reject it before the quadratic fallback DP.
    if p.min_triangle_edge > p.max_polygon_edge {
        return Err(
            "no feasible A06 n/h layout: min_triangle_edge exceeds max_polygon_edge".into(),
        );
    }
    let knots = surface.layout_knots();
    let radii: Vec<_> = knots.iter().map(|&s| surface.r(s)).collect();
    let max_r = radii.iter().copied().fold(0., f64::max);
    let min_r = radii.iter().copied().fold(f64::INFINITY, f64::min);
    let low = ((PI * max_r / p.max_polygon_edge).ceil() as usize).max(3);
    let high = (PI * min_r / p.min_triangle_edge).floor() as usize;
    let mut best: Option<(usize, f64, f64)> = None;
    for n in low..=high.min(4096) {
        if let Some((h, score, _)) = range(&radii, n, p) {
            if best.is_none_or(|(bn, _, bs)| {
                score < bs - 1e-12 || ((score - bs).abs() <= 1e-12 && n < bn)
            }) {
                best = Some((n, h, score));
            }
        }
    }
    let stretches = if let Some((n, h, _)) = best {
        vec![(0., surface.length(), n, h)]
    } else {
        // Dynamic programming over all profile-knot intervals: minimize number of
        // stretches, then stable count/path order. Every interval is checked against
        // every exact linear-profile extremum, not a radius sample approximation.
        let cap = (PI * max_r / p.min_triangle_edge).floor().min(4096.) as usize;
        if knots.len() > 2500 {
            return Err("A06 fallback knot cap exceeded".into());
        }
        type Path = Vec<(usize, usize, f64)>;
        let mut dp = vec![std::collections::BTreeMap::<usize, Path>::new(); knots.len()];
        for end in 1..knots.len() {
            for n in (3..=cap).filter(|n| n % p.count_step == 0) {
                let mut path: Option<Path> = None;
                for start in 0..end {
                    if let Some((h, _, _)) = range(&radii[start..=end], n, p) {
                        let candidate = if start == 0 {
                            Some(vec![(end, n, h)])
                        } else {
                            dp[start]
                                .iter()
                                .filter(|(old, _)| old.abs_diff(n) == p.count_step)
                                .map(|(_, v)| {
                                    let mut q = v.clone();
                                    q.push((end, n, h));
                                    q
                                })
                                .min_by_key(Vec::len)
                        };
                        if let Some(c) = candidate {
                            if path.as_ref().is_none_or(|v| c.len() < v.len()) {
                                path = Some(c);
                            }
                        }
                    }
                }
                if let Some(path) = path {
                    dp[end].insert(n, path);
                }
            }
        }
        let path = dp
            .last()
            .unwrap()
            .values()
            .min_by_key(|v| v.len())
            .ok_or("no feasible A06 n/h layout, including count-step fallback")?;
        let mut start = 0.;
        path.iter()
            .map(|&(end, n, h)| {
                let end = knots[end];
                let item = (start, end, n, h);
                start = end;
                item
            })
            .collect()
    };
    let mut rows = Vec::new();
    let mut at = 0.;
    let mut stretch = 0;
    let first = stretches[0];
    for k in [-2., -1.] {
        let s = k * first.3;
        rows.push(PlannedRow {
            s,
            count: first.2,
            height: first.3,
            width: TAU * surface.r(s) / first.2 as f64,
        });
    }
    while at <= surface.length() {
        while stretch + 1 < stretches.len() && at > stretches[stretch].1 {
            stretch += 1;
        }
        let (_, _, n, h) = stretches[stretch];
        rows.push(PlannedRow {
            s: at,
            count: n,
            height: h,
            width: TAU * surface.r(at) / n as f64,
        });
        at += h;
        if rows.len() > 10000 {
            return Err("A06 row cap exceeded".into());
        }
    }
    let (_, _, n, h) = *stretches.last().unwrap();
    for _ in 0..2 {
        rows.push(PlannedRow {
            s: at,
            count: n,
            height: h,
            width: TAU * surface.r(at) / n as f64,
        });
        at += h;
    }
    Ok(rows)
}
