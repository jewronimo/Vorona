# Controlled C4 depth-vs-position ablations (checkpoint eb9b3ae)

Experimental examples only. No production solver, tolerance, manufacturing limit, size band, or topology-target changes. Existing artifacts/jobs preserved; no commit/push/merge. Container-only Rust skill applied. Official check belongs to parent.

Both cases use exactly existing tight matrix settings: target_size6.5, size_variation.15, allowed_sides[5,6,7], depth1.5, randomness0, seed1, positional cap.1, fixture bevel60, max reflex230, min/max edge1/6, min corner70, thickness.375, min bounding square4, blank12 usable11.5. Profiles `cone_9_to_6` and `vase_bulge`. Complete original input params in each output.

## Three independent arms

- Baseline: native row%3 * D/2 depths, unchanged native topology/solver.
- Color: same UV sites, depths {0,D/2,D}; deterministic DSATUR (saturation, degree, lowest index), minimize incident conflicts with row-color preference, then at most64 ascending-id strict incident-conflict reducing single-vertex recoloring sweeps. Final conflicts explicitly reported; not a claim of proper 3-coloring. Odd-degree interior triangulations obstruct perfect triangular 3-coloring.
- Motion: ORIGINAL ROW depths, original graph kept EXACTLY (including corners); only ordinary interior sites at graph distance<=1 of non-hull degree5/7 can move. Spacing-relaxation field built from original sites, four signed bounded steps ±2%,±5% of target_size, each with up to12 feasibility halvings, first feasible candidate evaluated. Every candidate retriangulated; reject graph changes, any original size-band violation (native spacing oracle, squared residual <=1e-18), and rim crossings. Native final report ranks valid candidates lexicographically by distinct failed tiles then violation entries, original baseline included as fallback; strict improvement required. This is a small bounded search, not a global seed optimum. Maximum local tangent/arc vector norm .325in; actual metric spacing movements recorded per eligible site, including zeros. No combining coloring and motion.

## Interpretation

PASS only when unchanged native fit_and_polish returns Ok with its full native whole domain; no filtering/ignoring tests. This is not C5/C6 clipped-piece certification. Aggregates use native whole tiles, deduplicate failed IDs, and retain repeated violation entries. Near = graph distance<=1 to non-hull degree5/7, auxiliary defects included; far = remainder. Quartiles use anchor surface height. Whole-domain membership can vary with geometry, so changing rates is not a fixed-domain causal estimate. Native best/full geometry, complete reports and timing are retained in every JSON. Motion seconds include baseline/candidate search, other seconds include topology+one solver; not like-for-like per-fit timings. All single-core under potentially concurrent existing load.

Prior vase .75: near27/30, far14/20; exact-zero limit: near30/31, far18/20. Defects were never established sole cause. These ablations test two hypotheses, not proof of universal causation.

## Reproduction (host repository root)

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-ablation-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_ablation
for profile in vase_bulge cone_9_to_6; do
  for arm in baseline color motion; do
    docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-ablation-target/release/examples/c4_ablation "$profile" 1.5 "$arm" > "runs/logs/C4-ablation-${profile}-${arm}.json"
  done
