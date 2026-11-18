//! A06 physical-band adapter: actual clipped solids, paired outer closure and measured back gaps.
use crate::{
    closure,
    model::{Edge, Params, Piece, Plane},
    pieces::{self, Side},
    rectified::Rectified,
    rectified_planes::Solution,
    rims::{self, LevelCut},
    surface::Surface,
};
use nalgebra::Vector3;
use serde::Serialize;
use std::collections::BTreeMap;
type V = [f64; 3];
fn v(p: V) -> Vector3<f64> {
    p.into()
}
fn id(i: usize) -> String {
    format!("A{i:06}")
}
#[derive(Debug, Serialize)]
pub struct Assembly {
    pub pieces: Vec<Piece>,
    pub rim_heights: [f64; 2],
    pub pattern_phase: f64,
    pub closure: Result<closure::Certificate, String>,
    pub back_gap: [f64; 3],
    pub per_piece_back_gap: Vec<(String, [f64; 3])>,
    pub violations: Vec<(String, String, f64)>,
    pub solid_errors: Vec<String>,
    pub c6_pending: bool,
}
fn stats(mut x: Vec<f64>) -> [f64; 3] {
    if x.is_empty() {
        return [0.; 3];
    }
    x.sort_by(f64::total_cmp);
    let m = x.len() / 2;
    [
        x[0],
        if x.len().is_multiple_of(2) {
            (x[m - 1] + x[m]) / 2.
        } else {
            x[m]
        },
        *x.last().unwrap(),
    ]
}
/// Inch-valued native solids in the full physical band. No rejected in-band
/// piece is silently removed: errors/closure block acceptance. C6 remains held.
pub fn assemble(
    t: &Rectified,
    g: &Solution,
    surface: &Surface,
    p: &Params,
) -> Result<Assembly, String> {
    assemble_at(t, g, surface, p, surface.y_range())
}

/// Search row phase, not the design saw cuts. Each candidate rebuilds surface
/// seeds and solves the native geometry; at most sixteen deterministic phases
/// within the smallest row height are tried. A zero-error assembly wins early.
/// Returns the best diagnosed assembly, including closure/solid failures.
pub fn solve_assembly(surface: &Surface, p: &Params) -> Result<Assembly, String> {
    solve_assembly_reporting(surface, p, |_, _| {})
}

/// Same fixed-rim phase search with per-phase solver-start diagnostics.
pub fn solve_assembly_reporting(
    surface: &Surface,
    p: &Params,
    mut report: impl FnMut(f64, &crate::rectified_planes::Attempt),
) -> Result<Assembly, String> {
    let rows = crate::rectified::row_layout::choose(surface, p)?;
    let height = rows.iter().map(|r| r.height).fold(f64::INFINITY, f64::min);
    let mut best = None;
    let mut failures = Vec::new();
    // A bounded sixty-step zero-start pilot ranks phases by actual
    // clipped-solid defects plus C4 defects. This avoids exhausting five full
    // starts in each poor rim placement before considering a usable phase.
    // Pilot geometry is never accepted unless all original C4/C5 gates pass.
    let mut phases = Vec::new();
    // First probe the existing perturbed start that resolves the cone basin
    // in 2–3 iterations; then allow the zero-tilt vase start up to 60 steps.
    for (pilot_start, steps) in [(Some(2), 8), (None, 60)] {
        if pilot_start.is_some() && (p.depth == 0. || p.randomness > 0.) {
            continue;
        }
        for step in 0..16 {
            let phase = height * step as f64 / 16.;
            let t = crate::rectified::build_with_phase(surface, p, phase)?;
            let pilot = crate::rectified_planes::solve_budget_reporting(
                surface,
                &t,
                p,
                1,
                steps,
                pilot_start,
                |attempt| report(phase, attempt),
            );
            let (geometry, c4_errors) = match &pilot {
                Ok(g) => (Some(g), 0),
                Err(e) => (
                    e.best.as_deref(),
                    e.attempts
                        .iter()
                        .map(|a| a.violations.len())
                        .min()
                        .unwrap_or(usize::MAX / 4),
                ),
            };
            let mut score = (usize::MAX / 2, f64::MAX);
            if let Some(g) = geometry {
                if let Ok(mut candidate) = assemble(&t, g, surface, p) {
                    candidate.pattern_phase = phase;
                    let placement = placement_score(&candidate);
                    score = (placement.0.saturating_add(c4_errors), placement.1);
                    if pilot.is_ok() {
                        if best.as_ref().is_none_or(|a| placement < placement_score(a)) {
                            best = Some(candidate);
                        }
                        if placement == (0, 0.) {
                            return Ok(best.unwrap());
                        }
                    }
                }
            }
            if pilot_start.is_none() {
                phases.push((phase, score));
            }
        }
    }
    phases.sort_by(|a, b| {
        a.1 .0
            .cmp(&b.1 .0)
            .then(a.1 .1.total_cmp(&b.1 .1))
            .then(a.0.total_cmp(&b.0))
    });
    for (phase, _) in phases {
        let t = crate::rectified::build_with_phase(surface, p, phase)?;
        let g = match crate::rectified_planes::solve_reporting(surface, &t, p, |attempt| {
            report(phase, attempt)
        }) {
            Ok(g) => g,
            Err(e) => {
                failures.push(format!("phase {phase}: {e:?}"));
                continue;
            }
        };
        let mut candidate = assemble(&t, &g, surface, p)?;
        candidate.pattern_phase = phase;
        let score = placement_score(&candidate);
        if best.as_ref().is_none_or(|a| score < placement_score(a)) {
            best = Some(candidate);
        }
        if score == (0, 0.) {
            break;
        }
    }
    best.ok_or_else(|| format!("A06 phase search failed: {}", failures.join("; ")))
}

