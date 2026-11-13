# C4 conformal vase — native reproduction (2026-11-12)

Applied vorona-rust-container skill. Raw machine proof: `Darwin Spookys-MacBook-Pro-2.local 25.6.0 ... arm64`; `docker ps` listed Vorona-dev, wyoming-whisper, wyoming-piper, zigbee2mqtt, homeassistant, mosquitto. Only Vorona-dev used. No main edits, merge, push, or official certification.

## Scope and result

Exact owner-supplied `A05_vase_proto.py` preserved byte-for-byte from main (not reformatted). Diagnostic Rust uses its **analytic** vase r=6+4sin(pi*y/40), not the older sampled profile fixture, the same 20,001-entry conformal integration/interpolation table, angular pitch 2pi/N, conformal row pitch sqrt(3)/2 times angular pitch, k=-3 through ceil(v(H)/pitch)+3, and signed-k modulo-three depths. Native Spade periodic triangulation is independently compared with SciPy Qhull. This is not merely a deeper run of the old scaffold.

The native experimental penalty path uses the identical corner-target + .05 tilt fit, then .05 target/.05 tilt plus edge margins .2 inches and angle margins 3 degrees with reference penalty weights. Deterministic dense TRF with Jacobian scaling and 60 function-evaluation budget; not a claim of bitwise SciPy solver trajectory. Native tangent basis is a rotated/sign-flipped orthonormal basis, equivalent in geometry but finite-difference/SVD roundoff can choose different nonconvex fitted basins. Every output retains the complete native pre/post report, not just the prototype's selected summary. Experimental path is isolated in planes_conformal_experiment.rs and is **not called by production optimize**. Existing production monotone polish initially left violations; its restrictive no-new-violation-entry acceptance cannot reproduce the reference trajectory. It remains unchanged.

The prototype freezes fitted whole membership. Rust freezes precisely the same domain before penalty optimization; independent checks additionally assert final corners remain within physical rims. All four **chart-corrected** cases pass with the claimed final whole counts; no invalid tile is discarded. Whole membership excludes auxiliary seeds per approved policy. A preliminary support-chart interpolation bug produced a rim crossing at N=10/D=2.5; it is superseded, not a limitation of the corrected reproduction. An optional rim penalty was investigated on that preliminary chart, retained only as diagnostic code; **none of the final four results uses it**. Original preliminary artifacts are retained under C4-conformal-prechart-*.


## Evidence

Main repo `runs/logs/C4-conformal-summary.json` is the per-case numeric table, produced by independent `C4_conformal_metrics.py` assertions. Raw `C4-conformal-penalty-{N}-{D}.json` exports include parameters, all seeds, periodic triangles, adjacency, boundary flags, fitted and final plane equations, corner XYZ, ordered polygons, frozen whole flags, full native reports and timings. `C4-conformal-native-*.json` preserve the initial production-polish mismatch rather than overwriting it. `C4-conformal-python-*.log` contains unchanged prototype console output. `C4-conformal-parity-*.json` and parity log compare UV coordinates, depths, triangles and fitted whole counts; zero UV delta and identical triangle lists confirmed for all four cases on this reference platform; fitted native planes additionally checked against the exact Python offset seed coordinates.

Native reports check edge bounds 1..6, convex >=70 and reflex <=230 degrees, bevel <=60, determinant >=1e-8, minimum bounding square 4, entire inner+outer footprint <=11.5, simple outer and inner polygons, consistent inner orientation, thickness .375. Depth band validation precedes fitting; planes pass through offset points by construction. Corner distances are reported, not limited. Independent metrics check all measured pieces for six sides, exact manufacturing bounds, determinants and physical-rim containment. Targeted lint/tests are not the parent-owned official make dev-check.

## Why the old lattice differed

Old native symmetric scaffold stepped forward in meridional arc length using the current row's local radius; that is a forward-Euler approximation, **not** constant conformal spacing. Large pitch/radius variation accumulates row-position error on the vase. Earlier auxiliary-row bookkeeping also put physical first row at global row index 2, shifting its depth phase to D; the reference physical k=0 row has depth 0. The old sampled vase and its extended chart also differ from this analytic reference. Increasing depth on that old scaffold therefore did not test Claude's construction. This result identifies a working diagnostic construction, not evidence that only depth fixes production.

## Automatic selection proposal — bounded search, not a validated closed form

For arc-length profile (r(s),y(s)), principal curvatures are k_theta=y'(s)/r(s) and signed k_meridian (normal rotation per arc length). Use absolute normal variation across candidate tile spans: alpha_theta≈|k_theta|*ell_theta and alpha_s≈|k_meridian|*ell_s (dimensionless radians), or directly measure normal differences over each proposed cell. Do not use Gaussian K alone: cylinders/cones have K=0 but nonzero circumferential curvature. For polyline corners, use integrated normal jumps, not pointwise second derivatives.

1. Enumerate integer columns in a fixed bounded range around 2pi*r/target_size; rank by integrated squared relative size mismatch, then maximum normal variation, then integer column count. Generate constant-conformal rows, with deterministic phase candidates and support depth pattern tied to physical k.
2. Enumerate depth/target_size ratios e.g. .15,.20,.25,.30,.35,.40 and include the default 1.5-inch depth and observed 2.5-inch diagnostic. Warn outside approved recommended band; never silently reinterpret a warning as refusal or relaxation. Small-depth distortion and large-depth relief are nonmonotone, so do not use binary search in depth.
3. For each bounded candidate, validate native C2 feasibility AND native C3 local/average spacing, allowed degrees, then C4's complete frozen-domain manufacturing limits and final rim containment. Rank passing candidates by size mismatch, distance/relief preference, default-depth proximity, then stable integer/phase indices. Fixed candidate and solver caps; report best failures if none passes.

