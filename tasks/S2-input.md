# S2 — Run config and STEP profile reader

**Owner:** SHELL · **Depends on:** S1 · **Unblocks:** S7, G0
**Read first:** SPEC R-IN-1..3, examples/cylinder_18x60.toml, docs/INTERFACE.md (CoreInput)

## Goal
Turn a TOML config + STEP profile into a valid CoreInput.

## Deliverables
- `py/vstone/config.py` — load TOML (`tomllib`), validate types, ranges and enums, fill defaults,
  produce `RunConfig` (dataclass). Clear error messages naming the key.
- `py/vstone/profile_io.py` — read STEP AP214 with build123d; require exactly one open edge
  chain in the XY plane (|z| ≤ 1e-6); sample to a polyline with chord error ≤ 0.01 in;
  return `[[x, y], ...]`. Lines, arcs and splines supported.
- `py/vstone/build_input.py` — `RunConfig` + polyline → CoreInput dict (validated against
  `docs/schema/core_input.schema.json`).
- Fixture generator `tests/py/make_profiles.py` writing STEP files: cylinder (line x=9, y 0..60),
  cone 9→6, vase (spline), hourglass (spline), arc+line mix, and bad ones (closed, crosses axis,
  two curves, not in XY). Commit the generated files under `tests/fixtures/step/` and
  `examples/cylinder_18x60_profile.step`.

## Acceptance
- [x] Example config loads; produces CoreInput equal to `tests/fixtures/core_input_cylinder.json`
      except for profile sampling density (compare endpoints and radius).
- [x] Each bad STEP raises `ProfileError` with a message that names the problem.
- [x] Config errors: wrong type, side count 3 or 10, unknown rim style → `ConfigError` naming the key.
- [x] No geometry decisions in Python (sampling a curve is allowed; everything else goes to core).

## Agent notes

- Based on unreviewed draft/S2-qwen-v1; replaced BREP import/export and placeholder
  chain handling with actual AP214 STEP, connected-wire validation and OpenCascade
  deflection sampling. Import converts CAD millimetres to contract inches; fixtures
  explicitly export inch units with fixed timestamps.
- A04 percent/default/shortcut conversion is covered by test_a04_percent_conversion,
  test_a04_invalid_percent and test_defaults. Runtime JSON Schema validation uses
  jsonschema (added dependency, required by this card).
- Acceptance mapping in tests/py/test_input.py: example → test_example_matches_core_fixture;
  invalid STEP → test_bad_step_names_problem; config errors → test_config_errors_name_key;
  shell boundary → manual source audit plus test_no_geometry_decisions_in_shell.
  test_supported_curves_chord_error checks dense reference samples against output
  segments for all five supported profiles. test_polyline_input_is_preserved covers
  direct polyline input.
- Parent make dev-check: runs/logs/S2-verified-1001.log, exit 0; Rust fmt/clippy/tests,
  ruff and all 36 Python cases pass.
- Installed missing libgl1/libglu1-mesa into Vorona-dev and added them to
  Dockerfile.dev so build123d imports on clean builds. No other containers touched.
- S2 remains unmerged pending Jerry's review.
