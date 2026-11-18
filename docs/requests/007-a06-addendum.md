# A06 addendum 1 — clarifications (drafted by Claude 2026-11-16; owner approval required)

Append this to `amendments/A06-voronoi-triangles.md` as section 8 on `docs/A06`, then re-review.
Resolves the BLOCK in `runs/logs/A06-claude-review.log` and adopts `docs/requests/006` with one change
(item 5). Where this addendum and A06 sections 1–7 disagree, this addendum wins.

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
- Symmetric mode (`randomness = 0`): every polygon in a row identical; rows use the A05 three-row
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
