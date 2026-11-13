# C4 row-depth size-band experiments

Checkpoint eb9b3ae. Symmetric r0/seed1/depth1.5, target6.5, original fixture manufacturing limits unchanged. ROW depths only; no coloring or seed-motion intervention. No production source edits, commits/merge/push, or official check. Container-only native geometry, CPU0; concurrent-load wall timings.

Experimental example locally snapshots seeds/sidecount and replaces ONLY C2 pentagon feasibility/repair quota for vase[6] when there are no dislocations/heptagon requirements. Production C2 is unchanged. Snapshots reuse production Seed/Topology types and all original spacing/degree/repair/solver limits.

## vase-six-v30

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-sizeband-vase-six-v30.json`

```json
{
  "status": "limits_failed",
  "seconds": 6.636246253,
  "failed": 32,
  "whole": 48,
  "entries": {
    "max_corner_angle_deg": 28,
    "max_edge": 32,
    "min_corner_angle_deg": 32
  },
  "degree_all": {
    "3": 2,
    "4": 14,
    "5": 2,
    "6": 78
  },
  "degree_nonaux": {
    "6": 64
  },
  "degree_hull": {
    "3": 2,
    "4": 14,
    "5": 2,
    "6": 4
  },
  "degree_nonhull": {
    "6": 74
  },
  "bands": [
    {
      "band": 0,
      "failed": 8,
      "whole": 16,
      "entries": {
        "max_corner_angle_deg": 8,
        "max_edge": 16,
        "min_corner_angle_deg": 8
      }
    },
    {
      "band": 1,
      "failed": 8,
      "whole": 8,
      "entries": {
        "max_corner_angle_deg": 8,
        "min_corner_angle_deg": 8
      }
    },
    {
      "band": 2,
      "failed": 8,
      "whole": 16,
      "entries": {
        "max_corner_angle_deg": 8,
        "max_edge": 16,
        "min_corner_angle_deg": 8
      }
    },
    {
      "band": 3,
      "failed": 8,
      "whole": 8,
      "entries": {
        "max_corner_angle_deg": 4,
        "min_corner_angle_deg": 8
      }
    }
  ]
}
```

## vase-six-v25

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-sizeband-vase-six-v25.json`

```json
{
  "status": "topology_error",
  "error": "cannot seed an infeasible profile",
  "production_feasibility": {
    "column_changes": 2,
    "columns": [
      7,
      8,
      7
    ],
    "feasible": false,
    "heptagons_needed": 0,
    "pentagons_needed": 4,
    "small_radius_ranges": [],
    "suggestions": [
      [
        5,
        6,
        7
      ],
      [
        4,
        6,
        7
      ],
      [
        5,
        6,
        8
      ],
      [
        4,
        6,
        8
      ]
    ],
    "waist_ranges": []
  },
  "geometry": "NOT RUN; no generated topology, counts unavailable"
}
```

## vase-six-v35

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-sizeband-vase-six-v35.json`

```json
{
  "status": "limits_failed",
  "seconds": 8.984316379,
  "failed": 32,
  "whole": 48,
  "entries": {
    "max_corner_angle_deg": 32,
    "max_edge": 32,
    "min_corner_angle_deg": 32
  },
  "degree_all": {
    "3": 2,
    "4": 14,
    "5": 2,
    "6": 78
  },
  "degree_nonaux": {
    "6": 64
  },
  "degree_hull": {
    "3": 2,
    "4": 14,
    "5": 2,
    "6": 2
  },
  "degree_nonhull": {
    "6": 76
  },
  "bands": [
    {
      "band": 0,
      "failed": 8,
      "whole": 16,
      "entries": {
        "max_corner_angle_deg": 8,
        "max_edge": 16,
        "min_corner_angle_deg": 8
      }
    },
    {
      "band": 1,
      "failed": 8,
      "whole": 8,
      "entries": {
        "max_corner_angle_deg": 8,
        "min_corner_angle_deg": 8
      }
    },
    {
      "band": 2,
      "failed": 8,
      "whole": 16,
      "entries": {
        "max_corner_angle_deg": 8,
        "max_edge": 16,
        "min_corner_angle_deg": 8
      }
    },
    {
      "band": 3,
      "failed": 8,
      "whole": 8,
      "entries": {
        "max_corner_angle_deg": 8,
        "min_corner_angle_deg": 8
      }
    }
  ]
}
```

## cone-567-v25

Artifact: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-sizeband-cone-567-v25.json`

```json
{
  "status": "pass",
  "seconds": 7.545165378,
  "failed": 0,
  "whole": 63,
  "entries": {},
  "degree_all": {
    "4": 14,
    "6": 91
  },
  "degree_nonaux": {
    "6": 77
  },
  "degree_hull": {
    "4": 14
  },
  "degree_nonhull": {
    "6": 91
  },
  "bands": [
    {
      "band": 0,
      "failed": 0,
      "whole": 14,
      "entries": {}
    },
    {
      "band": 1,
      "failed": 0,
      "whole": 14,
      "entries": {}
    },
    {
      "band": 2,
      "failed": 0,
      "whole": 21,
      "entries": {}
    },
    {
      "band": 3,
      "failed": 0,
      "whole": 14,
      "entries": {}
    }
  ]
}
```

## Interpretation

Vase25% cannot reach the native optimizer under the authorized pentagon-only bypass: C2 requires TWO column changes ([7,8,7]) as well as four pentagons. The diagnostic deliberately retains that independent dislocation requirement and returns native `cannot seed an infeasible profile`. No geometry pass/fail, whole denominator, or actual degree counts may be fabricated for it. Vase30/35 are experimental only, not accepted production inputs. Degree tables include ALL graph degrees: hull fans are separated and are not polygon side counts. Quarter bands are normalized seed anchor HEIGHT, bottom to top [0,.25),[.25,.5),[.5,.75),[.75,1]. Native violation entries are repeated per corner/edge; failed tiles are distinct whole-tile IDs. PASS applies only to native whole-domain C4, not C5/C6 or rim clipping.

## Parent official-check recipe (NOT run by child)

From host worktree root, supplemental `/tmp/vorona-c4-sizeband-check.mk`:
```make
dev-check: dev-up
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-sizeband-official-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
```
```sh
make -f Makefile -f /tmp/vorona-c4-sizeband-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-sizeband-dev-check.log 2>&1
```
Default recipe targets /src MAIN and is wrong for this worktree.

## Verification
Container fmt --check and clippy --release --example c4_sizeband -D warnings PASS. Targeted cone_control_topology_matches_production PASS: complete topology equals production C3 for cone[5,6,7]25%. Logs: runs/logs/C4-sizeband-{fmt,clippy,topology-test}.log. Prior cone[6]25% final result geometry/report is byte-value identical to this cone[5,6,7]25% result (JSON comparison, excluding timings). No5/7 anywhere in cone, not merely absent on whole tiles.
