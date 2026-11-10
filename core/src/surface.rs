//! Open surfaces of revolution about Y. Lengths are inches; angles are radians.
//!
//! Positions and distances use the supplied polyline exactly. Tangents and curvature
//! use nonuniform three-point differences on the original vertices, interpolated
//! over an arc-length resampling (maximum step 0.05 inches). Thus a polygonal
//! approximation to a smooth curve has smooth curvature rather than impulse spikes.
use crate::tol::GEOM_EPS;
use std::f64::consts::TAU;
use std::fmt;

const MAX_STEP: f64 = 0.05;

/// Invalid open profile; no contract JSON types are changed by these internal errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceError {
    TooFewPoints,
    AxisIntersection,
    SelfIntersection,
    ClosedCurve,
    NonFinitePoint,
    InvalidLength,
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooFewPoints => "profile needs at least two distinct points",
            Self::AxisIntersection => "profile touches or crosses the Y axis",
            Self::SelfIntersection => "profile intersects or retraces itself",
            Self::ClosedCurve => "profile is closed",
            Self::NonFinitePoint => "profile coordinates must be finite",
            Self::InvalidLength => "profile arc length is zero or not representable",
        })
    }
}
impl std::error::Error for SurfaceError {}

/// Deterministic polyline surface with smoothed differential queries.
#[derive(Debug, Clone)]
pub struct Surface {
    points: Vec<[f64; 2]>,
    arcs: Vec<f64>,
    conformal: Vec<f64>,
    tangents: Vec<[f64; 2]>,
    second: Vec<f64>,
    heights: [f64; 2],
}

impl Surface {
    /// Build from XY points in inches. Rejects axis contact, closure, crossings,
    /// nonfinite coordinates and unrepresentable lengths. Consecutive duplicates
    /// within GEOM_EPS are removed. Endpoints orient the profile bottom to top.
    pub fn from_profile(points: &[[f64; 2]]) -> Result<Self, SurfaceError> {
        if points.len() < 2 {
            return Err(SurfaceError::TooFewPoints);
        }
        if points.iter().flatten().any(|x| !x.is_finite()) {
            return Err(SurfaceError::NonFinitePoint);
        }
        if points.iter().any(|p| p[0] <= GEOM_EPS) {
            return Err(SurfaceError::AxisIntersection);
        }
        if distance(points[0], points[points.len() - 1]) <= GEOM_EPS {
            return Err(SurfaceError::ClosedCurve);
        }
        let mut p = Vec::new();
        for &q in points {
            if p.last().is_none_or(|&last| distance(last, q) > GEOM_EPS) {
                p.push(q);
            }
        }
        if p.len() < 2 {
            return Err(SurfaceError::TooFewPoints);
        }
        let last = p.len() - 1;
        if p[last][1] < p[0][1] || (p[last][1] == p[0][1] && p[last][0] < p[0][0]) {
            p.reverse();
        }
        for i in 0..p.len() - 1 {
            if i + 2 < p.len() {
                let a = sub(p[i + 1], p[i]);
                let b = sub(p[i + 2], p[i + 1]);
                if cross(a, b).abs() <= GEOM_EPS * a[0].hypot(a[1])
                    && a[0] * b[0] + a[1] * b[1] < 0.
                {
                    return Err(SurfaceError::SelfIntersection);
                }
            }
            for j in i + 2..p.len() - 1 {
                if intersects(p[i], p[i + 1], p[j], p[j + 1]) {
                    return Err(SurfaceError::SelfIntersection);
                }
            }
        }
        let mut arc = vec![0.];
        for edge in p.windows(2) {
            let next = arc.last().unwrap() + distance(edge[0], edge[1]);
            if !next.is_finite() || next <= *arc.last().unwrap() {
                return Err(SurfaceError::InvalidLength);
            }
            arc.push(next);
        }
        let radii: Vec<_> = p.iter().map(|q| q[0]).collect();
        let ys: Vec<_> = p.iter().map(|q| q[1]).collect();
        let dx = differences(&arc, &radii);
        let dy = differences(&arc, &ys);
        let tangent: Vec<_> = dx
            .iter()
            .zip(dy.iter())
            .map(|(&x, &y)| unit([x, y]))
            .collect();
        let dr: Vec<_> = tangent.iter().map(|t| t[0]).collect();
        let second = differences(&arc, &dr);
        let mut result = Self {
            points: vec![p[0]],
            arcs: vec![0.],
            conformal: vec![0.],
            tangents: vec![tangent[0]],
            second: vec![second[0]],
            heights: [
                ys.iter().copied().fold(f64::INFINITY, f64::min),
                ys.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ],
        };
        for i in 0..p.len() - 1 {
            let len = arc[i + 1] - arc[i];
            let count = (len / MAX_STEP).ceil() as usize;
            for j in 1..=count {
                let t = j as f64 / count as f64;
                let q = lerp(p[i], p[i + 1], t);
                let s = if j == count {
                    arc[i + 1]
                } else {
                    arc[i] + t * len
                };
                let prev = result.points.len() - 1;
                let v = result.conformal[prev]
                    + conformal_integral(result.points[prev][0], q[0], s - result.arcs[prev]);
                result.points.push(q);
                result.arcs.push(s);
                result.conformal.push(v);
                result
                    .tangents
                    .push(unit(lerp(tangent[i], tangent[i + 1], t)));
                result
                    .second
                    .push(second[i] + t * (second[i + 1] - second[i]));
            }
        }
        Ok(result)
    }

