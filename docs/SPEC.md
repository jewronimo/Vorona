# SPEC — Voronoi stone tile generator (agent copy)

Snapshot of the owner's spec. If this file and a task card disagree, this file wins;
if this file is unclear, escalate (AGENTS.md §2). Requirement IDs are cited by task cards.

Values marked **(default, open)** are working defaults Jerry has not confirmed yet.
Keep them as config values, never hard-coded.

## Scope

- **R-SC-1** Personal tool for Jerry. CLI first; a UI comes later.
- **R-SC-2** v1 input: one profile curve revolved 360° around Y. Uploaded 3D surface comes later (milestone L1).
- **R-SC-3** v1 ends are open. Closed ends (caps) are out of scope.
- **R-SC-4** Out of v1: triangles, multiple stone colours, hand editing of the pattern, structural analysis, Pegasus CAM export (milestone L2), UI (milestone L3).

## Input

- **R-IN-1** Profile file: STEP AP214 containing one open curve in the XY plane (lines, arcs, splines). The curve is the **outer face** of the stone.
- **R-IN-2** Reject profiles that cross or touch the Y axis, are closed, are not planar in XY, or contain more than one curve.
- **R-IN-3** All settings come from a TOML run config (see `examples/cylinder_18x60.toml`).

## Tiles

- **R-TL-1** Every piece is a flat polygon with 4–9 sides. No triangles.
- **R-TL-2** Exactly three pieces meet at every corner (3-valent).
- **R-TL-3** Every tile corner lies on the true outer surface (within `CORNER_ON_SURFACE_TOL`); faces sit just inside. Nothing extends past the drawn shape.
- **R-TL-4** Allowed side counts are a user set within 4–9: one count (e.g. `[6]`) or several (`[5,6,7]`).
- **R-TL-5** Piece size targets `target_size`. Variation: `tight` = ±15 %, `wide` = ±40 % **(default, open: percentages)**.
- **R-TL-6** `randomness` 0 = regular pattern, 1 = fully organic; works with every side set.
- **R-TL-7** Same config + same seed = identical result.
- **R-TL-8** Max size: the piece's footprint fits in the usable square of a blank, `blanks.usable` = 11.5 **(default, open)**.
- **R-TL-9** Min size: the smallest square that encloses the piece has side ≥ `min_bounding_square` = 4.0 **(default, open: interpreted as "bounding square ≥ 4", i.e. longest extent ≥ 4)**.
- **R-TL-10** Edge length within [`min_edge`, `max_edge`] = [1.0, 6.0] **(default, open)**.
- **R-TL-11** Every corner angle ≥ `min_corner_angle_deg` = 70 **(default, open)**.
- **R-TL-12** Shapes with pinched (waist, negative-curvature) regions are rejected when `allow_waists = false` (default for v1).

## Feasibility

- **R-FE-1** Before generating, check whether the allowed side set can work for this profile and size range.
- **R-FE-2** If not, stop and report why (in plain words, with counts) and list alternative side sets that would work, nearest first. Never switch sets automatically.
- **R-FE-3** Also report profile sections whose radius is too small for the size and bevel limits.

## Rims

- **R-RM-1** Each open end has a rim style: `level` (cut level; partial pieces kept, exempt from side count) or `whole` (partial pieces dropped; rim follows tile edges, uneven).
- **R-RM-2** With `level`, seeds are placed so no partial piece falls below the minimum size.

## Stone and joints

- **R-ST-1** Stone thickness `t` = 0.375, constant, measured perpendicular to each tile, inward from the outer face.
- **R-ST-2** Each joint face is the bisector plane of the two neighbouring tiles; both pieces share it exactly.
- **R-ST-3** Bevel (tilt of the joint face from a square cut) ≤ `bevel_cap_deg` = 60 (input; may be lowered later).
- **R-ST-4** Joints are designed tight (no gap). Kerf is handled later by CAM.
- **R-ST-5** Level rim edges are cut by the horizontal rim plane; `whole` rim edges are square cuts (0° bevel).

## Outputs (one folder per run: `runs/<YYYYMMDD>-seed<seed>-<short hash>/`)

- **R-OUT-1** `assembled.step` — one solid per piece named `P001`, `P002`, … in final position.
- **R-OUT-2** `flat_layout.step` — every piece outer face up, nested onto 12 × 12 blanks, numbered.
- **R-OUT-3** `assembly_map.pdf` — the structure unrolled flat; each piece shows its number and each edge shows the number of the piece it meets.
- **R-OUT-4** `report.pdf` + `pieces.csv` — validation results; per piece: sides, size, edge lengths, bevel angles, blank number.
- **R-OUT-5** `core_output.json` and the run config copy, for reproducibility.
- **R-OUT-6** `preview.glb` for quick 3D viewing (later wired into the cam-processor viewer).
- **R-OUT-7** Pegasus CAM files — later (L2).

## Validation (run is accepted only if all pass; failures list piece ids)

- **R-VA-1** side count in allowed set (partial rim pieces exempt)
- **R-VA-2** footprint fits usable square; bounding square ≥ minimum
- **R-VA-3** edge lengths in range
- **R-VA-4** corner angles ≥ minimum; polygons convex unless waists are allowed
- **R-VA-5** bevels ≤ cap
- **R-VA-6** faces planar; every shared joint face identical on both pieces
- **R-VA-7** corners on surface; face depth inside surface ≤ `surface_tolerance`
- **R-VA-8** no overlaps, no gaps (topology closed; every interior edge has exactly two pieces)
- **R-VA-9** report totals: piece count by side count, blanks used, where 5/7-sided pieces were forced

## Example

- **R-EX-1** Reference case: 18 in diameter × 60 in tall cylinder, open both ends, `[6]` only.
  Expect ~10–12 columns around, pieces ~6–7 in, bevels ~15–18°, faceted look intended.
