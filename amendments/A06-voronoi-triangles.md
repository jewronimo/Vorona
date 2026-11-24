# Amendment A06 — Polygons + triangles (rectified Voronoi)

**Date:** 2026-11-15 · **Approved by:** Jerry · **Amends:** A05 (pattern and C4 method; depth-space
idea kept), docs/SPEC.md (R-TL-2, R-TL-11, R-ST-*, R-VA-*), docs/GEOMETRY.md §2/§4/§6, INTERFACE.md and
both JSON schemas, example config, tasks C3, C4, C5, C6 · **Adds:** task S2c. Read after A05. Where
this amendment and an earlier file disagree, this amendment wins.

## 1. Why

With A05 every corner is shared by exactly three flat stones. On surfaces that are straight in one
direction (cylinders, cones) three flat stones cannot all meet on the surface, so the wall must bulge
and pinch. Spread evenly, that shows as rings; the owner rejected the result ("the cylinder does not
look like one"). Smaller stones and a corner limit reduced the rings but did not remove them;
spiral and checkerboard depth patterns failed (Claude prototypes, 2026-11-14).

Owner decision: introduce **triangles**. Every corner is then shared by **four** pieces (two polygons
and two triangles). Four-piece corners can lie on any surface, and a triangle is always flat, so the
shape follows the surface and relief becomes a choice, not a side effect.

## 2. The model (owner decisions 2026-11-15)

1. **Seeds** (points) are laid out as before (C3): rows on the surface, random or symmetric, each
   seed with a relief depth `dᵢ` (see 6). Delaunay topology in (u, v), periodic, integer seam rule.
2. **Pattern = rectified Voronoi.**
   - One **polygon** per seed (the Voronoi cell, shrunk).
   - One **triangle** per Delaunay triangle (each former three-stone corner becomes a triangle).
   - One **corner** per Delaunay edge (i, j): it is shared by polygon i, polygon j and the two
     triangles on either side of that edge. Every corner has exactly 4 pieces.
3. **Polygon plane through its seed** (A05 rule kept for polygons): plane πᵢ passes through the seed
   at depth `dᵢ`; 2 unknowns per polygon (tilts about the two surface tangents).
4. **Corners on lines.** Corner (i, j) lies on the line πᵢ ∩ πⱼ, so both polygons are exactly flat by
   construction. 1 unknown per corner: its position along that line. Target position: the midpoint of
   the Voronoi edge between seeds i and j, on the surface at depth (dᵢ + dⱼ)/2. Triangles are flat by
   definition. No three-plane intersections are needed anywhere.
5. **Polygon side counts vary.** Rows get a seed count from the local circumference, so narrowing or
   widening surfaces (cone, vase) get 4-, 5-, 7- and 8-sided polygons where the count changes. This
   replaces `allowed_sides = [6]` and the C2 column-change machinery for this pattern.
6. **Relief.** `dᵢ = relief_base + randomness-driven depth in [0, depth]`.
   - Random (default): uniform in [0, depth], deterministic from `seed`; positions jittered ±10 % of
     spacing.
   - Symmetric (`randomness = 0`): every polygon in a row identical; rows may alternate depth.
     Rotational symmetry gate applies (see 9).
   - `depth = 0` gives a smooth surface.
   - `relief_base`: planes sit slightly inside the surface so corners straddle it. Choose it so the
     median signed corner distance is ≈ 0 (prototype: 0.6 in at 9.2 in seed spacing).
7. **Stones** = each piece offset inward by `thickness` (0.375 default) with bisector joints, as A05.
8. **Ready for uploaded surfaces (L1):** after seeding, only generic surface queries are used.

## 3. Limits (owner)

| Setting | Default | Meaning |
|---|---|---|
| `min_triangle_edge` (new) | 4.0 in | every triangle side |
| `layout_headroom` | **0.10** | nominal layout reserve fraction (2026-11-20 decision below) |
| `edge_shortfall` | **0.0** | finished whole-side shortfall fraction, [0, 0.5) |
| `min_edge` | not used by `voronoi_triangles` | `min_triangle_edge` applies to every side |
| `min_corner_angle_deg` | **30** (was 70) | plan-view corner angle, all pieces |
| `max_corner_angle_deg` | 230 | reflex polygon corners |
| `bevel_cap_deg` | 60 | taper of the grout face (waterjet limit) |
| `blank_usable` | 11.5 in | every piece fits this square (12 × 12 blank) |
| `depth` | **1.0** (was 1.5) | relief depth |
| `min_bounding_square` | 4.0 in | **rim partials only** (sliver guard), until the owner decides otherwise |

Note for the owner: in this pattern polygon sides equal triangle sides, so `min_triangle_edge` 4 in
sets polygon size: seed spacing ≈ 2.3 × `min_triangle_edge` (9.2 in), polygons ≈ 8–9 in across.
`target_size` becomes derived (report it); lowering `min_triangle_edge` is the size knob.

Corner distance to the surface is limited, not only reported: every corner within
`[-(relief_base + depth + 0.4), +0.75]` in (barrier). Report min/max/median.

## 4. Task changes

### S2c (SHELL) — settings
- Add `pattern` (`"voronoi_triangles"` default, `"voronoi"` = A05 behaviour), `min_triangle_edge`,
  `relief_base` (`"auto"` or inches). Change defaults: `min_corner_angle_deg` 30, `depth` 1.0.
  `min_bounding_square` documented as rim-partials only. ConfigError naming the key for bad values.
- Schemas, INTERFACE, example config, fixtures updated together.

### C3 — rectified topology
- Seeds per row from local circumference (5 above); jitter and relief depths as 6.
- Build polygons, triangles and corners (2). Every in-band corner has 4 pieces; open edges only at the
  outermost padding rows (A05 seam rule and hard error on open interior edges kept).
- Tests: piece/corner counts, every corner 4-valent in band, deterministic, seam-independent.

### C4 — solver for this pattern
- Unknowns: 2 tilts per polygon + 1 slide per corner (2–4 above). Residuals: corners to targets,
  small regularisation, barriers for every limit in 3 plus the inner-polygon-at-thickness guard
  (C5 lesson, keep). Sparse Jacobian (prototype runs 20–60 s in Python).
- **Multi-start required:** the prototype gets stuck on some starts (cone, relief 1.0: seed 1 fails,
  seed 2 passes). On failure retry with deterministic perturbed starts (e.g. 5 starts), keep the best;
  report every attempt. Never relax a limit to pass.
- Symmetric mode: solve one row representative and copy (rotation-exact), reuse the symmetry-repair
  findings.
- Keep the A05 solver available under `pattern = "voronoi"`.

### C5 — pieces
- Adapter accepts polygons and triangles. Joints = bisector planes as now.
- **Nearly coplanar neighbours** (half-angle < 1°): use a square joint (perpendicular to the face),
  bevel 0. The bisector is unstable there (prototype shows false huge back-face errors).
- **Back-face gap at 4-piece corners:** inner corners of the four pieces do not meet in one point.
  Measure the mismatch between the two inner edges of every joint; report min/median/max per piece
  and overall. Prototype: median 0.13–0.30 in. Not a failure unless > `max_back_gap` (new setting,
  default 0.5 in, owner may change); report every case above it.
- Rim level cut, sliver handling (`min_bounding_square` for partials), closure gates: unchanged.

### C6 — validation
- Rules of section 3 per piece type; back-gap report; corner-distance report; closure.

## 5. Acceptance (C3 + C4 + C5 together)

Profiles: cylinder 18 × 60, cone 9 → 6, vase bulge. Settings as section 3.
- [ ] Smooth (`depth = 0`): all three pass every limit and closure.
- [ ] Random relief `depth = 1.0`, seeds 1..10: all three pass (multi-start allowed, reported).
- [ ] Symmetric (`randomness = 0`, `depth = 1.0`): all three pass, rotational gate (equal spacing and
      size per gated row within 1e-6; owner count-transition exception §8.9).
- [ ] `depth = 1.5`: report only.
- [ ] Every polygon flat (|corner-to-plane| ≤ 1e-6 in), every corner 4-valent in band, determinism.
- [ ] Back-gap and corner-distance reports present; runtime per case reported (budget 90 s).
- [ ] Owner STEP export of cylinder, cone, vase (smooth and random seed 1) for review.

**If the cone or vase does not pass** after multi-start: stop, report the best result per case and
what was tried. The owner adjusts settings (e.g. `min_triangle_edge`, `depth`) from the evidence.

## 6. Prototype reference (Claude, 2026-11-15, not production code)

`docs/prototypes/A06_tri_proto.py`. Env: PROFILE cyl|cone|vase, A (seed spacing 9.2), D (relief
depth), JIT (0.1), SEED, NFEV (optimizer evaluations; 0 = construction only), BASE (0.6), OUT.

| Case (A = 9.2 in) | Result |
|---|---|
| Cylinder smooth / relief 1.0 / relief 1.5 | PASS / PASS / PASS |
| Vase smooth / relief 1.0 / relief 1.5 | PASS / PASS / 3 short triangle edges |
| Cone smooth / relief 1.0 | PASS / seed 1: 28 short triangle edges, **seed 2: PASS** |

Cylinder relief 1.0: triangle sides ≥ 4.1 in, sharpest corner 33°, max taper 53°, corners −1.7 to
+0.75 in from the surface, back-face gap median 0.17 in.

## 7. Process

- `task/symmetry-repair` (A05 solver): commit as WIP on its branch, do not merge; its findings feed the
  A06 symmetric mode.
- C6 waits for A06 C3–C5.
- Branch per card as before; Claude reviews before every merge.

## 8. Owner-approved addendum — clarifications (2026-11-16)

Approved by Jerry; supersedes request 006. Where this section and sections 1–7
disagree, this section wins.

## 8.1 Scope of the new defaults
- Everything in A06 §2.6 (relief) and §3 (new defaults) applies **only to `pattern = "voronoi_triangles"`**.
- `pattern = "voronoi"` keeps A05 exactly: `depth` 1.5, `min_corner_angle_deg` 70,
  `max_corner_angle_deg` 230, position-only randomness, repeating three-row depth pattern 0, D/2, D,
  its own seed-band validation and all A05 gates.
- Omitted `pattern` selects `voronoi_triangles`. Omission defaults for `depth` and
  `min_corner_angle_deg` follow the selected pattern **in Rust**, not only in shell defaults or schema
  annotations. Explicit values always win.

## 8.2 Depth and relief for `voronoi_triangles` (replaces A05 R-TL-3 / R-VA-7 for this pattern)
- `depth`: finite, **≥ 0** (0 = smooth). `relief_base`: `"auto"` or finite ≥ 0 inches.
- **R-TL-3 (triangles):** every seed lies at depth `dᵢ ∈ [bᵢ, bᵢ + depth]` along the inward normal,
  where `bᵢ` is the resolved relief base for that seed.
- **R-VA-7 (triangles):** validate exactly that interval (tolerance `GEOM_EPS`); corner distances to the
  outer surface are limited by the A06 §3 corner band and reported.
- This **supersedes the A05 owner decision of 2026-11-12** ("depth never varies per point") **for
  `voronoi_triangles` only**. Per-seed random depth is allowed here because no three-plane corners
  exist in this pattern (corners lie on two-plane lines).
- Symmetric mode (`randomness = 0`): every polygon in a gated row identical (owner exception §8.9); rows use the A05 three-row
  depth pattern scaled to [b, b + depth] (no two-row alternation).
- Schema bounds: `depth` ≥ 0 for `voronoi_triangles`; `voronoi` keeps `depth` > 0.

## 8.3 Seed spacing (replaces target_size for this pattern)
- `seed_spacing = 2.3 × min_triangle_edge` (9.2 in at the default 4.0 in). Fixed factor in v1.
- `target_size` and `allowed_sides` stay in the schema for compatibility but do not constrain
  `voronoi_triangles`. Report the derived `seed_spacing` and the actual piece sizes separately.

## 8.4 C2 under `voronoi_triangles`
- Seeds per row = round(local circumference / seed_spacing), at least 3.
- A04 side-set suggestions and the A05 depth warning do **not** apply. Feasibility is decided by the
  A06 solver gates. C2 still reports per-row counts and flags any row whose predicted polygon width
  exceeds `blank_usable`.

## 8.5 `relief_base = "auto"` — closed-form, no search (changes request 006 item 5)
Each seed gets its own base from the local surface curvature, so polygon planes straddle the
surface:

    bᵢ = (seed_spacing / 2)² · κmax,ᵢ / 4

where κmax,ᵢ = the larger principal curvature at the seed, clamped at ≥ 0 (0 on flat or
inward-curving directions). Cylinder R = 9 in, spacing 9.2 in: b = 4.6² / (4 · 9) = 0.59 in (the
prototype's 0.6). Deterministic, no extra solves. Report min/median/max of bᵢ. A numeric
`relief_base` replaces it with one constant.

## 8.6 Settings and schema migration (adopts request 006 items 6–9)
- Keep the `/1` schema identifiers. Rust, both schemas, INTERFACE, shell, fixtures and tests change
  together.
- `min_triangle_edge` finite > 0; `max_back_gap` finite ≥ 0 (default 0.5 in); `relief_base` numeric
  finite ≥ 0 or the literal `"auto"`, transported unchanged by SHELL and resolved only in Rust. No new
  geometry in Python.
- Historical regressions select `pattern = "voronoi"` explicitly and keep their explicit values (ring12:
  depth 2, max angle 225). Split the legacy cylinder input from the new A06 example. Migrated
  byte-identical fixtures recompute provenance from the exact migrated bytes; hand-checkable geometry
  unchanged.
- S2c is settings-only. Until C3/C4 support it, `voronoi_triangles` returns an explicit
  unsupported/pending error. It must never fall back to the A05 generator silently.

## 8.7 Prototype reproducibility (corrects §6)
The §6 table used `A=9.2`, `NFEV=150`, `BASE=0.6`; relief runs `JIT=0.1 SEED=1`; smooth `D=0 JIT=0`.
The cone relief PASS used `SEED=2` (and 3) with `NFEV=400`. Script defaults (A=8.5, JIT=0) are not the
documented settings: always pass them explicitly. The prototype verdict did not check
`max_corner_angle_deg` or the corner band (reported only). Its `verts()` adds 1e-6 to a determinant;
production must solve the two-plane line exactly to meet the ≤ 1e-6 in flatness gate.

## 8.8 Process
- Full §5 acceptance replaces the A05 gates for `voronoi_triangles`. A05 regressions remain required for
  `voronoi`. Update AGENTS.md §5.6 when A06 C4 lands (new matrix, waivers).
- C6 stays on hold until A06 C3–C5 merge.

## 8.9 Owner-approved symmetric count-transition exception (2026-11-17)

Jerry's decision supersedes the blanket per-row identity requirement in §8.2
and the symmetric rotational gate in §5, for `voronoi_triangles` only:

- A row whose seed count equals **both** immediate neighbouring row counts is
  solved using one row representative and copied by its rigid periodic rotation.
  The geometric rotational gate remains **1e-6 inch**, including equal spacing
  and piece size. No independent full-row solve followed by averaging.
- A row at a seed-count transition is solved freely, deterministically from
  `seed`. It carries **no symmetry claim**. The gate/report identifies every
  such row individually with its count and both neighbouring counts.
- Outermost padding rows have no pair of neighbours and carry no per-row
  symmetry claim; physical rows retain two support rows from C3.
- This exception changes no §3 limit, exact two-plane corner construction,
  per-seed depth/base rule, deterministic multi-start requirement, runtime
  budget, A05 regression, or C5/closure/owner review gate.

## 9. Owner-approved addendum — small symmetric stones, constant rows (2026-11-17)

Approved by Jerry in chat; source: `docs/requests/008-a06-addendum-2.md`.
Applies only to `pattern = "voronoi_triangles"`. Where this section and sections 1–8
disagree, this section wins. Resolves the C4 unequal-count row transition blocker.

## 9.1 Why
Owner review of prototypes (2026-11-17): small stones with constant rows and symmetric relief look
right on cylinder, cone and vase; large stones looked coarse, and random relief on small stones failed.

## 9.2 Stone size (replaces §3 `min_triangle_edge` default and §8.3 seed spacing)
| Setting | Default | Meaning |
|---|---|---|
| `max_polygon_edge` | none | optional hard cap on polygon and triangle sides; always report actual largest side |
| `min_triangle_edge` | **2.0 in** | minimum stone side: every polygon and triangle side |
| `layout_headroom` | **0.10** | nominal layout reserve fraction (2026-11-20 decision below) |
| `edge_shortfall` | **0.0** | finished whole-side shortfall fraction, [0, 0.5) |
| `min_edge` | not used by `voronoi_triangles` | `min_triangle_edge` applies to every side |
| `min_corner_angle_deg` | 30 | unchanged |
| `blank_usable` | 11.5 in | unchanged |
Triangles are isosceles in the symmetric layout: symmetric about their own vertical centreline, base
and legs may differ. Polygons are likewise symmetric about their vertical centreline.

### Owner rim-edge decision — 2026-11-17 (verbatim)

> edges created by level rim saw cut (kind rim_level, and square rim_whole edges) exempt from max_polygon_edge. Rim-cut pieces STILL limited by blank_usable, min_triangle_edge on remaining triangle sides, min_corner_angle_deg, min_bounding_square for rim partials.

### Owner rim-partial decision — 2026-11-17

For `voronoi_triangles` only, rim partial minimum enclosing square is **1.5 in**,
replacing the inherited 4.0 in minimum. Edges shortened by the cut are exempt
from `min_edge` and `min_triangle_edge`. Partials are judged only by the 1.5 in
square, `blank_usable`, and `min_corner_angle_deg`.

The level saw cuts are always at the design ends, exactly 0 and H. To avoid
small partials, shift the pattern row phase along the profile using a deterministic
search within one row height; never move the cut heights. Rejected neighbors
expose square `rim_whole` edges without dangling joint references. A05 remains
unchanged.

## 9.3 Row layout (replaces §2.5 per-row counts and §8.4 C2 counts)
**Owner decision 2026-11-19 (supersedes the earlier §9.2/§9.3 sizing policy):**
1. One global integer count `n >= 3` and physical arc row height `h`. No count changes,
   no fallback layout, and no layout setting. Stones grow where the surface widens.
2. Choose the largest `n` with `pi*r(s)/n >= min_triangle_edge` everywhere, including
   profile extrema between rows. Thus `n = floor(pi*r_min/min_triangle_edge)`.
3. Start `h = sqrt(3)/2 * 2*pi*r_min/n`. Raise it only enough for wider rows to meet
   `min_corner_angle_deg` (30 degrees). Report the original and raised height.
   With `w=2*pi*r/n`, triangle base is `w/2`, leg is `sqrt(w*w/4+h*h)/2`;
   apex angle is `2*atan(w/(2*h))`, base angles `(180-apex)/2`.
4. If the selected n/h fails any gate, return infeasible, naming row and limit;
   do not change count or relax limits. Optional `max_polygon_edge` is a hard cap.
   All existing fitted-geometry gates remain, including blank_usable=11.5 inches,
   corner angle 30 degrees, taper 60 degrees and the per-stone mirror gate.
5. Default minimum 2 inches gives cylinder r9 n14, cone 9→6 n9, vase r6–10 n9,
   flare 6→15 over 30 inches n9. Report global n/h, each row's base/leg/angles,
   actual maximum side, and whether h was raised. These layout estimates do not
   certify fitted C4/C5 geometry.


### Owner layout headroom / finished-edge decision — 2026-11-20

`layout_headroom` is a finite nonnegative **fraction**, default **0.10**.
`edge_shortfall` is a finite **fraction in [0, 0.5)**, default **0.0**.
This decision supersedes the earlier unreserved count formula and n14/n9 examples.
The owner tunes these per job; they are not fixed geometry constants.
The global count is `floor(pi*r_min / ((1+layout_headroom)*min_triangle_edge))`,
at least 3, using extrema anywhere on the profile. Height selection is unchanged.
Finished whole polygon and triangle edges must be at least
`(1-edge_shortfall)*min_triangle_edge`; polygon `min_edge` barrier uses the same
whole-stone minimum. Rim partial exemptions remain unchanged. Headroom changes
only nominal sizing; shortfall changes only the finished-edge lower gate.
Report measured shortest outer edge (including exempt partials) and maximum
whole-edge deficit below the requested minimum in inches; keep those distinct.
`stats.actual_shortest_side` is null without pieces; `stats.actual_edge_shortfall`
is zero without whole edges. Diagnostics emit the same fields.
At defaults: cylinder r9 n12, cone9→6 n8, vase6–10 n8, flare6→15 n8.
Layout side ranges are nominal, not a fitted-geometry manufacturing pass.

| Profile | n | h (in) | Nominal triangle sides (in) |
|---|---:|---:|---:|
| cylinder | 12 | 4.081048570 | 2.356194490–2.356194490 |
| cone | 8 | 4.081048570 | 2.356194490–3.534291735 |
| vase | 8 | 4.081048570 | 2.356194490–3.926990817 |
| flare | 8 | 4.081048570 | 2.356194490–5.890486225 |

Flare sizing is **infeasible at default blank_usable=11.5**: nominal row
envelope exceeds the blank (11.510404663 at s=30.122032655). Its table row
is sizing-only with the reporting blank cap disabled; production gates are unchanged.


### Owner row-height option — 2026-11-20

`row_height = "constant"` is the default and preserves the global-height policy.
`"proportional"` retains the same single global count, but sets local nominal
pitch to `sqrt(3)/2 * 2*pi*r(s)/n`. Rows advance deterministically by the local
pitch, starting with two auxiliary narrowest-pitch rows below the design end.
No count changes, fallback, or fitted-limit relaxations are introduced.
Report row count and physical pitch range; nominal aspect ratios do not certify
fitted stones. Phase shifts remain physical arc shifts, with local pitch recomputed.

## 9.4 Relief
- Default `randomness = 0` (symmetric): row depth pattern 0, D/2, D (§8.2) on top of the auto base;
  every stone in a row identical.
- Default `depth` = **0.25 in**. Explicit
  `depth` overrides.
- **Random relief (`randomness > 0`) is experimental:** allowed, reported, never a merge gate until
  reworked. Prototype: 0.25 in random relief on 3 in stones failed (torn pieces).

## 9.5 Symmetric solve (replaces the "solve one row and copy" rule where counts change)
- Owner decision (2026-11-19): randomness 0 is dihedral: zero sideways tilt,
  mirror-paired corner slides, and a maximum mirror error of 1e-6 inch for
  every whole stone. Include the mirror gate in reports and acceptance.
- One global count: solve one representative stone per row (dihedral-constrained tilt) plus its corner slides,
  copy by exact rotation `2π/n`. Gate: equal spacing and equal size per row within 1e-6.
- No count-change or sector-copy path; it remains only in git history.

## 9.6 Acceptance (replaces §5 matrix for this pattern)
Required, all limits + closure + flatness + determinism + rotational and mirror gates:
- Cylinder 18×60, cone 9→6, vase bulge: smooth (`depth = 0`) and symmetric relief at **depth = 0.25 in** (owner exact prototype parity request;
  matching the configured default of 0.25 in in §9.4).
- Report only: symmetric relief at 0.5 in; random relief at 0.25 in, seeds 1..3.
  These use twice the configured 0.25 in default for symmetric relief and the default itself for random relief.
- STEP export of all six required cases for owner review.

## 9.7 Prototype reproduction
`docs/prototypes/A06_tri_proto.py` is superseded for this section by the variant used in chat
(`COUNT=constrow:N:H`, `TMIN`, `MAXE`, `RELIEF=rows`, `BASE=auto`); results: cone n=10 h=4.5 smooth
PASS (sides 1.95–2.70 in, taper ≤ 11°), relief 0.25 PASS; vase n=11 h=4.5 smooth PASS (sides 1.69–2.92 in,
corners −0.06..+0.21 in), relief 0.25 PASS (taper ≤ 13°, back gap median 0.07 in).
