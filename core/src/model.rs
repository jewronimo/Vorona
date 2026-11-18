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
/// Selected topology; omission selects the A06 pattern.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Pattern {
    #[default]
    VoronoiTriangles,
    Voronoi,
}
/// Relief base in inches, transported unchanged until geometry resolves auto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ReliefBase {
    Auto(AutoRelief),
    Inches(f64),
}
/// The only accepted symbolic relief-base value.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum AutoRelief {
    #[default]
    #[serde(rename = "auto")]
    Auto,
}
impl Default for ReliefBase {
    fn default() -> Self {
        Self::Auto(AutoRelief::Auto)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "ParamsWire")]
pub struct Params {
    pub pattern: Pattern,
    pub min_triangle_edge: f64,
    #[serde(default = "default_max_polygon_edge")]
    pub max_polygon_edge: f64,
    #[serde(default = "default_count_step")]
    pub count_step: usize,
    pub relief_base: ReliefBase,
    pub max_back_gap: f64,
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
    pub depth: f64,
    pub max_corner_angle_deg: f64,
    pub min_bounding_square: f64,
    pub allow_waists: bool,
    pub blank_size: f64,
    pub blank_usable: f64,
    pub nest_gap: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsWire {
    #[serde(default, deserialize_with = "read_pattern")]
    pub pattern: Pattern,
    #[serde(
        default = "default_min_triangle_edge",
        deserialize_with = "read_min_triangle_edge"
    )]
    pub min_triangle_edge: f64,
    #[serde(default = "default_max_polygon_edge")]
    pub max_polygon_edge: f64,
    #[serde(default = "default_count_step")]
    pub count_step: usize,
    #[serde(default, deserialize_with = "read_relief_base")]
    pub relief_base: ReliefBase,
    #[serde(
        default = "default_max_back_gap",
        deserialize_with = "read_max_back_gap"
    )]
    pub max_back_gap: f64,
    pub thickness: f64,
    pub allowed_sides: Vec<u32>,
    pub target_size: f64,
    pub size_variation: f64,
    #[serde(default)]
    pub randomness: Option<f64>,
    pub seed: u64,
    pub ends: Ends,
    pub rims: Rims,
    pub min_edge: f64,
    pub max_edge: f64,
    #[serde(default, deserialize_with = "read_min_angle")]
    pub min_corner_angle_deg: Option<f64>,
    pub bevel_cap_deg: f64,
    #[serde(default, deserialize_with = "read_depth")]
    pub depth: Option<f64>,
    #[serde(default = "default_max_corner_angle")]
    pub max_corner_angle_deg: f64,
    pub min_bounding_square: f64,
    pub allow_waists: bool,
    pub blank_size: f64,
    pub blank_usable: f64,
    pub nest_gap: f64,
}
impl From<ParamsWire> for Params {
    fn from(w: ParamsWire) -> Self {
        Self {
            pattern: w.pattern,
            min_triangle_edge: w.min_triangle_edge,
            max_polygon_edge: w.max_polygon_edge,
            count_step: w.count_step,
            relief_base: w.relief_base,
            max_back_gap: w.max_back_gap,
            thickness: w.thickness,
            allowed_sides: w.allowed_sides,
            target_size: w.target_size,
            size_variation: w.size_variation,
            randomness: w.randomness.unwrap_or(0.0),
            seed: w.seed,
            ends: w.ends,
            rims: w.rims,
            min_edge: w.min_edge,
            max_edge: w.max_edge,
            min_corner_angle_deg: w.min_corner_angle_deg.unwrap_or(match w.pattern {
                Pattern::Voronoi => 70.0,
                Pattern::VoronoiTriangles => 30.0,
            }),
            bevel_cap_deg: w.bevel_cap_deg,
            depth: w.depth.unwrap_or(match w.pattern {
                Pattern::Voronoi => 1.5,
                Pattern::VoronoiTriangles => 0.25,
            }),
            max_corner_angle_deg: w.max_corner_angle_deg,
            min_bounding_square: w.min_bounding_square,
            allow_waists: w.allow_waists,
            blank_size: w.blank_size,
            blank_usable: w.blank_usable,
            nest_gap: w.nest_gap,
        }
    }
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
    /// A05 inward seed depth in inches; absent only in pre-A05 fixtures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_depth: Option<f64>,
    /// A05 maximum signed outer-corner distance in inches, positive outside.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_corner_distance: Option<f64>,
    /// A05 number of reflex corners in the outer polygon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reflex_corners: Option<usize>,
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

fn default_min_triangle_edge() -> f64 {
    1.5
}
fn default_max_back_gap() -> f64 {
    0.5
}
fn default_max_corner_angle() -> f64 {
    230.0
}

fn read_pattern<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Pattern, D::Error> {
    Pattern::deserialize(d).map_err(|e| serde::de::Error::custom(format!("pattern: {e}")))
}

fn read_relief_base<'de, D: serde::Deserializer<'de>>(d: D) -> Result<ReliefBase, D::Error> {
    ReliefBase::deserialize(d).map_err(|e| serde::de::Error::custom(format!("relief_base: {e}")))
}

fn read_depth<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    f64::deserialize(d)
        .map(Some)
        .map_err(|e| serde::de::Error::custom(format!("depth: {e}")))
}

fn read_min_angle<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    f64::deserialize(d)
        .map(Some)
        .map_err(|e| serde::de::Error::custom(format!("min_corner_angle_deg: {e}")))
}

fn read_min_triangle_edge<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    f64::deserialize(d).map_err(|e| serde::de::Error::custom(format!("min_triangle_edge: {e}")))
}

fn read_max_back_gap<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    f64::deserialize(d).map_err(|e| serde::de::Error::custom(format!("max_back_gap: {e}")))
}

fn default_max_polygon_edge() -> f64 {
    3.0
}
fn default_count_step() -> usize {
    2
}
