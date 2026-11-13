import runpy,os,json,numpy as np
for n,d in [(8,2.5),(10,1.5),(10,2),(10,2.5)]:
 os.environ.update(N=str(n),D=str(d),OPT='1',OPENBLAS_NUM_THREADS='1')
 a=runpy.run_path('/src/runs/worktrees/C4-depth/docs/prototypes/A05_vase_proto.py')
 native=json.load(open(f'/src/runs/logs/C4-conformal-penalty-{n}-{d:g}.json'))
 planes=native['diagnostic']['fit']['planes']; plane_error=float(np.max(np.abs(np.einsum('ij,ij->i', np.array([p['n'] for p in planes]),a['P3'])-np.array([p['d'] for p in planes]))));assert plane_error<1e-9,plane_error
 t=native['topology'];uv=np.array([[s['u'],s['v']] for s in t['tiles']]); depths=np.array([s['depth'] for s in t['tiles']]);assert np.allclose(uv,a['uv'],atol=1e-14,rtol=0);assert np.array_equal(depths,a['depth'])
 # Hull rectangle diagonals may differ between Qhull and Spade. Compare all complete fans.
 nt=set(map(tuple,t['corners']));pt=set(map(tuple,a['T']));diff=nt^pt
 internal=set(i for i,b in enumerate(t['boundary']) if not b)
 assert not any(any(i in internal for i in tri) for tri in diff),diff
 out={'columns':n,'depth':d,'uv_max_delta':float(np.max(np.abs(uv-a['uv']))),'depth_equal':True,'seed_plane_max_error':plane_error,'triangles_equal':nt==pt,'hull_triangle_symmetric_difference':sorted(diff),'python_whole':len(a['whole']),'native_whole':sum(native['diagnostic']['fit']['whole_tiles'])}
 assert out['python_whole']==out['native_whole']
 json.dump(out,open(f'/src/runs/logs/C4-conformal-parity-{n}-{d:g}.json','w'),indent=2)
