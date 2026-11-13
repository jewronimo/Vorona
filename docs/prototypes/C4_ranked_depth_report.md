# Owner-authorized ranked production depth iteration (2026-11-12)

Supersedes the exhaustive-polish policy in the earlier timeout diagnosis. Owner explicitly authorized fit-only screening and a deterministic ranked shortlist; this is NOT an exhaustive-minimal-depth search.

## Policy and unchanged guards

- Native C2/C3, then **up to4** ascending configured+ k×0.25 inch candidates, clipped by the existing physical-extent ceiling. Fixed window is a deterministic runtime policy, not a feasibility theorem. Thus candidates beyond configured+0.75 inches may remain unscreened.
- Each candidate runs the unchanged native reference fit ONLY. Exact complete fit geometry/report is retained in `attempts[].screened`; fitted tilt vectors retained internally.
- Rank by `limit_violation_sum_sq.total_cmp`, then native violation-entry count, then ascending candidate index. Best2 receive existing guarded reference penalty polish. Reconstructing native Problem reuses EXACT fitted tilts and baseline, without a second fit. No alternate initialization, warm start, time-based solver stopping, manufactured tolerance, or solver-budget change.
- Both shortlisted candidates are evaluated. Selected depth is the shallower successful shortlisted depth, if any. `attempts[].polished` and `optimizer` distinguish actual polished evidence from fit-only geometry. No fit-only failure is mislabeled a polished failure. Screen construction errors remain explicit diagnostics.
- All original native limits, fixed target size, frozen-domain/rim safety and never-worse guard remain. No global smallest feasible polished depth guarantee. An unsuccessful shortlist is not full physical-grid exhaustion or global impossibility. Output says so explicitly.
- Native C3 chart cache reuses depth-independent placement/repair and applies existing deterministic row offsets for each candidate.

## Matrix settings

`c4_production_matrix`: cylinder fixture unchanged (target6.5,size_variation.15,[6]); cone[6]/.25; vase[6]/.30. Randomness/depth/seed arguments unchanged. Measurement harness `production-ranked-depth-v2`. New `C4_ranked_depth_matrix_runner.py` preserves the original96case/deadline policy and adds a distinct harness label. Old runner and immutable target remain untouched.

## C2 false rejection separated from manufacturing feasibility

Old C2 integrated curvature into a mandatory interior5/7 quota even for a SINGLE sampled-profile-admissible periodic wrap. That rejected vase[6]/30% before native C3/C4 despite earlier native conformal experiments needing no interior pentagons. For an open annular chart a constant wrap can absorb taper/bulge within the requested width band; curvature alone does not force those combinatorial defects.

C2 now sets curvature defect estimates and average correction to zero ONLY when EVERY sampled size interval is nonempty and native interval projection requires zero column changes. Variable-wrap/dislocation estimates, empty intervals, small-radius, waist, size, side balance checks remain. Native C3 still enforces actual side counts, spacing/average bands and repair cap. C4 still certifies all manufacturing limits; C2 success alone is NOT production-pass.

Taper/bulge rejection text searches [6] at every whole percent5..50 and emits `[6] with size variation ≥ X%` only when native estimate passes, marking whole-percent-grid resolution and requiring C3/C4. No guessed25/30 constant in advice, no input override, no schemas/interface changes. Existing coarser5%-grid same-set suggestion remains; the more precise [6] advice is a verified sufficient threshold, not an exact continuous threshold.

## Verification and artifacts

- MAIN `runs/logs/C4-ranked-final-unit.log`: final four-screen injected ranking/no-double-fit/data-flow/order/cache equality tests2PASS (one native control intentionally ignored there).
- MAIN `runs/logs/C4-ranked-feasibility-depth.log`:3public depth tests +15feasibility tests PASS. New threshold test checks cone/vase[6] requested bands pass C2, no mandatory defects, suggested whole-percent threshold passes and previous percent fails.
- MAIN `runs/logs/C4-ranked-clippy.log`: lib/tests/production-example clippy PASS; container rustfmt PASS.
- MAIN `runs/logs/C4-ranked-build.log`: isolated release production example build PASS.
- MAIN `runs/logs/C4-ranked-reuse-gate.log`: release native screen+reuse polish equals direct production optimize byte-for-byte,1PASS (36.69s).
- Actual target6.5 C2 advice: cone[6] ≥25%, vase[6] ≥28% on whole-percent grid, not guessed constants. Native CLI input/output artifacts MAIN `runs/logs/C4-ranked-advice-*-{input,output}.json`. C2 advice is not C4 acceptance.
- Preliminary EIGHT-screen controls (not final window): cone r0/s1 PASS depth1.5,61.642s; vase r0/s1 PASS depth1.75,27.074s. Their reports are MAIN `runs/logs/C4-ranked-{cone_9_to_6,vase_bulge}-r0-s1.json`. Because cone exceeded60s, final deterministic window reduced to FOUR, still keeping the same top2 for these controls. Final-window controls are separate `C4-ranked-final-*-r0-s1.json`; see appended outcome.

