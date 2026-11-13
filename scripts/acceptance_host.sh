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
docker exec -e "ACCEPTANCE_STAMP=$stamp" \
    -e "ACCEPTANCE_LOGDIR=$workdir/runs/logs/acceptance-$stamp" \
    -w "$workdir" "$container" bash scripts/acceptance.sh
