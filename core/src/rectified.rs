//! A06 C3 only: surface seeds and combinatorial rectification, not fitted geometry.
use crate::{
    model::{Params, Pattern, ReliefBase},
    seeds::{extended_v, Seed},
    surface::Surface,
    topology::Topology,
};
#[path = "row_layout.rs"]
pub mod row_layout;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::{collections::BTreeMap, f64::consts::TAU};

/// Row measurements in inches; width is the predicted circumference/count.
/// Triangle diagnostics are nominal isosceles forward-strip measurements, using
/// midpoint radius for proportional pitch; not final fitted piece angles.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Row {
    pub s: f64,
    pub count: usize,
    pub predicted_width: f64,
    pub exceeds_blank: bool,
    pub height: f64,
    pub triangle_base: f64,
    pub triangle_leg: f64,
    pub triangle_apex_angle_deg: f64,
    pub triangle_base_angle_deg: f64,
}
/// One rectified corner per original edge. Piece IDs: polygons first, then triangles.
/// Only outermost padding edges may have three rather than four incident pieces.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Corner {
    pub seeds: [usize; 2],
    pub pieces: Vec<usize>,
    pub open: bool,
}
/// Cyclic corner IDs for each polygon / triangle, plus C3 diagnostic measurements.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Rectified {
    pub delaunay: Topology,
    pub seed_spacing: f64,
    pub rows: Vec<Row>,
    pub relief_bases: Vec<f64>,
    /// Resolved base min/median/max, inches.
    pub relief_base_stats: [f64; 3],
    pub polygons: Vec<Vec<usize>>,
    pub triangles: Vec<[usize; 3]>,
    pub corners: Vec<Corner>,
}
fn delta(a: f64, b: f64) -> f64 {
    (a - b + TAU / 2.0).rem_euclid(TAU) - TAU / 2.0
}

/// A06 seeds and topology in inches / dimensionless conformal coordinates.
/// Rejects invalid A06 settings, resource caps and nonmanifold/open interior edges.
/// Does not consult legacy allowed sides, target size, variation or C2 defect repair.
pub fn build(surface: &Surface, p: &Params) -> Result<Rectified, String> {
    build_with_phase(surface, p, 0.)
}

