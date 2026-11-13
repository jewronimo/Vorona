# C3 — Seeds, topology, side-count control

**Owner:** CORE · **Depends on:** C2 · **Unblocks:** C4
**Read first:** docs/GEOMETRY.md §2, SPEC R-TL-1,2,4,5,6,7, R-RM-1,2

## Goal
Place seeds deterministically and build a tile adjacency (Delaunay in conformal coordinates)
where every interior tile's side count is in the allowed set.

## Deliverables
- `core/src/seeds.rs` — seeding from `seed`, `randomness`, `target_size`, `size_variation`,
  including the extra rim row for `level` rims.
- `core/src/topology.rs` — periodic Delaunay (crate `spade`), `Topology { tiles, neighbors, corners }`
  with corners as sorted index triples; degree per tile; boundary flags.
- `core/src/sidecount.rs` — deterministic repair loop: edge flips, seed insert/delete, small moves,
  until all interior degrees ∈ allowed set or a fixed cap is hit (then return an error naming tiles).

## Acceptance
- [ ] Cylinder, `[6]`, randomness 0: every interior tile degree 6, column count = the one C2 chose,
      lattice regular (seed spacing in u within 1e-9 of 2π/n).
- [ ] Cylinder, `[5,6,7]`, randomness 1, seeds 1..20: all interior degrees ∈ {5,6,7}; 3-valent corners;
      each interior Delaunay edge in exactly two triangles.
- [ ] Cone 9 → 6, `[5,6,7]`: number of 5s and 7s ≥ C2's prediction, placed where columns change.
- [ ] Vase bulge, `[5,6,7]`: pentagons count within ±1 of C2's `pentagons_needed`.
- [ ] Same seed twice → identical topology; different seed → different topology.
- [ ] No tile's seed spacing outside the size-variation band (measured on the surface, not in u,v).

## Agent notes

### Implementation handoff (2026-11-10; not certified)
- Added internal Rust modules seeds/topology/sidecount and integration tests.
  No JSON contract, Python, original checkout, branch or commit changes.
- Uses one ChaCha8Rng for perturbations and deterministic candidate repair.
  Added card-listed dependencies rand/rand_chacha/spade; Cargo.lock must be
  resolved by the parent in Vorona-dev. No host Rust commands or checks run.
- Periodic quotient uses three chart copies, central-centroid triangles,
  sorted triple deduplication, ordered adjacency and hull-edge boundary flags.
- Repair retriangulates small moves (legal geometric edge flips), then tries
  insert/delete near degree violations. Fixed 64-pass budget; cannot certify
  convergence for arbitrary allowed sets. Errors name invalid tiles/degrees
  and the unresolved defect budget. No silent acceptance of cap failures.
- Surface spacing means nearest-neighbor straight conformal-chart path length,
  integrated with 64 fixed midpoint panels in the surface metric, not chord
  distance. It is not an exact geodesic solver. Checks exclude auxiliary/hull
  sites, not ordinary interior sites. This interpretation needs owner review
  if the card intended every Delaunay edge rather than nearest-site spacing.
- Auxiliary level-rim rows use a cylindrical endpoint chart extension. Actual
  clipped partial bounding squares cannot be guaranteed until C4/C5 determine
  the polygons; no claim of R-RM-2 certification is made here.
- Curvature/column quotas are repair objectives and success conditions. Cone
  tests count pairs; bulge tests use ±1 around C2's positive defect estimate.
  The locations of cone defects relative to column transitions still need a
  stronger acceptance test and algorithm review. Nonzero randomness changes
  site positions, but need not change the adjacency graph for every seed.
- Acceptance mapping: regular cylinder -> regular_cylinder_matches_c2_columns;
  random cylinders/3-valence/manifold -> twenty_organic_cylinders_are_allowed_and_manifold;
  cone and bulge counts -> cone_and_bulge_defect_budgets;
  determinism -> seeded_topology_is_byte_deterministic;
  surface spacing -> cylinder_nearest_spacing_uses_surface_metric;
  seam/invalid inputs -> periodic_seam_and_bad_sites.
- All acceptance boxes intentionally remain unchecked: parent must format,
  resolve lockfile and run make dev-check. In particular curved-profile repair
  convergence has not been demonstrated and should not be represented as done.

### Parent verification (2026-11-10)
- Parent ran `make -f Makefile -f /tmp/vorona-c3-check.mk dev-check`, exit 0,
  in `Vorona-dev`. Supplemental recipe changes only docker exec working directory
  to `/src/runs/worktrees/C3`; same pip install + make check as official recipe.
- Evidence: original checkout `runs/logs/C3-verify-1003.log`. Rust fmt/clippy/tests,
  all six topology tests, ruff and three baseline Python tests passed. This branch
  intentionally does not contain S2's 36 Python tests. Cargo.lock resolved and
  Rust formatted in container. No commit or merge performed.
- Acceptance audit: regular cylinder and 20 organic cylinder manifold cases PASS;
  cone counts PASS but transition-location assertion MISSING; bulge count PASS;
  same-seed byte determinism/different-seed serialized positions PASS, adjacency
  difference not asserted; spacing PASS on cylinder interior nearest neighbors
  only, not full no-tile/curved-profile acceptance. C3 remains NOT DONE.
