//! A05 fixed-budget depth-space planarization. Inches, outward normals, degrees in reports.
use crate::{
    model::{Params, Plane},
    topology::Topology,
};
use serde::Serialize;
use std::f64::consts::{PI, TAU};
type V = [f64; 3];
fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
fn mul(a: V, s: f64) -> V {
    a.map(|v| v * s)
}
fn dot(a: V, b: V) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: V) -> V {
    mul(a, 1.0 / norm(a).max(1e-30))
}
fn basis(n: V) -> (V, V) {
    let axis = if n[1].abs() < 0.9 {
        [0., 1., 0.]
    } else {
        [1., 0., 0.]
    };
    let a = unit(cross(axis, n));
    (a, cross(n, a))
}
/// Generic surface interface: C4 never inspects radius, axis or profile.
/// Chart coordinates are periodic in the first component with period 2π.
pub trait SurfaceQueries {
    /// Surface point and outward unit normal; extended chart supports auxiliary rows.
    fn frame(&self, uv: [f64; 2]) -> (V, V);
    /// Whether a point lies between the physical rim planes (inches).
    fn inside_rims(&self, _point: V) -> bool {
        true
    }
    /// Signed closest distance in inches, positive outside.
    fn distance(&self, point: V) -> f64;
    /// Inward offset in inches; cannot fail for a valid chart query.
    fn offset(&self, uv: [f64; 2], depth: f64) -> V {
        let (p, n) = self.frame(uv);
        sub(p, mul(n, depth))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DistanceStats {
    pub min: f64,
    pub max: f64,
    pub median: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OptReport {
    /// Sum of squared normalized positive limit residuals (independent of barrier weight).
    pub limit_violation_sum_sq: f64,
    pub iterations: usize,
    pub corner_distance: DistanceStats,
    pub reflex_corners: usize,
    pub violated: Vec<(usize, String)>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Planes {
    pub planes: Vec<Plane>,
    pub corners: Vec<V>,
    /// Corner indices in topological CCW order; hull fans are empty.
    pub polygons: Vec<Vec<usize>>,
    /// Complete non-auxiliary polygons wholly inside physical rims.
    pub whole_tiles: Vec<bool>,
    pub report: OptReport,
}
#[derive(Clone, Debug, Serialize)]
pub struct OptError {
    pub message: String,
    pub best: Option<Box<Planes>>,
}
#[derive(Clone)]
struct Problem<'a> {
    fixed_whole: Option<Vec<bool>>,
    surface: Option<&'a dyn SurfaceQueries>,
    t: &'a Topology,
    p: &'a Params,
    points: Vec<V>,
    normal: Vec<V>,
    axes: Vec<(V, V)>,
    targets: Vec<V>,
    fans: Vec<Vec<usize>>,
}
fn fans(t: &Topology) -> Vec<Vec<usize>> {
    (0..t.tiles.len())
        .map(|i| {
            if t.boundary[i] {
                return vec![];
            }
            let incident: Vec<_> = t
                .corners
                .iter()
                .enumerate()
                .filter(|(_, c)| c.contains(&i))
                .map(|(k, _)| k)
                .collect();
            if incident.is_empty() {
                return vec![];
            }
            let mut order = vec![incident[0]];
            while order.len() < incident.len() {
                let last = t.corners[*order.last().unwrap()];
                let next = incident.iter().copied().find(|k| {
                    !order.contains(k)
                        && t.corners[*k].iter().filter(|a| last.contains(a)).count() == 2
                });
                match next {
                    Some(k) => order.push(k),
                    None => return vec![],
                }
            }
            order
        })
        .collect()
}
impl Problem<'_> {
    fn geometry(&self, x: &[f64]) -> (Vec<Plane>, Vec<V>, Vec<f64>) {
        let planes: Vec<_> = (0..self.points.len())
            .map(|i| {
                let (a, b) = self.axes[i];
                let n = unit(add(
                    self.normal[i],
                    add(mul(a, x[2 * i]), mul(b, x[2 * i + 1])),
                ));
                Plane {
                    n,
                    d: dot(n, self.points[i]),
                }
            })
            .collect();
        let mut dets = vec![];
        let corners = self
            .t
            .corners
            .iter()
            .map(|&[i, j, k]| {
                let (a, b, c) = (&planes[i], &planes[j], &planes[k]);
                let det = dot(a.n, cross(b.n, c.n));
                dets.push(det.abs());
                let safe = if det.abs() < 1e-12 {
                    if det < 0. {
                        -1e-12
                    } else {
                        1e-12
                    }
                } else {
                    det
                };
                mul(
                    add(
                        add(mul(cross(b.n, c.n), a.d), mul(cross(c.n, a.n), b.d)),
                        mul(cross(a.n, b.n), c.d),
                    ),
                    1. / safe,
                )
            })
            .collect();
        (planes, corners, dets)
    }
    fn evaluate(&self, x: &[f64], weight: f64) -> (Vec<f64>, Planes) {
        let (planes, corners, dets) = self.geometry(x);
        let mut residual = vec![];
        let mut violated = vec![];
        let mut limit_violation_sum_sq = 0.;
        let mut reflex = 0;
        let mut polygons = self.fans.clone();
        let whole_tiles: Vec<bool> = self.fixed_whole.clone().unwrap_or_else(|| {
            polygons
                .iter()
                .enumerate()
                .map(|(i, ids)| {
                    ids.len() >= 3
                        && !self.t.tiles[i].auxiliary
                        && ids
                            .iter()
                            .all(|&k| self.surface.is_none_or(|s| s.inside_rims(corners[k])))
                })
                .collect()
        });
        for (v, target) in corners.iter().zip(&self.targets) {
            residual.extend(sub(*v, *target).map(|a| 0.03 * a));
        }
        let mut barrier = |tile: usize, name: &str, value: f64| {
            let value = if !whole_tiles[tile] {
                0.
            } else if value.is_finite() {
                value
            } else {
                f64::INFINITY
            };
            limit_violation_sum_sq += value.max(0.).powi(2);
            residual.push(weight.sqrt() * value.max(0.));
            if value > 1e-7 {
                violated.push((tile, name.to_string()));
            }
        };
        for (k, &d) in dets.iter().enumerate() {
            // Associate the corner with a measured incident piece, if any.
            let tile = self.t.corners[k]
                .iter()
                .copied()
                .find(|&i| whole_tiles[i])
                .unwrap_or(self.t.corners[k][0]);
            barrier(tile, "determinant", (1e-8 - d) * 1e8);
        }
        for (i, ids) in polygons.iter_mut().enumerate() {
            let n = planes[i].n;
            let (a, b) = basis(n);
            if ids.len() < 3 {
                continue;
            }
            let area: f64 = (0..ids.len())
                .map(|k| {
                    dot(
                        cross(
                            sub(corners[ids[k]], self.points[i]),
                            sub(corners[ids[(k + 1) % ids.len()]], self.points[i]),
                        ),
                        n,
                    )
                })
                .sum();
            if area < 0. {
                ids.reverse();
            }
            let q: Vec<_> = ids
                .iter()
                .map(|&k| {
                    let d = sub(corners[k], self.points[i]);
                    [dot(d, a), dot(d, b)]
                })
                .collect();
            for k in 0..ids.len() {
                let prev = corners[ids[(k + ids.len() - 1) % ids.len()]];
                let cur = corners[ids[k]];
                let next = corners[ids[(k + 1) % ids.len()]];
                let u = sub(cur, prev);
                let v = sub(next, cur);
                let edge = norm(v);
                barrier(i, "min_edge", self.p.min_edge - edge);
                barrier(i, "max_edge", edge - self.p.max_edge);
                let angle = 180. - dot(cross(u, v), n).atan2(dot(u, v)).to_degrees();
                reflex += usize::from(whole_tiles[i] && angle > 180. + 1e-7);
                barrier(
                    i,
                    "min_corner_angle_deg",
                    (self.p.min_corner_angle_deg - angle) / 30.,
                );
                barrier(
                    i,
                    "max_corner_angle_deg",
                    (angle - self.p.max_corner_angle_deg) / 30.,
                );
            }
            let mut footprint = q.clone();
            let inner: Vec<_> = ids
                .iter()
                .map(|&corner| {
                    let [j, k, l] = self.t.corners[corner];
                    let (u, v, w) = (planes[j].n, planes[k].n, planes[l].n);
                    let det = dot(u, cross(v, w));
                    let safe = if det.abs() < 1e-12 { 1e-12 } else { det };
                    let displacement = mul(
                        add(add(cross(v, w), cross(w, u)), cross(u, v)),
                        self.p.thickness / safe,
                    );
                    let d = sub(sub(corners[corner], displacement), self.points[i]);
                    [dot(d, a), dot(d, b)]
                })
                .collect();
            footprint.extend(&inner);
            let square = bounding_square(&q);
            let footprint_square = bounding_square(&footprint);
            barrier(
                i,
                "min_bounding_square",
                self.p.min_bounding_square - square,
            );
            barrier(i, "blank_usable", footprint_square - self.p.blank_usable);
            let inner_area: f64 = (0..inner.len())
                .map(|k| {
                    let a = inner[k];
                    let b = inner[(k + 1) % inner.len()];
                    a[0] * b[1] - a[1] * b[0]
                })
                .sum();
            barrier(
                i,
                "inner_orientation",
                if inner_area <= 0. { 1. } else { 0. },
            );
            for k in 0..inner.len() {
                for j in k + 2..inner.len() {
                    if k == 0 && j + 1 == inner.len() {
                        continue;
                    }
                    barrier(
                        i,
                        "simple_inner_polygon",
                        if intersects(
                            inner[k],
                            inner[(k + 1) % inner.len()],
                            inner[j],
                            inner[(j + 1) % inner.len()],
                        ) {
                            1.
                        } else {
                            0.
                        },
                    );
                }
            }
            for k in 0..q.len() {
                for j in k + 2..q.len() {
                    if k == 0 && j + 1 == q.len() {
                        continue;
                    }
                    barrier(
                        i,
                        "simple_polygon",
                        if intersects(q[k], q[(k + 1) % q.len()], q[j], q[(j + 1) % q.len()]) {
                            1.
                        } else {
                            0.
                        },
                    );
                }
            }
        }
        for i in 0..planes.len() {
            for &j in &self.t.neighbors[i] {
                if j <= i {
                    continue;
                }
                let angle = dot(planes[i].n, planes[j].n)
                    .clamp(-1., 1.)
                    .acos()
                    .to_degrees()
                    / 2.;
                barrier(
                    if whole_tiles[i] { i } else { j },
                    "bevel_cap_deg",
                    (angle - self.p.bevel_cap_deg) / 15.,
                );
            }
        }
        drop(barrier);
        // Weak tilt and adjacent-tilt fairness; no surface-distance residual.
        for &value in x {
            residual.push(0.005 * value);
        }
        for ids in &polygons {
            for k in 0..ids.len() {
                residual.push(
                    0.01 * (norm(sub(corners[ids[k]], corners[ids[(k + 1) % ids.len()]]))
                        - self.p.target_size / 3f64.sqrt()),
                );
            }
        }
        for i in 0..planes.len() {
            for &j in &self.t.neighbors[i] {
                if j > i {
                    residual.push(0.005 * (x[2 * i] - x[2 * j]));
                    residual.push(0.005 * (x[2 * i + 1] - x[2 * j + 1]));
                }
            }
        }

        (
            residual,
            Planes {
                planes,
                corners,
                polygons,
                whole_tiles,
                report: OptReport {
                    limit_violation_sum_sq,
                    iterations: 0,
                    corner_distance: DistanceStats {
                        min: 0.,
                        max: 0.,
                        median: 0.,
                    },
                    reflex_corners: reflex,
                    violated,
                },
            },
        )
    }
}
fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    fn turn(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    }
    turn(a, b, c) * turn(a, b, d) < -1e-16 && turn(c, d, a) * turn(c, d, b) < -1e-16
}
/// Exact candidate enumeration for a minimum enclosing square (inches).
/// Between support changes the optimum is a stationary width/height or width=height.
pub fn bounding_square(q: &[[f64; 2]]) -> f64 {
    let half = PI / 2.;
    let mut breaks = vec![0., half];
    for (i, a) in q.iter().enumerate() {
        for b in &q[i + 1..] {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            if dx != 0. || dy != 0. {
                breaks.push(dy.atan2(dx).rem_euclid(half));
            }
        }
    }
    breaks.sort_by(f64::total_cmp);
    breaks.dedup_by(|a, b| (*a - *b).abs() < 1e-14);
    let mut angles = breaks.clone();
    for span in breaks.windows(2) {
        let mid = (span[0] + span[1]) * 0.5;
        let (s, c) = mid.sin_cos();
        let mut lo = [0usize; 2];
        let mut hi = [0usize; 2];
        let projection = |p: [f64; 2], axis: usize| {
            if axis == 0 {
                p[0] * c + p[1] * s
            } else {
                -p[0] * s + p[1] * c
            }
        };
        for k in 1..q.len() {
            for axis in 0..2 {
                if projection(q[k], axis) < projection(q[lo[axis]], axis) {
                    lo[axis] = k;
                }
                if projection(q[k], axis) > projection(q[hi[axis]], axis) {
                    hi[axis] = k;
                }
            }
        }
        let a = [q[hi[0]][0] - q[lo[0]][0], q[hi[0]][1] - q[lo[0]][1]];
        let b = [q[hi[1]][0] - q[lo[1]][0], q[hi[1]][1] - q[lo[1]][1]];
        for base in [
            a[1].atan2(a[0]),
            (-b[0]).atan2(b[1]),
            (b[1] - a[0]).atan2(a[1] + b[0]),
        ] {
            for shift in -2..=2 {
                let t = base + shift as f64 * PI;
                if t >= span[0] && t <= span[1] {
                    angles.push(t);
                }
            }
        }
    }
    angles
        .into_iter()
        .map(|t| {
            let (s, c) = t.sin_cos();
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for p in q {
                let v = [p[0] * c + p[1] * s, -p[0] * s + p[1] * c];
                for k in 0..2 {
                    lo[k] = lo[k].min(v[k]);
                    hi[k] = hi[k].max(v[k]);
                }
            }
            (hi[0] - lo[0]).max(hi[1] - lo[1])
        })
        .fold(f64::INFINITY, f64::min)
}
/// Fit planes through inward-offset seeds, then optionally polish with strictly improving limits.
/// Failure preserves the best complete geometry/report; does not certify C5 inner footprints.
pub fn optimize<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
    _rng: &mut rand_chacha::ChaCha8Rng,
) -> Result<Planes, OptError> {
    optimize_stage(surface, t, p, false)
}
/// Native reference-method fit on production C3 topology (inches); no imported arrays.
/// Returns geometry with ALL available C4 limits reported, even when they fail.
pub fn fitted<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
) -> Result<Planes, OptError> {
    optimize_stage(surface, t, p, true)
}
fn optimize_stage<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
    fit_only: bool,
) -> Result<Planes, OptError> {
    let problem = checked_native_problem(surface, t, p)?;
    let (x, _) = prototype_fit(&problem, prototype_start(&problem));
    if fit_only {
        let (_, geometry) = problem.evaluate(&x, 1.);
        return Ok(with_distances(surface, geometry));
    }
    solve(surface, &problem, x)
}

