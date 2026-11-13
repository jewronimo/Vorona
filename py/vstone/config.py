"""Load and validate TOML configuration into RunConfig dataclass."""

import math
import tomllib
from dataclasses import dataclass
from typing import Literal


@dataclass
class RunConfig:
    """Configuration for a run."""

    # Input section
    input_kind: Literal["revolve"]  # Only "revolve" supported for now
    profile: str  # Path to STEP file

    # Stone section
    thickness: float

    # Pattern section
    allowed_sides: list[int]  # Subset of 4..9
    target_size: float
    size_variation: float
    randomness: float
    seed: int

    # Ends section
    top: Literal["open"]
    bottom: Literal["open"]
    top_rim: Literal["level", "whole"]
    bottom_rim: Literal["level", "whole"]

    # Limits section
    min_edge: float
    max_edge: float
    min_corner_angle_deg: float
    bevel_cap_deg: float
    depth: float
    max_corner_angle_deg: float
    min_bounding_square: float
    allow_waists: bool

    # Blanks section
    blanks_size: float
    blanks_usable: float
    blanks_nest_gap: float

    # Output section
    output_dir: str
    preview: bool


class ConfigError(ValueError):
    """Invalid run configuration; messages identify the key."""


def load_config(path: str) -> RunConfig:
    """Load inch-based TOML settings, raising ConfigError for invalid keys."""
    try:
        config = _load_config(path)
        for key in ("profile", "output_dir"):
            if not isinstance(getattr(config, key), str) or not getattr(config, key):
                raise ConfigError(f"{key}: expected nonempty string")
        for key, value in vars(config).items():
            if key in (
                "thickness",
                "target_size",
                "min_edge",
                "max_edge",
                "min_corner_angle_deg",
                "bevel_cap_deg",
                "depth",
                "min_bounding_square",
                "blanks_size",
                "blanks_usable",
            ) and (
                type(value) not in (int, float)
                or not math.isfinite(value)
                or value <= 0
            ):
                raise ConfigError(f"{key}: expected finite positive number")
        if not config.allowed_sides or any(
            type(side) is not int for side in config.allowed_sides
        ):
            raise ConfigError("allowed_sides: expected nonempty integer list")
        if len(set(config.allowed_sides)) != len(config.allowed_sides):
            raise ConfigError("allowed_sides: duplicate values")
        if type(config.seed) is not int or not 0 <= config.seed < 2**64:
            raise ConfigError("seed: expected unsigned 64-bit integer")
        if (
            type(config.randomness) not in (int, float)
            or not 0 <= config.randomness <= 1
        ):
            raise ConfigError("randomness: expected number in 0..1")
        if (
            type(config.blanks_nest_gap) not in (int, float)
            or not math.isfinite(config.blanks_nest_gap)
            or config.blanks_nest_gap < 0
        ):
            raise ConfigError("nest_gap: expected finite nonnegative number")
        if config.min_edge > config.max_edge:
            raise ConfigError("min_edge: exceeds max_edge")
        if config.blanks_usable > config.blanks_size:
            raise ConfigError("blanks_usable: exceeds blanks_size")
        for key in ("min_corner_angle_deg", "bevel_cap_deg"):
            if getattr(config, key) >= 180:
                raise ConfigError(f"{key}: must be below 180 degrees")
        if (
            type(config.max_corner_angle_deg) not in (int, float)
            or not 180 < config.max_corner_angle_deg < 270
        ):
            raise ConfigError("max_corner_angle_deg: expected 180 < angle < 270")
        return config
    except (ValueError, TypeError, OSError, KeyError) as error:
        raise ConfigError(str(error)) from error


