# PLAN

Goal of v1: `vstone run examples/cylinder_18x60.toml` produces the assembled STEP, flat layout
STEP, assembly map, report and preview for the 18 × 60 cylinder, all validation checks green,
and Jerry approves it in SolidWorks. Pegasus export, uploaded surfaces and the UI come after.

## Architecture

```
run config (TOML) ──► py/vstone (SHELL, Python)
                        │  read STEP profile → sampled polyline
                        │  build CoreInput JSON
                        ▼
                      core/ (CORE, Rust)  ── deterministic, all geometry ──
                        │  surface model → feasibility → seeds + topology
                        │  → flat tiles → rims → stone pieces + bevels
                        │  → validation → numbering → flat transforms → nesting
                        ▼
                      CoreOutput JSON
                        │
                      py/vstone: assembled.step, flat_layout.step, assembly_map.pdf,
                                 report.pdf, pieces.csv, preview.glb, run folder
```

Stack. Core: Rust stable, `nalgebra`, `rand_chacha`, `serde`/`serde_json`, `spade` (Delaunay),
`sha2`, `pyo3` + `maturin`. Shell: Python 3.11+, `build123d` (OpenCascade) for STEP,
`numpy`, `reportlab` for PDF, `trimesh` for GLB, `pytest`, `ruff`.

## Two tracks in parallel

The stub fixture from C0 lets SHELL build every output before the real core exists.

| Order | CORE track (ChatGPT, Rust) | SHELL track (Qwen, Python) |
| --- | --- | --- |
| 1 | C0 contract: types, JSON schemas, ring12 stub fixture | S1 scaffold: repo, Makefile, maturin, CI script |
| 2 | C1 surface model | S2 config + STEP profile reader |
| 3 | C2 feasibility check | S3 assembled STEP writer (from stub) |
| 4 | C3 seeds + topology + side-count control | S4 flat layout STEP writer (from stub) |
| 5 | C4 flat tile optimizer | S5 assembly map PDF (from stub) |
| 6 | C5 rims, stone pieces, bevels | S6 report PDF + CSV (from stub) |
| 7 | C6 validation + numbering | S7 CLI + run folder + preview.glb |
| 8 | C7 flat transforms + nesting | — |
| 9 | I1 integration: real core behind the CLI, end-to-end tests, golden hash | (joint) |
| 10 | G1 GATE: Jerry checks the cylinder in SolidWorks | |

Order at the start: C0 → S1 → everything else (C0 creates the Rust crate, S1 wraps it in the
Makefile and Python package). After that the tracks run independently until I1.
Each card lists its exact dependencies.

## Gates

- **G0 (after C2 + S2):** run the feasibility check alone on 4 profiles (cylinder, cone, bulge,
  hourglass). Jerry confirms the messages and suggestions make sense.
- **G1 (after I1):** Jerry opens `assembled.step` and `flat_layout.step` in SolidWorks for the
  cylinder at `randomness` 0 and 0.5. On approval, I1's golden hash is committed.
- **G2 (after G1):** same for a cone and a bulging vase profile with `[5,6,7]`.

## Risks and what the plan does about them

| Risk | Handling |
| --- | --- |
| The flat-tile optimizer (C4) is research-level and may not converge cleanly | It is isolated in one card with its own tests; GEOMETRY.md §4 gives the formulation; if C4 stalls, fall back to a Rhino trial for experiments only |
| On cylinders, planar hexes with corners on the surface must tilt (subtle zigzag) | Documented in GEOMETRY.md §4; G1 is where Jerry judges the look |
| Local model (Qwen) drifts into geometry code | AGENTS.md hard boundary; geometry lives only in Rust |
| Agents silently change the contract | Schema version + JSON Schema validation in tests on both sides |
| Non-reproducible output | Determinism rules + hash test in C0 onward + fixed STEP timestamps |

## After v1

- **L1** uploaded outer surface (STEP surface → triangle mesh → core `kind = "mesh"`)
- **L2** Pegasus CAM export (format decided by a one-hex import test)
- **L3** UI: reuse the cam-processor 3D viewer (repo `Jewronimo/cam-processor`) with sliders
- crease-aligned tiling, closed ends, colours, waists with concave pieces
