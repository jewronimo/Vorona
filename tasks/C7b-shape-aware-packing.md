# C7b — Shape-aware packing

**Owner:** CORE · **Depends on:** C7 · **Status:** not started

## Goal
Pack using real projected outer and inner polygons with `nest_gap`, including triangle pairing, while retaining every C7 check and deterministic output.

## Acceptance
- [ ] No overlap; pairwise projected polygon clearance >= `nest_gap`, including inner faces; all footprints within usable squares.
- [ ] Preserve all C7 flattening, clearance, bounds, ring12 and repeat-determinism checks.
- [ ] Target fewer than 50 / 41 / 31 blanks on cylinder / cone / vase smooth, respectively; document measurements and any misses. Baseline utilisation 51–55%; area-only lower bounds 26 / 22 / 17.
- [ ] Document triangle pairing, runtime, full-path evidence and any tradeoffs; stop for review before merge.