- Follow-up delegation blocked in current runtime: no attributed session-send or
  spawn tool exposed. `openclaw agent` continuation rejected by gateway because
  it would lose inter-session attribution; no alternate CLI route attempted.
  Resume vorona-core with these review gaps when attributed routing is available.

### Fresh strict repair handoff (2026-11-10; NOT CERTIFIED)
- Read AGENTS, PLAN, A01–A04, C1/C2 dependency cards and all historical C3
  notes. Applied vorona-rust-container skill. Worked only in this worktree;
  preserved the preexisting Cargo/lib/topology implementation. No Python,
  contract edits, commits, merges, branch changes, main edits, or checks.
- Replaced nearest-neighbor acceptance with EVERY edge whose two endpoints are
  ordinary interior sites. Physical rim rows and auxiliary rows are excluded;
  no additional near-rim row exemption. Surface lengths still use the existing
  fixed 64-panel conformal-path metric (not exact geodesics).
- Added a continuous squared band residual, simultaneous spring proposals,
  rotating invalid-site sweep, and retained legal retriangulated moves and
  insertion/deletion proposals. Fixed cap is 128 passes. Success requires strict
  spacing, degree and defect conditions; failure names tile degrees, initial and
  final residual, accepted steps, and minimum/maximum edge-to-target ratios.
- Added internal RepairReport (not contract JSON) with initial/final residual and
  objective, iterations and accepted steps. Added strict cone/vase convergence
  tests with deliberately perturbed initial sites, not just final count checks.
- Organic perturbations now cross flip thresholds instead of tiny serialized
  position jitter; tests REQUIRE pairwise different neighbor lists for seeds
  1–5 at randomness 0.5 and 1.0, and also require strict edge spacing. Whether
  bounded repair preserves this diversity is unverified, not asserted as fact.
- Cone objective penalizes defects outside one row of column-count transitions
  and lacking adjacent complementary 5/7. Test independently derives transitions
  from the original C2-seeded rows. Insert/delete can alter the objective's row
  counts; the independent original-row test intentionally detects discrepancies.
- Acceptance mapping: regular cylinder -> regular_cylinder_matches_c2_columns;
  20 random cylinders/manifold -> twenty_organic_cylinders_are_allowed_and_manifold;
  strict cylinder/cone/vase band -> every_interior_edge_on_cylinder_cone_and_vase_is_in_band;
  actual adjacency diversity -> seeds_one_to_five_change_adjacency_not_just_positions;
  deterministic hash -> seeded_topology_is_byte_deterministic;
  cone transition/pairing + vase count -> cone_and_bulge_defect_budgets;
  measured repair -> cone_and_vase_repair_reduce_nonzero_residual.
- Parent signaled Vorona-dev ready after image rebuild; ran container cargo fmt
  only. Parent alone runs official make dev-check and certifies. NO convergence
  or passing-test claim is made for this stricter revision, and no mathematical
  impossibility is claimed without measured evidence. Raw cap diagnostics are
  designed to expose the worst ratios if the strict tests fail.
- R-RM-2 partial bounding-square guarantee remains deferred to C4/C5 polygon
  geometry, as in the prior handoff; this revision does not claim to solve it.


### Parent strict verification (2026-11-10) — FAILED
- Ran `make -f Makefile -f /tmp/vorona-c3-check.mk dev-check` in Vorona-dev, exit 2. Log in main checkout: `runs/logs/C3-strict-1021.log`; exact last 40 lines saved as `runs/logs/C3-strict-1021-last40.log`. Supplemental recipe only selects the C3 worktree.
- fmt/clippy and baseline Rust tests passed. Topology: 2 passed, 6 failed (192.01 seconds). Python checks were not reached.
- regular_cylinder_matches_c2_columns and periodic_seam_and_bad_sites PASS.
- every_interior_edge_on_cylinder_cone_and_vase_is_in_band FAIL on cone after cylinder; cone residual 0.05817747811728 -> 3.145951164380e-18, min/max ratios 0.849999999/1.150000001, unresolved defect budget 4. Vase not reached.
- cone_and_bulge_defect_budgets FAIL before transition assertions; vase not reached.
- cone_and_vase_repair_reduce_nonzero_residual FAIL on cone: 0.2163087664653 -> 2.557168735717e-18, 48 accepted steps, unresolved defect budget 4. Reduction is NOT convergence. Vase not reached.
- seeds_one_to_five_change_adjacency_not_just_positions FAIL before comparing all graphs; min/max 0.849999790/1.150000300. Adjacency diversity unverified.
- seeded_topology_is_byte_deterministic FAIL while building topology; min/max 0.849536159/1.150308247.
- twenty_organic_cylinders_are_allowed_and_manifold FAIL: min/max 0.836796445/1.164137944 (worst observed strict-run ratios); degree 8 remains.
- No tests loosened, no impossibility claim, no commit or merge. Main remains e3f29f7.
- Further CORE repairs require attributed specialist routing. This resumed runtime exposes neither sessions_spawn nor sessions_send (checked tool registry); shell CLI bypass is not an attributed substitute. Existing fresh specialist settled. Preserve this evidence for next attributed specialist continuation.

