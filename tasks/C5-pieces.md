# C5 — Rims, stone pieces, bevels

**Owner:** CORE · **Depends on:** C4 · **Unblocks:** C6
**Read first:** docs/GEOMETRY.md §5–6, SPEC R-RM-1,2, R-ST-1..5

## Goal
Turn planes into closed stone pieces: outer face, inner face, side quads, bevel per edge.

## Deliverables
- `core/src/rims.rs` — `level` clipping and `whole` removal at each open end.
- `core/src/pieces.rs` — outer/inner polygons (shared corners), side faces, `edges[]` with
  `kind`, `neighbor`, `length`, `bevel_deg`, `side_normal`; `corner_angles_deg`, `face_depth`,
  `bounding_square`; fills CoreOutput `pieces` (temporary ids by seed index until C6).

## Acceptance
- [ ] Ring12 analytic case (build planes directly in the test, no optimizer): output matches
      `tests/fixtures/core_output_ring12.json` geometry within 1e-9.
- [ ] Cylinder: every joint bevel equals half the angle between neighbouring normals (±1e-9).
- [ ] Every side quad planar (±1e-9); shared joint quads identical on both pieces (±`JOINT_MATCH_TOL`).
- [ ] Inner polygons simple and same orientation as outer; a test with a tiny tile and t = 0.375
      returns the validation-failure flag instead of panicking.
- [ ] `level` rim: all rim edges have `kind = rim_level`, rim corners at the rim height (±1e-9),
      no partial piece below `min_bounding_square`.
- [ ] `whole` rim: no corner beyond the rim height; rim edges `rim_whole`, bevel 0.
- [ ] Sum of piece volumes = outer surface area of tiles × t within 2 % (sanity check).

## Agent notes
