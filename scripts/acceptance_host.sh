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
    printf 'host_branch=%s\n' "$(git branch --show-current)"
    git rev-parse HEAD
    git status --short
} > "$logdir/host-provenance.log"
printf 'logs=%s\n' "$logdir"
worker=
heartbeat=
host_pid=$$
cleanup() {
    result=$?
    trap - EXIT HUP INT TERM
    if [ -n "$heartbeat" ]; then
        kill "$heartbeat" 2>/dev/null || true
        wait "$heartbeat" 2>/dev/null || true
    fi
    docker exec -w "$workdir" "$container" python scripts/acceptance_control.py stop "$stamp" || result=1
    if [ -n "$worker" ]; then
        wait "$worker" 2>/dev/null || true
    fi
    printf '%s\n' "$result" > "$logdir/host-exit-status"
    if "$PWD/scripts/publish_review.sh" "$logdir" > "$logdir/publish-review.log" 2>&1; then
        cat "$logdir/publish-review.log"
    else
        cat "$logdir/publish-review.log" >&2
        printf 'Review publish failed; acceptance exit status remains %s. See %s/publish-review.log\n' "$result" "$logdir" >&2
    fi
    exit "$result"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
touch "$logdir/host-heartbeat"
(
    while kill -0 "$host_pid" 2>/dev/null; do
        touch "$logdir/host-heartbeat"
        sleep 2
    done
) &
heartbeat=$!
docker exec -e "ACCEPTANCE_STAMP=$stamp" \
    -e "ACCEPTANCE_LOGDIR=$workdir/runs/logs/acceptance-$stamp" \
    -e "ACCEPTANCE_HEARTBEAT=$workdir/runs/logs/acceptance-$stamp/host-heartbeat" \
    -w "$workdir" "$container" python scripts/acceptance_control.py run "$stamp" &
worker=$!
wait "$worker"
