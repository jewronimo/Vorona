"""A05 reference prototype (NOT production code): points in a depth space under the surface ->
one plane per point (through the point) -> corners where three planes meet.

Cylinder: outer surface radius R, height H. Depth space = radii [R - D, R]; D from env (default 2).
Usage: D=2 python3 A05_depth_proto.py <symmetric|random> [seed]
Known limits: crude fit (corner-to-Voronoi-target least squares, no barrier terms), so the random
mode gives poor pieces. C4 must do this properly (see amendment A05 §4).
Results 2026-11-11, symmetric: D=1 edges 2.65-4.82 in, angles >= 69 deg; D=2 angles >= 80 deg,
bevels 14-31 deg, 63/90 pieces with one reflex corner (<= 220 deg).
"""
import json
import sys

import numpy as np
from scipy.spatial import Delaunay

import os
R, H, D = 9.0, 60.0, float(os.environ.get("D", 2.0))
TARGET = 6.5
MIN_EDGE, MAX_EDGE, MIN_ANGLE, BEVEL_CAP = 1.0, 6.0, 70.0, 60.0
USABLE, MIN_SQUARE = 11.5, 4.0
mode = sys.argv[1] if len(sys.argv) > 1 else "symmetric"
seed = int(sys.argv[2]) if len(sys.argv) > 2 else 1
rng = np.random.default_rng(seed)

C = 2 * np.pi * R
n = int(round(C / TARGET))            # columns around
w = C / n                             # column spacing (arc length)
h = w * np.sqrt(3) / 2                # row spacing of a hex lattice
rows = int(np.ceil(H / h)) + 3        # extra rows beyond both ends, trimmed later

pts2, depth = [], []
for k in range(-2, rows):
    for j in range(n):
        u = j * w + (w / 2 if k % 2 else 0.0)
        y = k * h
        if mode == "symmetric":
            d = [0.0, D / 2, D][k % 3]          # sawtooth: every point's upper and lower neighbours differ
        else:
            u += rng.uniform(-0.25, 0.25) * w
            y += rng.uniform(-0.25, 0.25) * h
            d = rng.uniform(0.0, D)
        pts2.append((u % C, y))
        depth.append(d)
pts2, depth = np.array(pts2), np.array(depth)
N = len(pts2)
rad = R - depth
th = pts2[:, 0] / R
P3 = np.c_[rad * np.cos(th), pts2[:, 1], rad * np.sin(th)]

# neighbours: Delaunay of the unrolled points with periodic copies
tiled = np.r_[pts2 - [C, 0], pts2, pts2 + [C, 0]]
tri = Delaunay(tiled)
triangles = set()
for s in tri.simplices:
    cu = tiled[s, 0].mean()
    if 0 <= cu < C:
        t = tuple(sorted(int(i) % N for i in s))
        if len(set(t)) == 3:
            triangles.add(t)
triangles = sorted(triangles)
nbrs = [set() for _ in range(N)]
for a, b, c in triangles:
    nbrs[a] |= {b, c}; nbrs[b] |= {a, c}; nbrs[c] |= {a, b}

# Voronoi idea: each corner's target is the point equidistant from its three neighbouring points
# (circumcentre in the unrolled surface), at the mean depth of those three points.
TRI = np.array(triangles)
targets = []
for t in triangles:
    q = []
    for i in t:
        u, y = pts2[i]
        u += C * round((pts2[t[0], 0] - u) / C)   # unwrap across the seam
        q.append((u, y))
    (ax, ay), (bx, by), (cx, cy) = q
    dd = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    ux = ((ax**2 + ay**2) * (by - cy) + (bx**2 + by**2) * (cy - ay) + (cx**2 + cy**2) * (ay - by)) / dd
    uy = ((ax**2 + ay**2) * (cx - bx) + (bx**2 + by**2) * (ax - cx) + (cx**2 + cy**2) * (bx - ax)) / dd
    rr = R - depth[list(t)].mean()
    targets.append([rr * np.cos(ux / R), uy, rr * np.sin(ux / R)])
targets = np.array(targets)

radial = np.c_[np.cos(th), np.zeros(N), np.sin(th)]
etheta = np.c_[-np.sin(th), np.zeros(N), np.cos(th)]
ey = np.tile([0.0, 1.0, 0.0], (N, 1))

def planes(x):
    a, b = x[:N], x[N:]
    nn = radial + a[:, None] * etheta + b[:, None] * ey
    nn = nn / np.linalg.norm(nn, axis=1)[:, None]
    return nn, np.einsum("ij,ij->i", nn, P3)          # plane through its own point: n . x = n . p

