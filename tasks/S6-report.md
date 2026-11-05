# S6 — Report PDF and pieces CSV

**Owner:** SHELL · **Depends on:** S1, C0 (stub fixture) · **Unblocks:** I1
**Read first:** SPEC R-OUT-4, R-VA-1..9, R-FE-2

## Goal
One-page-first report a person reads in a minute, plus a CSV with every number per piece.

## Deliverables
- `py/vstone/report.py`:
  - `write_csv(core_output, path)` — one row per piece: `id, sides, partial, blank, bounding_square,
    face_depth, min_edge, max_edge, min_corner_angle, max_bevel, edge_lengths (;-joined),
    bevels (;-joined), neighbors (;-joined)`; numbers with 3 decimals.
  - `write_report(core_output, config, path)` — page 1: status line (the core `message`),
    pass/fail table of checks with failing ids, stats (pieces by sides, blanks used, max bevel,
    max face depth), feasibility summary and suggestions if any, config summary.
    Page 2+: per-piece table.
  - Infeasible runs: report shows the feasibility message and suggestions only.

## Acceptance
- [ ] Ring12: CSV has 12 rows; header exact; values match the fixture to 3 decimals.
- [ ] A hand-edited copy of the fixture with `status = failed_validation` shows the failing checks first.
- [ ] An infeasible fixture (write one: `[6]` on a cone) produces a one-page report with suggestions.
- [ ] Byte-identical on repeat writes.

## Agent notes
