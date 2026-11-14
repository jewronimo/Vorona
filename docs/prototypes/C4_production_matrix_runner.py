"""Sequential full historical96 production matrix, with an honest process deadline.
Runs a parent-built Rust binary; performs no geometry or candidate selection.
"""
import argparse
import json
import subprocess
import time
import sys

p = argparse.ArgumentParser(__doc__)
p.add_argument('binary')
p.add_argument('--symmetric-timeout', type=float, default=60)
p.add_argument('--organic-timeout', type=float, default=90)
a = p.parse_args()
if not (0 < a.symmetric_timeout < float("inf") and 0 < a.organic_timeout < float("inf")):
    p.error("deadlines must be positive finite seconds")
records = []
for profile in ('cylinder', 'cone_9_to_6', 'vase_bulge'):
    cases = [(1.5, r, s) for r in (0, .5, 1) for s in range(1, 11)]
    cases += [(1., 0, 1), (2., 0, 1)]
    for depth, randomness, seed in cases:
        deadline = a.symmetric_timeout if randomness == 0 else a.organic_timeout
        start = time.monotonic()
        record = dict(harness='production-configured-first-v3', profile=profile,
                      configured_depth=depth, randomness=randomness, seed=seed,
                      acceptance=depth == 1.5, deadline_seconds=deadline,
                      timed_out=False,
                      passed=False)
        try:
            done = subprocess.run([a.binary, profile, str(depth), str(randomness),
                                   str(seed)], capture_output=True, text=True,
                                  timeout=deadline, check=False)
            record['returncode'] = done.returncode
            record['stderr'] = done.stderr
            record['raw_stdout'] = done.stdout
            if done.returncode == 0:
                measurement = json.loads(done.stdout)
                if not isinstance(measurement, dict) or not isinstance(measurement.get('passed'), bool):
                    raise ValueError('native result must be an object with boolean passed')
                record['measurement'] = measurement
                record['passed'] = measurement.get('passed') is True
            else:
                record['error'] = 'native process failed'
        except subprocess.TimeoutExpired as error:
            record['timed_out'] = True
            record['error'] = 'hard case timeout; candidate search incomplete'
            record['stdout_partial'] = (error.stdout or b'').decode(errors='replace')
            record['stderr'] = (error.stderr or b'').decode(errors='replace')
        except (ValueError, OSError) as error:
            record['error'] = str(error)
        record['wall_seconds'] = time.monotonic() - start
        known_timeout = (profile == 'cylinder' and depth == 1.5
                         and randomness == .5 and seed == 3
                         and deadline == 90 and record['timed_out'])
        report_timeout = (profile == 'cylinder' and depth == 1. and randomness == 0
                          and seed == 1 and deadline == 60 and record['timed_out'])
        record['outcome'] = ('pass' if record['passed'] else
                             'accepted_known_timeout' if known_timeout else
                             'accepted_report_only_timeout' if report_timeout else
                             'new_failure' if record['acceptance'] else 'report_only_failure')
        records.append(record)
        print(json.dumps(record), flush=True)
summary = {name: sum(r['outcome'] == name for r in records) for name in
           ('pass', 'accepted_known_timeout', 'accepted_report_only_timeout', 'new_failure', 'report_only_failure')}
summary.update(records=len(records), acceptance_records=sum(r['acceptance'] for r in records),
               fully_passed=all(r['passed'] for r in records))
print(json.dumps(summary), file=sys.stderr, flush=True)
# The owner accepted one acceptance timeout and the cylinder depth1 symmetric
# report-only 60s timeout. Neither provides geometry or closure measurements.
# Report-only geometry failures are informational; process/JSON failures are not.
infrastructure_failure = any(not r.get('measurement') and
                             r['outcome'] not in ('accepted_known_timeout', 'accepted_report_only_timeout') for r in records)
sys.exit(1 if summary['new_failure'] or infrastructure_failure else 0)
