# C4 vase hex depth sweep

Native experimental results at checkpoint eb9b3ae; not an official make dev-check certification.

| Depth (in) | Status | Deduplicated failed / whole | Violation entries |
|---|---|---|---|
| 0.0 | limits_failed | 48/48 | {'max_edge': 81, 'min_edge': 2, 'min_corner_angle_deg': 1} |
| 0.5 | limits_failed | 48/48 | {'max_edge': 80, 'min_edge': 34, 'min_corner_angle_deg': 24, 'max_corner_angle_deg': 16} |
| 1.0 | limits_failed | 45/48 | {'min_corner_angle_deg': 36, 'min_edge': 3, 'max_corner_angle_deg': 35, 'max_edge': 58} |
| 1.5 | limits_failed | 32/48 | {'max_edge': 32, 'min_corner_angle_deg': 32, 'max_corner_angle_deg': 28} |

## Actual rows (not quarter bands)

Rows are original Seed.row identifiers, including auxiliary support rows. Heights are seed-anchor Y normalized bottom-to-top, clamped by surface.point outside the ends, not optimized corner heights. Row s/u values are retained in native topology JSON. Whole tiles are the native solver domain; 0/0 rows are NOT an acceptance pass. Row depths repeat (row % 3) * depth/2.

### Depth 0.0 in

| Row | Normalized height min–max | Failed / whole | Failed tile IDs | Violation counts/types |
|---|---|---|---|---|
| 0 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 1 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 2 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 3 | 0.097470729320–0.097470729320 | 8/8 | [24, 25, 26, 27, 28, 29, 30, 31] | {'max_edge': 8, 'min_edge': 2} |
| 4 | 0.215590953755–0.215590953755 | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 16} |
| 5 | 0.384600085609–0.384600087759 | 8/8 | [40, 41, 42, 43, 44, 45, 46, 47] | {'max_edge': 16, 'min_corner_angle_deg': 1} |
| 6 | 0.518044629405–0.518044631225 | 8/8 | [48, 49, 50, 51, 52, 53, 54, 55] | {'max_edge': 16} |
| 7 | 0.668544822014–0.668544822913 | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16} |
| 8 | 0.840871813473–0.840871814261 | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'max_edge': 9} |
| 9 | 0.972047702021–0.972047702021 | 0/0 | [] | {} |
| 10 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |
| 11 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |

Supplemental quarter bands, bottom to top: [{"band": 0, "failed": 16, "whole": 16, "violations": {"max_edge": 24, "min_edge": 2}}, {"band": 1, "failed": 8, "whole": 8, "violations": {"max_edge": 16, "min_corner_angle_deg": 1}}, {"band": 2, "failed": 16, "whole": 16, "violations": {"max_edge": 32}}, {"band": 3, "failed": 8, "whole": 8, "violations": {"max_edge": 9}}]

### Depth 0.5 in

| Row | Normalized height min–max | Failed / whole | Failed tile IDs | Violation counts/types |
|---|---|---|---|---|
| 0 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 1 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 2 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 3 | 0.097470729320–0.097470729320 | 8/8 | [24, 25, 26, 27, 28, 29, 30, 31] | {'max_edge': 8} |
| 4 | 0.215590953755–0.215590953755 | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'min_edge': 8, 'min_corner_angle_deg': 8, 'max_edge': 16, 'max_corner_angle_deg': 8} |
| 5 | 0.384600085609–0.384600087759 | 8/8 | [40, 41, 42, 43, 44, 45, 46, 47] | {'max_edge': 16, 'max_corner_angle_deg': 8, 'min_edge': 8, 'min_corner_angle_deg': 8} |
| 6 | 0.518044629405–0.518044631225 | 8/8 | [48, 49, 50, 51, 52, 53, 54, 55] | {'max_edge': 16} |
| 7 | 0.668544822014–0.668544822913 | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16, 'min_edge': 8} |
| 8 | 0.840871813473–0.840871814261 | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'min_edge': 10, 'max_edge': 8, 'min_corner_angle_deg': 8} |
| 9 | 0.972047702021–0.972047702021 | 0/0 | [] | {} |
| 10 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |
| 11 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |

