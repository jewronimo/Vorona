# C4 final review preparation — configured-first FAST (2026-11-12)

Status: **parent verification complete; ready for Claude review; NOT fully accepted**.
Final evidence: `C4_final_results.md` (95/96 pass; one unresolved90s timeout).
Parent owns official worktree dev-check, final native full96 and acceptance completion.
No merge/push. Review this document before historical handoffs, whose policies differ.

## Keeper and provenance

Inspected dirty `task/C4-depth-space` in `runs/worktrees/C4-depth`; preserved all existing work.
The selector ALREADY had the desired FAST behavior; it was NOT restored or rewritten.
Only its module description changed. Completion chronology is not source provenance.

Immutable MAIN `runs/logs/C4-fast-owner-matrix96.jsonl` SHA256:
`6598cec83b7635c4f86a2400671b14aa51ce2a932f0255112a0bb1f571e9cf02`.
94/96 pass: cylinder30/32, cone32/32, vase32/32 (includes two report-only depths/profile).
Cylinder r.5 seeds3/8 were externally killed at60.0604/60.0605s; these are incomplete
searches, not geometric failures. Cone attempt counts:26 one-candidate/6 four-candidate;
vase22/10; completed cylinder29/1. First record explicitly says configured guarded polish
passed and deeper screening unnecessary. This distinguishes FAST from all-four-fit-first.
Historical parent check: MAIN `runs/logs/C4-fast-dev-check-2036.log` (not final certification).
No historical source hash manifest was found; source/binary bit-identity is NOT claimed.
Fresh native reuse/direct production byte-equivalence is tested separately below.

Current selector: configured depth fit + guarded penalty polish immediately; return on
native success. On failure, screen only remaining3 quarter-inch depths and polish their
best-ranked fit once. At most4 fits/2 polishes. Rank: native violation energy, count, depth.
C3 chart is cached; deterministic row depths are reassigned. Unit test compares cached
and fresh complete topology at each candidate. No global minimum/exhaustion guarantee.

## Review of all pre-existing uncommitted changes

- `feasibility.rs`, feasibility tests: constant admissible open-chart wrap does not
  mandate curvature defects; native whole-percent hex advice is verified against C2.
  No caller parameter override; variable-column quotas remain. Prior A04 five-percent
  advice remains and a separate explicitly whole-percent native sentence is added.
- `seeds.rs`, topology tests: conformal rather than forward-Euler row pitch; same C2
  variable columns, two full auxiliary rows beyond each end, unchanged spacing guards.
  Test includes support rows in continuous C2 sequence; no stretched last ordinary row.
- `planes.rs`: retained reference-style fixed-budget penalty TRF, terminal full-limit
  guard, exact baseline fallback, frozen whole-tile domain, finite normals/corners and
  physical rim guard. Feasible baseline returns unchanged. No new time stopping.
  Original tall SVD fit remains; no compact-QR production fit. Superseded colored
  polish/QR algebra diagnostics remain test-only, not selected by production.
- `lib.rs`, `depth.rs`, depth tests: native bounded selector and complete diagnostics;
  failure preserves evidence. `generate()` is still the pre-C5 placeholder, not end-to-end.
- planes tests retain analytic vase regression and existing reference/fixed-point gates.
- production example uses native selector, embedded profiles, cylinder unchanged,
  cone `[6]`25%, vase `[6]`30%, target unchanged. These owner fixture settings are
  NOT an automatic size relaxation; every actual parameter is emitted per case.
- historical depth profiler, timeout diagnosis and ranked report are archived diagnostic
  evidence, NOT final acceptance. Old commands/deadlines there are historical only.

## Precise experiment isolation added in this preparation

`planes_conformal_experiment.rs` and its public unguarded diagnostic export now require
explicit Cargo feature `c4-experiments` (default OFF). `c4_conformal_vase` has
`required-features`; default production/library/examples no longer compile that module.
Its useful source/artifacts are preserved. To reproduce: `cargo run --release
--features c4-experiments --example c4_conformal_vase -- ...` inside Vorona-dev.
Regression-required `ReferenceInput`/`reference_stage` stay available for integration
reference parity; native selector/optimizer never reads imported arrays or calls them.
No dependency added. No changes to JSON schemas, Python production or solver math here.

