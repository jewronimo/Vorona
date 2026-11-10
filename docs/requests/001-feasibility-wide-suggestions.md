# C2: wide-variation suggestion metadata cannot be represented

The frozen `Feasibility.suggestions: Vec<Vec<u32>>` carries only side sets.
GEOMETRY §3 also requests recommendations with `size_variation = wide`.
For the cone, tight `[6]` is infeasible while wide `[6]` is feasible. Returning
`[6]` as a suggestion would claim the unchanged current parameters work, without
being able to identify the required variation change.

Owner decision requested: approve a future structured suggestion representation
(side set plus variation), or explicitly keep side-set-only recommendations.
No contract/schema changes made. C2 currently emits only side sets proven feasible
under the current variation. Explicit wide inputs are checked normally. The
wide-only suggestion part is stopped pending approval; other C2 work proceeds.

## Resolution — owner approved 2026-11-10 (A04)

Keep suggestions as side sets proven feasible at the requested numeric variation.
The message separately reports the unchanged set's smallest working variation
rounded upward to 5 percentage points. The core evaluates every ascending 5%
grid point through 50%, not a preset wide branch. Wrap intervals expand
monotonically and all other feasibility conditions are variation-independent,
so the first successful grid point is the rounded-up actual threshold.
For cone 9→6 with target size 6, the actual threshold is about 21.46%, hence
the recommendation is ±25% (not the illustrative ±20% from A04).
Python percent/shortcut conversion is a separate SHELL step. This resolves
the representation blocker without adding suggestion metadata.
