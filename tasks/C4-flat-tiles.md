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

### Exact prototype parity gate handoff — 2026-11-12 (not measured)

- Owner follow-up: acceptance is depth **1.5**, including the unchanged organic
  seed matrix and cone/vase; depths **1 and 2 report only**. No weakened limits.
- Incorporated approved C3 main `1a404d7` by ordinary merge, preserving the
  parent's dirty Python/example edits. No implementation commit, push or merge
  into main. Container `cargo fmt` only; parent owns all execution/certification.
- New `docs/prototypes/A05_export.py` executes the original prototype unchanged
  with `runpy`, exporting exact SciPy Delaunay triangles, 3D points, target
  corners, radial/tangent frames, fitted start, final least-squares solution,
  reference corners and exact whole-tile selection. Identity includes source
  SHA256, complete-export array digest, package versions, lattice parameters,
  mode, seed and depth. This is a reference experiment, not production Python.
- New `c4_reference` reconstructs both exact start and fitted geometry using
  Rust's three-plane intersection and asserts fitted corners agree within 1e-8.
  Metrics use the reference's whole-tile IDs, not C3's different rim policy.
  `barriers` starts **at that exact SciPy solution**, reusing current C4 barriers.
  It is deliberately NOT a claim that native C4 reproduces SciPy's fit solver.
- Root-cause evidence from source comparison: the previous measurement was NOT
  the reference experiment. Reference chooses round(2π·9/6.5)=9 columns, starts
  row k=-2, keeps fixed spacing h=(2π·9/9)√3/2, and extends beyond both ends.
  C3 requires an even column schedule (seed code uses 2*n), redistributes row
  spacing to hit the top, and uses its own auxiliary/rim rows. Thus lattice,
  depth phase/placement and triangle identities cannot match. Previous C4 never
  performed the prototype-only least-squares baseline: prototype residual is
  [corner-target, .05*x], whereas existing C4 begins with [.03*(corner-target),
  .005*x, .01*edge-regularity, .005*adjacent-tilt, sqrt(.001)*violations].
  It then increases barriers. These are different optimization problems before
  a prototype baseline has been established. They explain why prior output
  cannot be compared as a reproduction; assigning the 6-inch/140-edge failure
  to one particular difference still requires measured stage isolation.
- This is the requested baseline-stage handoff. No numerical parity, successful
  baseline reproduction, native baseline convergence, barrier convergence or
  acceptance is claimed until parent runs the gate. Production C3 lattice is
  unchanged; reference import is diagnostic only. All work preserved.

Parent commands (container, `/src/runs/worktrees/C4-depth`; use a scratch folder
for prototype-created tile JSON; numpy/scipy must be installed in the container):

```sh
mkdir -p /tmp/c4-reference
cd /tmp/c4-reference
D=1 python /src/runs/worktrees/C4-depth/docs/prototypes/A05_export.py /tmp/c4-reference/d1.json symmetric 1
D=1.5 python /src/runs/worktrees/C4-depth/docs/prototypes/A05_export.py /tmp/c4-reference/d15.json symmetric 1
D=2 python /src/runs/worktrees/C4-depth/docs/prototypes/A05_export.py /tmp/c4-reference/d2.json symmetric 1
cd /src/runs/worktrees/C4-depth/core
cargo run --release --example c4_reference -- /tmp/c4-reference/d15.json start
cargo run --release --example c4_reference -- /tmp/c4-reference/d15.json fit
cargo run --release --example c4_reference -- /tmp/c4-reference/d1.json fit
cargo run --release --example c4_reference -- /tmp/c4-reference/d2.json fit
cargo run --release --example c4_reference -- /tmp/c4-reference/d15.json barriers
cargo test --test contract --test planes
cargo test --release --test planes -- --ignored --nocapture
```

Save raw stdout and stage JSON under `runs/logs/` in parent. The report's signed
corner summary includes auxiliary corners; comparable whole-tile values are in
`metrics.signed_corners`. `metrics.unsigned_angles` reproduces the prototype's
printed small angle, not its separate signed convex/reflex acceptance check.
Barrier piece selection is frozen to reference fitted whole tiles for controlled
stage comparison; it is not production dynamic rim clipping or C5 certification.
Parent must run official worktree `make dev-check` after diagnostic review.


### Native baseline objective continuation — 2026-11-12 (UNMEASURED)

- Added `native-start`, `native-fit`, `native-barriers` to `c4_reference`.
  These recompute the two-tilt least-squares start in Rust, then solve exactly
  `[corner-target, .05*tilt]` with a native analytic implicit-intersection
  Jacobian and deterministic damped LM (300 iterations, bounded retries).
  `native-barriers` starts from THAT native result, never imported SciPy tilts.
  Existing `start`/`fit`/`barriers` gates remain intact for controlled comparisons.
- The native LM trajectory is not SciPy TRF: compare objective sum of squares,
  target RMS, reference-corner max error and all whole-tile metrics before
  claiming convergence/parity. The example now emits these measurements.