### Round 3 — owner spacing revision and constructive dislocations (2026-11-10)
- Preserved all inherited dirty work; no branch changes, commits, Python or
  contract changes. Applied vorona-rust-container instructions. Machine proof:
  Darwin Spookys-MacBook-Pro-2.local 25.6.0 arm64; docker ps listed Vorona-dev.
- PRE-CHANGE raw initial-seed edge diagnostics (before any implementation edits):
  main checkout runs/logs/C3-round3-before.log. Every out-of-band edge includes
  endpoint indices, degrees, rows, s/u locations and surface-length/target ratio.
  Cylinder: none. Cone: 18 edges outside nominal band, 6 valid defect exceptions
  and 12 violations; vase: 14 violations, no valid exceptions. These describe
  the unrepaired starting topology, NOT a successful repaired output.
- Owner supersedes the historical strict-every-edge note: each interior tile's
  average incident surface spacing must satisfy the nominal numeric band.
  Each interior-interior edge satisfies that band except exactly 0.05 extra
  variation for an endpoint of degree 5 or 7 (not degree 4/8/9). Both continuous
  residual and terminal acceptance now enforce both rules. Physical and
  auxiliary rim exclusions unchanged; average includes all incident neighbors.
- Construction preserves C2's compressed column sequence, moves transitions
  into the middle of the shared admissible radius interval (rather than C2's
  earliest-switch tie that hid a defect on the rim), then searches a fixed
  64-phase grid between entire row blocks to construct adjacent complementary
  5/7 dipoles. Count change by two requires TWO pairs, per C2, not one pair
  for the whole two-column jump. Nonmanifold phase candidates are discarded.
  Zero-randomness cone construction explicitly errors if required pairs cannot
  be constructed; generic repair is not allowed to insert/delete seeds on a
  column-changing profile. Repair is subsequent local position cleanup only.
- Local spacing springs use the endpoint-specific band and a small inward
  numerical margin, plus an average-spacing force; fixed budget 256 passes.
  Reduced initial organic jitter from 0.45 to 0.32 times sqrt(randomness);
  adjacency-diversity tests are unchanged and still require distinct graphs.
- Acceptance -> exact test mapping: regular C2 cylinder ->
  regular_cylinder_matches_c2_columns; twenty organic cylinders/manifold ->
  twenty_organic_cylinders_are_allowed_and_manifold; revised edge AND average
  spacing across cylinder/cone/vase ->
  every_interior_edge_on_cylinder_cone_and_vase_is_in_band (independent oracle);
  seeds1–5 adjacency diversity at .5/1 ->
  seeds_one_to_five_change_adjacency_not_just_positions; hash determinism ->
  seeded_topology_is_byte_deterministic; final counts and locations ->
  cone_and_bulge_defect_budgets; actual perturbed cone/vase convergence ->
  cone_and_vase_repair_reduce_nonzero_residual; NEW pre-relaxation C2 sequence,
  exact per-transition pair counts and complementary adjacency ->
  cone_pairs_are_constructed_before_repair; NEW exact policy including no
  exception for 4/8 ->
  edge_allowance_is_exactly_five_percentage_points_only_at_five_or_seven.
- No acceptance checks run by this subagent, as explicitly assigned. Only
  container cargo run --example c3_diagnostics for geometry diagnostics, and
  container cargo fmt. Parent must run official make dev-check. No convergence
  certification or impossibility claim. Spacing remains the documented fixed
  64-panel chart-path surface metric, not an exact geodesic solver. R-RM-2
  bounding-square geometry still belongs to C4/C5.
- Post-construction diagnostic (NOT repaired acceptance) saved to main checkout
  runs/logs/C3-round3-constructed.log: cone has four degree-5 sites on rows 3/10
  and four degree-7 sites on rows 4/11, two pairs per C2 transition. Locations
  s=10.392687/15.378202 and 48.801062/53.728365 inches. Remaining starting
  spacing violations are exposed in that log and still require relaxation.


### Round 4 — bounded organic repair handoff (2026-11-10; NOT CERTIFIED)
- Applied vorona-rust-container skill; uname reported Darwin
  Spookys-MacBook-Pro-2.local 25.6.0 arm64 and docker ps listed Vorona-dev.
  Preserved all inherited dirty changes. No branch/commit/merge/push, Python,
  contract or main-checkout edits. Only cargo fmt in Vorona-dev; no tests,
  checks or diagnostic builds run by this subagent.
- Inspected main runs/logs/C3-round3-1735.log: five failures / 638.79 seconds.
  Vase reached ZERO spacing residual but missed the ALREADY +/-1 defect target
  by one additional pentagon; this was not evidence of an exact-count target.
  The existing saturating_sub(1) objective and independent abs_diff <= 1
  acceptance remain consistent. Expanded defect-only geometric search to
  0.10/0.25/0.45 target-size moves to cross flips missed by tiny proposals.
  This is a proposed plateau repair, NOT measured convergence evidence.
- Organic jitter now scales linearly with randomness and available nominal
  circumferential band slack, not a fixed 0.32*sqrt(randomness). Rim angular
  jitter remains bounded by 1.5*variation; no new rim exclusions. Spacing
  remains authoritative in final acceptance, including nominal tile averages
  and exactly +0.05 only for edges touching degree 5 or 7.
