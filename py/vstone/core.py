"""JSON-only bridge; all geometry decisions belong to Rust."""

import json


def generate(input_data: dict) -> dict:
    """Call Rust with inch-based input; reject unsupported contract versions."""
    if input_data.get("schema") != "vstone.core_input/1":
        raise ValueError("Unsupported CoreInput schema")
    from vstone import vstone_core

    output = json.loads(vstone_core.generate_json(json.dumps(input_data)))
    if not isinstance(output, dict) or output.get("schema") != "vstone.core_output/1":
        raise ValueError("Unsupported CoreOutput schema")
    return output