- Remaining staged dependency: exact points/frames/targets/triangles/whole-tile
  selection still come from the reference export. No independent native
  prototype lattice/Delaunay reconstruction is implemented. Thus this is native
  OBJECTIVE/SOLVER parity code, NOT full native end-to-end baseline parity.
  In particular it cannot certify production C3's different lattice. Imported
  whole-tile selection is deliberately frozen for isolation; a different native
  fit may require a separate dynamic whole-tile comparison before acceptance.
- No tests, checks, diagnostics, compilation or measurements run by this child;
  parent owns execution. Formatting still required in Vorona-dev. No commits,
  push or merge. Depth 1.5 all-limits acceptance remains unproven; 1/2 report only.

Parent commands inside Vorona-dev, workdir `/src/runs/worktrees/C4-depth/core`:

```sh
cargo fmt
cargo run --release --example c4_reference -- /tmp/c4-reference/d15.json native-start
cargo run --release --example c4_reference -- /tmp/c4-reference/d15.json native-fit
cargo run --release --example c4_reference -- /tmp/c4-reference/d15.json native-barriers
cargo run --release --example c4_reference -- /tmp/c4-reference/d1.json native-fit
cargo run --release --example c4_reference -- /tmp/c4-reference/d2.json native-fit
```

Compare `start` vs `native-start` first (isolates initial LS), `fit` vs
`native-fit` second (isolates solver, objective held fixed), then `barriers` vs
`native-barriers` (isolates basin differences). Both barrier branches retain
all existing limits. Official worktree dev-check and strict acceptance matrix
must follow final edits; this handoff does not unblock C5.

### Verification correction — 2026-11-12 (parent checks pending)

- Official contract failure was stale `core_output.schema.json` Params.depth
  default 2.0 versus owner-approved input/Rust default 1.5. Updated output mirror
  to 1.5; retained exact schema-equality assertion and every range test.
- Bounded solver bookkeeping fix: rank the final accepted trial after the 80-step
  loop at the last iteration's barrier weight. Previously it could be discarded
  without evaluation for best-report selection. No iterations, barrier schedule,
  tolerances, limits or optimization objective changed. No convergence claimed.
- No tests or diagnostics run by child; parent owns verification and measurements.

### Parent reference/native measurements — 2026-11-12

- Exact reference exports reproduce depth 1 edges 2.6458–4.8223 in and minimum unsigned angle 69.1398 degrees; depth 2 edges 2.2422–6.7514 in, minimum angle 80.3034 degrees, signed whole-tile corners -2.8362/+0.8949 in. Logs: `runs/logs/C4-reference-d1-export.log`, `runs/logs/C4-reference-d2-export.log`.
- Controlled initialization agrees to floating-point precision. Depth 1.5 native fit objective 1099.9660015 versus reference 1099.9659939; maximum coordinate error 0.0010091 in. Depth 2 native fit maximum corner error 0.0042736 in. Depth 1 native fit reaches a different basin: max edge 6.8778 in versus reference 4.8223 in. Native solver parity is therefore incomplete, not certified by a close depth-1.5 objective.
- Post-bookkeeping-fix release `native-barriers` at depth 1.5, pinned to CPU 0, still fails. Whole-tile metrics: edges 1.01197–6.68515 in; unsigned angles 39.3487–142.0891 degrees; signed corners -1.85092/+1.68779 in. Full best report includes auxiliary geometry: signed corners -14.18770/+7.97630 in, median -1.53928; 117 reflex corners; violation entries max_edge 162, min_corner_angle_deg 108, max_corner_angle_deg 108. Entries are not unique tile counts. Saved exact geometry/report: `runs/logs/C4-d15-barriers-final.json`; build log: `runs/logs/C4-d15-barriers-final.log`.
- Earlier 6-inch/140-edge output was not a reference reproduction: lattice, rim selection and weighted objective differed. Controlled native fitting now separates initialization agreement from solver/basin disagreement; it does not isolate one cause for every earlier defect.
- STOP under A05 section 4 after the measured barrier failure. Depth-1.5 acceptance is NOT met; organic/cone/vase acceptance remains unmeasured. Reference lattice/triangulation/targets are still imported diagnostic inputs, not a native production implementation. C4 remains unmerged; C5 is not unblocked. No limit was relaxed.


### Final parent verification — 2026-11-12

- Final isolated-worktree `make dev-check` completed with exit 0 after the fixture warning regression fix. Rust fmt/clippy/non-ignored tests and Python ruff/49 tests pass. Log: `runs/logs/C4-review-check.log` (parent repository).
- Warning regression now explicitly uses depth 2.0 and checks that the approved default 1.5 remains warning-free and feasible; no geometry limits were changed.
- Standard check success does not certify ignored release acceptance experiments. The measured depth-1.5 barrier failure above remains the stop condition under A05 section 4. C4 is unmerged and incomplete; C5 remains blocked pending owner direction.

### Owner fitted-result implementation — 2026-11-12 (UNVERIFIED)

