"""Sample container cgroup memory while the acceptance host is alive."""
from pathlib import Path
import signal
import sys
import time

output = Path(sys.argv[1])
running = True

def stop(_signal, _frame):
    global running
    running = False

signal.signal(signal.SIGTERM, stop)
peak = 0
while running:
    peak = max(peak, int(Path('/sys/fs/cgroup/memory.current').read_text()))
    time.sleep(.25)
output.write_text(f'{peak}\n')
