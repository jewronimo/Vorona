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
- [ ] Cylinder r=9, 0..60: `K = 0` everywhere (|K| < 1e-9), `normal(0, s) = (1,0,0)`,
      `v_of_s(60) = 60/9` (±1e-9), `signed_distance([10,30,0]) = 1`, `([8,30,0]) = −1`.
- [ ] Cone r 9 → 6 over height 60: `K = 0` away from ends, normals tilt outward-up.
- [ ] Arc profile of radius 10 from the equator up to 80° latitude: `K ≈ 0.01` (±2 %) away from
      the ends; `total_curvature(0, s)` matches numeric integration of `K · 2πr ds` (±1 %).
- [ ] Hourglass profile: `K < 0` at the waist.
- [ ] Each error case returns the right `SurfaceError` variant.
- [ ] Profile given top → bottom gives the same surface as bottom → top.

## Out of scope
Mesh surfaces (L1).

## Agent notes
