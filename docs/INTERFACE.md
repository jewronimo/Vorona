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
    "min_corner_angle_deg": 70.0,
    "bevel_cap_deg": 60.0,
    "depth": 2.0,
    "max_corner_angle_deg": 225.0,
    "min_bounding_square": 4.0,
    "allow_waists": false,
    "blank_size": 12.0,
    "blank_usable": 11.5,
    "nest_gap": 0.25
  }
}
```

A05 settings: `depth` is positive and finite (inches; omitted defaults to 2.0).
`max_corner_angle_deg` is finite and strictly between 180 and 270 degrees
(omitted defaults to 225); it bounds reflex corners. `thickness` defaults to
0.375 inches in the shell and may be thicker. The obsolete surface-tolerance
setting is not accepted. Depth governs seeds only, not face/corner distances;
C2 warns outside `[0.15, 0.30] × target_size` without refusing feasibility.
C3/C4 seed-depth and plane generation and C6 distance reports follow separately.

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
