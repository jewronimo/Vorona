# C5 — Rims, stone pieces, bevels

**Owner:** CORE · **Depends on:** C4 · **Unblocks:** C6
**Read first:** docs/GEOMETRY.md §5–6, SPEC R-RM-1,2, R-ST-1..5

## Goal
Turn planes into closed stone pieces: outer face, inner face, side quads, bevel per edge.

## Deliverables
- `core/src/rims.rs` — `level` clipping and `whole` removal at each open end.
- `core/src/pieces.rs` — outer/inner polygons (shared corners), side faces, `edges[]` with
  `kind`, `neighbor`, `length`, `bevel_deg`, `side_normal`; `corner_angles_deg`, `face_depth`,
  `bounding_square`; fills CoreOutput `pieces` (temporary ids by seed index until C6).

## Acceptance — A05 owner mapping
- [ ] A1 `analytic_ring12_offsets_bevels_and_joint_quads`: analytic geometry and half-angle bevels, planar shared joint quads (R-ST-2/3).
- [ ] A2 `thickness_setting_and_reflex_offset`: configured thickness including thicker stone; simple reflex face supported, inner orientation preserved (A05).
- [ ] A3 `collapsed_thick_offset_returns_failure`: collapsed inner polygon explicitly fails, never panics (A05 R-VA-4).
- [ ] A4 `level_saw_and_sliver_are_explicit`: horizontal outer AND inner rim edges; real minimum enclosing square, not diameter (R-RM-1).
- [ ] A5 `placement_precedes_rejection`: choose cut height before cutting to avoid all undersized partials; fallback reports every rejected tile (owner “no slivers: shift pattern before cutting”, R-RM-2).
- [ ] A6 `disconnected_reflex_clip_rejected`: disconnected reflex half-space result explicitly rejected before solid construction; whole removal gives square exposed edges (A05 / R-ST-5).
- [ ] A7 `topology_adapter_metadata_and_cache`: deterministic topology/shared-corner adapter and CoreOutput geometry metadata, configured thickness, volume sanity and explicit pending C6/C7 validation.

## Agent notes

### First bounded increment — 2026-11-13 (not merge-ready)

- Clean starting branch task/C5-pieces at merged C4 1e1d6ed; no inherited edits.
- Added an internal geometry API, not wired to generate/CoreOutput: explicit
  face/side-plane input, inward constant-thickness offset, joint bisectors,
  bevels and closed side quads. Reflex/simple/orientation/collapse guards return
  structured errors rather than panic. No schema or Python changes.
- Added level saw-cut kernel with outgoing side provenance and explicit Outside /
  Sliver / Kept results. Diameter guard rejects definitely undersized partials;
  it is ONLY an early guard, NOT full min_bounding_square compliance. Row
  repositioning / avoidance of all slivers is pending, not silently waived.
- Analytic ring12 fixture tests cover inward 3/8 inch offset, 15-degree joints,
  planar quads and exact shared joint geometry; level-cut / sliver / collapsed
  offset regressions added. Parent owns official certification.
- Next increment: topology adapter, canonical shared-corner cache, robust reflex
  clipping (disconnected components must reject/split), minimum oriented square
  and rim-row placement, whole-removal adjacency/square cuts, face depth / corner
  angles / bounds and final CoreOutput wiring. Current generate unchanged.
- C5 requires Claude review before merge. No checklist marked complete; no
  official dev-check/acceptance, commit, merge or push by this child.

Parent-owned official check (host, exact worktree):

```sh
cd /Users/spooky/Projects/Vorona/voronoi-stone/runs/worktrees/C5-pieces
mkdir -p runs/logs
set -o pipefail
make dev-check 2>&1 | tee runs/logs/C5-dev-check.log
```

Current Makefile resolves the worktree with scripts/container_workdir.py and
executes pip install --no-deps -e . && make check in
/src/runs/worktrees/C5-pieces inside Vorona-dev. No supplemental Makefile needed.
Acceptance remains parent-owned later; do not rerun C4 for this handoff.

Targeted development evidence (not official certification):
`docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c5-target -w /src/runs/worktrees/C5-pieces/core Vorona-dev cargo test --test pieces`
compiled successfully and reported **3 passed, 0 failed**, including analytic
ring12, horizontal saw/sliver signaling, and collapsed offset failure.
Container cargo fmt completed. Full clippy/dev-check/acceptance not executed.

### Full C5 implementation follow-up — 2026-11-13 (parent gates pending)

The earlier bounded-increment omissions above are historical, superseded here.
Preserved all original uncommitted work; branch remains task/C5-pieces. Applied
vorona-rust-container skill; all Rust commands in Vorona-dev only.

