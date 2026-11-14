//! Native closure certificate for level-cut revolution shells (inches/radians).
//! A paired oriented mesh with an everywhere positive radial projection and
//! degree-one boundary covers the entire cylindrical parameter band: degree
//! cannot change away from a mesh boundary. This is not a point-sampling test.
use crate::model::Piece;
use serde::Serialize;
use std::f64::consts::TAU;
/// Explicit endpoint/joint tolerance in inches and angular coverage tolerance in radians.
pub const POSITION_TOL: f64 = 1e-7;
pub const ANGLE_TOL: f64 = 1e-8;
#[derive(Debug, Serialize)]
pub struct Certificate {
    pub rim_coverage_radians: [f64; 2],
    pub paired_edges: usize,
}
fn close(a: [f64; 3], b: [f64; 3]) -> bool {
    (0..3).all(|k| (a[k] - b[k]).abs() <= POSITION_TOL)
}
/// Certify full-band coverage by oriented edge pairing and positive radial
/// projection, plus exact angular interval union at both rims. Rejects leaks,
/// folds, overlapping rim intervals, missing pieces and nonhorizontal boundaries.
pub fn certify(pieces: &[Piece], heights: [f64; 2]) -> Result<Certificate, String> {
    if pieces.is_empty() || heights[0] >= heights[1] {
        return Err("closure: empty or inverted band".into());
    }
    let mut intervals: [Vec<(f64, f64)>; 2] = [vec![], vec![]];
    let mut paired = 0;
    for piece in pieces {
        let n = piece.plane.n;
        // This affine expression is positive over the WHOLE polygon if it is
        // positive at all vertices. Thus radial projection has no fold or pole.
        if piece.outer.iter().any(|p| {
            !p.iter().all(|x| x.is_finite())
                || n[0] * p[0] + n[2] * p[2] <= POSITION_TOL
                || p[1] < heights[0] - POSITION_TOL
                || p[1] > heights[1] + POSITION_TOL
        }) {
            return Err(format!(
                "closure: {} folds or leaves physical band",
                piece.id
            ));
        }
        let mut oriented_area = 0.;
        for k in 0..piece.outer.len() {
            let a = piece.outer[k];
            let b = piece.outer[(k + 1) % piece.outer.len()];
            oriented_area += n[0] * (a[1] * b[2] - a[2] * b[1])
                + n[1] * (a[2] * b[0] - a[0] * b[2])
                + n[2] * (a[0] * b[1] - a[1] * b[0]);
        }
        if oriented_area <= POSITION_TOL * POSITION_TOL {
            return Err(format!("closure: {} inverted/degenerate polygon", piece.id));
        }
        for edge in &piece.edges {
            let a = piece.outer[edge.from];
            let b = piece.outer[edge.to];
            if let Some(neighbor) = &edge.neighbor {
                let other = pieces
                    .iter()
                    .find(|p| &p.id == neighbor)
                    .ok_or_else(|| format!("closure: missing neighbor {neighbor}"))?;
                let matches = other
                    .edges
                    .iter()
                    .filter(|e| {
                        e.neighbor.as_ref() == Some(&piece.id)
                            && close(a, other.outer[e.to])
                            && close(b, other.outer[e.from])
                    })
                    .count();
                if matches != 1 {
                    return Err(format!(
                        "closure: {} unpaired/reversed joint to {neighbor}",
                        piece.id
                    ));
                }
                paired += 1;
            } else {
                let rim = (0..2)
                    .find(|&k| {
                        (a[1] - heights[k]).abs() <= POSITION_TOL
                            && (b[1] - heights[k]).abs() <= POSITION_TOL
                    })
                    .ok_or_else(|| format!("closure: {} null neighbor inside band", piece.id))?;
                let start = a[2].atan2(a[0]).rem_euclid(TAU);
                let delta = (b[2].atan2(b[0]) - start + std::f64::consts::PI).rem_euclid(TAU)
                    - std::f64::consts::PI;
                if delta.abs() <= ANGLE_TOL || delta.abs() >= std::f64::consts::PI - ANGLE_TOL {
                    return Err("closure: degenerate/ambiguous rim edge".into());
                }
                let lo = (start + delta.min(0.)).rem_euclid(TAU);
                let hi = lo + delta.abs();
                intervals[rim].push((lo, hi.min(TAU)));
                if hi > TAU {
                    intervals[rim].push((0., hi - TAU));
                }
            }
        }
    }
    let mut coverage = [0.; 2];
    for k in 0..2 {
        intervals[k].sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        let mut end = 0.;
        for &(lo, hi) in &intervals[k] {
            if (lo - end).abs() > ANGLE_TOL {
                return Err(format!(
                    "closure: rim {k} gap/overlap at {end}..{lo} radians"
                ));
            }
            coverage[k] += hi - lo;
            end = hi;
        }
        if (end - TAU).abs() > ANGLE_TOL || (coverage[k] - TAU).abs() > ANGLE_TOL {
            return Err(format!(
                "closure: rim {k} coverage {} radians, expected {TAU}",
                coverage[k]
            ));
        }
    }
    Ok(Certificate {
        rim_coverage_radians: coverage,
        paired_edges: paired / 2,
    })
}
