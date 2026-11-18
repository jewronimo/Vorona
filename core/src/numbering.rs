//! Deterministic assembly numbering and outside-view unwrapped maps.
use crate::{model::Piece, surface::Surface};
use std::{collections::BTreeMap, f64::consts::TAU};
/// Number inch-valued pieces bottom-to-top, then counter-clockwise seen from +Y, preserving
/// adjacency. Caller supplies the derived target pitch in inches.
pub fn number(pieces: &mut [Piece], surface: &Surface, target: f64) -> BTreeMap<String, String> {
    let centroid = |p: &Piece| {
        let mut c = [0.; 3];
        for v in &p.outer {
            for k in 0..3 {
                c[k] += v[k] / p.outer.len() as f64;
            }
        }
        c
    };
    // Surface theta increases from +X toward +Z, clockwise when seen from +Y.
    // Negate Z for the owner-required counter-clockwise numbering convention.
    pieces.sort_by(|a, b| {
        let x = centroid(a);
        let y = centroid(b);
        (x[1] / (0.5 * target))
            .floor()
            .total_cmp(&(y[1] / (0.5 * target)).floor())
            .then(
                (-x[2])
                    .atan2(x[0])
                    .rem_euclid(TAU)
                    .total_cmp(&(-y[2]).atan2(y[0]).rem_euclid(TAU)),
            )
            .then(a.id.cmp(&b.id))
    });
    let width = 3.max(pieces.len().to_string().len());
    let ids: BTreeMap<_, _> = pieces
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.clone(), format!("P{:0width$}", i + 1)))
        .collect();
    let radius = surface.r(surface.length() / 2.);
    for p in pieces {
        p.id = ids[&p.id].clone();
        for e in &mut p.edges {
            if let Some(n) = &mut e.neighbor {
                if let Some(new) = ids.get(n) {
                    *n = new.clone();
                }
            }
        }
        let (theta, _) = surface.closest_point(centroid(p));
        let center = (TAU - theta).rem_euclid(TAU);
        p.map_uv = p
            .outer
            .iter()
            .map(|&v| {
                let (t, s) = surface.closest_point(v);
                let phi = (TAU - t).rem_euclid(TAU);
                [
                    radius * (center + (phi - center + TAU / 2.).rem_euclid(TAU) - TAU / 2.),
                    s,
                ]
            })
            .collect();
        let area: f64 = (0..p.map_uv.len())
            .map(|i| {
                let a = p.map_uv[i];
                let b = p.map_uv[(i + 1) % p.map_uv.len()];
                a[0] * b[1] - b[0] * a[1]
            })
            .sum();
        if area < 0. {
            p.map_uv.reverse();
        }
    }
    ids
}
