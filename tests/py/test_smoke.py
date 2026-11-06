"""Shared contract and tolerance acceptance checks for S1."""

import json
import re
from pathlib import Path

from vstone import tol
from vstone.core import generate

ROOT = Path(__file__).resolve().parents[2]


def test_core_contract():
    input_data = json.loads(
        (ROOT / "tests/fixtures/core_input_cylinder.json").read_text()
    )
    assert generate(input_data)["schema"] == "vstone.core_output/1"


def test_shared_tolerances():
    source = (ROOT / "core/src/tol.rs").read_text()
    for name in ("GEOM_EPS", "CORNER_ON_SURFACE_TOL", "JOINT_MATCH_TOL"):
        match = re.search(rf"\b{name}\s*:\s*f64\s*=\s*([^;]+);", source)
        assert match is not None, name
        assert float(match.group(1).replace("_", "")) == getattr(tol, name)


def test_cli_entry_point():
    from importlib.metadata import entry_points

    (entry,) = [e for e in entry_points(group="console_scripts") if e.name == "vstone"]
    assert entry.load().__name__ == "main"
