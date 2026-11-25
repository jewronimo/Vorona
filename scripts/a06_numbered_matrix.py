"""Container-only production repeats and numbered STEP gates for all owner cases."""
import json
from pathlib import Path
import subprocess
import sys
import time
from acceptance_lanes import run_lanes

from vstone.step_out import write_assembled


def main():
    binary, destination = sys.argv[1:]
    out = Path(destination)
    fixture = json.loads(Path('tests/fixtures/core_input_a06_cylinder.json').read_text())
    profiles = {
        name: json.loads(Path(f'tests/fixtures/profiles/{name}.json').read_text())
        for name in ('cylinder', 'cone_9_to_6', 'vase_bulge')
    }
    profiles['flare'] = [[6.0, 0.0], [15.0, 30.0]]
    cases = []
    for mode in ('constant', 'proportional'):
        for profile, points in profiles.items():
            if mode == 'proportional' and profile == 'cylinder':
                continue
            for depth in (0.0, 0.25):
                cases.append((mode, profile, depth))
    durations = json.loads(Path('scripts/acceptance_durations.json').read_text())['numbered']
    def run(case, cpu):
                mode, profile, depth = case
                points = profiles[profile]
                key = f'{profile}-{mode}-d{depth}'
                record = {'case': key, 'budget_seconds': 90}
                start = time.monotonic()
                try:
                    data = json.loads(json.dumps(fixture))
                    data['surface']['profile'] = points
                    data['params'].update(row_height=mode, depth=depth, randomness=0, seed=1)
                    source = out / f'{key}-input.json'
                    source.write_text(json.dumps(data, indent=2))
                    results = []
                    for repeat in range(2):
                        target = out / f'{key}-output-{repeat}.json'
                        with target.open('wb') as stdout, (out / f'{key}-production-{repeat}.log').open('wb') as stderr:
                            subprocess.run(["taskset", "-c", str(cpu), binary, str(source)], stdout=stdout, stderr=stderr,
                                           timeout=max(0.001, 90 - (time.monotonic() - start)), check=True)
                        results.append(target.read_bytes())
                    record['determinism'] = results[0] == results[1]
                    result = json.loads(results[0])
                    if not record['determinism'] or not result['validation']['passed'] or result['status'] != 'ok':
                        raise RuntimeError('production result failed validation/status/determinism')
                    step = out / f'{key}-numbered.step'
                    write_assembled(result, step)
                    if not step.is_file() or step.stat().st_size == 0:
                        raise RuntimeError('numbered STEP missing or empty')
                    content = step.read_bytes()
                    if not result['pieces'] or any(piece['id'].encode() not in content for piece in result['pieces']):
                        raise RuntimeError('numbered STEP lacks piece labels')
                    record.update(passed=True, step=str(step), step_bytes=step.stat().st_size)
                except Exception as error:
                    record.update(passed=False, error=str(error))
                record['seconds'] = time.monotonic() - start
                return record
    def duration(case):
        mode, profile, depth = case
        return durations.get(f'{profile}-{mode}-d{depth}', 0)
    records = run_lanes(cases, run, duration)
    records.sort(key=lambda r: r['case'])
    (out / 'a06-numbered-summary.jsonl').write_text(''.join(json.dumps(r) + '\n' for r in records))
    return int(any(not r['passed'] for r in records))


if __name__ == '__main__':
    sys.exit(main())