fn placement_score(a: &Assembly) -> (usize, f64) {
    (
        a.solid_errors.len() + a.violations.len() + usize::from(a.closure.is_err()),
        a.violations.iter().map(|v| v.2).sum(),
    )
}
fn assemble_at(
    t: &Rectified,
    g: &Solution,
    surface: &Surface,
    p: &Params,
    heights: [f64; 2],
) -> Result<Assembly, String> {
    let n = t.polygons.len();
    let mut faces = g.planes.clone();
    for ids in &g.polygons[n..] {
        let normal = (v(g.corners[ids[1]]) - v(g.corners[ids[0]]))
            .cross(&(v(g.corners[ids[2]]) - v(g.corners[ids[0]])))
            .normalize();
        faces.push(Plane {
            n: normal.into(),
            d: normal.dot(&v(g.corners[ids[0]])),
        });
    }
    let mut owners = BTreeMap::<[usize; 2], Vec<usize>>::new();
    for (i, ids) in g.polygons.iter().enumerate() {
        for k in 0..ids.len() {
            let a = ids[k];
            let b = ids[(k + 1) % ids.len()];
            owners.entry([a.min(b), a.max(b)]).or_default().push(i);
        }
    }
    let mut raw = Vec::new();
    for (i, ids) in g.polygons.iter().enumerate() {
        let outer: Vec<_> = ids.iter().map(|&k| g.corners[k]).collect();
        let mut sides = Vec::new();
        for k in 0..ids.len() {
            let a = ids[k];
            let b = ids[(k + 1) % ids.len()];
            let neighbor = owners[&[a.min(b), a.max(b)]]
                .iter()
                .copied()
                .find(|&j| j != i);
            let side = match neighbor {
                Some(j) => {
                    pieces::triangle_joint(&faces[i], &faces[j], g.corners[a], g.corners[b], j)
                        .map_err(|e| format!("{} joint {e:?}", id(i)))?
                }
                None => {
                    let normal = (v(g.corners[b]) - v(g.corners[a]))
                        .cross(&v(faces[i].n))
                        .normalize();
                    Side {
                        plane: Plane {
                            n: normal.into(),
                            d: normal.dot(&v(g.corners[a])),
                        },
                        kind: "rim_whole",
                        neighbor: None,
                    }
                }
            };
            sides.push(side);
        }
        raw.push((outer, sides));
    }
    // Same 1.5in owner partial-square minimum at every candidate cut.
    let minimum = 1.5;
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let mut violations = Vec::new();
    let base = t.relief_bases.iter().copied().fold(f64::INFINITY, f64::min);
    let mut retained = Vec::new();
    for (i, (mut outer, mut sides)) in raw.into_iter().enumerate() {
        let mut partial = false;
        let mut absent = false;
        for (top, height, style) in [
            (false, heights[0], &p.rims.bottom),
            (true, heights[1], &p.rims.top),
        ] {
            let beyond = |a: &V| {
                if top {
                    a[1] > height + 1e-9
                } else {
                    a[1] < height - 1e-9
                }
            };
            if outer.iter().all(beyond) {
                absent = true;
                break;
            }
            if style == "whole" && outer.iter().any(beyond) {
                errors.push(format!("{} whole removal overlaps physical band", id(i)));
                absent = true;
                break;
            }
            match rims::triangle_level(&outer, &sides, height, top, minimum) {
                Ok(LevelCut::Kept {
                    outer: o,
                    sides: s,
                    partial: q,
                }) => {
                    outer = o;
                    sides = s;
                    partial |= q;
                }
                Ok(LevelCut::Outside) => {
                    // A tangent support face has no positive-area material in
                    // the physical band. It is not a rejected in-band solid;
                    // any genuinely missing adjacency is still caught below.
                    absent = true;
                    break;
                }
                other => {
                    errors.push(format!("{} physical rim rejected {other:?}", id(i)));
                    absent = true;
                    break;
                }
            }
        }
        if absent {
            continue;
        }
        retained.push((i, outer, sides, partial));
    }
    // Solid failures also remove material. Resolve their exposed neighbors to a
    // fixed point BEFORE exporting edges; a pre-build retained set is stale.
    let mut kept: std::collections::BTreeSet<_> = retained.iter().map(|r| r.0).collect();
    loop {
        let mut failed = Vec::new();
        for (i, outer, sides, _) in &retained {
            if !kept.contains(i) {
                continue;
            }
            let mut exposed = sides.clone();
            square_exposed(outer, &mut exposed, &faces[*i], &kept);
            if let Err(e) = pieces::build(&faces[*i], outer.clone(), exposed, p.thickness) {
                errors.push(format!("{} in-band solid {e:?}", id(*i)));
                failed.push(*i);
            }
        }
        if failed.is_empty() {
            break;
        }
        for i in failed {
            kept.remove(&i);
        }
    }
    for (i, outer, mut sides, partial) in retained {
        if !kept.contains(&i) {
            continue;
        }
        square_exposed(&outer, &mut sides, &faces[i], &kept);
        let solid = match pieces::build(&faces[i], outer, sides, p.thickness) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("{} in-band solid {e:?}", id(i)));
                continue;
            }
        };
        let m = solid.outer.len();
        let normal = v(faces[i].n);
        let axis = if normal[1].abs() < 0.9 {
            Vector3::y()
        } else {
            Vector3::x()
        };
        let a = normal.cross(&axis).normalize();
        let b = normal.cross(&a);
        let q: Vec<_> = solid
            .outer
            .iter()
            .chain(&solid.inner)
            .map(|&p| [v(p).dot(&a), v(p).dot(&b)])
            .collect();
        let square = crate::planes::bounding_square(&q);
        let mut check = |name: &str, deficit: f64| {
            if deficit > 1e-7 {
                violations.push((id(i), name.into(), deficit));
            }
        };
        check("blank_usable", square - p.blank_usable);
        if partial {
            check("min_bounding_square", minimum - square);
        }
        let mut angles = Vec::new();
        let mut edges = Vec::new();
        for k in 0..m {
            let point = solid.outer[k];
            let prev = solid.outer[(k + m - 1) % m];
            let next = solid.outer[(k + 1) % m];
            let u = (v(point) - v(prev)).normalize();
            let w = (v(next) - v(point)).normalize();
            let angle = 180. - normal.dot(&u.cross(&w)).atan2(u.dot(&w)).to_degrees();
            angles.push(angle);
            let length = (v(next) - v(point)).norm();
            if !partial {
                check(
                    if i < n {
                        "min_edge"
                    } else {
                        "min_triangle_edge"
                    },
                    if i < n {
                        p.min_edge - length
                    } else {
                        p.min_triangle_edge - length
                    },
                );
            }
            if !partial && maximum_edge_applies(solid.sides[k].kind) {
                check("max_polygon_edge", length - p.max_polygon_edge);
            }
            check("min_corner_angle_deg", p.min_corner_angle_deg - angle);
            if !partial {
                check("max_corner_angle_deg", angle - p.max_corner_angle_deg);
                check("bevel_cap_deg", solid.bevel_deg[k] - p.bevel_cap_deg);

                let distance = surface.signed_distance(point);
                check("corner_band_low", -(base + p.depth + 0.4) - distance);
                check("corner_band_high", distance - 0.75);
            }
            check(
                "flatness",
                (normal.dot(&v(point)) - faces[i].d).abs() - 1e-6,
            );
            edges.push(Edge {
                from: k,
                to: (k + 1) % m,
                neighbor: solid.sides[k].neighbor.map(id),
                kind: solid.sides[k].kind.into(),
                length,
                bevel_deg: if normal.dot(&v(solid.sides[k].plane.n)).abs() < 1e-14 {
                    0.
                } else {
                    solid.bevel_deg[k]
                },
                side_normal: solid.sides[k].plane.n,
            });
        }
        let max_distance = solid
            .outer
            .iter()
            .map(|&p| surface.signed_distance(p))
            .fold(f64::NEG_INFINITY, f64::max);
        let reflex = angles.iter().filter(|&&a| a > 180.).count();
        output.push(Piece {
            id: id(i),
            sides: m as u32,
            partial,
            plane: faces[i].clone(),
            outer: solid.outer,
            inner: solid.inner,
            edges,
            corner_angles_deg: angles,
            face_depth: 0.,
            seed_depth: if i < n {
                Some(t.delaunay.tiles[i].depth)
            } else {
                None
            },
            max_corner_distance: Some(max_distance),
            reflex_corners: Some(reflex),
            bounding_square: square,
            flat_xform: identity(),
            blank: 0,
            blank_xform: identity(),
            map_uv: Vec::new(),
        });
    }
    let certificate = closure::certify(&output, heights);
    let mut gaps = Vec::new();
    let mut per_piece = Vec::new();
    let ids: BTreeMap<_, _> = output
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.as_str(), i))
        .collect();
    for piece in &output {
        let mut local = Vec::new();
        for edge in &piece.edges {
            let Some(neighbor) = edge.neighbor.as_ref() else {
                continue;
            };
            let Some(&j) = ids.get(neighbor.as_str()) else {
                errors.push(format!("{} dangling {neighbor}", piece.id));
                continue;
            };
            let other = &output[j];
            let opposite = other.edges.iter().find(|e| {
                e.neighbor.as_ref() == Some(&piece.id)
                    && (v(other.outer[e.to]) - v(piece.outer[edge.from])).norm() < 1e-7
                    && (v(other.outer[e.from]) - v(piece.outer[edge.to])).norm() < 1e-7
            });
            if let Some(e) = opposite {
                let gap = (v(piece.inner[edge.from]) - v(other.inner[e.to]))
                    .norm()
                    .max((v(piece.inner[edge.to]) - v(other.inner[e.from])).norm());
                local.push(gap);
                if piece.id < other.id {
                    gaps.push(gap);
                }
                if !piece.partial && gap > p.max_back_gap + 1e-7 {
                    violations.push((
                        piece.id.clone(),
                        "max_back_gap".into(),
                        gap - p.max_back_gap,
                    ));
                }
            } else {
                errors.push(format!("{} unpaired joint {neighbor}", piece.id));
            }
        }
        per_piece.push((piece.id.clone(), stats(local)));
    }
    Ok(Assembly {
        pieces: output,
        rim_heights: heights,
        pattern_phase: 0.,
        closure: certificate,
        back_gap: stats(gaps),
        per_piece_back_gap: per_piece,
        violations,
        solid_errors: errors,
        c6_pending: true,
    })
}
fn square_exposed(
    outer: &[V],
    sides: &mut [Side],
    face: &Plane,
    kept: &std::collections::BTreeSet<usize>,
) {
    for k in 0..sides.len() {
        if sides[k].neighbor.is_some_and(|j| !kept.contains(&j)) {
            let normal = (v(outer[(k + 1) % outer.len()]) - v(outer[k]))
                .cross(&v(face.n))
                .normalize();
            sides[k] = Side {
                plane: Plane {
                    n: normal.into(),
                    d: normal.dot(&v(outer[k])),
                },
                kind: "rim_whole",
                neighbor: None,
            };
        }
    }
}
fn identity() -> [[f64; 4]; 4] {
    [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]
}