    /// Total profile arc length in inches; cannot fail.
    pub fn length(&self) -> f64 {
        *self.arcs.last().unwrap()
    }
    /// Minimum and maximum Y in inches, including overhangs; cannot fail.
    pub fn y_range(&self) -> [f64; 2] {
        self.heights
    }
    fn interval(&self, s: f64) -> (usize, f64) {
        interval(&self.arcs, s)
    }
    fn profile(&self, s: f64) -> [f64; 2] {
        let (i, t) = self.interval(s);
        lerp(self.points[i], self.points[i + 1], t)
    }
    fn tangent(&self, s: f64) -> [f64; 2] {
        let (i, t) = self.interval(s);
        unit(lerp(self.tangents[i], self.tangents[i + 1], t))
    }
    /// Position in inches at theta radians and arc length s inches. s is clamped.
    pub fn point(&self, theta: f64, s: f64) -> [f64; 3] {
        let [r, y] = self.profile(s);
        [r * theta.cos(), y, r * theta.sin()]
    }
    /// Outward unit normal at theta radians, s inches (clamped); cannot fail.
    pub fn normal(&self, theta: f64, s: f64) -> [f64; 3] {
        let [dr, dy] = self.tangent(s);
        [dy * theta.cos(), -dr, dy * theta.sin()]
    }
    /// Radius in inches at s inches (clamped); cannot fail.
    pub fn r(&self, s: f64) -> f64 {
        self.profile(s)[0]
    }
    /// Smoothed dimensionless radial arc derivative at s inches (clamped).
    pub fn dr(&self, s: f64) -> f64 {
        self.tangent(s)[0]
    }
    /// Smoothed second radial arc derivative, inverse inches, at clamped s inches.
    pub fn d2r(&self, s: f64) -> f64 {
        let (i, t) = self.interval(s);
        self.second[i] + t * (self.second[i + 1] - self.second[i])
    }
    /// Gaussian curvature in inverse square inches at clamped s inches.
    pub fn gaussian_curvature(&self, s: f64) -> f64 {
        -self.d2r(s) / self.r(s)
    }
    /// Dimensionless conformal coordinate integral ds/r at clamped s inches.
    pub fn v_of_s(&self, s: f64) -> f64 {
        let (i, t) = self.interval(s);
        let len = t * (self.arcs[i + 1] - self.arcs[i]);
        self.conformal[i] + conformal_integral(self.points[i][0], self.profile(s)[0], len)
    }
    /// Arc length in inches from dimensionless v, clamped to the open profile.
    pub fn s_of_v(&self, v: f64) -> f64 {
        let (i, t) = interval(&self.conformal, v);
        let dv = t * (self.conformal[i + 1] - self.conformal[i]);
        let len = self.arcs[i + 1] - self.arcs[i];
        let r = self.points[i][0];
        let slope = (self.points[i + 1][0] - r) / len;
        let ds = if slope == 0. {
            dv * r
        } else {
            r * (slope * dv).exp_m1() / slope
        };
        (self.arcs[i] + ds).clamp(self.arcs[i], self.arcs[i + 1])
    }
    /// Integrated Gaussian curvature (dimensionless) between clamped arc lengths in inches.
    pub fn total_curvature(&self, s1: f64, s2: f64) -> f64 {
        -TAU * (self.dr(s2) - self.dr(s1))
    }