## Deadline runner

Canonical `C4_production_matrix_runner.py`: symmetric randomness0 =60s; any r>0 =90s.
Compatibility `C4_ranked_depth_matrix_runner.py` delegates to it, preventing divergent
policies. Harness label `production-configured-first-v3` identifies new runs.
Every record includes actual `deadline_seconds`, `timed_out`, measured wall time,
return code/stdout/stderr or timeout partials. `subprocess.run` kills and reaps its own
native child on expiry, then proceeds; no solver clock or deterministic cap changed.
Stub verification exercised all96 records/60-90 choice and9 injected timeouts,
including partial evidence and continued execution; it ran NO native matrix.
Deadline failure is incomplete search. A90s organic deadline does NOT establish the
original <=60s cylinder timing claim: report actual timings separately.

## Targeted checks (this preparation only)

All Rust commands in Vorona-dev; isolated `/tmp/vstone-c4-review-target`.
- `cargo test --lib depth::tests -- --skip screened_fit_reuse`:3 PASS.
- `cargo fmt --check`: PASS.
- `cargo clippy --lib --examples -- -D warnings`: PASS, default experiment OFF.
- `cargo check --features c4-experiments --example c4_conformal_vase`: PASS.
- Runner96 stub/9 injected timeouts: PASS (not geometric matrix evidence).
- `cargo test --lib terminal_guard_checks_all_limits_membership_and_rims`:1 PASS.
- `/tmp/C4-final-check.mk` created; `make -n -f /tmp/C4-final-check.mk dev-check` verified both Docker commands select this worktree; did NOT execute official check.
- `cargo test --release --lib screened_fit_reuse_is_identical_to_direct_production_optimize -- --ignored --nocapture`:1 PASS,35.66s; compares entire serialized result including geometry/report.
No official dev-check/full native matrix run by this child.

## Parent verification — HOST commands from this WORKTREE

Owner instruction supersedes the preparatory ordering: commit the verified review package, then Claude reviews before any merge.
Use a new target, preserve historical binaries/logs. Commands are sequential.

```sh
cd /Users/spooky/Projects/Vorona/voronoi-stone/runs/worktrees/C4-depth
docker ps --format '{{.Names}}'
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-final-fast-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_production_matrix > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-build.log 2>&1
docker exec -w /src/runs/worktrees/C4-depth Vorona-dev taskset -c 0 python docs/prototypes/C4_production_matrix_runner.py /tmp/vstone-c4-final-fast-target/release/examples/c4_production_matrix --symmetric-timeout 60 --organic-timeout 90 > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-matrix96.jsonl 2> /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-matrix96.stderr
```

Official check must select THIS worktree, not `/src` main. Create a temporary Makefile
outside repository, including this worktree Makefile and overriding recipes as follows
(literal TAB before each recipe); then run `make -f /tmp/C4-final-check.mk dev-check`.
The `dev-up` override verifies the existing container instead of changing any mount.

```make
include /Users/spooky/Projects/Vorona/voronoi-stone/runs/worktrees/C4-depth/Makefile
dev-up:
	docker exec Vorona-dev test -f /src/runs/worktrees/C4-depth/core/src/depth.rs
dev-check:
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-final-fast-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
```

```sh
make -f /tmp/C4-final-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-dev-check.log 2>&1
```

GNU make warns overridden recipes; expected. This preserves the official `make dev-check`
entrypoint, install+full-check semantics and worktree cwd. Do not use ordinary unmodified
`make dev-check` here: its docker exec defaults to MAIN `/src`.

Parent final report must count96 unique profile/depth/randomness/seed records, deadlines,
timeouts, native failures and full successful limit reports separately; acceptance90 vs
report-only6 separately. Record actual cylinder timings (not timeout budget), full-limit
reports/corner statistics/reflex counts, deterministic evidence and any unresolved cases.
Final evidence is now recorded; the unresolved timeout is not accepted. Owner authorized the review commit before Claude review. No merge/push.