/// Build A06 with a deterministic fractional row phase in [0,1).
/// Surface queries, depths and topology are reconstructed, never translated off-surface.
pub fn build_with_phase(surface: &Surface, p: &Params, phase: f64) -> Result<Rectified, String> {
    let layout = row_layout::choose_with_phase(surface, p, phase)?;
    let spacing = layout[layout.len() / 2].width;
    if p.pattern != Pattern::VoronoiTriangles
        || !spacing.is_finite()
        || spacing <= 0.0
        || !p.depth.is_finite()
        || p.depth < 0.0
        || !p.randomness.is_finite()
        || !(0.0..=1.0).contains(&p.randomness)
        || !p.blank_usable.is_finite()
        || p.blank_usable <= 0.0
        || ![&p.rims.bottom, &p.rims.top]
            .iter()
            .all(|r| matches!(r.as_str(), "whole" | "level"))
    {
        return Err("invalid A06 seed parameters or rim style".into());
    }
    if let ReliefBase::Inches(b) = p.relief_base {
        if !b.is_finite() || b < 0.0 {
            return Err("invalid relief_base".into());
        }
    }
    let mut rng = ChaCha8Rng::seed_from_u64(p.seed);
    let mut tiles = Vec::new();
    let mut rows = Vec::new();
    for (row, planned) in layout.iter().enumerate() {
        let s = planned.s;
        let count = planned.count;
        let pitch = planned.height;
        if count > 100_000 || tiles.len() + count > 100_000 {
            return Err("A06 seed site cap exceeded".into());
        }
        let width = TAU * surface.r(s) / count as f64;
        // Triangle diagnostics describe the forward strip, not the lower row.
        // A proportional strip spans changing radii; sample its midpoint.
        let strip_width = if p.row_height == crate::model::RowHeight::Proportional {
            TAU * surface.r(s + pitch / 2.) / count as f64
        } else {
            width
        };
        rows.push(Row {
            s,
            count,
            predicted_width: width,
            exceeds_blank: width > p.blank_usable,
            height: pitch,
            triangle_apex_angle_deg: (2. * (strip_width / (2. * pitch)).atan()).to_degrees(),
            triangle_base_angle_deg: 90. - (strip_width / (2. * pitch)).atan().to_degrees(),
            triangle_base: strip_width / 2.0,
            triangle_leg: 0.5 * (strip_width.powi(2) / 4.0 + pitch.powi(2)).sqrt(),
        });
        for col in 0..count {
            let angular = TAU / count as f64;
            let (du, ds) = if p.randomness > 0.0 {
                (
                    rng.gen_range(-1.0..1.0) * angular * 0.1 * p.randomness,
                    rng.gen_range(-1.0..1.0) * pitch * 0.1 * p.randomness,
                )
            } else {
                (0.0, 0.0)
            };
            // Row-level support convention matches legacy C3: padding and
            // exact rim rows stay level; ordinary jitter cannot cross a rim.
            let auxiliary = s < 0.0 || s > surface.length();
            let moved = if auxiliary || s <= 0.0 || s >= surface.length() {
                s
            } else {
                (s + ds).clamp(1e-8, surface.length() - 1e-8)
            };
            let u = ((col as f64 + 0.5 * (row % 2) as f64) * angular + du).rem_euclid(TAU);
            tiles.push(Seed {
                u: if u == TAU { 0.0 } else { u },
                v: extended_v(surface, moved),
                s: moved,
                row,
                auxiliary,
                depth: 0.0,
            });
        }
    }
    let mut bases = Vec::new();
    for t in &mut tiles {
        let base = match p.relief_base {
            ReliefBase::Inches(b) => b,
            ReliefBase::Auto(_) => {
                let k = surface.principal_curvatures(t.s);
                (TAU * surface.r(t.s) / rows[t.row].count as f64).powi(2) / 16.0
                    * k[0].max(k[1]).max(0.0)
            }
        };
        let relief = if p.randomness == 0.0 {
            p.depth * (t.row % 3) as f64 / 2.0
        } else if p.depth == 0.0 {
            0.0
        } else {
            rng.gen_range(0.0..=p.depth)
        };
        t.depth = base + relief;
        if !t.depth.is_finite() {
            return Err("nonfinite A06 relief depth".into());
        }
        bases.push(base);
    }
    // The owner's global staggered grid has adjacent-row connectivity even
    // where conformal compression would flip to three-row triangles. Build
    // its combinatorics on the regular grid; retain the physical chart sites.
    let d = if p.randomness == 0. {
        let mut chart = tiles.clone();
        for tile in &mut chart {
            tile.v = tile.row as f64 * 3_f64.sqrt() / 2. * TAU / rows[tile.row].count as f64;
        }
        let mut topology = Topology::build_a06(chart)?;
        topology.tiles = tiles;
        topology
    } else {
        Topology::build_a06(tiles)?
    };
    rectify(d, spacing, rows, bases)
}

