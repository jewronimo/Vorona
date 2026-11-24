//! Deterministic stone geometry core. All lengths are inches.
pub mod assembly;
pub mod closure;
pub mod depth;
pub mod feasibility;
pub mod flat;
pub mod model;
pub mod nest;
pub mod numbering;
pub mod pieces;
pub mod planes;
#[cfg(feature = "python")]
mod py;
pub mod rectified;
pub mod rectified_assembly;
pub mod rectified_planes;
pub mod rims;
pub mod seeds;
pub mod sidecount;
pub mod surface;
pub mod tol;
pub mod topology;
pub mod validate;
use model::*;
use sha2::{Digest, Sha256};

/// Generate inch-valued pieces with validation, flat transforms and blank nesting.
pub fn generate(input: &CoreInput) -> CoreOutput {
    run(input, false)
}

/// Run only the advisory feasibility gate; successful checks contain no pieces.
pub fn feasibility_only(input: &CoreInput) -> CoreOutput {
    run(input, true)
}

fn run(input: &CoreInput, only: bool) -> CoreOutput {
    let mut out = placeholder(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(input).expect("serializable input"))
    ));
    let p = &input.params;
    if !p.depth.is_finite() || p.depth < 0.0 || (p.pattern == Pattern::Voronoi && p.depth == 0.0) {
        out.message =
            "invalid depth: expected finite nonnegative inches (strictly positive for voronoi)"
                .into();
        return out;
    }
    for (key, valid) in [
        (
            "layout_headroom",
            p.layout_headroom.is_finite() && p.layout_headroom >= 0.,
        ),
        (
            "edge_shortfall",
            p.edge_shortfall.is_finite() && p.edge_shortfall >= 0. && p.edge_shortfall < 0.5,
        ),
        (
            "min_triangle_edge",
            p.min_triangle_edge.is_finite() && p.min_triangle_edge > 0.0,
        ),
        (
            "max_back_gap",
            p.max_back_gap.is_finite() && p.max_back_gap >= 0.0,
        ),
        (
            "relief_base",
            match p.relief_base {
                ReliefBase::Auto(_) => true,
                ReliefBase::Inches(v) => v.is_finite() && v >= 0.0,
            },
        ),
        (
            "min_corner_angle_deg",
            p.min_corner_angle_deg.is_finite()
                && p.min_corner_angle_deg > 0.0
                && p.min_corner_angle_deg < 180.0,
        ),
    ] {
        if !valid {
            out.message = format!("invalid {key}: value outside the finite contract range");
            return out;
        }
    }
    if !p.max_corner_angle_deg.is_finite()
        || p.max_corner_angle_deg <= 180.0
        || p.max_corner_angle_deg >= 270.0
    {
        out.message = "invalid max_corner_angle_deg: expected a finite angle strictly between 180 and 270 degrees".into();
        return out;
    }
    if input.schema != "vstone.core_input/1"
        || input.surface.kind != "revolve"
        || !p.thickness.is_finite()
        || p.thickness <= 0.
        || !matches!(p.rims.top.as_str(), "level" | "whole")
        || !matches!(p.rims.bottom.as_str(), "level" | "whole")
        || (p.pattern == Pattern::Voronoi
            && (p.allowed_sides.is_empty()
                || p.allowed_sides.iter().any(|k| !(4..=9).contains(k))
                || !p.target_size.is_finite()
                || p.target_size <= 0.0))
        || !p.bevel_cap_deg.is_finite()
        || p.bevel_cap_deg <= 0.0
        || p.bevel_cap_deg > 90.0
        || !p.min_bounding_square.is_finite()
        || p.min_bounding_square <= 0.0
        || (p.pattern == Pattern::Voronoi
            && (!p.size_variation.is_finite() || !(0.05..=0.50).contains(&p.size_variation)))
    {
        out.message = "invalid feasibility input: expected revolve profile, sides 4..9, positive size and bevel limits, and numeric variation in 0.05..=0.50".into();
        return out;
    }
    match surface::Surface::from_profile(&input.surface.profile) {
        Err(e) => out.message = format!("invalid surface: {e}"),
        Ok(surface) => {
            let surface = surface.for_pattern(p.pattern);
            if p.pattern == Pattern::VoronoiTriangles {
                match rectified::row_layout::choose(&surface, p) {
                    Ok(rows) => {
                        out.feasibility.feasible = true;
                        out.feasibility.columns = rows.iter().map(|r| r.count as u32).collect();
                    }
                    Err(e) => {
                        out.message = e;
                        return out;
                    }
                }
            } else {
                out.feasibility = feasibility::check(&surface, p);
            }
            if !out.feasibility.feasible || only {
                out.status = if out.feasibility.feasible {
                    "ok"
                } else {
                    "infeasible"
                }
                .into();
                out.message = if p.pattern == Pattern::VoronoiTriangles {
                    "A06 row-layout advisory only; solver/closure gates still required".into()
                } else {
                    feasibility::message(&surface, &out.feasibility, p)
                };
            } else if p.pattern == Pattern::VoronoiTriangles {
                let result = rectified_assembly::solve_assembly(&surface, p);
                match result {
                    Err(e) => out.message = e,
                    Ok(mut a) => {
                        let renames =
                            numbering::number(&mut a.pieces, &surface, 2.3 * p.min_triangle_edge);
                        for violation in &mut a.violations {
                            if let Some(id) = renames.get(&violation.0) {
                                violation.0 = id.clone();
                            }
                        }
                        for (id, _) in &mut a.per_piece_back_gap {
                            if let Some(new) = renames.get(id) {
                                *id = new.clone();
                            }
                        }
                        // Preserve diagnostics while translating temporary source ids.
                        for error in &mut a.solid_errors {
                            for (old, new) in &renames {
                                *error = error.replace(old, new);
                            }
                        }
                        out.validation = validate::triangles(&a, p);
                        let failed = out.validation.checks.iter().filter(|c| !c.passed).count();
                        out.status = "failed_validation".into();
                        out.message=format!("{} pieces; {} C6 checks failed; C7 layout/nesting pending. Back gap min/median/max {:?}; per piece {:?}",a.pieces.len(),failed,a.back_gap,a.per_piece_back_gap);
                        out.pieces = a.pieces;
                        out.stats.piece_count = out.pieces.len() as u32;
                        for piece in &out.pieces {
                            out.stats.largest_piece_width =
                                out.stats.largest_piece_width.max(piece.bounding_square);
                            *out.stats
                                .by_sides
                                .entry(if piece.partial {
                                    "partial".into()
                                } else {
                                    piece.sides.to_string()
                                })
                                .or_default() += 1;
                            out.stats.max_face_depth =
                                out.stats.max_face_depth.max(piece.face_depth);
                            for edge in &piece.edges {
                                let (lo, hi) = if edge.kind == "joint" {
                                    (
                                        &mut out.stats.shortest_joint_edge,
                                        &mut out.stats.largest_joint_edge,
                                    )
                                } else {
                                    (
                                        &mut out.stats.shortest_saw_cut_edge,
                                        &mut out.stats.largest_saw_cut_edge,
                                    )
                                };
                                if edge.kind == "joint" || edge.kind == "rim_level" {
                                    *lo = Some(lo.map_or(edge.length, |v| v.min(edge.length)));
                                    *hi = Some(hi.map_or(edge.length, |v| v.max(edge.length)));
                                }
                                out.stats.actual_shortest_side = Some(
                                    out.stats
                                        .actual_shortest_side
                                        .map_or(edge.length, |v| v.min(edge.length)),
                                );
                                if !piece.partial && p.pattern == Pattern::VoronoiTriangles {
                                    out.stats.actual_edge_shortfall = out
                                        .stats
                                        .actual_edge_shortfall
                                        .max((p.min_triangle_edge - edge.length).max(0.));
                                }
                                out.stats.actual_largest_side =
                                    out.stats.actual_largest_side.max(edge.length);
                                out.stats.max_bevel_deg =
                                    out.stats.max_bevel_deg.max(edge.bevel_deg);
                            }
                        }
                        finish_layout(&mut out, p);
                    }
                }
            } else {
                let result = sidecount::build(&surface, p).and_then(|t| {
                    let g = planes::optimize(&surface, &t, p, &mut seeds::rng(p.seed))
                        .map_err(|e| format!("C4: {}", e.message))?;
                    assembly::assemble(&t, &g, &surface, p)
                });
                match result {
                    Err(e) => {
                        out.message = e;
                    }
                    Ok(a) => {
                        out.pieces = a.pieces;
                        out.stats.piece_count = out.pieces.len() as u32;
                        for piece in &out.pieces {
                            *out.stats
                                .by_sides
                                .entry(if piece.partial {
                                    "partial".into()
                                } else {
                                    piece.sides.to_string()
                                })
                                .or_default() += 1;
                            out.stats.max_face_depth =
                                out.stats.max_face_depth.max(piece.face_depth);
                            for edge in &piece.edges {
                                out.stats.actual_shortest_side = Some(
                                    out.stats
                                        .actual_shortest_side
                                        .map_or(edge.length, |v| v.min(edge.length)),
                                );
                                if !piece.partial && p.pattern == Pattern::VoronoiTriangles {
                                    out.stats.actual_edge_shortfall = out
                                        .stats
                                        .actual_edge_shortfall
                                        .max((p.min_triangle_edge - edge.length).max(0.));
                                }
                                out.stats.actual_largest_side =
                                    out.stats.actual_largest_side.max(edge.length);
                                out.stats.max_bevel_deg =
                                    out.stats.max_bevel_deg.max(edge.bevel_deg);
                            }
                        }
                        out.status = "failed_validation".into();
                        out.message = format!(
                            "C5 pieces built at rim heights {:?}; C6 validation/numbering pending.",
                            a.rim_heights
                        );
                        out.validation.checks.push(Check {
                            id: "C5-rejections".into(),
                            name: "every removed/rejected tile".into(),
                            passed: a.rejections.is_empty(),
                            failures: a.rejections,
                        });
                        out.validation.checks.push(Check {
                            id: "C6-pending".into(),
                            name: "legacy validation and numbering pending".into(),
                            passed: false,
                            failures: vec!["not certified for manufacturing".into()],
                        });
                        finish_layout(&mut out, p);
                    }
                }
            }
        }
    }
    out
}

