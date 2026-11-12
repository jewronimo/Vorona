# Amendment A05 — Depth space (Voronoi points in a band under the surface)

**Date:** 2026-11-11 · **Approved by:** Jerry · **Amends:** docs/SPEC.md (R-TL-3, R-TL-11, R-TL-12,
R-ST-1, R-VA-4, R-VA-7, R-OUT-4), docs/GEOMETRY.md §2 and §4, docs/INTERFACE.md and both JSON schemas,
examples/cylinder_18x60.toml, A04 (nothing changed there), the C2 surface-dip change of 2026-11-10,
tasks C2, C3, C4, C5, C6 · **Adds:** task S2b. Read after A04. Where this amendment and an earlier file
disagree, this amendment wins.

## 1. Why

The rule "every tile corner lies on the true surface" (R-TL-3) cannot be met on surfaces that are
straight in one direction (cylinders, cones) by flat convex pieces with three pieces per corner.
Gauss–Bonnet forces bulge-and-pinch, and the pinches need pieces with an inward corner (known
planar-hexagon result; reproduced in Jerry's prototype 2026-11-10: sphere exact, cylinder degenerate).
C4 failed for this reason, not because of a coding error.

Jerry's design replaces the rule with a **depth space**: points live in a band under the surface, each
point defines one flat polygon, and the polygons may sit wherever the geometry puts them.

## 2. The model (owner decision)

1. **Outer border** = the input surface (v1: the revolved profile; L1: an uploaded surface).
2. **Depth space** = everything between the outer border and the outer border offset inward by
   `depth` (inches, setting, default **2.0**).
3. **Points (seeds)** are placed in the depth space: a position on the surface plus a depth
   `0 ≤ dᵢ ≤ depth` along the inward normal. Random or symmetric (see C3 below).
4. **One plane per point, through the point.** Neighbouring polygons always share their edges;
   corners are where three neighbouring planes meet (three pieces per corner, R-TL-2 unchanged).
   The result is a closed faceted shell. Polygons are irregular and need not be symmetric.
5. **Faces and corners may lie inside or outside the depth space**, including outside the drawn
   surface. The depth space only governs where points go. How far corners end up from the outer
   border is reported, not limited.
6. **Stones** = the shell offset inward by `thickness` (setting, default **0.375**, may be thicker).
   Joints are the bisector planes of neighbouring faces (GEOMETRY §6, unchanged), so both pieces
   share each joint face exactly.
7. **Pieces may have inward (reflex) corners.** Every convex corner angle ≥ `min_corner_angle_deg`
   (unchanged, 70); every reflex corner ≤ `max_corner_angle_deg` (new setting, default **225**).
8. **Depth controls relief.** Larger depth = more uneven surface. Depth also decides whether pieces
   are well shaped on straight parts: the prototype (18 in cylinder, 6.5 in pieces) gave good pieces
   at 1–2 in and slivers below 0.5 in or distortion above 3 in. The core warns, it does not refuse,
   when `depth` is outside `[0.15, 0.30] × target_size`.
9. **Ready for uploaded surfaces (L1).** Everything after seeding (C4, C5, C6) uses only generic
   surface queries (point, inward normal, closest point, inward offset). No cylinder- or
   revolution-specific formulas outside the surface module and the (u, v) seeding of C3.

## 3. Changes to existing files

### docs/SPEC.md
- **R-TL-3** replaced: every seed lies in the depth space. Corners and faces are unconstrained by the
  surface; their signed distance to the outer border (+ = outside) is reported as min / max / median.
- **R-TL-11** extended: convex corners ≥ `min_corner_angle_deg`; reflex corners ≤ `max_corner_angle_deg`.
- **R-TL-12**: `allow_waists = false` stays the default, but waists are no longer a geometric
  impossibility (reflex corners are allowed); with `allow_waists = true` they are generated normally.
- **R-ST-1**: thickness is the setting `thickness` (default 0.375), constant within a run.
- **R-VA-4** replaced: convex corner angles ≥ minimum, reflex corner angles ≤ maximum, polygons simple
  (no self-intersection), inner polygon simple with the same orientation as the outer one.
- **R-VA-7** replaced: every seed within the depth space (tolerance `GEOM_EPS`); corner/face distance to
  the outer border reported, not validated.
- **R-OUT-4** adds per piece: seed depth, max corner distance from the outer border, reflex corner count.

### Settings, INTERFACE.md, schemas, example config
- Add `depth` (number > 0, default 2.0) and `max_corner_angle_deg` (180 < x < 270, default 225) to
  `[limits]` / CoreInput params.
- Remove `surface_tolerance` everywhere (config, CoreInput, schemas, fixtures, example).
- `thickness` already exists; document it as "default 0.375, may be thicker".

### GEOMETRY.md
- §2 seeding: seeds carry a depth (see C3 below). Topology stays Delaunay in (u, v).
- §4 is replaced by C4 below. The "important known result" paragraph is superseded by §1 of this
  amendment.
- Face depth (end of §4) is no longer a limit; it is reported.

### C2 (feasibility)
- Revert the surface-dip column rule of 2026-11-10 (commit 379d848 and follow-ups): column counts are
  chosen from the size band only. Keep the width-per-column-count information in the message.
- Add a depth warning (not infeasible): "depth X in is outside the recommended range Y–Z in for
  target size W in; expect slivers (too shallow) or distorted pieces (too deep)."

### C3 (seeds and topology)
- Each seed gets a depth `dᵢ`, deterministic from `seed`:
  - `randomness = 0` (symmetric): depth follows the row index in a repeating 3-row pattern
    `0, depth/2, depth`, so every seed's upper and lower neighbours differ in depth. (A 2-row
    alternation is mirror-symmetric and gives vertical planes that never meet in a point.)
  - `randomness > 0`: uniform in `[0, depth]`, blended with the symmetric pattern by `randomness`.
