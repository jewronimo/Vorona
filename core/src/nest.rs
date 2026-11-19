//! Deterministic conservative footprint nesting, in inches.
use crate::{
    flat,
    model::{Blank, Piece},
    tol::GEOM_EPS,
};
type Rect = [f64; 4];
fn bounds(points: &[[f64; 3]], angle: f64) -> Rect {
    let (s, c) = angle.sin_cos();
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for p in points {
        let x = c * p[0] - s * p[1];
        let y = s * p[0] + c * p[1];
        b[0] = b[0].min(x);
        b[1] = b[1].min(y);
        b[2] = b[2].max(x);
        b[3] = b[3].max(y);
    }
    b
}
fn separated(a: Rect, b: Rect, gap: f64) -> bool {
    a[2] + gap <= b[0] || b[2] + gap <= a[0] || a[3] + gap <= b[1] || b[3] + gap <= a[1]
}
/// Pack the projected outer/inner union conservatively using bounding rectangles.
/// Tries bottom-left contact positions, then 15-degree rotations, first blank first.
/// Returns an error for invalid blank settings or pieces that cannot fit. Transactional:
/// on failure no piece placements are changed. Blank origins follow INTERFACE.md.
pub fn pack(pieces: &mut [Piece], size: f64, usable: f64, gap: f64) -> Result<Vec<Blank>, String> {
    if !size.is_finite()
        || !usable.is_finite()
        || !gap.is_finite()
        || usable <= 0.0
        || size < usable
        || gap < 0.0
    {
        return Err("invalid blank_size, blank_usable or nest_gap".into());
    }
    let points: Vec<Vec<_>> = pieces
        .iter()
        .map(|p| {
            p.outer
                .iter()
                .chain(&p.inner)
                .map(|v| flat::apply(&p.flat_xform, *v))
                .collect()
        })
        .collect();
    if points
        .iter()
        .any(|p| p.len() < 6 || p.iter().flatten().any(|v| !v.is_finite()))
    {
        return Err("invalid nesting footprint".into());
    }
    let mut order: Vec<_> = (0..pieces.len()).collect();
    // Conservative footprint area gives a stable large-first order.
    order.sort_by(|&a, &b| {
        let area = |i: usize| {
            let r = bounds(&points[i], 0.0);
            (r[2] - r[0]) * (r[3] - r[1])
        };
        area(b)
            .total_cmp(&area(a))
            .then_with(|| pieces[a].id.cmp(&pieces[b].id))
            .then(a.cmp(&b))
    });
    let mut occupied: Vec<Vec<Rect>> = Vec::new();
    let mut blanks: Vec<Blank> = Vec::new();
    let mut placements = Vec::new();
    for i in order {
        let rotations: Vec<_> = (0..24)
            .map(|rotation| {
                let angle = (rotation as f64 * 15.0).to_radians();
                (angle, bounds(&points[i], angle))
            })
            .collect();
        let mut found = None;
        for b in 0..=occupied.len() {
            let empty = Vec::new();
            let rects = occupied.get(b).unwrap_or(&empty);
            let mut xs = vec![0.0];
            let mut ys = vec![0.0];
            for r in rects {
                xs.push(r[2] + gap);
                ys.push(r[3] + gap);
            }
            xs.sort_by(f64::total_cmp);
            xs.dedup();
            ys.sort_by(f64::total_cmp);
            ys.dedup();
            'positions: for &y in &ys {
                for &x in &xs {
                    for &(angle, r) in &rotations {
                        let placed = [x, y, x + r[2] - r[0], y + r[3] - r[1]];
                        if placed[2] > usable + GEOM_EPS
                            || placed[3] > usable + GEOM_EPS
                            || rects.iter().any(|&other| !separated(placed, other, gap))
                        {
                            continue;
                        }
                        found = Some((b, angle, r, placed));
                        break 'positions;
                    }
                }
            }
            if found.is_some() {
                break;
            }
        }
        let Some((b, angle, r, placed)) = found else {
            return Err(format!("{}: footprint cannot fit blank", pieces[i].id));
        };
        if b == occupied.len() {
            occupied.push(Vec::new());
            blanks.push(Blank {
                index: b as u32 + 1,
                pieces: Vec::new(),
            });
        }
        occupied[b].push(placed);
        blanks[b].pieces.push(pieces[i].id.clone());
        let (s, c) = angle.sin_cos();
        let margin = (size - usable) / 2.0;
        placements.push((
            i,
            b as u32 + 1,
            [
                [
                    c,
                    -s,
                    0.0,
                    placed[0] - r[0] + margin + b as f64 * (size + 2.0),
                ],
                [s, c, 0.0, placed[1] - r[1] + margin],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        ));
    }
    for (i, b, m) in placements {
        pieces[i].blank = b;
        pieces[i].blank_xform = m;
    }
    for blank in &mut blanks {
        blank.pieces.sort();
    }
    Ok(blanks)
}
