//! Periodic Delaunay dual topology; all indices refer to original tile sites.
use crate::seeds::Seed;
use serde::Serialize;
use spade::{DelaunayTriangulation, HasPosition, Point2, Triangulation};
use std::{
    collections::{BTreeMap, BTreeSet},
    f64::consts::TAU,
};

#[derive(Clone, Debug)]
struct Site {
    position: Point2<f64>,
    index: usize,
    shift: i32,
}
impl HasPosition for Site {
    type Scalar = f64;
    fn position(&self) -> Point2<f64> {
        self.position
    }
}

/// Stable topology for C4. Corners are sorted triples, not geometric coordinates.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Topology {
    pub tiles: Vec<Seed>,
    pub neighbors: Vec<Vec<usize>>,
    pub corners: Vec<[usize; 3]>,
    pub boundary: Vec<bool>,
}
impl Topology {
    /// Side count of a tile (dimensionless); caller supplies a valid tile index.
    pub fn degree(&self, index: usize) -> usize {
        self.neighbors[index].len()
    }
    /// Construct the periodic quotient in dimensionless conformal coordinates.
    /// Rejects duplicate/nonfinite sites, collapsed faces or nonmanifold quotients.
    pub fn build(tiles: Vec<Seed>) -> Result<Self, String> {
        Self::build_impl(tiles, false)
    }
    /// A06 periodic quotient with lexicographically smaller integer diagonal
    /// for cocircular ties (relative determinant tolerance 1e-12). Units are
    /// conformal; rejects the same invalid sites/manifolds as build, or tie cap.
    /// Site positions are never perturbed; legacy topology is unaffected.
    pub fn build_a06(tiles: Vec<Seed>) -> Result<Self, String> {
        Self::build_impl(tiles, true)
    }
    fn build_impl(tiles: Vec<Seed>, stable_ties: bool) -> Result<Self, String> {
        if tiles.len() < 4 {
            return Err("too few periodic sites".into());
        }
        let mut triangulation = DelaunayTriangulation::<Site>::new();
        for shift in [-1, 0, 1] {
            for (index, tile) in tiles.iter().enumerate() {
                if !tile.u.is_finite() || !tile.v.is_finite() || !(0.0..TAU).contains(&tile.u) {
                    return Err(format!("invalid conformal site {index}"));
                }
                triangulation
                    .insert(Site {
                        position: Point2::new(tile.u + f64::from(shift) * TAU, tile.v),
                        index,
                        shift,
                    })
                    .map_err(|e| format!("Delaunay insertion: {e:?}"))?;
            }
        }
        if triangulation.num_vertices() != 3 * tiles.len() {
            return Err("duplicate periodic sites".into());
        }
        let mut corners = BTreeSet::new();
        for face in triangulation.inner_faces() {
            let vertices = face.vertices();
            // Select the lift by an integer identity, never a floating-point
            // centroid at the chart cut. The smallest original site's central
            // copy owns this triangle, including triangles straddling u=0.
            let anchor = vertices.iter().min_by_key(|v| v.data().index).unwrap();
            if anchor.data().shift != 0 {
                continue;
            }
            let mut ids = vertices.map(|v| v.data().index);
            ids.sort_unstable();
            if ids[0] == ids[1] || ids[1] == ids[2] {
                return Err("collapsed periodic triangle".into());
            }
            corners.insert(ids);
        }
        let mut corners: Vec<_> = corners.into_iter().collect();
        if stable_ties {
            resolve_cocircular_ties(&tiles, &mut corners)?;
        }
        let mut edges = BTreeMap::<(usize, usize), usize>::new();
        for &[a, b, c] in &corners {
            for edge in [(a, b), (a, c), (b, c)] {
                *edges.entry(edge).or_default() += 1;
            }
        }
        let mut neighbors = vec![Vec::new(); tiles.len()];
        let mut boundary = vec![false; tiles.len()];
        let first_row = tiles.iter().map(|s| s.row).min().unwrap();
        let last_row = tiles.iter().map(|s| s.row).max().unwrap();
        for ((a, b), count) in edges {
            if count > 2 {
                return Err(format!("nonmanifold periodic edge {a}-{b}"));
            }
            neighbors[a].push(b);
            neighbors[b].push(a);
            if count == 1 {
                if tiles[a].row != tiles[b].row || ![first_row, last_row].contains(&tiles[a].row) {
                    return Err(format!("open periodic interior edge {a}-{b}"));
                }
                boundary[a] = true;
                boundary[b] = true;
            }
        }
        for (i, list) in neighbors.iter_mut().enumerate() {
            list.sort_unstable();
            if list.is_empty() {
                return Err(format!("isolated tile {i}"));
            }
        }
        Ok(Self {
            tiles,
            neighbors,
            corners,
            boundary,
        })
    }
}

// Every flip strictly decreases the sorted integer edge set. No coordinate
// perturbation or RNG: the same nearly cocircular quadrilateral chooses the
// same diagonal on either side of a floating-point chart cut.
fn resolve_cocircular_ties(tiles: &[Seed], faces: &mut [[usize; 3]]) -> Result<(), String> {
    for _ in 0..tiles.len().saturating_mul(tiles.len()) {
        let mut edges = BTreeMap::<[usize; 2], Vec<(usize, usize)>>::new();
        for (f, &[a, b, c]) in faces.iter().enumerate() {
            for (e, opposite) in [([a, b], c), ([a, c], b), ([b, c], a)] {
                edges.entry(e).or_default().push((f, opposite));
            }
        }
        let mut changed = false;
        for ([a, b], incident) in &edges {
            if incident.len() != 2 {
                continue;
            }
            let (fa, c) = incident[0];
            let (fb, d) = incident[1];
            let mut alternate = [c, d];
            alternate.sort_unstable();
            if alternate >= [*a, *b] || edges.contains_key(&alternate) {
                continue;
            }
            let xy = |i: usize| {
                [
                    (tiles[i].u - tiles[*a].u + TAU / 2.).rem_euclid(TAU) - TAU / 2.,
                    tiles[i].v - tiles[*a].v,
                ]
            };
            let q = [xy(*a), xy(*b), xy(c), xy(d)];
            let cross = |x: [f64; 2], y: [f64; 2], z: [f64; 2]| {
                (y[0] - x[0]) * (z[1] - x[1]) - (y[1] - x[1]) * (z[0] - x[0])
            };
            // Only a convex quadrilateral can exchange diagonals.
            if cross(q[0], q[1], q[2]) * cross(q[0], q[1], q[3]) >= 0.
                || cross(q[2], q[3], q[0]) * cross(q[2], q[3], q[1]) >= 0.
            {
                continue;
            }
            let mut m = [[0.; 3]; 3];
            let mut scale = 0.0_f64;
            for j in 0..3 {
                let x = q[j][0] - q[3][0];
                let y = q[j][1] - q[3][1];
                m[j] = [x, y, x * x + y * y];
                scale = scale.max(x.abs()).max(y.abs());
            }
            let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
            if det.abs() > 1e-12 * scale.powi(4) {
                continue;
            }
            faces[fa] = [c, d, *a];
            faces[fa].sort_unstable();
            faces[fb] = [c, d, *b];
            faces[fb].sort_unstable();
            changed = true;
            break;
        }
        if !changed {
            faces.sort_unstable();
            return Ok(());
        }
    }
    Err("A06 cocircular integer tie cap exceeded".into())
}
