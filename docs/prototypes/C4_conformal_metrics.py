"""Independent numeric checks on native diagnostic exports, not a geometry generator."""
import json, pathlib, numpy as np
root=pathlib.Path('/src/runs/logs')
rows=[]
for n,d in [(8,2.5),(10,1.5),(10,2),(10,2.5)]:
 raw=root/f'C4-conformal-penalty-{n}-{d:g}.json'; guarded=root/f'C4-conformal-rimguard-{n}-{d:g}.json'; raw=guarded if guarded.exists() and guarded.stat().st_size else raw
 x=json.load(open(raw));diag=x['diagnostic'];g=diag['result'].get('Ok') or diag['result']['Err']['best'];t=x['topology'];c=np.array(g['corners']);norm=np.array([p['n'] for p in g['planes']]);lens=[];angles=[];det=[];bevel=[];rim=[]
 for i,whole in enumerate(g['whole_tiles']):
  if not whole:continue
  ids=g['polygons'][i];rim.extend(c[ids,1]);assert len(ids)==6
  for k,ci in enumerate(ids):
   u=c[ci]-c[ids[k-1]];v=c[ids[(k+1)%len(ids)]]-c[ci];lens.append(np.linalg.norm(v));angles.append(180-np.degrees(np.arctan2(np.dot(np.cross(u,v),norm[i]),np.dot(u,v))));det.append(abs(np.linalg.det(norm[t['corners'][ci]])))
  for j in t['neighbors'][i]:bevel.append(np.degrees(np.arccos(np.clip(norm[i]@norm[j],-1,1)))/2)
 assert not g['report']['violated'],g['report']
 assert min(lens)>=1 and max(lens)<=6 and min(angles)>=70 and max(angles)<=230 and max(bevel)<=60 and min(det)>=1e-8
 assert min(rim)>=0 and max(rim)<=40
 rows.append(dict(columns=n,depth=d,whole_pre=sum(diag['fit']['whole_tiles']),whole_post=sum(g['whole_tiles']),violations_pre=len(diag['fit']['report']['violated']),violations_post=0,edge_range=[min(lens),max(lens)],angle_range=[min(angles),max(angles)],bevel_max=max(bevel),det_min=min(det),corner_y_range=[min(rim),max(rim)],report=g['report'],seconds=diag['total_seconds']))
json.dump(rows,open(root/'C4-conformal-summary.json','w'),indent=2)
print(json.dumps(rows,indent=2))