def corners_of(x):
    nn, dd = planes(x)
    return np.linalg.solve(nn[TRI], dd[TRI][..., None])[..., 0]

# start: each plane through its point, best fit to its own target corners (linear in the two tilts)
x0 = np.zeros(2 * N)
tri_idx = [[] for _ in range(N)]
for k, t in enumerate(triangles):
    for i in t:
        tri_idx[i].append(k)
for i in range(N):
    V = targets[tri_idx[i]] - P3[i]
    A = np.c_[V @ etheta[i], V @ ey[i]]
    sol, *_ = np.linalg.lstsq(A, -(V @ radial[i]), rcond=None)
    x0[i], x0[N + i] = sol

from scipy.optimize import least_squares
def resid(x):
    return np.r_[(corners_of(x) - targets).ravel(), 0.05 * x]
fit = least_squares(resid, x0, max_nfev=300)
normals, offsets = planes(fit.x)
CORN = corners_of(fit.x)
corner = {t: c for t, c in zip(triangles, CORN)}
dets = [abs(np.linalg.det(normals[list(t)])) for t in triangles]
rms = lambda x: np.sqrt(np.mean(np.sum((corners_of(x) - targets) ** 2, axis=1)))
print(f"  corner-to-Voronoi-target rms: start {rms(x0):.3f} in, after fit {rms(fit.x):.3f} in")

# polygon per point: its triangles' corners in angular order around the point (in its plane)
tri_of = [[] for _ in range(N)]
for t in triangles:
    for i in t:
        tri_of[i].append(t)

def polygon(i):
    """Corners of point i's polygon in fan order (walk the triangles around i), counter-clockwise seen from outside."""
    ts = tri_of[i]
    if len(ts) < 3:
        return [], [], None
    nxt = {}
    for t in ts:
        a, b = [v for v in t if v != i]
        nxt.setdefault(a, []).append((b, t)); nxt.setdefault(b, []).append((a, t))
    if any(len(v) != 2 for v in nxt.values()):
        return [], [], None                    # open fan: point on the border of the triangulated region
    start = next(iter(nxt)); prev, cur, order = None, start, []
    for _ in range(len(ts)):
        (n1, t1), (n2, t2) = nxt[cur]
        nb, tt = (n1, t1) if n1 != prev and (prev is not None or True) and (t1 not in order) else (n2, t2)
        order.append(tt); prev, cur = cur, nb
    nrm = normals[i]
    e1 = np.cross(nrm, [0.0, 1.0, 0.0]); e1 /= np.linalg.norm(e1)
    e2 = np.cross(nrm, e1)
    cs = [corner[t] for t in order]
    p2 = np.array([[(c - P3[i]) @ e1, (c - P3[i]) @ e2] for c in cs])
    area = 0.5 * sum(p2[k, 0] * p2[(k + 1) % len(p2), 1] - p2[(k + 1) % len(p2), 0] * p2[k, 1] for k in range(len(p2)))
    if area < 0:
        cs, order = cs[::-1], order[::-1]
    return cs, order, (e1, e2)

# keep whole tiles inside 0..H (rim style "whole"), complete fans only
keep = []
for i in range(N):
    poly, ts, _ = polygon(i)
    if len(ts) < 3:
        continue
    ys = [c[1] for c in poly]
    if min(ys) >= 0.0 and max(ys) <= H:
        keep.append(i)
keep_set = set(keep)

def interior_angles(poly2):
    m = len(poly2); out = []
    for k in range(m):
        a, b, c = poly2[k - 1], poly2[k], poly2[(k + 1) % m]
        v1, v2 = a - b, c - b
        out.append(np.degrees(np.arccos(np.clip(v1 @ v2 / np.linalg.norm(v1) / np.linalg.norm(v2), -1, 1))))
    return out

def is_convex(poly2):
    m = len(poly2); s = []
    for k in range(m):
        a, b, c = poly2[k], poly2[(k + 1) % m], poly2[(k + 2) % m]
        s.append(np.sign((b - a)[0] * (c - b)[1] - (b - a)[1] * (c - b)[0]))
    return all(x > 0 for x in s) or all(x < 0 for x in s)

def min_square(poly2):
    best = (np.inf, 0)
    for a in np.radians(np.arange(0, 90, 1.0)):
        Rm = np.array([[np.cos(a), -np.sin(a)], [np.sin(a), np.cos(a)]])
        q = poly2 @ Rm.T
        side = max(np.ptp(q[:, 0]), np.ptp(q[:, 1]))
        best = min(best, (side, a))
    return best[0]

