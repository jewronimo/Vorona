//! Deterministic staggered seeds. u and v are dimensionless; s is in inches.
use crate::{feasibility, model::Params, surface::Surface};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::f64::consts::TAU;

/// One tile site; negative/past-end s denotes an auxiliary level-rim row.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Seed {
    pub u: f64,
    pub v: f64,
    pub s: f64,
    pub row: usize,
    pub auxiliary: bool,
    /// Inward-normal offset in inches; finalized after side-count repair.
    pub depth: f64,
}

/// Continue the conformal chart tangentially beyond each open end (inches to v).
pub fn extended_v(surface: &Surface, s: f64) -> f64 {
    if s < 0.0 {
        s / surface.r(0.0)
    } else if s > surface.length() {
        surface.v_of_s(surface.length()) + (s - surface.length()) / surface.r(surface.length())
    } else {
        surface.v_of_s(s)
    }
}

/// Construct seeded rows using C2's minimum-change even column schedule.
/// Rejects invalid parameters, infeasibility, unsupported rims, or excessive site counts.
/// The caller owns the sole ChaCha8 stream for both seeding and subsequent repair.
pub fn place(surface: &Surface, p: &Params, rng: &mut ChaCha8Rng) -> Result<Vec<Seed>, String> {
    if !p.depth.is_finite()
        || p.depth <= 0.0
        || !p.randomness.is_finite()
        || !(0.0..=1.0).contains(&p.randomness)
        || !p.target_size.is_finite()
        || p.target_size <= 0.0
        || !p.size_variation.is_finite()
        || !(0.05..=0.50).contains(&p.size_variation)
        || ![&p.rims.bottom, &p.rims.top]
            .iter()
            .all(|r| matches!(r.as_str(), "whole" | "level"))
    {
        return Err("invalid seed parameters or rim style".into());
    }
    if p.allowed_sides.is_empty() || !feasibility::check(surface, p).feasible {
        return Err("cannot seed an infeasible profile".into());
    }
    let steps = (surface.length() / 0.05).ceil() as usize;
    let intervals: Vec<_> = (0..=steps)
        .map(|i| {
            let r = surface.r(surface.length() * i as f64 / steps as f64);
            let lo = (TAU * r / (p.target_size * (1.0 + p.size_variation)) / 2.0 - 1e-12)
                .ceil()
                .max(1.0) as usize;
            let hi = (TAU * r / (p.target_size * (1.0 - p.size_variation)) / 2.0 + 1e-12).floor()
                as usize;
            (lo, hi.max(lo))
        })
        .collect();
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
    let mut columns = Vec::new();
    for &(lo, hi) in intervals.iter().rev() {
        n = n.clamp(lo, hi);
        columns.push(2 * n);
    }
    columns.reverse();
    // C2 fixes the compressed column sequence, but its earliest-switch tie is
    // unsuitable for constructing defects: it can put one member on the rim.
    // Switch in the middle of each overlap of the two admissible intervals.
    let sequence = feasibility::check(surface, p).columns;
    let mut cursor = 0;
    for (i, value) in columns.iter_mut().enumerate() {
        let r = surface.r(surface.length() * i as f64 / steps as f64);
        if cursor + 1 < sequence.len() {
            let a = sequence[cursor] as f64;
            let b = sequence[cursor + 1] as f64;
            let low = a.max(b) * p.target_size * (1.0 - p.size_variation) / TAU;
            let high = a.min(b) * p.target_size * (1.0 + p.size_variation) / TAU;
            let switch = (low + high) * 0.5;
            if (b < a && r <= switch) || (b > a && r >= switch) {
                cursor += 1;
            }
        }
        *value = sequence[cursor] as usize;
    }
    // Row heights track the local lattice scale; no time-dependent stopping.
    let mut arcs = vec![0.0];
    loop {
        let s = *arcs.last().unwrap();
        let index = ((s / surface.length() * steps as f64).round() as usize).min(steps);
        let h = 3.0_f64.sqrt() / 2.0 * TAU * surface.r(s) / columns[index] as f64;
        if s + h >= surface.length() {
            break;
        }
        if arcs.len() >= 10_000 {
            return Err("seed row cap exceeded".into());
        }
        arcs.push(s + h);
    }
    if arcs.len() < 2 {
        return Err("profile too short for two lattice rows".into());
    }
    // Spread the residual uniformly so the top rim does not leave a tiny final row.
    let last = *arcs.last().unwrap();
    for s in &mut arcs {
        *s *= surface.length() / last;
    }
    if p.rims.bottom == "level" {
        arcs.insert(0, -(arcs[1] - arcs[0]));
    }
    if p.rims.top == "level" {
        let len = arcs.len();
        arcs.push(arcs[len - 1] + arcs[len - 1] - arcs[len - 2]);
    }
    let mut seeds = Vec::new();
    for (row, &s) in arcs.iter().enumerate() {
        let index = ((s.clamp(0.0, surface.length()) / surface.length() * steps as f64).round()
            as usize)
            .min(steps);
        let count = columns[index];
        if count < 4 || seeds.len() + count > 100_000 {
            return Err(
                "periodic topology needs 4..100000 sites per row and at most 100000 total".into(),
            );
        }
        for col in 0..count {
            let pitch = TAU / count as f64;
            // The requested band bounds organic displacement; randomness never
            // buys permission to violate spacing. Rim angular perturbations
            // have more freedom (rim edges are exempt), but are still capped.
            let rim = s <= 0.0 || s >= surface.length();
            let nominal_ratio = pitch * surface.r(s) / p.target_size;
            let available = (p.size_variation - (nominal_ratio - 1.0).abs()).max(0.0);
            let amplitude = p.randomness
                * if rim {
                    p.size_variation * 1.5
                } else {
                    available * 0.9
                };
            let du = rng.gen_range(-1.0..1.0) * pitch * amplitude;
            let ds = rng.gen_range(-1.0..1.0) * p.target_size * amplitude;
            let auxiliary = s < 0.0 || s > surface.length();
            let moved = if auxiliary || row == 0 || row + 1 == arcs.len() {
                s
            } else {
                s + ds
            };
            seeds.push(Seed {
                u: ((col as f64 + 0.5 * (row % 2) as f64) * pitch + du).rem_euclid(TAU),
                v: extended_v(surface, moved),
                s: moved,
                row,
                auxiliary,
                depth: symmetric_depth(row, p.depth),
            });
        }
    }
    // Independent jitter can remain inside one Delaunay chamber. Coherent
    // row rotations choose chambers too, without enlarging that jitter. A
    // near-pitch rotation leaves a small metric shear but moves sites through
    // actual flips; site indices/column identities are never permuted.
    if p.randomness > 0.5 && sequence.len() == 1 && arcs.len() > 3 {
        let pitch = TAU / sequence[0] as f64;
        for row in 0..arcs.len() {
            let phase = if rng.gen_bool(0.5) {
                (0.85 + rng.gen_range(0.0..0.1)) * (2.0 * p.randomness - 1.0)
            } else {
                0.0
            };
            for site in seeds.iter_mut().filter(|x| x.row == row) {
                site.u = (site.u + phase * pitch).rem_euclid(TAU);
            }
        }
    }
    // Construct each count-change dislocation before generic relaxation. A
    // relative phase between entire blocks changes only this interface. Search
    // a fixed angular grid for the required adjacent 5/7 dipoles; this is not
    // an insertion/deletion repair that can rewrite C2's row schedule.
    construct_dislocations(surface, p, &mut seeds)?;
    Ok(seeds)
}

