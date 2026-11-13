# C4 vase: deeper relief and taller-row experiments

Native exploratory diagnostics, not official certification. No case passes. Baseline fit reused without rerun.

| Case | Status | Failed / whole | Sites | Size-band/degree entries | C4 violations |
|---|---|---|---|---|---|
| baseline | limits_failed | 32/48 | 96 | 0 | {'max_edge': 32, 'min_corner_angle_deg': 32, 'max_corner_angle_deg': 28} |
| d2.0-s1 | limits_failed | 32/48 | 96 | 0 | {'max_edge': 32, 'max_corner_angle_deg': 16, 'min_corner_angle_deg': 15} |
| d2.5-s1 | limits_failed | 19/54 | 96 | 0 | {'min_edge': 2, 'min_corner_angle_deg': 2, 'max_edge': 32, 'max_corner_angle_deg': 4} |
| d3.0-s1 | limits_failed | 33/55 | 96 | 0 | {'min_corner_angle_deg': 33, 'min_edge': 31, 'max_corner_angle_deg': 22, 'max_edge': 48} |
| d1.5-s1.5 | topology_spacing_failed | None/None | 72 | 116 | {} |
| d1.5-s2 | topology_spacing_failed | None/None | 64 | 92 | {} |

## Interpretation and invariants

All cases use vase_bulge [6], size_variation .30, randomness0, seed1; unchanged target6.5in, edge[1,6]in, angle[70,230]deg and other native manufacturing limits. The prior guarded pentagon-only experimental C2 bypass is retained; production C2 still rejects this hex vase. All accepted depth scaffolds are EXACTLY equal to baseline after removing only depth. Repeating row depth=(row%3)*depth/2. No colouring or extra seed motion.

Row spacing means multiplying the native MERIDIONAL ARC-LENGTH row pitch sqrt(3)/2 * circumference/column_count by1.5 or2, leaving radius, profile, circumferential column schedule, first physical row s=0, and all limits unchanged. Subsequent pitches are evaluated at each new row position. This is NOT multiplying every existing s or stretching the vase: fewer rows fit on the original vase. Two full auxiliary rows beyond each rim use the same scaled local pitch. Stretch cases use direct symmetric placement and native periodic triangulation, with NO repair, because repair would move sites and undo the specified row spacing. Native edge AND per-tile-average size/degree checks fail on these fixed scaffolds. Thus C4 fitting is intentionally not executed for either spacing case, rather than silently relaxing bands or undoing the experiment. Failure rows and exact violating edge ratios are saved. This establishes failure of the requested fixed scaffold, not a proof that every possible taller-row topology is impossible. Degree6 and size compliance are distinct checks.

None/None below means C4 not run, not0/0 or a passing row. Whole membership changes at deeper depths; improved deduplicated count is not an identical-domain acceptance comparison. Auxiliary/rim support polygons may become whole under the native geometrical domain test; retain them and their row IDs honestly. Heights are anchor Y clamped at physical rims; s is unclamped meridional position and identifies auxiliary continuation. Offset xyz uses native extended chart, not the clamped anchor. All units inches.

## baseline

All degrees: {4: 14, 5: 2, 6: 78, 3: 2}; internal nonhull: {6: 74}. Spacing residual/min/max normalized ratio: None. Solver time: 6.636246253s.

| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |
|---|---|---|---|---|---|---|---|
| 0 | [-8.16209713905398, -8.16209713905398] | [0.0, 0.0] | 8 / {4: 8} | 0/0 | [] | {} | [] / {} |
| 1 | [-4.08104856952699, -4.08104856952699] | [0.0, 0.0] | 8 / {5: 2, 6: 6} | 0/0 | [] | {} | [] / {} |
| 2 | [0.0, 0.0] | [0.0, 0.0] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 3 | [4.08104856952699, 4.08104856952699] | [3.8988291727896147, 3.8988291727896147] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 4 | [8.982214697203407, 8.982214697203407] | [8.623638150196495, 8.623638150196495] | 8 / {6: 8} | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 16, 'min_corner_angle_deg': 8, 'max_corner_angle_deg': 8} | [] / {} |
| 5 | [15.858977157628882, 15.858977244156794] | [15.384003424366016, 15.384003510359864] | 8 / {6: 8} | 8/8 | [40, 41, 42, 43, 44, 45, 46, 47] | {'max_corner_angle_deg': 8, 'min_corner_angle_deg': 8} | [] / {} |
| 6 | [21.20651268168218, 21.20651275452661] | [20.72178517618265, 20.721785249018424] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 7 | [27.25575869715309, 27.25575873351871] | [26.741792880578103, 26.741792916508125] | 8 / {6: 8} | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16, 'max_corner_angle_deg': 8, 'min_corner_angle_deg': 8} | [] / {} |
| 8 | [34.321380149433885, 34.32138018211138] | [33.634872538928846, 33.63487257043189] | 8 / {6: 8} | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'min_corner_angle_deg': 8, 'max_corner_angle_deg': 4} | [] / {} |
| 9 | [39.79754926661301, 39.79754926661301] | [38.881908080821404, 38.881908080821404] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 10 | [44.11719684086931, 44.11719684086931] | [40.0, 40.0] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 11 | [48.43684441512561, 48.43684441512561] | [40.0, 40.0] | 8 / {3: 2, 4: 6} | 0/0 | [] | {} | [] / {} |

All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: [{'band': 0, 'failed_tiles': 8, 'violation_entries': 32, 'whole_tiles': 16}, {'band': 1, 'failed_tiles': 8, 'violation_entries': 16, 'whole_tiles': 8}, {'band': 2, 'failed_tiles': 8, 'violation_entries': 32, 'whole_tiles': 16}, {'band': 3, 'failed_tiles': 8, 'violation_entries': 12, 'whole_tiles': 8}]

## d2.0-s1

All degrees: {4: 14, 5: 2, 6: 78, 3: 2}; internal nonhull: {6: 74}. Spacing residual/min/max normalized ratio: None. Solver time: 5.211653918s.

| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |
|---|---|---|---|---|---|---|---|
| 0 | [-8.16209713905398, -8.16209713905398] | [0.0, 0.0] | 8 / {4: 8} | 0/0 | [] | {} | [] / {} |
| 1 | [-4.08104856952699, -4.08104856952699] | [0.0, 0.0] | 8 / {5: 2, 6: 6} | 0/0 | [] | {} | [] / {} |
| 2 | [0.0, 0.0] | [0.0, 0.0] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 3 | [4.08104856952699, 4.08104856952699] | [3.8988291727896147, 3.8988291727896147] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 4 | [8.982214697203407, 8.982214697203407] | [8.623638150196495, 8.623638150196495] | 8 / {6: 8} | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 16} | [] / {} |
| 5 | [15.858977157628882, 15.858977244156794] | [15.384003424366016, 15.384003510359864] | 8 / {6: 8} | 8/8 | [40, 41, 42, 43, 44, 45, 46, 47] | {'max_corner_angle_deg': 8} | [] / {} |
| 6 | [21.20651268168218, 21.20651275452661] | [20.72178517618265, 20.721785249018424] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 7 | [27.25575869715309, 27.25575873351871] | [26.741792880578103, 26.741792916508125] | 8 / {6: 8} | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16, 'max_corner_angle_deg': 8, 'min_corner_angle_deg': 7} | [] / {} |
| 8 | [34.321380149433885, 34.32138018211138] | [33.634872538928846, 33.63487257043189] | 8 / {6: 8} | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'min_corner_angle_deg': 8} | [] / {} |
| 9 | [39.79754926661301, 39.79754926661301] | [38.881908080821404, 38.881908080821404] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 10 | [44.11719684086931, 44.11719684086931] | [40.0, 40.0] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 11 | [48.43684441512561, 48.43684441512561] | [40.0, 40.0] | 8 / {3: 2, 4: 6} | 0/0 | [] | {} | [] / {} |