- Applied owner follow-up: production `optimize` now builds its points, frames,
  circumcenter/mean-depth targets from native C3 topology and runs the shared native
  reference objective `[corner-target, .05*tilt]` before any polishing. `fitted` exposes
  this baseline with the same complete C4 evaluator. No SciPy export is required
  by production or the new matrix harness. C3 lattice and side-count repair remain
  unchanged; this does NOT reproduce the prototype's distinct nine-column lattice,
  nor claim SciPy trust-region trajectory/basin parity. That remains a measurable
  difference, not an imported-input production claim.
- Feasible fits return immediately unchanged. Polish accepts only lower normalized
  squared positive limit residuals, no increased violation-entry count and no newly
  broken tile/limit category, in addition to lower solver cost. Target/fairness gains
  alone cannot authorize a step. Best selection uses violation count and unweighted
  limit severity, never the changing weighted target objective. New report field
  `limit_violation_sum_sq` is internal diagnostics, not CoreOutput schema.
- Tests added: actual feasible tetrahedral geometry is byte-identical after solve;
  guard rejects worsening severity and newly broken limits; production cylinder fit
  versus polish checks never-worse invariants. Parent must run them.
- `c4_matrix` emits 69 JSONL case records, catches individual panics/errors and
  continues. All three profiles: depth 1.5 symmetric plus randomness .5/1 seeds
  1..10; depths 1/2 symmetric report only. Reports full fit and polished geometry,
  all evaluator violations, C3 errors, topology coverage, separate fit and complete
  fit+polish runtime, and total per-case runtime. No subset of `keep` whole tiles
  is used. Open hull fans remain auxiliary/non-polygons; all closed fans, all
  intersections and all neighbour bevels are evaluated. Final C5 clipped rim
  geometry is still outside C4 certification. Collinear overlap robustness remains
  the previously documented C6 scope. This is not a claim that ALL limits pass.
- Owner default max reflex angle 230 in Rust, both schemas and INTERFACE; depth
  stays 1.5. Parent owns Python/examples and any explicit fixture synchronization.
- Only container formatting run by child. NO builds/tests/diagnostics/commits,
  merges or pushes. Parent owns execution and acceptance.

Exact parent commands from repository root (stderr separate from JSONL; no
per-case failure aborts matrix; pin a CPU allowed in this container):

```sh
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --lib --test contract --test planes
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo clippy --all-targets -- -D warnings
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_matrix
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /src/core/target/release/examples/c4_matrix > runs/logs/C4-fitted-matrix.jsonl 2> runs/logs/C4-fitted-matrix.stderr.log
```

If CARGO_TARGET_DIR differs, use `docker exec ... taskset -c 0 cargo run
--release --example c4_matrix` instead of the explicit binary; compiler output
stays on stderr. Official worktree `make dev-check` is still required. The
strict ignored test suite is not the batch measurement mechanism (it fail-fasts).


### Open-end support and whole-tile measurement — 2026-11-12 (UNVERIFIED)

- Preserved all pending changes. Applied vorona-rust-container skill. No old
  C4-planes reuse, commits, merge, push, tests, builds or diagnostics.
- C4 still fits all planes/targets. Generic `SurfaceQueries::inside_rims`
  delegates rim membership to the surface adapter (Y rim planes for revolve).
  A whole tile has a closed fan, is non-auxiliary, and all fitted corners lie
  between physical rims. `Planes.whole_tiles` exposes this mapping alongside
  ALL polygons/corners/planes. Selection recomputes from each evaluated geometry.
- All limit barriers (including corner determinant and shared-edge bevel) and
  reflex counts are masked to whole tiles. Corners shared by any measured tile
  get determinant checks; bevel checked if either incident tile is measured.
  All-plane fit and regularity retained; masked residual slots remain present
  to preserve finite-difference Jacobian indexing. Distance statistics use the
  unique union of whole-tile corners only; empty selection has zero statistics
  and is identifiable by `whole_tiles` (not a claim of produced pieces).
- Existing fit objective / never-worse polish logic untouched; defaults remain
  depth1.5, maxangle230. Matrix remains all 69 cases (63 acceptance, 6 report),
  pins target6.5/variation.15 explicitly and adds auxiliary seed counts. Each
  fit/polish result carries its own whole-tile mask for measured comparisons.
- Remaining limitations: no measured proof of C3 convergence or C4 acceptance,
  no C5 clipping/partial-piece compliance, no change to polygon crossing guard.
  Dynamic whole membership may differ fit versus polish and must be inspected;
  never-worse is evaluated by existing normalized limit violation/report ranking.

Parent commands (from host; all execution in existing container):

```sh
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --test topology exact_reference_full_topology_matrix -- --nocapture
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --test topology --test planes --test contract
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo run --release --example c4_matrix
```

Save the complete non-fail-fast 69-record matrix output. Then run the official
worktree dev-check recipe (not native make check), with raw logs, after resolving
any failures. The subagent cannot certify numerical success without execution;
no checklist is marked complete.

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

### Native TRF parity / monotone polish handoff — 2026-11-12 (UNMEASURED)

- Read binding AGENTS/PLAN/A01–A05 and applied container-only Rust skill.
  Preserved all pending edits, comparison script and corner-by-corner artifacts.
  No commits, merges, pushes, builds, checks, tests or diagnostics by this child;
  only `cargo fmt` in Vorona-dev. Acceptance remains explicitly unverified.
