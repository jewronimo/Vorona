# C3 — Seeds, topology, side-count control

**Owner:** CORE · **Depends on:** C2 · **Unblocks:** C4
**Read first:** docs/GEOMETRY.md §2, SPEC R-TL-1,2,4,5,6,7, R-RM-1,2

## Goal
Place seeds deterministically and build a tile adjacency (Delaunay in conformal coordinates)
where every interior tile's side count is in the allowed set.

## Deliverables
- `core/src/seeds.rs` — seeding from `seed`, `randomness`, `target_size`, `size_variation`,
  including the extra rim row for `level` rims.
- `core/src/topology.rs` — periodic Delaunay (crate `spade`), `Topology { tiles, neighbors, corners }`
  with corners as sorted index triples; degree per tile; boundary flags.
- `core/src/sidecount.rs` — deterministic repair loop: edge flips, seed insert/delete, small moves,
  until all interior degrees ∈ allowed set or a fixed cap is hit (then return an error naming tiles).

## Acceptance
- [ ] Cylinder, `[6]`, randomness 0: every interior tile degree 6, column count = the one C2 chose,
      lattice regular (seed spacing in u within 1e-9 of 2π/n).
- [ ] Cylinder, `[5,6,7]`, randomness 1, seeds 1..20: all interior degrees ∈ {5,6,7}; 3-valent corners;
      each interior Delaunay edge in exactly two triangles.
- [ ] Cone 9 → 6, `[5,6,7]`: number of 5s and 7s ≥ C2's prediction, placed where columns change.
- [ ] Vase bulge, `[5,6,7]`: pentagons count within ±1 of C2's `pentagons_needed`.
- [ ] Same seed twice → identical topology; different seed → different topology.
- [ ] No tile's seed spacing outside the size-variation band (measured on the surface, not in u,v).

## Agent notes
