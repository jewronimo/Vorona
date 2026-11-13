"""Aggregate native geometry and draw provided coordinates; no geometry decisions."""
import collections as C
import csv
import hashlib
import json
from pathlib import Path
R = Path('/Users/spooky/Projects/Vorona/voronoi-stone/runs/logs')
def read(n): return json.loads((R/n).read_text())
def write(n,d): (R/n).write_text(json.dumps(d,indent=2)+'\n')
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
b=read('C4-vaserows-baseline-native-export.json')
old=read('C4-depthsweep-d1.5-topology.json')
assert b['topology']==old['topology'] and b['input']==old['input']
base=read('C4-depthsweep-vase-six-v30-d1.5.json')
assert base['input']==b['input'] and not b['band_failures']
g=base['diagnostic']['result']['Err']['best']
export={'schema':'c4.vase-sites-cells/1','units':'inches; u/v radians/conformal; Y up','input':b['input'],'sites':b['sites'],'triangles':b['topology']['corners'],'adjacency':b['topology']['neighbors'],'boundary':b['topology']['boundary'],'corner_xyz':g['corners'],'cells_corner_indices':g['polygons'],'whole_tiles':g['whole_tiles'],'planes':g['planes'],'provenance':{'geometry_source':'C4-depthsweep-vase-six-v30-d1.5.json','geometry_sha256':sha(R/'C4-depthsweep-vase-six-v30-d1.5.json'),'native_sites_source':'C4-vaserows-baseline-native-export.json','native_sites_sha256':sha(R/'C4-vaserows-baseline-native-export.json'),'note':'No baseline fit rerun. Native topology byte-value equality asserted. Auxiliary xyz uses extended SurfaceQueries frame; anchor_xyz is clamped surface.point.'}}
write('C4-vaserows-baseline-sites-cells.json',export)
write('C4-vaserows-baseline-input.json',b['input'])
# Prior A05 tiles_symmetric_1.json format, preserving exact native plane/corners.
write('C4-vaserows-baseline-tiles_symmetric_1.json',{'D':1.5,'mode':'symmetric','seed':1,'surface':b['input']['surface'],'tiles':[{'id':i,'normal':g['planes'][i]['n'],'offset':g['planes'][i]['d'],'corners':[g['corners'][j] for j in poly]} for i,poly in enumerate(g['polygons']) if g['whole_tiles'][i]]})
with (R/'C4-vaserows-baseline-sites.csv').open('w') as f:
 w=csv.writer(f);w.writerow(['id','row','u','v','s_in','depth_in','degree','auxiliary','hull','x_in','y_in','z_in','anchor_x_in','anchor_y_in','anchor_z_in'])
 for t in b['sites']:w.writerow([t[k] for k in ['id','row','u','v','s','depth','degree','auxiliary','hull']]+t['xyz']+t['anchor_xyz'])
# Orthographic front rendering of exact exported coordinates; neutral strokes, no colouring.
svg=['<svg xmlns="http://www.w3.org/2000/svg" width="900" height="1000" viewBox="-12 -35 24 50"><rect x="-12" y="-35" width="24" height="50" fill="white"/><g transform="scale(1,-1)" stroke="black" fill="none" stroke-width=".025">']
for i,p in enumerate(g['polygons']):
 if g['whole_tiles'][i]:svg.append('<polygon points="'+' '.join(f'{g["corners"][j][0]},{g["corners"][j][1]}' for j in p)+'"/>')
for t in b['sites']:
 if not t['auxiliary']:svg.append(f'<circle cx="{t["xyz"][0]}" cy="{t["xyz"][1]}" r=".07"/>')
