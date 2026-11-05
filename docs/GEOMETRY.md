# GEOMETRY — math reference for the CORE agent

Everything here is in inches and radians. Sections map to task cards C1–C7.
Where this file says "suggested", CORE may choose a better method but must document it
in the card's `## Agent notes` and keep the determinism rules.

## 1. Surface of revolution (C1)

Profile: polyline in XY, resampled to arc length `s ∈ [0, L]`, oriented bottom → top
(if `y(end) < y(start)`, reverse it). Radius `r(s) = x(s) > 0`.

Surface point: `S(θ, s) = (r(s)·cos θ, y(s), r(s)·sin θ)`.

Outward unit normal: `N(θ, s) = (y'(s)·cos θ, −r'(s), y'(s)·sin θ)`
(primes = d/ds, so `r'² + y'² = 1`). Check sign: for a cylinder `N = (cos θ, 0, sin θ)`.

Gaussian curvature: `K(s) = −r''(s) / r(s)`.
- `K = 0` cylinder and cone (developable), `K > 0` bulge, `K < 0` waist.

Total curvature of the band between `s1` and `s2`:
`∫∫ K dA = −2π · (r'(s2) − r'(s1))`.
A kink in the profile (r' jumps) is concentrated curvature; the same formula applies across it.

Conformal coordinates (suggested for topology): `u = θ` (period 2π), `v(s) = ∫₀ˢ ds'/r(s')`.
Metric: `ds² + r² dθ² = r²·(dv² + du²)`, so angles are preserved and a length `ℓ` on the surface
is `ℓ / r(s)` in (u, v).

Distance from a point `p` to the surface: convert `p` to (radial distance `ρ = √(x²+z²)`, height `y`),
then find the closest point on the 2D profile polyline to `(ρ, y)`. Signed: negative inside.

## 2. Topology from seeds (C3)

- One seed per tile, placed in (u, v). Tile `i` ↔ seed `i`.
- Delaunay triangulation of the seeds in (u, v), periodic in `u` (copy seeds to `u ± 2π`,
  triangulate, keep triangles whose centroid has `u ∈ [0, 2π)`; dedupe by sorted index triple).
- Tile neighbours = Delaunay edges. Tile corners = Delaunay triangles. So every corner is 3-valent.
- **Side count of an interior tile = degree of its seed in the triangulation.**
- Changing side counts = topology edits: edge flip (moves one degree between two pairs of seeds),
  seed insert, seed delete, small seed moves. All in a fixed deterministic order.

Seeding (suggested): Poisson-disk or perturbed hex lattice in (u, v) with local spacing
`target_size / r(s)`, perturbation scaled by `randomness`. At `randomness = 0` and `[6]` on a
cylinder this gives a regular staggered hex lattice.

Rim rows: with `level` rims, seeds continue past the rim by one row so the rim plane cuts tiles
(partials); place the last row so every partial keeps bounding square ≥ minimum.
With `whole` rims, drop every tile whose polygon crosses the rim plane.

## 3. Feasibility (C2)

Inputs: `r(s)`, allowed set `A ⊆ {4..9}`, `target_size w`, variation `δ` (0.15 or 0.40),
`bevel_cap`, `min_bounding_square b`, `allow_waists`.

**Defects from curvature.** A tile of `k` sides carries defect `6 − k` (pentagon +1, heptagon −1,
square +2, octagon −2, nonagon −3). Over a region, `Σ(6 − k) ≈ ∫∫K dA / (π/3)`.
- Pentagons needed: `P = round(Σ over r'' < 0 segments of −6·Δr')`.
- Heptagons needed: `H = round(Σ over r'' > 0 segments of +6·Δr')`.
- Euler check for an open annulus: `Σ_interior(6 − k) + Σ_boundary(4 − deg) = 0`; the rim rows
  absorb the remainder, so only the local P and H matter.

**Column changes (dislocations).** Columns around at height `s` must satisfy
`2π r(s) / (w(1+δ)) ≤ n ≤ 2π r(s) / (w(1−δ))`. Find the integer sequence `n(s)` with fewest
changes (a regular staggered hex wrap needs `n` even, or any `n` with a helical shift — CORE
decides and documents). Each change of `n` by 1 needs one 5 + 7 pair.

**Waists.** Any segment with `r'' > 0` (beyond noise, threshold `1e-6`) and `allow_waists = false`
→ infeasible, message names the height range.

**Small radius.** A ring of `n` tiles bends `2π/n` per joint, bevel `π/n`, so `n ≥ ⌈180°/bevel_cap⌉`.
Tiles need `2π r / n ≥ b`. So `r ≥ b · ⌈180°/bevel_cap⌉ / (2π)`. Report heights where `r` is below.

**Decision.** Feasible if:
- `P > 0` → `A` contains a count < 6; `H > 0` or dislocations > 0 → `A` contains a count > 6
  (dislocations need both sides);
- average interior count `6 − D/F` (D = P − H, F = expected interior tile count) lies within
  `[min A, max A]`, and if `6 ∉ A` the set has counts on both sides of the average;
- no waist violation, no small-radius violation.

**Suggestions** (when infeasible): candidate sets = `A` plus the fewest added counts that make
it feasible, preferring counts nearest 6; then the same with `size_variation = wide`.
Return at most 4, sorted by (number of added counts, distance of added counts from 6, set as list).
Message format: `"6 only is not possible: this profile bulges, so it needs about 9 five-sided pieces. Try [5,6] or [5,6,7]."`

## 4. Flat tiles (C4)

Each tile `i` has a plane `πᵢ: nᵢ·x = dᵢ`, `|nᵢ| = 1`, `nᵢ` outward.

Corner of tiles `i, j, k` (a Delaunay triangle) = intersection of the three planes
(solve the 3×3 system; reject if `|det| < 1e-8`, which means near-parallel planes).

Unknowns: 3 per tile. Constraints: 1 per corner (on surface). A hex pattern has twice as many
corners as tiles, so the system is under-determined; regularity terms pick the solution.

Energy (suggested), minimized with Levenberg–Marquardt or Gauss–Newton, fixed iteration cap:
- `w_surf · Σ_corners dist(v, S)²` — corners on the surface (R-TL-3), dominant weight
- `w_size · Σ_edges (len − target_edge)²` — even sizes within the variation band
- `w_fair · Σ_neighbours |nᵢ − nⱼ|²` — smooth faceting
- barrier terms for `min_edge`, `min_corner_angle`, convexity, bevel cap

Initialization: `nᵢ = N(seedᵢ)`, `dᵢ = nᵢ·S(seedᵢ) − 0.5·expected_depth`.

**Important known result.** On `K = 0` regions (cylinder, cone) a hex pattern that is
mirror-symmetric about each tile's own radial plane cannot have all six corners on the surface
and stay planar. Solutions exist but tiles tilt slightly, alternating up/down (a subtle zigzag).
Do not force symmetric hexagons; let the optimizer tilt them. On `K < 0` planar hexagons
become non-convex — that is why waists are rejected in v1.

Face depth of tile `i` (R-VA-7): max distance from the surface to points of the tile polygon,
measured at the polygon centroid and edge midpoints.

## 5. Rims (C5)

`level` rim at height `y_top`: clip tile polygons by the half-space `y ≤ y_top` (bottom: `y ≥ y_bottom`).
The cut edge's side face is the horizontal plane. Tiles fully outside are removed.
`whole` rim: remove tiles crossing the rim plane; their former neighbours get boundary edges
whose side face contains `nᵢ` (square cut).

## 6. Stone pieces and bevels (C5)

Outer face of piece `i`: its polygon in `πᵢ`, corners `v` in a fixed CCW order seen from outside.

Inner face: plane `nᵢ·x = dᵢ − t`. Inner corner for corner `(i, j, k)` = intersection of the three
inner planes `= v − t·w`, where `w` solves `nᵢ·w = nⱼ·w = nₖ·w = 1`. All three pieces get the same
inner corner, so joints close exactly.

Joint face between `i` and `j`: bisector plane `{x : nᵢ·x − dᵢ = nⱼ·x − dⱼ}`, normal
`mᵢⱼ = (nᵢ − nⱼ)/|nᵢ − nⱼ|`. It contains the outer edge and the inner edge.

Bevel (tilt from a square cut, what the waterjet head sees) for any side face with unit normal `m`:
`β = asin(|m · nᵢ|)`. For a joint this equals half the bend: `β = ∠(nᵢ, nⱼ) / 2`.
Level rim: `m = (0, 1, 0)`. Whole rim: `β = 0`.

Each piece is then a closed polyhedron: outer face, inner face, one planar quad per edge
(outer edge `k` → inner edge `k`). Check the inner polygon is simple and has the same
orientation as the outer one; if not, the tile is too small for the thickness → validation failure.

## 7. Numbering, flat transforms, nesting (C6, C7)

**Numbering.** Sort pieces by row band (centroid height, bands of `0.5·target_size`, bottom first),
then by angle `θ ∈ [0, 2π)` of the centroid (counter-clockwise seen from +Y), ties by seed index.
Ids `P001`, `P002`, … (width grows to 4 digits past 999).

**Flat transform.** Rigid 4×4 that maps the outer face into the XY plane, outer face up (+Z),
outer-face centroid at the origin, and the longest edge along +X.

**Footprint.** Union of the outer and inner faces projected onto the face plane
(use the convex hull for nesting).

**Nesting (suggested).** Pieces sorted by footprint area (desc, tie by id). Each blank is a
`usable × usable` square centred on a `size × size` blank. For each piece, try rotations
0°, 15°, …, 345°, and bottom-left positions on a 0.05 in grid, keeping `nest_gap` clearance;
take the first fit in that order; else open a new blank. Deterministic by construction.