Supplemental quarter bands, bottom to top: [{"band": 0, "failed": 16, "whole": 16, "violations": {"max_edge": 24, "min_edge": 8, "min_corner_angle_deg": 8, "max_corner_angle_deg": 8}}, {"band": 1, "failed": 8, "whole": 8, "violations": {"max_edge": 16, "max_corner_angle_deg": 8, "min_edge": 8, "min_corner_angle_deg": 8}}, {"band": 2, "failed": 16, "whole": 16, "violations": {"max_edge": 32, "min_edge": 8}}, {"band": 3, "failed": 8, "whole": 8, "violations": {"min_edge": 10, "max_edge": 8, "min_corner_angle_deg": 8}}]

### Depth 1.0 in

| Row | Normalized height min–max | Failed / whole | Failed tile IDs | Violation counts/types |
|---|---|---|---|---|
| 0 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 1 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 2 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 3 | 0.097470729320–0.097470729320 | 5/8 | [24, 25, 26, 27, 29] | {'min_corner_angle_deg': 4, 'min_edge': 3, 'max_corner_angle_deg': 3, 'max_edge': 1} |
| 4 | 0.215590953755–0.215590953755 | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 17, 'min_corner_angle_deg': 8, 'max_corner_angle_deg': 8} |
| 5 | 0.384600085609–0.384600087759 | 8/8 | [40, 41, 42, 43, 44, 45, 46, 47] | {'max_corner_angle_deg': 8, 'max_edge': 8, 'min_corner_angle_deg': 8} |
| 6 | 0.518044629405–0.518044631225 | 8/8 | [48, 49, 50, 51, 52, 53, 54, 55] | {'max_edge': 12} |
| 7 | 0.668544822014–0.668544822913 | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 20, 'max_corner_angle_deg': 8, 'min_corner_angle_deg': 8} |
| 8 | 0.840871813473–0.840871814261 | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'min_corner_angle_deg': 8, 'max_corner_angle_deg': 8} |
| 9 | 0.972047702021–0.972047702021 | 0/0 | [] | {} |
| 10 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |
| 11 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |

Supplemental quarter bands, bottom to top: [{"band": 0, "failed": 13, "whole": 16, "violations": {"min_corner_angle_deg": 12, "min_edge": 3, "max_corner_angle_deg": 11, "max_edge": 18}}, {"band": 1, "failed": 8, "whole": 8, "violations": {"max_corner_angle_deg": 8, "max_edge": 8, "min_corner_angle_deg": 8}}, {"band": 2, "failed": 16, "whole": 16, "violations": {"max_edge": 32, "max_corner_angle_deg": 8, "min_corner_angle_deg": 8}}, {"band": 3, "failed": 8, "whole": 8, "violations": {"min_corner_angle_deg": 8, "max_corner_angle_deg": 8}}]

### Depth 1.5 in

| Row | Normalized height min–max | Failed / whole | Failed tile IDs | Violation counts/types |
|---|---|---|---|---|
| 0 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 1 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 2 | 0.000000000000–0.000000000000 | 0/0 | [] | {} |
| 3 | 0.097470729320–0.097470729320 | 0/8 | [] | {} |
| 4 | 0.215590953755–0.215590953755 | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 16, 'min_corner_angle_deg': 8, 'max_corner_angle_deg': 8} |
| 5 | 0.384600085609–0.384600087759 | 8/8 | [40, 41, 42, 43, 44, 45, 46, 47] | {'max_corner_angle_deg': 8, 'min_corner_angle_deg': 8} |
| 6 | 0.518044629405–0.518044631225 | 0/8 | [] | {} |
| 7 | 0.668544822014–0.668544822913 | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16, 'max_corner_angle_deg': 8, 'min_corner_angle_deg': 8} |
| 8 | 0.840871813473–0.840871814261 | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'min_corner_angle_deg': 8, 'max_corner_angle_deg': 4} |
| 9 | 0.972047702021–0.972047702021 | 0/0 | [] | {} |
| 10 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |
| 11 | 1.000000000000–1.000000000000 | 0/0 | [] | {} |

