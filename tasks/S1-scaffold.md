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
- [ ] Fresh clone: `python -m venv .venv && pip install -e . && make dev && make check` passes.
- [ ] `test_smoke.py`: `vstone.core.generate(<cylinder fixture>)` returns a dict with
      `schema == "vstone.core_output/1"`.
- [ ] `tol.py` and `tol.rs` values are equal (a test parses `tol.rs`).
- [ ] Works on Windows and Linux (note anything Windows-specific in `## Agent notes`).

## Out of scope
Geometry of any kind (AGENTS.md §1).

## Agent notes
