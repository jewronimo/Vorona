#!/usr/bin/env bash
# Container-only entrypoint. Run every gate, preserve failures and complete matrix.
set -uo pipefail
: "${ACCEPTANCE_STAMP:?Run make acceptance on the host to capture provenance}"
: "${ACCEPTANCE_LOGDIR:?Host must select the shared log directory}"
stamp="$ACCEPTANCE_STAMP"
logdir="$ACCEPTANCE_LOGDIR"
mkdir -p "$logdir"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/vstone-acceptance-$stamp}"
status=0
exec > >(tee "$logdir/acceptance.log") 2>&1
printf 'worktree=%s\ntarget=%s\nlogs=%s\n' "$PWD" "$CARGO_TARGET_DIR" "$logdir"
cat "$logdir/host-provenance.log" || exit 1
# One allowed CPU for ignored numerical gates and the sequential native matrix.
cpu=$(python -c 'import os; print(min(os.sched_getaffinity(0)))')
(cd core && taskset -c "$cpu" cargo test --release --lib --tests -- --ignored --nocapture --test-threads=1) > "$logdir/ignored-release.log" 2>&1 || status=1
(cd core && cargo build --release --features c4-experiments --example c4_production_matrix) > "$logdir/build.log" 2>&1
build_status=$?
if [ "$build_status" -eq 0 ]; then
    taskset -c "$cpu" python docs/prototypes/C4_production_matrix_runner.py "$CARGO_TARGET_DIR/release/examples/c4_production_matrix" --symmetric-timeout 60 --organic-timeout 90 > "$logdir/matrix96.jsonl" 2> "$logdir/matrix96.stderr.log" || status=1
else
    status=1
fi
printf 'acceptance exit=%s (0 allows only the explicitly accepted known timeout; inspect summary)\n' "$status"
printf 'logs=%s\n' "$logdir"
exit "$status"
