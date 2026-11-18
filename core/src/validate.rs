//! A06 manufacturing checks. C7 packing is deliberately not certified here.
use crate::{
    model::{Check, Params, Validation},
    rectified_assembly::Assembly,
};
use std::collections::BTreeSet;
/// Validate native A06 solids and the independently measured adapter gates.
/// Failure ids are sorted; all distances are inches, angles degrees.
pub fn triangles(a: &Assembly, p: &Params) -> Validation {
    let mut checks = Vec::new();
    for (id, name, limits) in [
        ("R-VA-1", "piece topology", vec![]),
        (
            "R-VA-2",
            "usable square and partial slivers",
            vec!["blank_usable", "min_bounding_square"],
        ),
        (
            "R-VA-3",
            "edge lengths",
            vec!["min_edge", "min_triangle_edge", "max_polygon_edge"],
        ),
        (
            "R-VA-4",
            "angles and solid validity",
            vec!["min_corner_angle_deg", "max_corner_angle_deg"],
        ),
        ("R-VA-5", "bevel cap", vec!["bevel_cap_deg"]),
        (
            "R-VA-6",
            "flatness and shared joint planes",
            vec!["flatness"],
        ),
        (
            "R-VA-7",
            "corner band and back gap",
            vec!["corner_band_low", "corner_band_high", "max_back_gap"],
        ),
        ("R-VA-8", "full band closure and adjacency", vec![]),
    ] {
        let mut failures: BTreeSet<String> = a
            .violations
            .iter()
            .filter(|v| limits.contains(&v.1.as_str()))
            .map(|v| v.0.clone())
            .collect();
        if id == "R-VA-4" {
            failures.extend(a.solid_errors.iter().cloned());
        }
        if id == "R-VA-8" {
            if let Err(e) = &a.closure {
                failures.insert(e.clone());
            }
        }
        for piece in &a.pieces {
            let bad = match id {
                "R-VA-1" => {
                    piece.sides < 3
                        || piece.outer.len() != piece.sides as usize
                        || piece.edges.len() != piece.outer.len()
                }
                "R-VA-2" => {
                    !piece.bounding_square.is_finite()
                        || piece.bounding_square > p.blank_usable + 1e-7
                        || (piece.partial && piece.bounding_square < 1.5 - 1e-7)
                }
                "R-VA-3" => {
                    !piece.partial
                        && piece.edges.iter().any(|e| {
                            !e.length.is_finite()
                                || e.length
                                    < (if piece.seed_depth.is_some() {
                                        p.min_edge
                                    } else {
                                        p.min_triangle_edge
                                    }) - 1e-7
                                || (e.kind == "joint" && e.length > p.max_polygon_edge + 1e-7)
                        })
                }
                "R-VA-4" => {
                    piece.inner.len() != piece.outer.len()
                        || piece.corner_angles_deg.iter().any(|&a| {
                            !a.is_finite()
                                || a < p.min_corner_angle_deg - 1e-7
                                || (!piece.partial && a > p.max_corner_angle_deg + 1e-7)
                        })
                }
                "R-VA-5" => {
                    !piece.partial
                        && piece.edges.iter().any(|e| {
                            !e.bevel_deg.is_finite() || e.bevel_deg > p.bevel_cap_deg + 1e-7
                        })
                }
                "R-VA-7" => {
                    !piece.partial
                        && a.per_piece_back_gap
                            .iter()
                            .any(|(id, g)| id == &piece.id && g[2] > p.max_back_gap + 1e-7)
                }
                "R-VA-6" => piece.outer.iter().any(|v| {
                    ((0..3).map(|k| v[k] * piece.plane.n[k]).sum::<f64>() - piece.plane.d).abs()
                        > 1e-6
                }),
                _ => false,
            };
            if bad {
                failures.insert(piece.id.clone());
            }
            for e in &piece.edges {
                if let Some(n) = &e.neighbor {
                    let other = a.pieces.iter().find(|q| &q.id == n);
                    if id == "R-VA-8"
                        && other.is_none_or(|q| {
                            !q.edges
                                .iter()
                                .any(|f| f.neighbor.as_ref() == Some(&piece.id))
                        })
                    {
                        failures.insert(piece.id.clone());
                    }
                    if id == "R-VA-6" {
                        if let Some(q) = other {
                            for f in q
                                .edges
                                .iter()
                                .filter(|f| f.neighbor.as_ref() == Some(&piece.id))
                            {
                                let dot: f64 =
                                    (0..3).map(|k| e.side_normal[k] * f.side_normal[k]).sum();
                                let d: f64 = (0..3)
                                    .map(|k| {
                                        e.side_normal[k]
                                            * (piece.outer[e.from][k] - q.outer[f.from][k])
                                    })
                                    .sum();
                                if (dot.abs() - 1.).abs() > 1e-7 || d.abs() > 1e-7 {
                                    failures.insert(piece.id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        checks.push(Check {
            id: id.into(),
            name: name.into(),
            passed: failures.is_empty(),
            failures: failures.into_iter().collect(),
        });
    }
    Validation {
        passed: checks.iter().all(|c| c.passed),
        checks,
    }
}