fn placeholder(input_sha256: String) -> CoreOutput {
    CoreOutput {
        schema: "vstone.core_output/1".into(),
        status: "error".into(),
        message: "not implemented".into(),
        feasibility: Feasibility {
            feasible: false,
            pentagons_needed: 0,
            heptagons_needed: 0,
            column_changes: 0,
            columns: vec![],
            small_radius_ranges: vec![],
            waist_ranges: vec![],
            suggestions: vec![],
        },
        pieces: vec![],
        blanks: vec![],
        validation: Validation {
            passed: false,
            checks: vec![],
        },
        stats: Stats {
            shortest_joint_edge: None,
            largest_joint_edge: None,
            shortest_saw_cut_edge: None,
            largest_saw_cut_edge: None,
            largest_piece_width: 0.,
            actual_largest_side: 0.,
            actual_shortest_side: None,
            actual_edge_shortfall: 0.,
            piece_count: 0,
            by_sides: Default::default(),
            blanks_used: 0,
            max_bevel_deg: 0.0,
            max_face_depth: 0.0,
        },
        provenance: Provenance {
            core_version: env!("CARGO_PKG_VERSION").into(),
            input_sha256,
        },
    }
}
/// Parse inch/degree contract JSON. Invalid JSON returns a structured error, never a panic.
pub fn generate_json(s: &str) -> String {
    json_run(s, false)
}