Supplemental quarter bands, bottom to top: [{"band": 0, "failed": 8, "whole": 16, "violations": {"max_edge": 16, "min_corner_angle_deg": 8, "max_corner_angle_deg": 8}}, {"band": 1, "failed": 8, "whole": 8, "violations": {"max_corner_angle_deg": 8, "min_corner_angle_deg": 8}}, {"band": 2, "failed": 8, "whole": 16, "violations": {"max_edge": 16, "max_corner_angle_deg": 8, "min_corner_angle_deg": 8}}, {"band": 3, "failed": 8, "whole": 8, "violations": {"min_corner_angle_deg": 8, "max_corner_angle_deg": 4}}]


## Isolation, provenance, and interpretation

- Machine: `Darwin Spookys-MacBook-Pro-2.local 25.6.0 ... arm64`; `docker ps` confirmed `Vorona-dev` alongside untouched Home Assistant services. Applied vorona-rust-container skill and read AGENTS/PLAN/A01–A03. All native geometry/build/lint ran inside Vorona-dev, CPU0 for measured geometry. No official check, commits, merge, push, stopped jobs, production source edits or manufacturing-limit relaxation.
- Exact input: vase_bulge only, allowed_sides=[6], size_variation=.30, randomness=0, seed=1, target_size=6.5in. Limits unchanged: edge[1,6]in, corner angles[70,230]deg; all other fixture limits unchanged. JSON inputs saved separately for all four cases.
- d1.5 is a byte-for-byte copy of prior C4-sizeband-vase-six-v30.json, NOT a new solver run. It exactly reproduces the prior 32/48 count, including 32 max_edge,32 min-angle,28 max-angle entries. A topology-only construction (no solver) at d1.5 establishes row metadata and is saved as C4-depthsweep-d1.5-topology.json. No production or support source changed relative to that run; new example is the existing sizeband harness plus row/topology instrumentation and topology-only early return.
- All four topologies have EXACTLY identical seed u/v/s, row IDs, auxiliary flags, corners, neighbors and boundary flags, after removing ONLY depth from seeds. Input parameters match after removing ONLY depth. Native summary asserts these comparisons. Thus depth, not a changed lattice or jitter seed, is isolated. Unchanged topology does not imply identical optimized supporting planes or whole membership in general; all four measured domains happen to be the same 48 tiles (IDs24–71).
- Zero is NOT a valid production depth. Its explicit `--zero-limit` route builds the existing symmetric scaffold at .75in, then sets every seed.depth to exactly zero before native fit_and_polish with params.depth=0. No epsilon depth substitution. The topology comparison proves this special route generates the same sites/graph as positive depths here. Positive depths use the normal experimental sizeband construction. The experiment-only C2 pentagon bypass remains guarded: requires mixed-side control feasible, no column changes, no heptagon demand; no other limits bypassed. Production C2 still rejects the vase hex case, even when diagnostic native C4 could pass.
- All96 graph-degree counts: degree3=2,4=14,5=2,6=78,7=0. Internal NONHULL degrees: 74 degree6, **0 degree5,0 degree7** at every depth. Nonaux:64 degree6. The two degree5 sites are hull fans, not internal pentagons. No defect proximity explanation applies to these failures.
- This sweep does NOT support the simple claim that shallow relief cures this vase: zero/.5 fail48/48,1.0 fails45/48,1.5 fails32/48. Native max-angle entries nearly disappear at zero (0 versus28 at1.5), so relief contributes to reflex/over-angle behavior, but edge failures get substantially worse (81 versus32). Min-angle failures also remain (1 atzero,24 at.5,36 at1,32 at1.5). Counts are not a monotone curvature law or a direct curvature measurement. Zero still has8 reflex corners, but no corner exceeds the unchanged230deg cap. No row-by-row curvature values or causal overtilt diagnosis were computed; these findings separate one angle symptom from total feasibility.
- **No shallow depth passes, so no empirically supported scaling rule is proposed or implemented.** A future curvature hypothesis must distinguish Gaussian K=k1*k2 (in^-2) from principal k1,k2 (in^-1). Cylinder and cone away from its apex have K=0 but a nonzero circumferential principal curvature. Therefore a rule required to keep full relief on them cannot simply taper by max|ki|; it would need a dimensionless Gaussian quantity such as |K|*L^2 using a declared local tile length L in inches. Any candidate d_eff=d_requested*f(|K|L^2) would need f(0)=1 and0<=f<=1 and deterministic neighboring-row continuity, but neither f nor a crossover can be calibrated from this failed sweep. No empirical constants invented. This is design guidance, not a validated proposal; uniform sweeps also do not prove local adaptive depth safe because adjacent tiles are coupled.
- In-scope next diagnostic: retain identical topology/input and examine native edge/angle geometry at zero and .5, especially row3 IDs24–31 (zero min_edge on26/27), row5 ID46 (zero min-angle), and persistent long edges rows4–7. At1.0 row3 only5/8 fail; exact IDs/types are in the table and per-tile JSON. These are real strict-limit failures, not something to reclassify or relax. A fixable solver defect is NOT established by this experiment; topology pitch/target6.5 versus max_edge6 and planar fitting may compete. Avoid claiming more without measuring actual corner geometry and optimizer residuals, which are preserved in diagnostic fit/result.

