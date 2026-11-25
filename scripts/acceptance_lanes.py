"""Run independent cases in isolated processes pinned to dedicated CPUs."""
import multiprocessing as mp
import os
import queue
import traceback


def lane_count():
    cpus = sorted(os.sched_getaffinity(0))
    requested = int(os.environ.get('ACCEPT_LANES', '10'))
    with open('/sys/fs/cgroup/cpu.max') as stream:
        quota = stream.read().split()
    capacity = len(cpus) if quota[0] == 'max' else min(len(cpus), int(quota[0]) // int(quota[1]))
    maximum = capacity - 2
    if requested < 1 or maximum < 1 or requested > maximum:
        raise ValueError(f'ACCEPT_LANES={requested}, allowed 1..{maximum} (CPU capacity {capacity})')
    return requested


def run_lanes(cases, function, duration):
    cpus = sorted(os.sched_getaffinity(0))
    requested = lane_count()
    if not cases:
        return []
    ordered = cases if requested == 1 else sorted(cases, key=duration, reverse=True)
    ctx = mp.get_context('fork')
    jobs, output = ctx.Queue(), ctx.Queue()
    for index, case in enumerate(ordered):
        jobs.put((index, case))
    count = min(requested, len(cases))
    for _ in range(count):
        jobs.put(None)
    def worker(cpu):
        os.sched_setaffinity(0, {cpu})
        while True:
            job = jobs.get()
            if job is None:
                return
            index, case = job
            try:
                output.put((index, function(case, cpu), None))
            except BaseException:
                output.put((index, None, traceback.format_exc()))
                return
    workers = [ctx.Process(target=worker, args=(cpu,)) for cpu in cpus[:count]]
    for process in workers:
        process.start()
    results = {}
    try:
        while len(results) < len(cases):
            try:
                index, value, error = output.get(timeout=1)
            except queue.Empty:
                if any(p.exitcode not in (None, 0) for p in workers) or all(not p.is_alive() for p in workers):
                    raise RuntimeError('acceptance lane exited before reporting all cases')
                continue
            if error:
                raise RuntimeError(error)
            results[index] = value
        return [results[i] for i in range(len(cases))]
    finally:
        for process in workers:
            if process.is_alive():
                process.terminate()
            process.join()
        jobs.close()
        output.close()


def run_with_retries(cases, function, duration, is_timeout, on_retry=None, should_retry=None):
    """Run lanes, then rerun only timed-out cases one at a time."""
    import time
    started = time.monotonic()
    first = run_lanes(cases, lambda case, cpu: function(case, cpu, False), duration)
    parallel_wall = time.monotonic() - started
    requested = int(os.environ.get('ACCEPT_LANES', '10'))
    # run_lanes returns its longest-first order; preserve the case/result association.
    ordered = cases if requested == 1 else sorted(cases, key=duration, reverse=True)
    retries = [(case, result) for case, result in zip(ordered, first)
               if is_timeout(result) and (should_retry is None or should_retry(case, result))] if requested > 1 else []
    retry_start = time.monotonic()
    retry_records = []
    for case, prior in retries:
        if on_retry:
            on_retry(case)
        previous = os.environ.get('ACCEPT_LANES')
        os.environ['ACCEPT_LANES'] = '1'
        try:
            solo = run_lanes([case], lambda c, cpu: function(c, cpu, True), duration)[0]
        finally:
            if previous is None:
                os.environ.pop('ACCEPT_LANES', None)
            else:
                os.environ['ACCEPT_LANES'] = previous
        first[ordered.index(case)] = solo
        retry_records.append((case, prior, solo))
    solo_wall = time.monotonic() - retry_start
    return first, retry_records, {'parallel_wall_seconds': parallel_wall,
                                  'solo_retry_wall_seconds': solo_wall,
                                  'total_wall_seconds': time.monotonic() - started}
