#!/usr/bin/env bash
# Container-only; complete owner matrix, deterministic repeats, then production STEP gates.
set -uo pipefail
binary=$1
logdir=$2
core_binary="$(dirname "$(dirname "$binary")")/vstone-core"
for required_binary in "$binary" "$core_binary"; do
    if [ ! -x "$required_binary" ]; then
        printf "Required binary missing or not executable: %s\n" "$required_binary" >&2
        exit 1
    fi
done
mkdir -p "$logdir"
status=0
python scripts/a06_parallel_matrix.py "$binary" "$logdir" "${3:-all}" || status=1
python scripts/a06_numbered_matrix.py "$core_binary" "$logdir" || status=1
exit "$status"
