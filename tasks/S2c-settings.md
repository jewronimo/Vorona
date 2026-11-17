# S2c — A06 settings

Owner: SHELL, with CORE contract portion delegated to vorona-core.
Depends on: A06 including owner-approved section 8.

## Acceptance

- [x] Pattern-specific defaults match in shell, Rust and both schema mirrors.
- [x] Smooth depth zero accepted only for voronoi_triangles.
- [x] Invalid settings name the key; finite numeric bounds enforced.
- [x] Relief base auto or numeric transported unchanged, resolved only in Rust.
- [x] Legacy regressions explicitly select voronoi and retain historical values.
- [x] A06 example and fixture agree; legacy cylinder fixture remains separate.
- [x] Unsupported new pattern fails explicitly pending C3/C4.
- [x] Official container make dev-check saved under runs/logs.
- [x] Claude review before merge.

## Agent notes

Settings-only migration on task/S2c-settings. No geometry in Python.
Full A06 section 5 geometry acceptance follows C3–C5, not claimed here.
C6 remains on hold. Symmetry WIP e506f46 remains unmerged.

Claude approved the settings implementation; review: runs/logs/S2c-claude-review.log.
Official container checks passed (exit 0): runs/logs/S2c-1755.log; Python 69 passed.
Rust/schema now reject min_corner_angle_deg outside (0, 180) for both patterns,
matching the existing shell restriction. The pending triangles dispatch can mask
other invalid legacy fields until C3/C4 routing lands; it still fails explicitly.
The single-item legacy-test loop is nonblocking cleanup, deferred.
