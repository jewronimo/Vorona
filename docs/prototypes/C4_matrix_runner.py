"""Container-only process supervisor; no geometry or builds. Sequential, isolated 60s cases.
Pass --caps .05,.075,.1,.125 for a bounded jitter sweep. JSONL is flushed per case.
No reported passing cap is a proof of a global maximum. Timings include topology and fit.
"""
import argparse
import json
import subprocess
import time

p = argparse.ArgumentParser(__doc__)
p.add_argument("binary", help="absolute path to parent-built c4_matrix")
p.add_argument("--caps", default=".1")
p.add_argument("--profiles", default="cylinder,cone_9_to_6,vase_bulge")
p.add_argument("--timeout", type=float, default=60)
p.add_argument("--sweep-only", action="store_true", help="omit report-only depths")
a = p.parse_args()
summary = {}
for cap in map(float, a.caps.split(",")):
    passing = True
    for profile in a.profiles.split(","):
        cases = [(1.5, r, s) for r in (0, .5, 1) for s in range(1, 11)]
        if not a.sweep_only:
            cases += [(1., 0, 1), (2., 0, 1)]
        for depth, randomness, seed in cases:
            start = time.monotonic()
            record = dict(profile=profile, depth=depth, randomness=randomness,
                          seed=seed, jitter_cap=cap, acceptance=depth == 1.5)
            try:
                done = subprocess.run([a.binary, profile, str(depth), str(randomness),
                                       str(seed), str(cap)], capture_output=True,
                                      text=True, timeout=a.timeout, check=True)
                record = json.loads(done.stdout)
                result = record["measurement"].get("result", {})
                passed = "Ok" in result and not result["Ok"]["report"]["violated"]
                record["passed"] = passed
                record["stderr"] = done.stderr
            except subprocess.TimeoutExpired:
                record.update(error="hard case timeout", passed=False)
            except (subprocess.CalledProcessError, ValueError, KeyError) as error:
                record.update(error=str(error), passed=False)
            record["wall_seconds"] = time.monotonic() - start
            if record["acceptance"]:
                passing &= record["passed"]
            print(json.dumps(record), flush=True)
    summary[cap] = passing
print(json.dumps(dict(summary="tested bounds only; not a global maximum",
                      tested_caps=summary,
                      largest_tested_passing_cap=max((c for c, ok in summary.items() if ok), default=None))), flush=True)