/// Non-contract diagnostics in seconds; timings never participate in solver decisions.
#[derive(Default, Debug, Serialize)]
pub struct FitProfile {
    pub nfev: usize,
    pub residual_calls: usize,
    pub residual_seconds: f64,
    pub jacobians: usize,
    pub jacobian_seconds: f64,
    pub svds: usize,
    pub svd_seconds: f64,
}
/// Single-fit diagnostic, retaining the fitted geometry before optional polish (inches).
#[derive(Debug, Serialize)]
pub struct FitAndPolish {
    pub fit: Planes,
    pub result: Result<Planes, OptError>,
    pub fit_seconds: f64,
    pub polish_seconds: f64,
    pub total_seconds: f64,
    pub fit_profile: FitProfile,
}
/// Compute the identical production start/fit once, then pass its unchanged tilt vector
/// into production polish. Fails on invalid topology/band; polish failure retains best.
/// Wall times are diagnostic only, never serialized into production CoreOutput.
pub fn fit_and_polish<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
) -> Result<FitAndPolish, OptError> {
    let started = std::time::Instant::now();
    let problem = checked_native_problem(surface, t, p)?;
    let mut fit_profile = FitProfile::default();
    let (x, _) = prototype_fit_profiled(&problem, prototype_start(&problem), &mut fit_profile);
    let (_, baseline) = problem.evaluate(&x, 1.);
    let fit = with_distances(surface, baseline.clone());
    let fit_seconds = started.elapsed().as_secs_f64();
    let polish_start = std::time::Instant::now();
    // Reuse only the exact deterministic evaluation at the same x and weight.
    let result = solve_from_baseline(surface, &problem, x, baseline);
    let polish_seconds = polish_start.elapsed().as_secs_f64();
    Ok(FitAndPolish {
        fit,
        result,
        fit_seconds,
        polish_seconds,
        total_seconds: started.elapsed().as_secs_f64(),
        fit_profile,
    })
}
fn checked_native_problem<'a, S: SurfaceQueries>(
    surface: &'a S,
    t: &'a Topology,
    p: &'a Params,
) -> Result<Problem<'a>, OptError> {
    let fail = |message: &str| OptError {
        message: message.into(),
        best: None,
    };
    if t.tiles.is_empty() || t.corners.is_empty() {
        return Err(fail("empty topology"));
    }
    if t.tiles
        .iter()
        .any(|s| !s.depth.is_finite() || s.depth < 0. || s.depth > p.depth)
    {
        return Err(fail("seed outside depth band"));
    }
    native_problem(surface, t, p)
}
fn native_problem<'a, S: SurfaceQueries>(
    surface: &'a S,
    t: &'a Topology,
    p: &'a Params,
) -> Result<Problem<'a>, OptError> {
    let fail = |message: &str| OptError {
        message: message.into(),
        best: None,
    };
    let mut problem = Problem {
        fixed_whole: None,
        surface: Some(surface),
        t,
        p,
        points: vec![],
        normal: vec![],
        axes: vec![],
        targets: vec![],
        fans: fans(t),
    };
    for seed in &t.tiles {
        let uv = [seed.u, seed.v];
        let (_, n) = surface.frame(uv);
        problem.points.push(surface.offset(uv, seed.depth));
        problem.normal.push(n);
        problem.axes.push(basis(n));
    }
    for &[i, j, k] in &t.corners {
        let a = &t.tiles[i];
        let coords = [i, j, k].map(|index| {
            let s = &t.tiles[index];
            [a.u + (s.u - a.u + PI).rem_euclid(TAU) - PI, s.v]
        });
        let [a, b, c] = coords;
        let denominator = 2. * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
        if denominator.abs() < 1e-16 {
            return Err(fail("degenerate chart triangle"));
        }
        let sq = |v: [f64; 2]| v[0] * v[0] + v[1] * v[1];
        let u =
            (sq(a) * (b[1] - c[1]) + sq(b) * (c[1] - a[1]) + sq(c) * (a[1] - b[1])) / denominator;
        let v =
            (sq(a) * (c[0] - b[0]) + sq(b) * (a[0] - c[0]) + sq(c) * (b[0] - a[0])) / denominator;
        problem.targets.push(surface.offset(
            [u, v],
            (t.tiles[i].depth + t.tiles[j].depth + t.tiles[k].depth) / 3.,
        ));
    }
    Ok(problem)
}

