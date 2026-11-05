# C0 — Rust crate, contract types, JSON schemas, stub fixture

**Owner:** CORE · **Depends on:** nothing · **Unblocks:** S1, S3–S6, C1
**Read first:** AGENTS.md, docs/INTERFACE.md

## Goal
Create the Rust crate and freeze the data contract so both tracks can start.

## Deliverables
- `core/Cargo.toml` — crate `vstone-core`, lib + bin `vstone-core`, feature `python` (PyO3 module `vstone_core`).
- `core/src/lib.rs` — `pub fn generate(input: &CoreInput) -> CoreOutput` (returns `status = "error"`,
  message `"not implemented"` for now) and `pub fn generate_json(s: &str) -> String`.
- `core/src/model.rs` — serde structs exactly matching docs/INTERFACE.md (field order = JSON order).
- `core/src/tol.rs` — constants from AGENTS.md §4.
- `core/src/main.rs` — reads a JSON file path, prints output JSON.
- `core/src/py.rs` — `#[pyfunction] generate_json(input_json: &str) -> String` behind feature `python`.
- `docs/schema/core_input.schema.json`, `docs/schema/core_output.schema.json` (JSON Schema draft 2020-12).
- `tests/fixtures/core_input_cylinder.json` — the example config as CoreInput (profile: two points
  `[9,0]`, `[9,60]` resampled every 0.5 in).
- `tests/fixtures/core_output_ring12.json` — hand-checkable stub described in INTERFACE.md.
- `core/tests/contract.rs`.

## Acceptance
- [x] `cargo test` passes; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean.
- [x] Both fixtures deserialize into the structs and re-serialize byte-identically.
- [x] Both fixtures validate against the JSON Schemas (use `jsonschema` crate in a dev-dependency test).
- [x] In `core_output_ring12.json`: 12 pieces, each 4 sides, face plane distance 8.693 (±1e-3),
      tile width 4.659 (±1e-3), joint bevels 15.0°, rim bevels 0.0°, inner face offset 0.375,
      every joint quad planar (±1e-9), shared joint faces identical on both pieces.
      These checks are a Rust test that recomputes them from the JSON.
- [x] The stub is complete: `flat_xform`, `map_uv`, `validation` (all passed) and nesting filled,
      3 pieces per blank (two upright, one turned 90° above), 4 blanks — same layout C7 must reproduce.
- [x] Determinism test: `generate_json` on the cylinder input twice → same SHA-256.

## Out of scope
Any real geometry. The stub output is hand-built (computed in the test helper), not generated.

## Agent notes


### Implementation and fixture conventions
- Implemented only `core/`, `docs/schema/`, `tests/fixtures/`, and this card. No branches or commits, per explicit task instruction.
- Ring12 is a separate six-inch band, not the generated output of the sixty-inch, six-sided cylinder example. Added `core_input_ring12.json` (radius 9, height 6, allowed sides [4], target size 6) to give the stub a real matching input SHA-256. The twelve rectangles are `partial: true` because both level rims cut them; `stats.by_sides` therefore reports `partial: 12`. All nine validation checks concern this ring input, not the cylinder input.
- Boundary edges have `neighbor: null`; range arrays are pairs of heights in inches. These previously unspecified empty/boundary cases are represented by `Option<String>` and `[f64; 2]` respectively; no existing contract fields or versions were changed.
- Flat +X follows the six-inch edge. Two pieces are rotated upright on each blank, and the third lies horizontally above them. Usable origin is (0.25, 0.25); blank origins advance 14 inches. Full outer/inner projected footprints keep 0.25-inch clearance.
- Dependency reasons: serde derives/serde_json implement the ordered contract; `float_roundtrip` preserves fixture f64 values exactly; sha2 supplies required determinism/provenance hashes; jsonschema supplies the mandated draft-2020-12 tests; optional PyO3 exposes the prescribed module. Optional `pyo3-build-config` adds required macOS extension-module linker flags. ABI3 targets Python 3.11+ (native import/call verified on 3.14.5).
- Toolchain installed in `/tmp/vstone-cargo` and `/tmp/vstone-rustup`, with no shell rc edits. Prefix commands with `PATH=/tmp/vstone-cargo/bin:$PATH CARGO_HOME=/tmp/vstone-cargo RUSTUP_HOME=/tmp/vstone-rustup`.

### Checks and S1 handoff
- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`: pass (4 integration tests, including schema, byte-identical serialization, geometry, nesting, provenance, and determinism).
- `cargo clippy --all-targets --features python -- -D warnings`: pass.
- `cargo build --features python --lib`: pass; `vstone_core` native extension directly loaded and called successfully on Python 3.14.5.
- `make check PYTHON=.venv/bin/python`: Rust checks and ruff pass; ordinary pytest fails because `vstone_core` is not installed. Both scaffold pytest tests pass when the native library is explicitly loaded into that process.
- **S1 packaging blocker, outside C0 ownership:** `maturin develop --features python --skip-install` rejects the parent's provisional `python-source = "py"`, `module-name = "vstone_core"` layout because `py/vstone_core` does not exist. Parent must reconcile its Python package layout with maturin before an installed-module `make check` can pass. The Rust module name and `PyInit_vstone_core` export match the contract; C0 does not change the Python scaffold to mask this failure.