done
```

## Parent official check (not executed by child)

From HOST worktree root; new target deliberately separate from diagnostics and all pre-existing jobs:

```sh
cat > /tmp/vorona-c4-ablation-check.mk <<'EOF'
dev-check: dev-up
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-ablation-official-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
EOF
make -f Makefile -f /tmp/vorona-c4-ablation-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-dev-check.log 2>&1
```

The ordinary base recipe lacks worktree cwd. Official standard nonignored tests do not establish geometric acceptance of failed diagnostic cases.

Aggregate regeneration (no geometry in Python): `python3 runs/worktrees/C4-depth/docs/prototypes/C4_ablation_summary.py` from host repo root. Summary retains near/far per-limit counts and height-band, including near-by-band cross-tabs.

## Measured six-arm results

All six FAIL unchanged native limits. Counts are unique failed / whole tiles.

| Case | Arm | Overall | Near | Far | Total s |
|---|---|---|---|---|---|
| cone_9_to_6 | baseline | 44/61 | 13/20 | 31/41 | 22.474434 |
| cone_9_to_6 | color | 62/63 | 20/20 | 42/43 | 31.323603 |
| cone_9_to_6 | motion | 44/61 | 13/20 | 31/41 | 17.036197 |
| vase_bulge | baseline | 36/51 | 23/31 | 13/20 | 10.827313 |
| vase_bulge | color | 50/51 | 30/31 | 20/20 | 16.691021 |
| vase_bulge | motion | 36/51 | 23/31 | 13/20 | 46.105816 |

Color reduces equal-depth edges vase86→23 (greedy25, two fix-up sweeps), cone108→19 (greedy21, two fix-up sweeps). Conflicts remain: neither is properly colored. Geometry failure rates worsen rather than improve. Whole domain changes on cone61→63.

Motion: vase37/37 eligible sites moved, selected fraction−0.00015625, max actual surface metric displacement0.001015631656in; bounded local vector norm0.325in, achieved selected local vector norm0.001015625in. Native entries105→104 but failed tiles36/51 unchanged; near23/31 and far13/20 unchanged. This is at most one fewer violation entry, NOT successful convergence or demonstrated failed-tile improvement. Four accepted candidates and rejected larger steps are recorded. Cone23 eligible, zero achieved moves: ALL52 candidate steps rejected by exact-graph/spacing/interior conjunction even after12 halvings. No feasible tested motion under this particular field; this is not evidence all admissible motion is impossible. Do not label the cone arm a successful seed optimization. Initial unshrunk experiments are retained as `C4-ablation-{profile}-motion-initial.json`; not included as extra primary arms.

Actual all-site degree5/7 counts: vase7/5 (nonaux5/5), cone4/4 (nonaux3/3). Hull degree4 sites vase12/cone14 are not defects. Graph and corner list preserved exactly for selected arms, so degree targets/counts unchanged; native spacing residual0 for all selected arms. Degree histograms and non-hull defect IDs in JSON.

### Violation entries by proximity and height (not deduplicated)

#### cone_9_to_6 / baseline

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-cone_9_to_6-baseline.json`

- overall: `{"max_corner_angle_deg": 32, "max_edge": 38, "min_corner_angle_deg": 16, "min_edge": 14}`
- near: `{"max_corner_angle_deg": 9, "max_edge": 10, "min_corner_angle_deg": 6, "min_edge": 8}`
- far: `{"max_corner_angle_deg": 23, "max_edge": 28, "min_corner_angle_deg": 10, "min_edge": 6}`

| Height quartile | Failed/whole | Violations by type | Near failed/whole | Near violations by type |
|---|---|---|---|---|
| 0 | 13/13 | `{"max_corner_angle_deg": 8, "max_edge": 20, "min_corner_angle_deg": 1, "min_edge": 6}` | 2/2 | `{"max_edge": 2, "min_edge": 2}` |
| 1 | 13/21 | `{"max_corner_angle_deg": 7, "max_edge": 18, "min_corner_angle_deg": 3}` | 4/8 | `{"max_corner_angle_deg": 3, "max_edge": 8}` |
| 2 | 7/14 | `{"max_corner_angle_deg": 7}` | 2/4 | `{"max_corner_angle_deg": 2}` |
| 3 | 11/13 | `{"max_corner_angle_deg": 10, "min_corner_angle_deg": 12, "min_edge": 8}` | 5/6 | `{"max_corner_angle_deg": 4, "min_corner_angle_deg": 6, "min_edge": 6}` |

#### cone_9_to_6 / color

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-cone_9_to_6-color.json`

- overall: `{"blank_usable": 3, "max_corner_angle_deg": 21, "max_edge": 82, "min_corner_angle_deg": 65, "min_edge": 17, "simple_inner_polygon": 8, "simple_polygon": 5}`
- near: `{"blank_usable": 1, "max_corner_angle_deg": 8, "max_edge": 26, "min_corner_angle_deg": 24, "min_edge": 7, "simple_inner_polygon": 1, "simple_polygon": 1}`
- far: `{"blank_usable": 2, "max_corner_angle_deg": 13, "max_edge": 56, "min_corner_angle_deg": 41, "min_edge": 10, "simple_inner_polygon": 7, "simple_polygon": 4}`

| Height quartile | Failed/whole | Violations by type | Near failed/whole | Near violations by type |
|---|---|---|---|---|
| 0 | 12/13 | `{"blank_usable": 1, "max_corner_angle_deg": 5, "max_edge": 15, "min_corner_angle_deg": 10, "min_edge": 5, "simple_inner_polygon": 3, "simple_polygon": 3}` | 2/2 | `{"max_corner_angle_deg": 2, "max_edge": 3, "min_corner_angle_deg": 2, "min_edge": 1, "simple_inner_polygon": 1, "simple_polygon": 1}` |
| 1 | 21/21 | `{"max_corner_angle_deg": 4, "max_edge": 26, "min_corner_angle_deg": 19, "min_edge": 5, "simple_inner_polygon": 2, "simple_polygon": 2}` | 8/8 | `{"max_corner_angle_deg": 2, "max_edge": 5, "min_corner_angle_deg": 11, "min_edge": 2}` |
| 2 | 14/14 | `{"max_corner_angle_deg": 4, "max_edge": 19, "min_corner_angle_deg": 12, "min_edge": 2}` | 4/4 | `{"max_corner_angle_deg": 2, "max_edge": 9, "min_corner_angle_deg": 3, "min_edge": 1}` |
| 3 | 15/15 | `{"blank_usable": 2, "max_corner_angle_deg": 8, "max_edge": 22, "min_corner_angle_deg": 24, "min_edge": 5, "simple_inner_polygon": 3}` | 6/6 | `{"blank_usable": 1, "max_corner_angle_deg": 2, "max_edge": 9, "min_corner_angle_deg": 8, "min_edge": 3}` |

#### cone_9_to_6 / motion

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-cone_9_to_6-motion.json`