/// Owner §9.2 exemption is provenance-specific, never piece-wide.
/// All other manufacturing checks still apply to every retained rim piece.
fn maximum_edge_applies(kind: &str) -> bool {
    !matches!(kind, "rim_level" | "rim_whole")
}

#[cfg(test)]
mod rim_policy_tests {
    #[test]
    fn failed_solid_neighbor_becomes_square_without_hiding_rejection() {
        use super::*;
        let outer = vec![[0., 0., 0.], [2., 0., 0.], [0., 2., 0.]];
        let face = Plane {
            n: [0., 0., 1.],
            d: 0.,
        };
        let mut sides: Vec<_> = (0..3)
            .map(|k| pieces::triangle_joint(&face, &face, outer[k], outer[(k + 1) % 3], 7).unwrap())
            .collect();
        let kept = [7].into_iter().collect();
        square_exposed(&outer, &mut sides, &face, &kept);
        assert!(sides.iter().all(|s| s.neighbor == Some(7)));
        square_exposed(&outer, &mut sides, &face, &Default::default());
        assert!(sides
            .iter()
            .all(|s| s.neighbor.is_none() && s.kind == "rim_whole"));
        let solid = pieces::build(&face, outer, sides, 0.375).unwrap();
        assert!(solid.bevel_deg.iter().all(|&b| b == 0.));
    }
    #[test]
    fn only_saw_edges_are_exempt_from_maximum() {
        assert!(!super::maximum_edge_applies("rim_level"));
        assert!(!super::maximum_edge_applies("rim_whole"));
        assert!(super::maximum_edge_applies("joint"));
        assert!(super::maximum_edge_applies("unknown"));
    }
}