- Numerical cap failure at residual 7.17e-17 exposed dead-zone optimization:
  optimize a 1e-7 tighter band, but keep original final acceptance and tests.
  No larger acceptance epsilon was introduced.
- Removed repeated C2 feasibility solves from every repair candidate score:
  reuse one immutable feasibility result throughout a repair call. Dev/test
  profiles now opt-level=2. Replaced 64 midpoint metric quadrature with its
  exact integral for the SAME straight conformal path: integral r(v)dv=delta s,
  length=hypot(delta u,delta v)*delta s/delta v (constant-v limit uses r).
  No approximation to a chord, and no claim of shortest-path geodesics.
- Timing is UNMEASURED: warm <3 minutes is a target, not certified. First run
  after the profile change rebuilds optimized artifacts/dependencies; cold
  compile time must be reported separately from warm official make dev-check.
- Acceptance mapping (exact integration-test names): regular C2 cylinder ->
  regular_cylinder_matches_c2_columns; 20 seeds, allowed degrees, 3-valent
  corners and edge manifold -> twenty_organic_cylinders_are_allowed_and_manifold;
  cone/vase counts and cone transition locations/complementary neighbors ->
  cone_and_bulge_defect_budgets; pre-repair C2 cone construction ->
  cone_pairs_are_constructed_before_repair; same-seed SHA256 ->
  seeded_topology_is_byte_deterministic; seeds1-5 pairwise adjacency at 1 and
  positions at .5 -> seeds_one_to_five_change_adjacency_not_just_positions;
  nominal average and edge bands on all three profiles ->
  every_interior_edge_on_cylinder_cone_and_vase_is_in_band; exact exception
  rule -> edge_allowance_is_exactly_five_percentage_points_only_at_five_or_seven;
  measured nonzero starting residual/strict final success ->
  cone_and_vase_repair_reduce_nonzero_residual; seam/input rejection ->
  periodic_seam_and_bad_sites; exact chart metric ->
  conformal_path_spacing_integrates_surface_metric_exactly.
- Unresolved pending parent's official check: no new convergence, adjacency
  diversity, manifold or runtime results are certified. Smaller interior jitter
  may preserve lattice adjacency; test still REQUIRES pairwise different graphs
  at randomness=1 (not .5) and has not been weakened at 1. Bounded local search
  remains heuristic, with explicit named-tile cap error, never silent success.
  R-RM-2 partial bounding-square guarantee remains deferred to C4/C5. No
  mathematical impossibility is asserted. All card boxes remain unchecked.

### Round 4 verification fixes (2026-11-10; awaiting parent check)
- Read the assigned round4 failure evidence: 7 topology passes / 4 failures,
  repeated seed2 adjacency at randomness1 and vase spacing residual0 but defect
  budget1. Applied vorona-rust-container; uname Darwin Spookys-MacBook-Pro-2.local
  25.6.0 arm64; docker ps included Vorona-dev. Only container cargo fmt run.
- Found a construction-objective mismatch: vase transitions were scored with
  cone-style independent local complementary-pair quotas, although final vase
  acceptance uses the GLOBAL C2 positive-curvature pentagon estimate +/-1.
  Phase construction now scores global vase pentagons and global allowed
  degrees; cone construction's local exact pairing objective is unchanged.
- Addressed the defect plateau's geometric barrier: a proposal that improves
  the integer defect quota may temporarily stretch its new diagonal. Such a
  proposal now gets up to48 deterministic metric-cleanup iterations before
  comparison to the original full score. No uphill endpoint is accepted.
  Defect-only search also visits physical rim sites, holding their arc height
  fixed; boundary angular changes can exchange charge with the open boundary.
  No auxiliary/physical-rim exclusions or ordinary interior definitions changed.
- Independent jitter retains its existing slack/variation caps. Constant-column
  organic patterns additionally choose coherent per-row rotations from the
  sole ChaCha8 stream: zero or 0.85..0.95 pitch at randomness1, ramped from zero
  above randomness0.5. This moves actual indexed sites (no reindex/permutation)
  through alternate Delaunay chambers, with only 0.05..0.15 pitch physical shear
  modulo a row pitch at full randomness. Unlike isolated jitter this explicitly
  explores alternate row-to-row connectivity. Final spacing remains mandatory.
- Nominal tile-average rule and exactly +0.05 ONLY at degree5/7 unchanged;
  optimized profiles, determinism ordering, periodic manifold builder, original
  tests and all prior dirty edits preserved. No Python/contracts, dependencies,
  checks/builds/diagnostics, commits/merge/push or branch changes.
- Test mapping: seeds_one_to_five_change_adjacency_not_just_positions covers
  row-phase diversity and strict spacing; cone_and_bulge_defect_budgets covers
  global vase quota and unchanged cone location/pairing; the vase arms of
  every_interior_edge_on_cylinder_cone_and_vase_is_in_band and
  cone_and_vase_repair_reduce_nonzero_residual cover plateau escape plus strict
  acceptance; cone_pairs_are_constructed_before_repair guards cone construction;
  twenty_organic_cylinders_are_allowed_and_manifold and
  seeded_topology_is_byte_deterministic cover phase manifold/determinism.
