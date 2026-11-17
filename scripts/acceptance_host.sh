#!/usr/bin/env bash
# Host owns provenance: worktree .git files contain host-only absolute paths.
set -eu
container="$1"
workdir="$2"
stamp=$(date -u +%Y%m%dT%H%M%SZ)-$$
logdir="$PWD/runs/logs/acceptance-$stamp"
mkdir -p "$logdir"
{
    printf 'host_worktree=%s\n' "$PWD"
    git rev-parse HEAD
    git status --short
} > "$logdir/host-provenance.log"
printf 'logs=%s\n' "$logdir"
worker=
cleanup() {
    result=$?
    trap - EXIT HUP INT TERM
    docker exec -w "$workdir" "$container" python scripts/acceptance_control.py stop "$stamp" || result=1
    if [ -n "$worker" ]; then
        wait "$worker" 2>/dev/null || true
    fi
    exit "$result"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
docker exec -e "ACCEPTANCE_STAMP=$stamp" \
    -e "ACCEPTANCE_LOGDIR=$workdir/runs/logs/acceptance-$stamp" \
    -w "$workdir" "$container" python scripts/acceptance_control.py run "$stamp" &
worker=$!
wait "$worker"
