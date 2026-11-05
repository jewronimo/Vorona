# AGENTS.md — rules for every agent working in this repo

Read this file, `PLAN.md`, and your task card in `tasks/` before touching code.
The project owner is Jerry. He approves contract changes and does visual checks in SolidWorks.

## 1. Who does what

| Agent | Model | Owns | Never touches |
| --- | --- | --- | --- |
| CORE | ChatGPT | `core/` (Rust crate): all geometry, all algorithms, all randomness | `py/` (except reading it) |
| SHELL | Qwen3-Coder 32B (+ sub-agents) | `py/` (Python package): config, STEP in/out, PDF/CSV, CLI, preview export | `core/src/` |
| OWNER | Jerry | `docs/SPEC.md`, `docs/INTERFACE.md` approval, visual gates | — |

Task cards are tagged `CORE`, `SHELL` or `GATE` (a check only Jerry can do).

**Hard boundary:** every computation that decides geometry happens in Rust. Python only
reads files, converts formats, draws pictures and writes files. If a SHELL task seems to
need math on tile geometry (anything beyond applying a 4×4 transform), stop and escalate.

## 2. Escalation

- SHELL needs a change in core output or behaviour → write `docs/requests/NNN-short-name.md`
  (what, why, example) and stop that part of the task. Do not work around it in Python.
- Anyone needs to change `docs/INTERFACE.md` or the JSON schemas → open a request; Jerry approves;
  CORE implements it; SHELL updates its mirror in the same pull request or the next one.
- A task card is wrong or impossible → write the problem at the bottom of the card under
  `## Agent notes`, and stop.

## 3. Determinism rules (CORE — mandatory)

Same input JSON + same seed must give byte-identical output JSON on the same machine,
and values equal within 1e-9 in on any machine.

1. All randomness comes from one `rand_chacha::ChaCha8Rng` seeded from `params.seed`.
   Never `thread_rng`, never time, never addresses, never environment.
2. No `HashMap`/`HashSet` iteration that can affect output. Use `Vec`, `BTreeMap`, `BTreeSet`.
3. Every sort uses a total order: `f64::total_cmp`, ties broken by stable integer index.
4. No parallelism in v1. (Later: `rayon` only for pure maps collected in order.)
5. Iterative solvers stop on fixed iteration caps and fixed thresholds. No wall-clock timeouts.
6. `f64` everywhere. No `f32`, no fast-math, no `mul_add` unless used everywhere consistently.
7. Floats are serialized with `serde_json` defaults. Output keys follow struct field order.
8. `cargo test` includes a determinism test: run the same input twice, compare SHA-256 of output.

SHELL rules: no randomness at all; iterate pieces in id order; write files in a fixed order;
STEP headers get a fixed timestamp (`1970-01-01T00:00:00`) so files are reproducible.

## 4. Units and tolerances

- Inches, `f64`, right-handed, Y up. Revolve axis = global Y.
- Angles in the JSON are degrees; inside the code use radians.
- Shared constants live in one place each: `core/src/tol.rs` and `py/vstone/tol.py`.
  `GEOM_EPS = 1e-9`, `CORNER_ON_SURFACE_TOL = 1e-3`, `JOINT_MATCH_TOL = 1e-4`.

## 5. How to work a task

1. Create branch `task/<card-id>` (example `task/C3-topology`).
2. Read the card's **Depends on** cards and `docs/` sections it names.
3. Write or update tests first when the card lists acceptance tests.
4. Keep each commit small and runnable. Message: `<card-id>: <what>`.
5. Run `make check` (Rust fmt + clippy + tests, Python ruff + pytest). It must pass.
6. Tick the card's checklist, add `## Agent notes` if anything deviated, open a PR.

No new dependency unless the card lists it, or you add it under `## Agent notes` with a reason.

## 6. Definition of done

- Every acceptance item on the card passes, `make check` is green.
- No TODOs left in code touched by the task (write them in `## Agent notes` instead).
- Public functions have a doc comment saying units and what can fail.
- A `GATE` card is only done when Jerry writes `Approved` under it.

## 7. Commands

```
make check        # everything below
make core-test    # cd core && cargo fmt --check && cargo clippy -- -D warnings && cargo test
make py-test      # ruff check py && pytest -q
make dev          # maturin develop (builds the Rust extension into the Python env)
vstone run <config.toml>          # full run (after S7)
vstone-core <input.json>          # Rust-only run, prints output JSON (after C1)
```