All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: [{'band': 0, 'failed_tiles': 8, 'violation_entries': 16, 'whole_tiles': 16}, {'band': 1, 'failed_tiles': 8, 'violation_entries': 8, 'whole_tiles': 8}, {'band': 2, 'failed_tiles': 8, 'violation_entries': 31, 'whole_tiles': 16}, {'band': 3, 'failed_tiles': 8, 'violation_entries': 8, 'whole_tiles': 8}]

## d2.5-s1

All degrees: {4: 14, 5: 2, 6: 78, 3: 2}; internal nonhull: {6: 74}. Spacing residual/min/max normalized ratio: None. Solver time: 8.717925296s.

| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |
|---|---|---|---|---|---|---|---|
| 0 | [-8.16209713905398, -8.16209713905398] | [0.0, 0.0] | 8 / {4: 8} | 0/0 | [] | {} | [] / {} |
| 1 | [-4.08104856952699, -4.08104856952699] | [0.0, 0.0] | 8 / {5: 2, 6: 6} | 0/0 | [] | {} | [] / {} |
| 2 | [0.0, 0.0] | [0.0, 0.0] | 8 / {6: 8} | 1/6 | [18] | {'min_edge': 1} | [] / {} |
| 3 | [4.08104856952699, 4.08104856952699] | [3.8988291727896147, 3.8988291727896147] | 8 / {6: 8} | 1/8 | [24] | {'min_corner_angle_deg': 1, 'min_edge': 1} | [] / {} |
| 4 | [8.982214697203407, 8.982214697203407] | [8.623638150196495, 8.623638150196495] | 8 / {6: 8} | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 16} | [] / {} |
| 5 | [15.858977157628882, 15.858977244156794] | [15.384003424366016, 15.384003510359864] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 6 | [21.20651268168218, 21.20651275452661] | [20.72178517618265, 20.721785249018424] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 7 | [27.25575869715309, 27.25575873351871] | [26.741792880578103, 26.741792916508125] | 8 / {6: 8} | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16, 'max_corner_angle_deg': 4} | [] / {} |
| 8 | [34.321380149433885, 34.32138018211138] | [33.634872538928846, 33.63487257043189] | 8 / {6: 8} | 1/8 | [64] | {'min_corner_angle_deg': 1} | [] / {} |
| 9 | [39.79754926661301, 39.79754926661301] | [38.881908080821404, 38.881908080821404] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 10 | [44.11719684086931, 44.11719684086931] | [40.0, 40.0] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 11 | [48.43684441512561, 48.43684441512561] | [40.0, 40.0] | 8 / {3: 2, 4: 6} | 0/0 | [] | {} | [] / {} |

All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: [{'band': 0, 'failed_tiles': 10, 'violation_entries': 19, 'whole_tiles': 22}, {'band': 1, 'failed_tiles': 0, 'violation_entries': 0, 'whole_tiles': 8}, {'band': 2, 'failed_tiles': 8, 'violation_entries': 20, 'whole_tiles': 16}, {'band': 3, 'failed_tiles': 1, 'violation_entries': 1, 'whole_tiles': 8}]

## d3.0-s1

All degrees: {4: 14, 5: 2, 6: 78, 3: 2}; internal nonhull: {6: 74}. Spacing residual/min/max normalized ratio: None. Solver time: 5.285615753s.

| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |
|---|---|---|---|---|---|---|---|
| 0 | [-8.16209713905398, -8.16209713905398] | [0.0, 0.0] | 8 / {4: 8} | 0/0 | [] | {} | [] / {} |
| 1 | [-4.08104856952699, -4.08104856952699] | [0.0, 0.0] | 8 / {5: 2, 6: 6} | 0/0 | [] | {} | [] / {} |
| 2 | [0.0, 0.0] | [0.0, 0.0] | 8 / {6: 8} | 1/2 | [19] | {'min_corner_angle_deg': 1} | [] / {} |
| 3 | [4.08104856952699, 4.08104856952699] | [3.8988291727896147, 3.8988291727896147] | 8 / {6: 8} | 3/8 | [24, 27, 29] | {'min_corner_angle_deg': 3, 'min_edge': 2, 'max_corner_angle_deg': 2} | [] / {} |
| 4 | [8.982214697203407, 8.982214697203407] | [8.623638150196495, 8.623638150196495] | 8 / {6: 8} | 8/8 | [32, 33, 34, 35, 36, 37, 38, 39] | {'max_edge': 16} | [] / {} |
| 5 | [15.858977157628882, 15.858977244156794] | [15.384003424366016, 15.384003510359864] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 6 | [21.20651268168218, 21.20651275452661] | [20.72178517618265, 20.721785249018424] | 8 / {6: 8} | 0/8 | [] | {} | [] / {} |
| 7 | [27.25575869715309, 27.25575873351871] | [26.741792880578103, 26.741792916508125] | 8 / {6: 8} | 8/8 | [56, 57, 58, 59, 60, 61, 62, 63] | {'max_edge': 16, 'min_edge': 8, 'max_corner_angle_deg': 8, 'min_corner_angle_deg': 8} | [] / {} |
| 8 | [34.321380149433885, 34.32138018211138] | [33.634872538928846, 33.63487257043189] | 8 / {6: 8} | 8/8 | [64, 65, 66, 67, 68, 69, 70, 71] | {'max_edge': 16, 'min_corner_angle_deg': 16, 'min_edge': 16, 'max_corner_angle_deg': 7} | [] / {} |
| 9 | [39.79754926661301, 39.79754926661301] | [38.881908080821404, 38.881908080821404] | 8 / {6: 8} | 5/5 | [73, 74, 75, 77, 78] | {'max_corner_angle_deg': 5, 'min_edge': 5, 'min_corner_angle_deg': 5} | [] / {} |
| 10 | [44.11719684086931, 44.11719684086931] | [40.0, 40.0] | 8 / {6: 8} | 0/0 | [] | {} | [] / {} |
| 11 | [48.43684441512561, 48.43684441512561] | [40.0, 40.0] | 8 / {3: 2, 4: 6} | 0/0 | [] | {} | [] / {} |

All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: [{'band': 0, 'failed_tiles': 12, 'violation_entries': 24, 'whole_tiles': 18}, {'band': 1, 'failed_tiles': 0, 'violation_entries': 0, 'whole_tiles': 8}, {'band': 2, 'failed_tiles': 8, 'violation_entries': 40, 'whole_tiles': 16}, {'band': 3, 'failed_tiles': 13, 'violation_entries': 70, 'whole_tiles': 13}]

## d1.5-s1.5

All degrees: {4: 16, 5: 8, 6: 48}; internal nonhull: {6: 45}. Spacing residual/min/max normalized ratio: [3.50282774682065, 0.9395806000217901, 1.6433702095961062]. Solver time: Nones.

| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |
|---|---|---|---|---|---|---|---|
| 0 | [-12.24314570858097, -12.24314570858097] | [0.0, 0.0] | 8 / {4: 8} | None/None | [] | {} | [] / {} |
| 1 | [-6.121572854290485, -6.121572854290485] | [0.0, 0.0] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [] / {} |
| 2 | [0.0, 0.0] | [0.0, 0.0] | 8 / {6: 8} | None/None | [] | {} | [] / {} |
| 3 | [6.121572854290485, 6.121572854290485] | [5.858018705392122, 5.858018705392122] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [24, 25, 26, 27, 28, 29, 30, 31] / {'degree': 2, 'edge_size_band': 12} |
| 4 | [14.05515437237898, 14.05515437237898] | [13.595727868016287, 13.595727868016287] | 8 / {6: 8} | None/None | [] | {} | [32, 33, 34, 35, 36, 37, 38, 39] / {'average_size_band': 8, 'edge_size_band': 28} |
| 5 | [23.75215447226332, 23.75215447226332] | [23.263988093263073, 23.263988093263073] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [40, 41, 42, 43, 44, 45, 46, 47] / {'average_size_band': 8, 'degree': 2, 'edge_size_band': 32} |
| 6 | [33.82137164786245, 33.82137164786245] | [33.1524910795069, 33.1524910795069] | 8 / {6: 8} | None/None | [] | {} | [48, 49, 50, 51, 52, 53, 54, 55] / {'average_size_band': 8, 'edge_size_band': 16} |
| 7 | [42.03336384558928, 42.03336384558928] | [40.0, 40.0] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [] / {} |
| 8 | [50.24535604331611, 50.24535604331611] | [40.0, 40.0] | 8 / {4: 8} | None/None | [] | {} | [] / {} |

All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: None

## d1.5-s2

All degrees: {4: 14, 5: 6, 6: 42, 3: 2}; internal nonhull: {6: 39}. Spacing residual/min/max normalized ratio: [18.684311299367717, 0.9964644034458642, 2.165993940618184]. Solver time: Nones.

| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |
|---|---|---|---|---|---|---|---|
| 0 | [-16.32419427810796, -16.32419427810796] | [0.0, 0.0] | 8 / {4: 8} | None/None | [] | {} | [] / {} |
| 1 | [-8.16209713905398, -8.16209713905398] | [0.0, 0.0] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [] / {} |
| 2 | [0.0, 0.0] | [0.0, 0.0] | 8 / {6: 8} | None/None | [] | {} | [] / {} |
| 3 | [8.16209713905398, 8.16209713905398] | [7.828156763264258, 7.828156763264258] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [24, 25, 26, 27, 28, 29, 30, 31] / {'average_size_band': 8, 'degree': 2, 'edge_size_band': 16} |
| 4 | [19.46273363977957, 19.46273363977957] | [18.97815251459128, 18.97815251459128] | 8 / {6: 8} | None/None | [] | {} | [32, 33, 34, 35, 36, 37, 38, 39] / {'average_size_band': 8, 'edge_size_band': 32} |
| 5 | [33.048630961883774, 33.048630961883774] | [32.40552271277552, 32.40552271277552] | 8 / {5: 2, 6: 6} | None/None | [] | {} | [40, 41, 42, 43, 44, 45, 46, 47] / {'average_size_band': 8, 'degree': 2, 'edge_size_band': 16} |
| 6 | [44.2671562974471, 44.2671562974471] | [40.0, 40.0] | 8 / {6: 8} | None/None | [] | {} | [] / {} |
| 7 | [55.48568163301043, 55.48568163301043] | [40.0, 40.0] | 8 / {3: 2, 4: 6} | None/None | [] | {} | [] / {} |

All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: None

## Findings and delivery

Depth2.5 is best TESTED, not passing:19/54 native whole tiles fail, compared with baseline32/48. On the identical baseline ID set24–71, every tile remains whole at every positive depth: failures baseline32,depth2=32,depth2.5=18,depth3=27. New whole membership therefore does not explain away the improvement at2.5, but does change global denominators. No extrapolation or recommendation to relax limits. All deeper cases have74 internal nonhull degree6 sites, zero internal5/7. Stretched scaffolds also have only degree6 internally (45 and39); failure is physical size bands, not an internal pentagon/heptagon issue. Edge failure entries are endpoint-directed, so do not interpret116/92 as distinct failed edges or tiles. Per-row deduplicated topology-failing IDs are listed separately.

