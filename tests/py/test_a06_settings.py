"""A06 settings transport and pattern-specific defaults, without geometry."""

import pytest
from vstone.build_input import build_core_input
from vstone.config import ConfigError, load_config


@pytest.mark.parametrize("pattern,depth,angle", [
    (None, 1.0, 30.0), ("voronoi_triangles", 1.0, 30.0),
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
    assert params["depth"] == depth
    assert params["min_corner_angle_deg"] == angle
    assert params["min_triangle_edge"] == 4.0
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
])
def test_bad_setting_names_key(tmp_path, key, value):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[limits]\n' + f'{key} = {value}\n'
    )
    with pytest.raises(ConfigError, match=key):
        load_config(config_path)


def test_unknown_pattern(tmp_path):
    config_path = tmp_path / "config.toml"
    config_path.write_text(
        '[input]\nprofile = "unused.step"\n[pattern]\npattern = "unknown"\n'
    )
    with pytest.raises(ConfigError, match="pattern"):
        load_config(config_path)
