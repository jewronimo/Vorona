//! C5 horizontal saw cut, preserving outgoing side-plane provenance.
use crate::{
    model::Plane,
    pieces::{Point, Side},
    tol::GEOM_EPS,
};
/// Result of one level cut; rejected slivers are explicit, never silently omitted.
#[derive(Clone, Debug)]
pub enum LevelCut {
    Outside,
    Sliver,
    Kept {
        outer: Vec<Point>,
        sides: Vec<Side>,
        partial: bool,
    },
}
/// Clip against y <= height (top) or y >= height (bottom), in inches.
/// Disconnected reflex intersections are explicitly rejected before constructing a solid.
pub fn level(
    outer: &[Point],
    sides: &[Side],
    height: f64,
    top: bool,
    minimum: f64,
) -> Result<LevelCut, &'static str> {
    if outer.len() < 3
        || sides.len() != outer.len()
        || !height.is_finite()
        || !minimum.is_finite()
        || minimum <= 0.
        || outer.iter().flatten().any(|x| !x.is_finite())
    {
        return Err("invalid level rim input");
    }
    let sign = if top { 1. } else { -1. };
    let distance = |p: Point| sign * (p[1] - height);
    let partial = outer.iter().any(|p| distance(*p) > GEOM_EPS);
    let rim = Side {
        plane: Plane {
            n: [0., sign, 0.],
            d: sign * height,
        },
        kind: "rim_level",
        neighbor: None,
    };
    // More than one retained boundary chain means a disconnected intersection.
    // A line coincident with a vertex is treated conservatively, never bridged.
    let entries = (0..outer.len())
        .filter(|&i| {
            distance(outer[i]) > GEOM_EPS && distance(outer[(i + 1) % outer.len()]) <= GEOM_EPS
        })
        .count();
    if entries > 1 {
        return Err("disconnected reflex level clip");
    }
    // Store incoming side labels while clipping, then rotate to outgoing labels.
    let mut points = Vec::new();
    let mut incoming = Vec::new();
    for i in 0..outer.len() {
        let a = outer[i];
        let b = outer[(i + 1) % outer.len()];
        let da = distance(a);
        let db = distance(b);
        let ia = da <= 0.;
        let ib = db <= 0.;
        if ia != ib {
            let t = da / (da - db);
            let mut x = std::array::from_fn(|k| a[k] + t * (b[k] - a[k]));
            x[1] = height;
            points.push(x);
            incoming.push(if ia { sides[i].clone() } else { rim.clone() });
        }
        if ib {
            points.push(b);
            incoming.push(sides[i].clone());
        }
    }
    // Remove duplicate consecutive vertices introduced by corners exactly on the cut.
    let mut k = 0;
    while points.len() > 1 && k < points.len() {
        let j = (k + 1) % points.len();
        if (0..3)
            .map(|d| (points[k][d] - points[j][d]).powi(2))
            .sum::<f64>()
            <= GEOM_EPS * GEOM_EPS
        {
            points.remove(k);
            incoming.remove(k);
        } else {
            k += 1;
        }
    }
    if points.len() < 3 {
        return Ok(LevelCut::Outside);
    }
    if partial && enclosing_square(&points) + GEOM_EPS < minimum {
        return Ok(LevelCut::Sliver);
    }
    incoming.rotate_left(1);
    Ok(LevelCut::Kept {
        outer: points,
        sides: incoming,
        partial,
    })
}

/// Actual minimum enclosing square side in the polygon's plane, in inches.
/// Returns zero for degenerate input; supports arbitrary simple reflex polygons.
pub fn enclosing_square(points: &[Point]) -> f64 {
    use nalgebra::Vector3;
    if points.len() < 3 {
        return 0.;
    }
    let origin = Vector3::from(points[0]);
    let mut normal = Vector3::zeros();
    for i in 1..points.len() - 1 {
        normal +=
            (Vector3::from(points[i]) - origin).cross(&(Vector3::from(points[i + 1]) - origin));
    }
    if normal.norm() < GEOM_EPS {
        return 0.;
    }
    let n = normal.normalize();
    let axis = if n[0].abs() < 0.9 {
        Vector3::x()
    } else {
        Vector3::y()
    };
    let u = n.cross(&axis).normalize();
    let w = n.cross(&u);
    let q: Vec<_> = points
        .iter()
        .map(|p| {
            let d = Vector3::from(*p) - origin;
            [d.dot(&u), d.dot(&w)]
        })
        .collect();
    crate::planes::bounding_square(&q)
}

/// Deterministically choose a cut height BEFORE clipping; prefer the requested height.
/// Search inward over one configured tile pitch; minimize every rejection, then displacement.
/// Returns chosen height and rejected candidate indices; never hides a failed placement.
pub fn place(
    polygons: &[(Vec<Point>, Vec<Side>)],
    requested: f64,
    top: bool,
    pitch: f64,
    minimum: f64,
) -> (f64, Vec<usize>) {
    let mut best = (requested, Vec::new());
    let mut count = usize::MAX;
    for step in 0..=128 {
        let height = requested + if top { -1. } else { 1. } * pitch * step as f64 / 128.;
        let failures: Vec<_> = polygons
            .iter()
            .enumerate()
            .filter_map(|(i, (p, s))| match level(p, s, height, top, minimum) {
                Ok(LevelCut::Sliver) | Err(_) => Some(i),
                _ => None,
            })
            .collect();
        if failures.len() < count {
            count = failures.len();
            best = (height, failures);
        }
        if count == 0 {
            break;
        }
    }
    best
}
