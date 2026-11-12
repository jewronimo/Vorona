//! Ordered JSON contract. Lengths are inches; angles are degrees.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CoreInput {
    pub schema: String,
    pub surface: Surface,
    pub params: Params,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub kind: String,
    pub profile: Vec<[f64; 2]>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Params {
    pub thickness: f64,
    pub allowed_sides: Vec<u32>,
    pub target_size: f64,
    pub size_variation: f64,
    pub randomness: f64,
    pub seed: u64,
    pub ends: Ends,
    pub rims: Rims,
    pub min_edge: f64,
    pub max_edge: f64,
    pub min_corner_angle_deg: f64,
    pub bevel_cap_deg: f64,
    #[serde(default = "default_depth")]
    pub depth: f64,
    #[serde(default = "default_max_corner_angle")]
    pub max_corner_angle_deg: f64,
    pub min_bounding_square: f64,
    pub allow_waists: bool,
    pub blank_size: f64,
    pub blank_usable: f64,
    pub nest_gap: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Ends {
    pub top: String,
    pub bottom: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rims {
    pub top: String,
    pub bottom: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CoreOutput {
    pub schema: String,
    pub status: String,
    pub message: String,
    pub feasibility: Feasibility,
    pub pieces: Vec<Piece>,
    pub blanks: Vec<Blank>,
    pub validation: Validation,
    pub stats: Stats,
    pub provenance: Provenance,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Feasibility {
    pub feasible: bool,
    pub pentagons_needed: u32,
    pub heptagons_needed: u32,
    pub column_changes: u32,
    pub columns: Vec<u32>,
    pub small_radius_ranges: Vec<[f64; 2]>,
    pub waist_ranges: Vec<[f64; 2]>,
    pub suggestions: Vec<Vec<u32>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub id: String,
    pub sides: u32,
    pub partial: bool,
    pub plane: Plane,
    pub outer: Vec<[f64; 3]>,
    pub inner: Vec<[f64; 3]>,
    pub edges: Vec<Edge>,
    pub corner_angles_deg: Vec<f64>,
    pub face_depth: f64,
    pub bounding_square: f64,
    pub flat_xform: [[f64; 4]; 4],
    pub blank: u32,
    pub blank_xform: [[f64; 4]; 4],
    pub map_uv: Vec<[f64; 2]>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Plane {
    pub n: [f64; 3],
    pub d: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub neighbor: Option<String>,
    pub kind: String,
    pub length: f64,
    pub bevel_deg: f64,
    pub side_normal: [f64; 3],
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Blank {
    pub index: u32,
    pub pieces: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Validation {
    pub passed: bool,
    pub checks: Vec<Check>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub failures: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    pub piece_count: u32,
    pub by_sides: BTreeMap<String, u32>,
    pub blanks_used: u32,
    pub max_bevel_deg: f64,
    pub max_face_depth: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub core_version: String,
    pub input_sha256: String,
}

fn default_depth() -> f64 {
    2.0
}
fn default_max_corner_angle() -> f64 {
    225.0
}
