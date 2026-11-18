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

### Implementation for review — 2026-11-18

Branch `task/C7-nesting`, separate worktree `../voronoi-stone-c7`, base
`cee456d`. Added rigid area-centroid flattening, deterministic longest-edge
selection, normal/thickness/planarity guards, and deterministic conservative
nesting. Outer AND inner projections are enclosed by rotated rectangles;
rectangle separation proves at least the requested polygon clearance, including
inner-face overhang. Unlike the suggested exhaustive 0.05-inch grid, placement
uses sorted bottom-left rectangle contact coordinates with 15-degree rotations
at each position, first existing blank first. This finite contact search avoids
grid cost and packs ring12 as three pieces per blank (four blanks). It is not
an optimal polygon nester; conservative boxes can waste material or reject a
piece that would fit at an unsampled angle. No schema or dependency changes.

A06 production output evaluates C7 instead of the pending gate, fills blanks and
counts, and only reports `ok` when all C6 and C7 checks pass. Legacy output also
gets transforms/nesting but retains its explicit C6 validation/numbering pending
gate (outside this card); its contract assertion now reflects that distinction.

Targeted development evidence (not certification):
`runs/logs/C7-targeted.log` — layout tests (ring12 four blanks, both faces,
clearance/bounds, repeat placements, invalid inputs, area centroid and longest
edge) plus C6 validation regressions and library clippy. Runs inside `Vorona-dev`
using a copied isolated source tree `/tmp/vstone-c7` and isolated target dir
`/tmp/vstone-c7-target`; no host Rust. Parent owns official `make dev-check`,
acceptance, numerical production follow-up and review. Checklist stays pending
those gates. No merge, push, or official checks executed by this child.