- overall: `{"max_corner_angle_deg": 32, "max_edge": 38, "min_corner_angle_deg": 16, "min_edge": 14}`
- near: `{"max_corner_angle_deg": 9, "max_edge": 10, "min_corner_angle_deg": 6, "min_edge": 8}`
- far: `{"max_corner_angle_deg": 23, "max_edge": 28, "min_corner_angle_deg": 10, "min_edge": 6}`

| Height quartile | Failed/whole | Violations by type | Near failed/whole | Near violations by type |
|---|---|---|---|---|
| 0 | 13/13 | `{"max_corner_angle_deg": 8, "max_edge": 20, "min_corner_angle_deg": 1, "min_edge": 6}` | 2/2 | `{"max_edge": 2, "min_edge": 2}` |
| 1 | 13/21 | `{"max_corner_angle_deg": 7, "max_edge": 18, "min_corner_angle_deg": 3}` | 4/8 | `{"max_corner_angle_deg": 3, "max_edge": 8}` |
| 2 | 7/14 | `{"max_corner_angle_deg": 7}` | 2/4 | `{"max_corner_angle_deg": 2}` |
| 3 | 11/13 | `{"max_corner_angle_deg": 10, "min_corner_angle_deg": 12, "min_edge": 8}` | 5/6 | `{"max_corner_angle_deg": 4, "min_corner_angle_deg": 6, "min_edge": 6}` |

#### vase_bulge / baseline

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-vase_bulge-baseline.json`

- overall: `{"max_corner_angle_deg": 18, "max_edge": 36, "min_corner_angle_deg": 37, "min_edge": 14}`
- near: `{"max_corner_angle_deg": 9, "max_edge": 24, "min_corner_angle_deg": 24, "min_edge": 11}`
- far: `{"max_corner_angle_deg": 9, "max_edge": 12, "min_corner_angle_deg": 13, "min_edge": 3}`

| Height quartile | Failed/whole | Violations by type | Near failed/whole | Near violations by type |
|---|---|---|---|---|
| 0 | 6/7 | `{"max_corner_angle_deg": 2, "max_edge": 6, "min_corner_angle_deg": 3, "min_edge": 4}` | 5/6 | `{"max_corner_angle_deg": 1, "max_edge": 5, "min_corner_angle_deg": 2, "min_edge": 3}` |
| 1 | 14/18 | `{"max_corner_angle_deg": 7, "max_edge": 18, "min_corner_angle_deg": 16}` | 6/10 | `{"max_corner_angle_deg": 2, "max_edge": 12, "min_corner_angle_deg": 8}` |
| 2 | 8/18 | `{"max_corner_angle_deg": 7, "max_edge": 10, "min_corner_angle_deg": 8, "min_edge": 5}` | 5/8 | `{"max_corner_angle_deg": 4, "max_edge": 5, "min_corner_angle_deg": 5, "min_edge": 3}` |
| 3 | 8/8 | `{"max_corner_angle_deg": 2, "max_edge": 2, "min_corner_angle_deg": 10, "min_edge": 5}` | 7/7 | `{"max_corner_angle_deg": 2, "max_edge": 2, "min_corner_angle_deg": 9, "min_edge": 5}` |

#### vase_bulge / color

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-vase_bulge-color.json`

- overall: `{"bevel_cap_deg": 2, "blank_usable": 2, "inner_orientation": 2, "max_corner_angle_deg": 12, "max_edge": 64, "min_bounding_square": 1, "min_corner_angle_deg": 40, "min_edge": 35, "simple_inner_polygon": 11, "simple_polygon": 5}`
- near: `{"bevel_cap_deg": 2, "blank_usable": 1, "inner_orientation": 1, "max_corner_angle_deg": 8, "max_edge": 35, "min_corner_angle_deg": 24, "min_edge": 22, "simple_inner_polygon": 7, "simple_polygon": 2}`
- far: `{"blank_usable": 1, "inner_orientation": 1, "max_corner_angle_deg": 4, "max_edge": 29, "min_bounding_square": 1, "min_corner_angle_deg": 16, "min_edge": 13, "simple_inner_polygon": 4, "simple_polygon": 3}`

