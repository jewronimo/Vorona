import json,sys,pathlib,time,math,statistics,collections
from vstone.core import generate
base=pathlib.Path(__file__).resolve().parent.parent; outdir=base/'runs/logs/C7-full-path'; outdir.mkdir(parents=True, exist_ok=True)
profile,mode=sys.argv[1:]; key=f'{profile}-{mode}'
d=json.loads((base/'tests/fixtures/core_input_cylinder.json').read_text());d['surface']['profile']=json.loads((base/f'tests/fixtures/profiles/{profile}.json').read_text()); d['params'].update(pattern='voronoi_triangles',min_triangle_edge=1.5,max_polygon_edge=3.0,count_step=2,relief_base='auto',min_edge=1.0,min_corner_angle_deg=30.,max_corner_angle_deg=230.,bevel_cap_deg=60.,blank_usable=11.5,depth=0 if mode=='smooth' else .25,randomness=0,seed=1)
def apply(m,p):return [sum(m[i][j]*p[j] for j in range(3))+m[i][3] for i in range(3)]
def area(p):return abs(sum(a[0]*b[1]-a[1]*b[0] for a,b in zip(p,p[1:]+p[:1])))/2
def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
def hull(pts):
 p=sorted(set(pts))
 def half(q):
  h=[]
  for x in q:
   while len(h)>1 and cross(h[-2],h[-1],x)<=0:h.pop()
   h.append(x)
  return h
 return half(p)[:-1]+half(p[::-1])[:-1]
def segdist(a,b,c,d):
 def point(p,x,y):
  vx=y[0]-x[0];vy=y[1]-x[1]; den=vx*vx+vy*vy;t=max(0,min(1,((p[0]-x[0])*vx+(p[1]-x[1])*vy)/den)) if den else 0
  return math.hypot(p[0]-x[0]-t*vx,p[1]-x[1]-t*vy)
 ca,cb,cc,cd=cross(a,b,c),cross(a,b,d),cross(c,d,a),cross(c,d,b)
 if ca*cb<0 and cc*cd<0:return 0.
 return min(point(a,c,d),point(b,c,d),point(c,a,b),point(d,a,b))
def inside(p,poly):return all(cross(a,b,p)>=-1e-10 for a,b in zip(poly,poly[1:]+poly[:1]))
def distance(a,b):
 if inside(a[0],b) or inside(b[0],a):return 0.
 return min(segdist(x,y,u,v) for x,y in zip(a,a[1:]+a[:1]) for u,v in zip(b,b[1:]+b[:1]))
t=time.perf_counter();o=generate(d);runtime=time.perf_counter()-t;t=time.perf_counter();again=generate(d);repeat_runtime=time.perf_counter()-t
(outdir/f'{key}.json').write_text(json.dumps(o,indent=2));(outdir/f'{key}-repeat.json').write_text(json.dumps(again,indent=2))
by=collections.defaultdict(list);total=0.;flaterr=0.;bounds=[]
for p in o['pieces']:
 outer=[apply(p['flat_xform'],x) for x in p['outer']];inner=[apply(p['flat_xform'],x) for x in p['inner']]; flaterr=max(flaterr,*[abs(x[2]) for x in outer],*[abs(x[2]+d['params'].get('thickness',.375)) for x in inner]);total+=area(outer)
 pts=[apply(p['blank_xform'],x)[:2] for x in outer+inner]; poly=hull([tuple(x) for x in pts]);by[p['blank']].append((p['id'],poly)); off=(p['blank']-1)*14.; bounds.extend((off+.25-1e-9 <= x <= off+11.75+1e-9 and .25-1e-9 <= y <=11.75+1e-9) for x,y in poly)
minimum=float('inf');bad=[]
for blank,items in by.items():
 for i,(id1,a) in enumerate(items):
  for id2,b in items[i+1:]:
   dist=distance(a,b);minimum=min(minimum,dist)
   if dist<.25-1e-9:bad.append([blank,id1,id2,dist])
counts=[len(v) for v in by.values()]; summary=dict(case=key,runtime_seconds=runtime,repeat_runtime_seconds=repeat_runtime,status=o['status'],message=o['message'],pieces=len(o['pieces']),blanks_used=len(o['blanks']),stone_outer_face_area_in2=total,utilisation_percent=100*total/(len(o['blanks'])*11.5**2),area_lower_bound_blanks=math.ceil(total/11.5**2),pieces_per_blank_min_median_max=[min(counts),statistics.median(counts),max(counts)],deterministic_repeat=o==again,max_flat_z_error_in=flaterr,flat_pass=flaterr<=1e-9,bounds_pass=all(bounds),min_conservative_footprint_distance_in=minimum,clearance_pass=not bad,clearance_failures=bad,validation=o['validation'])
(outdir/f'{key}-summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2),flush=True)
if not(all(bounds) and not bad and flaterr<=1e-9 and o==again):sys.exit(1)