- UNVERIFIED: pairwise diversity for the five specific seeds, vase convergence,
  lookahead runtime and full checks. Bounded heuristic can still hit its cap;
  no impossibility or successful convergence claim. Parent must run dev-check.

### Round 4 remaining band-compliance fix (2026-11-10; awaiting parent check)
- Read AGENTS, PLAN, A01-A04, C1/C2 cards and the parent's official
  runs/logs/C3-round4-fix-1814.log (9/11 topology pass). Applied
  vorona-rust-container. Machine: Darwin Spookys-MacBook-Pro-2.local
  25.6.0 arm64; docker ps listed Vorona-dev.
- Found an actual acceptance-domain mismatch, not a metric-integration issue:
  sidecount::interior excluded ALL triangulation hull sites via boundary[i],
  whereas the unchanged independent oracle excludes ONLY auxiliary/physical
  rim sites. Thus an ordinary interior hull site could hide its incident
  edge and average residual, allowing a false zero report. Removed that hull
  exemption from the shared predicate: scoring, terminal acceptance, defect
  accounting and cleanup now cover exactly the physical interior domain.
- Also anchored physical rim heights during simultaneous spring proposals.
  The defect-only search deliberately includes rim sites; its spring arm
  previously clamped their s into (1e-8, length-1e-8), inadvertently converting
  physical rim sites into ordinary interior hull sites. Angular motion stays
  permitted, just as in the existing individual rim proposals. The local
  flip-cleanup arm already limits motion to ordinary interior sites.
- No tests/bands/degree5-or7 extra0.05 allowance or +/-1 vase quota changed.
  No randomness, metric, cone construction, iteration budget, topology builder,
  dependencies, contracts, Python or main-checkout edits. No commits/merges.
  Container cargo fmt only; NO test/check/diagnostic execution by this child.
- Exact regression mapping: every_interior_edge_on_cylinder_cone_and_vase_is_in_band
  and cone_and_vase_repair_reduce_nonzero_residual independently expose the
  missing edge50-60 and false-zero residual. cone_and_bulge_defect_budgets
  guards physical-interior defect counts; cone_pairs_are_constructed_before_repair
  guards unchanged cone construction. The existing diversity, determinism,
  manifold and organic-cylinder tests remain untouched and parent-owned.
- This corrects both acceptance and rim-motion causes, but convergence and
  retained 9 passing requirements remain UNVERIFIED pending the parent's
  full timed official dev-check. No successful-test or impossibility claim.

### Parent verification — round 4 band fix
- Official timed `make -f Makefile -f /tmp/vorona-c3-check.mk dev-check` passed in `Vorona-dev`, working directory `/src/runs/worktrees/C3`; exit 0, wall time 11.66 seconds.
- Evidence: `runs/logs/C3-round4-band-181710.log` in the main checkout. Rust fmt/clippy, 44 Rust tests (including all 11 topology tests), Python ruff and 3 pytest tests passed.
- Vase C2 target ±1: `cone_and_bulge_defect_budgets`; spacing: `every_interior_edge_on_cylinder_cone_and_vase_is_in_band` and `cone_and_vase_repair_reduce_nonzero_residual`; seed diversity: `seeds_one_to_five_change_adjacency_not_just_positions` — all passed.
- Dev/test profiles use opt-level=2; measured complete check is below three minutes. Cone construction, manifold and determinism regressions passed.
- C3 remains on task/C3-topology, uncommitted and unmerged; no merge performed.

### A05 seed-depth handoff (2026-11-11; parent verification pending)
- Worktree `runs/worktrees/C3-depth`, branch `task/C3-seed-depths`, based on
  main `9f2b361`. Applied `vorona-rust-container`; read AGENTS, PLAN, A01–A05,
  C1/C2 dependency cards, C3 and its referenced geometry/spec requirements.
- Added internal `Seed.depth` (inches). Provisional placement/inserted sites use
  row modulo 3: `0, D/2, D`. After successful positional/side-count repair,
  final offsets use `(1-r)*symmetric + r*Uniform[0,D]` in stable tile order.
  This includes auxiliary level-rim rows. At zero randomness no relief draws
  are consumed. At nonzero randomness all relief draws happen AFTER every
  placement/repair draw, on the same caller-owned ChaCha8 stream. Existing
  placement, triangulation, repair scoring/decisions and RNG draws are unchanged.
- `seeds::place` deliberately returns symmetric PROVISIONAL depths; callers
  needing finished organic relief must use `sidecount::build` or `repair` /
  `repair_with_report`, whose successful results carry finalized depths. This
  sequencing prevents extra draws from changing positional repair. Insert/delete
  proposals likewise carry provisional depths until the final site list exists.
- Positive finite D validated at placement and direct repair entry points.
  No public JSON contract/schema/Params edits, Python, C4, dependencies,
  original-checkout edits, commits, merges or pushes.