| Height quartile | Failed/whole | Violations by type | Near failed/whole | Near violations by type |
|---|---|---|---|---|
| 0 | 7/7 | `{"max_corner_angle_deg": 1, "max_edge": 8, "min_corner_angle_deg": 5, "min_edge": 8, "simple_inner_polygon": 1}` | 6/6 | `{"max_corner_angle_deg": 1, "max_edge": 6, "min_corner_angle_deg": 3, "min_edge": 7, "simple_inner_polygon": 1}` |
| 1 | 17/18 | `{"bevel_cap_deg": 2, "blank_usable": 1, "inner_orientation": 2, "max_corner_angle_deg": 6, "max_edge": 19, "min_bounding_square": 1, "min_corner_angle_deg": 15, "min_edge": 15, "simple_inner_polygon": 7, "simple_polygon": 3}` | 9/10 | `{"bevel_cap_deg": 2, "blank_usable": 1, "inner_orientation": 1, "max_corner_angle_deg": 4, "max_edge": 10, "min_corner_angle_deg": 9, "min_edge": 6, "simple_inner_polygon": 6, "simple_polygon": 2}` |
| 2 | 18/18 | `{"blank_usable": 1, "max_corner_angle_deg": 3, "max_edge": 29, "min_corner_angle_deg": 14, "min_edge": 7, "simple_inner_polygon": 3, "simple_polygon": 2}` | 8/8 | `{"max_corner_angle_deg": 1, "max_edge": 12, "min_corner_angle_deg": 7, "min_edge": 5}` |
| 3 | 8/8 | `{"max_corner_angle_deg": 2, "max_edge": 8, "min_corner_angle_deg": 6, "min_edge": 5}` | 7/7 | `{"max_corner_angle_deg": 2, "max_edge": 7, "min_corner_angle_deg": 5, "min_edge": 4}` |

#### vase_bulge / motion

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-ablation-vase_bulge-motion.json`

- overall: `{"max_corner_angle_deg": 17, "max_edge": 36, "min_corner_angle_deg": 37, "min_edge": 14}`
- near: `{"max_corner_angle_deg": 8, "max_edge": 24, "min_corner_angle_deg": 23, "min_edge": 11}`
- far: `{"max_corner_angle_deg": 9, "max_edge": 12, "min_corner_angle_deg": 14, "min_edge": 3}`

| Height quartile | Failed/whole | Violations by type | Near failed/whole | Near violations by type |
|---|---|---|---|---|
| 0 | 6/7 | `{"max_corner_angle_deg": 2, "max_edge": 6, "min_corner_angle_deg": 3, "min_edge": 4}` | 5/6 | `{"max_corner_angle_deg": 1, "max_edge": 5, "min_corner_angle_deg": 2, "min_edge": 3}` |
| 1 | 14/18 | `{"max_corner_angle_deg": 7, "max_edge": 18, "min_corner_angle_deg": 17}` | 6/10 | `{"max_corner_angle_deg": 2, "max_edge": 12, "min_corner_angle_deg": 8}` |
| 2 | 8/18 | `{"max_corner_angle_deg": 7, "max_edge": 10, "min_corner_angle_deg": 8, "min_edge": 5}` | 5/8 | `{"max_corner_angle_deg": 4, "max_edge": 5, "min_corner_angle_deg": 5, "min_edge": 3}` |
| 3 | 8/8 | `{"max_corner_angle_deg": 1, "max_edge": 2, "min_corner_angle_deg": 9, "min_edge": 5}` | 7/7 | `{"max_corner_angle_deg": 1, "max_edge": 2, "min_corner_angle_deg": 8, "min_edge": 5}` |

### Verification and limitations

Container cargo fmt --check, clippy --example c4_ablation -D warnings, and cargo test --example c4_ablation PASS (one deterministic coloring/site/graph invariant test, 0 ignored, 0 filtered). Raw `runs/logs/C4-ablation-targeted-checks.log`. This targeted test pass is NOT a full geometry pass; every primary diagnostic FAILS. Build logs C4-ablation-build.log and C4-ablation-build-refined.log. The child did NOT run official make dev-check. Parent supplemental recipe above. All production source unchanged, existing checkpoint and diagnostics preserved.
