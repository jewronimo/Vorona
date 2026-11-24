# INTERFACE — contract between the Rust core and the Python shell

The core is called with one JSON document and returns one JSON document.
Same function behind three entry points:

| Entry point | Signature | Used by |
| --- | --- | --- |
| Rust library | `vstone_core::generate(input: &CoreInput) -> CoreOutput` | Rust tests |
| Rust binary | `vstone-core <input.json>` → output JSON on stdout | CORE testing without Python |
| Python module | `vstone_core.generate_json(input_json: str) -> str` (PyO3, built by maturin) | SHELL |

JSON in/out keeps the binding trivial and lets either side be tested alone.
Schemas are versioned in the `schema` field; a breaking change bumps the number and
needs Jerry's approval (AGENTS.md §2). JSON Schema files: `docs/schema/core_input.schema.json`
and `docs/schema/core_output.schema.json` (written in task C0, kept in sync by CORE).

All lengths in inches, angles in degrees, points as `[x, y, z]`, Y up.

## CoreInput — `vstone.core_input/1`

```json
{
  "schema": "vstone.core_input/1",
  "surface": {
    "kind": "revolve",
    "profile": [[9.0, 0.0], [9.0, 0.5], "... [x, y] pairs, bottom to top, chord error <= 0.01"]
  },
  "params": {
    "pattern": "voronoi_triangles",
    "min_triangle_edge": 4.0,
    "relief_base": "auto",
    "max_back_gap": 0.5,
    "thickness": 0.375,
    "allowed_sides": [6],
    "target_size": 6.5,
    "size_variation": 0.15,
    "randomness": 0.0,
    "seed": 42,
    "ends":  { "top": "open", "bottom": "open" },
    "rims":  { "top": "level", "bottom": "level" },
    "min_edge": 1.0,
    "max_edge": 6.0,
    "min_corner_angle_deg": 30.0,
    "bevel_cap_deg": 60.0,
    "depth": 1.0,
    "max_corner_angle_deg": 230.0,
    "min_bounding_square": 4.0,
    "allow_waists": false,
    "blank_size": 12.0,
    "blank_usable": 11.5,
    "nest_gap": 0.25
  }
}
```

A06 settings (owner-approved §8; coordinated `/1` migration):
- `pattern` is `"voronoi_triangles"` by default; explicit `"voronoi"` selects A05.
- Rust omission defaults depend on the selected pattern: `depth` 1.0 and
  `min_corner_angle_deg` 30 for triangles; 1.5 and 70 for legacy. Explicit values win.
- `depth` is finite and nonnegative for triangles (zero is smooth); strictly
  positive for legacy. `min_corner_angle_deg` is finite, strictly between 0 and 180.
- `min_triangle_edge` is finite > 0, default 4.0 inches.
- `max_back_gap` is finite >= 0, default 0.5 inches; C5 implements its gate later.
- `relief_base` is exactly `"auto"` (default) or finite nonnegative inches. SHELL
  transports it unchanged. Rust geometry later resolves auto per A06 §8.5;
  S2c performs no curvature calculation or base search.
- `max_corner_angle_deg` remains finite strictly between 180 and 270, default 230.
- `min_bounding_square` is a rim-partial sliver guard for triangles. Legacy
  settings and gates remain unchanged. Thickness defaults to 0.375 in SHELL.
- `target_size` and `allowed_sides` remain required compatibility fields, but do
  not constrain rectified topology. Future CORE derives seed spacing as
  `2.3 * min_triangle_edge` and reports spacing separately from actual piece sizes.
- The A05 depth warning and A04 side-set suggestions apply only to legacy.
  A06 seed-band validation is `[resolved base, resolved base + depth]`; legacy
  keeps `[0, depth]`. Geometry/report implementation follows in C3–C5/C6.
- Until A06 C3/C4 land, both generation and feasibility-only entry points return
  status `error`, an explicit unsupported/pending message, and no pieces for
  triangles. There is no silent legacy fallback or invented feasibility result.

Historical cylinder/ring inputs explicitly select legacy and preserve their
explicit settings. `core_input_a06_cylinder.json` is the separate A06 example
input. Ring12 provenance hashes the exact migrated ring-input bytes; its geometry
is unchanged. The obsolete surface-tolerance setting remains rejected.

Later (L1) `surface.kind = "mesh"` with `vertices` and `triangles`; not in v1.

## CoreOutput — `vstone.core_output/1`

