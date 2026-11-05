# I1 — Integration and golden tests

**Owner:** CORE leads, SHELL assists · **Depends on:** C7, S7 · **Unblocks:** G1
**Read first:** PLAN.md (goal of v1), SPEC R-EX-1

## Goal
The real core behind the CLI, end to end, on the reference cylinder.

## Deliverables
- Remove the stub path from the default run (keep `VSTONE_CORE_STUB` for tests).
- `tests/e2e/test_cylinder.py` — runs `vstone run examples/cylinder_18x60.toml` at randomness 0 and
  0.5; asserts status ok, all checks pass, 10–12 columns, bevels 12–25°, pieces fit blanks.
- `tests/e2e/test_profiles.py` — cone and vase with `[5,6,7]`: status ok; hourglass: infeasible.
- `tests/golden/` — SHA-256 of `core_output.json` for the cylinder at seed 42 (randomness 0 and 0.5).
  Recorded only after G1 approval; until then the test is skipped with reason "awaiting G1".

## Acceptance
- [ ] `make check` + e2e tests green on Jerry's machine.
- [ ] Full cylinder run ≤ 2 minutes.
- [ ] Repeat runs: identical run folder hashes.

## Agent notes
