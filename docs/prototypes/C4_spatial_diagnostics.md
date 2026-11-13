# Focused spatial diagnostics (checkpoint eb9b3ae)

Diagnostic example only: `core/examples/c4_spatial.rs`. No solver or contract changes.
All commands below run from the HOST repository root; container cwd explicitly selects the existing C4 worktree. Existing binaries, targets and jobs are preserved. New target: `/tmp/vstone-c4-spatial-target`.

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-spatial-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_spatial
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-spatial-target/release/examples/c4_spatial cone_9_to_6 1.5 > runs/logs/C4-spatial-cone-six-v25-d15.json
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-spatial-target/release/examples/c4_spatial vase_bulge .75 > runs/logs/C4-spatial-vase-d075.json
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-spatial-target/release/examples/c4_spatial vase_bulge 0 > runs/logs/C4-spatial-vase-d0.json
docker exec -w /src/runs/worktrees/C4-depth/core Vorona-dev taskset -c 0 /tmp/vstone-c4-spatial-target/release/examples/c4_spatial vase_bulge 0 --zero-limit > runs/logs/C4-spatial-vase-d0-experiment.json
```

All runs use target_size6.5, randomness0, seed1, cap.1, max reflex230, other limits exactly the checked-in cylinder fixture. Cone explicitly requests allowed_sides[6], variation.25. Vase explicitly requests [5,6,7], variation.15 (unchanged matrix control). Profiles are the checked-in `tests/fixtures/profiles/{cone_9_to_6,vase_bulge}.json`.

## JSON diagnostic schema c4.spatial/1

- `profile`, complete `params`, `status`, `seconds` (wall time under concurrent load).
- `degree_histogram_all`: every site, including auxiliary and incomplete hull fans; hull degrees are NOT polygon side counts.
- `degree_histogram_nonaux`: all physical seed sites, including rim-crossing pieces.
- `defect_tiles`: degree5/7 sites excluding incomplete open hull fans (auxiliary support sites included).
- `spatial.report`: native final/best OptReport, unchanged threshold and residuals.
- `spatial.tiles`: every site, tile id, normalized_height, band, whole, auxiliary, degree, near_5_7, failed and native limit entries.
- `spatial.bands`: whole_tiles denominator, distinct failed_tiles numerator, violation_entries (may repeat tile/limit for separate corners/edges).
- `spatial.proximity`: same counts separately near and far.
- `diagnostic`: native fit, final result or best failed geometry, exact planes/corners/whole selection, reports and fit/polish timings.
- Pre-solver errors have `status` and reason/error without fabricated spatial statistics.

Height is the surface anchor of the seed, h=(surface.point(u,s).y-ymin)/(ymax-ymin), NOT arc fraction or corner height. Four equal bands [0,.25), [.25,.5), [.5,.75), [.75,1]; final endpoint included. Auxiliary heights are clamped for the row table only; aggregates count native final WHOLE tiles exclusively.

Near5/7 means graph distance <=1 (itself or a direct Delaunay neighbor) to a non-hull degree5/7 site; NOT Euclidean distance. Rates are unique failed whole tiles / all whole tiles within the same group. Empty denominator means undefined, never 0% or a passing claim. Failures are native report entries (>1e-7 normalized hinge); failed tiles are deduplicated, entries are not. Compare rates, not raw totals; localization is correlation, not proof of causation. Check excluded rim/support domains before extrapolating.

PASS means production fit_and_polish returns Ok and native violations are empty on its whole domain; it is NOT C5/C6 validation or proof for clipped rim pieces. ERROR with best geometry remains a failed case, even if a previous fitted report differs.

Depth0 is rejected by the production contract (A05 settings require >0, seeds::place enforces it). The separate `--zero-limit` run is explicitly NON-PRODUCTION: builds the identical vase UV topology using depth.75, sets every seed depth to EXACTLY zero, then invokes the unchanged production plane fitter/polisher with params.depth0. No epsilon, changed tolerances, or claimed contract pass. Its result tests only the mathematical limiting case, not normal CLI acceptance.

## Parent official verification (worktree selected)

From HOST worktree root, write a supplemental Makefile outside the repo:

```sh
cat > /tmp/vorona-c4-spatial-check.mk <<'EOF'
dev-check: dev-up
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-spatial-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
EOF
make -f Makefile -f /tmp/vorona-c4-spatial-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-spatial-dev-check.log 2>&1
```

The parent owns this official check. The base dev-check omits -w and would check /src instead. No merge, push, restart, or new commit requested/performed.

## Measured results (2026-11-12)

| Case | Status | Whole failed / measured | Seconds |
|---|---|---|---|
| Cone [6], variation25%, r0, depth1.5 | PASS | 0/63 | 14.454 |
| Vase [5,6,7], variation15%, r0, depth.75 | FAIL | 41/50 | 11.103 |
| Vase depth0 normal production | INVALID INPUT | not evaluated | — |
| Vase exact depth0, explicit limiting experiment | FAIL | 48/51 | 10.259 |

Cone: ALL105 sites comprise degree6=91 and open-hull degree4=14. All77 nonauxiliary sites are degree6; no degree5/7 occurs ANYWHERE, including auxiliary sites. Final63 whole tiles pass native limits;42 reflex corners; corner signed distance min−2.62847, median+.00376851, max+1.06583in (reported, not constrained).

Vase.75: lower-to-upper quarter-band failed/whole counts 6/6,18/18,9/18,8/8; entry counts8,48,24,29. Near5/7 27/30=90%, far14/20=70%: failures are more common near defects but not confined there. Entries: min angle42, max reflex30, max edge21, min edge16 (109 total);62 polish iterations; normalized violation sumsq45.99316.

Exact-zero experiment: bands7/7,17/18,16/18,8/8; entries21,35,28,24. Near30/31=96.77%, far18/20=90%; broad failure, not a local-defect-only effect. Entries max edge65, min edge21, min angle21, max reflex1 (108 total);49 iterations; energy87.06860. Whole selection differs from depth.75 (51 vs50), so these energies/rates are not fixed-domain causal estimates.

Both vase runs: physical sites degree5=5, degree6=53, degree7=5; all sites additionally include auxiliary support sites and open hull fans. Depth-zero uses the exact .75 scaffold UV topology, only depths reset. A production input validity failure must not be labeled a solver convergence failure or a passing zero-depth run.

Artifacts are in MAIN repo `runs/logs/C4-spatial-*`, not a nonexistent worktree runs/logs directory. Build/execution logs are `C4-spatial-build.log` and `C4-spatial-execution.log`; formatting/clippy log `C4-spatial-static-check.log`. Official parent verification remains separate.
