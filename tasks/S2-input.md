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
- [ ] Example config loads; produces CoreInput equal to `tests/fixtures/core_input_cylinder.json`
      except for profile sampling density (compare endpoints and radius).
- [ ] Each bad STEP raises `ProfileError` with a message that names the problem.
- [ ] Config errors: wrong type, side count 3 or 10, unknown rim style → `ConfigError` naming the key.
- [ ] No geometry decisions in Python (sampling a curve is allowed; everything else goes to core).

## Agent notes
