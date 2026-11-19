# C7 — Flat transforms and nesting

**Owner:** CORE · **Depends on:** C6 · **Unblocks:** I1
**Read first:** docs/GEOMETRY.md §7, SPEC R-OUT-2, R-TL-8, docs/INTERFACE.md (`flat_xform`, `blank_xform`)

## Goal
Lay each piece flat (outer face up) and pack pieces onto 12 × 12 blanks.

## Deliverables
- `core/src/flat.rs` — `flat_xform` per piece.
- `core/src/nest.rs` — deterministic nesting into `blank_usable` squares with `nest_gap`;
  fills `blank`, `blank_xform`, `blanks[]`, `stats.blanks_used`.

## Acceptance
- [x] After `flat_xform`, every outer corner has z = 0 (±1e-9), inner corners z = −t (±1e-9),
      outer-face centroid at the origin, longest edge along +X. (`runs/logs/C7-targeted.log`, `runs/logs/C7-full-path/`)
- [x] Nested footprints never overlap and keep `nest_gap` (polygon distance test). (`runs/logs/C7-polygon-clearance.log`, `runs/logs/C7-full-path/`)
- [x] Every footprint lies inside its blank's usable square. (`runs/logs/C7-targeted.log`, `runs/logs/C7-full-path/`)
- [x] Ring12: 12 pieces of 4.659 × 6 fit 3 per blank with the default gap (two upright side by
      side, one turned 90° above them: 6.25 + 4.659 = 10.909 ≤ 11.5), so 4 blanks. (`runs/logs/C7-targeted.log`)
- [x] Deterministic placements on repeat runs. (`runs/logs/C7-targeted.log`, `runs/logs/C7-full-path/`)

## Agent notes

### Implementation for review — 2026-11-18

Branch `task/C7-nesting`, separate worktree `runs/worktrees/C7-nesting`, base
`cee456d`. Added rigid area-centroid flattening, deterministic longest-edge
selection, normal/thickness/planarity guards, and deterministic conservative
nesting. Outer AND inner projections are enclosed by rotated rectangles;
rectangle separation proves at least the requested polygon clearance, including
inner-face overhang. Unlike the suggested exhaustive 0.05-inch grid, placement
uses sorted bottom-left rectangle contact coordinates with 15-degree rotations
at each position, first existing blank first. This finite contact search avoids
grid cost and packs ring12 as three pieces per blank (four blanks). It is not
an optimal polygon nester; conservative boxes can waste material or reject a
piece that would fit at an unsampled angle. No schema or dependency changes.

A06 production output evaluates C7 instead of the pending gate, fills blanks and
counts, and only reports `ok` when all C6 and C7 checks pass. Legacy output also
gets transforms/nesting but retains its explicit C6 validation/numbering pending
gate (outside this card); its contract assertion now reflects that distinction.

Targeted development evidence (not certification):
`runs/logs/C7-targeted.log` — layout tests (ring12 four blanks, both faces,
clearance/bounds, repeat placements, invalid inputs, area centroid and longest
edge) plus C6 validation regressions and library clippy. Runs inside `Vorona-dev`
using a copied isolated source tree `/tmp/vstone-c7` and isolated target dir
`/tmp/vstone-c7-target`; no host Rust. Parent owns official `make dev-check`,
acceptance, numerical production follow-up and review. Checklist stays pending
those gates. No merge, push, or official checks executed by this child.

Additional targeted full-path legacy regression:
`runs/logs/C7-contract.log` — `cargo test --test contract deterministic_and_errors`
passed, including two generated-output SHA-256 equality checks and the retained
legacy C6 pending guard. This is a targeted development run, not official gate
certification. Working tree is clean after the implementation/evidence commits.

### Parent verification — 2026-11-18

Official `make dev-check` passed on commit `43f12c3`: `runs/logs/C7-dev-check-0926.log`. Main post-C6 merge check passed: main worktree `runs/logs/main-C6-post-merge-0915.log`. Targeted tests use enclosing rectangles to establish clearance, not the polygon-distance test explicitly named in the card. The full acceptance matrix and production A06 layout review remain outstanding; checklist left open pending review and those gates. No C7 merge.

### Independent polygon clearance follow-up — 2026-11-18

Added `core/tests/layout_clearance.rs`: an independent monotone-chain convex
hull of transformed outer AND inner corners (the specified footprint), with
point/segment Euclidean distance, crossing-edge and containment detection. Every
same-blank pair is tested without reusing the production rectangle oracle.
Regressions include ring12, synthetic inner overhang, deliberate overlapping
placements, crossing polygons, contained polygons, touching/collinear edges and
diagonal sub-gap separation. Production generation of the actual A06 cylinder
fixture also passes: 380 pieces, 48 blanks, 0.25-inch gap. This full-path test is
ignored by default and was explicitly run for this evidence.

Targeted container evidence: `runs/logs/C7-polygon-clearance.log` (3 tests passed,
including full-path production, 35.04s; the printed panic is the deliberately
corrupted placement caught by the regression),
`runs/logs/C7-polygon-clearance-final.log` (final ordinary regressions), and
`runs/logs/C7-polygon-clearance-clippy.log` (test clippy with warnings denied).
No nesting algorithm, public contract or dependency changes. These are development
checks, not official dev-check or acceptance certification. Parent still owns
those gates, six-case follow-up and merge review; checklist remains pending.

### Six-case full-path measurements — 2026-11-18

`scripts/c7_measure.py` calls `vstone.core.generate` twice per case. Raw outputs,
repeat outputs, full status/messages, validation records and summaries are in
`runs/logs/C7-full-path/`. Timings are the first generate call in seconds, not
build or measurement time. Area is total outer-face footprint area; utilisation
is that area divided by 11.5² times blanks. The rounded-up area bound ignores
spacing, shape and inner overhang, so is a theoretical lower bound, not an
achievable nesting claim. Polygon-distance checks conservatively use the convex
hull of both projected outer and inner faces.

| Case | Pieces | Blanks | Area util. | Lower bound | Pieces/blank min/median/max | Generate s |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| cylinder smooth | 380 | 50 | 51.32% | 26 | 4/4/16 | 0.35 |
| cylinder symmetric 0.25 | 380 | 49 | 51.70% | 26 | 4/4/20 | 13.40 |
| cone smooth | 450 | 41 | 52.23% | 22 | 7/10/20 | 0.43 |
| cone symmetric 0.25 | 460 | 41 | 51.42% | 22 | 6/10/21 | 3.64 |
| vase smooth | 286 | 31 | 53.53% | 17 | 4/8/20 | 0.27 |
| vase symmetric 0.25 | 286 | 30 | 54.67% | 17 | 4/7/24 | 33.71 |

All six: status `ok`, message begins “N pieces; M blanks; 0 checks failed”
(full message in per-case summary); R-VA-1..8 and C7-layout pass. Independent
checks: all footprints inside the usable square; minimum same-blank polygon
clearance ≥ 0.25 in within 1e-9; outer z=0 and inner z=-0.375 within 1e-9;
full output exactly equal on repeat. Maximum observed z error < 4.87e-11 in.
The six runs used the same production code as `43f12c3`; subsequent `44a57f6`
adds only independent tests and card notes. Acceptance matrix remains required
before merge; no C7 merge.
