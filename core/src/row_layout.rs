//! Owner's A06 §9 single global count/height layout; lengths are inches.
use crate::{
    model::{Params, RowHeight},
    surface::Surface,
};
use serde::Serialize;
use std::f64::consts::{PI, TAU};
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlannedRow {
    pub s: f64,
    pub count: usize,
    pub height: f64,
    pub width: f64,
}
/// Choose the largest count allowed by the narrowest radius, then one height.
/// Checks all piecewise-linear profile extrema, including between generated rows.
/// Returns the offending profile position and limit; never changes count to fit.
pub fn choose(surface: &Surface, p: &Params) -> Result<Vec<PlannedRow>, String> {
    choose_with_phase(surface, p, 0.)
}
/// Place rows at integer plus fractional phase in the integrated pitch coordinate.
/// Phase must be finite in [0,1); lengths and actual forward gaps are inches.
pub fn choose_with_phase(
    surface: &Surface,
    p: &Params,
    phase: f64,
) -> Result<Vec<PlannedRow>, String> {
    if !phase.is_finite() || !(0.0..1.0).contains(&phase) {
        return Err("A06 pattern phase must be a fraction in [0,1)".into());
    }
    if !p.layout_headroom.is_finite()
        || p.layout_headroom < 0.
        || !p.edge_shortfall.is_finite()
        || p.edge_shortfall < 0.
        || p.edge_shortfall >= 0.5
        || !p.min_triangle_edge.is_finite()
        || p.min_triangle_edge <= 0.
        || p.max_polygon_edge
            .is_some_and(|v| !v.is_finite() || v <= 0.)
        || !p.blank_usable.is_finite()
        || p.blank_usable <= 0.
        || !p.min_corner_angle_deg.is_finite()
        || !(0.0..90.0).contains(&p.min_corner_angle_deg)
    {
        return Err("invalid A06 row-layout settings".into());
    }
    let knots = surface.layout_knots();
    let min_r = knots
        .iter()
        .map(|&s| surface.r(s))
        .fold(f64::INFINITY, f64::min);
    let n = (PI * min_r / ((1. + p.layout_headroom) * p.min_triangle_edge)).floor() as usize;
    if n < 3 {
        let s = *knots
            .iter()
            .min_by(|&&a, &&b| surface.r(a).total_cmp(&surface.r(b)))
            .unwrap();
        return Err(format!(
            "infeasible A06 row s={s}: min_triangle_edge requires n < 3"
        ));
    }
    if n > 4096 {
        return Err("A06 global count resource cap exceeded (4096)".into());
    }
    let mut h = 3_f64.sqrt() / 2. * TAU * min_r / n as f64;
    // The base angles shrink at wider rows. Raise h only to their hard minimum.
    for &s in &knots {
        if p.row_height == RowHeight::Proportional {
            break;
        }
        let w = TAU * surface.r(s) / n as f64;
        h = h.max(w / 2. * p.min_corner_angle_deg.to_radians().tan());
    }
    let height_at = |s: f64| {
        if p.row_height == RowHeight::Proportional {
            h * surface.r(s) / min_r
        } else {
            h
        }
    };
    for &s in &knots {
        let h = height_at(s);
        let w = TAU * surface.r(s) / n as f64;
        let base = w / 2.;
        let leg = 0.5 * (w * w / 4. + h * h).sqrt();
        let angle = (2. * (w / (2. * h)).atan()).to_degrees();
        let fail = |limit: &str, value: f64, bound: f64| {
            format!("infeasible A06 row s={s} n={n} h={h}: {limit} value={value} limit={bound}")
        };
        if base.min(leg) < p.min_triangle_edge - 1e-9 {
            return Err(fail(
                "min_triangle_edge",
                base.min(leg),
                p.min_triangle_edge,
            ));
        }
        if let Some(maximum) = p.max_polygon_edge {
            if base.max(leg) > maximum + 1e-9 {
                return Err(fail("max_polygon_edge", base.max(leg), maximum));
            }
        }
        if angle.min((180. - angle) / 2.) < p.min_corner_angle_deg - 1e-9 {
            return Err(fail(
                "min_corner_angle_deg",
                angle.min((180. - angle) / 2.),
                p.min_corner_angle_deg,
            ));
        }
        // Circumferential pitch is not a lower bound on a fitted stone's
        // enclosing square (rectification and square rotation both reduce it).
        // Leave blank_usable to the unchanged actual C4/C5 footprint gates.
    }
    let mut rows = Vec::new();
    let max_r = knots.iter().map(|&s| surface.r(s)).fold(0., f64::max);
    // Use exactly the legacy arithmetic on cylinders in either mode.
    if p.row_height == RowHeight::Constant || max_r == min_r {
        let mut at = -2. * h;
        while at <= surface.length() + 2. * h {
            let position = at + phase * h;
            rows.push(PlannedRow {
                s: position,
                count: n,
                height: h,
                width: TAU * surface.r(position) / n as f64,
            });
            at += h;
            if rows.len() > 10000 {
                return Err("A06 row cap exceeded".into());
            }
        }
    } else {
        // t = min_r/h * integral ds/r. Surface's analytic piecewise-linear
        // conformal map and inverse avoid Euler stepping, including knot crossings.
        let scale = h / min_r;
        let end_v = surface.v_of_s(surface.length());
        let end_t = end_v / scale;
        let inverse = |t: f64| {
            if t < 0. {
                t * h * surface.r(0.) / min_r
            } else if t > end_t {
                surface.length() + (t - end_t) * h * surface.r(surface.length()) / min_r
            } else {
                surface.s_of_v(t * scale)
            }
        };
        let mut k = -2;
        while k as f64 + phase <= end_t + 2. {
            let t = k as f64 + phase;
            let at = inverse(t);
            rows.push(PlannedRow {
                s: at,
                count: n,
                height: inverse(t + 1.) - at,
                width: TAU * surface.r(at) / n as f64,
            });
            k += 1;
            if rows.len() > 10000 {
                return Err("A06 row cap exceeded".into());
            }
        }
    }
    Ok(rows)
}
