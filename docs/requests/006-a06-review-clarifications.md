# A06 review clarifications — owner decision requested

Status: proposed only; does not amend the approved contract.

Claude blocked the docs/A06 merge. Full review: runs/logs/A06-claude-review.log.
No merge or push has occurred.

Proposed resolutions for owner approval:

1. New defaults apply to voronoi_triangles. Explicit voronoi retains A05 defaults
   (depth 1.5, minimum corner angle 70, maximum corner angle 230) and its position-only randomness and
   repeating three-row seed-depth pattern. Explicit settings override defaults.
2. For voronoi_triangles, depth is finite and nonnegative; relief_base is auto or
   finite nonnegative inches. Replace A05 seed-band validation with seed depths
   in [resolved relief_base, resolved relief_base + depth]. This explicitly
   supersedes the A05 position-only randomness decision for this pattern only.
3. For voronoi_triangles, seed spacing is 2.3 * min_triangle_edge. Retain legacy
   target_size/allowed_sides fields for compatibility but do not use them to
   constrain rectified topology. Report derived seed spacing separately from reported piece/target size; they are not interchangeable.
4. C2 uses local circumference counts for voronoi_triangles; legacy side-set
   feasibility suggestions and the A05 recommended-depth warning do not apply.
   Actual geometric feasibility is decided by the A06 solver acceptance gates.
5. Auto relief_base needs a deterministic rule: propose a fixed-cap search over
   nonnegative base values, choosing smallest absolute median corner distance,
   with stable tie-breaking. CORE must document exact bounds/caps before coding.

Prototype PASS is reference evidence only, not certification of A06 acceptance.
Full section 5 acceptance replaces A05-only gates for the new pattern; legacy
regressions remain required for voronoi. C6 remains on hold.

## Contract audit additions (vorona-core)

6. Coordinated schema migration: retain the current /1 schema identifiers for
   this owner-approved migration; update Rust, both schemas, INTERFACE, shell,
   fixtures and tests together. Omitted pattern selects voronoi_triangles.
   Explicit voronoi selects legacy omission defaults. Explicit values remain
   unchanged. Core omission defaults for depth and minimum corner angle must
   follow the selected pattern, not just shell defaults or schema annotations.
7. New numeric settings: min_triangle_edge finite and strictly positive;
   max_back_gap finite and nonnegative (default 0.5 inches); numeric relief_base
   finite and nonnegative. The literal auto is transported unchanged by SHELL
   and resolved only in Rust. No new geometry computations in Python.
8. Preserve historical regression geometry: explicitly select voronoi in ring12
   and A05 inputs; preserve explicit historical depth/angle values (ring12 uses
   2 inches / 225 degrees). Split legacy cylinder regression input from the
   new A06 example fixture. Migrate byte-identical serialization fixtures and
   recompute ring provenance from the exact migrated input bytes, without
   changing its hand-checkable geometry.
9. S2c is settings-only. Until C3/C4 dispatch exists, voronoi_triangles must
   return an explicit unsupported/pending error, never silently use the A05
   generator. Full A06 acceptance runs after C3–C5, before geometry review.

Decision 5 authorizes CORE to propose exact deterministic search bounds, cap,
tie-break and solver/runtime accounting before implementing auto resolution;
it does not approve an unspecified production search. Transporting auto in
S2c does not require that search implementation.

## Evidence / current state

- Claude: BLOCK; runs/logs/A06-claude-review.log.
- vorona-core read-only audit: completed; no builds, tests or edits.
- Symmetry WIP: e506f46 on task/symmetry-repair, unmerged and unverified.
- Main: 96f3668; docs/A06: a6f9c85, not merged or pushed.
- These proposals require Jerry's approval under AGENTS.md section 2. After
  approval, incorporate the clarification addendum into A06 and obtain Claude
  re-review before merging. C6 remains on hold.
