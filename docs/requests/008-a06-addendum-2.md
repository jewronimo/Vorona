# A06 addendum 2 — small symmetric stones, constant rows (drafted by Claude 2026-11-17; owner approved in chat)

Append to `amendments/A06-voronoi-triangles.md` as section 9 on a docs branch, then review. Applies to
`pattern = "voronoi_triangles"` only. Where this section and sections 1–8 disagree, this section wins.
Resolves the C4 blocker "unequal-count row transition unsupported".

## 9.1 Why
Owner review of prototypes (2026-11-17): small stones with constant rows and symmetric relief look
right on cylinder, cone and vase; large stones looked coarse, and random relief on small stones failed.

## 9.2 Stone size (replaces §3 `min_triangle_edge` default and §8.3 seed spacing)
| Setting | Default | Meaning |
|---|---|---|
| `max_polygon_edge` (new) | 3.0 in | every polygon side **and** every triangle side (they are the same edges) |
| `min_triangle_edge` | **1.5 in** (was 4.0) | every triangle side, including the base |
| `min_edge` | 1.0 in | polygon sides (unchanged) |
| `min_corner_angle_deg` | 30 | unchanged |
| `blank_usable` | 11.5 in | unchanged |
Triangles are isosceles in the symmetric layout: symmetric about their own vertical centreline, base
and legs may differ. Polygons are likewise symmetric about their vertical centreline.

## 9.3 Row layout (replaces §2.5 per-row counts and §8.4 C2 counts)
1. **Constant count, constant row height.** Choose one seed count `n` per row for the whole surface and
   one physical row height `h` (arc length along the profile). Stones narrow or widen with the radius;
   rows stay level. Width per row `w(s) = 2π r(s) / n`.
2. **Choosing n and h (deterministic):** among integer `n ≥ 3`, take those for which every row satisfies,
   for the rectified triangles (base `w/2`, legs `½·√(w²/4 + h²)`):
   base ≥ `min_triangle_edge`, base and legs ≤ `max_polygon_edge`, apex angle ≥ `min_corner_angle_deg`,
   piece fits `blank_usable`. Choose `h = (√3/2)·w(median row)` clipped into the feasible range; pick the
   feasible `n` whose median-row triangle is closest to equilateral; tie → smaller `n`. Report n, h and
   the per-row base/leg range.
3. **If no single n is feasible:** split the surface into the fewest stretches of constant count; at each
   change the count changes by a step `g` (default 2, setting `count_step`) and the 5/7 pairs sit at `g`
   equally spaced angles. Report every change (row, old → new count). Each stretch keeps constant row
   height.
4. Prototype reference: cylinder 9 in radius → n = 10; cone 9 → 6 in → n = 10, h = 4.5 in; vase 6–10 in →
   n = 11, h = 4.5 in (no count change needed).

## 9.4 Relief
- Default `randomness = 0` (symmetric): row depth pattern 0, D/2, D (§8.2) on top of the auto base;
  every stone in a row identical.
- Default `depth` = **0.1 × `max_polygon_edge`** (0.3 in at 3 in; prototype passed at 0.25 in). Explicit
  `depth` overrides.
- **Random relief (`randomness > 0`) is experimental:** allowed, reported, never a merge gate until
  reworked. Prototype: 0.25 in random relief on 3 in stones failed (torn pieces).

## 9.5 Symmetric solve (replaces the "solve one row and copy" rule where counts change)
- Constant-count stretches: solve one representative stone per row (2 tilts) plus its corner slides,
  copy by exact rotation `2π/n`. Gate: equal spacing and equal size per row within 1e-6.
- At a count change (9.3.3): solve one sector of angle `2π/g` and copy `g` times; gate the `g`-fold
  repeat within 1e-6.

## 9.6 Acceptance (replaces §5 matrix for this pattern)
Required, all limits + closure + flatness + determinism + rotational gate:
- Cylinder 18×60, cone 9→6, vase bulge: smooth (`depth = 0`) and symmetric relief at the default depth.
- Report only: symmetric relief at 2× default depth; random relief at default depth, seeds 1..3.
- STEP export of all six required cases for owner review.

## 9.7 Prototype reproduction
`docs/prototypes/A06_tri_proto.py` is superseded for this section by the variant used in chat
(`COUNT=constrow:N:H`, `TMIN`, `MAXE`, `RELIEF=rows`, `BASE=auto`); results: cone n=10 h=4.5 smooth
PASS (sides 1.95–2.70 in, taper ≤ 11°), relief 0.25 PASS; vase n=11 h=4.5 smooth PASS (sides 1.69–2.92 in,
corners −0.06..+0.21 in), relief 0.25 PASS (taper ≤ 13°, back gap median 0.07 in).
