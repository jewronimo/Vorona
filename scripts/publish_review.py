#!/usr/bin/env python3
"""Publish host acceptance evidence through the owner's SSH alias."""
import datetime as dt
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time

CONFIG = Path.home() / '.config/vorona'
QUEUE = CONFIG / 'review_pending'
SSH_OPTIONS = ['-o', 'BatchMode=yes', '-o', 'ConnectTimeout=10', '-o', 'ControlMaster=auto', '-o', f'ControlPath={Path.home()}/.ssh/cm-%r@%h:%p', '-o', 'ControlPersist=60']


def ssh(host, command, *, check=True):
    for attempt in range(3):
        result = subprocess.run(['ssh', *SSH_OPTIONS, host, command], text=True, capture_output=True)
        if result.returncode == 0 or 'Not allowed at this time' not in result.stderr or attempt == 2:
            break
        time.sleep(30)
    if check and result.returncode:
        raise RuntimeError(result.stderr.strip() or f'ssh failed ({result.returncode})')
    return result


def enqueue(paths):
    CONFIG.mkdir(parents=True, exist_ok=True)
    existing = QUEUE.read_text().splitlines() if QUEUE.exists() else []
    for path in paths:
        if str(path) not in existing:
            existing.append(str(path))
    tmp = QUEUE.with_suffix('.tmp')
    tmp.write_text(''.join(p + '\n' for p in existing))
    os.replace(tmp, QUEUE)


def run_info(path):
    match = re.fullmatch(r'acceptance-(\d{8}T\d{6}Z)-\d+', path.name)
    if not match:
        raise ValueError(f'Unrecognized acceptance directory: {path}')
    when = dt.datetime.strptime(match.group(1), '%Y%m%dT%H%M%SZ').replace(tzinfo=dt.timezone.utc).astimezone()
    provenance = (path / 'host-provenance.log').read_text().splitlines()
    commit = next((x for x in provenance if re.fullmatch(r'[0-9a-f]{40}', x)), None)
    if not commit:
        raise ValueError('Missing commit in host provenance')
    worktree = next((x.split('=', 1)[1] for x in provenance if x.startswith('host_worktree=')), '')
    branch = subprocess.run(['git', '-C', worktree, 'name-rev', '--name-only', '--no-undefined', commit], capture_output=True, text=True)
    # The recorded worktree name is stable when the commit is shared across branches.
    branch_name = next((x.split('=', 1)[1] for x in provenance if x.startswith('host_branch=')), '')
    branch_name = branch_name or (Path(worktree).name if worktree else (branch.stdout.strip() or 'unknown'))
    branch_name = re.sub(r'[^A-Za-z0-9._-]+', '-', branch_name.removeprefix('task/'))
    dirty = any(x.startswith((' M ', 'M ', '?? ', 'A ', ' D ', 'D ')) for x in provenance)
    acceptance = (path / 'acceptance.log').read_text() if (path / 'acceptance.log').exists() else ''
    status_file = path / 'host-exit-status'
    status = status_file.read_text().strip() if status_file.exists() else None
    if status is None:
        found = re.findall(r'acceptance exit=(\d+)', acceptance)
        status = found[-1] if found else 'unknown'
    outcome = 'PASS' if status == '0' else 'FAIL'
    months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']
    day = f'{months[when.month-1]} {when.day}'
    name = f'{when:%H%M} {branch_name} {commit[:7]}{"-dirty" if dirty else ""} {outcome}'
    return when, day, name, commit, branch_name, outcome


