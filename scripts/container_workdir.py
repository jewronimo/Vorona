"""Resolve a host worktree through the existing container's bind mounts."""
import json
import pathlib
import subprocess
import sys

container, directory = sys.argv[1:]
root = pathlib.Path(directory).resolve()
info = json.loads(subprocess.check_output(['docker', 'inspect', container]))[0]
for mount in sorted(info['Mounts'], key=lambda m: len(m['Source']), reverse=True):
    if mount['Type'] != 'bind':
        continue
    try:
        relative = root.relative_to(pathlib.Path(mount['Source']).resolve())
    except ValueError:
        continue
    print(pathlib.PurePosixPath(mount['Destination']) / relative.as_posix())
    break
else:
    sys.exit('Worktree is not mounted in ' + container)
