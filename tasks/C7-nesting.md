# C7 — Flat transforms and nesting

**Owner:** CORE · **Depends on:** C6 · **Unblocks:** I1
**Read first:** docs/GEOMETRY.md §7, SPEC R-OUT-2, R-TL-8, docs/INTERFACE.md (`flat_xform`, `blank_xform`)

## Goal
Lay each piece flat (outer face up) and pack pieces onto 12 × 12 blanks.

## Deliverables
- `core/src/flat.rs` — `flat_xform` per piece.
- `core/src/nest.rs` — deterministic nesting into `blank_usable` squares with `nest_gap`;
  fills `blank`, `blank_xform`, `blanks[]`, `stats.blanks_used`.

## Acceptance
- [ ] After `flat_xform`, every outer corner has z = 0 (±1e-9), inner corners z = −t (±1e-9),
      outer-face centroid at the origin, longest edge along +X.
- [ ] Nested footprints never overlap and keep `nest_gap` (polygon distance test).
- [ ] Every footprint lies inside its blank's usable square.
- [ ] Ring12: 12 pieces of 4.659 × 6 fit 3 per blank with the default gap (two upright side by
      side, one turned 90° above them: 6.25 + 4.659 = 10.909 ≤ 11.5), so 4 blanks.
- [ ] Deterministic placements on repeat runs.

## Agent notes