- Positions, Delaunay topology and side-count control are unchanged.
- New test: every seed's depth is in `[0, depth]`; the symmetric pattern has no two vertically
  adjacent seeds at the same depth.

### C5 (pieces)
- Unchanged formulas (inner corners `v − t·w`, bisector joints). Must handle reflex corners: the
  inner polygon check (simple, same orientation) is the guard. Thickness comes from `thickness`.

### C6 (validation)
- Implement R-VA-4 and R-VA-7 as replaced above. Report the corner-distance statistics.

## 4. Task card — C4 (revised): planes through points

**Owner:** CORE · **Depends on:** C3 (with seed depths) · **Unblocks:** C5
**Read first:** this amendment §2; GEOMETRY §6; `docs/prototypes/A05_depth_proto.py` (reference
only, not production code). Background: Pluta et al., "PH-CPF: Planar Hexagonal Meshing using
Coordinate Power Fields", SIGGRAPH 2021 (arXiv 2105.02507): face shape vs curvature (convex / brick /
bow-tie) and its Appendix C planarization. Note the opposite trade-off: PH-CPF keeps corners on the
surface and accepts ~1 % non-planarity; we need exactly flat stone, so planarity is by construction
and surface closeness is the flexible part.

### Goal
For each seed `i` find a plane `πᵢ` through the seed's 3D position `pᵢ` such that the corners
(three-plane intersections over the Delaunay triangles) form good pieces within the limits.

### Method (suggested; "or better, documented")
- Unknowns: the normal of each plane, 2 parameters per seed (tilt about the surface's two tangent
  directions); the plane passes through `pᵢ` by construction.
- Target for each corner `(i, j, k)`: the point equidistant from the three seeds in (u, v) (the Voronoi
  vertex = circumcentre of the Delaunay triangle), placed at the mean depth of the three seeds.
- Start: each plane fitted (least squares, through `pᵢ`) to its own corner targets. Never start from
  tangent planes: on a cylinder they are all parallel to the axis and three of them never meet.
- Minimise: corner-to-target distance + regularity (edge lengths toward `target_size`-consistent
  values, smooth tilts) with barrier terms for `min_edge`, `max_edge`, `min_corner_angle_deg`,
  `max_corner_angle_deg`, `bevel_cap_deg`, `|det| ≥ 1e-8` and piece size (usable square / minimum
  bounding square). Damped Gauss–Newton / Levenberg–Marquardt, fixed iteration cap, fixed thresholds,
  deterministic term order.
- Barrier schedule (as in PH-CPF Appendix C): start the barrier weights small (e.g. 1e-3) and double
  them every 5 iterations, keeping a weak pull toward the corner targets throughout. The problem is
  non-convex, so the starting planes matter as much as the solver.
- `OptReport { iterations, corner_distance: {min, max, median}, reflex_corners, violated: Vec<(tile, limit)> }`.

### Acceptance
- [ ] Cylinder 18 × 60, `[6]`, randomness 0, depth 2: all limits hold (edges, convex and reflex
      angles, bevels ≤ cap, piece size), every corner `|det| ≥ 1e-8`. Report corner distances and
      reflex count. (Prototype reference to beat: angles ≥ 80°, bevels 14–31°, edges 2.2–6.8 in.)
- [ ] Same with randomness 0.5 and 1.0, seeds 1..10: all limits hold.
- [ ] Depth 1.0 and 3.0 on the cylinder: all limits hold, or a clear report of which limit fails.
- [ ] Cone 9 → 6 and vase bulge, `[5,6,7]`, depth 2: all limits hold.
- [ ] Every seed lies on its own plane (|distance| ≤ 1e-9) and within the depth space.
- [ ] Deterministic: identical planes on repeat runs.
- [ ] Cylinder runtime ≤ 60 s, release build, one core (report it).

### If it does not converge
Record the best `OptReport` and what was tried in `## Agent notes`, then stop. Jerry decides.

## 5. New task card — S2b: shell follows A05

**Owner:** SHELL · **Depends on:** this amendment
- Config: add `depth` and `max_corner_angle_deg`, remove `surface_tolerance`; defaults as in §3;
  `ConfigError` naming the key for out-of-range values.
- CoreInput built with the new keys; schemas validated.
- Update `examples/cylinder_18x60.toml`, fixtures and tests.
- Acceptance: example loads with depth 2.0 and thickness 0.375; `depth = 0`, `max_corner_angle_deg =
  170` and an unknown `surface_tolerance` key each raise `ConfigError` naming the key.
