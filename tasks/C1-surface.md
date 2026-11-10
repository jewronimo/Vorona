# C1 — Surface-of-revolution model

**Owner:** CORE · **Depends on:** C0, S1 · **Unblocks:** C2, C3
**Read first:** docs/GEOMETRY.md §1, SPEC R-IN-2

## Goal
A `Surface` that everything else queries: points, normals, curvature, conformal coordinates,
signed distance.

## Deliverables
`core/src/surface.rs` with:
- `Surface::from_profile(points: &[[f64; 2]]) -> Result<Surface, SurfaceError>`
  - resample to arc length (step ≤ 0.05 in), orient bottom → top
  - errors: fewer than 2 points, any `x ≤ GEOM_EPS` (touches/crosses axis), self-intersection,
    closed curve (start = end)
- `point(θ, s)`, `normal(θ, s)`, `r(s)`, `dr(s)`, `d2r(s)` (smoothed second difference),
  `gaussian_curvature(s)`, `v_of_s(s)`, `s_of_v(v)`, `length()`, `y_range()`
- `signed_distance(p: [f64;3]) -> f64` (negative inside), `closest_point(p) -> (θ, s)`
- `total_curvature(s1, s2) = −2π (r'(s2) − r'(s1))`

## Acceptance (unit tests)
- [x] Cylinder r=9, 0..60: `K = 0` everywhere (|K| < 1e-9), `normal(0, s) = (1,0,0)`,
      `v_of_s(60) = 60/9` (±1e-9), `signed_distance([10,30,0]) = 1`, `([8,30,0]) = −1`.
- [x] Cone r 9 → 6 over height 60: `K = 0` away from ends, normals tilt outward-up.
- [x] Arc profile of radius 10 from the equator up to 80° latitude: `K ≈ 0.01` (±2 %) away from
      the ends; `total_curvature(0, s)` matches numeric integration of `K · 2πr ds` (±1 %).
- [x] Hourglass profile: `K < 0` at the waist.
- [x] Each error case returns the right `SurfaceError` variant.
- [x] Profile given top → bottom gives the same surface as bottom → top.

## Out of scope
Mesh surfaces (L1).

## Agent notes

### Owner-approved follow-ups (2026-11-10)
- `closest_point` scans every segment linearly; revisit when C4 calls it often.
- Sharp profile corners give smoothed curvature; fine for v1.

### Implementation handoff (2026-11-10)
- Implemented `core/src/surface.rs` and exported the module; added tests first in
  `core/tests/surface.rs`. No contract/schema changes, dependencies, Python edits,
  commits, pushes or PRs.
- Positions, conformal integrals and closest distances retain the exact input
  polyline. Each original edge is subdivided to <=0.05 inches while preserving
  its endpoints. Conformal integration and inversion are analytic per segment.
- Differential queries use nonuniform three-point differences at original vertices
  (one-sided quadratic differences at endpoints), with normalized interpolated
  tangents and interpolated second differences. This avoids resampling-induced
  curvature impulses. Curvature is an approximation of the smooth source curve;
  coarse/sharp polylines have smoothed rather than distributional local curvature.
- Queries clamp s/v to the open profile; nonfinite s/v use the lower endpoint for
  NaN and clamp infinities. Closest-distance points and theta require finite input.
  Equal-distance ties use first segment. Signed distance is defined by the nearest
  oriented segment (no caps); a tangential rim extension is positive. This also
  defines behavior for overhanging profiles, for which global “inside” is ambiguous.
- Duplicate consecutive samples within GEOM_EPS are ignored; nonfinite samples
  and unrepresentable arc lengths return additional internal error variants.
  Equal-height endpoints use increasing radius as a deterministic orientation tie.
- Loaded the repository vorona-rust-container skill. Formatting only was run in
  Vorona-dev. Parent alone runs and certifies `make dev-check`; acceptance boxes
  remain unchecked until that result. No native Rust commands were run.

### Parent verification (2026-11-10)
- Parent ran `make dev-check` in `Vorona-dev`: exit 0. Raw log: `runs/logs/C1-0743.log`.
- Rust fmt/clippy passed; 14 Rust tests passed (1 unit, 4 contract, 9 surface); ruff passed; 3 Python tests passed.
- All six acceptance items are covered by passing surface tests. Python source and tests were not modified.
- Left uncommitted on `task/C1-surface` for owner review; no merge, push or PR.