/// Export exact native C3 sites and C4 fit arrays in stable order (inches).
/// Fails on degenerate chart triangles; this is diagnostic, not CoreOutput.
pub fn export_native<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
) -> Result<serde_json::Value, OptError> {
    let problem = native_problem(surface, t, p)?;
    let start = prototype_start(&problem);
    let (fit, iterations) = prototype_fit(&problem, start.clone());
    let (_, geometry) = problem.evaluate(&fit, 1.);
    let block = |x: &[f64]| {
        (0..2)
            .flat_map(|axis| (0..t.tiles.len()).map(move |i| x[2 * i + axis]))
            .collect::<Vec<_>>()
    };
    Ok(
        serde_json::json!({"identity":{"depth":p.depth,"source":"native-c3","params":p},
        "sites":t.tiles,"points":problem.points,"radial":problem.normal,
        "etheta":problem.axes.iter().map(|a|a.0).collect::<Vec<_>>(),
        "axis_b":problem.axes.iter().map(|a|a.1).collect::<Vec<_>>(),
        "targets":problem.targets,"triangles":t.corners,"start":block(&start),"fitted":block(&fit),
        "reference_corners":geometry.corners,"keep":geometry.whole_tiles.iter().enumerate().filter_map(|(i,&v)|v.then_some(i)).collect::<Vec<_>>(),
        "native_geometry":geometry,"native_iterations":iterations}),
    )
}

