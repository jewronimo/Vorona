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
        details = stage / 'details'
        details.mkdir()
        for pattern in ('*.step', '*.png', 'a06-summary.log', 'a06-numbered-summary.jsonl', 'host-provenance.log'):
            for source in path.glob(pattern):
                if source.is_file():
                    destination = stage if source.suffix == '.step' else details
                    shutil.copy2(source, destination / source.name)
        matrix = path / 'matrix96.stderr.log'
        if matrix.exists():
            (details / 'matrix96-summary.txt').write_text(matrix.read_text())
        lines = [f'Acceptance: {path.name}', f'Branch: {branch}', f'Commit: {commit}', f'Result: {outcome}', '', 'Each numbered STEP is the named shape and settings, for visual review.', 'd0.0 is smooth; d0.25 is relief 0.25. Constant/proportional describes row sizing.', '']
        summary = path / 'a06-numbered-summary.jsonl'
        if summary.exists():
            for line in summary.read_text().splitlines():
                record = json.loads(line)
                case = record['case']
                output = path / f'{case}-output-0.json'
                blanks = 'not recorded'
                if output.exists():
                    message = json.loads(output.read_text()).get('message', '')
                    found = re.search(r'(\d+) blanks', message)
                    if found:
                        blanks = found.group(1)
                lines.append(f"{case}: {'PASS' if record.get('passed') else 'FAIL'}; blanks {blanks}; file {case}-numbered.step")
        else:
            lines.append('No numbered case summary was produced.')
        lines += ['', 'a06-summary.log: native case outcomes; a06-numbered-summary.jsonl: numbered case results.', 'matrix96-summary.txt: A05 matrix summary; host-provenance.log: source state.', 'SHA256SUMS verifies the files in this folder.']
        # Reconstruct the old publisher's exact manifest to recognize existing flat-layout
        # publications. This is a byte-for-byte comparison, not a filename-only match.
        # Earlier flat publications used 'unknown' when blank counts were absent.
        legacy_readme = ('\n'.join(lines) + '\n').replace('blanks not recorded;', 'blanks unknown;')
        legacy_hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                         for p in stage.rglob('*') if p.is_file()}
        legacy_hashes['README.txt'] = hashlib.sha256(legacy_readme.encode()).hexdigest()
        legacy_manifest = ''.join(f'{legacy_hashes[n]}  {n}\n' for n in sorted(legacy_hashes))
        lines[-3:] = ['Technical logs and renders are in details/.',
                      'details/matrix96-summary.txt: A05 matrix summary; details/host-provenance.log: source state.',
                      'details/SHA256SUMS verifies all files relative to this folder.']
        (stage / 'README.txt').write_text('\n'.join(lines) + '\n')

        def write_manifest():
            checks = []
            for file in sorted(stage.rglob('*')):
                if not file.is_file() or file == details / 'SHA256SUMS':
                    continue
                digest = hashlib.sha256()
                with file.open('rb') as data:
                    for chunk in iter(lambda: data.read(1024 * 1024), b''):
                        digest.update(chunk)
                checks.append(f'{digest.hexdigest()}  {file.relative_to(stage).as_posix()}\n')
            text = ''.join(checks)
            (details / 'SHA256SUMS').write_text(text)
            return text

        manifest = write_manifest()
        base_readme = (stage / 'README.txt').read_text()
        reason = 'This is a new copy because the contents differ from an earlier publication of this acceptance.\n'
        (stage / 'README.txt').write_text(base_readme + reason)
        changed_manifest = write_manifest()
        (stage / 'README.txt').write_text(base_readme)
        write_manifest()
        q = shlex.quote
        ssh(host, f'mkdir -p {q(parent)}')
        # Only a matching acceptance ID and exact manifest qualify as a duplicate.
        # Check both the current layout and the previously published flat layout.
        scan = (f'for f in {q(parent)}/*/README.txt; do '
                f'[ -f "$f" ] || continue; '
                f'if grep -F -x -q -- {q("Acceptance: " + path.name)} "$f"; then '
                'printf "%s\\n" "${f%/README.txt}"; fi; done')
        found = ssh(host, scan).stdout.splitlines()
        # The manually published final acceptance predates the publisher README format.
        # Recognize it only when its recorded run and all review artifacts match bytes.
        manual_scan = (f'for f in {q(parent)}/*/details/acceptance.log; do '
                       f'[ -f "$f" ] || continue; '
                       f'if grep -F -q -- {q(path.name)} "$f"; then '
                       'printf "%s\\n" "${f%/details/acceptance.log}"; fi; done')
        for existing in ssh(host, manual_scan).stdout.splitlines():
            expected = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in path.glob('*-numbered.step')}
            for filename in ('a06-numbered-summary.jsonl', 'a06-summary.log',
                             'host-provenance.log', 'source-sha256.txt', 'acceptance.log'):
                expected['details/' + filename] = hashlib.sha256((path / filename).read_bytes()).hexdigest()
            result = ssh(host, f'cd {q(existing)} && sha256sum -c SHA256SUMS', check=False)
            remote = ssh(host, f'cat {q(existing + "/SHA256SUMS")}', check=False)
            entries = dict((line.split('  ', 1)[1], line.split('  ', 1)[0])
                           for line in remote.stdout.splitlines() if '  ' in line)
            if result.returncode == 0 and expected and all(entries.get(k) == v for k, v in expected.items()):
                print(f'already published at {existing}', flush=True)
                return
        partial_candidates = []
        different = False
        for existing in found:
            is_partial = existing.endswith('.partial')
            manifest_path = existing + '/details/SHA256SUMS'
            result = ssh(host, f'if [ -f {q(manifest_path)} ]; then cat {q(manifest_path)}; '
                               f'elif [ -f {q(existing + "/SHA256SUMS")} ]; then cat {q(existing + "/SHA256SUMS")}; fi')
            same = result.stdout in (manifest, changed_manifest, legacy_manifest)
            if not is_partial and same:
                print(f'already published at {existing}', flush=True)
                return
            if is_partial:
                recorded = result.stdout
                if not recorded:
                    readme = ssh(host, f'cat {q(existing + "/README.txt")}').stdout
                    if readme == base_readme:
                        recorded = manifest
                    elif readme == base_readme + reason:
                        recorded = changed_manifest
                partial_candidates.append((existing, recorded))
            else:
                different = True
        if different:
            # The reason is recorded before checksums are computed.
            with (stage / 'README.txt').open('a') as readme:
                readme.write(reason)
            manifest = write_manifest()
        partial_match = next((p for p, m in partial_candidates if m == manifest), None)
        if partial_match:
            partial = partial_match
            final = partial[:-len('.partial')]
            print(f'Resuming partial upload: {partial}', flush=True)
        else:
            index = 1
            while True:
                suffix = '' if index == 1 else f' ({index})'
                final = f'{parent}/{name}{suffix}'
                partial = final + '.partial'
                cmd = f'if [ -e {q(final)} ] || [ -e {q(partial)} ]; then exit 17; fi; mkdir {q(partial)}'
                res = ssh(host, cmd, check=False)
                if res.returncode == 0:
                    break
                if res.returncode != 17:
                    raise RuntimeError(res.stderr or 'Unable to reserve destination')
                index += 1
            # Record identity and expected content first, so interrupted transfers can resume.
            ssh(host, f'mkdir -p {q(partial + "/details")} && '
                      f'printf %s {q((stage / "README.txt").read_text())} > {q(partial + "/README.txt")} && '
                      f'printf %s {q(manifest)} > {q(partial + "/details/SHA256SUMS")}')
        shell = shlex.join(['ssh', *SSH_OPTIONS])
        subprocess.run([rsync, '-s', '-a', '--partial', '-e', shell, str(stage) + '/', f'{host}:{partial}/'], check=True)
        verify = ssh(host, f'cd {q(partial)} && sha256sum -c details/SHA256SUMS')
        print(verify.stdout, end='')
        relative = f'{when.year}/{day}/{Path(final).name}'
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
