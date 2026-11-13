# C4 final FAST matrix evidence

Status: prepared for Claude review, NOT merged and NOT fully accepted.

Final code uses configured-first FAST. External deadlines: symmetric 60s; organic 90s. No numerical edits were made during this final verification.

Final matrix: 96 unique records; 89/90 acceptance passes; 6/6 report-only passes. Timeouts are incomplete search, not infeasibility.

## Acceptance → test → result

| Acceptance | Test/evidence | Result |
| --- | --- | --- |
| cone_9_to_6, r=0, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 10.112/10.122/10.148s; depths {'1.5': 10} |
| cone_9_to_6, r=0.5, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 7.454/9.154/28.523s; depths {'1.5': 9, '1.75': 1} |
| cone_9_to_6, r=1, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 7.101/9.286/32.519s; depths {'1.5': 6, '1.75': 3, '2.0': 1} |
| cylinder, r=0, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 19.100/19.263/19.432s; depths {'1.5': 10} |
| cylinder, r=0.5, seeds1–10 | final96 native full-limit reports | 9/10 pass; 1 timeout; wall min/median/max 22.808/28.426/90.090s; depths {'1.5': 8, '1.75': 1} |
| cylinder, r=1, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 18.742/24.193/57.759s; depths {'1.5': 9, '1.75': 1} |
| vase_bulge, r=0, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 16.422/16.550/16.615s; depths {'1.75': 10} |
| vase_bulge, r=0.5, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 3.869/5.078/8.353s; depths {'1.5': 10} |
| vase_bulge, r=1, seeds1–10 | final96 native full-limit reports | 10/10 pass; 0 timeout; wall min/median/max 4.159/5.051/7.243s; depths {'1.5': 10} |
| Report-only configured depths1/2 | six profile controls | 6/6 pass |
| Repository verification | C4-final-fast-dev-check.log | 69 Rust passed,13 ignored;49 Python passed;fmt/clippy/ruff passed |
| Build | C4-final-fast-build.log | release build passed |
| Determinism / exact fit reuse | final official checks plus targeted native byte-exact equivalence in C4_final_review.md | passed tested gates; full96 is not a repeated-output matrix |

## Unresolved deadline evidence

- cylinder, configured depth1.5, r=0.5, seed3: timeout=True, wall=90.090s. No completed geometry failure/infeasibility claim.

## Native report scope

All successful cases pass the native whole-tile guards. C4 does not certify C5 rim-clipped pieces or C6 output. Original <=60s cylinder claim is not satisfied for all organic cases. Historical original checklist is not silently marked complete.

Successful selected optimizer reports (corner distances are signed native report values, not the superseded original absolute surface tolerance):

| Profile | Corner distance min/max | Reflex corner count range | Violations |
| --- | --- | --- | --- |
| cone_9_to_6 | -3.49339498/1.46720756 | 42–42 | 0 |
| cylinder | -3.0570922/1.27066955 | 51–54 | 0 |
| vase_bulge | -3.5634912/1.9744469 | 32–40 | 0 |

## Lifecycle and provenance

Original host pipeline ended after73 records; container recovery appended only23 missing case keys using the same immutable release executable and canonical runner. No completed case was replaced or retried. Recovery completion marker exists and both stderr files are empty. Timings are sums of recorded per-case wall times, excluding lifecycle downtime.

Total recorded wall time: 1631.449s.

Logs below are in MAIN runs/logs (outside the worktree, intentionally not committed):

- `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-matrix96.jsonl` SHA256 `f93346c41715392cf8007ecfae459505afc40ceb1ef255e0dfd1fede58989c9b`
- `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-dev-check.log` SHA256 `39bce5292d89cbefd05b6b2a9ee547d62361dd67b729acb20cd63409b513a9db`
- `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-final-fast-build.log` SHA256 `e2632e77a84bba27f1631965b3c738dbe9cbcf384b4e48540c1fc037a61040eb`

Source SHA256 at verification (documentation excluded):