svg+=['</g></svg>'];(R/'C4-vaserows-baseline-front.svg').write_text(''.join(svg))
summary=[]
cases=[('baseline',base,b)]+[(f'd{dep}-s{sc}',read(f'C4-vaserows-d{dep}-s{sc}.json'),None) for dep,sc in [('2.0','1'),('2.5','1'),('3.0','1'),('1.5','1.5'),('1.5','2')]]
for name,d,metadata in cases:
 t=(metadata or d)['topology'];sites=(metadata or d)['sites'];sp=d.get('spatial');limits={x['tile']:x['limits'] for x in sp['tiles']} if sp else {}
 if sp:
  assert all(x['depth']==(x['row']%3)*d['input']['params']['depth']/2 for x in t['tiles'])
 whole={x['tile'] for x in sp['tiles'] if x['whole']} if sp else set()
 failed={i for i,v in limits.items() if v}
 if name.startswith('d') and d.get('row_scale')==1:
  assert {k:v for k,v in d['input']['params'].items() if k!='depth'}=={k:v for k,v in base['input']['params'].items() if k!='depth'}
  a=json.loads(json.dumps(t));z=json.loads(json.dumps(b['topology']))
  for group in [a,z]:
   for site in group['tiles']:site.pop('depth')
  assert a==z
 rows=[]
 for row in sorted({x['row'] for x in t['tiles']}):
  ids=[i for i,x in enumerate(t['tiles']) if x['row']==row];ss=[sites[i] for i in ids];viol=C.Counter(v for i in ids for v in limits.get(i,[]));bf=[v for v in d.get('band_failures',[]) if v['row']==row]
  rows.append({'row':row,'s_in':[min(x['s'] for x in ss),max(x['s'] for x in ss)],'anchor_y_in':[min(x['anchor_xyz'][1] for x in ss),max(x['anchor_xyz'][1] for x in ss)],'ids':ids,'tile_count':len(ids),'degrees':dict(C.Counter(len(t['neighbors'][i]) for i in ids)),'whole':len(whole.intersection(ids)) if sp else None,'failed':len(failed.intersection(ids)) if sp else None,'failed_ids':sorted(failed.intersection(ids)),'violations':dict(viol),'topology_failed_ids':sorted({v['tile'] for v in bf}),'topology_violations':dict(C.Counter(v['type'] for v in bf))})
 summary.append({'case':name,'status':d['status'],'total_sites':len(t['tiles']),'failed':len(failed) if sp else None,'whole':len(whole) if sp else None,'violations':dict(C.Counter(v for vv in limits.values() for v in vv)),'rows':rows,'degrees_all':dict(C.Counter(map(len,t['neighbors']))),'degrees_internal_nonhull':dict(C.Counter(len(t['neighbors'][i]) for i in range(len(t['tiles'])) if not t['boundary'][i])),'bands':sp['bands'] if sp else None,'size_band_failures':len(d.get('band_failures',[])),'spacing_residual':d.get('spacing_residual'),'seconds':d.get('seconds')})
 if name!='baseline':write(f'C4-vaserows-input-{name}.json',d['input'])
write('C4-vaserows-summary.json',summary)
lines=['# C4 vase: deeper relief and taller-row experiments','','Native exploratory diagnostics, not official certification. No case passes. Baseline fit reused without rerun.','','| Case | Status | Failed / whole | Sites | Size-band/degree entries | C4 violations |','|---|---|---|---|---|---|']
for d in summary:lines.append(f"| {d['case']} | {d['status']} | {d['failed']}/{d['whole']} | {d['total_sites']} | {d['size_band_failures']} | {d['violations']} |")
lines+=['','## Interpretation and invariants','','All cases use vase_bulge [6], size_variation .30, randomness0, seed1; unchanged target6.5in, edge[1,6]in, angle[70,230]deg and other native manufacturing limits. The prior guarded pentagon-only experimental C2 bypass is retained; production C2 still rejects this hex vase. All accepted depth scaffolds are EXACTLY equal to baseline after removing only depth. Repeating row depth=(row%3)*depth/2. No colouring or extra seed motion.','','Row spacing means multiplying the native MERIDIONAL ARC-LENGTH row pitch sqrt(3)/2 * circumference/column_count by1.5 or2, leaving radius, profile, circumferential column schedule, first physical row s=0, and all limits unchanged. Subsequent pitches are evaluated at each new row position. This is NOT multiplying every existing s or stretching the vase: fewer rows fit on the original vase. Two full auxiliary rows beyond each rim use the same scaled local pitch. Stretch cases use direct symmetric placement and native periodic triangulation, with NO repair, because repair would move sites and undo the specified row spacing. Native edge AND per-tile-average size/degree checks fail on these fixed scaffolds. Thus C4 fitting is intentionally not executed for either spacing case, rather than silently relaxing bands or undoing the experiment. Failure rows and exact violating edge ratios are saved. This establishes failure of the requested fixed scaffold, not a proof that every possible taller-row topology is impossible. Degree6 and size compliance are distinct checks.','','None/None below means C4 not run, not0/0 or a passing row. Whole membership changes at deeper depths; improved deduplicated count is not an identical-domain acceptance comparison. Auxiliary/rim support polygons may become whole under the native geometrical domain test; retain them and their row IDs honestly. Heights are anchor Y clamped at physical rims; s is unclamped meridional position and identifies auxiliary continuation. Offset xyz uses native extended chart, not the clamped anchor. All units inches.']
for d in summary:
 lines += ['',f"## {d['case']}",'',f"All degrees: {d['degrees_all']}; internal nonhull: {d['degrees_internal_nonhull']}. Spacing residual/min/max normalized ratio: {d['spacing_residual']}. Solver time: {d['seconds']}s.",'','| Row | s min–max (in) | Anchor Y min–max (in) | Sites / degrees | Failed / whole | Failed IDs | C4 entries | Topology failing IDs / entries |','|---|---|---|---|---|---|---|---|']
 for row in d['rows']:lines.append(f"| {row['row']} | {row['s_in']} | {row['anchor_y_in']} | {row['tile_count']} / {row['degrees']} | {row['failed']}/{row['whole']} | {row['failed_ids']} | {row['violations']} | {row['topology_failed_ids']} / {row['topology_violations']} |")
 lines += ['',f"All site IDs by row, exact row ranges and supplemental normalized bands: C4-vaserows-summary.json. Bands: {d['bands']}"]
Path('docs/prototypes/C4_vaserows_report.md').write_text('\n'.join(lines)+'\n')
print(json.dumps([{k:d[k] for k in ['case','status','failed','whole','size_band_failures','degrees_internal_nonhull']} for d in summary],indent=2))