/// Rectify a validated periodic Delaunay topology; all corner IDs are stable
/// sorted-edge indices. Rejects missing/mismatched bases or invalid incidence.
/// No surface geometry or three-plane intersections are constructed here.
pub fn rectify(
    d: Topology,
    spacing: f64,
    rows: Vec<Row>,
    bases: Vec<f64>,
) -> Result<Rectified, String> {
    let n = d.tiles.len();
    if bases.len() != n || bases.is_empty() || bases.iter().any(|b| !b.is_finite() || *b < 0.0) {
        return Err("invalid rectified relief bases".into());
    }
    let mut edges = BTreeMap::<[usize; 2], Vec<usize>>::new();
    for (ti, &[a, b, c]) in d.corners.iter().enumerate() {
        for e in [[a, b], [a, c], [b, c]] {
            edges.entry(e).or_default().push(n + ti);
        }
    }
    let mut corners = Vec::new();
    let mut ids = BTreeMap::new();
    let mut polygons = vec![Vec::new(); n];
    let first = d
        .tiles
        .iter()
        .map(|t| t.row)
        .min()
        .ok_or("empty topology")?;
    let last = d.tiles.iter().map(|t| t.row).max().unwrap();
    for (e, ts) in edges {
        if ts.is_empty()
            || ts.len() > 2
            || (ts.len() == 1
                && (d.tiles[e[0]].row != d.tiles[e[1]].row
                    || ![first, last].contains(&d.tiles[e[0]].row)))
        {
            return Err(format!("open or nonmanifold rectified interior edge {e:?}"));
        }
        let id = corners.len();
        ids.insert(e, id);
        for &i in &e {
            polygons[i].push(id);
        }
        let open = ts.len() == 1;
        let mut pieces = e.to_vec();
        pieces.extend(ts);
        corners.push(Corner {
            seeds: e,
            pieces,
            open,
        });
    }
    for (i, poly) in polygons.iter_mut().enumerate() {
        let angle = |id: usize| {
            let e = corners[id].seeds;
            let j = if e[0] == i { e[1] } else { e[0] };
            (d.tiles[j].v - d.tiles[i].v).atan2(delta(d.tiles[j].u, d.tiles[i].u))
        };
        poly.sort_by(|&a, &b| angle(a).total_cmp(&angle(b)).then(a.cmp(&b)));
    }
    let triangles = d
        .corners
        .iter()
        .map(|&[a, b, c]| {
            let mut b = b;
            let mut c = c;
            if delta(d.tiles[b].u, d.tiles[a].u) * (d.tiles[c].v - d.tiles[a].v)
                - (d.tiles[b].v - d.tiles[a].v) * delta(d.tiles[c].u, d.tiles[a].u)
                < 0.0
            {
                std::mem::swap(&mut b, &mut c);
            }
            let edge = |x: usize, y: usize| ids[&[x.min(y), x.max(y)]];
            [edge(a, b), edge(b, c), edge(c, a)]
        })
        .collect();
    let mut sorted = bases.clone();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    let median = if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    };
    Ok(Rectified {
        delaunay: d,
        seed_spacing: spacing,
        rows,
        relief_bases: bases,
        relief_base_stats: [sorted[0], median, *sorted.last().unwrap()],
        polygons,
        triangles,
        corners,
    })
}

impl Rectified {
    /// Incident seed IDs for a polygon or triangle, in stable index order.
    pub fn piece_seeds(&self, piece: usize) -> Vec<usize> {
        if piece < self.polygons.len() {
            return vec![piece];
        }
        self.triangles[piece - self.polygons.len()]
            .iter()
            .flat_map(|&k| self.corners[k].seeds)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    /// Resolved inward base in inches for a polygon or triangle. Triangle base
    /// is the arithmetic mean of its three incident polygon seed bases.
    pub fn piece_relief_base(&self, piece: usize) -> f64 {
        let seeds = self.piece_seeds(piece);
        seeds.iter().map(|&i| self.relief_bases[i]).sum::<f64>() / seeds.len() as f64
    }
}

/// Owner Task A relief-only twin margins in inches: inward, outward.
/// Inputs are finite [minimum, maximum] whole-corner ranges for one incident
/// row at the same chart/phase; depth is nonnegative inches. Negative fails.
pub fn smooth_twin_margins(relief: [f64; 2], smooth: [f64; 2], depth: f64) -> [f64; 2] {
    [
        relief[0] - smooth[0] + depth + 0.20,
        smooth[1] + 0.10 - relief[1],
    ]
}
