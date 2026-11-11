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
- [x] File re-imports with 12 solids, labels `P001`…`P012`, each solid valid (`is_valid()`).
- [x] Each solid volume within 0.1 % of the polyhedron volume computed from the fixture's faces
      (divergence-theorem formula in a test-only helper).
- [x] Writing twice gives byte-identical files.
- [x] Opens in SolidWorks without repair (Jerry checks once; note result in `## Agent notes`).

## Agent notes

Implemented `piece_solid` by sewing the supplied outer, inner and edge quads;
no tile geometry is decided in Python. Export uses inch units and fixed timestamp.
OpenCascade increments assembly occurrence IDs process-wide; export normalizes
only those metadata IDs to stable id-order numbers for byte reproducibility.

Acceptance mapping: `test_reimport_labels_validity_and_volume` covers twelve
valid labelled solids, assembled bounds and independent divergence-theorem volume
within 0.1%; `test_byte_identical_and_id_order` covers repeated export with reversed
input ordering and fixed timestamp. `test_invalid_vertex_counts` covers malformed
input. Official container log: `runs/logs/S3-assembled-verify.log`.
Jerry approved the SolidWorks visual check on 2026-11-10: `assembled.step` opens
without repair, contains 12 bodies labelled P001–P012, and the joints are flush.
Review artifact: `runs/S3-review/assembled.step`.