- Exact new acceptance -> tests:
  * Every finalized depth within [0,D], randomness 0/.5/1, seeds 1–3,
    D=1/2/3 -> `a05_depth_bounds_determinism_and_depth_independent_topology`.
  * Symmetric exact repeating 0,D/2,D and ALL upper/lower adjacent-row
    Delaunay neighbors unequal, including level-rim auxiliary rows ->
    `a05_symmetric_depths_differ_across_every_vertical_neighbor`.
  * Exact blend, independent uniform draw for each seed and exactly one
    shared-stream draw per seed at positive randomness -> unit test
    `seeds::depth_tests::a05_blend_uses_exact_uniform_draws_after_the_symmetric_pattern`.
  * Byte-identical repeat results, linearly scaled relief across D and exact
    equality of every old seed field, corners, neighbors and boundary flags
    across D -> `a05_depth_bounds_determinism_and_depth_independent_topology`;
    same test asserts zero-randomness relief does not advance the stream.
  * Existing SHA256 determinism/manifold/side-count/spacing/diversity tests
    remain unchanged (except adding depth to two explicit metric-test literals).
- Ran ONLY `docker exec -w /src/runs/worktrees/C3-depth/core Vorona-dev cargo fmt`.
  No tests/checks/diagnostic binaries run. All new acceptance is UNVERIFIED;
  parent must run official dev-check and record results. No C4 implementation.

### Parent A05 seed-depth verification
- Official `make -f /tmp/vorona-c3-depth-check.mk dev-check` completed in
  `Vorona-dev`: Rust fmt/clippy and all Rust tests passed, including 13 topology
  tests; Python ruff and all 49 Python tests passed.
- Log in the main checkout: `runs/logs/C3-depth-verify.log`.
- Bounds, deterministic blending and unchanged topology ->
  `a05_depth_bounds_determinism_and_depth_independent_topology` -> PASS.
- Symmetric vertical-neighbor separation ->
  `a05_symmetric_depths_differ_across_every_vertical_neighbor` -> PASS.
- Exact shared-stream blending ->
  `seeds::depth_tests::a05_blend_uses_exact_uniform_draws_after_the_symmetric_pattern`
  -> PASS. Implementation remains unmerged for owner review.


### Exact-reference padding/repair follow-up — 2026-11-12 (UNVERIFIED)

- Same dirty C4-depth worktree retained. Two fixed-pitch support rows at each
  open end for BOTH rim styles, explicit auxiliary flags; physical rim seeds
  retain exact arc position under jitter (not merely triangulation-hull rows).
- Tight-band repair now differentiates the actual squared edge AND incident
  average residual with fixed-chamber central differences. This includes
  neighboring average derivatives missing from endpoint-only springs. Every
  trial is retriangulated; legal bands, degree sets and defect quotas unchanged.
- Fixed-budget row-phase chamber proposals plus multiscale single-site proposals;
  integer-penalty-improving trials get bounded metric cleanup before comparison.
  Strict decreasing score acceptance retained; cap 512 outer / 96 cleanup steps.
  Same seeded ChaCha stream, stable BTree row order, no parallelism/dependencies.
- `exact_reference_full_topology_matrix`: cylinder [6], cone/vase [5,6,7],
  target 6.5, variation .15, r=0/.5/1, seeds1..10 (90 cases). Asserts build,
  manifold, exact edge/average bands, allowed degrees, >=2 auxiliary rows each
  side, exact repeat topology. `build` still checks the original defect budget.
- No tests/builds/checks/diagnostics executed. This is implemented repair, NOT
  evidence the full matrix passes; parent must execute and retain failures.


### Reference cylinder metric-resolution follow-up — 2026-11-12 (UNVERIFIED)

- Read parent `runs/logs/C3-C4-rim-matrix.jsonl`: half-random cylinder seeds
  1–8 all hit the cap with zero defect budget, degree-six violations and small
  positive spacing residuals (e.g. seed 1: 7.739694810859e-6). This is failure,
  not acceptance. Preserved all dirty changes; no checks/builds/diagnostics run.
- Identified a resolution floor in metric descent: eight normalized line-search
  steps stop at target/640 = 0.01015625 inches for target6.5, overshooting tiny
  active band violations. Replaced with fixed 40-step halving backtracking and
  sufficient decrease; finite-difference probes reduced below the optimization
  inset to avoid straddling inactive band hinges. Acceptance bands, tolerance,
  degree requirements and physical-rim exclusions unchanged.
- New `reference_cylinder_half_randomness_cleans_small_spacing_residuals` uses
  the exact sampled matrix fixture (not the two-point profile helper), target6.5,
  variation.15, depth1.5, maxangle230, [6], randomness.5, seeds1–10. It asserts
  successful repair, initial nonzero residual reduction, manifoldness, unchanged
  spacing oracle and interior degree6. Full exact matrix test now also retains
  the sampled cylinder fixture, matching c4_matrix rather than replacing it.
- Parent must rebuild and run the focused regression, full topology tests,
  matrix and official checks. This source-only fix is not measured convergence
  evidence. No commits/merges/pushes; formatting only in Vorona-dev.

### Reference v2 cancellation repair — 2026-11-12 (UNVERIFIED)

- Preserved dirty C4-depth worktree, no commit/merge. Applied container skill;
  only container cargo fmt, no builds/tests/diagnostic runs. Parent verifies.
- Inspected all nine completed v2 JSONL records available at final inspection:
  cylinder r=.5 seeds 1/4/5/7 hit cap512, defect budget0, degree6 throughout.
  Residuals respectively 1.41697e-15, 8.04287e-15, 6.56058e-10, 1.77915e-7.
  Seed1's in-band edge extrema do NOT prove tile averages in band: acceptance
  and residual both include averages over ALL incident neighbors, including rims.