Evidence calibrates **only this vase**: N=8/D=2.5 and N=10/D=1.5,2,2.5 are tested ; no universal curvature-to-depth law is established. Both fixed-column candidates are experimental C2/C3 bypasses. For target 6.5 and ±15%, ring pitches for N=8 range 4.71..7.85 and N=10 3.77..6.28 inches, outside the approved size band in some regions. C2's curvature/defect request and C3's size bands are not overridden by a C4 manufacturing pass. Automatic production integration requires separately resolving those guards; do not claim full pipeline acceptance from these experiments. C5 deferred; domes not solved here.

## Exact commands (host)

All paths below refer to main's mount; worktree-correct `-w` is essential:

```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-vaserows-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_conformal_vase
docker exec Vorona-dev /tmp/vstone-c4-vaserows-target/release/examples/c4_conformal_vase 8 2.5
docker exec Vorona-dev /tmp/vstone-c4-vaserows-target/release/examples/c4_conformal_vase 10 1.5
docker exec Vorona-dev /tmp/vstone-c4-vaserows-target/release/examples/c4_conformal_vase 10 2
docker exec Vorona-dev /tmp/vstone-c4-vaserows-target/release/examples/c4_conformal_vase 10 2.5
docker exec -e OPENBLAS_NUM_THREADS=1 Vorona-dev python /src/runs/worktrees/C4-depth/docs/prototypes/C4_conformal_parity.py
docker exec Vorona-dev python /src/runs/worktrees/C4-depth/docs/prototypes/C4_conformal_metrics.py
```

Save native stdout to main `runs/logs/C4-conformal-penalty-{N}-{D}.json`; parity script reads these files. Build/check logs: C4-conformal-build.log, C4-conformal-penalty-build.log, C4-conformal-final-targeted.log. Initial unsuccessful lint log retained; corrected final lint is authoritative for targeted scope only.

### Parent-owned official check — NOT RUN

From HOST worktree root, write `/tmp/vorona-c4-conformal-check.mk`:

```make
dev-check: dev-up
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-conformal-official-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
```

Then run:

```sh
make -f Makefile -f /tmp/vorona-c4-conformal-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-conformal-dev-check.log 2>&1
```

Default `/src` is main; unmodified `make dev-check` would check the wrong checkout. No C4 full acceptance checkbox marked.

### Preliminary penalty baseline table (SUPERSEDED; not final evidence)

| Columns | Depth | Fitted whole | Fit native violation entries | Post frozen count | Post native violation entries | Post iterations | Seconds fit+penalty |
|---|---|---|---|---|---|---|---|
| 8 | 2.5 | 39 | 29 | 39 | 0 | 60 | 9.23 |
| 10 | 1.5 | 80 | 20 | 80 | 0 | 21 | 23.70 |
| 10 | 2 | 80 | 40 | 80 | 0 | 60 | 40.83 |
| 10 | 2.5 | 80 | 60 | 80 | 0 | 60 | 44.51 |

Counts are not deduplicated failure counts: entries include repeated edge/angle limits. The preliminary last row crossed a rim; final corrected results below do not. The prototype prints the fitted count unchanged and does not recheck rim membership. Corner-distance stats and reflex counts are in each full native report. These preliminary numbers came before an independently detected support-chart endpoint-clamping mismatch; retained as prechart artifacts, not final evidence. NumPy interpolation clamps outside its global endpoint range before binary search; matching this explicitly fixed support-point deltas (maximum 7.82e-14 inches). Final case results below supersede this table. The original file SHA-256 is `9d2c371afa87be12ddb07c5459c5bf13a17cc1d85dd20894e7e0aa4f78be3389` (identical in main and worktree).


## Final chart-corrected evidence (no rim guard)

| N | Depth | Whole pre/post | Native violations pre/post | Edge range | Angle range | Max bevel | Min determinant | Seconds |
|---|---|---|---|---|---|---|---|---|
| 8 | 2.5 | 39/39 | 28/0 | 1.1995–5.8014 | 72.991–227.024 | 33.911 | 0.11806 | 9.97 |
| 10 | 1.5 | 80/80 | 20/0 | 1.7844–5.8000 | 73.736–226.738 | 28.748 | 0.18652 | 15.69 |
| 10 | 2 | 80/80 | 40/0 | 1.5563–5.8001 | 78.924–219.317 | 34.024 | 0.18432 | 24.10 |
| 10 | 2.5 | 80/80 | 63/0 | 1.1965–5.8006 | 72.999–227.002 | 36.995 | 0.03287 | 21.24 |

Every corrected case independently passes final rim containment, exact edge/angle/bevel/determinant bounds, six-sided measured fans, and the entire native manufacturing report (zero violations including square, blank footprint, simplicity and inner orientation). Native whole flags are frozen from fit, but independent physical revalidation confirms the same actual whole counts. No full C2/C3 or official C4 acceptance is claimed.

Final cross-language checks: all four UV arrays match exactly, all depth arrays and full triangle lists match exactly, and native fitted planes contain the reference Python offset seeds with maximum errors 3.55e-14..3.91e-14 inches. Parity success log: main `runs/logs/C4-conformal-parity-final.log`; independent manufacturing assertions: `C4-conformal-metrics.log`. Targeted final container clippy passes with -D warnings, and two example regression tests pass (lattice counts/degrees/depth pattern and NumPy endpoint clamping). These checks are supplemental; the official worktree dev-check remains parent-owned and unrun.
