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

### Revised A05 implementation handoff (2026-11-11; NOT certified)

- This experiment starts at verified C3 `84ac53e`, branch `task/C4-depth-space`.
  A05 §4 supersedes the original card's on-surface/convex/tangent assumptions.
  Old `task/C4-planes` was not read or reused. No C3 algorithm changes.
- `planes.rs`: two tilt parameters per seed; fitted circumcenter/mean-depth
  target initialization; exact three-plane intersections and through-seed offsets.
  Surface-specific chart continuation is implemented in `surface.rs`; solver
  uses only the generic `SurfaceQueries` trait. No production Python edits.
- Deterministic finite-difference sparse Jacobian, matrix-free damped GN normal
  equations (80 CG steps maximum), 80 outer steps, bounded 0.25 tilt increments;
  barrier weight 0.001 doubled every five iterations. Weak target pull, edge
  regularity, tilt and adjacent-tilt regularity remain throughout.
- Barriers/report: edge min/max, signed convex/reflex angle bounds, determinant,
  bevel, simple outer/inner polygons, inner orientation, minimum outer enclosing
  square and usable enclosing square of projected outer+inner footprints.
  Minimum square enumerates support-change intervals and analytic stationary/
  equal-width-height candidates (not an angular sampling approximation).
- OptError retains best full geometry plus OptReport, ranked by violated-term
  count then normalized objective. Signed corner min/max/median are reported,
  never constrained to the surface. Best iteration is reported (not elapsed
  outer iteration count). No new serialized production CoreOutput fields:
  OptReport/Planes are internal diagnostic structures; A05 input already exists.
- IMPORTANT omissions/limitations for owner review: open hull fans have no
  polygon constraints (they are auxiliary C5 boundary work). Complete polygons
  near physical rims are optimized before C5 clipping; no claim of final clipped
  partial-piece compliance. Segment crossing guard detects proper crossings;
  collinear overlapping/touching edges require C6 robustness work. This is a
  bounded experiment, not a convergence claim or completed acceptance gate.
- Parent must compile/check and measure. Subagent ran ONLY container cargo fmt,
  no tests, clippy, build, timings, commits, merge or push. No best measured
  OptReport exists yet; record parent measurements here and stop for Jerry if
  limits remain violated, per A05. Do not mark acceptance complete from code.

#### Parent commands (inside Vorona-dev, workdir /src/runs/worktrees/C4-depth/core)

```
cargo test --test planes
cargo test --release --test planes -- --ignored --nocapture
cargo run --release --example c4_report -- ../tests/fixtures/core_input_cylinder.json 0 42 2
```

- `symmetric_seed_plane_band_and_deterministic_planes`: identical planes/report,
  every seed depth in band and on its plane within 1e-9; finite signed statistics.
- `enclosing_square_rotates_and_handles_reflex_points`: analytic size primitive.
- Ignored strict experiments: `symmetric_cylinder_all_limits`,
  `organic_half_and_full_seeds_one_to_ten`, `cone_and_vase_all_limits`.
  They ASSERT all limits; failure is not silently converted into passing tests.
- `shallow_and_deep_clear_reports`: depth 1/3 report experiment.
- c4_report takes JSON input plus optional randomness/seed/depth overrides;
  emits full Result (including best geometry/report on failure), optimizer wall
  seconds excluding topology preparation. Parent should pin one CPU via
  `taskset -c <allowed CPU>` and record release timing, then run randomness
  0.5/1, seeds 1..10 and depth 1/3. Full input JSON can supply cone/vase settings.

### Parent measured verification — 2026-11-11

- Official `make dev-check` using a temporary Makefile recipe selecting this worktree: PASS (fmt, clippy, non-ignored Rust tests, ruff, 49 Python tests). Raw log: `runs/logs/C4-depth-verify.log`. Four strict/research plane tests remain ignored in this standard check.
- Parent corrected cone/vase acceptance fixture names to existing `cone_9_to_6.json` and `vase_bulge.json`; no geometry change.
- Release diagnostic pinned with `taskset -c 0`: cylinder 18×60, [6], randomness 0, seed 42, depth 2, all other fixture parameters unchanged (notably bevel cap 60 degrees). Optimizer time: 15.03289434 s. This meets timing only, NOT geometric acceptance.
- Best report: iteration 42; signed corner distances min/max/median {'max': 6.009897189486033, 'median': -0.5372374353201854, 'min': -4.544585807785525} inches; reflex corners 75. Violations: {'max_corner_angle_deg': 42, 'max_edge': 140, 'min_corner_angle_deg': 49, 'min_edge': 32, 'simple_inner_polygon': 10, 'simple_polygon': 10, 'blank_usable': 1}.
- Full exact best geometry and OptReport: `runs/logs/C4-depth-cylinder-r0-s42-d2.json`. Explicit strict assertion log: `runs/logs/C4-depth-strict-symmetric.log`.
- Tried fitted target initialization and bounded damped Gauss–Newton with scheduled barriers, as described above. Failed the first baseline even with fixture bevel cap 60 degrees; stricter cap cannot turn this recorded geometry into an accepted result. No limits relaxed.
- STOP for Jerry per A05 §4: "Record the best OptReport and what was tried ... then stop. Jerry decides." Randomness 0.5/1 seed matrix, depths 1/3, cone and vase remain UNMEASURED; no acceptance claimed. C4 is an unmerged experimental checkpoint, not done; C5 is blocked.
