"""Rectified-Voronoi prototype: polygons (one per seed, plane through its seed) + triangles at every
former 3-stone corner. Every corner is shared by 4 pieces (2 polygons + 2 triangles), so corners can
sit on the surface; triangles are always flat.
Env: PROFILE cyl|cone|vase, A (seed spacing, in), D (relief depth, in), JIT (position jitter, frac),
SEED, OUT. Output: stats + JSON of pieces."""
import json
import os

import numpy as np
from scipy.spatial import Delaunay

PROFILE = os.environ.get("PROFILE", "cyl")
A = float(os.environ.get("A", 8.5))
D = float(os.environ.get("D", 0.0))
BASE = float(os.environ.get("BASE", 0.6))
JIT = float(os.environ.get("JIT", 0.0))
SEED = int(os.environ.get("SEED", 1))
T_STONE, MIN_SQ, MIN_ANG, BEVEL_CAP, BLANK = 0.375, 4.0, 30.0, 60.0, 11.5
rng = np.random.default_rng(SEED)
if PROFILE == "vase":
    H = 40.0
    def r_core(y): return 6 + 4 * np.sin(np.pi * y / 40)
    def dr_core(y): return 4 * np.pi / 40 * np.cos(np.pi * y / 40)
elif PROFILE == "cone":
    H = 60.0
    def r_core(y): return 9 - 3 * y / 60
    def dr_core(y): return -3 / 60 + 0 * y
else:
    H = 60.0
    def r_core(y): return 9.0 + 0 * y
    def dr_core(y): return 0 * y
def r_of(y):
    y = np.asarray(y, float)
    r = np.where(y < 0, r_core(0) + dr_core(0) * y, np.where(y > H, r_core(H) + dr_core(H) * (y - H), r_core(np.clip(y, 0, H))))
    return np.maximum(r, 3.0)
def dr_of(y):
    y = np.asarray(y, float)
    return np.where(y < 0, dr_core(0), np.where(y > H, dr_core(H), dr_core(np.clip(y, 0, H))))
YS = np.linspace(-30, H + 30, 24001)
dv = np.sqrt(1 + dr_of(YS) ** 2) / r_of(YS)
V = np.r_[0, np.cumsum((dv[1:] + dv[:-1]) / 2 * np.diff(YS))]
V -= np.interp(0.0, YS, V)
def y_of_v(v): return np.interp(v, V, YS)
def surf(u, v, d):
    y = y_of_v(v); r = r_of(y); rp = dr_of(y)
    nrm = np.stack([np.cos(u), -rp, np.sin(u)], -1) / np.sqrt(1 + rp ** 2)[..., None]
    return np.stack([r * np.cos(u), y, r * np.sin(u)], -1) - np.asarray(d)[..., None] * nrm, nrm

# --- seeds: rows ~A*sqrt(3)/2 apart (physical), per-row count from the local circumference
seeds, rowk = [], []
v = np.interp(-1.6 * A, YS, V); k = 0
while y_of_v(v) < H + 1.6 * A:
    r = float(r_of(y_of_v(v))); n = max(3, int(round(2 * np.pi * r / A)))
    for j in range(n):
        u = (j + 0.5 * (k % 2)) * 2 * np.pi / n
        if JIT:
            u += JIT * rng.uniform(-1, 1) * 2 * np.pi / n
        seeds.append((u % (2 * np.pi), v + (JIT * rng.uniform(-1, 1) * A * 0.866 / r if JIT else 0))); rowk.append(k)
    v += A * np.sqrt(3) / 2 / r; k += 1
uv = np.array(seeds); N = len(uv)
depth = BASE + (rng.uniform(0, D, N) if D > 0 else np.zeros(N))
S, NS = surf(uv[:, 0], uv[:, 1], depth)

# --- periodic Delaunay (integer seam anchor), Voronoi vertices
tiled = np.r_[uv - [2 * np.pi, 0], uv, uv + [2 * np.pi, 0]]
TR = []
for s in Delaunay(tiled).simplices:
    idx = [int(i) for i in s]
    if min(idx, key=lambda i: i % N) // N == 1:
        if len({i % N for i in idx}) == 3:
            TR.append(idx)
