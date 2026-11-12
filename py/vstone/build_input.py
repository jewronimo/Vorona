"""Build CoreInput dict from RunConfig and profile polyline."""

import json
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from vstone.config import RunConfig
from vstone.profile_io import ProfileError, read_and_sample_profile


def build_core_input(config: RunConfig, profile_path=None) -> dict[str, Any]:
    """
    Convert RunConfig and profile to CoreInput dict.

    Args:
        config: Run configuration
        profile_path: Path to the profile STEP file

    Returns:
        CoreInput dictionary validated against schema

    Raises:
        ProfileError: If profile sampling fails
    """
    # Sample the profile
    try:
        profile_points = (
            profile_path
            if isinstance(profile_path, list)
            else read_and_sample_profile(profile_path or config.profile)
        )
    except ProfileError:
        raise
    except (OSError, ValueError, RuntimeError) as e:
        raise ProfileError(f"Failed to sample profile from {profile_path}: {e!s}")

    # Build the CoreInput dictionary
    core_input = {
        "schema": "vstone.core_input/1",
        "surface": {"kind": config.input_kind, "profile": profile_points},
        "params": {
            "thickness": config.thickness,
            "allowed_sides": config.allowed_sides,
            "target_size": config.target_size,
            "size_variation": config.size_variation / 100,
            "randomness": config.randomness,
            "seed": config.seed,
            "ends": {"top": config.top, "bottom": config.bottom},
            "rims": {"top": config.top_rim, "bottom": config.bottom_rim},
            "min_edge": config.min_edge,
            "max_edge": config.max_edge,
            "min_corner_angle_deg": config.min_corner_angle_deg,
            "bevel_cap_deg": config.bevel_cap_deg,
            "depth": config.depth,
            "max_corner_angle_deg": config.max_corner_angle_deg,
            "min_bounding_square": config.min_bounding_square,
            "allow_waists": config.allow_waists,
            "blank_size": config.blanks_size,
            "blank_usable": config.blanks_usable,
            "nest_gap": config.blanks_nest_gap,
        },
    }

    # Validate against schema (basic validation)
    _validate_core_input(core_input)
    schema = Path(__file__).resolve().parents[2] / "docs/schema/core_input.schema.json"
    Draft202012Validator(json.loads(schema.read_text())).validate(core_input)

    return core_input


def _validate_core_input(core_input: dict[str, Any]) -> None:
    """Basic validation of CoreInput against the schema."""
    # Check required top-level keys
    required_keys = ["schema", "surface", "params"]
    for key in required_keys:
        if key not in core_input:
            raise ValueError(f"Missing required key in CoreInput: {key}")

    if core_input["schema"] != "vstone.core_input/1":
        raise ValueError(f"Unsupported CoreInput schema: {core_input['schema']}")

    # Check surface
    if "surface" not in core_input:
        raise ValueError("Missing 'surface' in CoreInput")

    surface = core_input["surface"]
    required_surface_keys = ["kind", "profile"]
    for key in required_surface_keys:
        if key not in surface:
            raise ValueError(f"Missing required key in surface: {key}")

    if surface["kind"] != "revolve":
        raise ValueError(f"Unsupported surface kind: {surface['kind']}")

    # Check profile (should be a list of [x, y] points)
    if not isinstance(surface["profile"], list):
        raise TypeError("'profile' must be a list")

    for i, point in enumerate(surface["profile"]):
        if not isinstance(point, list) or len(point) != 2:
            raise TypeError(f"Profile point {i} must be [x, y] list")
        if not all(isinstance(coord, (int, float)) for coord in point):
            raise TypeError(f"Profile point {i} coordinates must be numbers")

    # Check params
    if "params" not in core_input:
        raise ValueError("Missing 'params' in CoreInput")

    params = core_input["params"]
    required_params_keys = [
        "thickness",
        "allowed_sides",
        "target_size",
        "size_variation",
        "randomness",
        "seed",
        "ends",
        "rims",
        "min_edge",
        "max_edge",
        "min_corner_angle_deg",
        "bevel_cap_deg",
        "depth",
        "max_corner_angle_deg",
        "min_bounding_square",
        "allow_waists",
        "blank_size",
        "blank_usable",
        "nest_gap",
    ]

    for key in required_params_keys:
        if key not in params:
            raise ValueError(f"Missing required key in params: {key}")

    # Validate specific param types and values
    if not isinstance(params["thickness"], (int, float)):
        raise TypeError("'thickness' must be a number")

    if not isinstance(params["allowed_sides"], list):
        raise TypeError("'allowed_sides' must be a list")

    for side in params["allowed_sides"]:
        if not isinstance(side, int) or side < 4 or side > 9:
            raise TypeError(
                "All values in 'allowed_sides' must be integers between 4 and 9"
            )

    if not isinstance(params["target_size"], (int, float)):
        raise TypeError("'target_size' must be a number")

    if not 0.05 <= params["size_variation"] <= 0.50:
        raise ValueError(
            f"'size_variation' must be 'tight' or 'wide', got '{params['size_variation']}'"
        )

    if not isinstance(params["randomness"], (int, float)):
        raise TypeError("'randomness' must be a number")

    if not isinstance(params["seed"], int):
        raise TypeError("'seed' must be an integer")

    # Validate ends
    if "ends" not in params:
        raise ValueError("Missing 'ends' in params")

    ends = params["ends"]
    required_ends_keys = ["top", "bottom"]
    for key in required_ends_keys:
        if key not in ends:
            raise ValueError(f"Missing required key in ends: {key}")

    if ends["top"] != "open":
        raise ValueError(f"'top' in ends must be 'open', got '{ends['top']}'")

    if ends["bottom"] != "open":
        raise ValueError(f"'bottom' in ends must be 'open', got '{ends['bottom']}'")

    # Validate rims
    if "rims" not in params:
        raise ValueError("Missing 'rims' in params")

    rims = params["rims"]
    required_rims_keys = ["top", "bottom"]
    for key in required_rims_keys:
        if key not in rims:
            raise ValueError(f"Missing required key in rims: {key}")

    if rims["top"] not in ["level", "whole"]:
        raise ValueError(
            f"'top' in rims must be 'level' or 'whole', got '{rims['top']}'"
        )

    if rims["bottom"] not in ["level", "whole"]:
        raise ValueError(
            f"'bottom' in rims must be 'level' or 'whole', got '{rims['bottom']}'"
        )

    # Validate numeric parameters
    numeric_params = [
        "min_edge",
        "max_edge",
        "min_corner_angle_deg",
        "bevel_cap_deg",
        "depth",
        "max_corner_angle_deg",
        "min_bounding_square",
        "blank_size",
        "blank_usable",
        "nest_gap",
    ]

    for key in numeric_params:
        if not isinstance(params[key], (int, float)):
            raise TypeError(f"'{key}' must be a number")

    # Validate boolean parameters
    if not isinstance(params["allow_waists"], bool):
        raise TypeError("'allow_waists' must be a boolean")
