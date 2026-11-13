# Production auto-depth timeout diagnosis (2026-11-12)

## Evidence (native Rust in Vorona-dev)

The immutable official binary and matrix were not rebuilt/restarted. Official full checks remain parent-owned. Diagnostic target: `/tmp/vstone-c4-timeout-diagnosis-target` only.

- **Cylinder r0.5 seed3**: unchanged production binary, depth1.5, CPU1, external600s budget: **PASS at depth1.75 after71.102101283s**. The first candidate has60 native violation entries; the second has0. Exactly2 ascending candidates. Thus the official60s timeout is demonstrably NOT geometric exhaustion. Complete geometry/selection: MAIN `runs/logs/C4-timeout-cylinder-r05-s3-long.json` (stderr companion empty).
- **Cone r0 seed1**: CPU2 diagnostic mirrors the original ascending policy using production-equivalent `fit_and_polish`, rebuilding native C3 to profile the original integration cost. At121.863655639s,9 candidates depths1.5..3.5 all fail; depth3.75 started at123.495451348s. Investigator stopped only this diagnostic PID via SIGTERM after that evidence (exit143); original external budget600s was not reached. This is **incomplete search, NOT exhaustion**. MAIN `runs/logs/C4-timeout-cone-r0-s1-profile.jsonl` contains each full OptReport and fit counters; `...-progress.jsonl` records starts.
- Across9 completed cone candidates: C3 **18.1384s**, fit **57.7133s**, polish **46.0112s**; fit SVD alone **56.5669s** (98.0% of fit). Candidate1 has108 sites/216 unknowns,102 SVDs,3.8869s SVD versus0.0480s Jacobians. Dense repeated SVD, not residual/Jacobian evaluation, dominates fit. Reference penalty polish also uses dense tall SVD and scalar finite differences; old test-only colored polish is not the active production reference penalty path.
- Finite physical-extent policy schedules **255 cone candidates** (bound65.1766829472013) and **190 vase candidates** (bound48.98979485566356) from1.5 by.25. The selector independently cold-fits and polishes EVERY failed candidate.60s is a whole-search deadline, not one-candidate budget. Cone/vase eventual success/exhaustion is not established by this bounded diagnostic.
- Measurements occurred alongside the official CPU0 matrix and compilation; different CPU affinity and concurrent load mean these are diagnostic timings, not controlled speedup benchmarks.

## Conservative integration fix

`core/src/depth.rs`: retain the first successful native C3 chart graph, clone it for subsequent depths, assign EXACT existing `seeds::assign_depths` row offsets. Placement/repair never uses depth except assigning these offsets and validating positivity. This removes repeat deterministic C3 repairs (~1.6-2s/candidate here) without skipping ANY candidate, changing geometry objectives, starts, solver budgets, limits, grid, first-success semantics or never-worse guard. C2 still checked each candidate. Native C4 still independently cold-starts each candidate. No solver arithmetic/performance shortcut introduced; earlier QR changes caused basin regressions, so not reintroduced. Cache cannot fix dominant SVD cost or promise60s.

New unit regression compares complete cached Topology against fresh native sidecount builds at depths1.5/1.75/2 for both cone and vase, randomness.5/seed3. Existing ascending/no-skips and exhaustion tests remain. Container `cargo test --lib depth::tests -- --test-threads=1`: **3 PASS**,9.69s test runtime. Container rustfmt PASS; `cargo clippy --lib --example c4_depth_profile --example c4_production_matrix -- -D warnings` PASS. Isolated release production example build PASS. No official make dev-check or full matrix by child.

New diagnostic `core/examples/c4_depth_profile.rs` deliberately preserves uncached C3 rebuilding to measure the original cost, emits candidate-start progress on stderr and profile/complete report JSONL on stdout. It is not the official matrix runner and changes no production output schema.

## Parent follow-up

1. Official worktree-selected dev-check required after cache edit; previous1918 log predates it.
2. Full current matrix remains attributable to immutable pre-cache binary; do NOT relabel it as cache validation.
3. Optional targeted cached-vs-immutable cylinder control: isolated target release example already built, same4args; compare entire `selection` byte-for-byte (exclude top-level timing). Cache regression establishes exact C3 identity, not an end-to-end60s speedup.
4. Cone/vase require substantially longer bounded runs (with candidate progress) to establish first success versus full finite-policy exhaustion. Do not call timeout infeasible. Further SVD acceleration needs nonlinear trajectory/matrix validation, not merely exact-arithmetic algebra equivalence.

No merge, commit or push. No runner/deadline/limit changes. Existing production matrix/jobs left untouched.