## Artifacts and checks

All inputs, native outputs, stderr, summary, topology-only baseline, and executed geometry argv are under `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-depthsweep-*`. Full native geometry/report is in each output's diagnostic field, not just counts. Prior outputs remain untouched. Summarizer: `docs/prototypes/C4_depthsweep_summary.py` (aggregation only; no Python geometry). New Rust example `core/examples/c4_depthsweep.rs` uses existing sizeband support. Container release build and targeted clippy PASS; log C4-depthsweep-clippy.log. Direct container rustfmt on new example completed. An initial `cargo fmt --example c4_depthsweep` invocation was rejected by cargo-fmt syntax and made no edits; corrected to `rustfmt --edition 2021 examples/c4_depthsweep.rs`.

Build command (container only):
```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-sizeband-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_depthsweep
```
Geometry command form (exact argv also saved in C4-depthsweep-commands.json):
```sh
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-sizeband-target/release/examples/c4_depthsweep vase_bulge 0 .30 --zero-limit
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-sizeband-target/release/examples/c4_depthsweep vase_bulge 0.5 .30
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-sizeband-target/release/examples/c4_depthsweep vase_bulge 1.0 .30
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev /tmp/vstone-c4-sizeband-target/release/examples/c4_depthsweep vase_bulge 1.5 .30 --topology-only
```

### Parent-owned official check (NOT run by child)

From HOST worktree root, write `/tmp/vorona-c4-depthsweep-check.mk`:
```make
dev-check: dev-up
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-depthsweep-official-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
```
Then:
```sh
make -f Makefile -f /tmp/vorona-c4-depthsweep-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-depthsweep-dev-check.log 2>&1
```
Inherited default dev-check checks /src MAIN, not this worktree. Scope since inherited checkpoint: diagnostic example, summary script, report, task notes and new ignored runs/logs artifacts ONLY; preexisting dirty files preserved. No production rule, solver or thresholds changed, no task acceptance marked complete.
