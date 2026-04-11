"""Generate ring-band OBJ files and oblate spheroid for the solar system scene."""
import math
import os

OUT = os.path.join(os.path.dirname(__file__), "..", "mods", "asteroids", "assets", "3d")
OUT = os.path.normpath(OUT)


def fmt(f):
    return f"{f:.6f}"


def write_v(x, y, z):
    return f"v {fmt(x)} {fmt(y)} {fmt(z)}"


# ── ring half-annulus ────────────────────────────────────────────────────────


def ring_half(inner_r, n_segs, front=True):
    """Return (verts, faces) for a half annulus in XZ plane.
    front=True: theta 0->pi (z-positive arc)
    front=False: theta pi->2pi (z-negative arc)
    """
    verts, faces = [], []
    start = 0.0 if front else math.pi
    end = math.pi if front else 2 * math.pi
    angles = [start + i * (end - start) / n_segs for i in range(n_segs + 1)]
    for theta in angles:
        verts.append((math.cos(theta), 0.0, math.sin(theta)))
        verts.append((inner_r * math.cos(theta), 0.0, inner_r * math.sin(theta)))
    for i in range(n_segs):
        o0, i0 = 2 * i, 2 * i + 1
        o1, i1 = 2 * (i + 1), 2 * (i + 1) + 1
        faces.append((o0 + 1, o1 + 1, i1 + 1, i0 + 1))  # quad, 1-indexed
    return verts, faces


def write_ring_half(name, obj_id, inner_r, n_segs, front):
    path = os.path.join(OUT, name)
    verts, faces = ring_half(inner_r, n_segs, front)
    lines = [
        f"# {name}  inner_r={inner_r:.3f}  n_segs={n_segs}  {'front' if front else 'back'}",
        f"o {obj_id}",
        "",
    ]
    for x, y, z in verts:
        lines.append(write_v(x, y, z))
    lines.append("")
    for a, b, c, d in faces:
        lines.append(f"f {a} {b} {c} {d}")
    lines.append("")
    with open(path, "w", newline="\n") as f:
        f.write("\n".join(lines))
    print(f"  wrote {name}  ({len(verts)} verts, {len(faces)} quads)")


# ── oblate spheroid ───────────────────────────────────────────────────────────


def write_oblate(name, n_lon=24, n_lat=16, flatten_y=0.902):
    """UV sphere with polar axis (y) squished by flatten_y.
    Saturn flattening: f=0.09796 => b/a = 0.902."""
    path = os.path.join(OUT, name)
    verts, faces = [], []
    lat_angles = [-math.pi / 2 + i * math.pi / n_lat for i in range(n_lat + 1)]
    lon_angles = [j * 2 * math.pi / n_lon for j in range(n_lon)]
    for phi in lat_angles:
        for lam in lon_angles:
            x = math.cos(phi) * math.cos(lam)
            y = math.sin(phi) * flatten_y
            z = math.cos(phi) * math.sin(lam)
            verts.append((x, y, z))

    def idx(lat_i, lon_j):
        return lat_i * n_lon + lon_j

    for i in range(n_lat):
        for j in range(n_lon):
            a = idx(i, j)
            b = idx(i, (j + 1) % n_lon)
            c = idx(i + 1, (j + 1) % n_lon)
            d = idx(i + 1, j)
            faces.append((a + 1, b + 1, c + 1))
            faces.append((a + 1, c + 1, d + 1))

    lines = [
        f"# {name} -- oblate spheroid a=1.0 b={flatten_y} (Saturn flattening f~0.098)",
        "o oblate_spheroid",
        "",
    ]
    for x, y, z in verts:
        lines.append(write_v(x, y, z))
    lines.append("")
    for tri in faces:
        lines.append(f"f {tri[0]} {tri[1]} {tri[2]}")
    lines.append("")
    with open(path, "w", newline="\n") as f:
        f.write("\n".join(lines))
    print(f"  wrote {name}  ({len(verts)} verts, {len(faces)} tris)")


# ── main ──────────────────────────────────────────────────────────────────────

N = 24  # segments per half-ring

# Saturn-proportional inner radii (inner_edge / outer_edge for each band)
# Scaled to planet Req=5.8:
#   C  ring: inner=7.17  outer=8.86  -> scale=8.9,  inner_r=7.17/8.86=0.809
#   B  ring: inner=8.86  outer=11.32 -> scale=11.3, inner_r=8.86/11.32=0.783
#   Cassini: inner=11.32 outer=11.76 -> scale=11.8, inner_r=11.32/11.76=0.963
#   A  ring: inner=11.76 outer=13.17 -> scale=13.2, inner_r=11.76/13.17=0.893
rings = [
    ("ring_c", 0.809),
    ("ring_b", 0.783),
    ("ring_cassini", 0.963),
    ("ring_a", 0.893),
]

print("Generating ring OBJ files ...")
for base, inner_r in rings:
    write_ring_half(f"{base}_front.obj", f"{base}_front", inner_r, N, front=True)
    write_ring_half(f"{base}_back.obj", f"{base}_back", inner_r, N, front=False)

print("\nGenerating oblate spheroid ...")
write_oblate("planet_oblate.obj")

print("\nDone.")
