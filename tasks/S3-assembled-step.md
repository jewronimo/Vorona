# S3 — Assembled STEP writer

**Owner:** SHELL · **Depends on:** S1, C0 (stub fixture) · **Unblocks:** I1
**Read first:** SPEC R-OUT-1, docs/INTERFACE.md (`outer`, `inner`, `edges` rules)

## Goal
Write `assembled.step`: one closed solid per piece, named by id, in final position.

## Deliverables
- `py/vstone/step_out.py` — `piece_solid(piece) -> Solid`: outer face, inner face, one planar quad
  per edge (`outer[k], outer[k+1], inner[k+1], inner[k]`), sewn into a shell → solid.
  `write_assembled(core_output, path)`: solids in id order, each labelled with its id.
- Fixed STEP header timestamp (AGENTS.md §3).

## Acceptance (against `tests/fixtures/core_output_ring12.json`)
- [ ] File re-imports with 12 solids, labels `P001`…`P012`, each solid valid (`is_valid()`).
- [ ] Each solid volume within 0.1 % of the polyhedron volume computed from the fixture's faces
      (divergence-theorem formula in a test-only helper).
- [ ] Writing twice gives byte-identical files.
- [ ] Opens in SolidWorks without repair (Jerry checks once; note result in `## Agent notes`).

## Agent notes
