"""Run A06 native cases in isolated, CPU-pinned lanes."""
import json
from pathlib import Path
import subprocess
import sys
import time
from acceptance_lanes import run_lanes

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
def run(case, cpu):
    profile, row, mode, depth, randomness, seed, gate = case
    key = f'{profile}-{row}-{mode}-s{seed}'
    profile_path = out / 'flare-profile.json' if profile == 'flare' else Path(f'tests/fixtures/profiles/{profile}.json')
    cmd = ['timeout', '--signal=TERM', '--kill-after=2', '90', 'taskset', '-c', str(cpu), binary, 'tests/fixtures/core_input_cylinder.json', str(depth), str(randomness), str(seed), str(profile_path), 'repeat', row]
    start = time.monotonic()
    with (out / f'a06-{key}.json').open('wb') as stdout, (out / f'a06-{key}.stderr.log').open('wb') as stderr:
        rc = subprocess.run(cmd, stdout=stdout, stderr=stderr).returncode
    if rc < 0:
        rc = 128 - rc
    return key, rc, gate, time.monotonic()-start
results = run_lanes(cases, run, lambda c: {'random':90, 'doubled':90, 'symmetric':40, 'smooth':5}[c[2]])
results.sort()
(out / 'a06-summary.log').write_text(''.join(f'{k} exit={rc} budget=90s-including-deterministic-repeat gate={gate}\n' for k,rc,gate,_ in results))
(out / 'a06-timings.json').write_text(json.dumps({k: t for k,_,_,t in results}, sort_keys=True, indent=2)+'\n')
sys.exit(int(any(rc != 0 and gate == 'required' for _,rc,gate,_ in results)))