- FIRST replaced acceptance with seven named A05/owner-mapped regressions.
- Real minimum enclosing square: reuses C4 exact support-change/analytic candidate
  enumeration after orthonormal projection into the face plane. No diameter proxy.
- Before cutting, choose each level cut height deterministically, requested height
  first, then 128 inward samples over one target pitch (capped at profile-height/4).
  Select fewest failed partials, then least displacement. Bottom is considered
  before top, and top placement evaluates bottom-clipped geometry. This is a
  bounded deterministic search, not a proof no feasible continuous placement exists.
  Fallback rejects only after placement search; all outside/whole/sliver/invalid
  removals are reported by seed id. No setting or manufacturing tolerance relaxed.
- Reflex disconnected half-space results explicitly reject at clip entry, rather
  than later bridged-polygon validity. Inner collapse still explicitly fails.
- Topology adapter uses C4 fan provenance for outgoing neighbor edges; whole
  removal exposes square sides. Canonical shared outer/inner corner cache solves
  sorted face/rim constraints exactly once; no quantized-coordinate matching.
- CoreOutput generate path now calls C3/C4/C5. C6/C7 explicitly pending, status
  failed_validation, temporary seed ids, blank zero / identity layout placeholders;
  this is NOT a claim of completed validation or nesting. Every removed polygon
  appears in C5 check failures; expected outside support removal is distinguished.
- Implemented owner-approved A05 per-piece fields with optional legacy-fixture
  compatibility: seed_depth, max_corner_distance, reflex_corners. Both schemas and
  INTERFACE updated; mapping recorded in docs/requests/002-c5-a05-metadata.md for
  owner review / SHELL consumption. No Python edits.
- Seven regressions include analytic ring12 shared quads/bevels, reflex/thicker
  offset, collapse, horizontal rims, actual square-vs-diameter counterexample,
  pre-cut placement on adjacent tiles, explicit disconnected U polygon, cube
  topology/shared-cache byte equality and computed closed-mesh volume, plus whole
  removal/square exposed edges. Contract determinism test now checks generated
  C5 output rather than historical not-implemented status.

Parent exact official recipes (HOST, after child targeted jobs settle):

```sh
cd /Users/spooky/Projects/Vorona/voronoi-stone/runs/worktrees/C5-pieces
mkdir -p runs/logs
set -o pipefail
make dev-check 2>&1 | tee runs/logs/C5-dev-check.log
make acceptance 2>&1 | tee runs/logs/C5-acceptance-driver.log
```

Acceptance preserves its own raw timestamped log tree. Parent must inspect every
failure and report the existing owner-accepted cylinder depth1.5/r0.5/seed3 timeout
as incomplete, never passed. Owner/Claude review before merge; child did not
commit, merge or push. Unchanged C6/C7 remain separate tasks.

Targeted child evidence (development only, NOT official certification):
- `runs/logs/C5-targeted-pieces.log`: seven passed, zero failed, including computed
  mesh-volume and bounded-search rejection fallback assertions.
- `runs/logs/C5-targeted-clippy.log`: cargo clippy --all-targets -- -D warnings passed.
- `runs/logs/C5-schema.log`: historical fixture roundtrip/schema passed with the
  optional A05 reports omitted byte-identically.
- Release `contract::deterministic_and_errors` passed (1/1, 36.87s for TWO complete
  generation calls). Captured tool output, not a complete raw file log. Does not
  certify unchanged production C4 acceptance budgets.
- Final actual cylinder output preserved in `runs/logs/C5-cylinder-output.json`:
  98 pieces, 15 partials, rim heights [0,60], smallest partial enclosing square
  5.0881242987694435 in versus unchanged 4in minimum. 46 explicit removals:
  31 open auxiliary hull fans, eight outside top and seven outside bottom.
  HISTORICAL CLAIM INVALIDATED by blocking seam review: checking only references
  among kept pieces missed 34 null-neighbor in-band edges and missing physical
  tiles. This output has holes (bottom ~320deg / top ~278deg coverage), and is
  NOT closure evidence. Zero reported solid/clip/sliver failures was insufficient.
  New metadata present on every piece. Generated output validated with container
  Python jsonschema against the amended output schema (PASS). Build stderr in
  `runs/logs/C5-cylinder-build.log`. This is ONE targeted symmetric fixture, not
  the official production matrix or general proof of placement feasibility.


### Blocking seam review repair — 2026-11-13 (NOT CERTIFIED)

Applied vorona-rust-container; machine Darwin arm64, docker ps lists Vorona-dev.
All inherited dirty changes and running acceptance jobs/executables/logs preserved.
No commit, merge or push. Parent owns all official acceptance.

