//! Deterministic stone geometry core. All lengths are inches.
pub mod model;
#[cfg(feature = "python")]
mod py;
pub mod tol;
use model::*;
use sha2::{Digest, Sha256};

/// Return the C0 error placeholder; input lengths are inches. Geometry is not implemented.
pub fn generate(input: &CoreInput) -> CoreOutput {
    placeholder(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(input).expect("serializable input"))
    ))
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
    let input = serde_json::from_str::<CoreInput>(s);
    let mut output = match &input {
        Ok(input) => generate(input),
        Err(_) => placeholder(format!("{:x}", Sha256::digest(s.as_bytes()))),
    };
    if let Err(e) = input {
        output.message = format!("invalid input JSON: {e}");
        output.provenance.input_sha256 = format!("{:x}", Sha256::digest(s.as_bytes()));
    }
    serde_json::to_string(&output).expect("finite output")
}
