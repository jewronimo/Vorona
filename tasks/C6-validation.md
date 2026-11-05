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
