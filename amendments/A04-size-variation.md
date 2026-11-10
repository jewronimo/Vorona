# Amendment A04 — Numeric size variation

**Date:** 2026-11-10 · **Approved by:** Jerry · **Amends:** docs/INTERFACE.md,
JSON schemas, SPEC size variation and GEOMETRY §3; read after A03.

- TOML `size_variation` is a percentage in 5–50 inclusive, default 15.
  `tight` and `wide` are shell-only shortcuts for 15 and 40 percent.
- Python converts percent to fraction. CoreInput accepts only a finite numeric
  fraction in 0.05–0.50 inclusive; no string presets or implicit core default.
- `Feasibility.suggestions` remains side sets only, feasible at the requested
  variation. Never insert unchanged sets that require another variation.
- When the unchanged set becomes feasible solely by increasing variation, the
  message states the smallest working variation rounded upward to 5 percentage
  points, computed deterministically from this profile and numeric parameters.
  Example: `6 only is not possible at ±15 %: … With size variation of at least
  ±20 %, [6] works.` Side-set alternatives may follow.
- Python conversion/mirror changes are a separate SHELL step. CORE does not edit
  Python source or Python tests in this amendment's implementation.