/// Parse contract JSON and return feasibility-only output in the unchanged output schema.
pub fn feasibility_only_json(s: &str) -> String {
    json_run(s, true)
}

fn json_run(s: &str, only: bool) -> String {
    let input = serde_json::from_str::<CoreInput>(s);
    let mut output = match &input {
        Ok(input) => run(input, only),
        Err(_) => placeholder(format!("{:x}", Sha256::digest(s.as_bytes()))),
    };
    if let Err(e) = input {
        output.message = format!("invalid input JSON: {e}");
        output.provenance.input_sha256 = format!("{:x}", Sha256::digest(s.as_bytes()));
    }
    serde_json::to_string(&output).expect("finite output")
}

fn finish_layout(out: &mut CoreOutput, p: &Params) {
    let result = out
        .pieces
        .iter_mut()
        .try_for_each(|piece| flat::flatten(piece, p.thickness))
        .and_then(|()| nest::pack(&mut out.pieces, p.blank_size, p.blank_usable, p.nest_gap));
    let failures = match result {
        Ok(blanks) => {
            out.blanks = blanks;
            out.stats.blanks_used = out.blanks.len() as u32;
            vec![]
        }
        Err(error) => vec![error],
    };
    out.validation.checks.push(Check {
        id: "C7-layout".into(),
        name: "flat transforms and nesting".into(),
        passed: failures.is_empty(),
        failures,
    });
    out.validation.passed = out.validation.checks.iter().all(|c| c.passed);
    out.status = if out.validation.passed {
        "ok"
    } else {
        "failed_validation"
    }
    .into();
    out.message = format!(
        "{} pieces; {} blanks; {} checks failed. {}",
        out.pieces.len(),
        out.blanks.len(),
        out.validation.checks.iter().filter(|c| !c.passed).count(),
        out.message
            .replace("C7 layout/nesting pending.", "C7 layout/nesting evaluated.")
    );
}
