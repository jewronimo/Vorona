//! C5 topology adapter. C6 numbering/validation and C7 nesting remain explicit pending gates.
use crate::{
    model::*,
    pieces::{self, Point, Side},
    planes::{Planes, SurfaceQueries},
    rims::{self, LevelCut},
    surface::Surface,
    topology::Topology,
};
use nalgebra::{Matrix3, Vector3};
use std::collections::{BTreeMap, BTreeSet};
/// A05 metadata retained alongside pieces for adapter diagnostics.
#[derive(Clone, Debug)]
pub struct Metadata {
    pub seed: usize,
    pub seed_depth: f64,
    pub max_corner_distance: f64,
    pub reflex_corners: usize,
}
/// Complete adapter result, including EVERY removed/rejected tile and selected rim heights.
#[derive(Debug)]
pub struct Assembly {
    pub pieces: Vec<Piece>,
    pub metadata: Vec<Metadata>,
    pub rejections: Vec<String>,
    pub rim_heights: [f64; 2],
}
fn v(p: Point) -> Vector3<f64> {
    Vector3::from(p)
}
fn id(i: usize) -> String {
    format!("S{i:06}")
}
fn square_side(face: &Plane, a: Point, b: Point) -> Side {
    let n = (v(b) - v(a)).cross(&v(face.n)).normalize();
    Side {
        plane: Plane {
            n: n.into(),
            d: n.dot(&v(a)),
        },
        kind: "rim_whole",
        neighbor: None,
    }
}
/// Build pieces from fitted planes/topological fans in inches; rejects malformed fans explicitly.
/// Shared corner cache is keyed by sorted constraints, not traversal or approximate coordinates.
pub fn assemble(
    t: &Topology,
    g: &Planes,
    surface: &Surface,
    p: &Params,
) -> Result<Assembly, String> {
    if g.planes.len() != t.tiles.len()
        || g.polygons.len() != t.tiles.len()
        || g.corners.len() != t.corners.len()
    {
        return Err("inconsistent C5 topology".into());
    }
    let mut raw = Vec::new();
    let mut seeds = Vec::new();
    let mut rejects = Vec::new();
    for (i, fan) in g.polygons.iter().enumerate() {
        if fan.len() < 3 {
            if !t.tiles[i].auxiliary
                || (t.tiles[i].row != t.tiles.iter().map(|s| s.row).min().unwrap()
                    && t.tiles[i].row != t.tiles.iter().map(|s| s.row).max().unwrap())
            {
                return Err(format!("closure: physical seed {i} has incomplete fan"));
            }
            rejects.push(format!(
                "{}: {}",
                id(i),
                if t.boundary[i] {
                    "open auxiliary support fan (not a physical piece)"
                } else {
                    "incomplete fan"
                }
            ));
            continue;
        }
        let outer: Vec<_> = fan.iter().map(|&k| g.corners[k]).collect();
        let mut sides = Vec::new();
        for k in 0..fan.len() {
            let a = t.corners[fan[k]];
            let b = t.corners[fan[(k + 1) % fan.len()]];
            let j = a
                .iter()
                .copied()
                .find(|&j| j != i && b.contains(&j))
                .ok_or("nonadjacent fan corners")?;
            sides.push(
                pieces::joint(&g.planes[i], &g.planes[j], j)
                    .map_err(|e| format!("{}: {e:?}", id(i)))?,
            );
        }
        seeds.push(i);
        raw.push((outer, sides));
    }
    let physical = surface.y_range();
    let mut heights = physical;
    // Placement is evaluated before any rejection/cutting. End searches are independent
    // on an extended lattice; for short profiles the second pass checks the actual first cut.
    if p.rims.bottom == "level" {
        heights[0] = rims::place(
            &raw,
            physical[0],
            false,
            p.target_size.min((physical[1] - physical[0]) / 4.),
            p.min_bounding_square,
        )
        .0;
    }
    let mut bottom = Vec::new();
    for (outer, sides) in &raw {
        bottom.push(
            match rims::level(outer, sides, heights[0], false, p.min_bounding_square) {
                Ok(LevelCut::Kept { outer, sides, .. }) => (outer, sides),
                _ => (Vec::new(), Vec::new()),
            },
        );
    }
    if p.rims.top == "level" {
        let candidates: Vec<_> = bottom
            .iter()
            .filter(|(o, _)| o.len() >= 3)
            .cloned()
            .collect();
        heights[1] = rims::place(
            &candidates,
            physical[1],
            true,
            p.target_size.min((physical[1] - physical[0]) / 4.),
            p.min_bounding_square,
        )
        .0;
    }
    let mut cut = Vec::new();
    for ((mut outer, mut sides), i) in raw.into_iter().zip(seeds) {
        let mut partial = false;
        let mut removed = false;
        for (top, h, style) in [
            (false, heights[0], &p.rims.bottom),
            (true, heights[1], &p.rims.top),
        ] {
            let beyond = |a: &Point| {
                if top {
                    a[1] > h + 1e-9
                } else {
                    a[1] < h - 1e-9
                }
            };
            if outer.iter().all(beyond) {
                rejects.push(format!(
                    "{}: outside {} rim",
                    id(i),
                    if top { "top" } else { "bottom" }
                ));
                removed = true;
                break;
            }
            if style == "whole" {
                if outer.iter().any(beyond) {
                    return Err(format!(
                        "closure: {} whole removal overlaps physical band",
                        id(i)
                    ));
                }
            } else {
                match rims::level(&outer, &sides, h, top, p.min_bounding_square) {
                    Ok(LevelCut::Kept {
                        outer: o,
                        sides: s,
                        partial: q,
                    }) => {
                        outer = o;
                        sides = s;
                        partial |= q;
                    }
                    other => {
                        return Err(format!(
                            "closure: {} rejected in-band tile {other:?}",
                            id(i)
                        ));
                    }
                }
            }
        }
        if !removed {
            cut.push((i, outer, sides, partial));
        }
    }
    let kept: BTreeSet<_> = cut.iter().map(|x| x.0).collect();
    // Constraints: face i uses key i; bottom/top horizontal planes use N,N+1.
    let n = g.planes.len();
    let mut cache = BTreeMap::<Vec<usize>, (Point, Point)>::new();
    let mut output = Vec::new();
    let mut metadata = Vec::new();
    for (i, mut outer, mut sides, partial) in cut {
        for k in 0..sides.len() {
            if sides[k].neighbor.is_some_and(|j| !kept.contains(&j)) {
                sides[k] = square_side(&g.planes[i], outer[k], outer[(k + 1) % outer.len()]);
            }
        }
        // Canonical solve for all-joint and joint/rim corners; whole edges retain local solve.
        let mut cached = Vec::new();
        for k in 0..outer.len() {
            let pair = [&sides[(k + sides.len() - 1) % sides.len()], &sides[k]];
            if pair.iter().any(|s| s.kind == "rim_whole") {
                cached.push(None);
                continue;
            }
            let mut keys = vec![i];
            for s in pair {
                keys.push(
                    s.neighbor
                        .unwrap_or(if s.plane.n[1] < 0. { n } else { n + 1 }),
                );
            }
            keys.sort_unstable();
            keys.dedup();
            if keys.len() != 3 {
                return Err(format!("closure: {} invalid corner constraints", id(i)));
            }
            if !cache.contains_key(&keys) {
                let constraints: Vec<_> = keys
                    .iter()
                    .map(|&k| {
                        if k < n {
                            (g.planes[k].clone(), p.thickness)
                        } else {
                            (
                                Plane {
                                    n: [0., 1., 0.],
                                    d: heights[k - n],
                                },
                                0.,
                            )
                        }
                    })
                    .collect();
                let m = Matrix3::from_rows(&std::array::from_fn::<_, 3, _>(|k| {
                    v(constraints[k].0.n).transpose()
                }));
                if m.determinant().abs() < 1e-8 {
                    return Err(format!(
                        "closure: {} singular canonical corner {keys:?}",
                        id(i)
                    ));
                }
                let rhs = Vector3::from_fn(|k, _| constraints[k].0.d);
                let inward = Vector3::from_fn(|k, _| constraints[k].0.d - constraints[k].1);
                let lu = m.lu();
                let a = lu.solve(&rhs).ok_or("corner solve")?;
                let b = lu.solve(&inward).ok_or("inner corner solve")?;
                cache.insert(keys.clone(), (a.into(), b.into()));
            }
            outer[k] = cache[&keys].0;
            cached.push(Some(cache[&keys].1));
        }
        let mut solid = match pieces::build(&g.planes[i], outer, sides, p.thickness) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("closure: {} rejected in-band solid {e:?}", id(i)));
            }
        };
        for (k, x) in cached.into_iter().enumerate() {
            if let Some(x) = x {
                solid.inner[k] = x;
            }
        }
        let len = solid.outer.len();
        let normal = v(g.planes[i].n);
        let angles: Vec<_> = (0..len)
            .map(|k| {
                let a = (v(solid.outer[k]) - v(solid.outer[(k + len - 1) % len])).normalize();
                let b = (v(solid.outer[(k + 1) % len]) - v(solid.outer[k])).normalize();
                180. - normal.dot(&a.cross(&b)).atan2(a.dot(&b)).to_degrees()
            })
            .collect();
        let centroid = solid.outer.iter().fold(Vector3::zeros(), |a, &b| a + v(b)) / len as f64;
        let face_depth = std::iter::once(centroid.into())
            .chain(
                (0..len).map(|k| ((v(solid.outer[k]) + v(solid.outer[(k + 1) % len])) / 2.).into()),
            )
            .map(|x| -surface.distance(x))
            .fold(0., f64::max);
        metadata.push(Metadata {
            seed: i,
            seed_depth: t.tiles[i].depth,
            max_corner_distance: solid
                .outer
                .iter()
                .map(|&x| surface.distance(x))
                .fold(f64::NEG_INFINITY, f64::max),
            reflex_corners: angles.iter().filter(|&&a| a > 180.).count(),
        });
        let edges = (0..len)
            .map(|k| Edge {
                from: k,
                to: (k + 1) % len,
                neighbor: solid.sides[k].neighbor.map(id),
                kind: solid.sides[k].kind.into(),
                length: (v(solid.outer[k]) - v(solid.outer[(k + 1) % len])).norm(),
                bevel_deg: solid.bevel_deg[k],
                side_normal: solid.sides[k].plane.n,
            })
            .collect();
        let bounding_square = rims::enclosing_square(&solid.outer);
        output.push(Piece {
            id: id(i),
            sides: len as u32,
            partial,
            plane: g.planes[i].clone(),
            outer: solid.outer,
            inner: solid.inner,
            edges,
            corner_angles_deg: angles,
            face_depth,
            seed_depth: Some(t.tiles[i].depth),
            max_corner_distance: Some(
                metadata
                    .last()
                    .expect("metadata inserted")
                    .max_corner_distance,
            ),
            reflex_corners: Some(metadata.last().expect("metadata inserted").reflex_corners),
            bounding_square,
            flat_xform: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
            blank: 0,
            blank_xform: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
            map_uv: vec![],
        });
    }
    // A failed solid must not leave dangling joint neighbors. Do not silently square
    // those edges: geometry was built with bisectors, so expose a validation failure.
    let ids: BTreeSet<_> = output.iter().map(|p| p.id.clone()).collect();
    for piece in &output {
        for edge in &piece.edges {
            if edge.neighbor.as_ref().is_some_and(|j| !ids.contains(j)) {
                rejects.push(format!("{}: joint neighbor rejected", piece.id));
            }
        }
    }
    crate::closure::certify(&output, heights)?;
    Ok(Assembly {
        pieces: output,
        metadata,
        rejections: rejects,
        rim_heights: heights,
    })
}
