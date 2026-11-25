"""Run A06 native cases in isolated, CPU-pinned lanes."""
import json
from pathlib import Path
import subprocess
import sys
import time
from acceptance_lanes import run_with_retries

binary, dest, *args = sys.argv[1:]
out = Path(dest)
(out / 'flare-profile.json').write_text('[[6.0,0.0],[15.0,30.0]]\n')
cases = []
for row in ('constant', 'proportional'):
    for profile in ('cylinder', 'cone_9_to_6', 'vase_bulge', 'flare'):
        if row == 'proportional' and profile == 'cylinder':
            continue
        for mode, depth, randomness, seeds, gate in [('smooth', 0, 0, [1], 'required'), ('symmetric', .25, 0, [1], 'required'), ('doubled', .5, 0, [1], 'report-only'), ('random', .25, 1, [1,2,3], 'report-only')]:
            if (args and args[0] == 'required' or row == 'proportional' or profile == 'flare') and gate != 'required':
                continue
            for seed in seeds:
                cases.append((profile, row, mode, depth, randomness, seed, gate))
def run(case, cpu, solo):
    profile, row, mode, depth, randomness, seed, gate = case
    key = f'{profile}-{row}-{mode}-s{seed}'
    profile_path = out / 'flare-profile.json' if profile == 'flare' else Path(f'tests/fixtures/profiles/{profile}.json')
    cmd = ['taskset', '-c', str(cpu), binary, 'tests/fixtures/core_input_cylinder.json', str(depth), str(randomness), str(seed), str(profile_path), 'repeat', row]
    start = time.monotonic()
    timed_out = False
    with (out / f'a06-{key}.json').open('wb') as stdout, (out / f'a06-{key}.stderr.log').open('wb') as stderr:
        try:
            rc = subprocess.run(cmd, stdout=stdout, stderr=stderr, timeout=90).returncode
        except subprocess.TimeoutExpired:
            timed_out = True
            rc = 124
    if rc < 0:
        rc = 128 - rc
    return key, rc, gate, time.monotonic()-start, timed_out
def preserve(case):
    profile, row, mode, _, _, seed, _ = case
    key = f'{profile}-{row}-{mode}-s{seed}'
    for suffix in ('.json', '.stderr.log'):
        path = out / f'a06-{key}{suffix}'
        if path.exists():
            path.rename(path.with_name(path.name + '.parallel'))
results, retries, timing = run_with_retries(cases, run, lambda c: {'random':90, 'doubled':90, 'symmetric':40, 'smooth':5}[c[2]], lambda r: r[4], preserve)
retry_by_key = {solo[0]: solo for _, _, solo in retries}
(out / 'a06-stage-timing.json').write_text(json.dumps(timing, indent=2) + '\n')

results.sort()
(out / 'a06-summary.log').write_text(''.join(
    f'{k} exit={rc} budget=90s-including-deterministic-repeat gate={gate}' +
    (f' timed out in parallel, solo retry: {"pass" if rc == 0 else "fail"}, {seconds:.3f}s' if k in retry_by_key else '') + '\n'
    for k,rc,gate,seconds,_ in results))
(out / 'a06-timings.json').write_text(json.dumps({k: t for k,_,_,t,_ in results}, sort_keys=True, indent=2)+'\n')
sys.exit(int(any(rc != 0 and gate == 'required' for _,rc,gate,_,_ in results)))
