"""Compatibility entry point; canonical runner uses owner 60s/90s deadlines.
Historical all-four-fit-first artifacts remain archived; this is not that solver.
"""
import runpy
from pathlib import Path

runpy.run_path(str(Path(__file__).with_name("C4_production_matrix_runner.py")),
               run_name="__main__")
