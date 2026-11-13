"""Aggregate native diagnostics only; computes no geometry."""
import collections
import hashlib
import json
from pathlib import Path

ROOT = Path('/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs')
base = json.loads((ROOT / 'C4-depthsweep-d1.5-topology.json').read_text())
summary = []
for depth in ['0', '0.5', '1.0', '1.5']:
    path = ROOT / f'C4-depthsweep-vase-six-v30-d{depth}.json'
    d = json.loads(path.read_text())
    (ROOT / f'C4-depthsweep-input-d{depth}.json').write_text(json.dumps(d['input'], indent=2)+'\n')
    topology = d.get('topology', base['topology'])
    canonical = json.loads(json.dumps(topology))
    for tile in canonical['tiles']:
        tile.pop('depth')
    ref = json.loads(json.dumps(base['topology']))
    for tile in ref['tiles']:
        tile.pop('depth')
    assert canonical == ref
    assert {k:v for k,v in d['params'].items() if k != 'depth'} == {k:v for k,v in base['input']['params'].items() if k != 'depth'}
    tiles = d['spatial']['tiles']
    rows = []
    for row in sorted({t['row'] for t in topology['tiles']}):
        ids = [i for i,t in enumerate(topology['tiles']) if t['row']==row]
        rr = [tiles[i] for i in ids]
        rows.append(dict(row=row, tile_ids=ids, normalized_height_range=[min(t['normalized_height'] for t in rr),max(t['normalized_height'] for t in rr)], auxiliary=all(t['auxiliary'] for t in rr), failed_ids=[t['tile'] for t in rr if t['failed']], failed=sum(t['failed'] for t in rr), whole=sum(t['whole'] for t in rr), violations=dict(collections.Counter(n for t in rr for n in t['limits']))))
    internal = collections.Counter(len(topology['neighbors'][i]) for i in range(len(tiles)) if not topology['boundary'][i])
    bands=[]
    for band in range(4):
        rr=[t for t in tiles if t['whole'] and t['band']==band]
        bands.append(dict(band=band,failed=sum(t['failed'] for t in rr),whole=len(rr),violations=dict(collections.Counter(n for t in rr for n in t['limits']))))
    summary.append(dict(depth=float(depth),status=d['status'],failed=len({t['tile'] for t in tiles if t['failed']}),whole=sum(t['whole'] for t in tiles),violations=dict(collections.Counter(n for t in tiles for n in t['limits'])),rows=rows,bands=bands,internal_degree_histogram=dict(internal),topology_equal_excluding_depth=True,source=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
(ROOT/'C4-depthsweep-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
lines=['# C4 vase hex depth sweep', '', 'Native experimental results at checkpoint eb9b3ae; not an official make dev-check certification.', '', '| Depth (in) | Status | Deduplicated failed / whole | Violation entries |', '|---|---|---|---|']
for d in summary:
    lines.append(f"| {d['depth']} | {d['status']} | {d['failed']}/{d['whole']} | {d['violations']} |")
lines += ['', '## Actual rows (not quarter bands)', '', 'Rows are original Seed.row identifiers, including auxiliary support rows. Heights are seed-anchor Y normalized bottom-to-top, clamped by surface.point outside the ends, not optimized corner heights. Row s/u values are retained in native topology JSON. Whole tiles are the native solver domain; 0/0 rows are NOT an acceptance pass. Row depths repeat (row % 3) * depth/2.', '']
for d in summary:
    lines += [f"### Depth {d['depth']} in", '', '| Row | Normalized height min–max | Failed / whole | Failed tile IDs | Violation counts/types |', '|---|---|---|---|---|']
    for row in d['rows']:
        lo,hi=row['normalized_height_range']
        lines.append(f"| {row['row']} | {lo:.12f}–{hi:.12f} | {row['failed']}/{row['whole']} | {row['failed_ids']} | {row['violations']} |")
    lines += ['', 'Supplemental quarter bands, bottom to top: '+json.dumps(d['bands']), '']
Path('docs/prototypes/C4_depthsweep_report.md').write_text('\n'.join(lines)+'\n')