- Found numerical instability in the actual chart metric: ds/(v_b-v_a) loses
  significant digits for nearly same-height sites far from chart origin. Arc
  finite differences at 6.5e-8 inches can thus measure metric cancellation noise
  instead of the band gradient. Surface::conformal_delta now integrates local
  endpoint segments and full intervening chart intervals; same mathematical
  chart path, endpoint-radius continuation and geometric tolerances unchanged.
  spacing uses that stable signed separation, with exact equal-height limit.
- Probe size now shrinks with optimization residual to avoid differentiating a
  smoothed hinge near convergence; fixed backtracking40/outer512/cleanup96 caps,
  inset1e-7, final acceptance epsilon1e-9 and all contract bounds unchanged.
- Added analytic-cylinder near-horizontal regression at arc30/50/60/61, with
  separations0 through1e-12..1e-2, both endpoint orders; 1e-12-inch error limit.
  Full exact-reference90-case regression now aggregates build errors instead
  of stopping on the first cap, asserts90 attempts, retains all independent
  manifold/spacing/degree/support/determinism assertions on successful builds.
- Parent should rebuild v3; run near_horizontal_spacing_has_no_absolute_chart_cancellation,
  reference_cylinder_half_randomness_cleans_small_spacing_residuals, exact90,
  official checks and matrix. No convergence claim until those checks pass.

### Reference v3 finite feasibility-gap repair — 2026-11-12 (UNVERIFIED)

- Read v3 topology failure: stable-chart regression passes, but half-random
  cylinder seed1 still fails after512 accepted moves, original residual
  2.111213397185e-14, edge extrema1.045600915/1.149999953, bad81/83/87 degree6.
  This disproves the previous cancellation fix as a sufficient convergence fix.
  Acceptance and residual are consistent: rim-incident averages can fail even
  when every interior-interior edge is legal. No rim-average exemption added.
- Remaining conditioning issue: old search inset m=1e-7 gives no finite-width
  useful feasibility gap. For each nominal constraint excess e, search penalizes
  max(e+m,0)^2; only total search residual <m^2=1e-14 certifies EVERY nominal
  constraint is satisfied. Steepest descent approaches that boundary while
  spending the fixed budget resolving other active averages/edges.
- Set a shared optimization-only margin m=0.01*size_variation (m=.0015 at
  reference inputs). Thus search residual <2.25e-6 certifies nominal feasibility,
  rather than requiring <1e-14. All metric, spring and flip-cleanup proposals
  use the SAME margin. This is a strictly inward search target, NOT permission
  to violate any original limit. Original residual and final acceptance retain
  original bands, degree exceptions and epsilon1e-9. Caps unchanged512/96/40.
  Build still accepts any nominally valid candidate before further tightening.
- New private unit regression constructs a six-neighbor local star with physical
  rim neighbors: NO checked interior-interior edges, but average ratio exceeds
  nominal1.15 by5e-8. Verifies failure despite legal edge extrema, substantial
  inward descent, and strict nominal feasibility once the margin certificate
  is met. Synthetic star tests objective only; manifold tests remain real builds.
- Last allowed move now receives terminal acceptance/report refresh, previously
  omitted after pass512. No additional repair proposal/pass. Failure diagnostic
  now includes max average excess and optimized residual, not only edge extrema.
- At final read v3 matrix had zero completed records; official check was still
  in planes tests (no additional failures available). Only container cargo fmt;
  no builds/tests/diagnostics/commits/merges. Parent must run new unit regression,
  focused half-random10-seed regression, exact90 matrix and official checks.
  This establishes a mathematical feasibility gap, not measured convergence;
  no heuristic solver success is claimed before parent verification.

### Integer lattice / identical-native-input comparison handoff — 2026-11-12 (UNMEASURED)

- Preserved ALL inherited dirty edits. Applied container skill; machine reported
  Darwin Spookys-MacBook-Pro-2.local 25.6.0 arm64; docker ps included Vorona-dev.
  Only container cargo fmt executed; no check/build/test/benchmark/solver run.
- C2 now allows every integer count; nearest circumference/target breaks the
  minimum-change schedule tie (reference cylinder 9). C3 consumes that schedule,
  never stretches row arcs, and continues the exact last pitch through >=2
  support rows beyond each rim. Added explicit nine-column/pitch/padding test.
- Extracted shared native Problem construction, not a second target generator.
  c4_native_export emits sites/depths, complete triangle/array ordering, both
  tangent axes (do NOT assume native tangent signs equal prototype signs),
  native LS initialization, fitted tilts/corners, whole selection and evaluator.
  `legacy` argument reconstructs the preceding 8-column stretched symmetric
  cylinder, then uses exactly the same native fit/export code.