def _load_config(path: str) -> RunConfig:
    """Load TOML file and validate it into a RunConfig."""
    with open(path, "rb") as f:
        data = tomllib.load(f)
    defaults = {
        "input": {"kind": "revolve"},
        "stone": {"thickness": 0.375},
        "pattern": {
            "allowed_sides": [6],
            "target_size": 6.5,
            "size_variation": 15,
            "randomness": 0.0,
            "seed": 42,
        },
        "ends": {
            "top": "open",
            "bottom": "open",
            "top_rim": "level",
            "bottom_rim": "level",
        },
        "limits": {
            "min_edge": 1.0,
            "max_edge": 6.0,
            "min_corner_angle_deg": 70.0,
            "bevel_cap_deg": 60.0,
            "depth": 1.5,
            "max_corner_angle_deg": 230.0,
            "min_bounding_square": 4.0,
            "allow_waists": False,
        },
        "blanks": {"size": 12.0, "usable": 11.5, "nest_gap": 0.25},
        "output": {"dir": "runs", "preview": True},
    }
    for section in data:
        if section not in defaults:
            raise ConfigError(f"{section}: unknown section")
    for section, values in defaults.items():
        supplied = data.get(section, {})
        if not isinstance(supplied, dict):
            raise ConfigError(f"{section}: expected table")
        for key in supplied:
            if key not in values and (section, key) != ("input", "profile"):
                raise ConfigError(f"{section}.{key}: unknown key")
        data[section] = values | supplied

    # Validate input section
    if "input" not in data:
        raise ValueError("Missing [input] section")

    input_data = data["input"]
    if "kind" not in input_data:
        raise ValueError("[input] section missing 'kind' key")

    if input_data["kind"] != "revolve":
        raise ValueError(
            f"Unsupported input kind '{input_data['kind']}', only 'revolve' is supported"
        )

    if "profile" not in input_data:
        raise ValueError("[input] section missing 'profile' key")

    # Validate stone section
    if "stone" not in data:
        raise ValueError("Missing [stone] section")

    stone_data = data["stone"]
    if "thickness" not in stone_data:
        raise ValueError("[stone] section missing 'thickness' key")

    thickness = stone_data["thickness"]
    if not isinstance(thickness, (int, float)):
        raise TypeError("[stone] section 'thickness' must be a number")

    # Validate pattern section
    if "pattern" not in data:
        raise ValueError("Missing [pattern] section")

    pattern_data = data["pattern"]
    pattern_data.setdefault("size_variation", 15)
    required_pattern_keys = ["allowed_sides", "target_size", "randomness", "seed"]
    for key in required_pattern_keys:
        if key not in pattern_data:
            raise ValueError(f"[pattern] section missing '{key}' key")

    allowed_sides = pattern_data["allowed_sides"]
    if not isinstance(allowed_sides, list):
        raise TypeError("[pattern] section 'allowed_sides' must be a list")

    for side in allowed_sides:
        if not isinstance(side, int) or side < 4 or side > 9:
            raise ValueError(
                f"All values in 'allowed_sides' must be integers between 4 and 9, got {side}"
            )

    target_size = pattern_data["target_size"]
    if not isinstance(target_size, (int, float)):
        raise TypeError("[pattern] section 'target_size' must be a number")

    size_variation = pattern_data["size_variation"]
    if isinstance(size_variation, str):
        size_variation = {"tight": 15, "wide": 40}.get(size_variation)
    if (
        type(size_variation) not in (int, float)
        or not math.isfinite(size_variation)
        or not 5 <= size_variation <= 50
    ):
        raise ConfigError("pattern.size_variation must be 5–50 percent, tight or wide")

    randomness = pattern_data["randomness"]
    if not isinstance(randomness, (int, float)):
        raise TypeError("[pattern] section 'randomness' must be a number")

    seed = pattern_data["seed"]
    if not isinstance(seed, int):
        raise TypeError("[pattern] section 'seed' must be an integer")

    # Validate ends section
    if "ends" not in data:
        raise ValueError("Missing [ends] section")

    ends_data = data["ends"]
    required_ends_keys = ["top", "bottom", "top_rim", "bottom_rim"]
    for key in required_ends_keys:
        if key not in ends_data:
            raise ValueError(f"[ends] section missing '{key}' key")

    top = ends_data["top"]
    if top != "open":
        raise ValueError(f"[ends] section 'top' must be 'open', got '{top}'")

    bottom = ends_data["bottom"]
    if bottom != "open":
        raise ValueError(f"[ends] section 'bottom' must be 'open', got '{bottom}'")

    top_rim = ends_data["top_rim"]
    if top_rim not in ["level", "whole"]:
        raise ValueError(
            f"[ends] section 'top_rim' must be 'level' or 'whole', got '{top_rim}'"
        )

    bottom_rim = ends_data["bottom_rim"]
    if bottom_rim not in ["level", "whole"]:
        raise ValueError(
            f"[ends] section 'bottom_rim' must be 'level' or 'whole', got '{bottom_rim}'"
        )

    # Validate limits section
    if "limits" not in data:
        raise ValueError("Missing [limits] section")

    limits_data = data["limits"]
    required_limits_keys = [
        "min_edge",
        "max_edge",
        "min_corner_angle_deg",
        "bevel_cap_deg",
        "depth",
        "min_bounding_square",
        "allow_waists",
    ]
    for key in required_limits_keys:
        if key not in limits_data:
            raise ValueError(f"[limits] section missing '{key}' key")

    min_edge = limits_data["min_edge"]
    if not isinstance(min_edge, (int, float)):
        raise TypeError("[limits] section 'min_edge' must be a number")

    max_edge = limits_data["max_edge"]
    if not isinstance(max_edge, (int, float)):
        raise TypeError("[limits] section 'max_edge' must be a number")

    min_corner_angle_deg = limits_data["min_corner_angle_deg"]
    if not isinstance(min_corner_angle_deg, (int, float)):
        raise TypeError("[limits] section 'min_corner_angle_deg' must be a number")

    bevel_cap_deg = limits_data["bevel_cap_deg"]
    if not isinstance(bevel_cap_deg, (int, float)):
        raise TypeError("[limits] section 'bevel_cap_deg' must be a number")

    depth = limits_data["depth"]
    if not isinstance(depth, (int, float)):
        raise TypeError("[limits] section 'depth' must be a number")

    min_bounding_square = limits_data["min_bounding_square"]
    if not isinstance(min_bounding_square, (int, float)):
        raise TypeError("[limits] section 'min_bounding_square' must be a number")

    allow_waists = limits_data["allow_waists"]
    if not isinstance(allow_waists, bool):
        raise TypeError("[limits] section 'allow_waists' must be a boolean")

    # Validate blanks section
    if "blanks" not in data:
        raise ValueError("Missing [blanks] section")

    blanks_data = data["blanks"]
    required_blanks_keys = ["size", "usable", "nest_gap"]
    for key in required_blanks_keys:
        if key not in blanks_data:
            raise ValueError(f"[blanks] section missing '{key}' key")

    blanks_size = blanks_data["size"]
    if not isinstance(blanks_size, (int, float)):
        raise TypeError("[blanks] section 'size' must be a number")

    blanks_usable = blanks_data["usable"]
    if not isinstance(blanks_usable, (int, float)):
        raise TypeError("[blanks] section 'usable' must be a number")

    blanks_nest_gap = blanks_data["nest_gap"]
    if not isinstance(blanks_nest_gap, (int, float)):
        raise TypeError("[blanks] section 'nest_gap' must be a number")

    # Validate output section
    if "output" not in data:
        raise ValueError("Missing [output] section")

    output_data = data["output"]
    required_output_keys = ["dir", "preview"]
    for key in required_output_keys:
        if key not in output_data:
            raise ValueError(f"[output] section missing '{key}' key")

    output_dir = output_data["dir"]
    preview = output_data["preview"]
    if not isinstance(preview, bool):
        raise TypeError("[output] section 'preview' must be a boolean")

    # Create RunConfig instance
    config = RunConfig(
        input_kind="revolve",
        profile=input_data["profile"],
        thickness=thickness,
        allowed_sides=allowed_sides,
        target_size=target_size,
        size_variation=size_variation,
        randomness=randomness,
        seed=seed,
        top="open",
        bottom="open",
        top_rim=top_rim,
        bottom_rim=bottom_rim,
        min_edge=min_edge,
        max_edge=max_edge,
        min_corner_angle_deg=min_corner_angle_deg,
        bevel_cap_deg=bevel_cap_deg,
        depth=depth,
        max_corner_angle_deg=limits_data["max_corner_angle_deg"],
        min_bounding_square=min_bounding_square,
        allow_waists=allow_waists,
        blanks_size=blanks_size,
        blanks_usable=blanks_usable,
        blanks_nest_gap=blanks_nest_gap,
        output_dir=output_dir,
        preview=preview,
    )

    return config
