"""S2 acceptance tests against real STEP fixtures and the approved contract."""

import ast
import json
import math
from itertools import pairwise
from pathlib import Path

import pytest
from build123d import Vector, import_step
from vstone.build_input import build_core_input
from vstone.config import ConfigError, load_config
from vstone.profile_io import ProfileError, read_and_sample_profile

ROOT = Path(__file__).resolve().parents[2]
STEP = ROOT / "tests/fixtures/step"
EXAMPLE = ROOT / "examples/cylinder_18x60.toml"


def test_example_matches_core_fixture():
    config = load_config(EXAMPLE)
    actual = build_core_input(config, ROOT / config.profile)
    expected = json.loads(
        (ROOT / "tests/fixtures/core_input_cylinder.json").read_text()
    )
    assert actual["params"] == expected["params"]
    assert actual["surface"]["kind"] == expected["surface"]["kind"]
    assert actual["surface"]["profile"][0] == pytest.approx([9, 0])
    assert actual["surface"]["profile"][-1] == pytest.approx([9, 60])
    assert all(point[0] == pytest.approx(9) for point in actual["surface"]["profile"])


@pytest.mark.parametrize(
    ("name", "message"),
    [
        ("closed", "closed"),
        ("crosses_axis", "axis"),
        ("touches_axis", "axis"),
        ("two_curves", "curve"),
        ("not_xy", "XY"),
    ],
)
def test_bad_step_names_problem(name, message):
    with pytest.raises(ProfileError, match=message):
        read_and_sample_profile(STEP / f"{name}.step")


@pytest.mark.parametrize("name", ["cylinder", "cone", "vase", "hourglass", "mixed"])
def test_supported_curves_chord_error(name):
    points = read_and_sample_profile(STEP / f"{name}.step")
    assert len(points) >= 2
    assert points[0] == pytest.approx([9, 0])
    assert points[-1][1] == pytest.approx(60)
    segments = [(Vector(*start), Vector(*end)) for start, end in pairwise(points)]
    for edge in import_step(STEP / f"{name}.step").scale(1 / 25.4).edges():
        for index in range(1001):
            point = edge.position_at(index / 1000)
            distances = []
            for start, end in segments:
                delta = end - start
                fraction = max(0, min(1, (point - start).dot(delta) / delta.dot(delta)))
                distances.append((point - (start + delta * fraction)).length)
            assert min(distances) <= 0.010001


@pytest.mark.parametrize(
    ("old", "new", "key"),
    [
        ("thickness = 0.375", 'thickness = "bad"', "thickness"),
        ("allowed_sides = [6]", "allowed_sides = [3]", "allowed_sides"),
        ("allowed_sides = [6]", "allowed_sides = [10]", "allowed_sides"),
        ('top_rim = "level"', 'top_rim = "unknown"', "top_rim"),
        ("randomness = 0.0", "randomness = true", "randomness"),
        ("seed = 42", "seed = true", "seed"),
        ("min_edge = 1.0", "min_edge = nan", "min_edge"),
    ],
)
def test_config_errors_name_key(tmp_path, old, new, key):
    path = tmp_path / "bad.toml"
    path.write_text(EXAMPLE.read_text().replace(old, new))
    with pytest.raises(ConfigError, match=key):
        load_config(path)


@pytest.mark.parametrize(
    ("value", "fraction"),
    [
        ("5", 0.05),
        ("20", 0.20),
        ("50", 0.50),
        ('"tight"', 0.15),
        ('"wide"', 0.40),
        ("15.5", 0.155),
    ],
)
def test_a04_percent_conversion(tmp_path, value, fraction):
    path = tmp_path / "variation.toml"
    path.write_text(
        EXAMPLE.read_text().replace(
            'size_variation = "tight"', f"size_variation = {value}"
        )
    )
    config = load_config(path)
    assert config.size_variation == fraction * 100
    assert math.isclose(
        build_core_input(config, STEP / "cylinder.step")["params"]["size_variation"],
        fraction,
    )


@pytest.mark.parametrize("value", ["4", "51", "true", "nan", '"unknown"', "[]"])
def test_a04_invalid_percent(tmp_path, value):
    path = tmp_path / "bad.toml"
    path.write_text(
        EXAMPLE.read_text().replace(
            'size_variation = "tight"', f"size_variation = {value}"
        )
    )
    with pytest.raises(ConfigError, match="size_variation"):
        load_config(path)


def test_defaults(tmp_path):
    path = tmp_path / "minimal.toml"
    path.write_text('[input]\nprofile = "profile.step"\n')
    assert load_config(path).size_variation == 15


def test_polyline_input_is_preserved():
    points = [[9.0, 0.0], [9.0, 60.0]]
    result = build_core_input(load_config(EXAMPLE), points)
    assert result["surface"]["profile"] == points


def test_no_geometry_decisions_in_shell():
    for name in ("config", "build_input", "profile_io"):
        tree = ast.parse((ROOT / f"py/vstone/{name}.py").read_text())
        imports = [
            node.module or ""
            for node in ast.walk(tree)
            if isinstance(node, ast.ImportFrom)
        ]
        assert not any("core" in module for module in imports)
    assert not list((ROOT / "py/vstone").glob("*feasibility*"))


@pytest.mark.parametrize(("key", "value"), [
    ("depth", "0"), ("depth", "nan"), ("depth", "true"),
    ("max_corner_angle_deg", "170"), ("max_corner_angle_deg", "180"),
    ("max_corner_angle_deg", "270"), ("max_corner_angle_deg", "nan"),
    ("max_corner_angle_deg", "true"), ("surface_tolerance", "0.75"),
])
def test_a05_invalid_settings(tmp_path, key, value):
    path = tmp_path / "bad.toml"
    path.write_text('[input]\nprofile = "profile.step"\n[limits]\n' + f'{key} = {value}\n')
    with pytest.raises(ConfigError, match=key):
        load_config(path)


def test_a05_defaults_and_core_keys(tmp_path):
    path = tmp_path / "minimal.toml"
    path.write_text('[input]\nprofile = "profile.step"\n')
    for config in (load_config(path), load_config(EXAMPLE)):
        assert config.depth == 2.0
        assert config.max_corner_angle_deg == 225.0
        assert config.thickness == 0.375
        params = build_core_input(config, [[9., 0.], [9., 60.]])["params"]
        assert params["depth"] == 2.0
        assert params["max_corner_angle_deg"] == 225.0
        assert "surface_tolerance" not in params
