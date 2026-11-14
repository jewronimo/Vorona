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
        let corners: Vec<_> = corners.into_iter().collect();
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
