"""Serialize acceptance in the container and own its complete process group."""

import fcntl
import os
from pathlib import Path
import signal
import subprocess
import sys


LOCK = Path("/tmp/vstone-acceptance.lock")


def main():
    action, stamp = sys.argv[1:]
    state = Path(f"/tmp/vstone-acceptance-{stamp}.pid")
    cancelled = Path(f"/tmp/vstone-acceptance-{stamp}.cancelled")
    if action == "stop":
        cancelled.touch()
        if state.exists():
            pid = int(state.read_text())
            try:
                command = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
                if stamp.encode() in command and b"run" in command:
                    os.kill(pid, signal.SIGTERM)
            except (FileNotFoundError, ProcessLookupError):
                pass
        return 0
    if action != "run":
        raise ValueError("expected run or stop")
    with LOCK.open("a+") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("Acceptance already running in container; refusing to start.", flush=True)
            return 1
        child = None

        def terminate(signum, frame):
            raise SystemExit(128 + signum)

        for signum in (signal.SIGHUP, signal.SIGINT, signal.SIGTERM):
            signal.signal(signum, terminate)
        try:
            state.write_text(str(os.getpid()))
            if cancelled.exists():
                return 143
            blocked = {signal.SIGHUP, signal.SIGINT, signal.SIGTERM}
            previous = signal.pthread_sigmask(signal.SIG_BLOCK, blocked)
            try:
                child = subprocess.Popen(
                    ["bash", "scripts/acceptance.sh"], start_new_session=True
                )
            finally:
                signal.pthread_sigmask(signal.SIG_SETMASK, previous)
            return child.wait()
        finally:
            for signum in (signal.SIGHUP, signal.SIGINT, signal.SIGTERM):
                signal.signal(signum, signal.SIG_IGN)
            if child is not None:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                child.wait()
            state.unlink(missing_ok=True)


if __name__ == "__main__":
    sys.exit(main())