- Identical-input evidence: native-nine objective 484.443319898 versus SciPy
  484.443333306 already shares a basin; legacy-eight 643.574497245 versus
  520.028678783 exposes the solver trajectory gap, not a lattice impossibility.
  Starts differed from SVD by only 2.6e-15 or less. Existing native LM was NOT
  the reference solver: analytic Jacobian, ad hoc damping, 1e-12 cost stopping.
- Replaced that fit with unbounded exact-subproblem TRF: two-point differences
  in reference block order, unit scaling, initial radius ||x0||, More SVD secular
  solve (10 iterations, .01 radius tolerance), ratio-based radius updates,
  ftol/xtol/gtol=1e-8, max_nfev=300 excluding numerical Jacobian calls. Start
  now uses SVD rank cutoff eps*max(m,n)*smax rather than normal equations.
  Same objective and regularization. Different SVD/intersection arithmetic means
  bitwise SciPy parity is NOT promised; numerical parity MUST be measured.
- Dependency note (AGENTS exception documentation): added nalgebra 0.33/std,
  no default features, for stable dense SVD and minimum-norm initialization;
  this replaces bespoke normal-equation algebra. Parent's first cargo invocation
  must resolve/update Cargo.lock (child intentionally performed no build).
- Polish now freezes fitted whole-tile membership and accepts only a strict
  aggregate violation decrease with NO increase in any individual barrier and
  NO newly broken tile/name limit. It no longer requires target/fairness cost
  descent. Weight=1 avoids the old early tiny barrier versus target competition.
  Supporting planes remain fitted; feasible baseline remains byte-identical.
  This is a proposed reflex correction, NOT evidence that <=230 is reached.
- Added frozen SciPy nine/legacy-eight input+result fixtures with input hashes,
  solver provenance, objective gate 0.001 and per-coordinate corner gate .02in.
  Existing comparison harness remains intact for full corner-by-corner review.
  Added strict release polish gate (no new failures, same selection, no reflex
  excess); failure is surfaced, never converted into success.
- Updated regular_cylinder_matches_c2_columns with an independent nearest-size-
  band integer oracle; cone_pairs_are_constructed_before_repair now independently
  checks actual row widths in the nominal band without any even-count assumption.
  Existing exact transition defect and complementary-neighbor assertions remain.
- Conflict discovered through /proc cmdlines: preexisting `target/release/examples/
  c4_matrix`, `make check`, `cargo test`, and topology test process were ACTIVE.
  No process was stopped. Those binaries predate these edits and cannot certify
  this revision. Parent must settle/label those runs before rebuilding; do not
  mix their reports with this handoff's evidence.

Parent gates FIRST (all commands inside Vorona-dev; cwd
`/src/runs/worktrees/C4-depth/core`, no simultaneous old check/matrix):

```
cargo test --release --test planes identical_input_reference_solver_parity -- --ignored --nocapture
cargo test --release --test planes fitted_cylinder_polish_removes_reflex_excess_without_new_failures -- --ignored --nocapture
cargo test --test topology regular_cylinder_matches_c2_columns -- --nocapture
cargo test --test topology cone_pairs_are_constructed_before_repair -- --nocapture
cargo run --release --example c4_native_export > /src/runs/logs/C4-trf-nine.json
python /src/runs/worktrees/C4-depth/docs/prototypes/A05_native_compare.py /src/runs/logs/C4-trf-nine.json /src/runs/logs/C4-trf-nine-comparison.json
cargo run --release --example c4_native_export -- legacy > /src/runs/logs/C4-trf-legacy-eight.json
python /src/runs/worktrees/C4-depth/docs/prototypes/A05_native_compare.py /src/runs/logs/C4-trf-legacy-eight.json /src/runs/logs/C4-trf-legacy-eight-comparison.json
```

Then full official checks via parent's existing worktree-selecting dev-check
recipe. Only after targeted parity/polish gates, full requested 96-case report:

```
cargo build --release --example c4_matrix
taskset -c 0 target/release/examples/c4_matrix > /src/runs/logs/C4-trf-matrix-96.jsonl
```

The unchanged matrix is 3 profiles * (30 acceptance cases at depth1.5 plus two
report-only depth1/2 cases), non-fail-fast. Parent owns timing, all builds/tests,
full limit analysis, corner-by-corner parity judgment and actual measurements.

### Parent parity pass / residual-floor correction — 2026-11-12

- Parent measured release frozen parity gate PASS, 19.57s, isolated target
  `/tmp/vstone-c4-trf-target`; log `/src/runs/logs/C4-trf-parity-0918.log`.
  Fit implementation is unchanged by this follow-up.
