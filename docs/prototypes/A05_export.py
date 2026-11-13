"""Run the unmodified A05 prototype and export its EXACT inputs and SciPy solution.
Usage: D=1.5 python A05_export.py OUTPUT.json [symmetric|random] [seed]
Run in a scratch directory: the reference also writes tiles_MODE_SEED.json there.
Requires the same numpy/scipy as the reference; not production shell geometry.
"""
import hashlib
import json
from pathlib import Path
import runpy
import sys

output = Path(sys.argv[1]).resolve()
reference = Path(__file__).with_name("A05_depth_proto.py")
sys.argv = [str(reference), *(sys.argv[2:] or ["symmetric", "1"])]
g = runpy.run_path(str(reference))
keys = {"points": "P3", "radial": "radial", "etheta": "etheta",
        "targets": "targets", "triangles": "TRI", "start": "x0",
        "fitted": None, "reference_corners": "CORN", "keep": "keep"}
data = {key: (g["fit"].x if value is None else g[value]) for key, value in keys.items()}
data = {key: value.tolist() if hasattr(value, "tolist") else value for key, value in data.items()}
data["identity"] = {"source_sha256": hashlib.sha256(reference.read_bytes()).hexdigest(),
                    "depth": g["D"], "mode": g["mode"], "seed": g["seed"],
                    "n": g["n"], "w": g["w"], "h": g["h"], "rows": g["rows"],
                    "numpy": g["np"].__version__, "scipy": __import__("scipy").__version__}
data["identity"]["arrays_sha256"] = hashlib.sha256(json.dumps(data, sort_keys=True).encode()).hexdigest()
output.write_text(json.dumps(data, sort_keys=True) + "\n")
print("exported exact reference:", output, data["identity"])
