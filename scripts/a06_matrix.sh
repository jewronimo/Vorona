#!/usr/bin/env bash
# Container-only, caller pins a single CPU. §9.6: six required + twelve report-only.
set -uo pipefail
binary=$1
logdir=$2
mkdir -p "$logdir"
status=0
for profile in cylinder cone_9_to_6 vase_bulge; do
    for mode in smooth symmetric doubled random; do
        if [ "${3:-all}" = required ] && [ "$mode" != smooth ] && [ "$mode" != symmetric ]; then continue; fi
        case "$mode" in
            smooth) depth=0; randomness=0; seeds=1; gate=required ;;
            symmetric) depth=0.25; randomness=0; seeds=1; gate=required ;;
            doubled) depth=0.5; randomness=0; seeds=1; gate=report-only ;;
            random) depth=0.25; randomness=1; seeds='1 2 3'; gate=report-only ;;
        esac
        for seed in $seeds; do
            key="$profile-$mode-s$seed"
            timeout --signal=TERM --kill-after=2 90 "$binary" tests/fixtures/core_input_cylinder.json "$depth" "$randomness" "$seed" "tests/fixtures/profiles/$profile.json" repeat > "$logdir/a06-$key.json" 2> "$logdir/a06-$key.stderr.log"
            rc=$?
            printf '%s exit=%s budget=90s-including-deterministic-repeat gate=%s\n' "$key" "$rc" "$gate" >> "$logdir/a06-summary.log"
            if [ "$rc" -ne 0 ] && [ "$gate" = required ]; then status=1; fi
        done
    done
done
exit "$status"
