"""A06 settings transport and pattern-specific defaults, without geometry."""

import pytest
from vstone.build_input import build_core_input
from vstone.config import ConfigError, load_config


@pytest.mark.parametrize("pattern,depth,angle", [
    (None, 0.3, 30.0), ("voronoi_triangles", 0.3, 30.0),
    ("voronoi", 1.5, 70.0),
])
def test_pattern_defaults(tmp_path, pattern, depth, angle):
    config_path = tmp_path / "config.toml"
    text = '[input]\nprofile = "unused.step"\n'
    if pattern is not None:
        text += f'[pattern]\npattern = "{pattern}"\n'
    config_path.write_text(text)
    config = load_config(config_path)
    params = build_core_input(config, [[9., 0.], [9., 60.]])["params"]
    assert params["pattern"] == (pattern or "voronoi_triangles")
    assert params["depth"] == pytest.approx(depth)
    assert params["min_corner_angle_deg"] == angle
    assert params["min_triangle_edge"] == (4.0 if pattern == "voronoi" else 1.5)
    assert params["max_polygon_edge"] == 3.0
    assert params["count_step"] == 2
    assert params["max_back_gap"] == 0.5
    assert params["relief_base"] == "auto"


@pytest.mark.parametrize("base", ['"auto"', "0", "0.75"])
def test_smooth_and_explicit_transport(tmp_path, base):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[limits]\n'
        f'depth = 0\nrelief_base = {base}\nmax_back_gap = 0\n'
        'min_corner_angle_deg = 45\n'
    )
    config = load_config(config_path)
    params = build_core_input(config, [[9., 0.], [9., 60.]])["params"]
    assert params["depth"] == 0
    assert params["max_back_gap"] == 0
    assert params["min_corner_angle_deg"] == 45
    assert params["relief_base"] == config.relief_base


@pytest.mark.parametrize("key,value", [
    ("depth", "-1"), ("depth", "nan"), ("depth", "true"),
    ("min_triangle_edge", "0"), ("min_triangle_edge", "inf"),
    ("min_triangle_edge", "true"), ("max_back_gap", "-1"),
    ("max_back_gap", "nan"), ("max_back_gap", "false"),
    ("relief_base", '"unknown"'), ("relief_base", "-1"),
    ("relief_base", "inf"), ("relief_base", "true"),
    ("max_polygon_edge", "0"), ("max_polygon_edge", "nan"),
    ("max_polygon_edge", "true"),
])
def test_bad_setting_names_key(tmp_path, key, value):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[limits]\n' + f'{key} = {value}\n'
    )
    with pytest.raises(ConfigError, match=key):
        load_config(config_path)


@pytest.mark.parametrize("pattern", ["voronoi_triangles", "voronoi"])
@pytest.mark.parametrize("depth", [None, 0.0, 0.75])
def test_depth_default_tracks_size_only_when_omitted(tmp_path, pattern, depth):
    config_path = tmp_path / "config.toml"
    text = (
        '[input]\nprofile = "unused.step"\n'
        f'[pattern]\npattern = "{pattern}"\ncount_step = 4\n'
        '[limits]\nmax_polygon_edge = 5.0\n'
    )
    if depth is not None:
        text += f"depth = {depth}\n"
    config_path.write_text(text)
    if pattern == "voronoi" and depth == 0:
        with pytest.raises(ConfigError, match="depth"):
            load_config(config_path)
        return
    config = load_config(config_path)
    expected = depth if depth is not None else (1.5 if pattern == "voronoi" else 0.5)
    assert config.depth == pytest.approx(expected)
    assert config.count_step == 4


@pytest.mark.parametrize("value", ["0", "-2", "2.5", "true"])
def test_count_step_requires_positive_integer(tmp_path, value):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[pattern]\n' + f"count_step = {value}\n"
    )
    with pytest.raises(ConfigError, match="count_step"):
        load_config(config_path)


def test_triangle_size_bounds_consistent(tmp_path):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[limits]\n'
        'min_triangle_edge = 3.5\nmax_polygon_edge = 3\n'
    )
    with pytest.raises(ConfigError, match="min_triangle_edge"):
        load_config(config_path)


def test_unknown_pattern(tmp_path):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[pattern]\npattern = "unknown"\n'
    )
    with pytest.raises(ConfigError, match="pattern"):
        load_config(config_path)