```json
{
  "schema": "vstone.core_output/1",
  "status": "ok | infeasible | failed_validation | error",
  "message": "plain-language summary, one or two sentences",
  "feasibility": {
    "feasible": true,
    "pentagons_needed": 0,
    "heptagons_needed": 0,
    "column_changes": 0,
    "columns": [12],
    "small_radius_ranges": [],
    "waist_ranges": [],
    "suggestions": [[5, 6], [5, 6, 7]]
  },
  "pieces": [
    {
      "id": "P001",
      "sides": 6,
      "partial": false,
      "plane": { "n": [1.0, 0.0, 0.0], "d": 8.7 },
      "outer": [[0, 0, 0], "... corners, CCW seen from outside"],
      "inner": [[0, 0, 0], "... same count and order as outer; inner[k] pairs with outer[k]"],
      "edges": [
        {
          "from": 0, "to": 1,
          "neighbor": "P002",
          "kind": "joint | rim_level | rim_whole",
          "length": 3.61,
          "bevel_deg": 15.0,
          "side_normal": [0.0, 0.0, 0.0]
        }
      ],
      "corner_angles_deg": [120.0],
      "face_depth": 0.31,
      "bounding_square": 6.6,
      "flat_xform": [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],
      "blank": 1,
      "blank_xform": [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],
      "map_uv": [[0.0, 0.0], "... outer corners in unrolled map coordinates (R_ref·θ, s)"]
    }
  ],
  "blanks": [ { "index": 1, "pieces": ["P001", "P007"] } ],
  "validation": {
    "passed": true,
    "checks": [ { "id": "R-VA-1", "name": "side_count", "passed": true, "failures": [] } ]
  },
  "stats": {
    "piece_count": 134,
    "by_sides": { "5": 0, "6": 120, "partial": 14 },
    "blanks_used": 40,
    "max_bevel_deg": 15.0,
    "max_face_depth": 0.31
  },
  "provenance": {
    "core_version": "0.1.0",
    "input_sha256": "hex"
  }
}
```

Rules:
- `edges[k]` runs from `outer[k]` to `outer[(k+1) % n]`; its side face is the quad
  `outer[k], outer[k+1], inner[k+1], inner[k]` and is planar with normal `side_normal`.
- `flat_xform` maps assembled 3D → flat layout at the origin. `blank_xform` maps flat → its
  position on its blank (blank `b` origin at `x = (b−1)·(blank_size + 2)`, `y = 0`).
- `map_uv` is for the assembly map only (not to scale); seam at `θ = 0`; pieces crossing the
  seam are shifted to the right-hand side.
- When `status = infeasible`, `pieces` and `blanks` are empty and `feasibility` explains why.
- When `status = failed_validation`, everything is filled so the shell can still draw a report.
- `pieces` sorted by `id`; `blanks` by `index`; maps in `by_sides` sorted by key.

## Stub output for early shell work

Task C0 ships `tests/fixtures/core_output_ring12.json`: a valid output for one ring of
12 flat rectangular tiles, 6 in tall, inscribed in a radius-9 cylinder (corners on the cylinder,
face plane at `9·cos 15° = 8.693`, tile width `2·9·sin 15° = 4.659`, joint bevels 15°,
level rims top and bottom, thickness 0.375). Every number in it is exact and hand-checkable.
SHELL builds S3–S6 against it before the real core exists.

## Size variation (A04)

TOML accepts percentages 5–50 (default 15), with shell-only shortcuts `tight` = 15
and `wide` = 40. Python divides by 100. CoreInput requires a finite numeric
fraction 0.05–0.50 inclusive, never strings or an omitted default. Both JSON
schemas carry this constraint. Suggestions remain side sets at the current
variation; messages separately name the smallest working increased variation,
rounded upward to 5 percentage points. Python migration is a separate SHELL step.

## A05 C5 piece reports

C5 emits `seed_depth` (inward inches), `max_corner_distance` (maximum signed
outer-corner distance in inches, positive outside) and `reflex_corners` (count)
on every generated piece, implementing the owner-approved A05 R-OUT-4 addition.
They are optional only for compatibility with historical ring12 fixture JSON.
Temporary C5 ids are seed-index ids `S000000`; C6 assigns final P ids.
Until C6/C7 run, status remains `failed_validation`, with an explicit pending
check; blank zero, identity transforms and empty map coordinates are not a
manufacturing-ready layout. Every removed/rejected polygon is listed in the
C5 check failures, including expected outside/whole-rim removals.
Selected level cut heights (deterministic inward search within one tile pitch,
capped at one quarter of the profile height) are stated in the run message.

## A06 section 9 settings (owner-approved 2026-11-19)

`min_triangle_edge` defaults to 2 inches and applies to every polygon and triangle
side. `max_polygon_edge` is optional (omitted or JSON null = no maximum); a finite
positive value is a hard cap. Actual largest side is always reported. `count_step`
is removed and rejected as an unknown setting. No alternative-layout settings.
One global n=floor(pi*r_min/min_triangle_edge), n>=3, and one h starting at
sqrt(3)/2*2*pi*r_min/n; raise h only for the corner-angle minimum. An infeasible
row/limit is an error, never a count change. C4/C5 retain all actual geometry gates.
Omitted randomness is 0 and depth is 0.25 inches, independent of size; depth 0
is smooth. A05 retains depth 1.5 and its existing geometry/default-angle rules.

`stats.actual_largest_side` reports the measured largest outer piece edge in inches
(including rim edges), or zero if no pieces exist. Layout estimates are not actual sides.

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

### A06 owner reporting and row-height additions (2026-11-20)

`params.row_height`: `constant` (default) or `proportional`; proportional uses
local nominal arc pitch `sqrt(3)/2 * 2*pi*r(s)/n`, with unchanged global count.
Stats separate `shortest_joint_edge` / `largest_joint_edge` from
`shortest_saw_cut_edge` / `largest_saw_cut_edge` (`rim_level` only).
Ranges are null without matching edges. Joint remnants on partials are measured,
not falsely certified as whole-edge gates; existing partial exemptions stay.
`largest_piece_width` is the largest actual outer+inner enclosing-square width
in inches, compared with `params.blank_usable`, and zero without pieces.
