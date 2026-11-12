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
    pub report: OptReport,
}
#[derive(Clone, Debug, Serialize)]
pub struct OptError {
    pub message: String,
    pub best: Option<Box<Planes>>,
}
struct Problem<'a> {
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
        let mut reflex = 0;
        let mut polygons = self.fans.clone();
        for (v, target) in corners.iter().zip(&self.targets) {
            residual.extend(sub(*v, *target).map(|a| 0.03 * a));
        }
        let mut barrier = |tile: usize, name: &str, value: f64| {
            residual.push(weight.sqrt() * value.max(0.));
            if value > 1e-7 {
                violated.push((tile, name.to_string()));
            }
        };
        for (k, &d) in dets.iter().enumerate() {
            barrier(self.t.corners[k][0], "determinant", (1e-8 - d) * 1e8);
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
                reflex += usize::from(angle > 180. + 1e-7);
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
                barrier(i, "bevel_cap_deg", (angle - self.p.bevel_cap_deg) / 15.);
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
                report: OptReport {
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
/// Fit planes through inward-offset seeds and optimize deterministically (80 LM steps).
/// Failure preserves the best complete geometry/report; does not certify C5 inner footprints.
pub fn optimize<S: SurfaceQueries>(
    surface: &S,
    t: &Topology,
    p: &Params,
    _rng: &mut rand_chacha::ChaCha8Rng,
) -> Result<Planes, OptError> {
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
    let mut problem = Problem {
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
    let mut x = vec![0.; 2 * t.tiles.len()];
    for i in 0..t.tiles.len() {
        let (a, b) = problem.axes[i];
        let mut aa = 1e-10;
        let mut ab = 0.;
        let mut bb = 1e-10;
        let mut ar = 0.;
        let mut br = 0.;
        for (k, c) in t.corners.iter().enumerate() {
            if !c.contains(&i) {
                continue;
            }
            let d = sub(problem.targets[k], problem.points[i]);
            let u = dot(d, a);
            let v = dot(d, b);
            let r = -dot(d, problem.normal[i]);
            aa += u * u;
            ab += u * v;
            bb += v * v;
            ar += u * r;
            br += v * r;
        }
        let det = aa * bb - ab * ab;
        x[2 * i] = (ar * bb - br * ab) / det;
        x[2 * i + 1] = (br * aa - ar * ab) / det;
    }
    let mut damping = 1e-2;
    let mut best = None;
    let mut best_rank = (usize::MAX, f64::INFINITY);
    for iteration in 0..80 {
        let weight = 1e-3 * 2f64.powi((iteration / 5) as i32);
        let (r, mut candidate) = problem.evaluate(&x, weight);
        let cost = dot_slice(&r, &r);
        candidate.report.iterations = iteration;
        let rank = (candidate.report.violated.len(), cost / weight);
        if rank < best_rank {
            best_rank = rank;
            best = Some(candidate);
        }
        let columns: Vec<Vec<(usize, f64)>> = (0..x.len())
            .map(|j| {
                let mut trial = x.clone();
                let h = 1e-6 * (1. + x[j].abs());
                trial[j] += h;
                let (rr, _) = problem.evaluate(&trial, weight);
                rr.iter()
                    .zip(&r)
                    .enumerate()
                    .filter_map(|(k, (a, b))| {
                        let d = (a - b) / h;
                        if d.abs() > 1e-12 {
                            Some((k, d))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();
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
            if rr < 1e-18 {
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
        let trial: Vec<_> = x
            .iter()
            .zip(step)
            .map(|(a, b)| a + b.clamp(-0.25, 0.25))
            .collect();
        let (next, _) = problem.evaluate(&trial, weight);
        if dot_slice(&next, &next) < cost {
            x = trial;
            damping = (damping * 0.5).max(1e-8);
        } else {
            damping = (damping * 4.).min(1e10);
        }
    }
    let mut result = best.unwrap();
    let mut distances: Vec<_> = result
        .corners
        .iter()
        .map(|&v| surface.distance(v))
        .collect();
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
fn dot_slice(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
