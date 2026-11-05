# voronoi-stone

Generates flat, beveled stone tiles in a Voronoi-style pattern that cover a revolved
surface, ready to waterjet from 12 × 12 in tiles and epoxy together.

- `docs/SPEC.md` — what the tool must do (requirements with IDs)
- `docs/GEOMETRY.md` — the math the core implements
- `docs/INTERFACE.md` — the JSON contract between the Rust core and the Python shell
- `PLAN.md` — milestones, order of work, who does what
- `AGENTS.md` — rules every agent follows
- `tasks/` — one card per unit of work
- `examples/` — run configs

Layout once built:

```
core/      Rust crate `vstone-core` (lib + binary + PyO3 module)
py/vstone/ Python package and `vstone` CLI
tests/     shared fixtures and golden files
runs/      run outputs (git-ignored)
```