**Actual root cause:** not demonstrated Delaunay wrap disagreement. The quotient
used floating centroid membership in `[0,2*pi)` to select a triangle lift.
For the symmetric seam triangle [0,9,17], the central centroid is
-1.6653345369377348e-16 (excluded), while its translated centroid rounds to
exactly 6.283185307179586 (also excluded). Actual triangles were thrown away,
leaving singly counted interior edges. `boundary` then suppressed real physical
fans and C4 measurements. No limit or solver adjustment repairs this omission.
Integer lift ownership (smallest original vertex index's shift == 0) replaces
centroid ownership. Interior single-count edges now hard-error; only first/last
padding-row boundary edges are permitted. Exact seam and shifted-cut regression
checks all ordinary degrees and exactly 18 outermost auxiliary boundary sites.

Targeted before/after logs:
- `runs/logs/C5-seam-before.log`: 47 single-count edges, including in-band seam.
- `runs/logs/C5-seam-anchor.log`: 18 single-count edges, exclusively padding rims.
- `runs/logs/C5-closure-targeted.log`: 144 sites, 18 boundary, 126 complete fans,
  90 measured complete physical tiles, zero fitted C4 violations; 108 kept pieces;
  36 removals ONLY 18 outermost open support fans + 18 fully outside polygons.
  Native rim coverage [6.283185307179585,6.283185307179586] radians; 306 paired
  joints. Former output had 98 kept and 31 falsely labeled open support fans.

New mandatory native closure guard in assembly AND both C4 matrix harnesses:
null neighbors only on actual horizontal rims; reciprocal reverse-oriented
geometrically matching joint edges; no folds under radial projection anywhere
in each polygon (affine normal/radial positivity at vertices); positively oriented
faces; exact circular rim interval union without gap/overlap, 360deg within
1e-8 radians. Joint endpoint tolerance 1e-7 inches. These oriented mesh/degree
conditions certify full-band coverage without spot sampling. All in-band
whole/sliver/invalid/solid removals hard-fail. A whole-rim pattern that leaves
physical-band gaps is now a failure, not a successful square-edged rejection.

The old cube adapter fixture is NOT a revolution band. Its previous successful
assembly/metadata assertion is superseded with explicit closure rejection,
including whole-removal rejection. Analytic solid kernels remain independently
tested. `core/tests/closure.rs` certifies the full ring fixture then fault-injects
missing tile, null in-band edge and mismatched endpoint: all must fail.

Targeted container evidence (not official): final closure+pieces 8/8 passed;
exact seam/rotated cut regression 1/1 passed; default all-target clippy passed.
Logs: C5-closure-kernel-tests-final.log, C5-closure-tests.log,
C5-seam-clippy-final.log. Full C4 matrix and C5 generation/acceptance MUST be
rerun by parent: old C4 numbers excluded the skipped seam polygons.

Exact parent recipes, from this worktree on host, AFTER existing jobs settle:
```sh
make dev-check
make acceptance
```
Preserve raw logs using the existing host acceptance driver. Do not overwrite
or interrupt the older immutable acceptance executable. A zero targeted result
is not certification and does not supersede owner/Claude review before merge.

Follow-up targeted canonical evidence:
`runs/logs/C5-production-closure-case.json` is the ACTUAL canonical production
matrix harness (required feature c4-experiments), one cylinder depth1.5/r0/seed1
case, `passed=true`, closure Ok with 306 paired joints and both full rim circles.
`runs/logs/C5-seam-feature-clippy.log` verifies feature-enabled all-target clippy.
Initial omitted-feature invocation was rejected by Cargo and then corrected;
no acceptance run was interrupted/replaced. A7 cache coverage is now tested on
an actual physical cylinder (108 pieces, all metadata, byte-identical repeated
assembly and exact shared OUTER AND INNER joint endpoints), replacing cube-only
positive coverage; cube remains a separate explicit negative closure test.

### Configured-thickness barrier / seam-count audit — 2026-11-13 (official gates PENDING)

Preserved dirty C5/seam work; no commit/merge/push, official dev-check or acceptance.
Applied vorona-rust-container; raw machine proof: Darwin arm64; docker ps includes
Vorona-dev (other containers untouched).

- Added shared `MIN_INNER_EDGE = 0.001 in` (one mil), separate from and NOT a
  relaxation of configured outer min_edge. C4 uses signed inner-edge projection
  onto its corresponding outer direction: positive minimum simultaneously excludes
  reversed edges and collapsed edges. Existing inner simplicity and orientation
  guards remain. C5 build applies the same minimum (normalized 1e-7 numerical
  report tolerance, i.e. 1e-10 inch). Thickness, outer edges, angles, bevel cap,
  blank and all other settings unchanged. Production penalty adds a continuous
  restoring term starting 0.02in before this minimum, not a relaxed validity rule.
- Crucial reproducer finding: cone r1/seed9 S000025 is NON-auxiliary but PARTIAL,
  excluded from whole-tile C4 metrics. A first whole-only edge barrier still
  produced `CollapsedOffset` (raw C5-thickness-cone-seed9.json). Thus the three
  inner-validity checks now also protect complete physical partial fans BEFORE
  clipping; all OTHER C4 metrics retain original frozen-whole membership. The
  production polish residual includes these physical partial inner edges too.
  This explicitly extends thickness safety, not the historical whole-only count.
- Final native production reproducer `cone_9_to_6 1.5 1 9` at thickness .375:
  selected configured depth1.5, 60 fixed-budget evaluations, zero violations,
  score0; paired joints216, rim circles 6.283185307179586 and
  6.283185307179585 radians. Single-core runtime10.645530838s. Signed corner
  distance min/median/max -2.9291152709243513 / .15171028869351655 /
  1.3227574267331592in; reflex42. Raw JSON is final line of
  `runs/logs/C5-thickness-cone-seed9-v2.log`. This is one measured case, NOT matrix
  certification. Initial whole-only failed result intentionally preserved.
- Frozen vase count audit: N8/D2.5 **39 -> 48**, with owner-requested source
  comment identifying nine restored seam measurements. N10/D1.5 **80 -> 80**,
  N10/D2 **80 -> 80**, N10/D2.5 **80 -> 80**. Four-case targeted regression
  passed in72.74s (initial barrier version); final-version rerun recorded separately
  in `C5-thickness-final-targeted.log`. Count changes reflect seam recovery,
  never tolerance relaxation.
- Runner now distinguishes owner-approved cylinder configured-depth1/r0/seed1
  report-only60s timeout (`accepted_report_only_timeout`) from the unchanged
  acceptance depth1.5/r.5/seed3/90s `accepted_known_timeout`. Both remain
  passed=false, incomplete, WITHOUT fabricated geometry/closure measurements.
  Other missing measurements/infrastructure errors still block. Every completed
  native matrix invocation still attempts mandatory C5 closure; absent selected
  geometry explicitly reports closure Err, never a closure pass. No full matrix
  rerun performed. Isolated mock-process runner semantics test (NOT numerical
  evidence) verifies only these two timeout exceptions yield exit0 and adding a
  cone report-only timeout yields exit1: C5-runner-timeout-semantics.log.
- Added positive inner minimum regression plus ignored release cone full selector /
  assembly / closure / every-inner-edge regression. Parent owns official gates
  and owner/Claude review. Checklist remains pending.

Exact targeted commands from this worktree (all Rust inside Vorona-dev):
```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c5-thickness-target -w /src/runs/worktrees/C5-pieces/core Vorona-dev sh -c 'cargo fmt && cargo build --release --features c4-experiments --example c4_production_matrix'
docker exec -w /src/runs/worktrees/C5-pieces/core Vorona-dev sh -c 'taskset -c $(python -c "import os; print(min(os.sched_getaffinity(0)))") /tmp/vstone-c5-thickness-target/release/examples/c4_production_matrix cone_9_to_6 1.5 1 9'
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c5-thickness-target -w /src/runs/worktrees/C5-pieces/core Vorona-dev sh -c 'cargo test --release --test planes all_four_accepted_vases_use_guarded_production_polish -- --ignored --nocapture && cargo test --release --test planes cone_seed9_configured_thickness_requires_valid_inner_edges -- --ignored && cargo test --release --test pieces --test closure && cargo clippy --all-targets --features c4-experiments -- -D warnings && cargo fmt --check'
```

Final-version targeted result (raw `runs/logs/C5-thickness-final-targeted.log`):
four-vase ignored regression PASS (69.22s), cone selector/closure/inner-edge
regression PASS (10.15s), closure1/1 and pieces9/9 PASS (14.20s for pieces),
feature-enabled all-target clippy and fmt --check PASS; host git diff --check PASS.
No official gate executed; these are targeted development checks only.

### Parent official thickness-barrier gate — blocked

`make dev-check` failed (exit 2). Raw log: `runs/logs/C5-1826.log`; exit sidecar: `runs/logs/C5-1826.log.exit`.

Rust unit regression `planes::polishing_tests::colored_polish_matches_every_scalar_probe` fails at column 54/displacement 0: colored probe omits the new inner-edge residual rows present in the scalar probe. No acceptance rerun or commit performed after this failure. Requires CORE correction and fresh official gates. Current parent tool catalog exposes neither spawn nor peer-session send; CLI delegation is explicitly prohibited by owner, so no CLI workaround was attempted.


### Inner-residual coloring repair — 2026-11-13 (parent gates still pending)

Preserved every inherited dirty file. Used vorona-rust-container skill; all Rust
commands ran in Vorona-dev. No official gates, commits, merges or pushes.

Root cause is stale residual ROW OFFSETS, not a missing fan-plane union: the
existing support already includes every triangle's three neighboring normals
needed by the shared bisector inner corners. Each n-corner polygon now emits n
additional signed inner-edge rows. Its correct block size is n²+2n+3 (5n
edge/angle rows + 3 size/orientation + n(n-3) inner/outer intersection rows),
not n²+n+3. Updated this symbolic dependency stencil before recoloring; unchanged
scalar-probe regression passes all columns at all three displacements. No test,
solver objective, thickness or limit changed in this repair. PolishPattern is
cfg(test); production uses its existing independent reference penalty solver.

**MIN_INNER_EDGE = .001 inch is a validity guard ONLY, not an owner-approved
manufacturing minimum.** Owner may choose a real manufacturing minimum later.
Existing outer min_edge and all limits remain unchanged.

Targeted development evidence: runs/logs/C5-coloring-targeted.log: unchanged
colored_polish_matches_every_scalar_probe PASS (1/1, 2.24s), feature-enabled
all-target clippy PASS, fmt --check PASS. Host git diff --check PASS.

Exact targeted command (host worktree):
```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c5-thickness-target -w /src/runs/worktrees/C5-pieces/core Vorona-dev sh -c 'cargo test --lib colored_polish_matches_every_scalar_probe && cargo clippy --all-targets --features c4-experiments -- -D warnings && cargo fmt --check' > runs/logs/C5-coloring-targeted.log 2>&1
```

Representative barrier runtime comparison (not matrix certification): same
Vorona-dev, release production-configured-first-v3 harness, identical complete
params verified programmatically, one allowed CPU. Baseline AFTER seam/closure
fix but BEFORE thickness barrier: acceptance-20261114T005859Z-37869/matrix96.jsonl.
Corrected raw output/build log: runs/logs/C5-coloring-runtime.log. Machine-readable
comparison: runs/logs/C5-coloring-runtime-comparison.json.

- Cylinder depth1.5/r0/seed1: 14.865851133s -> 14.693875339s (-1.16%);
  both complete closure, 306 paired joints, both complete rim circles.
- Cone depth1.5/r1/seed9: 9.282464837s -> 10.579368005s (+13.97%);
  baseline closure FAILED CollapsedOffset S000025; corrected closure PASS,
  216 paired joints and both complete rim circles. Both selected depth1.5.

One sample per case: includes barrier cost AND changed solver trajectory; cannot
isolate pure per-evaluation overhead or establish a statistically significant
speedup. Load/cache/toolchain history uncontrolled; cylinder difference is
noise-scale. The old cone is NOT a successful comparable geometry baseline.
This is runtime evidence for two cases, not global budget or closure evidence.

Exact corrected benchmark command (host worktree, compilation excluded from the
harness's total_seconds):
```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c5-coloring-runtime-target -w /src/runs/worktrees/C5-pieces/core Vorona-dev sh -c 'cargo build --release --features c4-experiments --example c4_production_matrix && cpu=$(python -c "import os; print(min(os.sched_getaffinity(0)))"); for args in "cylinder 1.5 0 1" "cone_9_to_6 1.5 1 9"; do taskset -c "$cpu" /tmp/vstone-c5-coloring-runtime-target/release/examples/c4_production_matrix $args; done' > runs/logs/C5-coloring-runtime.log 2>&1
```

Parent owns fresh official gates from this exact worktree:
```sh
cd /Users/spooky/Projects/Vorona/voronoi-stone/runs/worktrees/C5-pieces
set -o pipefail
make dev-check 2>&1 | tee runs/logs/C5-coloring-dev-check.log
make acceptance 2>&1 | tee runs/logs/C5-coloring-acceptance-driver.log
```
Inspect ignored-release AND all matrix failures. Report 96 attempted separately
from completed closure counts. The owner-accepted cylinder depth1/r0/seed1
report-only 60s timeout and cylinder depth1.5/r.5/seed3 90s timeout remain
INCOMPLETE, passed=false, never closure-certified. Do not assume 94 completed
until actual logs establish it; any additional timeout/failure blocks. No task
closure or official certification claimed here.