- Parent's strict polish failed at iteration3: sum_sq5.716322701501695e-8,
  tiles44/98 max_corner_angle_deg, `/src/runs/logs/C4-trf-polish.log`.
  Actual cause: feasibility steps still included target/fairness gradients;
  these balance tiny positive hinges and create a nonzero violation floor.
  Polish now zeros ONLY the non-limit residuals and their Jacobian terms.
  No limit or report tolerance changed. Deterministic16-step backtracking
  handles overshooting a satisfied hinge without damping lockup; smaller
  finite-difference/CG/damping floors support final feasibility restoration.
  Individual-barrier never-worse guard, fixed measured domain, no-new-failures
  check and exact feasible-baseline fixed point remain unchanged.
- Added a synthetic regression that has nonzero target-fit cost but exactly
  zero feasibility residual, exposing accidental restoration fairness pulls.
- Parent regular-cylinder test failed its frozen row-gap assertion after
  sidecount::build. That combines initial lattice construction with positional
  repair, which is permitted to move sites. Exact spacing is now independently
  asserted on seeds::place (azimuth-sorted, count equals nearest-band oracle),
  while final repaired topology must preserve each row count, degree6,
  manifold structure AND pass the existing independent per-edge AND per-tile-
  average physical spacing oracle. No production C3 code or bands changed.
- Child again ran ONLY container formatting, no builds/tests/diagnostics. Parent
  should rerun strict polish and regular-cylinder gates first, then synthetic
  feasible_fit_is_byte_identical_fixed_point, followed by full check/matrix.
  Reflex convergence remains unverified until that parent measurement.

### Position-only / isolated supervisor handoff — 2026-11-12 (UNVERIFIED)

- Child used container-only Rust skill; preserved existing dirty changes, no
  commits/merges/pushes. Ran ONLY cargo fmt, no builds/tests/diagnostic geometry.
  Existing historical target/release/examples/c4_matrix and corrected
  /tmp/vstone-c4-trf-target/release/examples/c4_matrix plus check shell were
  observed active via /proc; left untouched.
- Seeds now always use symmetric row depth, consuming zero depth draws.
  Production positional jitter uses ±.1 * randomness times local angular pitch
  and next-row arc spacing (previous spacing at final row). Removed the old
  up-to-.95-pitch coherent random row rotation. Rim heights remain fixed.
  C3 repair can still displace initial sites to enforce unchanged spacing/degree
  limits: the jitter cap describes INPUT perturbations, not a bound on repair.
- Cross-seed mechanism diagnosed by code: zero-amplitude placement still drew
  random numbers and repair used random proposal angles even at randomness0.
  Consequently symmetric runs could enter different repair trajectories, not
  just different fit trajectories. build now canonicalizes the ENTIRE stream
  to seed1 when randomness0. No claim the two observed severe cases have been
  numerically reproduced or resolved until parent runs regression/matrix.
- Polish is actual least-squares penalty GN with corner targets retained at
  3e-8 coordinate weight, original edge/convex/reflex and other limit hinges.
  Frozen whole-tile set and all supporting unknowns retained. Removed overly
  restrictive individual-hinge monotonicity; accepts combined objective descent,
  strict aggregate violation descent, no increase in count/no newly broken limit.
  Explicit final comparison against original fit guarantees fallback; feasible
  original remains byte-identical. No acceptance threshold or limits changed.
- Owner ±10% 36->0 prototype evidence MUST be reproduced by parent, not assumed.
  This implementation and regression have NOT been compiled or measured.
- c4_matrix accepts optional PROFILE DEPTH RANDOMNESS SEED CAP; one invocation
  with all five selects exactly one case. No-argument historical96 still works
  but does NOT isolate/time-limit cases; use new supervisor for official matrix.
  Supervisor executes cylinder FIRST, then cone, then vase, single process/case,
  subprocess timeout60 includes topology+fit+polish; timeout kills and reaps only
  its own case and proceeds. JSONL flushes timings/results/failures each case.
  stdout geometry may be large. Existing binary fits twice (diagnostic fit and
  optimize refit); recorded whole-case60 includes BOTH, intentionally conservative.
- Diagnostic caps are explicit APIs (no environment geometry settings).
  Sweep builds once, runs multiple bounded caps; summary largest TESTED passing
  cap requires all selected acceptance cases pass, never a global maximum claim.

Parent commands (inside Vorona-dev, cwd /src/runs/worktrees/C4-depth/core;
wait/label existing jobs independently before builds):

```
cargo test --lib a05_depth_is_row_symmetric_and_consumes_no_random_draws
cargo test --release --test planes symmetric_geometry_is_seed_independent_end_to_end -- --ignored --nocapture
cargo test --release --test planes fitted_cylinder_polish_removes_reflex_excess_without_new_failures -- --ignored --nocapture
cargo test --release --test planes identical_input_reference_solver_parity -- --ignored --nocapture
cargo build --release --example c4_matrix
python ../docs/prototypes/C4_matrix_runner.py "$PWD/target/release/examples/c4_matrix" > /src/runs/logs/C4-position-only-matrix96.jsonl
python ../docs/prototypes/C4_matrix_runner.py "$PWD/target/release/examples/c4_matrix" --caps .05,.075,.1,.125 --sweep-only > /src/runs/logs/C4-position-only-jitter-sweep.jsonl
```