    fn closest(&self, p: [f64; 3]) -> (f64, usize, [f64; 2], f64) {
        let q = [p[0].hypot(p[2]), p[1]];
        let mut best = (f64::INFINITY, 0, [0., 0.], 0.);
        for i in 0..self.points.len() - 1 {
            let a = self.points[i];
            let edge = sub(self.points[i + 1], a);
            let delta = sub(q, a);
            let t = ((delta[0] * edge[0] + delta[1] * edge[1])
                / (edge[0] * edge[0] + edge[1] * edge[1]))
                .clamp(0., 1.);
            let c = lerp(a, self.points[i + 1], t);
            let d = distance(q, c);
            if d < best.0 {
                best = (d, i, sub(q, c), t);
            }
        }
        best
    }
    /// Closest (theta radians in [0,2pi), arc length inches) on the open surface.
    /// Equal-distance ties choose the first segment; axis points use theta=0.
    pub fn closest_point(&self, p: [f64; 3]) -> (f64, f64) {
        let (_, i, _, t) = self.closest(p);
        let theta = if p[0] == 0. && p[2] == 0. {
            0.
        } else {
            p[2].atan2(p[0]).rem_euclid(TAU)
        };
        (theta, self.arcs[i] + t * (self.arcs[i + 1] - self.arcs[i]))
    }
    /// Distance in inches to the open surface, negative on the inward side of
    /// the nearest oriented segment. No end caps are added. At rim-tangent
    /// extensions sign is positive; finite input points are required.
    pub fn signed_distance(&self, p: [f64; 3]) -> f64 {
        let (d, i, delta, _) = self.closest(p);
        let edge = sub(self.points[i + 1], self.points[i]);
        if delta[0] * edge[1] - delta[1] * edge[0] < 0. {
            -d
        } else {
            d
        }
    }
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = sub(a, b);
    d[0].hypot(d[1])
}
fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    if t == 1. {
        return b;
    }
    [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
}
fn unit(v: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    [v[0] / n, v[1] / n]
}
fn interval(xs: &[f64], x: f64) -> (usize, f64) {
    let x = if x.is_nan() {
        xs[0]
    } else {
        x.clamp(xs[0], xs[xs.len() - 1])
    };
    let i = xs
        .partition_point(|&s| s <= x)
        .saturating_sub(1)
        .min(xs.len() - 2);
    (i, (x - xs[i]) / (xs[i + 1] - xs[i]))
}
fn conformal_integral(r0: f64, r1: f64, len: f64) -> f64 {
    let change = (r1 - r0) / r0;
    if change == 0. {
        len / r0
    } else {
        len / r0 * change.ln_1p() / change
    }
}
// Three-point nonuniform differences, including one-sided quadratic endpoints.
fn differences(s: &[f64], v: &[f64]) -> Vec<f64> {
    if s.len() == 2 {
        let d = (v[1] - v[0]) / (s[1] - s[0]);
        return vec![d, d];
    }
    (0..s.len())
        .map(|i| {
            let j = i.saturating_sub(1).min(s.len() - 3);
            let h0 = s[j + 1] - s[j];
            let h1 = s[j + 2] - s[j + 1];
            let a = (v[j + 1] - v[j]) / h0;
            let b = (v[j + 2] - v[j + 1]) / h1;
            a + (b - a) / (h0 + h1) * (2. * (s[i] - s[j]) - h0)
        })
        .collect()
}
fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let orient = |u: [f64; 2], v: [f64; 2], w: [f64; 2]| {
        let e = sub(v, u);
        let x = cross(e, sub(w, u));
        if x.abs() <= GEOM_EPS * e[0].hypot(e[1]) {
            0
        } else if x > 0. {
            1
        } else {
            -1
        }
    };
    let on = |u: [f64; 2], v: [f64; 2], w: [f64; 2]| {
        (0..2).all(|k| w[k] >= u[k].min(v[k]) - GEOM_EPS && w[k] <= u[k].max(v[k]) + GEOM_EPS)
    };
    let (o1, o2, o3, o4) = (
        orient(a, b, c),
        orient(a, b, d),
        orient(c, d, a),
        orient(c, d, b),
    );
    (o1 * o2 < 0 && o3 * o4 < 0)
        || (o1 == 0 && on(a, b, c))
        || (o2 == 0 && on(a, b, d))
        || (o3 == 0 && on(c, d, a))
        || (o4 == 0 && on(c, d, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resampling_preserves_vertices_and_bounds_step() {
        let surface = Surface::from_profile(&[[9., 0.], [8., 0.123], [8., 1.234]]).unwrap();
        assert!(surface.points.contains(&[8., 0.123]));
        assert!(surface
            .arcs
            .windows(2)
            .all(|w| w[1] - w[0] <= MAX_STEP + 1e-12));
        for i in 0..surface.points.len() {
            assert_eq!(surface.profile(surface.arcs[i]), surface.points[i]);
        }
    }
}