def circ(p):
    (ax, ay), (bx, by), (cx, cy) = p
    dd = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    return np.array([((ax**2 + ay**2) * (by - cy) + (bx**2 + by**2) * (cy - ay) + (cx**2 + cy**2) * (ay - by)) / dd,
                     ((ax**2 + ay**2) * (cx - bx) + (bx**2 + by**2) * (ax - cx) + (cx**2 + cy**2) * (bx - ax)) / dd])
CC = np.array([circ(tiled[t]) for t in TR])
TRm = [tuple(i % N for i in t) for t in TR]
edge_tris = {}
for ti, t in enumerate(TRm):
    for a, b in ((t[0], t[1]), (t[1], t[2]), (t[0], t[2])):
        edge_tris.setdefault(tuple(sorted((a, b))), []).append(ti)
def wrap_near(x, ref):
    return x + 2 * np.pi * np.round((ref - x) / (2 * np.pi))

# --- one vertex per Delaunay edge: on line plane_i ∩ plane_j, nearest the Voronoi-edge midpoint
VX, VE, VT = {}, {}, {}
for (i, j), ts in edge_tris.items():
    if len(ts) != 2:
        continue
    c1, c2 = CC[ts[0]].copy(), CC[ts[1]].copy()
    c2[0] = wrap_near(c2[0], c1[0])
    m = (c1 + c2) / 2
    T, _ = surf(np.array(m[0]), np.array(m[1]), np.array((depth[i] + depth[j]) / 2))
    ni, nj = NS[i], NS[j]; ci, cj = ni @ S[i], nj @ S[j]
    g = ni @ nj
    M = np.array([[1, g], [g, 1]]) + 1e-9 * np.eye(2)
    a, b = np.linalg.solve(M, [ci - ni @ T, cj - nj @ T])
    VX[(i, j)] = T + a * ni + b * nj
    VE[(i, j)] = 1 - abs(g)          # small = nearly parallel planes (ill-conditioned)
    VT[(i, j)] = T
def vid(a, b): return tuple(sorted((a, b)))

# --- pieces
pieces = []
nbrs = {}
for (i, j) in VX:
    nbrs.setdefault(i, []).append(j); nbrs.setdefault(j, []).append(i)
for i in range(N):   # polygons
    js = nbrs.get(i, [])
    if len(js) < 3 or len(js) != sum(1 for (a, b) in edge_tris if i in (a, b)):
        continue
    e1 = np.cross(NS[i], [0, 1, 0]); e1 /= np.linalg.norm(e1); e2 = np.cross(NS[i], e1)
    P = np.array([VX[vid(i, j)] for j in js]) - S[i]
    order = np.argsort(np.arctan2(P @ e2, P @ e1))
    pieces.append({"kind": "poly", "seed": i, "keys": [vid(i, js[o]) for o in order]})
for ti, t in enumerate(TRm):   # triangles
    keys = [vid(t[0], t[1]), vid(t[1], t[2]), vid(t[0], t[2])]
    if all(k in VX for k in keys):
        pieces.append({"kind": "tri", "tri": ti, "keys": keys})
for p in pieces:
    C = np.array([VX[k] for k in p["keys"]])
    c0 = C.mean(0)
    if p["kind"] == "poly":
        n = NS[p["seed"]]
    else:
        n = np.cross(C[1] - C[0], C[2] - C[0]); n /= np.linalg.norm(n)
        _, ns = surf(np.array(np.arctan2(c0[2], c0[0])), np.interp(c0[1], YS, V), np.array(0.0))
        if n @ ns < 0:
            n = -n
    e1 = np.cross(n, [0, 1, 0]); e1 /= np.linalg.norm(e1); e2 = np.cross(n, e1)
    q = np.c_[(C - c0) @ e1, (C - c0) @ e2]
    if np.sum(q[:, 0] * np.roll(q[:, 1], -1) - np.roll(q[:, 0], -1) * q[:, 1]) < 0:
        p["keys"] = p["keys"][::-1]; C = C[::-1]; q = q[::-1]
    p["n"] = n; p["d"] = float(n @ C[0]); p["C"] = C; p["q"] = q
# keep pieces touching the band; "inside" = wholly within 0..H (measured)
pieces = [p for p in pieces if p["C"][:, 1].max() > 0 and p["C"][:, 1].min() < H]
for p in pieces:
    p["inside"] = bool(p["C"][:, 1].min() >= 0 and p["C"][:, 1].max() <= H)