If parent sets CARGO_TARGET_DIR, supply its actual absolute example binary path
instead. Pin supervisor with taskset -c0 (space-separated `-c 0`) for inherited
single-core timing. Then run official worktree-selecting dev-check described
above and save raw logs. No gate/checklist marked complete. Unresolved: measured
reflex convergence, exact two severe-case diagnosis/reproduction, all96 limits,
reference parity after canonical repair stream, largest tested passing cap.


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

### Single-fit profiling handoff — 2026-11-12 (UNMEASURED)

- Applied container-only Rust skill; existing full96 `tidal-meadow` and sweep
  `good-cove` processes/binaries/targets left entirely untouched. Historical
  60-second timeout records are DOUBLE-FIT HARNESS timeouts, not evidence of
  production solver timeouts. Their already-running cone/vase matrix continues.
- `planes::fit_and_polish` computes the identical production initialization and
  reference TRF fit ONCE, retains fitted geometry/report, and passes unchanged x
  to the same polish. Reuses the exact baseline `evaluate(x,1)` instead of
  recomputing it. Production entrypoints share validation and polish helper;
  objective, regularization, stopping, finite differences/SVD, frozen whole-tile
  domain, all limits and final never-worse fallback are unchanged.
- New c4_matrix records carry `harness: single-fit-v1`, topology_seconds,
  fit_seconds, polish_seconds, optimizer_total_seconds and outer total_seconds.
  Fit time includes preparation/start and fitted report. FitProfile records nfev,
  residual_calls, residual_seconds, jacobians/jacobian_seconds, svds/svd_seconds.
  Jacobian time INCLUDES its residual calls: do not sum overlapping counters.
  These diagnostic clocks do not affect numerical decisions or CoreOutput.
- No speculative solver replacement/Jacobian change: parent should measure first.
  If Jacobian dominates, evaluate structurally sparse *identical two-point*
  residual updates (not analytic derivatives first, which change rounding and
  potentially the TRF trajectory). If dense SVD dominates, sparse/iterative
  replacement is NOT parity-neutral and needs separate evidence/gates. If polish
  dominates, profile its full-limit evaluations before optimizing them.
- Parent owns all builds/tests/measurements. Child performs source edits and
  container formatting ONLY; no pass/runtime claim or acceptance checkbox.

Parent commands from HOST repository root (new isolated target; no need to
stop or rebuild any historical job):

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-single-fit-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes single_fit_diagnostic_matches_production_stages -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-single-fit-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes identical_input_reference_solver_parity -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-single-fit-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes symmetric_geometry_is_seed_independent_end_to_end -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-single-fit-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_matrix
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-single-fit-target/release/examples/c4_matrix cylinder 1.5 .5 1 .1 > runs/logs/C4-single-fit-cylinder-r05-s1.jsonl
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-single-fit-target/release/examples/c4_matrix cylinder 1.5 1 1 .1 > runs/logs/C4-single-fit-cylinder-r1-s1.jsonl
```

The direct per-case commands deliberately have no 60-second kill: complete
stage timings diagnose why historical double-fit jobs exceeded 60. Repeat for
actual historical timeout profile/seed identifiers; collect fit_profile and
all four stage times. Pinning to one logical CPU is inherited by solver work.
Concurrent historical runs may contend; report concurrent-load measurements as
such, then repeat only if contention materially obscures the result.

After profiling, the unchanged supervisor can run the new single-fit binary
with its 60-second per-case bound and full cylinder/cone/vase matrix:

```sh
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 python ../docs/prototypes/C4_matrix_runner.py /tmp/vstone-c4-single-fit-target/release/examples/c4_matrix > runs/logs/C4-single-fit-matrix96.jsonl
```

Regression compares complete fitted and polished serialized results (including
failure best geometry/report) against separate production calls and asserts
residual_calls == nfev + jacobians * number_of_unknowns, proving one fit's
accounting. Existing feasible-fit fixed-point and strict polish gates still
apply. Finish official worktree-selecting dev-check after measured follow-up.

### Orthogonal compact-SVD / colored polish handoff — 2026-11-12 (UNMEASURED)

- Parent measured the single-fit cylinder depth1.5/randomness.5/seed3/cap.1
  under concurrent historical load: total132.1103s, fit54.3822s, polish77.7265s;
  257 dense SVDs consumed53.3524s. Jacobian .4242s is not the fit bottleneck.
- Fit now performs Householder QR of the tall Jacobian, applies Q transpose to
  the residual WITHOUT materializing Q, and runs SVD only on square R. This
  preserves singular values/right vectors and the full exact TRF trust-region
  subproblem mathematically; it does NOT form ill-conditioned normal equations.
  Starts, finite differences, objective, rank threshold, 300 nfev cap, secular
  solve and stopping thresholds are unchanged. QR rounding can alter later TRF
  trajectories, so existing frozen SciPy objective/corner parity gates remain
  mandatory. FitProfile.svd_seconds now includes QR plus compact SVD/projection.
- Polish finite differences now use deterministic greedy structural coloring.
  Each corner target/determinant depends on its three planes; each entire
  polygon block conservatively depends on ALL planes incident to its fan;
  bevel rows depend on both neighbors. Colors have disjoint residual supports.
  Simultaneous probes preserve each scalar finite difference, including the
  original h and derivative cutoff; zero fairness suffix is omitted. This
  reduces expensive full-geometry/bounding-square evaluations, not budgets.
  Polygon reversal, inner footprints and bounding-square active-set switches
  are covered by whole-block supports. Frozen domain is essential.
- Removed one duplicate current-x polish evaluation per outer step, retaining
  that iteration's exact report for acceptance (not the best-so-far report).
  Objective, 80 outer/CG steps,16 backtracks and final never-worse guard intact.
- Added qr_reduced_svd_preserves_damped_steps (ill-conditioned tall matrix,
  range-orthogonal residual, four damping scales) and
  colored_polish_matches_every_scalar_probe (real native cylinder at three
  displaced states, EXACT full sparse-column equality including missing rows,
  assertion coloring actually reduces evaluation count). Existing parity,
  strict polish, deterministic and single-fit gates remain unchanged.
- Child ran ONLY container cargo fmt. NO builds/tests/diagnostics, no commits,
  merges or pushes. Parent's running seed1/full96/sweep processes untouched.
  No timing improvement or geometric acceptance is claimed before validation.

Parent commands (HOST; use a NEW isolated target so prior processes/artifacts
remain attributable; run tests/checks sequentially):

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-compact-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --lib qr_reduced_svd_preserves_damped_steps
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-compact-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --lib colored_polish_matches_every_scalar_probe
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-compact-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes identical_input_reference_solver_parity -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-compact-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes fitted_cylinder_polish_removes_reflex_excess_without_new_failures -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-compact-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes single_fit_diagnostic_matches_production_stages -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-compact-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_matrix
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-compact-target/release/examples/c4_matrix cylinder 1.5 .5 3 .1 > runs/logs/C4-compact-cylinder-r05-s3.jsonl
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-compact-target/release/examples/c4_matrix cylinder 1.5 1 1 .1 > runs/logs/C4-compact-cylinder-r1-s1.jsonl
```

