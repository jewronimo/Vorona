"""Identical-input A05 reference solver; no seeding, reordering or triangulation.
Usage: python A05_native_compare.py native.json comparison.json
The planes/corners/resid and scipy least_squares call are the A05 method,
with its ey basis generalized to the explicitly exported native second axis.
This is a CORE research diagnostic, never production Python geometry.
"""
import hashlib
import json
from pathlib import Path
import sys
import numpy as np
import scipy
from scipy.optimize import least_squares

source = Path(sys.argv[1])
data = json.loads(source.read_text())
P3, radial, etheta, targets = (np.array(data[k], dtype=float) for k in
                               ("points", "radial", "etheta", "targets"))
N = len(P3)
ey = np.array(data.get("axis_b", np.tile([0., 1., 0.], (N, 1))))
TRI = np.array(data["triangles"], dtype=int)

# Same A05 equations, same residual component order, same solver settings.
def planes(x):
    a, b = x[:N], x[N:]
    nn = radial + a[:, None] * etheta + b[:, None] * ey
    nn = nn / np.linalg.norm(nn, axis=1)[:, None]
    return nn, np.einsum("ij,ij->i", nn, P3)

def corners_of(x):
    nn, dd = planes(x)
    return np.linalg.solve(nn[TRI], dd[TRI][..., None])[..., 0]

def resid(x):
    return np.r_[(corners_of(x) - targets).ravel(), 0.05 * x]

x0 = np.array(data["start"])
# Independently verify native normal-equations LS initialization against A05 SVD.
svd_start = np.zeros(2 * N)
for i in range(N):
    ids = np.flatnonzero(np.any(TRI == i, axis=1))
    V = targets[ids] - P3[i]
    A = np.c_[V @ etheta[i], V @ ey[i]]
    sol, *_ = np.linalg.lstsq(A, -(V @ radial[i]), rcond=None)
    svd_start[i], svd_start[N+i] = sol

fit = least_squares(resid, x0, max_nfev=300)
native = np.array(data["fitted"])
c_native, c_ref = corners_of(native), corners_of(fit.x)
c_start = corners_of(x0)
fans = data["native_geometry"]["polygons"]
aux = [s["auxiliary"] for s in data["sites"]]

def metrics(x, frozen=False):
    cs = corners_of(x)
    edges, angles = [], []
    keep = data["keep"] if frozen else [i for i, ids in enumerate(fans)
        if len(ids) >= 3 and not aux[i] and np.all((cs[ids, 1] >= 0) & (cs[ids, 1] <= 60))]
    for i in keep:
        poly = cs[fans[i]]
        a, b = np.roll(poly, 1, axis=0)-poly, np.roll(poly, -1, axis=0)-poly
        edges.extend(np.linalg.norm(b, axis=1).tolist())
        angles.extend(np.degrees(np.arccos(np.clip(np.einsum('ij,ij->i', a,b)/np.linalg.norm(a,axis=1)/np.linalg.norm(b,axis=1),-1,1))).tolist())
    return {"objective_sum_sq":float(resid(x) @ resid(x)),
            "target_rms":float(np.sqrt(np.mean(np.sum((cs-targets)**2,axis=1)))),
            "whole_tiles":keep,"edge_range":[min(edges,default=None),max(edges,default=None)],
            "unsigned_angle_range":[min(angles,default=None),max(angles,default=None)]}

out = {"identity":data["identity"],"input_sha256":hashlib.sha256(source.read_bytes()).hexdigest(),
       "prototype_sha256":hashlib.sha256(Path(__file__).with_name('A05_depth_proto.py').read_bytes()).hexdigest(),
       "numpy":np.__version__,"scipy":scipy.__version__,
       "start_max_svd_error":float(np.max(np.abs(x0-svd_start))),
       "export_corner_reconstruction_max_error":float(np.max(np.abs(c_native-np.array(data['reference_corners'])))),
       "native":metrics(native),"reference":metrics(fit.x),
       "reference_native_whole_selection":metrics(fit.x, True),
       "solver":{"status":int(fit.status),"nfev":fit.nfev,"message":fit.message,"optimality":float(fit.optimality)},
       "corner_errors":[{"index":k,"triangle":ids.tolist(),"native":a.tolist(),"reference":b.tolist(),
         "coordinate_error":(a-b).tolist(),"error_norm":float(np.linalg.norm(a-b)),
         "start_target_error_sq":float(np.sum((c_start[k]-t)**2)),
         "native_target_error_sq":float(np.sum((a-t)**2)),"reference_target_error_sq":float(np.sum((b-t)**2))}
         for k,(ids,a,b,t) in enumerate(zip(TRI,c_native,c_ref,targets))],
       "reference_fitted":fit.x.tolist(),"reference_corners":c_ref.tolist()}
Path(sys.argv[2]).write_text(json.dumps(out, sort_keys=True)+'\n')
print(json.dumps({k:v for k,v in out.items() if k not in ('corner_errors','reference_fitted','reference_corners')}))
