# C4 — Flat tile optimizer

**Owner:** CORE · **Depends on:** C3 · **Unblocks:** C5
**Read first:** docs/GEOMETRY.md §4 (including the "important known result"), SPEC R-TL-3, R-TL-8..11, R-ST-3

This is the hardest card. Keep it isolated; report progress in `## Agent notes` even if partial.

## Goal
Given the topology, find one plane per tile so that every corner (intersection of 3 planes)
lies on the surface and the limits hold.

## Deliverables
- `core/src/planes.rs` — `optimize(surface, topology, params, rng) -> Result<Planes, OptError>`.
- Energy and solver as in GEOMETRY.md §4 (or better, documented). Fixed iteration cap,
  fixed convergence thresholds, deterministic term order.
- `OptReport { iterations, max_corner_dist, max_face_depth, violated: Vec<(tile, limit)> }`.

## Acceptance
- [ ] Cylinder `[6]` randomness 0: max corner distance to surface ≤ `CORNER_ON_SURFACE_TOL`;
      all tiles convex; corner angles ≥ min; edges within [min_edge, max_edge]; bevels ≤ 25°.
- [ ] Same, randomness 0.5 and seeds 1..10: same limits hold.
- [ ] Cone 9 → 6 and vase bulge with `[5,6,7]`: same limits hold.
- [ ] Every corner's 3×3 system has `|det| ≥ 1e-8`.
- [ ] Face depth ≤ `surface_tolerance` for every tile.
- [ ] Runtime for the cylinder ≤ 60 s in release build on one core (report the time).
- [ ] Deterministic: identical planes on repeat runs.

## If it does not converge
Write what was tried and the best `OptReport` in `## Agent notes`, then stop. Jerry decides
whether to loosen limits or try the Rhino-trial experiment (PLAN.md risks).

## Agent notes