# --- edge adjacency (outer edge -> two pieces)
edge_owner = {}
for pi, p in enumerate(pieces):
    ks = p["keys"]
    for a in range(len(ks)):
        edge_owner.setdefault(tuple(sorted((ks[a], ks[(a + 1) % len(ks)]))), []).append((pi, a))


# --- optimiser: 2 tilts per polygon plane (still through its seed) + 1 slide per corner along its line
if int(os.environ.get("NFEV", 0)) > 0:
    from scipy.optimize import least_squares
    VK = list(VX.keys()); VI = {k: n for n, k in enumerate(VK)}; NE = len(VK)
    I = np.array([k[0] for k in VK]); J = np.array([k[1] for k in VK]); TT = np.array([VT[k] for k in VK])
    yv = y_of_v(uv[:, 1]); rps = dr_of(yv)
    E1 = np.c_[-np.sin(uv[:, 0]), np.zeros(N), np.cos(uv[:, 0])]
    E2 = np.c_[rps * np.cos(uv[:, 0]), np.ones(N), rps * np.sin(uv[:, 0])]; E2 /= np.linalg.norm(E2, axis=1)[:, None]
    def verts(x):
        n = NS + x[:N, None] * E1 + x[N:2 * N, None] * E2; n /= np.linalg.norm(n, axis=1)[:, None]
        c = np.einsum("ij,ij->i", n, S)
        ni, nj = n[I], n[J]; g = np.einsum("ij,ij->i", ni, nj); det = 1 - g * g + 1e-6
        r1 = c[I] - np.einsum("ij,ij->i", ni, TT); r2 = c[J] - np.einsum("ij,ij->i", nj, TT)
        al = (r1 - g * r2) / det; be = (r2 - g * r1) / det
        X = TT + al[:, None] * ni + be[:, None] * nj
        w = np.cross(ni, nj); w /= np.linalg.norm(w, axis=1)[:, None] + 1e-12
        return X + x[2 * N:, None] * w, n
    M = [p for p in pieces if 0 <= p["C"][:, 1].mean() <= H]
    for p in M:
        p["ix"] = np.array([VI[k] for k in p["keys"]])
        if p["kind"] == "tri":
            p["sgn"] = 1.0
    shared = []
    for key, own in edge_owner.items():
        if len(own) == 2 and all(0 <= pieces[o[0]]["C"][:, 1].mean() <= H for o in own):
            shared.append((pieces[own[0][0]], pieces[own[1][0]]))
    LO_B, HI_B = -(BASE + D + 0.4), 0.75
    def pnorm(p, X, n):
        if p["kind"] == "poly":
            return n[p["seed"]]
        C = X[p["ix"]]; m = np.cross(C[1] - C[0], C[2] - C[0]); return m / (np.linalg.norm(m) + 1e-12)
    def resid(x):
        X, n = verts(x)
        r = [0.05 * (X - TT).ravel(), 0.05 * x[:2 * N], 0.02 * x[2 * N:]]
        for p in M:
            C = X[p["ix"]]; m = len(C); nn = pnorm(p, X, n)
            e1 = np.cross(nn, [0, 1, 0]); e1 /= np.linalg.norm(e1); e2 = np.cross(nn, e1)
            q = np.c_[C @ e1, C @ e2]
            L = np.linalg.norm(np.roll(C, -1, 0) - C, axis=1)
            r.append(5 * np.maximum(0, (4.1 if p["kind"] == "tri" else 1.1) - L))
            a = np.roll(q, 1, 0) - q; b = np.roll(q, -1, 0) - q
            cr = a[:, 0] * b[:, 1] - a[:, 1] * b[:, 0]; dt = (a * b).sum(1)
            ang = np.degrees(np.arctan2(np.abs(cr), dt)); ang = np.where(cr <= 0, ang, 360 - ang)
            r.append(0.5 * np.maximum(0, 33 - ang)); r.append(0.5 * np.maximum(0, ang - 227))
            dia = max(np.linalg.norm(C[u] - C[v]) for u in range(m) for v in range(u + 1, m))
            r.append([5 * max(0, dia - 11.3)])
            off_ = (np.hypot(C[:, 0], C[:, 2]) - r_of(C[:, 1])) / np.sqrt(1 + dr_of(C[:, 1]) ** 2)
            r.append(5 * np.maximum(0, off_ - HI_B)); r.append(5 * np.maximum(0, LO_B - off_))
        bv = []
        for pa, pb in shared:
            g = np.clip(pnorm(pa, X, n) @ pnorm(pb, X, n), -1, 1); bv.append(max(0, np.degrees(np.arccos(g)) / 2 - 57))
        r.append(0.5 * np.array(bv))
        return np.concatenate([np.atleast_1d(np.asarray(v, float)) for v in r])
    x0 = np.zeros(2 * N + NE)
    print("  opt start cost", round(float(np.sum(resid(x0) ** 2)), 3))
    from scipy.sparse import lil_matrix
    NV_ = 2 * N + NE
    vvars = [{I[e], N + I[e], J[e], N + J[e], 2 * N + e} for e in range(NE)]
    rows = []
    for e in range(NE):
        rows += [vvars[e]] * 3
    rows += [{v} for v in range(2 * N)] + [{2 * N + e} for e in range(NE)]
    def pvars(p):
        sv = set().union(*[vvars[e] for e in p["ix"]])
        return sv | ({p["seed"], N + p["seed"]} if p["kind"] == "poly" else set())
    for p in M:
        rows += [pvars(p)] * (5 * len(p["ix"]) + 1)
    for pa, pb in shared:
        rows.append(pvars(pa) | pvars(pb))
    assert len(rows) == len(resid(x0)), (len(rows), len(resid(x0)))
    SP = lil_matrix((len(rows), NV_), dtype=int)
    for r_, vs in enumerate(rows):
        for v in vs:
            SP[r_, v] = 1
    sol = least_squares(resid, x0, max_nfev=int(os.environ["NFEV"]), x_scale="jac", jac_sparsity=SP)
    print("  opt end cost", round(float(np.sum(sol.fun ** 2)), 3), "nfev", sol.nfev)
    X, n = verts(sol.x)
    for k in VK:
        VX[k] = X[VI[k]]
    NS = n
    for p in pieces:
        C = np.array([VX[k] for k in p["keys"]]); c0 = C.mean(0)
        nn = NS[p["seed"]] if p["kind"] == "poly" else None
        if nn is None:
            nn = np.cross(C[1] - C[0], C[2] - C[0]); nn /= np.linalg.norm(nn)
        e1 = np.cross(nn, [0, 1, 0]); e1 /= np.linalg.norm(e1); e2 = np.cross(nn, e1)
        p["n"] = nn; p["d"] = float(nn @ C[0]); p["C"] = C; p["q"] = np.c_[(C - c0) @ e1, (C - c0) @ e2]
        p["inside"] = bool(C[:, 1].min() >= 0 and C[:, 1].max() <= H)

