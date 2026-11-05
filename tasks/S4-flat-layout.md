# S4 — Flat layout STEP writer

**Owner:** SHELL · **Depends on:** S3 · **Unblocks:** I1
**Read first:** SPEC R-OUT-2, docs/INTERFACE.md (`flat_xform`, `blank_xform`, `blanks`)

## Goal
Write `flat_layout.step`: every piece outer face up, placed on its blank, with blank outlines
and piece numbers.

## Deliverables
- In `py/vstone/step_out.py`: `write_flat_layout(core_output, path)`.
  - Each piece solid transformed by `blank_xform @ flat_xform` (just apply the matrices).
  - Each blank: a 12 × 12 outline and the usable square outline as wires, named `BLANK_01`…
  - Piece number as text on the outer face (build123d `Text`, engraved 0.02 deep or as a wire label —
    pick the one SolidWorks shows reliably and note it).

## Acceptance (ring12 fixture)
- [ ] 12 solids + 4 blanks (or whatever the fixture's `blanks` lists), labels correct.
- [ ] Every solid's outer face lies at z = 0 (±1e-6) with normal +Z.
- [ ] No solid crosses its blank's usable square (bounding-box check is enough).
- [ ] Byte-identical on repeat writes.

## Agent notes