fn construct_dislocations(
    surface: &Surface,
    p: &Params,
    sites: &mut Vec<Seed>,
) -> Result<(), String> {
    let f = feasibility::check(surface, p);
    if f.column_changes == 0 {
        return Ok(());
    }
    let mut rows = std::collections::BTreeMap::<usize, usize>::new();
    for site in sites.iter().filter(|x| !x.auxiliary) {
        *rows.entry(site.row).or_default() += 1;
    }
    let rows: Vec<_> = rows.into_iter().collect();
    for pair in rows.windows(2).filter(|w| w[0].1 != w[1].1) {
        let row = pair[1].0;
        let required = pair[0].1.abs_diff(pair[1].1);
        let base = sites.clone();
        let mut best: Option<(usize, f64, Vec<Seed>)> = None;
        for phase in 0..64 {
            let offset = TAU / pair[1].1 as f64 * phase as f64 / 64.0;
            let mut candidate = base.clone();
            for site in candidate.iter_mut().filter(|x| x.row >= row) {
                site.u = (site.u + offset).rem_euclid(TAU);
            }
            let Ok(t) = crate::topology::Topology::build(candidate.clone()) else {
                continue;
            };
            let local: Vec<_> = (0..t.tiles.len())
                .filter(|&i| {
                    !t.tiles[i].auxiliary
                        && t.tiles[i].s > 1e-9
                        && t.tiles[i].s < surface.length() - 1e-9
                        && t.tiles[i].row.abs_diff(row) <= 1
                })
                .collect();
            let count = |k| local.iter().filter(|&&i| t.degree(i) == k).count();
            let unpaired = local
                .iter()
                .filter(|&&i| {
                    matches!(t.degree(i), 5 | 7)
                        && !t.neighbors[i].iter().any(|&j| {
                            t.tiles[j].row.abs_diff(row) <= 1 && t.degree(j) == 12 - t.degree(i)
                        })
                })
                .count();
            let invalid = local
                .iter()
                .filter(|&&i| !matches!(t.degree(i), 5..=7))
                .count();
            // Cone transitions require complementary dipoles. A curved vase
            // instead has a GLOBAL positive-curvature quota; forcing the cone
            // quota independently at each interface overproduces pentagons.
            let penalty = if f.pentagons_needed > 0 {
                let pentagons = (0..t.tiles.len())
                    .filter(|&i| crate::sidecount::interior(&t, i, surface) && t.degree(i) == 5)
                    .count();
                let invalid_global = (0..t.tiles.len())
                    .filter(|&i| {
                        crate::sidecount::interior(&t, i, surface)
                            && !p.allowed_sides.contains(&(t.degree(i) as u32))
                    })
                    .count();
                pentagons
                    .abs_diff(f.pentagons_needed as usize)
                    .saturating_sub(1)
                    + invalid_global
            } else {
                count(5).abs_diff(required) + count(7).abs_diff(required) + unpaired + invalid
            };
            let residual = crate::sidecount::spacing_residual(surface, p, &t).0;
            if best
                .as_ref()
                .is_none_or(|(a, b, _)| penalty < *a || (penalty == *a && residual < *b))
            {
                best = Some((penalty, residual, candidate));
            }
        }
        let (penalty, _, constructed) =
            best.ok_or_else(|| format!("no valid periodic construction at row {row}"))?;
        if feasibility::check(surface, p).pentagons_needed == 0
            && p.randomness == 0.0
            && penalty != 0
        {
            return Err(format!("cannot construct {required} adjacent 5/7 pairs at row {row}: residual defect count {penalty}"));
        }
        *sites = constructed;
    }
    Ok(())
}