Artifacts root: `/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/` (main repo, not worktree). Prefix `C4-vaserows-` is new; old sweeps preserved. `summary.json` includes every tile ID by row, per-row arc/Y heights, degree histograms, failed/whole counts, violation types and normalized bands. Raw five case JSONs retain exact inputs and topology; successful topology cases also retain all native fitted/polished planes, corners, polygons and reports. Stretch cases retain exact edge ratios and failed average-band entries; C4 whole/failure reports are unavailable because native topology prerequisites failed.

Baseline deliverables:

- `C4-vaserows-baseline-sites-cells.json`: site u/v/s, extended offset xyz and clamped surface anchor xyz, depth/row/degree/auxiliary/hull, triangulation triples, adjacency, native corner xyz and ordered cell indices, planes, whole flags, exact input, provenance hashes.
- `C4-vaserows-baseline-sites.csv`: directly usable coordinate/metadata table.
- `C4-vaserows-baseline-tiles_symmetric_1.json`: prior A05 tile-export shape (id, normal, offset, corners), with vase surface instead of fictitious cylinder R/H.
- `C4-vaserows-baseline-input.json` and `C4-vaserows-baseline-native-export.json`: exact input and native topology/coordinates.
- `C4-vaserows-baseline-front.svg`: neutral black orthographic X/Y wireframe of exact whole cells and sites, both front/back overlaid. XML generated; rendered interaction/appearance unverified. Prior A05 export was machine-readable tiles JSON; no existing vase visual-export format was found.

## Commands and checks

Applied vorona-rust-container skill. Raw machine proof: Darwin Spookys-MacBook-Pro-2.local25.6.0 arm64; docker ps listed Vorona-dev and unchanged wyoming-whisper,wyoming-piper,zigbee2mqtt,homeassistant,mosquitto. All Rust commands ran ONLY in Vorona-dev. No production source edits, no changes to inherited files, no commits/merges/pushes, official check or interruption of other jobs. Initial export-only rebuild failed due missing local xyz binding; five runs used the already-successfully-built isolated harness (same geometry). Corrected export-only binding and rebuilt for native baseline coordinates. Both build logs retained; no case re-fit for this correction.

`C4-vaserows-all-commands.json` records exact geometry argv. Isolated target `/tmp/vstone-c4-vaserows-target`. Logs `C4-vaserows-build.log`, `C4-vaserows-export-build.log`, `C4-vaserows-clippy.log`, `C4-vaserows-invariant-test.log`, `C4-vaserows-export-invariants.log` retained. Corrected release build and targeted clippy -D warnings PASS. Four example/support invariant tests PASS (depth RNG, cone-control parity, average-only descent/certification gap, local/global objective delta). Aggregation asserts baseline exact input/topology identity, deeper topology identity excluding ONLY depth, exact repeating depths, symmetric exported adjacency and valid cell/triangle indices. These are targeted checks, not official certification.

Build / lint / targeted tests (HOST commands):
```sh
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-vaserows-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo build --release --example c4_vaserows
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-vaserows-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo clippy --release --example c4_vaserows -- -D warnings
docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-vaserows-target -w /src/runs/worktrees/C4-depth/core Vorona-dev cargo test --release --example c4_vaserows
```

### Parent-owned official worktree check — NOT RUN

From HOST C4 worktree root, create `/tmp/vorona-c4-vaserows-check.mk`:
```make
dev-check: dev-up
	docker exec -e CARGO_TARGET_DIR=/tmp/vstone-c4-vaserows-official-target -w /src/runs/worktrees/C4-depth Vorona-dev bash -c 'pip install --no-deps -e . && make check'
```
Then:
```sh
make -f Makefile -f /tmp/vorona-c4-vaserows-check.mk dev-check > /Users/spooky/Projects/Vorona/voronoi-stone/runs/logs/C4-vaserows-dev-check.log 2>&1
```
Default dev-check selects `/src` MAIN and is NOT worktree-correct without this override. No C4 acceptance gate marked done; no C5 work.