Compare objective/corner parity, complete limit reports, stage and profile times
against the recorded single-fit controls, labeling concurrent load. Then run
remaining deterministic/fixed-point gates and official worktree dev-check using
parent's existing selecting recipe; full supervisor96 after targeted gates.

### Parent validation of compact optimizer — 2026-11-12

- Container new unit gates, frozen reference parity, strict polish and single-fit/production equivalence passed: `runs/logs/C4-compact-gates.log`. End-to-end seed independence passed: `runs/logs/C4-compact-determinism.log`.
- Official worktree-selected `make dev-check` exited 0, including 49 Python tests: `runs/logs/C4-compact-dev-check-1211.log`.
- Exact formerly timed-out cylinder controls (depth1.5, cap.1), pinned CPU0, no deadline: r.5/s3 52.746s (fit41.010/polish11.732), r1/s1 47.175s (36.177/10.618), r1/s3 24.019s (14.367/9.353), r1/s6 22.230s (12.546/9.386). Raw `runs/logs/C4-compact-timeout-controls.jsonl`. Earlier single-fit controls were132.110s and116.408s for first two, respectively: about2.50x and2.47x improvement under non-identical concurrent load, not a controlled benchmark. All four complete but remain geometry errors. QR changes floating trajectories; no exact baseline geometry identity claimed.
- Historical double-fit matrix finished96/96: acceptance cylinder26/30 with4timeouts, cone0/30, vase0/30; extra six report-only depths included in raw matrix. Historical geometry failures are not runtime failures. No optimized full-matrix acceptance claim.
- Per-case historical96 plus optimized4 timings: `runs/logs/C4-optimizer-per-case.csv`. Original jitter sweep produced no records before termination; not silently restarted. No merge/commit/push performed.


### Compact acceptance regression isolation — 2026-11-12 (parent execution pending)

- Applied container-only Rust skill; child read source/artifacts and ran ONLY
  container cargo fmt. No builds/tests/profiles, commits or merges. Existing
  `/tmp/vstone-c4-compact-target` and running jobs left untouched.
- Historical `C4-position-only-matrix96.jsonl` versus
  `C4-compact-matrix96.jsonl` proves divergence already exists BEFORE polish.
  At depth1.5/cap.1 cylinder r.5/s7 fitted violation energy changes
  .680589345707343 -> .6872411843081795; r1/s4
  .5007907380998251 -> .5007946693287183; r1/s8
  2.157422869424739 -> 10.138208398848683 and reflex count54 ->55.
  The last case changes fitted whole geometry/domain substantially, not merely
  final tolerance rounding. Historical final violation lists are empty for all
  three, with polish iterations8/32/16; compact final lists contain22/8/54 entries.
  QR preserves the exact-arithmetic TRF problem, NOT floating-point trajectory
  or nonlinear basin. Frozen parity on one fixture cannot certify the matrix.
