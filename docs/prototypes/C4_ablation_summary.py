import json, pathlib, collections
root=pathlib.Path('/Users/spooky/Projects/Vorona/voronoi-stone')
rows=[]
for p in sorted((root/'runs/logs').glob('C4-ablation-*-*.json')):
 if p.name.endswith('-initial.json'): continue
 d=json.loads(p.read_text()); tiles=d['spatial']['tiles']; whole=[t for t in tiles if t['whole']]
 def group(ts):
  return {'failed':sum(t['failed'] for t in ts),'whole':len(ts),'violations':dict(collections.Counter(v for t in ts for v in t['limits']))}
 summary={'profile':d['profile'],'arm':d['arm'],'status':d['status'],'seconds':d['seconds'],'overall':group(whole),'near':group([t for t in whole if t['near_5_7']]),'far':group([t for t in whole if not t['near_5_7']]),'bands':[{'band':b,**group([t for t in whole if t['band']==b])} for b in range(4)],'near_bands':[{'band':b,**group([t for t in whole if t['band']==b and t['near_5_7']])} for b in range(4)],'degree_histogram_all':d['degree_histogram_all'],'degree_histogram_nonaux':d['degree_histogram_nonaux'],'defects':d['defect_tiles'],'intervention':d['intervention'],'path':str(p),'params':d['params']}
 rows.append(summary)
(root/'runs/logs/C4-ablation-summary.json').write_text(json.dumps(rows,indent=2)+'\n')
for x in rows: print(x['profile'],x['arm'],x['status'],[(k,x[k]['failed'],x[k]['whole']) for k in ('overall','near','far')],round(x['seconds'],3),x['intervention'].get('selected_fraction'),x['intervention'].get('equal_edges_before'),x['intervention'].get('equal_edges_after'))
