# C2 — Feasibility check

**Owner:** CORE · **Depends on:** C1 · **Unblocks:** C3, G0
**Read first:** docs/GEOMETRY.md §3, SPEC R-FE-1..3, R-TL-12

## Goal
Before any tiling, decide whether the allowed side set can work, explain why not, and suggest sets.

## Deliverables
- `core/src/feasibility.rs`: `check(surface, params) -> Feasibility` (struct from C0).
- `generate()` runs it first; if infeasible returns `status = "infeasible"` with the message.
- `vstone-core --feasibility-only <input.json>` flag for G0.
- Test fixtures in `tests/fixtures/profiles/`: `cylinder.json`, `cone_9_to_6.json`,
  `vase_bulge.json` (r: 6 → 10 → 6 over 40 in, smooth), `hourglass.json` (r: 9 → 5 → 9).

## Acceptance
- [x] Cylinder, `[6]`, tight: feasible, `columns` contains one even count between 8 and 12, no suggestions.
- [x] Cone 9 → 6, `[6]`, tight: infeasible (column changes > 0); suggestions start with `[5,6,7]`;
      with `wide` variation: feasible.
- [x] Vase bulge, `[6]`: infeasible, `pentagons_needed ≥ 1`, first suggestion contains 5.
- [x] Hourglass, any set, `allow_waists = false`: infeasible, `waist_ranges` non-empty, message names heights.
- [x] Cylinder radius 1.5, `bevel_cap 60`, `min_bounding_square 4`: `small_radius_ranges` covers it.
- [x] `[5]` only on cylinder: infeasible with a suggestion containing 6 or both 5 and 7.
- [x] Messages follow the format in GEOMETRY.md §3 (snapshot tests on the exact strings).
- [x] Same input twice → identical output.

## Agent notes

### Implementation handoff (2026-11-10)
- Implemented estimator, generate-first gate, feasibility-only CLI, four profile
  fixtures, and eight acceptance-focused integration tests. No new dependencies,
  contract/schema changes, Python edits, commits or branch changes.
- Loaded vorona-rust-container skill; formatting only in Vorona-dev. Parent must
  run/certify make dev-check; acceptance boxes remain unchecked pending that run.
- Even column wraps (no helical seam); interval projection/backtracking minimizes
  total absolute column changes on a fixed <=0.05-inch arc grid. `columns` is the
  compressed sequence, not per-height samples. Changes count individual columns,
  so an even-count transition of 2 requires two 5/7 pairs.
- Curvature defect estimates integrate positive/negative changes of the existing
  smoothed surface tangent. Expected interior count uses area / target_size²;
  this is an advisory estimate, not an optimizer convergence guarantee.
- Waist and small-radius ranges merge contiguous sampled arc cells, reporting
  heights in inches; threshold crossings are resolved to <=0.05 inches of arc,
  not exact roots. All positive-curvature and negative-curvature diagnostics are
  retained even if a side set can support them. Waists are hard blockers unless
  explicitly allowed; alternative side sets cannot fix radius/waist blockers.
- Plain messages follow the documented `N only is not possible: ... Try [...]`
  format; the exact geometry-reference example is snapshot-tested. Up to four
  current-variation feasible supersets are ranked by additions, distance from 6,
  then lexical set order. Input sets are never silently changed.
- Frozen contract cannot identify wide-only suggestions: recorded request
  docs/requests/001-feasibility-wide-suggestions.md; that part is stopped pending
  owner decision. Explicit wide checks work. No misleading bare wide-only sets
  are returned. This is the sole deferred deliverable detail.
- Feasibility-only succeeds with status ok and empty geometry; normal generate
  returns infeasible immediately for failed checks, and retains not-implemented
  status error for feasible profiles until C3+ implement tiling. No validation
  success is claimed for the empty feasibility-only output.

### Parent verification (2026-11-10)
- `make dev-check` exited 0 in `Vorona-dev`; saved output:
  `runs/logs/C2-0801.log`. Rust fmt/clippy passed, 22 Rust tests passed,
  Python ruff passed and 3 Python tests passed. No Python files modified.
- Acceptance mapping (all eight tests reported `ok` in that log):
  1. Cylinder: `cylinder_even_wrap`.
  2. Cone tight/wide and suggestions: `cone_dislocations_and_wide`.
  3. Bulge: `bulge_requires_positive_defects`.
  4. All 63 side sets and waist heights: `waist_blocks_all_sets_and_names_heights`.
  5. Small radius: `small_radius_covers_full_height`.
  6. Pentagon-only rejection: `pentagons_only_cannot_balance_cylinder`.
  7. Exact messages: `exact_message_snapshots`.
  8. Determinism, generation gate and CLI: `generation_gate_cli_and_determinism`.
- Eight listed acceptance items pass; full GEOMETRY wide-only suggestion
  recommendations remain deferred as documented above. No contract changes.
- Left uncommitted on `task/C2-feasibility` for owner review. Main unchanged.

### A04 implementation handoff (2026-11-10)
- Owner approved numeric variation; wrote A04 before implementation. Core accepts
  fractions 0.05–0.50 only; both schemas, input fixtures and ring provenance hash
  updated. No Python edits. SHELL must separately migrate its config/mirror.