/// Symmetric inward offset in inches for the repeating three-row relief.
/// `depth` must be positive and finite, as checked by placement.
pub fn symmetric_depth(row: usize, depth: f64) -> f64 {
    (row % 3) as f64 * 0.5 * depth
}

/// Finalize inward offsets in inches after ALL positional repair draws.
/// Uses the caller's sole seeded stream in stable tile order. Placement carries
/// symmetric provisional depths so triangulation and repair never consume relief
/// draws or change their pre-A05 random-stream results.
pub(crate) fn assign_depths(sites: &mut [Seed], p: &Params, rng: &mut ChaCha8Rng) {
    for site in sites {
        let symmetric = symmetric_depth(site.row, p.depth);
        site.depth = if p.randomness == 0.0 {
            symmetric
        } else {
            let uniform = rng.gen_range(0.0..=1.0) * p.depth;
            ((1.0 - p.randomness) * symmetric + p.randomness * uniform).clamp(0.0, p.depth)
        };
    }
}

/// Start the sole deterministic random stream from the numeric contract seed.
pub fn rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// Surface-metric length of the straight conformal-chart path, in inches.
/// Since dv = ds/r, the integral of r(v) dv is exactly the arc difference.
/// Auxiliary rows retain the same endpoint-cylinder extension as extended_v.
pub fn spacing(surface: &Surface, a: &Seed, b: &Seed) -> f64 {
    let du = (b.u - a.u + TAU / 2.0).rem_euclid(TAU) - TAU / 2.0;
    let dv = b.v - a.v;
    let mean_radius = if dv.abs() > 1e-12 {
        (b.s - a.s) / dv
    } else {
        surface.r((a.s + b.s) * 0.5)
    };
    du.hypot(dv) * mean_radius
}

#[cfg(test)]
mod depth_tests {
    use super::*;

    #[test]
    fn a05_blend_uses_exact_uniform_draws_after_the_symmetric_pattern() {
        let input: crate::model::CoreInput = serde_json::from_str(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        for randomness in [0.25, 0.5, 1.0] {
            let mut p = input.params.clone();
            p.randomness = randomness;
            p.depth = 2.0;
            let mut sites: Vec<_> = (0..12)
                .map(|row| Seed {
                    u: 0.0,
                    v: 0.0,
                    s: 0.0,
                    row,
                    auxiliary: false,
                    depth: 0.0,
                })
                .collect();
            let mut actual = rng(42);
            let mut expected = actual.clone();
            assign_depths(&mut sites, &p, &mut actual);
            for site in sites {
                let uniform = expected.gen_range(0.0..=1.0) * p.depth;
                assert_eq!(
                    site.depth,
                    (1.0 - randomness) * symmetric_depth(site.row, p.depth) + randomness * uniform
                );
            }
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>());
        }
    }
}
