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
printf '[[6.0,0.0],[15.0,30.0]]\n' > "$logdir/flare-profile.json"
for row_height in constant proportional; do
for profile in cylinder cone_9_to_6 vase_bulge flare; do
    if [ "$row_height" = proportional ] && [ "$profile" = cylinder ]; then continue; fi
    profile_path="tests/fixtures/profiles/$profile.json"
    if [ "$profile" = flare ]; then profile_path="$logdir/flare-profile.json"; fi
    for mode in smooth symmetric doubled random; do
        if [ "${3:-all}" = required ] && [ "$mode" != smooth ] && [ "$mode" != symmetric ]; then continue; fi
        if { [ "$row_height" = proportional ] || [ "$profile" = flare ]; } && [ "$mode" != smooth ] && [ "$mode" != symmetric ]; then continue; fi
        case "$mode" in
            smooth) depth=0; randomness=0; seeds=1; gate=required ;;
            symmetric) depth=0.25; randomness=0; seeds=1; gate=required ;;
            doubled) depth=0.5; randomness=0; seeds=1; gate=report-only ;;
            random) depth=0.25; randomness=1; seeds='1 2 3'; gate=report-only ;;
        esac
        for seed in $seeds; do
            key="$profile-$row_height-$mode-s$seed"
            timeout --signal=TERM --kill-after=2 90 "$binary" tests/fixtures/core_input_cylinder.json "$depth" "$randomness" "$seed" "$profile_path" repeat "$row_height" > "$logdir/a06-$key.json" 2> "$logdir/a06-$key.stderr.log"
            rc=$?
            printf '%s exit=%s budget=90s-including-deterministic-repeat gate=%s\n' "$key" "$rc" "$gate" >> "$logdir/a06-summary.log"
            if [ "$rc" -ne 0 ] && [ "$gate" = required ]; then status=1; fi
        done
    done
done
done
python scripts/a06_numbered_matrix.py "$core_binary" "$logdir" || status=1
exit "$status"