def publish(path, host, root, rsync):
    when, day, name, commit, branch, outcome = run_info(path)
    parent = f'{root}/{when.year}/{day}'
    with tempfile.TemporaryDirectory(prefix='vorona-review-') as tmp:
        stage = Path(tmp)
        for pattern in ('*.step', '*.png', 'a06-summary.log', 'a06-numbered-summary.jsonl', 'host-provenance.log'):
            for source in path.glob(pattern):
                if source.is_file():
                    shutil.copy2(source, stage / source.name)
        matrix = path / 'matrix96.stderr.log'
        if matrix.exists():
            (stage / 'matrix96-summary.txt').write_text(matrix.read_text())
        lines = [f'Acceptance: {path.name}', f'Branch: {branch}', f'Commit: {commit}', f'Result: {outcome}', '', 'Each numbered STEP is the named shape and settings, for visual review.', 'd0.0 is smooth; d0.25 is relief 0.25. Constant/proportional describes row sizing.', '']
        summary = path / 'a06-numbered-summary.jsonl'
        if summary.exists():
            for line in summary.read_text().splitlines():
                record = json.loads(line)
                case = record['case']
                output = path / f'{case}-output-0.json'
                blanks = 'unknown'
                if output.exists():
                    message = json.loads(output.read_text()).get('message', '')
                    found = re.search(r'(\d+) blanks', message)
                    if found:
                        blanks = found.group(1)
                lines.append(f"{case}: {'PASS' if record.get('passed') else 'FAIL'}; blanks {blanks}; file {case}-numbered.step")
        else:
            lines.append('No numbered case summary was produced.')
        lines += ['', 'a06-summary.log: native case outcomes; a06-numbered-summary.jsonl: numbered case results.', 'matrix96-summary.txt: A05 matrix summary; host-provenance.log: source state.', 'SHA256SUMS verifies the files in this folder.']
        (stage / 'README.txt').write_text('\n'.join(lines) + '\n')
        checks = []
        for file in sorted(stage.iterdir()):
            digest = hashlib.sha256()
            with file.open('rb') as data:
                for chunk in iter(lambda: data.read(1024 * 1024), b''):
                    digest.update(chunk)
            checks.append(f'{digest.hexdigest()}  {file.name}\n')
        (stage / 'SHA256SUMS').write_text(''.join(checks))
        q = shlex.quote
        ssh(host, f'mkdir -p {q(parent)}')
        index = 1
        while True:
            suffix = '' if index == 1 else f' ({index})'
            final = f'{parent}/{name}{suffix}'
            partial = final + '.partial'
            # mkdir is atomic and never reuses an existing incomplete transfer.
            cmd = f'if [ -e {q(final)} ] || [ -e {q(partial)} ]; then exit 17; fi; mkdir {q(partial)}'
            res = ssh(host, cmd, check=False)
            if res.returncode == 0:
                break
            if res.returncode != 17:
                raise RuntimeError(res.stderr or 'Unable to reserve destination')
            index += 1
        shell = shlex.join(['ssh', *SSH_OPTIONS])
        subprocess.run([rsync, '-s', '-a', '--partial', '-e', shell, str(stage) + '/', f'{host}:{partial}/'], check=True)
        verify = ssh(host, f'cd {q(partial)} && sha256sum -c SHA256SUMS')
        print(verify.stdout, end='')
        relative = f'{when.year}/{day}/{name}{suffix}'
        # latest.txt is the sole intentionally updated pointer; review directories are never overwritten.
        command = f'test ! -e {q(final)} && mv {q(partial)} {q(final)} && printf "%s\\n" {q(relative)} > {q(root + "/latest.txt")}'
        ssh(host, command)
        print(f'Published: {final}', flush=True)


def main():
    if len(sys.argv) != 2:
        raise ValueError('usage: publish_review.sh <acceptance-log-dir>')
    current = Path(sys.argv[1]).expanduser().resolve(strict=True)
    target = (CONFIG / 'review_target').read_text().strip()
    host, sep, root = target.partition(':')
    if not sep or not host or not root.startswith('/') or '\n' in target:
        raise ValueError('review_target must be one line host:/absolute/path')
    pending = QUEUE.read_text().splitlines() if QUEUE.exists() else []
    paths = list(dict.fromkeys([*pending, str(current)]))
    reach = ssh(host, 'true', check=False)
    if reach.returncode:
        enqueue(paths)
        raise RuntimeError(f'NAS unreachable; queued {current}: {reach.stderr.strip()}')
    rsync = shutil.which('rsync')
    if not rsync or subprocess.run([rsync, '--version'], capture_output=True, text=True).stdout.startswith('openrsync'):
        enqueue(paths)
        raise RuntimeError('A modern rsync supporting -s is required; queued runs')
    for index, entry in enumerate(paths):
        try:
            publish(Path(entry), host, root.rstrip('/'), rsync)
        except Exception:
            QUEUE.write_text(''.join(p + '\n' for p in paths[index:]))
            raise
        QUEUE.write_text(''.join(p + '\n' for p in paths[index + 1:]))


if __name__ == '__main__':
    try:
        CONFIG.mkdir(parents=True, exist_ok=True)
        with (CONFIG / 'review_publish.lock').open('a') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f'Review publish failed: {error}', file=sys.stderr)
        sys.exit(1)
