#!/usr/bin/env bash
# Container-only entrypoint. Run every gate, preserve failures and complete matrix.
set -uo pipefail
: "${ACCEPTANCE_STAMP:?Run make acceptance on the host to capture provenance}"
: "${ACCEPTANCE_LOGDIR:?Host must select the shared log directory}"
stamp="$ACCEPTANCE_STAMP"
logdir="$ACCEPTANCE_LOGDIR"
mkdir -p "$logdir"
# The host resolves Git worktree metadata and captures source before gates.
test -s "$logdir/source-sha256.txt" || { echo "Missing host source manifest" >&2; exit 1; }
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/vstone-acceptance-$stamp}"
status=0
lanes=$(python -c 'import sys; sys.path.insert(0, "scripts"); from acceptance_lanes import lane_count; print(lane_count())') || exit 1
SECONDS=0
exec > >(tee "$logdir/acceptance.log") 2>&1
printf 'worktree=%s\ntarget=%s\nlogs=%s\n' "$PWD" "$CARGO_TARGET_DIR" "$logdir"
cat "$logdir/host-provenance.log" || exit 1
# Compile ignored-test binaries in parallel, then run the same tests with lane-count threads.
stage_start=$SECONDS
(cd core && cargo test --release --lib --tests --no-run) > "$logdir/ignored-release-build.log" 2>&1
ignored_build_status=$?
printf 'ignored_release_build_seconds=%s\n' "$((SECONDS-stage_start))" > "$logdir/stage-wall-times.log"
stage_start=$SECONDS
if [ "$ignored_build_status" -eq 0 ]; then
    (cd core && cargo test --release --lib --tests -- --ignored --nocapture --test-threads="$lanes") > "$logdir/ignored-release.log" 2>&1 || status=1
else
    status=1
fi
printf 'ignored_release_tests_seconds=%s\n' "$((SECONDS-stage_start))" >> "$logdir/stage-wall-times.log"
stage_start=$SECONDS
(cd core && cargo build --release --features c4-experiments --example c4_production_matrix) > "$logdir/build.log" 2>&1
build_status=$?
printf 'a05_build_seconds=%s\n' "$((SECONDS-stage_start))" >> "$logdir/stage-wall-times.log"
stage_start=$SECONDS
if [ "$build_status" -eq 0 ]; then
    python docs/prototypes/C4_production_matrix_runner.py "$CARGO_TARGET_DIR/release/examples/c4_production_matrix" --symmetric-timeout 60 --organic-timeout 90 > "$logdir/matrix96.jsonl" 2> "$logdir/matrix96.stderr.log" || status=1
else
    status=1
fi
printf 'a05_matrix_seconds=%s\n' "$((SECONDS-stage_start))" >> "$logdir/stage-wall-times.log"
stage_start=$SECONDS
(cd core && cargo build --release --example a06_report --bin vstone-core) > "$logdir/a06-build.log" 2>&1
build_status=$?
printf 'a06_build_seconds=%s\n' "$((SECONDS-stage_start))" >> "$logdir/stage-wall-times.log"
stage_start=$SECONDS
if [ "$build_status" -eq 0 ]; then
    bash scripts/a06_matrix.sh "$CARGO_TARGET_DIR/release/examples/a06_report" "$logdir" || status=1
else
    status=1
fi
printf 'a06_and_numbered_seconds=%s\n' "$((SECONDS-stage_start))" >> "$logdir/stage-wall-times.log"
cat "$logdir/stage-wall-times.log"
python - "$logdir" <<'PYTIMING'
import json, sys
from pathlib import Path
p = Path(sys.argv[1]); names = ('a05-stage-timing.json', 'a06-stage-timing.json', 'numbered-stage-timing.json')
totals = {'parallel_wall_seconds': 0, 'solo_retry_wall_seconds': 0, 'total_wall_seconds': 0}
for name in names:
    if (p / name).exists():
        timing = json.loads((p / name).read_text())
        print(f'{name}: {json.dumps(timing)}')
        for key in totals:
            totals[key] += timing[key]
print('case-stage aggregate: ' + json.dumps(totals))
PYTIMING
printf 'A06 C5 combined acceptance requires all fourteen native/production closure/limit/determinism/numbered-STEP gates and owner STEP review; C6 held\n'
printf 'full acceptance wall_seconds=%s\n' "$SECONDS"
printf 'acceptance exit=%s (0 allows only the explicitly accepted known timeout; inspect summary)\n' "$status"
printf 'logs=%s\n' "$logdir"
exit "$status"