- Conservative correction: restore original tall nalgebra SVD/projection for
  ALL production/reference fits (no seed/profile special cases, no retries or
  dual solver). Retain single-fit harness, baseline reuse and colored polish.
  This removes the unvalidated numerical fit change instead of modifying
  objectives, limits, starts, stopping criteria, budgets or never-worse guards.
  The QR damped-step unit test remains an algebraic diagnostic only, NOT a
  production trajectory/parity guarantee. No production compact SVD now remains.
- Three independently runnable ignored strict regressions match exact matrix
  params and require successful full-limit results. No timing claims, robust
  passing claim, or full matrix claim until parent executes them. Dense fit may
  cost more: single-fit and colored-polish savings remain; <=60s is UNVERIFIED.
- Parent should first build in a NEW isolated target (suggest
  `/tmp/vstone-c4-dense-colored-target`), leaving compact binary available as
  control. Run tests sequentially, pin single core for case timings:

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-dense-colored-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes historical_cylinder_ -- --ignored --nocapture --test-threads=1
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-dense-colored-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --lib colored_polish_matches_every_scalar_probe
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-dense-colored-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --test planes identical_input_reference_solver_parity -- --ignored --nocapture
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-dense-colored-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_matrix
```

- Capture direct cases `cylinder 1.5 .5 7 .1`, `cylinder 1.5 1 4 .1`,
  `cylinder 1.5 1 8 .1` with new binary, taskset CPU0, same supervisor deadline60.
  Compare fitted planes/corners/whole_tiles/report byte-for-byte against the
  historical artifacts (timings are not geometry). Historical fit wrapper is
  `measurement.fit.Ok`; new fit is `measurement.fit`. With identical fitted
  arrays, compare final report, geometry and polish iteration against historical
  `measurement.result.Ok`, isolating colored polish from fit. If fits match but
  polish regresses, colored-probe scalar equality needs extension at that exact
  fitted state; do not attribute that failure to QR or loosen acceptance.
- Then rerun strict polish, single-fit stage equivalence, feasible fixed-point,
  symmetry independence, official worktree-selected dev-check and supervisor96.
  Existing cone/vase geometry failures remain distinct. Report timeouts honestly;
  do not infer <=60s from historical DOUBLE-FIT timings or compact measurements.

### Four ROW-depth size-band diagnostics — 2026-11-12

- Preserved eb9b3ae and all prior coloring/motion artifacts/jobs; neither promoted.
  New experimental-only `c4_sizeband` snapshots seed/repair modules, sharing native
  Seed/Topology/solver and unchanged spacing/manufacturing checks. Locally clears
  the pentagon feasibility/quota only for vase[6] with zero column changes and no
  heptagon requirement. Production sources/contracts unchanged.
- r0/seed1/depth1.5/target6.5: vase[6]30% FAIL32/48; vase[6]35% FAIL32/48.
  Vase25% cannot seed: native C2 requires column sequence[7,8,7], TWO changes,
  as well as4pentagons. Pentagon-only authorization does NOT bypass that separate
  dislocation requirement; geometry and actual degrees unavailable, not a solver
  failure or passing case. No hidden size relaxation to make it run.
- Cone[5,6,7]25% PASS0/63, no degree5/7 ANYWHERE. Complete final result equals
  historical cone[6]25% result. Physical sites all77 degree6; all sites91degree6
  +14 open-hull degree4. Vase30/35 physical64degree6; all2degree3/14degree4/
  2degree5/78degree6; degree5 are exclusively hull fans, not interior pentagons.
- Exact inputs/full result outputs/commands/summary: MAIN runs/logs/C4-sizeband-*.
  Report including all height-quarter per-type native violations and parent
  worktree-selecting official-check recipe: docs/prototypes/C4_sizeband_report.md.
- Container fmt/clippy and targeted production-cone topology equality PASS.
  No official make dev-check by child; no commit/merge/push. These are native C4
  whole-tile measurements, not production approval or C5/C6 clipped-piece checks.

### Vase[6]30% symmetric depth sweep — 2026-11-12

New C4-depthsweep-prefixed diagnostics/report preserve existing jobs/artifacts. Depth0(explicit zero-limit)/.5/1/1.5: native failed48/48,48/48,45/48,32/48;1.5 reuses identical prior sizeband result. Exact actual rows, tile IDs, violation types, normalized height ranges, quarter bands, internal5/7 counts and topology-equality assertions: docs/prototypes/C4_depthsweep_report.md and runs/logs/C4-depthsweep-summary.json. All seed positions/graphs equal after removing depth; internal5/7=0/0. No shallow pass, no production scaling rule proposed/implemented. New diagnostic example only plus aggregation/docs; existing production/support code untouched. Targeted container build/clippy passed; parent owns official worktree-correct dev-check recipe in report. No acceptance tick, commit, merge, push or old-job changes.

### Conformal vase reproduction (2026-11-12)

Preserved owner A05_vase_proto.py unchanged; added native analytic/conformal diagnostic
and isolated reference penalty optimiser (production optimize unchanged). See
`docs/prototypes/C4_conformal_vase_report.md` and main `runs/logs/C4-conformal-*`.
Manufacturing success is experimental C2/C3 bypass evidence, not production acceptance.
Parent owns official worktree-correct dev-check; not run by this subagent. C5 deferred.
