# Amendment A03 — Two contract fixes found in review

**Date:** 2026-11-06 · **Approved by:** Jerry · **Amends:** docs/INTERFACE.md, tasks/C6, tasks/S1

## 1. `map_uv` was mirrored

INTERFACE.md defined `map_uv = (R_ref·θ, s)`. Seen from outside the structure, θ increases to the
**left**, so that map was a mirror image of the real object: neighbours on the map sat on the wrong side.

**New definition:** `map_uv = (R_ref·φ, s)` with `φ = (2π − θ) mod 2π`, so the map reads like the
structure seen from outside. Pieces crossing the seam (`φ = 0`) are unwrapped to the right-hand end.
Every `map_uv` polygon is counter-clockwise (positive signed area), like `outer` seen from outside.

- `tests/fixtures/core_output_ring12.json` updated to the new definition (only `map_uv` changed).
- `core/tests/contract.rs` asserts every `map_uv` polygon has positive area.
- C6 must produce `map_uv` this way. Piece numbering is unchanged (counter-clockwise seen from +Y),
  so on the map and on the object the numbers increase from right to left.

## 2. Python package layout

The Rust extension was installed as a separate top-level module and the `vstone` package itself
was not part of the install, so the `vstone` command could not work outside pytest.

**New layout:** maturin mixed project — `python-source = "py"`, `module-name = "vstone.vstone_core"`.
- Python imports the core as `from vstone import vstone_core` (INTERFACE.md's `vstone_core.generate_json`
  is now `vstone.vstone_core.generate_json`).
- `pip install -e .` installs both the `vstone` package and the `vstone` console script.
- `tests/py/test_smoke.py` checks the `vstone` console script resolves to `vstone.cli:main`.
