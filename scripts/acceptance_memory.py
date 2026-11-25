"""Sample Docker's reported container memory usage every five seconds."""
import re
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

container, directory = sys.argv[1:]
out = Path(directory)
stop = threading.Event()
signal.signal(signal.SIGTERM, lambda *_: stop.set())
signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGTERM})
peak = 0
units = {'B': 1, 'KiB': 1024, 'MiB': 1024**2, 'GiB': 1024**3,
         'kB': 1000, 'MB': 1000**2, 'GB': 1000**3}
with (out / 'docker-memory-samples.txt').open('w', buffering=1) as log:
    while not stop.is_set():
        started = time.monotonic()
        try:
            result = subprocess.run(['docker', 'stats', '--no-stream', '--format', '{{.MemUsage}}', container],
                                    capture_output=True, text=True, timeout=4, check=True)
            usage = result.stdout.strip().split('/')[0].strip()
            match = re.fullmatch(r'([\d.]+)\s*([A-Za-z]+)', usage)
            if not match or match[2] not in units:
                raise ValueError(f'unrecognized docker memory: {usage}')
            value = int(float(match[1]) * units[match[2]])
            peak = max(peak, value)
            log.write(f'{time.time():.3f} {value} {usage}\n')
        except (OSError, subprocess.SubprocessError, ValueError) as error:
            log.write(f'{time.time():.3f} ERROR {error}\n')
        stop.wait(max(0, 5 - (time.monotonic() - started)))
(out / 'container-peak-memory-bytes.txt').write_text(f'{peak}\n')
