# C2 — Feasibility check

**Owner:** CORE · **Depends on:** C1 · **Unblocks:** C3, G0
**Read first:** docs/GEOMETRY.md §3, SPEC R-FE-1..3, R-TL-12

## Goal
Before any tiling, decide whether the allowed side set can work, explain why not, and suggest sets.

## Deliverables
- `core/src/feasibility.rs`: `check(surface, params) -> Feasibility` (struct from C0).
- `generate()` runs it first; if infeasible returns `status = "infeasible"` with the message.
- `vstone-core --feasibility-only <input.json>` flag for G0.
- Test fixtures in `tests/fixtures/profiles/`: `cylinder.json`, `cone_9_to_6.json`,
  `vase_bulge.json` (r: 6 → 10 → 6 over 40 in, smooth), `hourglass.json` (r: 9 → 5 → 9).

## Acceptance
- [ ] Cylinder, `[6]`, tight: feasible, `columns` contains one even count between 8 and 12, no suggestions.
- [ ] Cone 9 → 6, `[6]`, tight: infeasible (column changes > 0); suggestions start with `[5,6,7]`;
      with `wide` variation: feasible.
- [ ] Vase bulge, `[6]`: infeasible, `pentagons_needed ≥ 1`, first suggestion contains 5.
- [ ] Hourglass, any set, `allow_waists = false`: infeasible, `waist_ranges` non-empty, message names heights.
- [ ] Cylinder radius 1.5, `bevel_cap 60`, `min_bounding_square 4`: `small_radius_ranges` covers it.
- [ ] `[5]` only on cylinder: infeasible with a suggestion containing 6 or both 5 and 7.
- [ ] Messages follow the format in GEOMETRY.md §3 (snapshot tests on the exact strings).
- [ ] Same input twice → identical output.

## Agent notes
