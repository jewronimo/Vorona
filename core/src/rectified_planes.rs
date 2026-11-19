//! A06 two-plane-line solver. Internal C4 geometry, not C5 manufacturing output.
use crate::{
    model::{Params, Plane},
    planes::{bounding_square, SurfaceQueries},
    rectified::Rectified,
};
use nalgebra::{Matrix3, Vector3};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    f64::consts::TAU,
};
type V = [f64; 3];
type Violations = Vec<(usize, String)>;
type Geometry = (Vec<Plane>, Vec<V>);
type Evaluation = (Vec<f64>, Solution, Violations);
type LocalBest = (f64, Solution, Violations, usize);
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
fn unit(a: V) -> Result<V, String> {
    let l = norm(a);
    if !l.is_finite() || l < 1e-14 {
        Err("degenerate direction".into())
    } else {
        Ok(mul(a, 1. / l))
    }
}
fn basis(n: V) -> (V, V) {
    let a = unit(cross(
        if n[1].abs() < 0.9 {
            [0., 1., 0.]
        } else {
            [1., 0., 0.]
        },
        n,
    ))
    .unwrap();
    (a, cross(n, a))
}
/// Generic rigid periodic action supplied by the surface adapter, not C4.
/// Angles are chart radians. A non-symmetric surface must reject symmetric use.
pub trait PeriodicSymmetry: SurfaceQueries {
    /// Reflect a vector in the chart's zero-angle meridian; reject unsupported surfaces.
    fn reflect_vector(&self, _vector: V) -> Result<V, String> {
        Err("surface does not support meridian reflection".into())
    }
    /// Reflect an inch-valued point in that same meridian.
    fn reflect_point(&self, point: V) -> Result<V, String> {
        self.reflect_vector(point)
    }
    /// Original physical end planes in inches; generic unbounded charts have none.
    fn physical_rims(&self) -> Option<[f64; 2]> {
        None
    }
    /// Rotate a vector between equivalent chart positions, or reject the action.
    fn rotate_vector(&self, vector: V, angle: f64) -> Result<V, String>;
    /// Transform an inch-valued point by the periodic rigid action.
    fn rotate_point(&self, point: V, angle: f64) -> Result<V, String>;
}
fn delta(a: f64, b: f64) -> f64 {
    (a - b + TAU / 2.).rem_euclid(TAU) - TAU / 2.
}
fn directed(t: &Rectified, e: [usize; 2]) -> [usize; 2] {
    let [i, j] = e;
    let seeds = &t.delaunay.tiles;
    if seeds[i].row == seeds[j].row && delta(seeds[j].u, seeds[i].u) < 0. {
        [j, i]
    } else {
        e
    }
}
/// Exact nearest point on two distinct planes plus an inch-valued line slide.
/// Rejects parallel/ill-conditioned lines instead of regularizing a determinant.
pub fn corner(a: &Plane, b: &Plane, target: V, slide: f64) -> Result<V, String> {
    let w = cross(a.n, b.n);
    let det = dot(w, w);
    if !det.is_finite() || det < 1e-20 {
        return Err("parallel two-plane corner".into());
    }
    // Cross-product dual basis avoids cancellation in 1-(ni.nj)^2.
    let r1 = a.d - dot(a.n, target);
    let r2 = b.d - dot(b.n, target);
    Ok(add(
        add(
            target,
            mul(
                add(mul(cross(b.n, w), r1), mul(cross(w, a.n), r2)),
                1. / det,
            ),
        ),
        mul(w, slide / det.sqrt()),
    ))
}
/// Per-row A06 §8.9 gate. Transition/boundary/random rows make no symmetry claim.
/// Distances and size errors are inches; absent measurements are not passes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RowGate {
    pub row: usize,
    pub count: usize,
    pub repeats: usize,
    pub neighbor_counts: [Option<usize>; 2],
    pub mode: String,
    pub required: bool,
    pub representative: Option<usize>,
    pub rotation_error: Option<f64>,
    pub spacing_error: Option<f64>,
    pub size_error: Option<f64>,
    pub passed: Option<bool>,
}
/// Classify all rows before optimization, including unmeasured free transitions.
/// Counts are dimensionless; no geometric symmetry is claimed by classification.
pub fn symmetry_policy(t: &Rectified, p: &Params) -> Vec<RowGate> {
    t.rows
        .iter()
        .enumerate()
        .map(|(row, r)| {
            let before = row.checked_sub(1).map(|j| t.rows[j].count);
            let after = t.rows.get(row + 1).map(|v| v.count);
            let constant =
                before.is_none_or(|n| n == r.count) && after.is_none_or(|n| n == r.count);
            let required = p.randomness == 0.;
            let repeats = if !required {
                1
            } else if constant {
                r.count
            } else {
                p.count_step
            };
            let mode = if !required {
                "experimental_random"
            } else if constant {
                "row_representative_copy"
            } else {
                "sector_copy"
            };
            RowGate {
                row,
                count: r.count,
                repeats,
                neighbor_counts: [before, after],
                mode: mode.into(),
                required,
                representative: if required {
                    t.delaunay.tiles.iter().position(|v| v.row == row)
                } else {
                    None
                },
                rotation_error: None,
                spacing_error: None,
                size_error: None,
                passed: None,
            }
        })
        .collect()
}
/// Gated row corner, matching representative corner and periodic rigid angle.
type CornerCopy = (usize, usize, usize, f64);
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Attempt {
    pub index: usize,
    pub corner_distance: Option<[f64; 3]>,
    pub symmetry_rows: Vec<RowGate>,
    pub iterations: usize,
    pub objective: f64,
    pub violations: Vec<(usize, String)>,
    pub error: Option<String>,
    /// Signed configured-limit deficits by name: min/max; positive fails.
    pub limit_deficits: BTreeMap<String, [f64; 2]>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Solution {
    pub planes: Vec<Plane>,
    pub symmetry_rows: Vec<RowGate>,
    pub corners: Vec<V>,
    pub polygons: Vec<Vec<usize>>,
    pub inner: Vec<Vec<V>>,
    pub whole: Vec<bool>,
    pub attempts: Vec<Attempt>,
    /// Stable start identifier (`Attempt::index`), not a vector offset.
    pub best_attempt: usize,
    pub corner_distance: [f64; 3],
    pub relief_base: [f64; 3],
    pub seed_spacing: f64,
    pub bounding_squares: Vec<f64>,
    pub c5_pending: bool,
    pub unknowns: usize,
    pub jacobian_colors: usize,
    pub limit_deficits: BTreeMap<String, [f64; 2]>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Failure {
    pub message: String,
    pub best: Option<Box<Solution>>,
    pub attempts: Vec<Attempt>,
}
struct Problem<'a, S> {
    s: &'a S,
    t: &'a Rectified,
    p: &'a Params,
    points: Vec<V>,
    normals: Vec<V>,
    axes: Vec<(V, V)>,
    targets: Vec<V>,
    map: Vec<usize>,
    nv: usize,
    rows: Vec<RowGate>,
    copies: Vec<CornerCopy>,
    plane_reps: Vec<usize>,
    corner_reps: Vec<(usize, f64)>,
    slide_scale: Vec<f64>,
}
impl<'a, S: PeriodicSymmetry> Problem<'a, S> {
    fn new(s: &'a S, t: &'a Rectified, p: &'a Params) -> Result<Self, String> {
        let seeds = &t.delaunay.tiles;
        let n = seeds.len();
        let mut points = Vec::new();
        let mut normals = Vec::new();
        let mut axes = Vec::new();
        for seed in seeds {
            let (q, normal) = s.frame([seed.u, seed.v]);
            points.push(sub(q, mul(normal, seed.depth)));
            normals.push(normal);
            // Transport the zero-meridian basis, rather than switching the
            // arbitrary Cartesian basis on steep profile sections.
            let axes_at_seed = if p.randomness == 0. {
                let (_, meridian_normal) = s.frame([0., seed.v]);
                let (a, b) = basis(meridian_normal);
                (s.rotate_vector(a, seed.u)?, s.rotate_vector(b, seed.u)?)
            } else {
                basis(normal)
            };
            axes.push(axes_at_seed);
        }
        // Circumcentres in an edge-local unwrapped periodic chart.
        let mut targets = Vec::new();
        for e in &t.corners {
            let [i, j] = e.seeds;
            let reference = seeds[i].u;
            let mut centers = Vec::new();
            for &piece in e.pieces.iter().skip(2) {
                let ids = t.delaunay.corners[piece - n];
                let q = ids.map(|k| [reference + delta(seeds[k].u, reference), seeds[k].v]);
                let a = [q[1][0] - q[0][0], q[1][1] - q[0][1]];
                let b = [q[2][0] - q[0][0], q[2][1] - q[0][1]];
                let det = 2. * (a[0] * b[1] - a[1] * b[0]);
                if det.abs() < 1e-16 {
                    return Err("degenerate chart triangle".into());
                }
                let aa = a[0] * a[0] + a[1] * a[1];
                let bb = b[0] * b[0] + b[1] * b[1];
                centers.push([
                    q[0][0] + (aa * b[1] - bb * a[1]) / det,
                    q[0][1] + (a[0] * bb - b[0] * aa) / det,
                ]);
            }
            let uv = [
                centers.iter().map(|v| v[0]).sum::<f64>() / centers.len() as f64,
                centers.iter().map(|v| v[1]).sum::<f64>() / centers.len() as f64,
            ];
            targets.push(s.offset(uv, (seeds[i].depth + seeds[j].depth) / 2.));
        }
        let rows = symmetry_policy(t, p);
        let mut plane_reps = Vec::new();
        let mut row_ids = vec![Vec::new(); rows.len()];
        for (i, seed) in seeds.iter().enumerate() {
            row_ids[seed.row].push(i);
        }
        for (i, seed) in seeds.iter().enumerate() {
            let row = &rows[seed.row];
            if !row.count.is_multiple_of(row.repeats) {
                return Err("count_step does not divide transition count".into());
            }
            let local = row_ids[seed.row].iter().position(|&k| k == i).unwrap();
            plane_reps.push(row_ids[seed.row][local % (row.count / row.repeats)]);
        }
        let mut copies = Vec::new();
        for (i, seed) in seeds.iter().enumerate() {
            if !rows[seed.row].required {
                continue;
            }
            let r = plane_reps[i];
            let key = |k: usize, own: usize| {
                let [a, b] = t.corners[k].seeds;
                let other = if a == own { b } else { a };
                (
                    seeds[other].row,
                    (delta(seeds[other].u, seeds[own].u) * 1e8).round() as i64,
                )
            };
            let reference: BTreeMap<_, _> = t.polygons[r].iter().map(|&k| (key(k, r), k)).collect();
            for &k in &t.polygons[i] {
                let &reference_corner = reference
                    .get(&key(k, i))
                    .ok_or("row/sector topology does not repeat")?;
                copies.push((seed.row, k, reference_corner, seed.u - seeds[r].u));
            }
        }
        let mut map = Vec::new();
        let mut groups = BTreeMap::new();
        for &rep in &plane_reps {
            for k in 0..2 {
                let k = if p.randomness == 0. { 1 } else { k };
                let next = groups.len();
                map.push(*groups.entry((0, rep, k, 0)).or_insert(next));
            }
        }
        let edge_ids: BTreeMap<_, _> = t
            .corners
            .iter()
            .enumerate()
            .map(|(k, e)| (e.seeds, k))
            .collect();
        let mut corner_reps = Vec::new();
        for e in &t.corners {
            let [i, j] = e.seeds;
            let ri = seeds[i].row;
            let rj = seeds[j].row;
            let mut a = rows[ri].repeats;
            let mut b = rows[rj].repeats;
            while b != 0 {
                let r = a % b;
                a = b;
                b = r;
            }
            let order = a;
            let ci = row_ids[ri].iter().position(|&k| k == i).unwrap();
            let cj = row_ids[rj].iter().position(|&k| k == j).unwrap();
            let mut rep = *edge_ids.get(&e.seeds).unwrap();
            let mut angle = 0.;
            for k in 0..order {
                let x = row_ids[ri][(ci + k * rows[ri].count / order) % rows[ri].count];
                let y = row_ids[rj][(cj + k * rows[rj].count / order) % rows[rj].count];
                let key = [x.min(y), x.max(y)];
                let &candidate = edge_ids
                    .get(&key)
                    .ok_or("Delaunay corner orbit is not sector-periodic")?;
                if candidate < rep {
                    rep = candidate;
                    angle = -(k as f64) * TAU / order as f64;
                }
            }
            corner_reps.push((rep, angle));
            let next = groups.len();
            map.push(*groups.entry((1, rep, 0, 0)).or_insert(next));
        }
        // Reflection reverses the orientation of the two-plane line. Pair the
        // signed slide variables, not the resulting positions (no averaging).
        let mut slide_scale = vec![1.; t.corners.len()];
        if p.randomness == 0. {
            let reflected_seed: Vec<usize> = seeds
                .iter()
                .map(|seed| {
                    row_ids[seed.row]
                        .iter()
                        .copied()
                        .min_by(|&a, &b| {
                            delta(seeds[a].u, -seed.u)
                                .abs()
                                .total_cmp(&delta(seeds[b].u, -seed.u).abs())
                        })
                        .unwrap()
                })
                .collect();
            let old_map = map.clone();
            for (k, e) in t.corners.iter().enumerate() {
                let [a, b] = e.seeds.map(|i| reflected_seed[i]);
                let reflected = *edge_ids
                    .get(&[a.min(b), a.max(b)])
                    .ok_or("Delaunay topology does not admit meridian reflection")?;
                let (r, angle) = corner_reps[reflected];
                let (own, _) = corner_reps[k];
                let [i, j] = directed(t, t.corners[own].seeds);
                let [a, b] = directed(t, t.corners[r].seeds);
                // At zero sideways tilt the line's reflection parity is fixed,
                // independent of the meridional tilt magnitude.
                let w = cross(normals[i], normals[j]);
                let reflected_w = s.rotate_vector(s.reflect_vector(w)?, -angle)?;
                let sign = if dot(reflected_w, cross(normals[a], normals[b])) < 0. {
                    -1.
                } else {
                    1.
                };
                if r < own {
                    map[2 * n + k] = old_map[2 * n + r];
                    slide_scale[k] = sign;
                } else if r == own && sign < 0. {
                    slide_scale[k] = 0.;
                }
            }
            for (k, &scale) in slide_scale.iter().enumerate() {
                if scale == 0. {
                    // This slot is multiplied by zero; do not retain a dead
                    // optimization variable for an on-axis corner.
                    map[2 * n + k] = map[0];
                }
            }
            // Compact away variables eliminated by reflection constraints.
            let mut compact = BTreeMap::new();
            for index in &mut map {
                let next = compact.len();
                *index = *compact.entry(*index).or_insert(next);
            }
        }
        let nv = map.iter().copied().max().map_or(0, |v| v + 1);
        Ok(Self {
            s,
            t,
            p,
            points,
            normals,
            axes,
            targets,
            map,
            nv,
            rows,
            copies,
            plane_reps,
            corner_reps,
            slide_scale,
        })
    }
    fn geometry(&self, x: &[f64]) -> Result<Geometry, String> {
        let mut planes: Vec<Plane> = Vec::new();
        for (i, seed) in self.t.delaunay.tiles.iter().enumerate() {
            let r = self.plane_reps[i];
            let n = if r != i {
                self.s
                    .rotate_vector(planes[r].n, seed.u - self.t.delaunay.tiles[r].u)?
            } else {
                unit(add(
                    self.normals[i],
                    add(
                        mul(
                            self.axes[i].0,
                            if self.p.randomness == 0. {
                                0.
                            } else {
                                x[self.map[2 * i]]
                            },
                        ),
                        mul(self.axes[i].1, x[self.map[2 * i + 1]]),
                    ),
                ))?
            };
            planes.push(Plane {
                n,
                d: dot(n, self.points[i]),
            });
        }
        let n = planes.len();
        let mut corners = Vec::new();
        for (k, e) in self.t.corners.iter().enumerate() {
            let [i, j] = directed(self.t, e.seeds);
            let (rep, angle) = self.corner_reps[k];
            let point = if rep != k {
                self.s.rotate_point(corners[rep], angle)?
            } else {
                corner(
                    &planes[i],
                    &planes[j],
                    self.targets[k],
                    self.slide_scale[k] * x[self.map[2 * n + k]],
                )?
            };
            if !point.iter().all(|v| v.is_finite()) {
                return Err("nonfinite corner".into());
            }
            corners.push(point);
        }
        Ok((planes, corners))
    }
    // Unordered corner-set reflection in the stone's vertical chart meridian.
    // Checks polygons AND interstitial triangles; rotation alone is insufficient.
    fn mirror_error(&self, piece: usize, ids: &[usize], corners: &[V]) -> Result<f64, String> {
        let seeds = &self.t.delaunay.tiles;
        let n = seeds.len();
        let angles: Vec<f64> = if piece < n {
            vec![seeds[piece].u]
        } else {
            self.t.delaunay.corners[piece - n]
                .iter()
                .map(|&i| seeds[i].u)
                .collect()
        };
        let angle = angles
            .iter()
            .map(|a| a.sin())
            .sum::<f64>()
            .atan2(angles.iter().map(|a| a.cos()).sum::<f64>());
        let mut error = 0.0_f64;
        for &k in ids {
            let local = self.s.rotate_point(corners[k], -angle)?;
            let reflected = self.s.rotate_point(self.s.reflect_point(local)?, angle)?;
            let distance = ids
                .iter()
                .map(|&j| norm(sub(reflected, corners[j])))
                .fold(f64::INFINITY, f64::min);
            error = error.max(distance);
        }
        Ok(error)
    }
    fn symmetry_rows(&self, corners: &[V], squares: &[f64]) -> Result<Vec<RowGate>, String> {
        let mut rows = self.rows.clone();
        for row in &mut rows {
            let Some(_r) = row.representative else {
                continue;
            };
            let seeds = &self.t.delaunay.tiles;
            let ids: Vec<_> = seeds
                .iter()
                .enumerate()
                .filter(|(_, s)| s.row == row.row)
                .map(|(i, _)| i)
                .collect();
            let lengths: Vec<_> = ids
                .iter()
                .enumerate()
                .map(|(k, &i)| norm(sub(self.points[i], self.points[ids[(k + 1) % ids.len()]])))
                .collect();
            let spacing = lengths
                .iter()
                .map(|v| (v - lengths[0]).abs())
                .fold(0., f64::max);
            let size = ids
                .iter()
                .map(|&i| (squares[i] - squares[self.plane_reps[i]]).abs())
                .fold(0., f64::max);
            let mut rotation = 0.0_f64;
            for &(rr, k, reference, angle) in &self.copies {
                if rr == row.row {
                    rotation = rotation.max(norm(sub(
                        corners[k],
                        self.s.rotate_point(corners[reference], angle)?,
                    )));
                }
            }
            row.rotation_error = Some(rotation);
            row.spacing_error = Some(spacing);
            row.size_error = Some(size);
            row.passed = Some(rotation <= 1e-6 && spacing <= 1e-6 && size <= 1e-6);
        }
        Ok(rows)
    }
    fn sparsity(&self) -> Vec<Vec<usize>> {
        let n = self.t.polygons.len();
        let mut polygons = self.t.polygons.clone();
        polygons.extend(self.t.triangles.iter().map(|v| v.to_vec()));
        let corner_deps: Vec<BTreeSet<usize>> = self
            .t
            .corners
            .iter()
            .enumerate()
            .map(|(k, e)| {
                let [i, j] = e.seeds;
                [
                    self.map[2 * i],
                    self.map[2 * i + 1],
                    self.map[2 * j],
                    self.map[2 * j + 1],
                    self.map[2 * n + k],
                ]
                .into_iter()
                .collect()
            })
            .collect();
        let mut normal_deps: Vec<BTreeSet<usize>> = (0..n)
            .map(|i| [self.map[2 * i], self.map[2 * i + 1]].into_iter().collect())
            .collect();
        for ids in &self.t.triangles {
            normal_deps.push(
                ids.iter()
                    .flat_map(|&k| corner_deps[k].iter().copied())
                    .collect(),
            );
        }
        let mut owners = BTreeMap::<[usize; 2], Vec<usize>>::new();
        for (i, ids) in polygons.iter().enumerate() {
            for k in 0..ids.len() {
                let a = ids[k];
                let b = ids[(k + 1) % ids.len()];
                owners.entry([a.min(b), a.max(b)]).or_default().push(i);
            }
        }
        let piece_deps: Vec<BTreeSet<usize>> = polygons
            .iter()
            .enumerate()
            .map(|(i, ids)| {
                let mut deps = normal_deps[i].clone();
                for &k in ids {
                    deps.extend(&corner_deps[k]);
                }
                for k in 0..ids.len() {
                    let a = ids[k];
                    let b = ids[(k + 1) % ids.len()];
                    for &j in &owners[&[a.min(b), a.max(b)]] {
                        deps.extend(&normal_deps[j]);
                    }
                }
                deps
            })
            .collect();
        let mut support = Vec::new();
        for deps in &corner_deps {
            for _ in 0..3 {
                support.push(deps.iter().copied().collect());
            }
        }
        for k in 0..self.nv {
            support.push(vec![k]);
        }
        for (i, ids) in polygons.iter().enumerate() {
            let m = ids.len();
            let pairs = m * m.saturating_sub(3) / 2;
            let rows = m + m * (if i < n { 8 } else { 7 }) + 2 * pairs + 2;
            for _ in 0..rows {
                support.push(piece_deps[i].iter().copied().collect());
            }
        }
        // Back gaps depend on BOTH complete inner fans.
        for (edge, owners) in &owners {
            if owners.len() == 2 {
                let deps: BTreeSet<_> = piece_deps[owners[0]]
                    .union(&piece_deps[owners[1]])
                    .copied()
                    .collect();
                for _ in edge {
                    support.push(deps.iter().copied().collect());
                }
            }
        }
        for _ in 0..2 * n {
            support.push(Vec::new());
        }
        if self.p.randomness == 0. {
            for deps in &piece_deps {
                support.push(deps.iter().copied().collect());
            }
        }
        for row in &self.rows {
            if row.required {
                let deps: BTreeSet<usize> = self
                    .t
                    .delaunay
                    .tiles
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.row == row.row)
                    .flat_map(|(i, _)| piece_deps[i].iter().copied())
                    .collect();
                for _ in 0..3 {
                    support.push(deps.iter().copied().collect());
                }
            }
        }
        for &(_, k, r, _) in &self.copies {
            let deps: BTreeSet<usize> = corner_deps[k].union(&corner_deps[r]).copied().collect();
            for _ in 0..3 {
                support.push(deps.iter().copied().collect());
            }
        }
        support
    }
    fn evaluate(&self, x: &[f64]) -> Result<Evaluation, String> {
        let (planes, corners) = self.geometry(x)?;
        let n = planes.len();
        // A corner belongs to four pieces; query the generic surface only once
        // per corner, retaining the exact distance used by all four barriers
        // and the diagnostic summary.
        let corner_distances: Vec<_> = corners.iter().map(|&v| self.s.distance(v)).collect();
        let mut polygons = self.t.polygons.clone();
        polygons.extend(self.t.triangles.iter().map(|v| v.to_vec()));
        let mut normals: Vec<_> = planes.iter().map(|p| p.n).collect();
        for ids in &self.t.triangles {
            let mut normal = unit(cross(
                sub(corners[ids[1]], corners[ids[0]]),
                sub(corners[ids[2]], corners[ids[0]]),
            ))?;
            let seed = self.t.corners[ids[0]].seeds[0];
            if dot(normal, self.normals[seed]) < 0. {
                normal = mul(normal, -1.);
            }
            normals.push(normal);
        }
        for (i, ids) in polygons.iter_mut().enumerate() {
            let area = (0..ids.len())
                .map(|k| {
                    dot(
                        cross(corners[ids[k]], corners[ids[(k + 1) % ids.len()]]),
                        normals[i],
                    )
                })
                .sum::<f64>();
            if area < 0. {
                ids.reverse();
            }
        }
        let whole: Vec<_> = polygons
            .iter()
            .enumerate()
            .map(|(i, ids)| {
                ids.len() >= 3
                    && (i >= n || !self.t.delaunay.tiles[i].auxiliary)
                    && ids.iter().all(|&k| self.s.inside_rims(corners[k]))
            })
            .collect();
        let mut edge_owners = BTreeMap::<[usize; 2], Vec<usize>>::new();
        for (i, ids) in polygons.iter().enumerate() {
            for k in 0..ids.len() {
                let a = ids[k];
                let b = ids[(k + 1) % ids.len()];
                edge_owners.entry([a.min(b), a.max(b)]).or_default().push(i);
            }
        }
        let mut residual = Vec::new();
        let mut violations = Vec::new();
        let mut limit_deficits = BTreeMap::<String, [f64; 2]>::new();
        for (a, b) in corners.iter().zip(&self.targets) {
            residual.extend(sub(*a, *b).map(|v| 0.05 * v));
        }
        // Prototype uses .05 tilt and .02 line-slide regularization.
        let tilt_end = self.map[2 * n - 1] + 1;
        residual.extend(x.iter().enumerate().map(
            |(k, v)| {
                if k < tilt_end {
                    0.05 * v
                } else {
                    0.02 * v
                }
            },
        ));
        let mut barrier = |i: usize, name: &str, v: f64, enabled: bool| {
            if enabled {
                if !v.is_finite() {
                    violations.push((i, format!("nonfinite_{name}")));
                }
                let bounds = limit_deficits.entry(name.into()).or_insert([v, v]);
                bounds[0] = bounds[0].min(v);
                bounds[1] = bounds[1].max(v);
            }
            let physical = if enabled { v.max(0.) } else { 0. };
            let margin = match name {
                "min_edge" | "min_triangle_edge" => 0.04,
                "max_polygon_edge" => 0.04,
                "corner_band_low" | "corner_band_high" => 0.04,
                "blank_usable" => 0.05,
                "min_corner_angle_deg" | "max_corner_angle_deg" => 0.05,
                "bevel_cap_deg" => 1.,
                "min_inner_edge" => 1.,
                "flatness" => 0.0000001,
                _ => 0.,
            };
            let v = if enabled { (v + margin).max(0.) } else { 0. };
            residual.push(10. * v);
            if physical > 1e-7 {
                violations.push((i, name.into()));
            }
        };
        let mut inner = Vec::new();
        let mut squares = Vec::new();
        for (i, ids) in polygons.iter().enumerate() {
            let normal = normals[i];
            let (a, b) = basis(normal);
            let origin = corners[ids[0]];
            let project = |p: V| {
                let d = sub(p, origin);
                [dot(d, a), dot(d, b)]
            };
            let q: Vec<_> = ids.iter().map(|&k| project(corners[k])).collect();
            let mut joints = Vec::new();
            for k in 0..ids.len() {
                let c = ids[k];
                let d = ids[(k + 1) % ids.len()];
                let owners = &edge_owners[&[c.min(d), c.max(d)]];
                let neighbor = owners.iter().copied().find(|&j| j != i);
                let bend =
                    neighbor.map_or(0., |j| dot(normal, normals[j]).clamp(-1., 1.).acos() / 2.);
                barrier(
                    i,
                    "bevel_cap_deg",
                    bend.to_degrees() - self.p.bevel_cap_deg,
                    whole[i],
                );
                let m = if bend.to_degrees() < 1. {
                    unit(cross(sub(corners[d], corners[c]), normal))?
                } else {
                    unit(sub(normal, normals[neighbor.unwrap()]))?
                };
                joints.push(m);
            }
            let mut back = Vec::new();
            for k in 0..ids.len() {
                let u = joints[(k + ids.len() - 1) % ids.len()];
                let v = joints[k];
                let matrix = Matrix3::from_row_slice(&[
                    normal[0], normal[1], normal[2], u[0], u[1], u[2], v[0], v[1], v[2],
                ]);
                let disp = matrix
                    .lu()
                    .solve(&Vector3::new(-self.p.thickness, 0., 0.))
                    .ok_or("singular inner joint intersection")?;
                back.push(add(corners[ids[k]], [disp[0], disp[1], disp[2]]));
            }
            let iq: Vec<_> = back.iter().map(|&v| project(v)).collect();
            let guard = whole[i]
                || (i < n && !self.t.delaunay.tiles[i].auxiliary)
                || (i >= n && ids.iter().any(|&k| self.s.inside_rims(corners[k])));
            for k in 0..ids.len() {
                let prev = corners[ids[(k + ids.len() - 1) % ids.len()]];
                let cur = corners[ids[k]];
                let next = corners[ids[(k + 1) % ids.len()]];
                let u = sub(cur, prev);
                let v = sub(next, cur);
                let l = norm(v);
                barrier(
                    i,
                    if i < n {
                        "min_edge"
                    } else {
                        "min_triangle_edge"
                    },
                    if i < n {
                        self.p.min_edge - l
                    } else {
                        self.p.min_triangle_edge - l
                    },
                    whole[i],
                );
                barrier(i, "max_polygon_edge", l - self.p.max_polygon_edge, whole[i]);
                // A06 has no minimum whole-piece square limit.
                let angle = 180. - dot(cross(u, v), normal).atan2(dot(u, v)).to_degrees();
                barrier(
                    i,
                    "min_corner_angle_deg",
                    (self.p.min_corner_angle_deg - angle) / 30.,
                    whole[i],
                );
                barrier(
                    i,
                    "max_corner_angle_deg",
                    (angle - self.p.max_corner_angle_deg) / 30.,
                    whole[i],
                );
                let signed = dot(sub(back[(k + 1) % ids.len()], back[k]), unit(v)?);
                barrier(
                    i,
                    "min_inner_edge",
                    (crate::tol::MIN_INNER_EDGE - signed) / crate::tol::MIN_INNER_EDGE,
                    guard,
                );
                let distance = corner_distances[ids[k]];
                let e = &self.t.corners[ids[k]];
                let base = (self.t.relief_bases[e.seeds[0]] + self.t.relief_bases[e.seeds[1]]) / 2.;
                barrier(
                    i,
                    "corner_band_low",
                    -(base + self.p.depth + 0.4) - distance,
                    self.s.inside_rims(cur),
                );
                barrier(
                    i,
                    "corner_band_high",
                    distance - 0.75,
                    self.s.inside_rims(cur),
                );
                if i < n {
                    barrier(
                        i,
                        "flatness",
                        (dot(planes[i].n, cur) - planes[i].d).abs() - 1e-6,
                        whole[i],
                    );
                }
            }
            let area = |v: &Vec<[f64; 2]>| {
                (0..v.len())
                    .map(|k| v[k][0] * v[(k + 1) % v.len()][1] - v[k][1] * v[(k + 1) % v.len()][0])
                    .sum::<f64>()
            };
            barrier(
                i,
                "inner_orientation",
                if area(&iq) <= 0. {
                    1. + area(&iq).abs()
                } else {
                    0.
                },
                guard,
            );
            for k in 0..ids.len() {
                for j in k + 2..ids.len() {
                    if k == 0 && j + 1 == ids.len() {
                        continue;
                    }
                    barrier(
                        i,
                        "simple_polygon",
                        intersection_barrier(
                            q[k],
                            q[(k + 1) % ids.len()],
                            q[j],
                            q[(j + 1) % ids.len()],
                        ),
                        whole[i],
                    );
                    barrier(
                        i,
                        "simple_inner_polygon",
                        intersection_barrier(
                            iq[k],
                            iq[(k + 1) % ids.len()],
                            iq[j],
                            iq[(j + 1) % ids.len()],
                        ),
                        guard,
                    );
                }
            }
            let mut footprint = q;
            footprint.extend(iq);
            let square = footprint_square(&footprint);
            barrier(i, "blank_usable", square - self.p.blank_usable, whole[i]);
            squares.push(square);
            inner.push(back);
        }
        for (edge, owners) in &edge_owners {
            if owners.len() != 2 {
                continue;
            }
            let [i, j] = [owners[0], owners[1]];
            for &k in edge {
                let a = polygons[i].iter().position(|&v| v == k).unwrap();
                let b = polygons[j].iter().position(|&v| v == k).unwrap();
                barrier(
                    i,
                    "max_back_gap",
                    norm(sub(inner[i][a], inner[j][b])) - self.p.max_back_gap,
                    self.s.inside_rims(corners[k]),
                );
            }
        }
        for (i, seed) in self.t.delaunay.tiles.iter().enumerate() {
            let base = self.t.relief_bases[i];
            barrier(i, "seed_depth_low", base - seed.depth - 1e-9, true);
            barrier(
                i,
                "seed_depth_high",
                seed.depth - base - self.p.depth - 1e-9,
                true,
            );
        }
        if self.p.randomness == 0. {
            for (i, ids) in polygons.iter().enumerate() {
                let inner_ids: Vec<_> = (0..inner[i].len()).collect();
                let error = self
                    .mirror_error(i, ids, &corners)?
                    .max(self.mirror_error(i, &inner_ids, &inner[i])?);
                barrier(i, "stone_mirror", error / 1e-6 - 1., whole[i]);
            }
        }
        let symmetry_rows = self.symmetry_rows(&corners, &squares)?;
        for row in &symmetry_rows {
            if let Some(representative) = row.representative {
                // Deficits are normalized, so the existing report tolerance does
                // not turn the 1e-6 inch gate into a manufacturing tolerance.
                for (name, error) in [
                    ("row_rotation", row.rotation_error.unwrap()),
                    ("row_spacing", row.spacing_error.unwrap()),
                    ("row_size", row.size_error.unwrap()),
                ] {
                    barrier(representative, name, error / 1e-6 - 1., true);
                }
            }
        }
        if self.p.randomness == 0. {
            for (i, ids) in polygons.iter().enumerate() {
                let inner_ids: Vec<_> = (0..inner[i].len()).collect();
                let error = self
                    .mirror_error(i, ids, &corners)?
                    .max(self.mirror_error(i, &inner_ids, &inner[i])?);
                if whole[i] && error > 1e-6 {
                    violations.push((i, "stone_mirror_gate".into()));
                }
            }
        }
        // The row gate is exact at its stated tolerance, independently of the
        // normalized manufacturing residual/report epsilon used above.
        for row in &symmetry_rows {
            if row.required && row.passed != Some(true) {
                violations.push((row.representative.unwrap(), "row_symmetry_gate".into()));
            }
        }
        // Signed smooth equality rows aid convergence even inside the gate;
        // they contain no independent gated-row corner unknowns.
        for &(_, k, reference, angle) in &self.copies {
            let expected = self.s.rotate_point(corners[reference], angle)?;
            residual.extend(sub(corners[k], expected).map(|v| v * 10.));
        }
        let mut distances: Vec<_> = corners
            .iter()
            .zip(&corner_distances)
            .filter(|(v, _)| self.s.inside_rims(**v))
            .map(|(_, &d)| d)
            .collect();
        distances.sort_by(f64::total_cmp);
        let stats = if distances.is_empty() {
            [0.; 3]
        } else {
            [
                distances[0],
                distances[distances.len() / 2],
                *distances.last().unwrap(),
            ]
        };
        if residual.iter().any(|v| !v.is_finite()) {
            return Err("nonfinite limit residual".into());
        }
        Ok((
            residual,
            Solution {
                planes,
                symmetry_rows,
                corners,
                polygons,
                inner,
                whole,
                attempts: Vec::new(),
                best_attempt: 0,
                corner_distance: stats,
                relief_base: self.t.relief_base_stats,
                seed_spacing: self.t.seed_spacing,
                bounding_squares: squares,
                c5_pending: true,
                unknowns: self.nv,
                jacobian_colors: 0,
                limit_deficits,
            },
            violations,
        ))
    }
}
// Convex A06 face half-space clipping for placement residuals only. Native
// C5 remains the authority for provenance, solids, and physical-band closure.
#[cfg(test)]
fn clip_rim(points: &[V], height: f64, top: bool) -> Vec<V> {
    let mut out = Vec::new();
    for k in 0..points.len() {
        let a = points[k];
        let b = points[(k + 1) % points.len()];
        let da = if top { height - a[1] } else { a[1] - height };
        let db = if top { height - b[1] } else { b[1] - height };
        if da >= 0. {
            out.push(a);
        }
        if (da < 0. && db > 0.) || (da > 0. && db < 0.) {
            let mut q = add(a, mul(sub(b, a), da / (da - db)));
            q[1] = height;
            out.push(q);
        }
    }
    out
}
fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let orient = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let on = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        orient(p, q, r).abs() < 1e-12
            && (0..2).all(|i| r[i] >= p[i].min(q[i]) - 1e-12 && r[i] <= p[i].max(q[i]) + 1e-12)
    };
    orient(a, b, c) * orient(a, b, d) < 0. && orient(c, d, a) * orient(c, d, b) < 0.
        || on(a, b, c)
        || on(a, b, d)
        || on(c, d, a)
        || on(c, d, b)
}
// Retain the exact crossing/touching predicate and its unit failure floor,
// but expose penetration to finite differences. A binary 0/1 residual has
// zero derivative throughout a crossing and gives GN no unfolding direction.
fn intersection_barrier(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> f64 {
    if !intersects(a, b, c, d) {
        return 0.;
    }
    let distance = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        let u = [q[0] - p[0], q[1] - p[1]];
        ((u[0] * (r[1] - p[1]) - u[1] * (r[0] - p[0])).abs()) / u[0].hypot(u[1]).max(1e-12)
    };
    1. + distance(a, b, c)
        .min(distance(a, b, d))
        .min(distance(c, d, a))
        .min(distance(c, d, b))
}
type Color = (Vec<usize>, Vec<(usize, usize)>);
fn color_columns(n: usize, support: &[Vec<usize>]) -> Vec<Color> {
    let mut conflicts = vec![BTreeSet::new(); n];
    for row in support {
        for &a in row {
            for &b in row {
                if a != b {
                    conflicts[a].insert(b);
                }
            }
        }
    }
    let mut assignment = vec![usize::MAX; n];
    let mut colors: Vec<Color> = Vec::new();
    for k in 0..n {
        let used: BTreeSet<_> = conflicts[k].iter().map(|&j| assignment[j]).collect();
        let color = (0..).find(|v| !used.contains(v)).unwrap();
        assignment[k] = color;
        if colors.len() <= color {
            colors.push((Vec::new(), Vec::new()));
        }
        colors[color].0.push(k);
    }
    for (row, deps) in support.iter().enumerate() {
        for &k in deps {
            colors[assignment[k]].1.push((row, k));
        }
    }
    colors
}
// Interior footprint points cannot be square supports. The exact square routine
// enumerates pairwise support changes, so removing them first avoids quartic
// work for the combined outer/inner footprint without approximating the gate.
fn footprint_square(points: &[[f64; 2]]) -> f64 {
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    sorted.dedup();
    if sorted.len() <= 3 {
        return bounding_square(&sorted);
    }
    let turn = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    };
    let mut hull = Vec::new();
    for &p in &sorted {
        while hull.len() >= 2 && turn(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0. {
            hull.pop();
        }
        hull.push(p);
    }
    let lower = hull.len();
    for &p in sorted.iter().rev().skip(1) {
        while hull.len() > lower && turn(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0. {
            hull.pop();
        }
        hull.push(p);
    }
    hull.pop();
    bounding_square(&hull)
}

fn objective(r: &[f64]) -> f64 {
    r.iter().map(|v| v * v).sum()
}
/// Fixed-cap deterministic five-start sparse finite-difference damped GN.
/// All attempts retain their own best report. Failure retains best geometry;
/// success is C4-only: C5 clipping, gaps and closure remain explicitly pending.
pub fn solve<S: PeriodicSymmetry>(s: &S, t: &Rectified, p: &Params) -> Result<Solution, Failure> {
    solve_budget(s, t, p, 5, 150)
}
/// Same fixed deterministic solver with a reporting-only completed-attempt hook.
/// Callback timing or output must not modify geometry, limits or solver caps.
pub fn solve_reporting<S: PeriodicSymmetry>(
    s: &S,
    t: &Rectified,
    p: &Params,
    report: impl FnMut(&Attempt),
) -> Result<Solution, Failure> {
    solve_budget_reporting(s, t, p, 5, 150, None, report)
}
fn solve_budget<S: PeriodicSymmetry>(
    s: &S,
    t: &Rectified,
    p: &Params,
    starts: usize,
    steps: usize,
) -> Result<Solution, Failure> {
    solve_budget_reporting(s, t, p, starts, steps, None, |_| {})
}
pub(crate) fn solve_budget_reporting<S: PeriodicSymmetry>(
    s: &S,
    t: &Rectified,
    p: &Params,
    starts: usize,
    steps: usize,
    pilot_start: Option<usize>,
    mut report: impl FnMut(&Attempt),
) -> Result<Solution, Failure> {
    let problem = Problem::new(s, t, p).map_err(|message| Failure {
        message,
        best: None,
        attempts: vec![],
    })?;
    let support = problem.sparsity();
    let colors = color_columns(problem.nv, &support);
    let mut attempts = Vec::new();
    let mut best: Option<(usize, f64, Solution)> = None;
    // Symmetric relief's zero-tilt start can sit in the wrong basin (cone
    // reference parity diagnostics). Try the same existing second perturbed
    // start first; preserve all five starts, caps, objectives and stable IDs.
    let mut order: Vec<_> = (0..starts).collect();
    if p.randomness == 0. && p.depth > 0. && starts > 2 {
        order.remove(2);
        order.insert(0, 2);
    }
    if let Some(index) = pilot_start {
        order = vec![index];
    }
    for attempt in order {
        let mut x: Vec<_> = (0..problem.nv)
            .map(|k| {
                if attempt == 0 {
                    0.
                } else {
                    0.08 * ((attempt * problem.nv + k + 1 + (p.seed % 104729) as usize) as f64)
                        .sin()
                }
            })
            .collect();
        let mut local: Option<LocalBest> = None;
        let mut damping = 0.01;
        // Rejected LM trials change damping, not x or its Jacobian (as in
        // prototype least_squares). Reuse the exact finite differences.
        let mut cached_columns = None;
        let mut error = None;
        for iteration in 0..=steps {
            let (r, geometry, violated) = match problem.evaluate(&x) {
                Ok(v) => v,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };
            let cost = objective(&r);
            if local.as_ref().is_none_or(|v| {
                violated.len() < v.2.len() || (violated.len() == v.2.len() && cost < v.0)
            }) {
                local = Some((cost, geometry, violated.clone(), iteration));
            }
            if iteration == steps || violated.is_empty() {
                break;
            }
            // Structural graph coloring: one geometry evaluation per COLOR,
            // never one full evaluation per variable. A residual row contains at
            // most one perturbed member of a color, so derivatives separate exactly.
            if r.len() != support.len() {
                error = Some(format!(
                    "sparse residual layout mismatch {} != {}",
                    r.len(),
                    support.len()
                ));
                break;
            }
            let (columns, scales) = if let Some(cached) = cached_columns.take() {
                cached
            } else {
                let mut columns = vec![Vec::new(); problem.nv];
                for (variables, owners) in &colors {
                    let mut trial = x.clone();
                    let mut h = vec![0.; x.len()];
                    for &k in variables {
                        h[k] = 1e-6 * (1. + x[k].abs());
                        trial[k] += h[k];
                    }
                    if let Ok((rr, _, _)) = problem.evaluate(&trial) {
                        for &(row, k) in owners {
                            let d = (rr[row] - r[row]) / h[k];
                            if d.abs() > 1e-12 {
                                columns[k].push((row, d));
                            }
                        }
                    }
                }
                // Prototype x_scale="jac": solve in column-normalized variables.
                // Limits and physical evaluation remain unchanged.
                let scales: Vec<_> = columns
                    .iter()
                    .map(|c| 1. / c.iter().map(|&(_, v)| v * v).sum::<f64>().sqrt().max(1e-12))
                    .collect();
                for (column, scale) in columns.iter_mut().zip(&scales) {
                    for (_, value) in column {
                        *value *= scale;
                    }
                }
                (columns, scales)
            };
            let gradient: Vec<_> = columns
                .iter()
                .map(|c| -c.iter().map(|&(j, v)| v * r[j]).sum::<f64>())
                .collect();
            let apply = |v: &[f64]| {
                let mut jv = vec![0.; r.len()];
                for (c, &a) in columns.iter().zip(v) {
                    for &(j, b) in c {
                        jv[j] += a * b;
                    }
                }
                columns
                    .iter()
                    .enumerate()
                    .map(|(k, c)| c.iter().map(|&(j, a)| a * jv[j]).sum::<f64>() + damping * v[k])
                    .collect::<Vec<_>>()
            };
            let mut step = vec![0.; x.len()];
            let mut rr = gradient.clone();
            let mut direction = rr.clone();
            let mut rr2 = objective(&rr);
            for _ in 0..80 {
                if rr2 < 1e-16 {
                    break;
                }
                let ad = apply(&direction);
                let denom = direction.iter().zip(&ad).map(|(a, b)| a * b).sum::<f64>();
                if denom <= 0. || !denom.is_finite() {
                    break;
                }
                let alpha = rr2 / denom;
                for k in 0..x.len() {
                    step[k] += alpha * direction[k];
                    rr[k] -= alpha * ad[k];
                }
                let next = objective(&rr);
                let beta = next / rr2;
                for k in 0..x.len() {
                    direction[k] = rr[k] + beta * direction[k];
                }
                rr2 = next;
            }
            for (value, scale) in step.iter_mut().zip(&scales) {
                *value *= scale;
            }
            let max = step.iter().map(|v| v.abs()).fold(0., f64::max);
            if max > 0.25 {
                for v in &mut step {
                    *v *= 0.25 / max;
                }
            }
            let trial: Vec<_> = x.iter().zip(&step).map(|(a, b)| a + b).collect();
            if problem
                .evaluate(&trial)
                .is_ok_and(|(r, _, _)| objective(&r) < cost)
            {
                x = trial;
                damping = (damping * 0.5).max(1e-8);
            } else {
                damping = (damping * 4.).min(1e12);
                cached_columns = Some((columns, scales));
            }
        }
        if let Some((cost, geometry, violated, iterations)) = local {
            let count = violated.len();
            attempts.push(Attempt {
                index: attempt,
                corner_distance: Some(geometry.corner_distance),
                symmetry_rows: geometry.symmetry_rows.clone(),
                iterations,
                objective: cost,
                violations: violated,
                limit_deficits: geometry.limit_deficits.clone(),
                error,
            });
            if best
                .as_ref()
                .is_none_or(|(v, c, _)| count < *v || (count == *v && cost < *c))
            {
                let mut geometry = geometry;
                geometry.best_attempt = attempt;
                best = Some((count, cost, geometry));
            }
        } else {
            attempts.push(Attempt {
                index: attempt,
                corner_distance: None,
                symmetry_rows: problem.rows.clone(),
                iterations: 0,
                objective: f64::MAX,
                violations: vec![],
                limit_deficits: BTreeMap::new(),
                error,
            });
        }
        report(attempts.last().expect("completed attempt recorded"));
        if best.as_ref().is_some_and(|(count, _, g)| {
            *count == 0
                && g.symmetry_rows
                    .iter()
                    .all(|r| !r.required || r.passed == Some(true))
        }) {
            break;
        }
    }
    if let Some((count, _, mut geometry)) = best {
        geometry.attempts = attempts.clone();
        geometry.jacobian_colors = colors.len();
        if count == 0
            && geometry
                .symmetry_rows
                .iter()
                .all(|r| !r.required || r.passed == Some(true))
        {
            Ok(geometry)
        } else {
            Err(Failure {
                message:
                    "A06 limits remain violated after deterministic multi-start; no limits relaxed"
                        .into(),
                best: Some(Box::new(geometry)),
                attempts,
            })
        }
    } else {
        Err(Failure {
            message: "all two-plane starts invalid".into(),
            best: None,
            attempts,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hull_square_matches_full_footprints() {
        for count in 3..=20 {
            for phase in 0..20 {
                let points: Vec<_> = (0..count)
                    .map(|k| {
                        let angle = k as f64 * 2.399963229728653 + phase as f64 * 0.17;
                        let radius = 1. + (k % 4) as f64 * 0.2;
                        [radius * angle.cos(), radius * angle.sin()]
                    })
                    .collect();
                assert!((footprint_square(&points) - bounding_square(&points)).abs() < 1e-12);
            }
        }
        for points in [
            vec![[0., 0.]; 12],
            vec![[0., 0.], [1., 1.], [2., 2.], [1., 1.]],
        ] {
            assert!((footprint_square(&points) - bounding_square(&points)).abs() < 1e-12);
        }
    }
    #[test]
    fn exact_line_and_parallel_rejection() {
        let a = Plane {
            n: [1., 0., 0.],
            d: 2.,
        };
        let b = Plane {
            n: [0., 1., 0.],
            d: 3.,
        };
        let v = corner(&a, &b, [8., 9., 4.], 0.7).unwrap();
        assert!((dot(a.n, v) - a.d).abs() < 1e-12);
        assert!((dot(b.n, v) - b.d).abs() < 1e-12);
        assert!((v[2] - 4.7).abs() < 1e-12);
        assert!(corner(&a, &a, [0.; 3], 0.).is_err());
    }
    #[test]
    fn crossing_barrier_keeps_gate_and_supplies_unfolding_gradient() {
        let a = [-2., 0.];
        let b = [2., 0.];
        let c = [0., -0.5];
        let d = [0., 1.];
        let crossed = intersection_barrier(a, b, c, d);
        let toward_boundary = intersection_barrier(a, b, [0., -0.5 + 1e-6], d);
        assert!(crossed > toward_boundary && toward_boundary >= 1.);
        assert_eq!(intersection_barrier(a, b, [0., 0.], d), 1.);
        assert_eq!(intersection_barrier(a, b, [0., 0.1], d), 0.);
    }
    #[test]
    fn touching_and_crossing_rejected() {
        assert!(intersects([0., 0.], [2., 0.], [1., 0.], [3., 0.]));
        assert!(!intersects([0., 0.], [1., 0.], [2., 0.], [3., 0.]));
    }
}
#[cfg(test)]
mod geometry_tests {
    use super::*;
    use crate::{
        model::{CoreInput, Pattern},
        rectified,
        surface::Surface,
    };
    fn setup() -> (Surface, Params, Rectified) {
        let mut i: CoreInput = serde_json::from_str(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        i.params.pattern = Pattern::VoronoiTriangles;
        i.params.randomness = 0.;
        // A06 §9 uses small stones; 4 > the approved 3-inch maximum
        // makes topology infeasible before either geometry assertion runs.
        i.params.min_triangle_edge = 1.5;
        i.params.max_polygon_edge = 3.;
        i.params.min_edge = 1.;
        i.params.depth = 1.;
        i.params.min_corner_angle_deg = 30.;
        let s = Surface::from_profile(&i.surface.profile).unwrap();
        let t = rectified::build(&s, &i.params).unwrap();
        (s, i.params, t)
    }
    #[test]
    fn exact_flatness_four_valence_and_row_rotation() {
        let (s, p, t) = setup();
        let problem = Problem::new(&s, &t, &p).unwrap();
        let x: Vec<_> = (0..problem.nv)
            .map(|k| 0.01 * ((k % 5) as f64 - 2.))
            .collect();
        let (planes, corners) = problem.geometry(&x).unwrap();
        for (k, e) in t.corners.iter().enumerate() {
            if !e.open {
                assert_eq!(e.pieces.len(), 4);
            }
            for &i in &e.seeds {
                assert!(
                    (dot(planes[i].n, corners[k]) - planes[i].d).abs() < 1e-9,
                    "corner {k} plane {i}"
                );
            }
        }
        for (row_index, row) in t.rows.iter().enumerate() {
            if !problem.rows[row_index].required {
                continue;
            }
            let ids: Vec<_> = t
                .delaunay
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, v)| v.s == row.s)
                .map(|(i, _)| i)
                .collect();
            for &i in &ids {
                let r = ids[0];
                let expected = s
                    .rotate_vector(planes[r].n, t.delaunay.tiles[i].u - t.delaunay.tiles[r].u)
                    .unwrap();
                assert!(norm(sub(expected, planes[i].n)) < 1e-12);
            }
        }
        assert_eq!(problem.geometry(&x).unwrap(), problem.geometry(&x).unwrap());
    }
    #[test]
    fn limits_do_not_silently_pass_and_attempts_are_deterministic() {
        let (s, mut p, t) = setup();
        p.min_triangle_edge = 100.;
        p.blank_usable = 0.1;
        let a = solve_budget(&s, &t, &p, 2, 0).unwrap_err();
        let b = solve_budget(&s, &t, &p, 2, 0).unwrap_err();
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert_eq!(a.attempts.len(), 2);
        assert!(a.attempts.iter().any(|v| v
            .violations
            .iter()
            .any(|(_, s)| s == "blank_usable" || s == "min_triangle_edge")));
        assert!(a.best.unwrap().c5_pending);
    }
}

#[cfg(test)]
mod section9_tests {
    use super::*;
    use crate::{
        model::{CoreInput, Pattern},
        rectified,
        surface::Surface,
    };
    fn setup(profile: &str) -> (Surface, Params, Rectified) {
        let mut i: CoreInput = serde_json::from_str(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        i.surface.profile = serde_json::from_str(
            &std::fs::read_to_string(format!(
                "{}/../tests/fixtures/profiles/{profile}.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
        )
        .unwrap();
        i.params.pattern = Pattern::VoronoiTriangles;
        i.params.randomness = 0.;
        i.params.depth = 0.25;
        i.params.min_triangle_edge = 1.5;
        i.params.max_polygon_edge = 3.;
        i.params.min_corner_angle_deg = 30.;
        let s = Surface::from_profile(&i.surface.profile).unwrap();
        let t = rectified::build(&s, &i.params).unwrap();
        (s, i.params, t)
    }
    #[test]
    fn placement_clip_preserves_physical_plane_and_tangent_incidence() {
        let triangle = [[0., -1., 0.], [2., 1., 0.], [-2., 1., 0.]];
        let q = clip_rim(&triangle, 0., false);
        assert_eq!(q.len(), 4);
        assert!(q.iter().all(|v| v[1] >= 0.));
        assert_eq!(q.iter().filter(|v| v[1] == 0.).count(), 2);
        assert_eq!(clip_rim(&triangle, -1., false), triangle);
        assert_eq!(clip_rim(&triangle, 1., true), triangle);
    }
    #[test]
    fn constant_rows_reduce_exactly_and_sparse_colors_do_not_collide() {
        for profile in ["cylinder", "cone_9_to_6", "vase_bulge"] {
            let (s, p, t) = setup(profile);
            let problem = Problem::new(&s, &t, &p).unwrap();
            assert!(problem
                .rows
                .iter()
                .all(|r| r.required && r.repeats == r.count));
            let x: Vec<_> = (0..problem.nv).map(|k| 0.005 * (k as f64).sin()).collect();
            let (planes, corners) = problem.geometry(&x).unwrap();
            for (k, e) in t.corners.iter().enumerate() {
                for &i in &e.seeds {
                    assert!((dot(planes[i].n, corners[k]) - planes[i].d).abs() < 1e-9);
                }
            }
            let gates = problem
                .symmetry_rows(&corners, &vec![0.; t.polygons.len()])
                .unwrap();
            assert!(gates.iter().all(|r| r.passed == Some(true)));
            let support = problem.sparsity();
            let colors = color_columns(problem.nv, &support);
            for (vars, _) in &colors {
                let vars: BTreeSet<_> = vars.iter().copied().collect();
                for row in &support {
                    assert!(row.iter().filter(|v| vars.contains(v)).count() <= 1);
                }
            }
            let evaluated = problem.evaluate(&x).unwrap();
            assert_eq!(support.len(), evaluated.0.len());
            // Verify support pruning on real projected outer+inner footprints,
            // not only synthetic point clouds (including all padding pieces).
            let g = &evaluated.1;
            assert!(g.symmetry_rows.iter().all(|r| {
                !r.required
                    || (r.rotation_error.is_some()
                        && r.spacing_error.is_some()
                        && r.size_error.is_some()
                        && r.passed.is_some())
            }));
            let assembly = crate::rectified_assembly::assemble(&t, g, &s, &p).unwrap();
            assert_eq!(assembly.symmetry_rows, g.symmetry_rows);
            for (i, ids) in g.polygons.iter().enumerate() {
                let normal = if i < g.planes.len() {
                    g.planes[i].n
                } else {
                    unit(cross(
                        sub(g.corners[ids[1]], g.corners[ids[0]]),
                        sub(g.corners[ids[2]], g.corners[ids[0]]),
                    ))
                    .unwrap()
                };
                let (a, b) = basis(normal);
                let origin = g.corners[ids[0]];
                let footprint: Vec<_> = ids
                    .iter()
                    .map(|&k| g.corners[k])
                    .chain(g.inner[i].iter().copied())
                    .map(|v| {
                        let d = sub(v, origin);
                        [dot(d, a), dot(d, b)]
                    })
                    .collect();
                assert!((bounding_square(&footprint) - g.bounding_squares[i]).abs() < 1e-10);
            }
            assert!(colors.len() < problem.nv);
            assert_eq!(problem.geometry(&x).unwrap(), problem.geometry(&x).unwrap());
        }
    }
    #[test]
    fn dihedral_unknowns_preserve_every_stone_and_detect_mirror_damage() {
        for profile in ["cylinder", "cone_9_to_6", "vase_bulge", "flare"] {
            for depth in [0., 0.25] {
                let (mut s, mut p, _) = setup(if profile == "flare" {
                    "cylinder"
                } else {
                    profile
                });
                if profile == "flare" {
                    s = Surface::from_profile(&[[6., 0.], [9., 60.]]).unwrap();
                }
                p.depth = depth;
                let t = rectified::build(&s, &p).unwrap();
                let problem = Problem::new(&s, &t, &p).unwrap();
                let x: Vec<_> = (0..problem.nv).map(|k| 0.015 * (k as f64).cos()).collect();
                let (planes, mut corners) = problem.geometry(&x).unwrap();
                for (i, seed) in t.delaunay.tiles.iter().enumerate() {
                    let tangent = s.rotate_vector([0., 0., 1.], seed.u).unwrap();
                    assert!(dot(planes[i].n, tangent).abs() < 1e-12);
                }
                let mut polygons = t.polygons.clone();
                polygons.extend(t.triangles.iter().map(|v| v.to_vec()));
                for (i, ids) in polygons.iter().enumerate() {
                    let error = problem.mirror_error(i, ids, &corners).unwrap();
                    assert!(
                        error <= 1e-6,
                        "{profile} depth={depth} piece={i} mirror={error}"
                    );
                }
                let (_, measured, violations) = problem.evaluate(&x).unwrap();
                assert!(
                    measured.limit_deficits["stone_mirror"][1] <= 0.,
                    "{profile} depth={depth}: inner or outer mirror gate failed"
                );
                assert!(!violations
                    .iter()
                    .any(|(_, name)| name == "stone_mirror_gate"));
                let k = polygons[0][0];
                corners[k][1] += 0.01;
                assert!(problem.mirror_error(0, &polygons[0], &corners).unwrap() > 1e-6);
            }
        }
    }
    #[test]
    fn gates_detect_geometric_corruption_instead_of_averaging() {
        let (s, p, t) = setup("cylinder");
        let problem = Problem::new(&s, &t, &p).unwrap();
        let (_, mut corners) = problem.geometry(&vec![0.; problem.nv]).unwrap();
        let row = problem.copies.iter().find(|(_, k, r, _)| k != r).unwrap().0;
        let corner = problem
            .copies
            .iter()
            .find(|(rr, k, r, _)| *rr == row && k != r)
            .unwrap()
            .1;
        corners[corner][1] += 0.001;
        let gates = problem
            .symmetry_rows(&corners, &vec![0.; t.polygons.len()])
            .unwrap();
        assert_eq!(gates[row].passed, Some(false));
    }
    #[test]
    fn native_solver_reports_are_byte_deterministic() {
        let (s, mut p, _) = setup("cylinder");
        let short = Surface::from_profile(&[[9., 0.], [9., 8.]]).unwrap();
        p.depth = 0.;
        let t = rectified::build(&short, &p).unwrap();
        let a = solve_budget(&short, &t, &p, 1, 1);
        let b = solve_budget(&short, &t, &p, 1, 1);
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        drop(s);
    }
}
