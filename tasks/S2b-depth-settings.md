# S2b — A05 depth settings handoff

**Owner:** SHELL (CORE contract portion delegated) · **Depends on:** A05

Read A01–A05 and S2. Approved A05 preserves schema /1 for this coordinated migration.

## CORE handoff

- Implemented depth > 0 finite, default 2.0; max_corner_angle_deg strictly
  180..270 finite, default 225. Defaults apply to omitted JSON keys.
- Removed obsolete tolerance from Params, both schema mirrors and input fixtures.
- Ring12 provenance rehashed for migrated input; geometry unchanged.
- C2 public message now reads params.depth; column selection remains size-only.
- C3/C4 and new output reports are deferred, not implemented here.
- Main integrated into task/S2b-depth-settings; existing Python/example edits preserved.

## Verification mapping (not executed by CORE)

- core/tests/contract.rs::depth_and_reflex_angle_contract_defaults_and_ranges:
  omission defaults, valid ranges, exclusive endpoints, non-finite Rust values,
  structured key-naming errors, rejection of obsolete JSON key.
- core/tests/contract.rs::roundtrip_and_schema: cylinder byte roundtrip/schema.
- core/tests/contract.rs::ring_input_provenance: ring byte roundtrip/schema/hash.
- core/tests/contract.rs::numeric_variation_contract_and_range: schema mirror parity.
- core/tests/feasibility.rs::reference_cylinder_uses_size_band_only_and_keeps_widths:
  depth cannot screen columns; width suggestions retained.
- core/tests/feasibility.rs::configured_depth_reaches_public_feasibility_message:
  configured depth warning/quiet range and unchanged feasibility.
- Existing depth_warning_is_inclusive_advisory_and_deterministic and
  default_depth_warning_reaches_generation_messages_without_refusal remain applicable.
- Parent owns shell acceptance from A05 §5 and full container checks.

## Agent notes

No tests/checks run, per requester. Container formatting only. No commit or push.

## Parent verification — 2026-11-11

- Official `make -f /tmp/vorona-a05-check.mk dev-check` passed, exit 0, in
  Vorona-dev at /src/runs/worktrees/A05. Override changes only working directory.
- Main-checkout evidence: runs/logs/S2b-A05-2043.log. Rust fmt/clippy/all tests,
  Python ruff and 49 pytest tests passed.
- Shell acceptance -> test_a05_defaults_and_core_keys (example/defaults, thickness,
  input keys); test_a05_invalid_settings (key-naming invalid/obsolete settings).
- Implementation remains unmerged for owner review.
- C3 continuation blocked by runtime: no attributed spawn/send tool exposed;
  gateway rejected specialist CLI continuation: “Gateway agent from agent exec
  would lose inter-session attribution.” No alternate CLI bypass attempted.
  Resume attributed vorona-core delegation for C3 then revised C4 when available.