def bsq(q):
    best = 1e9
    for t in np.radians(np.arange(0, 90, 0.5)):
        R = np.array([[np.cos(t), -np.sin(t)], [np.sin(t), np.cos(t)]]); z = q @ R
        best = min(best, max(np.ptp(z[:, 0]), np.ptp(z[:, 1])))
    return best
stats = {"poly": {"n": 0, "sides": {}}, "tri": {"n": 0}}
ang_min = {"poly": 999, "tri": 999}; sq = {"poly": [], "tri": []}; edges = {"poly": [], "tri": []}
planar = 0.0; bevels = []; refl = 0
for p in pieces:
    if not p["inside"]:
        continue
    k = p["kind"]; stats[k]["n"] += 1
    if k == "poly":
        stats[k]["sides"][len(p["keys"])] = stats[k]["sides"].get(len(p["keys"]), 0) + 1
    q = p["q"]; m = len(q)
    planar = max(planar, float(np.abs(p["C"] @ p["n"] - p["d"]).max()))
    for a in range(m):
        v1, v2 = q[a - 1] - q[a], q[(a + 1) % m] - q[a]
        an = np.degrees(np.arctan2(abs(v1[0] * v2[1] - v1[1] * v2[0]), v1 @ v2))
        cr = (q[a] - q[a - 1])[0] * (q[(a + 1) % m] - q[a])[1] - (q[a] - q[a - 1])[1] * (q[(a + 1) % m] - q[a])[0]
        if cr < 0:
            an = 360 - an; refl += 1
        ang_min[k] = min(ang_min[k], an)
        edges[k].append(np.linalg.norm(p["C"][a] - p["C"][(a + 1) % m]))
    sq[k].append(bsq(q))