// Feasibility restoration must have zero target/fairness gradient. Otherwise
// their nonzero gradient balances an arbitrarily small positive angle hinge,
// leaving a nonzero violation floor even after successful early iterations.
fn polish_evaluate(problem: &Problem<'_>, x: &[f64]) -> (Vec<f64>, Planes) {
    let (mut residual, geometry) = problem.evaluate(x, 1.);
    let start = 3 * problem.targets.len();
    let suffix = x.len()
        + problem.fans.iter().map(Vec::len).sum::<usize>()
        + problem.t.neighbors.iter().map(Vec::len).sum::<usize>();
    let end = residual.len() - suffix;
    // A genuine least-squares penalty objective retains corner targets.
    // Tiny but nonzero target weight avoids replacing the fit by pure feasibility.
    for value in &mut residual[..start] {
        *value *= 1e-6;
    }
    residual[end..].fill(0.);
    (residual, geometry)
}
// Conservative symbolic row supports for the frozen-domain polish residual.
// Entire polygon blocks share the union of incident planes: reversal, inner
// footprints and support-switches in bounding_square cannot invalidate this.
struct PolishPattern {
    rows: Vec<Vec<usize>>,
    colors: Vec<Vec<usize>>,
}
impl PolishPattern {
    fn new(problem: &Problem<'_>, n: usize) -> Self {
        let mut rows = vec![vec![]; n];
        let mut row = 0;
        let mut block = |planes: &[usize], count: usize| {
            let mut planes = planes.to_vec();
            planes.sort_unstable();
            planes.dedup();
            for i in planes {
                for axis in 0..2 {
                    rows[2 * i + axis].extend(row..row + count);
                }
            }
            row += count;
        };
        for triangle in &problem.t.corners {
            block(triangle, 3);
        }
        for triangle in &problem.t.corners {
            block(triangle, 1);
        }
        for (i, fan) in problem.fans.iter().enumerate() {
            let len = fan.len();
            if len < 3 {
                continue;
            }
            let mut support = vec![i];
            for &k in fan {
                support.extend(problem.t.corners[k]);
            }
            block(&support, len * len + len + 3);
        }
        for (i, neighbors) in problem.t.neighbors.iter().enumerate() {
            for &j in neighbors {
                if j > i {
                    block(&[i, j], 1);
                }
            }
        }
        // The suffix is identically zero in polish_evaluate.
        let mut colors: Vec<Vec<usize>> = vec![];
        let mut used: Vec<std::collections::BTreeSet<usize>> = vec![];
        for (j, support) in rows.iter().enumerate() {
            let c = used
                .iter()
                .position(|set| support.iter().all(|r| !set.contains(r)))
                .unwrap_or(used.len());
            if c == used.len() {
                used.push(Default::default());
                colors.push(vec![]);
            }
            used[c].extend(support);
            colors[c].push(j);
        }
        Self { rows, colors }
    }
}
fn colored_polish_columns(
    problem: &Problem<'_>,
    x: &[f64],
    r: &[f64],
    pattern: &PolishPattern,
) -> Vec<Vec<(usize, f64)>> {
    let mut columns = vec![vec![]; x.len()];
    for color in &pattern.colors {
        let mut trial = x.to_vec();
        for &j in color {
            trial[j] += 1e-7 * (1. + x[j].abs());
        }
        let (rr, _) = polish_evaluate(problem, &trial);
        for &j in color {
            let h = 1e-7 * (1. + x[j].abs());
            columns[j] = pattern.rows[j]
                .iter()
                .filter_map(|&k| {
                    let d = (rr[k] - r[k]) / h;
                    (d.abs() > 1e-12).then_some((k, d))
                })
                .collect();
        }
    }
    columns
}
fn solve<S: SurfaceQueries>(
    surface: &S,
    problem: &Problem<'_>,
    x: Vec<f64>,
) -> Result<Planes, OptError> {
    let (_, baseline) = problem.evaluate(&x, 1.);
    solve_from_baseline(surface, problem, x, baseline)
}
fn solve_from_baseline<S: SurfaceQueries>(
    surface: &S,
    problem: &Problem<'_>,
    mut x: Vec<f64>,
    baseline: Planes,
) -> Result<Planes, OptError> {
    if baseline.report.violated.is_empty() {
        return Ok(with_distances(surface, baseline));
    }
    // Freeze the fitted measurement domain: polishing cannot hide a piece by
    // moving it across a rim. Every supporting plane remains an unknown.
    let mut frozen = problem.clone();
    frozen.fixed_whole = Some(baseline.whole_tiles.clone());
    let problem = &frozen;
    let pattern = PolishPattern::new(problem, x.len());
    let mut damping = 1e-6;
    let mut best = None;
    let mut best_rank = (usize::MAX, f64::INFINITY);
    for iteration in 0..80 {
        let (r, mut candidate) = polish_evaluate(problem, &x);
        candidate.report.iterations = iteration;
        let current_report = candidate.report.clone();
        let rank = (
            candidate.report.violated.len(),
            candidate.report.limit_violation_sum_sq,
        );
        if rank < best_rank {
            best_rank = rank;
            best = Some(candidate);
        }
        if best.as_ref().is_some_and(|g| g.report.violated.is_empty()) {
            break;
        }
        let columns = colored_polish_columns(problem, &x, &r, &pattern);
        let apply = |v: &[f64]| {
            let mut jv = vec![0.; r.len()];
            for (column, &value) in columns.iter().zip(v) {
                for &(k, d) in column {
                    jv[k] += d * value;
                }
            }
            columns
                .iter()
                .enumerate()
                .map(|(i, col)| col.iter().map(|&(k, d)| d * jv[k]).sum::<f64>() + damping * v[i])
                .collect::<Vec<_>>()
        };
        let rhs: Vec<_> = columns
            .iter()
            .map(|col| -col.iter().map(|&(k, d)| d * r[k]).sum::<f64>())
            .collect();
        let mut step = vec![0.; x.len()];
        let mut rem = rhs.clone();
        let mut direction = rem.clone();
        let mut rr = dot_slice(&rem, &rem);
        for _ in 0..80 {
            if rr < 1e-28 {
                break;
            }
            let ad = apply(&direction);
            let denom = dot_slice(&direction, &ad);
            if denom <= 0. {
                break;
            }
            let alpha = rr / denom;
            for i in 0..x.len() {
                step[i] += alpha * direction[i];
                rem[i] -= alpha * ad[i];
            }
            let next = dot_slice(&rem, &rem);
            let beta = next / rr;
            for i in 0..x.len() {
                direction[i] = rem[i] + beta * direction[i];
            }
            rr = next;
        }
        let start = 3 * problem.targets.len();
        let suffix = x.len()
            + problem.fans.iter().map(Vec::len).sum::<usize>()
            + problem.t.neighbors.iter().map(Vec::len).sum::<usize>();
        let end = r.len() - suffix;
        let mut accepted = false;
        // Backtrack rather than amplifying damping forever when a full GN
        // step crosses a previously satisfied hinge. All guards are unchanged.
        for backtrack in 0..16 {
            let scale = 0.5_f64.powi(backtrack);
            let trial: Vec<_> = x
                .iter()
                .zip(&step)
                .map(|(a, b)| a + scale * b.clamp(-0.25, 0.25))
                .collect();
            let (next, trial_geometry) = polish_evaluate(problem, &trial);
            let finite = next[start..end].iter().all(|a| a.is_finite());
            if finite
                && dot_slice(&next, &next) < dot_slice(&r, &r)
                && limits_improve(&current_report, &trial_geometry.report)
            {
                x = trial;
                damping = (damping * 0.5).max(1e-12);
                accepted = true;
                break;
            }
        }
        if !accepted {
            damping = (damping * 4.).min(1e10);
        }
    }
    // The final accepted trial is not visited by the next loop iteration.
    // Rank it at the final iteration's weight so the full fixed budget counts.
    let weight = 1.;
    let (_, mut candidate) = problem.evaluate(&x, weight);
    candidate.report.iterations = 80;
    let rank = (
        candidate.report.violated.len(),
        candidate.report.limit_violation_sum_sq,
    );
    if rank < best_rank {
        best = Some(candidate);
    }
    let proposed = best.unwrap();
    // Final guard is relative to the ORIGINAL fit, never a moving baseline.
    let result = with_distances(
        surface,
        if limits_improve(&baseline.report, &proposed.report) {
            proposed
        } else {
            baseline
        },
    );
    if result.report.violated.is_empty() {
        Ok(result)
    } else {
        Err(OptError {
            message: "A05 fixed iteration budget: limits remain violated; owner decision required"
                .into(),
            best: Some(Box::new(result)),
        })
    }
}
// Do not trade a newly broken limit for improvements elsewhere. The merit is
// independent of target fit, fairness, barrier schedule and iteration number.
fn limits_improve(current: &OptReport, trial: &OptReport) -> bool {
    trial.limit_violation_sum_sq.is_finite()
        && trial.limit_violation_sum_sq < current.limit_violation_sum_sq
        && trial.violated.len() <= current.violated.len()
        && trial
            .violated
            .iter()
            .all(|entry| current.violated.contains(entry))
}
fn with_distances<S: SurfaceQueries>(surface: &S, mut result: Planes) -> Planes {
    let mut selected = std::collections::BTreeSet::new();
    for (i, ids) in result.polygons.iter().enumerate() {
        if result.whole_tiles[i] {
            selected.extend(ids.iter().copied());
        }
    }
    let mut distances: Vec<_> = selected
        .into_iter()
        .map(|k| surface.distance(result.corners[k]))
        .collect();
    if distances.is_empty() {
        result.report.corner_distance = DistanceStats {
            min: 0.,
            max: 0.,
            median: 0.,
        };
        return result;
    }
    distances.sort_by(f64::total_cmp);
    let n = distances.len();
    result.report.corner_distance = DistanceStats {
        min: distances[0],
        max: distances[n - 1],
        median: if n % 2 == 0 {
            (distances[n / 2 - 1] + distances[n / 2]) * 0.5
        } else {
            distances[n / 2]
        },
    };
    result
}
fn dot_slice(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

// Owner-selected fitted objective, shared by production and reference diagnostics.
fn prototype_residual(problem: &Problem<'_>, x: &[f64]) -> Vec<f64> {
    let (_, corners, _) = problem.geometry(x);
    let mut r: Vec<_> = corners
        .iter()
        .zip(&problem.targets)
        .flat_map(|(&c, &t)| sub(c, t))
        .collect();
    r.extend(x.iter().map(|v| 0.05 * v));
    r
}
// Unbounded TRF, matching least_squares(resid, x0, max_nfev=300):
// two-point differences, unit scaling, exact SVD subproblem, ftol/xtol/gtol=1e-8.
// Block column order matches the reference's [all a, all b] layout.
fn prototype_fit(problem: &Problem<'_>, x: Vec<f64>) -> (Vec<f64>, usize) {
    prototype_fit_profiled(problem, x, &mut FitProfile::default())
}
fn timed_fit_residual(problem: &Problem<'_>, x: &[f64], profile: &mut FitProfile) -> Vec<f64> {
    let started = std::time::Instant::now();
    let r = prototype_residual(problem, x);
    profile.residual_calls += 1;
    profile.residual_seconds += started.elapsed().as_secs_f64();
    r
}
fn prototype_fit_profiled(
    problem: &Problem<'_>,
    mut x: Vec<f64>,
    profile: &mut FitProfile,
) -> (Vec<f64>, usize) {
    use nalgebra::{DMatrix, DVector};
    let n = x.len();
    let order: Vec<_> = (0..2)
        .flat_map(|axis| (0..n / 2).map(move |i| 2 * i + axis))
        .collect();
    let mut radius = dot_slice(&x, &x).sqrt();
    if radius == 0. {
        radius = 1.;
    }
    let mut alpha = 0.;
    let mut nfev = 1;
    let mut r = timed_fit_residual(problem, &x, profile);
    loop {
        let cost = 0.5 * dot_slice(&r, &r);
        let jac_start = std::time::Instant::now();
        let mut jac = DMatrix::zeros(r.len(), n);
        for (col, &j) in order.iter().enumerate() {
            let mut trial = x.clone();
            let h = f64::EPSILON.sqrt() * if x[j] < 0. { -1. } else { 1. } * x[j].abs().max(1.);
            trial[j] += h;
            let dx = trial[j] - x[j];
            let rr = timed_fit_residual(problem, &trial, profile);
            for k in 0..r.len() {
                jac[(k, col)] = (rr[k] - r[k]) / dx;
            }
        }
        profile.jacobians += 1;
        profile.jacobian_seconds += jac_start.elapsed().as_secs_f64();
        let rv = DVector::from_column_slice(&r);
        let gradient = jac.transpose() * &rv;
        if gradient.iter().map(|v| v.abs()).fold(0., f64::max) < 1e-8 || nfev == 300 {
            profile.nfev = nfev;
            return (x, nfev);
        }
        let svd_start = std::time::Instant::now();
        // Preserve the original tall-SVD floating-point trajectory. QR reduction
        // is equivalent in exact arithmetic but perturbs TRF steps enough to
        // change the basin and frozen whole-tile domain on organic cylinders.
        // Keep single-fit reuse and exact colored polish probes independently.
        let svd = jac.clone().svd(true, true);
        profile.svds += 1;
        profile.svd_seconds += svd_start.elapsed().as_secs_f64();
        let singular = svd.singular_values;
        let uf = svd.u.unwrap().transpose() * rv;
        let v = svd.v_t.unwrap().transpose();
        let suf = singular.component_mul(&uf);
        let full_rank = singular[n - 1] > f64::EPSILON * r.len() as f64 * singular[0];
        let mut stop = false;
        while nfev < 300 {
            let gn = -(&v * uf.component_div(&singular));
            let step = if full_rank && gn.norm() <= radius {
                alpha = 0.;
                gn
            } else {
                let phi = |a: f64| {
                    let denom = singular.map(|s| s * s + a);
                    let norm = suf.component_div(&denom).norm();
                    let derivative = -suf
                        .iter()
                        .zip(denom.iter())
                        .map(|(u, d)| u * u / (d * d * d))
                        .sum::<f64>()
                        / norm;
                    (norm - radius, derivative)
                };
                let mut upper = suf.norm() / radius;
                let mut lower = if full_rank {
                    let (f, d) = phi(0.);
                    -f / d
                } else {
                    0.
                };
                if !full_rank && alpha == 0. {
                    alpha = (0.001 * upper).max((lower * upper).sqrt());
                }
                for _ in 0..10 {
                    if alpha < lower || alpha > upper {
                        alpha = (0.001 * upper).max((lower * upper).sqrt());
                    }
                    let (f, d) = phi(alpha);
                    if f < 0. {
                        upper = alpha;
                    }
                    let ratio = f / d;
                    lower = lower.max(alpha - ratio);
                    alpha -= (f + radius) * ratio / radius;
                    if f.abs() < 0.01 * radius {
                        break;
                    }
                }
                let mut step = -(&v * suf.component_div(&singular.map(|s| s * s + alpha)));
                step *= radius / step.norm();
                step
            };
            let step_norm = step.norm();
            let predicted = -(0.5 * (&jac * &step).norm_squared() + gradient.dot(&step));
            let mut trial = x.clone();
            for (col, &j) in order.iter().enumerate() {
                trial[j] += step[col];
            }
            let rr = timed_fit_residual(problem, &trial, profile);
            nfev += 1;
            if rr.iter().any(|v| !v.is_finite()) {
                radius = 0.25 * step_norm;
                continue;
            }
            let reduction = cost - 0.5 * dot_slice(&rr, &rr);
            let ratio = if predicted > 0. {
                reduction / predicted
            } else if predicted == 0. && reduction == 0. {
                1.
            } else {
                0.
            };
            let next_radius = if ratio < 0.25 {
                0.25 * step_norm
            } else if ratio > 0.75 && step_norm > 0.95 * radius {
                2. * radius
            } else {
                radius
            };
            stop = (reduction < 1e-8 * cost && ratio > 0.25)
                || step_norm < 1e-8 * (1e-8 + dot_slice(&x, &x).sqrt());
            if !stop {
                alpha *= radius / next_radius;
                radius = next_radius;
            }
            if reduction > 0. {
                x = trial;
                r = rr;
                break;
            }
            if stop {
                break;
            }
        }
        if stop || nfev == 300 {
            profile.nfev = nfev;
            return (x, nfev);
        }
    }
}
fn prototype_start(problem: &Problem<'_>) -> Vec<f64> {
    let mut x = vec![0.; 2 * problem.points.len()];
    for i in 0..problem.points.len() {
        let (a, b) = problem.axes[i];
        let rows: Vec<_> = problem
            .t
            .corners
            .iter()
            .enumerate()
            .filter(|(_, ids)| ids.contains(&i))
            .map(|(k, _)| sub(problem.targets[k], problem.points[i]))
            .collect();
        let matrix = nalgebra::DMatrix::from_fn(rows.len(), 2, |k, j| {
            dot(rows[k], if j == 0 { a } else { b })
        });
        let rhs = nalgebra::DVector::from_iterator(
            rows.len(),
            rows.iter().map(|&d| -dot(d, problem.normal[i])),
        );
        let threshold = f64::EPSILON * rows.len().max(2) as f64;
        let svd = matrix.svd(true, true);
        let cutoff = threshold * svd.singular_values[0];
        let solution = svd.solve(&rhs, cutoff).expect("finite seed least squares");
        x[2 * i] = solution[0];
        x[2 * i + 1] = solution[1];
    }
    x
}

/// Exact reference-prototype input (inches); diagnostics only, never a production contract.
#[derive(serde::Deserialize)]
pub struct ReferenceInput {
    pub points: Vec<V>,
    pub radial: Vec<V>,
    pub etheta: Vec<V>,
    #[serde(default)]
    pub axis_b: Vec<V>,
    pub targets: Vec<V>,
    pub triangles: Vec<[usize; 3]>,
    pub start: Vec<f64>,
    pub fitted: Vec<f64>,
    pub reference_corners: Vec<V>,
    pub keep: Vec<usize>,
}
/// Reconstruct reference geometry and optionally continue barriers from its exact fitted solution.
/// Rejects inconsistent array lengths; all coordinates and residuals are in inches.
pub fn reference_stage<S: SurfaceQueries>(
    surface: &S,
    p: &Params,
    input: &ReferenceInput,
    stage: &str,
) -> Result<Planes, OptError> {
    let n = input.points.len();
    assert_eq!(input.radial.len(), n);
    assert_eq!(input.etheta.len(), n);
    assert_eq!(input.start.len(), 2 * n);
    assert_eq!(input.fitted.len(), 2 * n);
    assert_eq!(input.targets.len(), input.triangles.len());
    let mut neighbors = vec![vec![]; n];
    for ids in &input.triangles {
        for &i in ids {
            for &j in ids {
                if i != j && !neighbors[i].contains(&j) {
                    neighbors[i].push(j);
                }
            }
        }
    }
    let mut t = Topology {
        tiles: (0..n)
            .map(|_| crate::seeds::Seed {
                u: 0.,
                v: 0.,
                s: 0.,
                row: 0,
                auxiliary: false,
                depth: 0.,
            })
            .collect(),
        neighbors,
        corners: input.triangles.clone(),
        boundary: vec![false; n],
    };
    // Only whole tiles selected by the actual prototype are measured as pieces.
    for i in 0..n {
        t.boundary[i] = !input.keep.contains(&i);
    }
    let problem = Problem {
        fixed_whole: None,
        surface: None,
        t: &t,
        p,
        points: input.points.clone(),
        normal: input.radial.clone(),
        axes: input
            .etheta
            .iter()
            .enumerate()
            .map(|(i, &a)| (a, input.axis_b.get(i).copied().unwrap_or([0., 1., 0.])))
            .collect(),
        targets: input.targets.clone(),
        fans: fans(&t),
    };
    let raw = if stage == "start" {
        &input.start
    } else {
        &input.fitted
    };
    let mut x: Vec<_> = (0..n).flat_map(|i| [raw[i], raw[n + i]]).collect();
    let mut iterations = 0;
    if stage.starts_with("native-") {
        x = prototype_start(&problem);
        if stage != "native-start" {
            (x, iterations) = prototype_fit(&problem, x);
        }
    }
    if stage == "barriers" || stage == "native-barriers" {
        return solve(surface, &problem, x);
    }
    let (_, mut geometry) = problem.evaluate(&x, 1.);
    geometry.report.iterations = iterations;

    Ok(with_distances(surface, geometry))
}

#[cfg(test)]
mod polishing_tests {
    #[test]
    fn qr_reduced_svd_preserves_damped_steps() {
        use nalgebra::{DMatrix, DVector};
        // Tall, ill-conditioned but full-rank: normal equations would square
        // this condition number. Include a component outside range(J).
        let a = DMatrix::from_fn(43, 7, |i, j| {
            ((i * 17 + j * 31 + 1) as f64).sin() * 10_f64.powi(-(j as i32))
                + if i == j { 10_f64.powi(-(j as i32)) } else { 0. }
        });
        let rhs = DVector::from_fn(43, |i, _| (i as f64).cos());
        let dense = a.clone().svd(true, true);
        let qr = a.qr();
        let mut transformed = rhs.clone();
        qr.q_tr_mul(&mut transformed);
        let reduced = qr.r().svd(true, true);
        let project = |svd: &nalgebra::linalg::SVD<f64, nalgebra::Dyn, nalgebra::Dyn>,
                       r: &DVector<f64>,
                       alpha: f64| {
            let uf = svd.u.as_ref().unwrap().transpose() * r;
            svd.v_t.as_ref().unwrap().transpose()
                * svd
                    .singular_values
                    .component_mul(&uf)
                    .component_div(&svd.singular_values.map(|s| s * s + alpha))
        };
        for alpha in [0., 1e-12, 1e-6, 1.] {
            let direct = project(&dense, &rhs, alpha);
            let compact = project(&reduced, &transformed.rows(0, 7).into_owned(), alpha);
            assert!((&direct - compact).norm() <= 1e-8 * direct.norm().max(1.));
        }
    }
    #[test]
    fn colored_polish_matches_every_scalar_probe() {
        let input: crate::model::CoreInput = serde_json::from_str(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        let surface = crate::surface::Surface::from_profile(&input.surface.profile).unwrap();
        let topology = crate::sidecount::build(&surface, &input.params).unwrap();
        let mut problem = super::native_problem(&surface, &topology, &input.params).unwrap();
        let start = super::prototype_start(&problem);
        let (_, baseline) = problem.evaluate(&start, 1.);
        problem.fixed_whole = Some(baseline.whole_tiles);
        let pattern = super::PolishPattern::new(&problem, start.len());
        assert!(
            pattern.colors.len() < start.len() / 2,
            "coloring must reduce evaluations"
        );
        for scale in [0., 0.02, 0.3] {
            let x: Vec<_> = start
                .iter()
                .enumerate()
                .map(|(j, a)| a + scale * (j as f64).sin())
                .collect();
            let (r, _) = super::polish_evaluate(&problem, &x);
            let colored = super::colored_polish_columns(&problem, &x, &r, &pattern);
            for j in 0..x.len() {
                let mut trial = x.clone();
                let h = 1e-7 * (1. + x[j].abs());
                trial[j] += h;
                let (rr, _) = super::polish_evaluate(&problem, &trial);
                let scalar: Vec<_> = rr
                    .iter()
                    .zip(&r)
                    .enumerate()
                    .filter_map(|(k, (a, b))| {
                        let d = (a - b) / h;
                        (d.abs() > 1e-12).then_some((k, d))
                    })
                    .collect();
                assert_eq!(colored[j], scalar, "column {j}, displacement {scale}");
            }
        }
    }

    use super::*;
    struct Flat;
    impl SurfaceQueries for Flat {
        fn frame(&self, _: [f64; 2]) -> (V, V) {
            ([0.; 3], [0., 1., 0.])
        }
        fn distance(&self, p: V) -> f64 {
            p[1]
        }
    }
    #[test]
    fn feasible_fit_is_byte_identical_fixed_point() {
        let mut p: Params = serde_json::from_str::<crate::model::CoreInput>(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap()
        .params;
        p.min_edge = 0.01;
        p.max_edge = 100.;
        p.min_corner_angle_deg = 1.;
        p.max_corner_angle_deg = 269.;
        p.bevel_cap_deg = 90.;
        p.min_bounding_square = 0.01;
        p.blank_usable = 100.;
        p.thickness = 0.01;
        let normals = vec![
            unit([1., 1., 1.]),
            unit([1., -1., -1.]),
            unit([-1., 1., -1.]),
            unit([-1., -1., 1.]),
        ];
        let seed = crate::seeds::Seed {
            u: 0.,
            v: 0.,
            s: 0.,
            row: 0,
            auxiliary: false,
            depth: 0.,
        };
        let t = Topology {
            tiles: vec![seed; 4],
            neighbors: (0..4)
                .map(|i| (0..4).filter(|&j| j != i).collect())
                .collect(),
            corners: vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]],
            boundary: vec![false; 4],
        };
        let problem = Problem {
            fixed_whole: None,
            surface: None,
            t: &t,
            p: &p,
            points: normals.clone(),
            axes: normals.iter().map(|&n| basis(n)).collect(),
            normal: normals,
            targets: vec![[0.; 3]; 4],
            fans: fans(&t),
        };
        let x = vec![0.; 8];
        let (fit_residual, raw) = problem.evaluate(&x, 1.);
        assert!(raw.report.violated.is_empty(), "{:?}", raw.report);
        // Targets are deliberately different from this feasible shell. They
        // must not exert a competing gradient during feasibility restoration.
        assert!(dot_slice(&fit_residual, &fit_residual) > 0.);
        let (feasibility_residual, _) = polish_evaluate(&problem, &x);
        assert!(
            dot_slice(&feasibility_residual, &feasibility_residual)
                < 1e-12 * dot_slice(&fit_residual, &fit_residual)
        );
        assert!(raw.polygons.iter().all(|v| v.len() == 3));
        // Excluding pieces must not discard supporting planes or corners, and
        // a corner incident to any retained whole piece remains measured.
        let mut excluded = raw.clone();
        excluded.whole_tiles.fill(false);
        let empty = with_distances(&Flat, excluded);
        assert_eq!(empty.corners, raw.corners);
        assert_eq!(
            serde_json::to_vec(&empty.planes).unwrap(),
            serde_json::to_vec(&raw.planes).unwrap()
        );
        assert_eq!(
            empty.report.corner_distance,
            DistanceStats {
                min: 0.,
                max: 0.,
                median: 0.
            }
        );
        let baseline = with_distances(&Flat, raw);
        let polished = solve(&Flat, &problem, x).unwrap();
        assert_eq!(
            serde_json::to_vec(&baseline).unwrap(),
            serde_json::to_vec(&polished).unwrap()
        );
    }
    #[test]
    fn never_trade_limits_for_target_cost_or_new_broken_limits() {
        let report = |score, violated: Vec<(usize, String)>| OptReport {
            limit_violation_sum_sq: score,
            iterations: 0,
            corner_distance: DistanceStats {
                min: 0.,
                max: 0.,
                median: 0.,
            },
            reflex_corners: 0,
            violated,
        };
        let a = report(2., vec![(0, "max_edge".into())]);
        assert!(!limits_improve(&a, &report(3., a.violated.clone())));
        assert!(!limits_improve(
            &a,
            &report(1., vec![(1, "bevel_cap_deg".into())])
        ));
        assert!(limits_improve(&a, &report(1., a.violated.clone())));
        assert!(!limits_improve(&a, &a));
        assert!(limits_improve(&a, &report(0., vec![])));
    }
}

#[path = "planes_conformal_experiment.rs"]
mod conformal_experiment;
pub use conformal_experiment::experimental_conformal_penalty;
