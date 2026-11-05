# S7 — CLI, run folder, preview

**Owner:** SHELL · **Depends on:** S2–S6 · **Unblocks:** I1
**Read first:** SPEC R-OUT-*, AGENTS.md §7

## Goal
`vstone run <config.toml>` does the whole job and prints a short summary.

## Deliverables
- `py/vstone/cli.py` (argparse):
  - `vstone run <config> [--seed N] [--out DIR]` — load config → CoreInput → core → write outputs.
  - `vstone check <config>` — feasibility only (prints message + suggestions, exit 0/2).
  - `vstone preview <run-folder>` — rewrite `preview.glb` only.
- Run folder `runs/<YYYYMMDD>-seed<seed>-<first 8 hex of input_sha256>/` with:
  `config.toml` (copy), `core_input.json`, `core_output.json`, `assembled.step`,
  `flat_layout.step`, `assembly_map.pdf`, `report.pdf`, `pieces.csv`, `preview.glb`.
- `py/vstone/preview.py` — GLB with one mesh node per piece (named by id), triangulated from the
  faces in the core output (fan triangulation of convex faces is fine; concave faces → ear clipping).
- Exit codes: 0 ok, 2 infeasible, 3 failed validation, 1 error. Infeasible and failed runs still
  write the report.

## Acceptance
- [ ] With the core stubbed to return ring12 (env var `VSTONE_CORE_STUB=ring12`), `vstone run` on
      the example config writes all 9 files and exits 0.
- [ ] `vstone check` on the cone fixture exits 2 and prints the suggestion list (after C2 lands).
- [ ] Running twice with the same seed creates the same folder name and identical hashes for every file.
- [ ] Summary printed: status message, piece count, blanks used, run folder path.

## Agent notes