for key, own in edge_owner.items():
    if len(own) == 2 and pieces[own[0][0]]["inside"] and pieces[own[1][0]]["inside"]:
        g = np.clip(pieces[own[0][0]]["n"] @ pieces[own[1][0]]["n"], -1, 1)
        bevels.append(np.degrees(np.arccos(g)) / 2)

# --- back-face (inner) mismatch at 4-piece corners: compare the two inner edges of each joint
def side_plane(pa, pb):
    n = pa["n"] - pb["n"]; L = np.linalg.norm(n)
    return n / L, (pa["d"] - pb["d"]) / L
def inner_corners(pi):
    p = pieces[pi]; ks = p["keys"]; m = len(ks); sides = []
    for a in range(m):
        own = edge_owner[tuple(sorted((ks[a], ks[(a + 1) % m])))]
        other = [o for o in own if o[0] != pi]
        if not other:
            return None
        sides.append(side_plane(p, pieces[other[0][0]]))
    out = []
    for a in range(m):
        sp, sn = sides[a - 1], sides[a]
        M = np.array([p["n"], sp[0], sn[0]])
        if abs(np.linalg.det(M)) < 1e-9:
            return None
        out.append(np.linalg.solve(M, [p["d"] - T_STONE, sp[1], sn[1]]))
    return np.array(out)
IC = {pi: inner_corners(pi) for pi, p in enumerate(pieces) if p["inside"]}
mism = []
for key, own in edge_owner.items():
    if len(own) != 2:
        continue
    (pa, a), (pb, b) = own
    if IC.get(pa) is None or IC.get(pb) is None:
        continue
    ma, mb = len(pieces[pa]["keys"]), len(pieces[pb]["keys"])
    A2 = np.array([IC[pa][a], IC[pa][(a + 1) % ma]]); B2 = np.array([IC[pb][b], IC[pb][(b + 1) % mb]])
    mism.append(min(np.linalg.norm(A2 - B2, axis=1).max(), np.linalg.norm(A2 - B2[::-1], axis=1).max()))
allC = np.array([VX[k] for k in VX])
allC = allC[(allC[:, 1] >= 0) & (allC[:, 1] <= H)]
off = (np.hypot(allC[:, 0], allC[:, 2]) - r_of(allC[:, 1])) / np.sqrt(1 + dr_of(allC[:, 1]) ** 2)
bad = (sum(e < 4.0 - 1e-6 for e in edges["tri"]) + sum(e < 1.0 for e in edges["poly"]) + sum(x > BLANK for x in sq["poly"] + sq["tri"])
       + (ang_min["poly"] < MIN_ANG) + (ang_min["tri"] < MIN_ANG) + sum(b > BEVEL_CAP for b in bevels))
print(f"{PROFILE} A={A} D={D} jit={JIT}: polygons {stats['poly']['n']} {stats['poly']['sides']}, triangles {stats['tri']['n']}")
for k in ("poly", "tri"):
    print(f"  {k}: size(min square) {min(sq[k]):.2f}-{max(sq[k]):.2f} (over blank: {sum(x > BLANK for x in sq[k])}); edges {min(edges[k]):.2f}-{max(edges[k]):.2f} (short: {sum(e < (4.0 - 1e-6 if k == "tri" else 1.0) for e in edges[k])}); min angle {ang_min[k]:.0f}")
print(f"  taper(bevel) max {max(bevels):.1f} deg; reflex corners {refl}; planarity err {planar:.1e} in")
print(f"  corners vs surface {off.min():+.2f}..{off.max():+.2f} in; back-face joint mismatch median {np.median(mism):.3f} max {max(mism):.3f} in")
print("  RESULT", "PASS" if bad == 0 else f"FAIL ({bad})")
json.dump({"profile": PROFILE, "A": A, "D": D, "pieces": [{"kind": p["kind"], "normal": p["n"].tolist(), "corners": p["C"].tolist()} for p in pieces]},
          open(os.environ.get("OUT", f"tri_{PROFILE}.json"), "w"))
