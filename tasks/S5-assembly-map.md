# S5 — Assembly map PDF

**Owner:** SHELL · **Depends on:** S1, C0 (stub fixture) · **Unblocks:** I1
**Read first:** SPEC R-OUT-3, docs/INTERFACE.md (`map_uv`, `edges[].neighbor`)

## Goal
A printable map of the structure unrolled flat, so any piece can be placed by matching numbers.

## Deliverables
- `py/vstone/assembly_map.py` — `write_assembly_map(core_output, path)` with reportlab:
  - draw each piece's `map_uv` polygon; piece id large in the middle
  - on each edge, the neighbour's id small, just inside the edge (rim edges: "rim")
  - title block: run id, piece count, "not to scale", date of run, seam marked at both sides
  - fits the drawing to pages (tile across several Letter-landscape pages with overlap marks
    when one page would make text smaller than 6 pt)

## Acceptance (ring12 fixture)
- [ ] PDF has 12 polygons, 12 big ids, 24 neighbour labels and 24 "rim" labels
      (check via the text layer with `pypdf`).
- [ ] No text below 6 pt.
- [ ] Byte-identical on repeat writes (fixed creation date in PDF metadata).

## Agent notes
