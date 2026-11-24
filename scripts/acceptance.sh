#!/usr/bin/env bash
# Container-only entrypoint. Run every gate, preserve failures and complete matrix.
set -uo pipefail
: "${ACCEPTANCE_STAMP:?Run make acceptance on the host to capture provenance}"
: "${ACCEPTANCE_LOGDIR:?Host must select the shared log directory}"
stamp="$ACCEPTANCE_STAMP"
logdir="$ACCEPTANCE_LOGDIR"
mkdir -p "$logdir"
# Record exact source bytes, including untracked non-ignored files, before any gate.
python3 - "$logdir/source-sha256.txt" <<'PYHASH'
import hashlib, os, subprocess, sys
paths = subprocess.check_output(["git", "ls-files", "-co", "--exclude-standard", "-z", "--", "core/", "py/", "scripts/", "docs/schema/"]).split(b"\0")
with open(sys.argv[1], "w", encoding="utf-8") as out:
    for raw in sorted(set(filter(None, paths))):
        path = os.fsdecode(raw)
        with open(path, "rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        out.write(f"{digest}  {path}\n")
PYHASH
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
# A06 C4 gates are additional to, never replacements for, A05 regressions.
(cd core && cargo build --release --example a06_report --bin vstone-core) > "$logdir/a06-build.log" 2>&1
if [ "$?" -eq 0 ]; then
    taskset -c "$cpu" bash scripts/a06_matrix.sh "$CARGO_TARGET_DIR/release/examples/a06_report" "$logdir" || status=1
else
    status=1
fi
printf 'A06 C5 combined acceptance requires all fourteen native/production closure/limit/determinism/numbered-STEP gates and owner STEP review; C6 held\n'
printf 'acceptance exit=%s (0 allows only the explicitly accepted known timeout; inspect summary)\n' "$status"
printf 'logs=%s\n' "$logdir"
exit "$status"
