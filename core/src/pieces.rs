//! First C5 increment: closed solids from a planar boundary and explicit side planes.
//! No contract/numbering/nesting placeholders are emitted by this internal geometry API.
use crate::{model::Plane, tol::GEOM_EPS};
use nalgebra::{Matrix3, Vector3};
pub type Point = [f64; 3];
/// One side plane; joints use the difference of the adjacent face equations.
#[derive(Clone, Debug)]
pub struct Side {
    pub plane: Plane,
    pub kind: &'static str,
    pub neighbor: Option<usize>,
}
/// Closed geometry in inches. Quads follow outer edge then reversed inner edge.
#[derive(Clone, Debug)]
pub struct Solid {
    pub outer: Vec<Point>,
    pub inner: Vec<Point>,
    pub sides: Vec<Side>,
    pub quads: Vec<[Point; 4]>,
    pub bevel_deg: Vec<f64>,
}
/// Explicit failure, rather than a panicking or falsely valid thin/sliver piece.
#[derive(Clone, Debug, PartialEq)]
pub enum PieceError {
    InvalidInput,
    SingularCorner,
    InvalidPolygon,
    CollapsedOffset,
}
fn v(p: Point) -> Vector3<f64> {
    Vector3::from(p)
}
/// Joint bisector in inches, valid for distinct unit outward normals; rejects parallel faces.
pub fn joint(a: &Plane, b: &Plane, neighbor: usize) -> Result<Side, PieceError> {
    let n = v(a.n) - v(b.n);
    let len = n.norm();
    if !len.is_finite() || len <= GEOM_EPS {
        return Err(PieceError::InvalidInput);
    }
    Ok(Side {
        plane: Plane {
            n: (n / len).into(),
            d: (a.d - b.d) / len,
        },
        kind: "joint",
        neighbor: Some(neighbor),
    })
}
fn valid_plane(p: &Plane) -> bool {
    p.d.is_finite() && p.n.iter().all(|x| x.is_finite()) && (v(p.n).norm() - 1.).abs() <= GEOM_EPS
}
fn polygon_ok(points: &[Point], n: Point) -> bool {
    let normal = v(n);
    let axis = if n[0].abs() < 0.9 {
        Vector3::x()
    } else {
        Vector3::y()
    };
    let u = normal.cross(&axis).normalize();
    let w = normal.cross(&u);
    let q: Vec<_> = points
        .iter()
        .map(|p| [v(*p).dot(&u), v(*p).dot(&w)])
        .collect();
    let orient = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    };
    let len = q.len();
    let area: f64 = (0..len)
        .map(|i| q[i][0] * q[(i + 1) % len][1] - q[i][1] * q[(i + 1) % len][0])
        .sum();
    if area <= GEOM_EPS {
        return false;
    }
    for i in 0..len {
        if (v(points[i]) - v(points[(i + 1) % len])).norm() <= GEOM_EPS {
            return false;
        }
        for j in i + 1..len {
            if j == (i + 1) % len || (j + 1) % len == i {
                continue;
            }
            let (a, b, c, d) = (q[i], q[(i + 1) % len], q[j], q[(j + 1) % len]);
            let o = [
                orient(a, b, c),
                orient(a, b, d),
                orient(c, d, a),
                orient(c, d, b),
            ];
            // Includes touching/overlapping nonadjacent segments.
            let overlap = (0..2).all(|k| {
                a[k].min(b[k]) <= c[k].max(d[k]) + GEOM_EPS
                    && c[k].min(d[k]) <= a[k].max(b[k]) + GEOM_EPS
            });
            if overlap && o[0] * o[1] <= GEOM_EPS * GEOM_EPS && o[2] * o[3] <= GEOM_EPS * GEOM_EPS {
                return false;
            }
        }
    }
    true
}
/// Offset a CCW face inward by thickness (inches), keeping side planes fixed.
/// Returns explicit validation errors for non-simple/reversed/collapsed offsets.
/// Supports reflex outer corners; input sides correspond to outgoing polygon edges.
pub fn build(
    face: &Plane,
    outer: Vec<Point>,
    sides: Vec<Side>,
    thickness: f64,
) -> Result<Solid, PieceError> {
    let len = outer.len();
    if len < 3
        || sides.len() != len
        || !valid_plane(face)
        || !thickness.is_finite()
        || thickness <= 0.
        || sides.iter().any(|s| !valid_plane(&s.plane))
        || outer.iter().flatten().any(|x| !x.is_finite())
    {
        return Err(PieceError::InvalidInput);
    }
    for i in 0..len {
        if (v(face.n).dot(&v(outer[i])) - face.d).abs() > GEOM_EPS
            || [outer[i], outer[(i + 1) % len]]
                .iter()
                .any(|p| (v(sides[i].plane.n).dot(&v(*p)) - sides[i].plane.d).abs() > GEOM_EPS)
        {
            return Err(PieceError::InvalidInput);
        }
    }
    if !polygon_ok(&outer, face.n) {
        return Err(PieceError::InvalidPolygon);
    }
    let mut inner = Vec::with_capacity(len);
    for i in 0..len {
        let a = &sides[(i + len - 1) % len].plane;
        let b = &sides[i].plane;
        let m = Matrix3::from_rows(&[
            v(face.n).transpose(),
            v(a.n).transpose(),
            v(b.n).transpose(),
        ]);
        if m.determinant().abs() < 1e-8 {
            return Err(PieceError::SingularCorner);
        }
        let x = m
            .lu()
            .solve(&Vector3::new(face.d - thickness, a.d, b.d))
            .ok_or(PieceError::SingularCorner)?;
        if !x.iter().all(|x| x.is_finite()) {
            return Err(PieceError::CollapsedOffset);
        }
        inner.push(x.into());
    }
    // Edge reversal can leave a convex polygon simple after an offset passes its collapse.
    if !polygon_ok(&inner, face.n)
        || (0..len).any(|i| {
            (v(inner[(i + 1) % len]) - v(inner[i])).dot(&(v(outer[(i + 1) % len]) - v(outer[i])))
                / (v(outer[(i + 1) % len]) - v(outer[i])).norm()
                < crate::tol::MIN_INNER_EDGE * (1. - 1e-7)
        })
    {
        return Err(PieceError::CollapsedOffset);
    }
    let quads = (0..len)
        .map(|i| {
            [
                outer[i],
                outer[(i + 1) % len],
                inner[(i + 1) % len],
                inner[i],
            ]
        })
        .collect();
    let bevel_deg = sides
        .iter()
        .map(|s| {
            v(s.plane.n)
                .dot(&v(face.n))
                .abs()
                .clamp(0., 1.)
                .asin()
                .to_degrees()
        })
        .collect();
    Ok(Solid {
        outer,
        inner,
        sides,
        quads,
        bevel_deg,
    })
}
