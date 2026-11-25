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
printf 'logs=%s\n' "$logdir"
worker=
heartbeat=
memory_sampler=
host_pid=$$
cleanup() {
    result=$?
    trap - EXIT HUP INT TERM
    if [ -n "$heartbeat" ]; then
        kill "$heartbeat" 2>/dev/null || true
        wait "$heartbeat" 2>/dev/null || true
    fi
    if [ -n "$memory_sampler" ]; then
        kill "$memory_sampler" 2>/dev/null || true
        wait "$memory_sampler" 2>/dev/null || true
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
python3 "$PWD/scripts/acceptance_memory.py" "$container" "$logdir" &
memory_sampler=$!
touch "$logdir/host-heartbeat"
(
    while kill -0 "$host_pid" 2>/dev/null; do
        touch "$logdir/host-heartbeat"
        sleep 2
    done
) &
heartbeat=$!
docker exec -e "ACCEPT_LANES=${ACCEPT_LANES:-10}" -e "ACCEPTANCE_STAMP=$stamp" \
    -e "ACCEPTANCE_LOGDIR=$workdir/runs/logs/acceptance-$stamp" \
    -e "ACCEPTANCE_HEARTBEAT=$workdir/runs/logs/acceptance-$stamp/host-heartbeat" \
    -w "$workdir" "$container" python scripts/acceptance_control.py run "$stamp" &
worker=$!
wait "$worker"