- Request001 resolved: side sets remain current-variation only. Messages show
  the deterministic minimum working 5%-grid variation for the unchanged set.
  Cone target_size=6 needs 21.46% before rounding, therefore recommends ±25%.
- Added cone_numeric_variation_message_snapshot (exact message and analytic
  threshold probes) and numeric_variation_contract_and_range (schema/runtime
  numeric bounds, invalid strings/percent values/null/missing, schema parity).
  Existing eight C2 tests remain the acceptance mapping listed above, now numeric.
- Formatting only in Vorona-dev; NO checks run for A04. Parent must run
  make dev-check; the previous verification does not certify this amendment.
  No commits, branch switches, merges, pushes, C3 work or new dependencies.

### Parent A04 verification (2026-11-10)
- `make dev-check` exited 0 in Vorona-dev; output: `runs/logs/C2-A04-0950.log`.
- All eight acceptance tests above, the cone numeric message snapshot and numeric
  contract/schema range test reported `ok`. Rust fmt/clippy and Python ruff/pytest passed.
- Request001 is resolved by A04; earlier pending/deferred notes are historical.
- No Python files modified; Python percent conversion remains separate.

### Surface-dip implementation handoff (2026-11-10, unverified)
- Isolated branch `task/C2-surface-dip` from main, worktree `runs/worktrees/C2-dip`.
  C4 worktree, main checkout's dirty S3 files, and S3 artifacts untouched.
- Each existing even-column/size-variation interval is intersected with
  `r*(1-cos(pi/n)) <= surface_tolerance`. Deterministic integer bisection tests
  the inclusive inequality directly; existing minimum-change projection and
  low-count ties are retained. Reference cylinder selects 10 instead of 8.
- Empty intervals make feasibility false and leave columns empty (no fictitious
  usable sequence). Existing output message reports the dip inequality, tolerance
  in inches and sampled blocked heights; no schema/contract/Python changes.
  Height ranges retain the estimator's <=0.05-inch arc sampling resolution.
- A04 suggestions and first-working 5%-grid variation use the same constrained
  estimator, so no side-set suggestion bypasses dip and variation advice remains
  the smallest working rounded grid value. This is a C2 screening constraint,
  not proof that optimized C4 faces satisfy their measured face-depth limit.
- Added tests: `reference_cylinder_dip_selects_ten` (actual frozen input),
  `dip_boundary_is_inclusive_and_respects_even_counts` (equal/above/below),
  `no_dip_compatible_count_reports_limit_without_side_suggestions` (message,
  both generation gates, no misleading suggestions/advice), and
  `dip_variation_advice_uses_first_working_grid_point` (10% fails,15% works).
  Legacy C2 defect/variation tests and C3 curved-profile topology tests explicitly
  use a loose 2-inch dip tolerance to
  preserve their isolated analytic thresholds and message snapshots; actual
  reference parameters are independently tested without overrides.
- NO tests, checks or diagnostic programs executed per task instruction; parent
  must run official dev-check. No commit, merge, push, rebase, or new dependency.
- After parent verifies C2: preserve C4 dirty changes as a local reviewable commit
  or stash, commit approved C2, rebase task/C4-planes onto task/C2-surface-dip,
  restore preserved changes, then investigate tile10's measured 1.97-inch face
  depth against ~0.69-inch geometric expectation. Trace polygon/plane geometry,
  centroid and midpoint probes, signed-distance calculation, and exported units/
  piece identity; do not treat this discrepancy as explained by cylinder sagitta.
- Container `cargo fmt` completed; this is formatting only, not verification.

### Revised surface-dip handoff (2026-11-10, unverified)
- Rebased `task/C2-surface-dip` onto main `59aecb1`; prior dip commit is now
  `379d848`. Owner revised default to 0.75 inches; parent owns config/example/docs
  defaults concurrently. No parent-owned files changed here.
- Inclusive per-run sagitta inequality and empty-interval infeasibility are
  unchanged. Reference cylinder regression now independently checks tolerance
  0.75 selects 8 columns and 0.50 selects 10 columns; no other acceptance relaxed.
- Feasibility messages now include local size-range candidate even counts at the
  bottom profile height, circumference/n piece width, sagitta dip and explicit
  within/exceeds-tolerance labels. Reference: 8 -> ~7.1 in / 0.69 in dip;
  10 -> ~5.7 in / 0.44 in dip. At most 16 candidates are listed with an explicit
  omission marker for longer intervals. Local estimates do not certify C4 depth.
- Updated exact-message snapshots and reference diagnostics assertions; retained
  no-compatible-count, inclusive-boundary and variation-grid regressions.
- Loaded vorona-rust-container skill; `cargo fmt` only in Vorona-dev. NO checks,
  tests, diagnostic binaries or certification run. Parent must verify.
- C4 and its dirty files untouched; no merges or pushes. Historical 1.97-inch
  C4 value above was global, not established as tile10. Latest parent evidence:
  release max 1.1548 on tile58 versus tile10 0.0971 after ordering fix.
