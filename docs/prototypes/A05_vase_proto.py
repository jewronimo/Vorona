# Reference prototype by Claude (2026-11-12), NOT production code. Usage examples:
#   N=8 D=2.5 OPT=1 python3 A05_vase_proto.py   -> vase, 8 columns, all limits pass after the optimiser
#   N=10 D=2.0 OPT=1 python3 A05_vase_proto.py  -> vase, 10 columns, all limits pass
#   PROFILE=cyl N=9 D=1.5 python3 A05_vase_proto.py
"""Depth-space prototype on a surface of revolution (vase r = 6 + 4 sin(pi y / 40), 0..40 in).
Conformal hex lattice of points, row depth pattern, one plane per point through the point,
fit to Voronoi targets, then a penalty optimiser for the limits.
Env: D (depth), N (columns), STRETCH (row spacing factor), OPT (1 = run optimiser), NFEV, PROFILE (vase|cyl|sphere)."""
import os
import numpy as np
from scipy.optimize import least_squares
from scipy.spatial import Delaunay

D = float(os.environ.get("D", 1.5)); NCOL = int(os.environ.get("N", 8)); STRETCH = float(os.environ.get("STRETCH", 1.0))
PROFILE = os.environ.get("PROFILE", "vase")
MIN_EDGE, MAX_EDGE, MIN_ANG, MAX_REF, BEVEL = 1.0, 6.0, 70.0, 230.0, 60.0
if PROFILE == "vase":
    H = 40.0
    def r_core(y): return 6 + 4 * np.sin(np.pi * y / 40)
    def dr_core(y): return 4 * np.pi / 40 * np.cos(np.pi * y / 40)
elif PROFILE == "cyl":
    H = 60.0
    def r_core(y): return 9.0 + 0 * y
    def dr_core(y): return 0 * y
else:  # sphere band (radius 10), y from -8 .. 8 shifted to 0..16
    H = 16.0
    def r_core(y): return np.sqrt(100 - (y - 8) ** 2)
    def dr_core(y): return -(y - 8) / np.sqrt(100 - (y - 8) ** 2)
def r_of(y):    # tangent (cone) extension beyond the ends
    y = np.asarray(y, float)
    return np.where(y < 0, r_core(0) + dr_core(0) * y, np.where(y > H, r_core(H) + dr_core(H) * (y - H), r_core(np.clip(y, 0, H))))
def dr_of(y):
    y = np.asarray(y, float)
    return np.where(y < 0, dr_core(0), np.where(y > H, dr_core(H), dr_core(np.clip(y, 0, H))))

# conformal coordinate v(y) = int sqrt(1 + r'^2) / r dy, tabulated
YS = np.linspace(-25, H + 25, 20001)
dv = np.sqrt(1 + dr_of(YS) ** 2) / r_of(YS)
V = np.r_[0, np.cumsum((dv[1:] + dv[:-1]) / 2 * np.diff(YS))]
V -= np.interp(0.0, YS, V)
def y_of_v(v): return np.interp(v, V, YS)

du = 2 * np.pi / NCOL
dvrow = du * np.sqrt(3) / 2 * STRETCH
v0, v1 = np.interp(0.0, YS, V), np.interp(H, YS, V)
ks = range(int(np.floor(v0 / dvrow)) - 3, int(np.ceil(v1 / dvrow)) + 4)
uv, depth, rowk = [], [], []
for k in ks:
    for j in range(NCOL):
        uv.append((j * du + (du / 2 if k % 2 else 0), k * dvrow)); depth.append([0, D / 2, D][k % 3]); rowk.append(k)
uv, depth = np.array(uv), np.array(depth)
N = len(uv)

def surf(u, v, d):
    y = y_of_v(v); r = r_of(y); rp = dr_of(y)
    nrm = np.stack([np.cos(u), -rp, np.sin(u)], -1) / np.sqrt(1 + rp ** 2)[..., None]   # outward normal
    pt = np.stack([r * np.cos(u), y, r * np.sin(u)], -1)
    return pt - np.asarray(d)[..., None] * nrm, nrm

P3, NRM = surf(uv[:, 0], uv[:, 1], depth)
tiled = np.r_[uv - [2 * np.pi, 0], uv, uv + [2 * np.pi, 0]]
tri = Delaunay(tiled)
T = set()
for s in tri.simplices:
    if 0 <= tiled[s, 0].mean() < 2 * np.pi:
        t = tuple(sorted(int(i) % N for i in s))
        if len(set(t)) == 3:
            T.add(t)
T = np.array(sorted(T))
# Voronoi targets: circumcentre in (u, v), at the mean depth of the three points
tg = []
for t in T:
    q = uv[list(t)].copy()
    q[:, 0] += 2 * np.pi * np.round((q[0, 0] - q[:, 0]) / (2 * np.pi))
    (ax, ay), (bx, by), (cx, cy) = q
    dd = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    ux = ((ax**2 + ay**2) * (by - cy) + (bx**2 + by**2) * (cy - ay) + (cx**2 + cy**2) * (ay - by)) / dd
    uy = ((ax**2 + ay**2) * (cx - bx) + (bx**2 + by**2) * (ax - cx) + (cx**2 + cy**2) * (bx - ax)) / dd
    tg.append(surf(np.array(ux), np.array(uy), np.array(depth[list(t)].mean()))[0])
TG = np.array(tg)
# tangent frame per point
yv = y_of_v(uv[:, 1]); rp = dr_of(yv)
E1 = np.c_[-np.sin(uv[:, 0]), np.zeros(N), np.cos(uv[:, 0])]
E2 = np.c_[rp * np.cos(uv[:, 0]), np.ones(N), rp * np.sin(uv[:, 0])]; E2 /= np.linalg.norm(E2, axis=1)[:, None]