- A05_native_compare.py applies the prototype planes/intersections/residual and
  SciPy least_squares(max_nfev=300) to the EXPORTED arrays and EXPORTED initial
  vector: no Delaunay, no reseeding, no recomputed targets. Independently compares
  the native LS start against numpy lstsq. Writes objective/target RMS, dynamic
  and frozen-native whole-tile unsigned-angle/edge ranges, and every corner's
  native/reference coordinate error and start/native/reference target error.
  Native corner reconstruction error detects ordering/axis parameter mistakes.
  This enables diagnosis, but DOES NOT establish a measured fit-bug cause/fix.
  Do not blame lattice alone or claim parity before executing this gate.
- C3 performance: finite differences formerly reevaluated the entire graph for
  each coordinate. They now evaluate ONLY incident hinges and this site's and
  neighbors' averages: all other terms cancel identically. Full trial score,
  retriangulation, caps and terminal limits unchanged. Added local/global
  objective-difference test. Runtime improvement remains unmeasured.
- Actual previous v4 matrix file contained 42 records: cylinder23, cone19,
  ending cone depth1.5 r1 seed8. Thus vase was NOT removed from source; the run
  had not reached it. Existing processes untouched. Parent coordinates settlement
  before new execution. Current full matrix explicitly includes ALL r0/.5/1
  seeds1..10 at depth1.5 for cylinder/cone/vase: 90 acceptance + 6 symmetric
  depth1/2 report records = 96 (not69). Optional positional profile argument
  `vase_bulge` enables an immediate 32-record vase diagnostic without weakening
  the eventual full matrix. Defaults1.5/230 and never-worse polish unchanged.

Parent commands (host, from repository root; wait for existing v4 jobs first):

```sh
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo run --release --example c4_native_export > runs/logs/C4-native-nine.json 2> runs/logs/C4-native-nine.stderr.log
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo run --release --example c4_native_export -- legacy > runs/logs/C4-native-legacy-eight.json 2> runs/logs/C4-native-legacy-eight.stderr.log
docker exec -w /src/runs/worktrees/C4-depth Vorona-dev python docs/prototypes/A05_native_compare.py /src/runs/logs/C4-native-nine.json /src/runs/logs/C4-native-nine-comparison.json > runs/logs/C4-native-nine-comparison.log
docker exec -w /src/runs/worktrees/C4-depth Vorona-dev python docs/prototypes/A05_native_compare.py /src/runs/logs/C4-native-legacy-eight.json /src/runs/logs/C4-native-legacy-eight-comparison.json > runs/logs/C4-native-legacy-eight-comparison.log
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --lib local_probe_matches_global_objective_difference
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --test topology reference_lattice_has_nine_columns_without_row_stretching
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --test feasibility --test planes --test contract
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev bash -c 'time cargo test --test topology exact_reference_full_topology_matrix -- --nocapture' > runs/logs/C3-nine-warm-timing.log 2>&1
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo clippy --all-targets -- -D warnings
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo run --release --example c4_matrix -- vase_bulge > runs/logs/C4-nine-vase.jsonl 2> runs/logs/C4-nine-vase.stderr.log
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo run --release --example c4_matrix > runs/logs/C4-nine-full-96.jsonl 2> runs/logs/C4-nine-full-96.stderr.log
```

For official worktree dev-check, create a supplemental Makefile overriding ONLY
`dev-check`'s docker exec working directory to `/src/runs/worktrees/C4-depth`,
keeping `dev-up`, `pip install --no-deps -e . && make check` intact, then run
`make -f Makefile -f /tmp/vorona-c4-nine-check.mk dev-check` and save raw output.
This is needed because inherited Makefile dev-check has no -w and otherwise
checks /src, NOT this worktree. No numerical acceptance or checklist completion
is claimed. Parent owns execution, measured diagnosis and any ensuing solver fix.


### Position-only adjacency regression correction — 2026-11-12 (parent checks pending)

- Owner's latest bounded ±10% LOCAL position-only jitter supersedes the old
  full-randomness pairwise graph-uniqueness experiment. Delaunay connectivity
  is piecewise constant under position changes: a small perturbation may remain
  in the same triangulation chamber. Repeated seed2 adjacency alone is therefore
  NOT evidence of failed positional randomness; no flip is required by this rule.
- Replaced `seeds_one_to_five_change_adjacency_not_just_positions` with
  `seeds_one_to_five_change_positions_with_valid_geometry`: still requires
  pairwise different final sites at randomness .5/1, independent physical
  edge/average spacing acceptance, and now explicitly checks manifold geometry.
  No geometric acceptance, degree/defect tests, or C4 strict limits were relaxed.
- Added independent cylinder placement component-cap/depth oracle covering
  randomness 0/.5/1 and seeds1..5, fixed rim heights, and zero-randomness site
  independence. Added byte-identical repaired topology seed1..5 regression at
  randomness0 with spacing/manifold checks (full planes regression remains).
- ONLY tests and task notes changed for this correction. Production source is
  untouched: running isolated position-only matrix remains valid for the same
  production sources/binary. No restart/rebuild needed for this correction.
  Container fmt only; no tests/builds/diagnostics/commits/merges by child.
- Parent: in Vorona-dev, cwd `/src/runs/worktrees/C4-depth/core`, run
  `cargo test --test topology` then the existing official worktree-selecting
  `make dev-check`; preserve raw logs. Full end-to-end seed-independent plane
  gate remains `cargo test --release --test planes
  symmetric_geometry_is_seed_independent_end_to_end -- --ignored --nocapture`.
