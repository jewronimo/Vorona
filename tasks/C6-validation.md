# C6 — Validation and numbering

**Owner:** CORE · **Depends on:** C5 · **Unblocks:** C7, S6 (real data)
**Read first:** SPEC R-VA-1..9, docs/GEOMETRY.md §7 (numbering)

## Goal
Run every check, number the pieces, fill `validation`, `stats`, `status`, `message`.

## Deliverables
- `core/src/validate.rs` — one function per check, ids `R-VA-1` … `R-VA-8`, each returns
  `passed` and the failing piece ids (sorted).
- `core/src/numbering.rs` — ids `P001…` per GEOMETRY.md §7; `neighbor` fields rewritten to ids;
  `map_uv` filled.
- `stats` (R-VA-9), `status = failed_validation` when any check fails, plain-language `message`
  (e.g. `"134 pieces, all checks passed."` or `"3 checks failed: edge_length (P012, P047) …"`).

## Acceptance
- [ ] Ring12: all checks pass.
- [ ] Each check has a test with a hand-broken input that makes exactly that check fail and
      names the right piece ids.
- [ ] Numbering: on the cylinder, P001 is the bottom row tile whose centroid angle is nearest
      0 going counter-clockwise from +X seen from +Y; ids increase around then up.
- [ ] Every `neighbor` refers to an existing id, and neighbour relations are symmetric.

## Agent notes

### A06 C6 implementation — 2026-11-17

Isolated branch `task/C5-C6-triangles`, base main `2435e48`. Added A06
R-VA-1 through R-VA-8 grouping of the native C5 limit/solid/closure evidence,
independent piece/adjacency/shared-plane checks, deterministic row/angle numbering,
neighbor-id translation, outside-view seam-unwrapped positive-area maps and totals.
Polygon versus triangle edge limits use native seed provenance, never side count
(a polygon may itself have three sides after clipping). Partial exemptions are
unchanged. Measured face depth uses centroid and edge-midpoint surface distances.
C7 remains explicit pending; output never claims manufacturing success or blanks
packed. Legacy `voronoi` path unchanged.

C5 near-coplanar joints now share the plane through the edge perpendicular to
the mean face normal. Opposite owners get equal planes up to sign, rather than
two individually square but different planes. Actual small bevels are retained
instead of falsely reporting zero on noncoplanar faces; exact coplanar bevel is zero.
Tests include both-owner plane equality and inner-edge incidence, ring12 success,
each isolated failing check, symmetric-neighbor failure ids and positive map area.

Targeted development log: `runs/logs/C5-C6-targeted.log`. Parent owns official
`make dev-check`, acceptance and review; none executed by this child. Historical
card-wide checklist remains unclaimed pending parent gates and native six-case
numerical follow-up with the changed shared planes. No merge or push.