Parent owns official worktree-selected dev-check and full matrix. Previous official check predates these edits.

## Exact parent recipe (HOST)

All Rust runs inside Vorona-dev. NEW target preserves old immutable production artifacts:

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-ranked-depth-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_production_matrix
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-ranked-depth-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --lib depth::tests -- --test-threads=1
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-ranked-depth-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --test depth --test feasibility -- --test-threads=1
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-ranked-depth-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --lib screened_fit_reuse_is_identical -- --ignored --test-threads=1
docker exec -w /src/runs/worktrees/C4-depth Vorona-dev taskset -c 0 python docs/prototypes/C4_ranked_depth_matrix_runner.py /tmp/vstone-c4-ranked-depth-target/release/examples/c4_production_matrix --timeout 60 > runs/logs/C4-ranked-matrix96.jsonl
```

Run parent-owned official worktree-selected dev-check before certification. Do not relabel previous full matrix as this iteration's evidence.

## Risks

Heuristic score may rank a poorly polishable fit above a repairable one;4screen/2polish caps can miss deeper or unshortlisted successes. Dense tall SVD remains the dominant fitting cost;4screens can cost MORE than early one-candidate success in the old cylinder path. No60s guarantee and no broad matrix passing claim. Whole-percent C2 suggestion is advisory and does not prove actual C3/C4 feasibility. Earlier timestamped diagnosis describes previous policy/binaries only.

Changed for THIS owner iteration: `core/src/depth.rs`, `core/src/planes.rs` (screen/polish reuse helpers only), `core/src/feasibility.rs`, `core/tests/feasibility.rs`, `core/examples/c4_production_matrix.rs`, new `docs/prototypes/C4_ranked_depth_matrix_runner.py`, this report, task notes. Existing unrelated dirty work preserved. No commits/merge/push.

## Final measured native controls (FOUR-screen window)

Pinned CPU3, same180s external investigative budget, alongside parent workloads; timings are not controlled benchmark speedups. MAIN `runs/logs/C4-ranked-final-cone_9_to_6-r0-s1.json`: **PASS depth1.5,37.198835059s**,4screened,2polished (depth1.5/1.75 both0native violations). MAIN `runs/logs/C4-ranked-final-vase_bulge-r0-s1.json`: **PASS depth1.75,18.143009217s**,4screened,2polished (depth1.75/2 both0native violations). Complete full geometry, exact params, fitted records and guarded polished results are retained. These are TWO symmetric native C2/C3/C4 controls, NOT full-matrix or C5/C6 certification.

All first4attempts (complete fitted AND polished records) and selected topology compare EXACTLY equal to preliminary8screen runs across independent builds, excluding top-level elapsed time. Final isolated build: `C4-ranked-final-build.log` PASS; final lib/tests/example clippy: `C4-ranked-final-clippy.log` PASS; full crate cargo fmt --check PASS. Final unit log:2PASS,1intentionally ignored native gate; separate native gate1PASS as recorded above.

## Guarded configured-depth fast path (supersedes mandatory four-screen policy above)

Native investigation found the configured cylinder needs penalty polish: a fit-only zero-violation shortcut is insufficient. The configured fit now receives the unchanged native guarded polish immediately. Actual guarded success stops: no later depth can improve selected depth. Both fitted geometry and guarded result/topology remain recorded. If it fails, the other three depths are screened and their best ONE receives polish. Total maximum remains four fits/two polishes. This intentionally reserves one polish slot for configured depth, rather than selecting both slots solely by fit score; it can miss a feasible second-ranked deeper candidate and is still a bounded heuristic, not exhaustive/minimality-certified. No target, tolerance, fit/penalty budget, deterministic stopping or geometry guard changes.

Tests cover successful early stop, failure followed by ranked fallback, exact C3 retarget/cache equality, and two-polish cap. Official checks/matrix remain parent-owned. Immutable ranked target and runners untouched. Timings and parity are returned to parent separately.

Targeted final evidence (CPU4, alongside other work; not controlled speedup): cylinder depth1.5/r0 seeds1/2 PASS20.754712425s/20.256393801s; one screen/one native guarded polish each; selected1.5. Seed1 repeat19.664202759s and entire JSON byte-equivalent after excluding elapsed. Direct configured profile fit10.270059921s +polish7.780048253s, exact native report equality to selection. Release screen/reuse/direct optimize byte-exact geometry gate1PASS37.17s (`C4-cylinder-fast-native-parity.log`). Unit3PASS/1ignored; clippy PASS in `C4-cylinder-fast-final-check.log`, fmt PASS. Earlier zero-violation-only investigation did NOT fix cylinder because its fit needs polish; that variant was superseded. Only `core/src/depth.rs`, this report and task card changed by this follow-up; existing dirty files preserved. Distinct target `/tmp/vstone-c4-cylinder-fast-target` only. Source documentation/message wording finalized after measurement; numerical code identical.
