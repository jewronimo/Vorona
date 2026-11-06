# S1 — Repo scaffold, Makefile, Python package

**Owner:** SHELL · **Depends on:** C0 · **Unblocks:** everything after
**Read first:** AGENTS.md, README.md, PLAN.md (stack)

## Goal
One command checks the whole repo; the Rust core is importable from Python.

## Deliverables
- `pyproject.toml` — maturin build backend pointing at `core/` with feature `python`;
  package `vstone` in `py/vstone/`; console script `vstone = vstone.cli:main`.
  Dependencies: `build123d`, `numpy`, `reportlab`, `trimesh`; dev: `pytest`, `ruff`.
- `py/vstone/__init__.py`, `py/vstone/tol.py` (same constants as `core/src/tol.rs`),
  `py/vstone/core.py` — thin wrapper: `generate(input: dict) -> dict` calling
  `vstone_core.generate_json` and checking `schema` fields.
- `Makefile` with targets from AGENTS.md §7.
- `.gitignore` (`runs/`, `target/`, `.venv/`, `__pycache__/`, `*.so`).
- `tests/py/test_smoke.py`.

## Acceptance
- [x] Fresh clone: `python -m venv .venv && pip install -e . && make dev && make check` passes.
      (Per A02: verified as `make dev-check` inside `Vorona-dev`; host installs do not count.)
- [x] `test_smoke.py`: `vstone.core.generate(<cylinder fixture>)` returns a dict with
      `schema == "vstone.core_output/1"`.
- [x] `tol.py` and `tol.rs` values are equal (a test parses `tol.rs`).
- [~] Works on Windows and Linux — Linux (arm64 container) yes; Windows not tested and not needed: A01/A02 make the MacBook dev container the platform.

## Out of scope
Geometry of any kind (AGENTS.md §1).

## Agent notes

- 2026-11-06 (Claude, review before handoff): `make dev-check` in `Vorona-dev` on the MacBook,
  rustc/cargo 1.99.1, Python 3.12.16: cargo fmt/clippy clean, 4/4 Rust tests, ruff clean, 3/3 pytest, EXIT=0.
  Log: `runs/logs/dev-check.log` (local, gitignored).
- Packaging changed per A03 §2: maturin mixed layout (`python-source = "py"`, `module-name = "vstone.vstone_core"`),
  core imported as `from vstone import vstone_core`; new test checks the `vstone` console script.