tiles, stats = [], {"sides": {}, "edges": [], "angles": [], "bevels": [], "stick": [], "convex_fail": 0,
                     "too_big": 0, "too_small": 0, "edge_fail": 0, "angle_fail": 0}
for i in keep:
    poly, ts, (e1, e2) = polygon(i)
    p2 = np.array([[(c - P3[i]) @ e1, (c - P3[i]) @ e2] for c in poly])
    m = len(poly)
    stats["sides"][m] = stats["sides"].get(m, 0) + 1
    edges = [np.linalg.norm(poly[k] - poly[(k + 1) % m]) for k in range(m)]
    angs = interior_angles(p2)
    conv = is_convex(p2)
    sq = min_square(p2)
    stats["edges"] += edges; stats["angles"] += angs
    stats["convex_fail"] += (not conv)
    stats["edge_fail"] += any(e < MIN_EDGE or e > MAX_EDGE for e in edges)
    stats["angle_fail"] += any(a < MIN_ANGLE for a in angs)
    stats["too_big"] += sq > USABLE
    stats["too_small"] += sq < MIN_SQUARE
    for c in poly:
        stats["stick"].append(np.hypot(c[0], c[2]) - R)       # + = outside the drawn surface
    for j in nbrs[i]:
        if j in keep_set and j > i:
            ang = np.degrees(np.arccos(np.clip(normals[i] @ normals[j], -1, 1)))
            stats["bevels"].append(ang / 2)
    tiles.append({"id": i, "normal": normals[i].tolist(), "offset": float(offsets[i]),
                  "corners": [c.tolist() for c in poly]})

def rng_s(a):
    a = np.array(a); return f"{a.min():.2f} .. {a.max():.2f} (median {np.median(a):.2f})"

print(f"mode {mode} seed {seed}: columns {n}, spacing {w:.2f} in, rows spacing {h:.2f} in, depth space {D} in")
print(f"  whole tiles kept: {len(keep)}   side counts: {dict(sorted(stats['sides'].items()))}")
print(f"  smallest |det| of a corner's 3 planes: {min(dets):.2e}")
print(f"  edges in:   {rng_s(stats['edges'])}   tiles with an edge outside [{MIN_EDGE},{MAX_EDGE}]: {stats['edge_fail']}")
print(f"  corner angles deg: {rng_s(stats['angles'])}   tiles with an angle < {MIN_ANGLE}: {stats['angle_fail']}")
print(f"  bevels deg: {rng_s(stats['bevels'])}   over cap {BEVEL_CAP}: {sum(b > BEVEL_CAP for b in stats['bevels'])}")
print(f"  corners vs drawn surface in (+ outside): {rng_s(stats['stick'])}")
print(f"  non-convex tiles: {stats['convex_fail']}   bigger than {USABLE} square: {stats['too_big']}   smaller than {MIN_SQUARE}: {stats['too_small']}")
json.dump({"R": R, "H": H, "D": D, "mode": mode, "seed": seed,
           "tiles": [{"id": t["id"], "normal": t["normal"], "offset": t["offset"], "corners": t["corners"]} for t in tiles]},
          open(f"tiles_{mode}_{seed}.json", "w"))

# angle defect at each corner shared by 3 kept tiles: 360 - sum of the 3 face angles there
face_angle = {}
for i in keep:
    poly, ts, _ = polygon(i)
    m = len(poly)
    for k in range(m):
        a, b, c = poly[k - 1], poly[k], poly[(k + 1) % m]
        v1, v2 = a - b, c - b
        ang = np.degrees(np.arccos(np.clip(v1 @ v2 / np.linalg.norm(v1) / np.linalg.norm(v2), -1, 1)))
        # reflex test in the tile's plane
        e1, e2 = polygon(i)[2]
        cr = ((a - b) @ e1) * ((c - b) @ e2) - ((a - b) @ e2) * ((c - b) @ e1)
        face_angle.setdefault(ts[k], []).append(ang if cr < 0 else 360 - ang)
defects = [360 - sum(v) for v in face_angle.values() if len(v) == 3]
d = np.array(defects)
print(f"  corner angle defects deg: {d.min():.1f} .. {d.max():.1f}; positive {np.sum(d > 0.5)}, ~zero {np.sum(abs(d) <= 0.5)}, negative {np.sum(d < -0.5)}")