def planes(x):
    nn = NRM + x[:N, None] * E1 + x[N:, None] * E2
    nn = nn / np.linalg.norm(nn, axis=1)[:, None]
    return nn, np.einsum("ij,ij->i", nn, P3)
def corners(x):
    nn, dd = planes(x)
    return np.linalg.solve(nn[T], dd[T][..., None])[..., 0]

tri_idx = [[] for _ in range(N)]
for k, t in enumerate(T):
    for i in t:
        tri_idx[i].append(k)
x0 = np.zeros(2 * N)
for i in range(N):
    Vv = TG[tri_idx[i]] - P3[i]
    A = np.c_[Vv @ E1[i], Vv @ E2[i]]
    sol, *_ = np.linalg.lstsq(A, -(Vv @ NRM[i]), rcond=None)
    x0[i], x0[N + i] = sol
fit = least_squares(lambda x: np.r_[(corners(x) - TG).ravel(), 0.05 * x], x0, max_nfev=300)
xfit = fit.x

# whole tiles: complete fan, all corners within 0..H; fan order by angle in the tangent frame
def fan(i):
    ks_ = tri_idx[i]
    nb = {}
    for k in ks_:
        a, b = [v for v in T[k] if v != i]
        nb.setdefault(a, []).append(k); nb.setdefault(b, []).append(k)
    if len(ks_) < 3 or any(len(v) != 2 for v in nb.values()):
        return None
    X = corners(xfit)[ks_] - P3[i]
    ang = np.arctan2(X @ E2[i], X @ E1[i])
    return [ks_[j] for j in np.argsort(ang)]
whole = []
Xf = corners(xfit)
for i in range(N):
    f = fan(i)
    if f and Xf[f][:, 1].min() >= 0 and Xf[f][:, 1].max() <= H:
        whole.append((i, f))
PT, PC, PP, PN, EA = [], [], [], [], set()
for i, f in whole:
    m = len(f)
    for k in range(m):
        PT.append(i); PC.append(f[k]); PP.append(f[k - 1]); PN.append(f[(k + 1) % m])
        EA.add(tuple(sorted((f[k], f[(k + 1) % m]))))
PT, PC, PP, PN = map(np.array, (PT, PC, PP, PN)); EA = np.array(sorted(EA))
NB = set()
for k, t in enumerate(T):
    for a, b in ((t[0], t[1]), (t[1], t[2]), (t[0], t[2])):
        NB.add((a, b))
NBa = np.array(sorted(NB))
wset = set(i for i, _ in whole)
NBw = np.array([p for p in NBa if p[0] in wset and p[1] in wset])

def measure(x):
    X = corners(x); nn, _ = planes(x)
    L = np.linalg.norm(X[EA[:, 0]] - X[EA[:, 1]], axis=1)
    a = X[PP] - X[PC]; b = X[PN] - X[PC]
    cr = np.einsum("ij,ij->i", np.cross(b, a), nn[PT]); dt = np.einsum("ij,ij->i", a, b)
    ang = np.degrees(np.arctan2(np.abs(cr), dt)); ang = np.where(cr >= 0, ang, 360 - ang)
    bev = np.degrees(np.arccos(np.clip(np.einsum("ij,ij->i", nn[NBw[:, 0]], nn[NBw[:, 1]]), -1, 1))) / 2
    return X, L, ang, bev
X, L, A, B = measure(xfit)
if np.median(A) > 180:   # orientation convention
    FLIP = True
else:
    FLIP = False
def meas(x):
    X, L, A, B = measure(x)
    return X, L, (360 - A) if FLIP else A, B
def summary(x, label):
    X, L, A, B = meas(x)
    conv, refl = A[A <= 180], A[A > 180]
    bad_t = set()
    for k, (i, f) in enumerate(whole):
        pass
    be = np.sum((L < MIN_EDGE) | (L > MAX_EDGE)); ba = np.sum(conv < MIN_ANG) + np.sum(refl > MAX_REF)
    dist = np.hypot(X[:, 0], X[:, 2]) - r_of(X[:, 1])
    print(f"  {label}: tiles {len(whole)}; edges {L.min():.2f}..{L.max():.2f} (bad {be}); convex >= {conv.min():.1f}; "
          f"reflex <= {refl.max() if len(refl) else 0:.1f} (bad corners {ba}); bevel <= {B.max():.1f}")
    return be + ba
print(f"{PROFILE}: columns {NCOL}, depth {D}, row stretch {STRETCH}, points {N}")
bad = summary(xfit, "fit")
if os.environ.get("OPT") == "1" and bad:
    def resid(x):
        X, L, A, B = meas(x)
        conv = np.where(A <= 180, np.maximum(0, MIN_ANG + 3 - A), 0) / 10
        refl = np.where(A > 180, np.maximum(0, A - MAX_REF + 3), 0) / 10
        return np.r_[0.05 * (X - TG).ravel(), 0.05 * x, 5 * np.maximum(0, MIN_EDGE + 0.2 - L),
                     5 * np.maximum(0, L - MAX_EDGE + 0.2), 5 * conv, 5 * refl]
    sol = least_squares(resid, xfit, max_nfev=int(os.environ.get("NFEV", 60)), x_scale="jac")
    summary(sol.x, "optimiser")

if os.environ.get("DUMP"):
    import json
    xf = sol.x if (os.environ.get("OPT") == "1" and bad) else xfit
    Xd = corners(xf); nn, _ = planes(xf)
    tiles = [{"id": int(i), "normal": nn[i].tolist(), "corners": Xd[f].tolist()} for i, f in whole]
    json.dump({"tiles": tiles}, open(f"tiles_{os.environ['DUMP']}.json", "w"))
    print("  dumped", len(tiles), "tiles")