- `core/Cargo.toml` `3696c17528a2e7a935361398da4f4c8807c204678e27df58caf1d0d09676ace7`
- `core/build.rs` `f872c46e388fbf36b45d5733b84e95346617d6c2fd550ec5cc56e12fb67d1205`
- `core/examples/c3_diagnostics.rs` `2ffd549028f74b5f043aa95d03a0cf5c66d797885b84b4a97b6037a69a72f338`
- `core/examples/c4_ablation.rs` `5bda6988957e218f42cb2d5013c4026d7af92c3874fbbc8e2464ed4b9b030421`
- `core/examples/c4_ablation_support/mod.rs` `81912c889d7b9bb50f5a7285b73e081d019b64f2498966893e9aa15830028bfd`
- `core/examples/c4_conformal_vase.rs` `1bc2be365398c94103cea0c68d9b532aa535956a39981dafdc69ff8134a0d6b5`
- `core/examples/c4_depth_profile.rs` `bef88ae8ea31dbe62105a6fd56565c4c39eadf4f3ef0170e2a0a64e56001a5d0`
- `core/examples/c4_depthsweep.rs` `c7ee3a35ed0ab986e107686fcf14eead9d4e4a1c0d54dd7e64898efa1511d9f4`
- `core/examples/c4_matrix.rs` `937230c1a5526d12ddb759231f9c19c6f928bea8710704e427a65acbb56b5102`
- `core/examples/c4_native_export.rs` `db9a1e8ec25560fc5ded6ac46c6de0e27959c8c868839fe159aef5869d4d5baa`
- `core/examples/c4_production_matrix.rs` `62afb2a60e39694e7e59521d4d78df4de22b860e6dbbb491411093015c422f1c`
- `core/examples/c4_reference.rs` `ebbeae5f3b68670cd606d800f81c10836c0c7a690b724544a1d019430164d665`
- `core/examples/c4_report.rs` `a56e5220bd1ed02ec8d0c0fcead2927bdb8a2a58dee2987243048b32cb8f1f64`
- `core/examples/c4_sizeband.rs` `5725114743ca75c2d71c5d76ea0e722b2f1e89fe876deaba6af54e20aea0d585`
- `core/examples/c4_sizeband_support/seeds.rs` `33b729264161fbab09edb75cdd28711686ded8cb1012d8dc4caaf77700e17757`
- `core/examples/c4_sizeband_support/sidecount.rs` `0809bffbd3d942999f500214ba31c2fa2b81043577a5eaf3fa07ade41256c439`
- `core/examples/c4_spatial.rs` `c8e95b19c68aab3a1aa30ec4f7a5ab434a8333114f95d8006fa87dab54ca4498`
- `core/examples/c4_vaserows.rs` `5079be99bc48746cc7c5214fb3524058c8f9d0b7b92190be4f0038add729e839`
- `core/examples/c4_vaserows_support/seeds.rs` `d2d7d72ee84facfb87aa173422700b243e68e56fc813384342b5835bd772a906`
- `core/src/depth.rs` `eb020cc80c1b0911f091fc8db0288fc28135986fb6392fdffa31bd495c5c040e`
- `core/src/feasibility.rs` `68ccbc9152a49b4973d2c7d8ad90fbd02c94deb539078eb7f0f117345c8b2635`
- `core/src/lib.rs` `4c26c0830dbd427de6924efaa4fe5b47b759b45b7ae9dbc03f1c15b7961208a5`
- `core/src/main.rs` `270a59c24408a2d981d5e4e1445647819e4c9a37a1e2a51e9f1ef36d5dd26fe4`
- `core/src/model.rs` `87df933a101881ac8bbff08e0436b2d2c4c9caa4bdc038e03ca8e49d3e12f0da`
- `core/src/planes.rs` `0cdc0c0f0c7c8b8da90735c2b3b8edd34fb5d17ee66d20bdd7f1d8b2905873e8`
- `core/src/planes_conformal_experiment.rs` `7eae22aa9edf18569d13956cb9cfe829b547837c888454fa5c1c8c2ae1dfcdef`
- `core/src/py.rs` `1910ebef4cd8b333e8c7b73029d654978b9e1aeadfd5ba097a39072e196a9f32`
- `core/src/seeds.rs` `da10194bcd11480a8553bd59f506aeabbc610a2b999683092bb604ea756cacf9`
- `core/src/sidecount.rs` `49383ce37b67d175baeca50ea1f99ea828606bec3920bf26a09029c19e29771a`
- `core/src/surface.rs` `1093e57b056f29c56ddfcb8533eada0dae2c05e5837dd831d683c2cee54660f6`
- `core/src/tol.rs` `3fdcc46a9dda306fc1760e5c2c75905f15d375a5101e12a7b173aa22f9e8cbd5`
- `core/src/topology.rs` `06fecb7fb94fefd3a229cbc22e8e513959f13e46e4b631c8c1b41f5606bd739d`
- `core/tests/contract.rs` `76f78769739e416d1b5ecc37b900a373469baf73f833978620262a46761f1c29`
- `core/tests/depth.rs` `1567d09b5777e601f64935a4d5dc355d65d8652d8c0364002fc3bfbec0d1e057`
- `core/tests/feasibility.rs` `50955e59f3db2a76ed33ef9dd5a3ca5c999fed68561843e8bd83c260db9ae946`
- `core/tests/planes.rs` `90d433ff182141e6e1c42f8b8291b4e772b69237cc4746405d7def797f1bd578`
- `core/tests/surface.rs` `b76ea64bef542cb58fd7653039ca54ae4e7e6b968bd89b03b4b6753a0fc0ad14`
- `core/tests/topology.rs` `b91112636e78b5d7290b070b7f342f7c211de1f0e244d6d2b334a15f0a4c10c7`
