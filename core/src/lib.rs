//! Deterministic stone geometry core. All lengths are inches.
pub mod feasibility;
pub mod model;
#[cfg(feature = "python")]
mod py;
pub mod seeds;
pub mod sidecount;
pub mod surface;
pub mod tol;
pub mod topology;
use model::*;
use sha2::{Digest, Sha256};

/// Check feasibility before generating inch-valued geometry. Tiling remains unimplemented.
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
    if !p.depth.is_finite() || p.depth <= 0.0 {
        out.message = "invalid depth: expected a positive finite value in inches".into();
        return out;
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
        || p.allowed_sides.is_empty()
        || p.allowed_sides.iter().any(|k| !(4..=9).contains(k))
        || !p.target_size.is_finite()
        || p.target_size <= 0.0
        || !p.bevel_cap_deg.is_finite()
        || p.bevel_cap_deg <= 0.0
        || p.bevel_cap_deg > 90.0
        || !p.min_bounding_square.is_finite()
        || p.min_bounding_square <= 0.0
        || !p.size_variation.is_finite()
        || !(0.05..=0.50).contains(&p.size_variation)
    {
        out.message = "invalid feasibility input: expected revolve profile, sides 4..9, positive size and bevel limits, and numeric variation in 0.05..=0.50".into();
        return out;
    }
    match surface::Surface::from_profile(&input.surface.profile) {
        Err(e) => out.message = format!("invalid surface: {e}"),
        Ok(surface) => {
            out.feasibility = feasibility::check(&surface, p);
            if !out.feasibility.feasible || only {
                out.status = if out.feasibility.feasible {
                    "ok"
                } else {
                    "infeasible"
                }
                .into();
                out.message = feasibility::message(&surface, &out.feasibility, p);
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
