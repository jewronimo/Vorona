//! Rigid assembled-to-flat transforms, in inches.
use crate::{model::Piece, tol::GEOM_EPS};
use nalgebra::Vector3;

/// Flatten an inch-valued planar piece, outer face up and longest directed edge +X.
/// Rejects nonfinite, degenerate, or nonplanar geometry; does not alter vertices.
pub fn flatten(piece: &mut Piece, thickness: f64) -> Result<(), String> {
    let n = Vector3::from(piece.plane.n);
    if piece.outer.len() < 3
        || piece.inner.len() != piece.outer.len()
        || !n.iter().all(|v| v.is_finite())
        || (n.norm() - 1.0).abs() > GEOM_EPS
    {
        return Err(format!("{}: invalid face", piece.id));
    }
    let origin = Vector3::from(piece.outer[0]);
    let mut area = 0.0;
    let mut centroid = Vector3::zeros();
    let mut longest = 0.0;
    let mut x = Vector3::zeros();
    for i in 0..piece.outer.len() {
        let a = Vector3::from(piece.outer[i]);
        let b = Vector3::from(piece.outer[(i + 1) % piece.outer.len()]);
        let weight = (a - origin).cross(&(b - origin)).dot(&n);
        area += weight;
        centroid += (origin + a + b) * (weight / 3.0);
        let edge = b - a;
        if edge.norm_squared() > longest {
            longest = edge.norm_squared();
            x = edge;
        }
    }
    if !area.is_finite() || area.abs() <= GEOM_EPS || longest <= GEOM_EPS * GEOM_EPS {
        return Err(format!("{}: degenerate face", piece.id));
    }
    centroid /= area;
    x = (x - n * x.dot(&n)).normalize();
    let y = n.cross(&x);
    let mut transform = [[0.0; 4]; 4];
    for (i, axis) in [x, y, n].iter().enumerate() {
        for j in 0..3 {
            transform[i][j] = axis[j];
        }
        transform[i][3] = -axis.dot(&centroid);
    }
    transform[3][3] = 1.0;
    for (face, z) in [(&piece.outer, 0.0), (&piece.inner, -thickness)] {
        for point in face {
            let p = apply(&transform, *point);
            if !p.iter().all(|v| v.is_finite()) || (p[2] - z).abs() > GEOM_EPS {
                return Err(format!("{}: face is not at flat z={z}", piece.id));
            }
        }
    }
    piece.flat_xform = transform;
    Ok(())
}

/// Apply a row-major homogeneous rigid transform to an inch-valued point.
pub fn apply(m: &[[f64; 4]; 4], p: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| m[i][0] * p[0] + m[i][1] * p[1] + m[i][2] * p[2] + m[i][3])
}
