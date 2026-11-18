# A06 C5 owner rim/parity handoff — 2026-11-17 (UNVERIFIED)

Existing dirty task/C4-a06-solver worktree preserved; no commit/merge/push.
Only container cargo fmt executed. Parent owns all builds, checks, numerical
comparisons and STEP exports. A05 unchanged; C6 held.

## Source comparison completed before solver edits

Exact requested reference: A06_tri_proto_v2.py, PROFILE=vase, COUNT=constrow:11:4.5,
D=.25, BASE=auto, TMIN=1.5, MAXE=3, RELIEF=rows, JIT=0. NFEV must be supplied
(otherwise prototype does NOT optimize). A also affects padding/row phase even
under constrow; its default is 8.5. Do not claim identical inputs without recording A.

Concrete differences:

- Prototype starts rows at y=-1.6*A; native starts at profile arc s=-2*h and
  includes s=0. Prototype advances conformal v by h/r at each row; native
  advances exact profile arc h. `constrow` comment says physical height, but
  the two slope factors on that line cancel; it is NOT exact Y or arc spacing
  on a curved vase. Different phase also changes row modulo-3 relief.
- Native layout selects n/h by its own deterministic §9.3 oracle; prototype
  constrow forces n11/h4.5. Neither result is automatically identical input.
- Prototype uses determinant +1e-6 during optimization (+1e-9 initially);
  native exact cross-product dual intersection preserves true shared planes.
  Retained exact native intersection, no flatness weakening.
- Prototype optimizes initial centroid-in-band pieces (frozen); native validity
  gates wholly-in-band pieces dynamically, plus physical inner guards. Native
  additionally checks exact blank square, simple polygons, signed inner edges,
  back gap, and exact rotational copies. Prototype diameter surrogate and
  half-degree sampled square do not certify these native limits.
- Prototype has .05 tilt and .02 slide regularization; native used .02 for both.
  Corrected native weights to .05/.02, without changing configured validity.
- Prototype scipy sparse TRF uses x_scale=jac; native damped GN/CG used raw
  tilt-versus-inch-slide columns. Added column normalization and physical step
  unscaling. Native remains fixed-cap damped GN, NOT claimed TRF parity.

Previous rim-aware candidate duplicated residual/barrier code inside geometry,
referencing polygons/inner/barrier out of scope (parent E0425). Removed that
entire misplaced block. Also removed fixed-physical-end rim residual rows from
evaluate and sparsity: these fought owner's cut-height-first policy and assumed
4in partials were impossible for <=3in triangles. Retained actual back-gap rows
in evaluate/sparsity (both inner fan dependencies); they are real limits.

## Owner rim policy implementation

§9.2 records both saw-edge max exemption and 1.5in partial square. A06 assembly
now uses its provenance-preserving equivalent of existing C5 requested-first,
128-step bounded inward placement (bottom, then bottom-clipped top). All final
C5 clipping still rejects/report failures and certifies closure at ACTUAL selected
cut heights. This is authorized saw placement, not a solver shortcut secretly
certifying a smaller fixed band. Rejected rim neighbors become square rim_whole
sides before solids build. No exemption from remaining limits or closure.

## Parent serial verification

From this worktree, save raw logs. First `make dev-check`; only after it passes:

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-a06-owner-rim -w /src/runs/worktrees/C4-a06-solver/core Vorona-dev cargo build --release --example a06_report
docker exec -w /src/runs/worktrees/C4-a06-solver Vorona-dev taskset -c 0 bash scripts/a06_matrix.sh /tmp/vstone-a06-owner-rim/release/examples/a06_report runs/logs/C5-owner-rim-required required
```

Harness now uses exact symmetric .25 (not stale .3), smooth0; required six cases
90s including deterministic repeat. Report-only .5 symmetric and .25 random,
seeds1..3. Configured §9.4 default remains .3; §9.6 records explicit distinction.

Only export a report whose accepted=true (result Ok, assembly Ok, no errors or
violations, closure Ok, determinism true). For each of the six JSON files:

```sh
docker exec -w /src/runs/worktrees/C4-a06-solver Vorona-dev python -c 'import json,sys; from vstone.step_out import write_assembled; d=json.load(open(sys.argv[1])); assert d["accepted"] and d["determinism"]; write_assembled({"pieces":d["assembly"]["Ok"]["pieces"]},sys.argv[2])' runs/logs/C5-owner-rim-required/a06-vase_bulge-symmetric-s1.json runs/logs/C5-owner-rim-required/vase_bulge-symmetric.step
```

Repeat serially for cylinder/cone_9_to_6/vase_bulge × smooth/symmetric. Parent
must inspect per-case errors/back-gap/rim heights; no numerical success or
readiness for owner review claimed by this source-only handoff. Source differences
above remain to be measured; this is NOT an identical-input parity certificate.

## Solid-aware placement repair — child source handoff (UNVERIFIED)

Fresh parent `/tmp/vstone-a06-owner-rim` matrix is immutable evidence for the
PREVIOUS source. Child did not build/run tests/numerics or touch those logs.
Current changes require a new parent build after dev-check.

Root causes established from source and saved parent failures:

1. `triangle_place` scores ONLY outer clipping/sliver status. It stops at requested
   height when that count is zero, even if the actual thickness offset collapses
   or the cut creates edges below the unchanged minima. Thus zero slivers did not
   mean a usable saw placement. A06 assembly now scores actual complete assembly
   candidates (solid errors + manufacturing violations + closure failure), ties
   broken by total reported deficit; exact ties retain the earliest height.
   Requested-first, 128 inward samples over capped one-row pitch, bottom before top
   remain unchanged. Top candidates use the chosen bottom. No acceptance limit
   or failure exemption changed. The search may still fail; report it honestly.
   It can evaluate up to 257 assemblies, so parent must measure its runtime.
2. The kept-neighbor set was populated BEFORE solids were built. A collapsed
   solid therefore left its neighbors referencing absent IDs. Build rejection is
   now propagated to a fixed-point retained set before exported solids are built;
   exposed sides become square `rim_whole`. Every rejected solid's original error
   remains, and missing physical material still fails closure. This is NOT hole
   suppression or a waiver of the failed solid.

Regression source: unit `failed_solid_neighbor_becomes_square_without_hiding_rejection`
checks existing neighbors retained, removed neighbors square/null and zero bevel.
The ignored release smooth-cylinder regression now requires ALL solid errors and
violations empty and actual closure Ok, instead of checking only offset errors.
Parent should run this ignored regression plus fresh six-case required matrix;
the six-case report remains the full cone/vase/symmetric regression coverage.
Only container cargo fmt and host git diff --check ran in this child (passed).
No claim these source changes compile or make any required case accepted.

Vase convergence/prototype remains UNMEASURED in this child: parent owns all
numerical jobs. Historical C4-a06-vase-symmetric.json is stale unequal-count
failure, not evidence for the current constant-count source. Read the fresh
C5-owner-rim-required vase symmetric result before making further solver edits.
Exact reference settings and meaningful-input differences remain recorded above;
no evidence supports calling the reference/native geometries identical. Prototype
also has diameter/angle/edge safety margins different from native objective and
no equivalent constant-thickness/closure certification. Do not weaken native
checks to imitate prototype success. Six accepted+deterministic reports followed
by six STEP exports remain mandatory and NOT completed by this child.
