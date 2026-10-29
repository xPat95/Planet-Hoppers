"""Generador del planeta Smiling Friends (v01, blockout con identidad).

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/smilingfriends/build_smilingfriends_planet.py

Crea la escena desde cero, guarda assets/models/planets/smilingfriends/smilingfriends_planet.blend
y renderiza capturas en docs/references/smilingfriends/. Diseño: docs/cami-guidelines.md, sección 1.

La distribución sigue el boceto de Cami (docs/references/smilingfriends/smilingfriends_layout_sketch.png).
Las coordenadas "sk" son píxeles de ese boceto escalado a 2000 px de ancho: el círculo del boceto
(centro SK_C, radio SK_R) es el borde del disco.
"""

import bpy
import bmesh
import math
import colorsys
import os
import random
from mathutils import Matrix, Vector
from mathutils.geometry import tessellate_polygon

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "planets", "smilingfriends", "smilingfriends_planet.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "smilingfriends")
VERSION = "v01"

SEED = 2026          # Semilla de toda la distribución procedural
R = 24.0             # Radio del disco (South Park: esfera de radio 22)
T = 2.0              # Grosor del disco
TOP = T / 2          # Cara superior (océano)
LAND_H = 0.25        # Altura de la tierra sobre el océano
LAND = TOP + LAND_H

SK_C, SK_R = (1030.0, 735.0), 715.0   # Centro y radio del disco en el boceto (px)
SK_K = R / SK_R                        # Unidades de Blender por píxel del boceto

# ---------------------------------------------------------------------------
# Utilidades
# ---------------------------------------------------------------------------

def srgb(h):
    h = h.lstrip("#")
    c = [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    return tuple(((v + 0.055) / 1.055) ** 2.4 if v > 0.04045 else v / 12.92 for v in c)


def hex_lerp(c0, c1, t):
    a, b = [int(c0[i:i + 2], 16) for i in (1, 3, 5)], [int(c1[i:i + 2], 16) for i in (1, 3, 5)]
    return "#%02x%02x%02x" % tuple(round(x + (y - x) * t) for x, y in zip(a, b))


MATS = {}


def mat(name, hexcol, emit=0.0, rough=0.85):
    full = "SF_" + name
    if full in MATS:
        return MATS[full]
    m = bpy.data.materials.new(full)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get("Principled BSDF")
    c = srgb(hexcol)
    bsdf.inputs["Base Color"].default_value = (*c, 1)
    bsdf.inputs["Roughness"].default_value = rough
    if emit:
        bsdf.inputs["Emission Color"].default_value = (*c, 1)
        bsdf.inputs["Emission Strength"].default_value = emit
    m.diffuse_color = (*c, 1)
    MATS[full] = m
    return m


def sk(px, py):
    """Píxel del boceto -> (x, y) del mundo. Lo que cae fuera del disco se trae hasta el borde."""
    x, y = (px - SK_C[0]) * SK_K, -(py - SK_C[1]) * SK_K
    r, lim = math.hypot(x, y), R - 0.5
    if r > lim:
        x, y = x * lim / r, y * lim / r
    return x, y


def mtx(loc=(0, 0, 0), rot_z=0.0, rot_x=0.0, scale=(1, 1, 1)):
    return (Matrix.Translation(loc) @ Matrix.Rotation(rot_z, 4, "Z")
            @ Matrix.Rotation(rot_x, 4, "X") @ Matrix.Diagonal((*scale, 1)))


def inside(p, poly):
    """Punto dentro de polígono (par/impar)."""
    x, y = p
    c = False
    for (x1, y1), (x2, y2) in zip(poly, poly[1:] + poly[:1]):
        if (y1 > y) != (y2 > y) and x < x1 + (y - y1) * (x2 - x1) / (y2 - y1):
            c = not c
    return c


def dist_to_path(p, path):
    best = 1e9
    for a, b in zip(path, path[1:]):
        ax, ay = b[0] - a[0], b[1] - a[1]
        t = max(0.0, min(1.0, ((p[0] - a[0]) * ax + (p[1] - a[1]) * ay) / (ax * ax + ay * ay or 1)))
        best = min(best, math.hypot(p[0] - a[0] - ax * t, p[1] - a[1] - ay * t))
    return best


def smooth(pts, sub=8):
    """Curva Catmull-Rom que pasa por pts."""
    P = [pts[0]] + list(pts) + [pts[-1]]
    out = []
    for i in range(1, len(P) - 2):
        p0, p1, p2, p3 = P[i - 1], P[i], P[i + 1], P[i + 2]
        for s in range(sub):
            t = s / sub
            out.append(tuple(0.5 * (2 * p1[k] + (-p0[k] + p2[k]) * t + (2 * p0[k] - 5 * p1[k] + 4 * p2[k] - p3[k]) * t * t
                                    + (-p0[k] + 3 * p1[k] - 3 * p2[k] + p3[k]) * t ** 3) for k in (0, 1)))
    out.append(tuple(pts[-1]))
    return out


def resample(pts, step):
    """Puntos equiespaciados a lo largo de la polilínea."""
    out, carry = [pts[0]], 0.0
    for a, b in zip(pts, pts[1:]):
        seg = math.dist(a, b)
        d = step - carry
        while d <= seg:
            t = d / seg
            out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t))
            d += step
        carry = seg - (d - step)
    if math.dist(out[-1], pts[-1]) > step * 0.3:
        out.append(pts[-1])
    return out


ISLAND_BEVEL = 0.16   # Chaflán de la orilla de las islas


def offset_poly(poly, d):
    """Polígono antihorario desplazado hacia afuera una distancia d (inglete en cada vértice)."""
    out, n = [], len(poly)
    for i in range(n):
        a, b, c = poly[i - 1], poly[i], poly[(i + 1) % n]
        n1 = Vector((b[1] - a[1], -(b[0] - a[0]))).normalized()
        n2 = Vector((c[1] - b[1], -(c[0] - b[0]))).normalized()
        m = (n1 + n2).normalized() if (n1 + n2).length > 1e-6 else n1
        k = d / max(m.dot(n1), 0.35)
        out.append((b[0] + m.x * k, b[1] + m.y * k))
    return out


def shore(bm, poly, z_top, z_bot, bevel=ISLAND_BEVEL):
    """Costado de una isla: chaflán desde el borde de arriba hacia afuera y luego pared vertical. Devuelve las caras."""
    poly = [tuple(q[:2]) for q in poly]
    if sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(poly, poly[1:] + poly[:1])) < 0:
        poly = poly[::-1]
    outer = offset_poly(poly, bevel)
    top = [bm.verts.new((x, y, z_top)) for x, y in poly]
    mid = [bm.verts.new((x, y, z_top - bevel)) for x, y in outer]
    bot = [bm.verts.new((x, y, z_bot)) for x, y in outer]
    faces = []
    for k in range(len(poly)):
        j = (k + 1) % len(poly)
        faces.append(bm.faces.new((mid[k], mid[j], top[j], top[k])))
        faces.append(bm.faces.new((bot[k], bot[j], mid[j], mid[k])))
    return faces


class Part:
    """Malla en construcción con varias ranuras de material."""

    def __init__(self, name):
        self.name = name
        self.bm = bmesh.new()
        self.mats = []
        self.detail = False   # True: las caras nuevas son detalles (sin contorno propio)
        self._dl = self.bm.faces.layers.int.new("detail")

    def mi(self, m):
        if m not in self.mats:
            self.mats.append(m)
        return self.mats.index(m)

    def _tag(self, before, m, smooth=False):
        i = self.mi(m)
        for f in self.bm.faces:
            if f not in before:
                f.material_index = i
                f.smooth = smooth
                f[self._dl] = int(self.detail)

    def op(self, m, kind, smooth=False, **kw):
        before = set(self.bm.faces)
        getattr(bmesh.ops, kind)(self.bm, **kw)
        self._tag(before, m, smooth)

    def box(self, m, size, loc, rot_z=0.0, rot_x=0.0):
        self.op(m, "create_cube", size=1.0, matrix=mtx(loc, rot_z, rot_x, scale=size))

    def cyl(self, m, r1, r2, depth, loc, seg=16, rot_z=0.0, rot_x=0.0, caps=True, scale=(1, 1, 1)):
        self.op(m, "create_cone", cap_ends=caps, cap_tris=False, segments=seg,
                radius1=r1, radius2=r2, depth=depth, matrix=mtx(loc, rot_z, rot_x, scale))

    def sphere(self, m, r, loc, seg=10, rings=6, scale=(1, 1, 1), rot_z=0.0):
        self.op(m, "create_uvsphere", smooth=True, u_segments=seg, v_segments=rings, radius=r,
                matrix=mtx(loc, rot_z, scale=scale))

    def dome(self, m, r, loc, seg=16, rings=10, scale=(1, 1, 1)):
        """Media esfera superior (base abierta en z=loc.z)."""
        before = set(self.bm.verts)
        self.sphere(m, r, loc, seg, rings, scale)
        low = [v for v in self.bm.verts if v not in before and v.co.z < loc[2] - 1e-4]
        bmesh.ops.delete(self.bm, geom=low, context="VERTS")

    def ico(self, m, r, loc, subdiv=1, scale=(1, 1, 1)):
        self.op(m, "create_icosphere", subdivisions=subdiv, radius=r, matrix=mtx(loc, scale=scale))

    def prism(self, m, pts, offset, bottom=True, bevel=0.0):
        """Polígono 3D (pts) extruido por offset. Corrige la orientación sola. bevel: orilla con chaflán (islas)."""
        off = Vector(offset)
        n = Vector((0, 0, 0))
        for i, p in enumerate(pts):
            q = pts[(i + 1) % len(pts)]
            n += Vector(((p[1] - q[1]) * (p[2] + q[2]), (p[2] - q[2]) * (p[0] + q[0]), (p[0] - q[0]) * (p[1] + q[1])))
        if n.dot(off) < 0:
            pts = list(reversed(pts))
        bm = self.bm
        i = self.mi(m)
        vb = [bm.verts.new(p) for p in pts]
        vt = [bm.verts.new(Vector(p) + off) for p in pts]
        faces = []
        # Tapas trianguladas a mano: bmesh.ops.triangulate voltea triángulos en polígonos cóncavos
        for a, b, c in tessellate_polygon([[Vector(p) for p in pts]]):
            if (Vector(pts[b]) - Vector(pts[a])).cross(Vector(pts[c]) - Vector(pts[a])).dot(off) < 0:
                b, c = c, b
            faces.append(bm.faces.new((vt[a], vt[b], vt[c])))
            if bottom:
                faces.append(bm.faces.new((vb[a], vb[c], vb[b])))
        if bevel:
            faces += shore(bm, [Vector(q) + off for q in pts], pts[0][2] + off.z, pts[0][2], bevel)
        else:
            for k in range(len(pts)):
                j = (k + 1) % len(pts)
                faces.append(bm.faces.new((vb[k], vb[j], vt[j], vt[k])))
        for f in faces:
            f.material_index = i

    def ribbon(self, m, pts, width, z_top, z_bot=None, dash=None):
        """Cinta (carretera o sendero) a lo largo de pts (x, y). z_bot añade costados; dash = material de la línea.
        m puede ser un material o una función tramo -> material (carretera arcoíris)."""
        n = len(pts)
        bm = self.bm
        edges = []
        for i, (x, y) in enumerate(pts):
            a, b = pts[max(i - 1, 0)], pts[min(i + 1, n - 1)]
            tx, ty = b[0] - a[0], b[1] - a[1]
            ln = math.hypot(tx, ty) or 1.0
            nx, ny = -ty / ln, tx / ln
            edges.append(((x + nx * width / 2, y + ny * width / 2), (x - nx * width / 2, y - ny * width / 2), (nx, ny)))
        zc = [z_top(*q) if callable(z_top) else z_top for q in pts]   # Altura por punto (plana de lado a lado)
        lt = [bm.verts.new((e[0][0], e[0][1], zc[i])) for i, e in enumerate(edges)]
        rt = [bm.verts.new((e[1][0], e[1][1], zc[i])) for i, e in enumerate(edges)]
        faces = [bm.faces.new((rt[i], rt[i + 1], lt[i + 1], lt[i])) for i in range(n - 1)]
        seg = list(range(n - 1))
        if z_bot is not None:
            lb = [bm.verts.new((e[0][0], e[0][1], z_bot)) for e in edges]
            rb = [bm.verts.new((e[1][0], e[1][1], z_bot)) for e in edges]
            for i in range(n - 1):
                faces.append(bm.faces.new((lt[i], lt[i + 1], lb[i + 1], lb[i])))
                faces.append(bm.faces.new((rb[i], rb[i + 1], rt[i + 1], rt[i])))
                seg += [i, i]
        faces[0].normal_update()
        if faces[0].normal.z < 0:
            bmesh.ops.reverse_faces(bm, faces=faces)
        for f, i in zip(faces, seg):
            f.material_index = self.mi(m(i) if callable(m) else m)
        if dash:  # Línea central discontinua: un tramo sí y uno no
            dfaces = []
            for k in range(0, n - 1, 2):
                (x0, y0), (x1, y1) = pts[k], pts[k + 1]
                nx, ny = edges[k][2]
                q = [bm.verts.new((x0 - nx * 0.06, y0 - ny * 0.06, zc[k] + 0.03)),
                     bm.verts.new((x1 - nx * 0.06, y1 - ny * 0.06, zc[k + 1] + 0.03)),
                     bm.verts.new((x1 + nx * 0.06, y1 + ny * 0.06, zc[k + 1] + 0.03)),
                     bm.verts.new((x0 + nx * 0.06, y0 + ny * 0.06, zc[k] + 0.03))]
                f = bm.faces.new(q)
                f.normal_update()
                if f.normal.z < 0:
                    f.normal_flip()
                dfaces.append(f)
            j = self.mi(dash)
            for f in dfaces:
                f.material_index = j

    def place(self, matrix):
        bmesh.ops.transform(self.bm, matrix=matrix, verts=self.bm.verts)

    def build(self, coll, parent, recalc=False):
        me = bpy.data.meshes.new(self.name)
        if recalc:  # Solo para mallas cerradas; en mallas abiertas puede voltear caras
            bmesh.ops.recalc_face_normals(self.bm, faces=self.bm.faces)
        self.bm.to_mesh(me)
        self.bm.free()
        for m in self.mats:
            me.materials.append(m)
        ob = bpy.data.objects.new(self.name, me)
        coll.objects.link(ob)
        ob.parent = parent
        return ob


def collection(name, parent):
    c = bpy.data.collections.new(name)
    parent.children.link(c)
    return c


def scatter(rng, n, radius, min_dist, center=(0, 0), tries=40):
    """Muestreo por rechazo dentro de un círculo: misma semilla, mismas posiciones."""
    pts = []
    for _ in range(n * tries):
        if len(pts) >= n:
            break
        a = rng.uniform(0, math.tau)
        d = radius * math.sqrt(rng.random())
        p = (center[0] + d * math.cos(a), center[1] + d * math.sin(a))
        if all((p[0] - q[0]) ** 2 + (p[1] - q[1]) ** 2 >= min_dist ** 2 for q in pts):
            pts.append(p)
    return pts


def scatter_poly(rng, n, poly, min_dist, ok=lambda p: True, tries=80):
    """Muestreo por rechazo dentro de un polígono, con filtro opcional."""
    xs, ys = [p[0] for p in poly], [p[1] for p in poly]
    pts = []
    for _ in range(n * tries):
        if len(pts) >= n:
            break
        p = (rng.uniform(min(xs), max(xs)), rng.uniform(min(ys), max(ys)))
        if inside(p, poly) and ok(p) and all(math.dist(p, q) >= min_dist for q in pts):
            pts.append(p)
    return pts


def scatter_ellipse(rng, n, cx, cy, a, b, min_dist, tries=80):
    pts = []
    for _ in range(n * tries):
        if len(pts) >= n:
            break
        t, d = rng.uniform(0, math.tau), math.sqrt(rng.random())
        p = (cx + a * d * math.cos(t), cy + b * d * math.sin(t))
        if all(math.dist(p, q) >= min_dist for q in pts):
            pts.append(p)
    return pts


# ---------------------------------------------------------------------------
# Distribución del boceto (px del boceto a 2000 px de ancho)
# ---------------------------------------------------------------------------

BR_PX = 7.8   # Tamaño de Brasil (px del boceto por grado)
LAND_SK = {
    "NorthWest": [(470, 282), (560, 252), (680, 258), (790, 255), (860, 295), (885, 370), (875, 430), (820, 460),
                  (775, 500), (748, 560), (735, 620), (700, 665), (665, 705), (645, 745), (600, 725), (540, 695),
                  (480, 650), (448, 580), (455, 500), (478, 430), (468, 360)],
    "Central": [(700, 605), (760, 560), (840, 515), (900, 480), (1000, 458), (1080, 455), (1140, 478), (1165, 540),
                (1250, 575), (1360, 585), (1400, 640), (1395, 760), (1380, 840), (1340, 900), (1250, 940),
                (1100, 965), (950, 965), (820, 945), (760, 905), (700, 855), (655, 790), (640, 730), (665, 670)],
    "FrogPeninsula": [(960, 410), (1000, 385), (1060, 365), (1100, 330), (1130, 275), (1165, 215), (1185, 140),
                      (1205, 85), (1255, 62), (1310, 88), (1375, 125), (1425, 170), (1432, 210), (1405, 265),
                      (1365, 320), (1300, 375), (1340, 450), (1348, 560), (1285, 582), (1245, 470), (1230, 425),
                      (1170, 465), (1120, 478), (1060, 468), (1000, 455),
                      (965, 440)],
    "East": [(1330, 585), (1390, 545), (1470, 522), (1570, 508), (1700, 505), (1742, 600), (1745, 760), (1725, 860),
             (1655, 940), (1560, 990), (1470, 985), (1395, 950), (1370, 880), (1385, 800), (1360, 700)],
    "Enchanted": [(445, 870), (520, 830), (600, 840), (650, 855), (690, 885), (705, 930), (700, 1000), (690, 1060),
                  (650, 1110), (598, 1142), (520, 1132), (470, 1080), (448, 1000), (438, 930)],
    "DemonIsland": [(830, 991), (900, 981), (990, 989), (1080, 981), (1192, 981), (1198, 1041), (1172, 1101),
                    (1100, 1143), (1000, 1153), (900, 1143), (850, 1083), (825, 1031)],
    # Silueta simplificada de Brasil (longitud, latitud) llevada al lugar del boceto: BR_PX px por grado
    "Brazil": [(1325 + (lo + 54) * BR_PX, 1200 - (la + 14.5) * BR_PX) for lo, la in (
        (-60, 5), (-51, 4), (-50, 0), (-44, -2), (-35, -5), (-35, -9), (-39, -13), (-39, -18), (-41, -22),
        (-45, -24), (-48, -26), (-49, -29), (-53, -34), (-57, -30), (-54, -26), (-55, -23), (-58, -20),
        (-58, -16), (-60, -14), (-65, -12), (-70, -11), (-73, -8), (-70, -4), (-70, 1), (-67, 2), (-64, 4))],
}
LANDW = {k: [sk(*p) for p in v] for k, v in LAND_SK.items()}
BR_ROT = math.radians(-20)                       # Brasil girado un poco a la derecha (sentido horario)
BR_PIV = sk(1325, 1200)


def br_rot(q):
    """Gira un punto de Brasil alrededor de su centro."""
    dx, dy = q[0] - BR_PIV[0], q[1] - BR_PIV[1]
    c, s_ = math.cos(BR_ROT), math.sin(BR_ROT)
    return BR_PIV[0] + dx * c - dy * s_, BR_PIV[1] + dx * s_ + dy * c


LANDW["Brazil"] = [br_rot(q) for q in LANDW["Brazil"]]

# Zona Shmaloogle (colinas, pueblo y castillo): fijada aquí porque la tierra del este se ajusta a ella
HILL_C = sk(1445, 612)
HILL2_C = (HILL_C[0] + 4.6, HILL_C[1] + 0.8)   # Segunda colina idéntica: la carretera pasa por las dos
SH_C, SH_A, SH_B = (19.4, -3.5), 3.3, 3.9      # Pueblo (elipse del muro)
CASTLE_C = (13.9, -1.7)                         # Castillo del mago, entre la carretera y el pueblo
# Carretera del este: rodea las colinas por el sur (sin subirlas) y entra al pueblo por el norte.
# La tierra del este se agranda para que quepa con margen.
# Como en el boceto de Cami: sube a la primera colina, cruza las dos por la cima en línea recta (por sus
# centros, así queda simétrica), baja por el este y entra al pueblo por una puerta al noreste.
GATE_T = math.atan2((3.2 - SH_C[1]) / SH_B, (21.4 - SH_C[0]) / SH_A)
_gate_pt = lambda dr: (SH_C[0] + (SH_A + dr) * math.cos(GATE_T), SH_C[1] + (SH_B + dr) * math.sin(GATE_T))
EAST_ROAD_PTS = [sk(1365, 712), (sk(1365, 712)[0] + 0.6, sk(1365, 712)[1]), (sk(1365, 712)[0] + 1.3, sk(1365, 712)[1]),
                 (13.0, 1.6), HILL_C, HILL2_C, (21.4, 3.2), _gate_pt(1.2), _gate_pt(0.1)]


def hull(pts):
    """Envolvente convexa (cadena monótona)."""
    pts = sorted(set(pts))
    def half(seq):
        h = []
        for q in seq:
            while len(h) >= 2 and ((h[-1][0] - h[-2][0]) * (q[1] - h[-2][1])
                                   - (h[-1][1] - h[-2][1]) * (q[0] - h[-2][0])) <= 0:
                h.pop()
            h.append(q)
        return h
    lo, up = half(pts), half(pts[::-1])
    return lo[:-1] + up[:-1]


_east = [(11.0, 3.6), (11.0, -2.6), (11.3, -4.2), (13.4, -5.6)]   # Se une a la tierra central; lomo redondeado al suroeste del castillo
for _q in resample(EAST_ROAD_PTS, 0.5):         # Margen alrededor de la carretera
    _east += [(_q[0] + 1.8 * math.cos(t), _q[1] + 1.8 * math.sin(t)) for t in (k / 12 * math.tau for k in range(12))]
for (cx, cy), ra, rb in ((HILL_C, 3.1, 3.1), (HILL2_C, 3.1, 3.1), (SH_C, SH_A + 0.8, SH_B + 0.8), (CASTLE_C, 2.0, 2.0)):
    _east += [(cx + ra * math.cos(t), cy + rb * math.sin(t)) for t in (k / 24 * math.tau for k in range(24))]
_east = [(x * min(1.0, 23.0 / math.hypot(x, y)), y * min(1.0, 23.0 / math.hypot(x, y))) for x, y in hull(_east)]
LANDW["East"] = _east


def chaikin(poly, n):
    """Redondea las esquinas de un polígono cerrado (corte de esquinas de Chaikin)."""
    for _ in range(n):
        out = []
        for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
            out += [(0.75 * x0 + 0.25 * x1, 0.75 * y0 + 0.25 * y1), (0.25 * x0 + 0.75 * x1, 0.25 * y0 + 0.75 * y1)]
        poly = out
    return poly


for _k, _n in (("Central", 2), ("East", 2), ("NorthWest", 2), ("FrogPeninsula", 2), ("Enchanted", 2), ("Brazil", 1)):
    LANDW[_k] = chaikin(LANDW[_k], _n)          # Orillas un poco redondeadas (la isla del demonio sigue dentada)


def merge_rounded(polys, res=0.12, radius=1.6):
    """Une varios polígonos en una sola tierra y redondea las uniones (cierre morfológico sobre una rejilla):
    rellena las muescas donde se juntan las islas y devuelve una sola orilla suave."""
    import numpy as np
    pts = [q for poly in polys for q in poly]
    pad = radius + 1.0
    x0, y0 = min(q[0] for q in pts) - pad, min(q[1] for q in pts) - pad
    x1, y1 = max(q[0] for q in pts) + pad, max(q[1] for q in pts) + pad
    nx_, ny_ = int((x1 - x0) / res) + 1, int((y1 - y0) / res) + 1
    gx, gy = np.meshgrid(x0 + np.arange(nx_) * res, y0 + np.arange(ny_) * res)
    mask = np.zeros(gx.shape, bool)
    for poly in polys:                                    # Punto dentro de polígono (par/impar), vectorizado
        inside_ = np.zeros(gx.shape, bool)
        for (ax, ay), (bx, by) in zip(poly, poly[1:] + poly[:1]):
            cond = (ay > gy) != (by > gy)
            xint = ax + (gy - ay) * (bx - ax) / ((by - ay) if by != ay else 1e-9)
            inside_ ^= cond & (gx < xint)
        mask |= inside_
    r = int(radius / res)
    offs = [(i, j) for i in range(-r, r + 1) for j in range(-r, r + 1) if i * i + j * j <= r * r]

    def shift_any(m, fn):
        out = np.zeros_like(m) if fn is np.logical_or else np.ones_like(m)
        padded = np.pad(m, r, constant_values=(fn is np.logical_and) and False)
        for i, j in offs:
            out = fn(out, padded[r + i:r + i + m.shape[0], r + j:r + j + m.shape[1]])
        return out
    closed = shift_any(shift_any(mask, np.logical_or), np.logical_and)
    # Orilla: aristas entre celda adentro y celda afuera, encadenadas en un lazo
    nxt = {}
    H, W = closed.shape
    for i in range(H):
        for j in range(W):
            if not closed[i, j]:
                continue
            if i == 0 or not closed[i - 1, j]:
                nxt[(i, j)] = (i, j + 1)
            if j == W - 1 or not closed[i, j + 1]:
                nxt[(i, j + 1)] = (i + 1, j + 1)
            if i == H - 1 or not closed[i + 1, j]:
                nxt[(i + 1, j + 1)] = (i + 1, j)
            if j == 0 or not closed[i, j - 1]:
                nxt[(i + 1, j)] = (i, j)
    best = []
    seen = set()
    for start in list(nxt):
        if start in seen:
            continue
        loop, cur = [], start
        while cur not in seen and cur in nxt:
            seen.add(cur)
            loop.append(cur)
            cur = nxt[cur]
        if len(loop) > len(best):
            best = loop
    out = [(x0 + (j - 0.5) * res, y0 + (i - 0.5) * res) for i, j in best[::3]]
    for _ in range(6):                                    # Suavizado: quita la escalera de la rejilla
        out = [((a[0] + 2 * b[0] + c[0]) / 4, (a[1] + 2 * b[1] + c[1]) / 4)
               for a, b, c in zip(out[-1:] + out[:-1], out, out[1:] + out[:1])]
    return out[::2]


# Lógica (árboles, Spamtopia) usa la tierra central original; el dibujo usa la tierra central y la del este
# fundidas en una sola, con las uniones redondeadas.
# El Enchanted Forest también es parte de la tierra central (un parque cercado): se une por un istmo.
ENCHANTED_NECK = [sk(*q) for q in ((640, 848), (705, 835), (795, 915), (765, 965), (705, 962), (688, 900))]
LAND_LOGIC = {"Central": LANDW["Central"], "Enchanted": LANDW["Enchanted"]}
# Spamtopia y la península de Mr. Frog también entran en la unión (su suelo con degradado va encima)
LANDW["Central"] = merge_rounded([LANDW["Central"], LANDW["East"], LANDW["Enchanted"], ENCHANTED_NECK,
                                  LANDW["NorthWest"], LANDW["FrogPeninsula"]])
del LANDW["East"], LANDW["Enchanted"]


def jagged(poly, step, amp, seed):
    """Orilla dentada: subdivide cada lado y desplaza los puntos hacia dentro o fuera al azar (semilla fija)."""
    jr = random.Random(seed)
    out = []
    for a, b in zip(poly, poly[1:] + poly[:1]):
        n = max(1, int(math.dist(a, b) / step))
        nx, ny = b[1] - a[1], -(b[0] - a[0])
        ln = math.hypot(nx, ny) or 1.0
        for k in range(n):
            t, o = k / n, (0.0 if k == 0 else jr.uniform(-amp, amp))
            out.append((a[0] + (b[0] - a[0]) * t + nx / ln * o, a[1] + (b[1] - a[1]) * t + ny / ln * o))
    return out


LANDW["DemonIsland"] = jagged(LANDW["DemonIsland"], 0.55, 0.32, SEED + 5)

# Carretera: de la colina de Shmaloogleville (túnel), por la calle central, curva por Spamtopia y sale por el borde
ROAD_SK = [(1365, 712), (1200, 712), (1022, 712), (850, 712), (720, 710), (650, 695), (605, 655), (595, 605),
           (625, 562), (690, 515), (748, 465), (775, 420), (760, 372), (715, 330), (640, 292), (560, 262), (470, 238)]
# Camino de Mr. Frog (T1 EP2): en zigzag, a propósito larguísimo y complicado, cruzando cinco paisajes
# Sale de la calle principal (extremo este, junto a las colinas), sube por el cuello al este del fondo morado
# y zigzaguea hacia el norte cruzando los cinco paisajes hasta la casa (coordenadas del mundo).
FROG_PATH_PTS = [(10.2, 2.3), (9.9, 5.0), (9.0, 8.0), (4.5, 10.5), (9.5, 12.0), (5.0, 13.8), (10.5, 15.0),
                 (6.0, 16.5), (11.0, 17.3), (8.5, 18.6), (9.67, 19.57)]

# ---------------------------------------------------------------------------
# Escena
# ---------------------------------------------------------------------------

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "SmilingFriends_Planet"
rng = random.Random(SEED)

ROOT = bpy.data.objects.new("SmilingFriends_Planet_ROOT", None)
scene.collection.objects.link(ROOT)

C_BASE = collection("Planet_Base", scene.collection)
C_MAP = collection("Land", scene.collection)
C_RIM = collection("Rim", scene.collection)
C_HELL = collection("Hell", scene.collection)
C_ZONES = collection("Zones", scene.collection)
C_PRES = collection("Presentation", scene.collection)

M = {
    "ocean": mat("Ocean", "#3d8fd6"),
    "land": mat("Land", "#7bc062"),
    "crust": mat("Crust", "#7a5a3c"),
    "ice": mat("Ice", "#eef5fa"),
    "cloud": mat("Cloud", "#f7fafd", rough=1.0),
    "water": mat("Waterfall", "#7fd0f5", rough=0.3),
    "hellrock": mat("HellRock", "#4a1510"),
    "hellfloor": mat("HellFloor", "#8a1e0e", emit=0.6),
    "lava": mat("Lava", "#ff5a0a", emit=1.6),
    "glow": mat("HellGlow", "#ff3b12", emit=2.5),
    "black": mat("Black", "#141414"),
    "white": mat("White", "#fbfbf7"),
    "yellow": mat("SmileYellow", "#f7d23e"),
    "orange": mat("DoorOrange", "#f08a24"),
    "brick": mat("Brick", "#a6533f"),
    "asylum": mat("AsylumWall", "#cdd8d4"),
    "asylum_win": mat("AsylumWindow", "#5ccaca"),
    "asylum_sign": mat("AsylumSign", "#2fb3b3"),
    "toilet": mat("ToiletWall", "#ead6cc"),
    "toilet_win": mat("ToiletWindow", "#5f8fc6"),
    "toilet_sign": mat("ToiletSign", "#7a2a1f"),
    "ronsdeli": mat("RonsDeliWall", "#f4b8d8"),
    "ronsdeli_sign": mat("RonsDeliSign", "#8f2a6a"),
    "awning": mat("Awning", "#d8262e"),
    "grayshop": mat("GrayShop", "#aab3b8"),
    "lilac": mat("LilacShop", "#bba6da"),
    "yellowshop": mat("YellowShop", "#efd27a"),
    "antique": mat("AntiqueWall", "#aab8c6"),
    "antique_sign": mat("AntiqueSign", "#55697f"),
    "pinkbrick": mat("PinkBrick", "#cfb3b0"),
    "dumpster": mat("Dumpster", "#4f7a3a"),
    "trim": mat("Trim", "#e9e6e1"),
    "window": mat("Window", "#3b4a5a"),
    "glass": mat("Glass", "#8fc6e8", rough=0.3),
    "sidewalk": mat("Sidewalk", "#c9c6cc"),
    "road": mat("Road", "#77748a"),
    "leaf": mat("StreetLeaf", "#5bb14a"),
    "leaf2": mat("StreetLeaf2", "#4c9a3f"),
    "frown_face": mat("FrownBlue", "#5b8fd6"),
    "red": mat("SatanRed", "#c0261b"),
    "charlie": mat("CharlieYellow", "#f4d35e"),
    "wood": mat("BridgeWood", "#9a6b43"),
    "post": mat("BridgePost", "#6e4a2e"),
    "metal": mat("LadderGray", "#9a9aa2"),
    "hill": mat("Hill", "#68ad4c"),
}

# --- Disco ---------------------------------------------------------------
disc = Part("SF_Disc")
disc.cyl(M["ocean"], R, R, T, (0, 0, 0), seg=128)
side_i, bot_i = disc.mi(mat("OceanSide", "#3580cc")), disc.mi(M["hellrock"])   # Costado de océano: el mar envuelve la orilla
for f in disc.bm.faces:
    f.normal_update()
    if f.normal.z < -0.5:
        f.material_index = bot_i
    elif abs(f.normal.z) <= 0.5:
        f.material_index = side_i
# La cara de abajo la cubre SF_Underside_Ground: dos superficies casi iguales parpadean (z-fighting)
bmesh.ops.delete(disc.bm, geom=[f for f in disc.bm.faces if f.normal.z < -0.5], context="FACES_ONLY")
disc.build(C_BASE, ROOT)

# --- Tierra del boceto -------------------------------------------------------
LAND_MAT = {"Enchanted": mat("EnchantedGround", "#8be08a"), "DemonIsland": mat("DemonGround", "#2b2a3a")}
for k, (name, poly) in enumerate(LANDW.items()):
    if name in ("NorthWest", "DemonIsland", "FrogPeninsula"):
        continue  # Se construyen en su zona: llevan degradados (Spamtopia y la orilla del bosque del demonio)
    p = Part("SF_Land_" + name)
    h = LAND_H + 0.004 * k  # alturas distintas para evitar z-fighting donde se superponen
    p.prism(LAND_MAT.get(name, M["land"]), [(x, y, TOP) for x, y in poly], (0, 0, h), bottom=False,
            bevel=ISLAND_BEVEL)
    p.build(C_MAP, ROOT)


def on_land(p, margin=0.0):
    pts = [p] if not margin else [p] + [(p[0] + margin * math.cos(a), p[1] + margin * math.sin(a))
                                        for a in (0, math.pi / 2, math.pi, 1.5 * math.pi)]
    return any(inside(q, poly) for q in pts for poly in LANDW.values())


# --- Borde: Antártida de nubes y cascada ----------------------------------------
# Como en la serie, el borde es una franja de nubes/hielo. Cada nube es una bola independiente.
# Hacia el centro las nubes son más escasas, pequeñas y altas: el borde se desvanece en vez de
# terminar en una línea. Semilla propia para no mover la distribución de las zonas.
crng = random.Random(SEED + 1)

shelf = Part("SF_IceShelf")  # Anillo bajo de hielo bajo las nubes del borde: tapa el océano entre bolas
N, r_in, r_out, z_top = 96, R - 0.55, R + 0.05, TOP + 0.05
ring = [(shelf.bm.verts.new((r_in * math.cos(i / N * math.tau), r_in * math.sin(i / N * math.tau), z_top)),
         shelf.bm.verts.new((r_out * math.cos(i / N * math.tau), r_out * math.sin(i / N * math.tau), z_top)),
         shelf.bm.verts.new((r_out * math.cos(i / N * math.tau), r_out * math.sin(i / N * math.tau), TOP - 0.4)))
        for i in range(N)]
ice_i = shelf.mi(M["ice"])
for i in range(N):
    p0, p1 = ring[i], ring[(i + 1) % N]
    for f in (shelf.bm.faces.new((p0[0], p0[1], p1[1], p1[0])), shelf.bm.faces.new((p0[1], p0[2], p1[2], p1[1]))):
        f.material_index = ice_i
shelf.build(C_RIM, ROOT, recalc=True)

# Familias de nubes. Todas se arman con bolas: una o varias principales y otras menores alrededor.
# size es el radio de la bola principal; tangent es la dirección en que la nube se alarga.

CLOUD_LOD = False   # True: versión lejana de las nubes (mismas bolas, muchos menos segmentos)


def _puff(part, cx, cy, cz, r, flat=0.85):
    if CLOUD_LOD:
        part.sphere(M["cloud"], r, (cx, cy, cz), seg=5, rings=3, scale=(1, 1, flat))
        return
    small = r < 0.6
    part.sphere(M["cloud"], r, (cx, cy, cz), seg=7 if small else 9, rings=4 if small else 5, scale=(1, 1, flat))


def _at(cx, cy, t, d, n=0.0):
    """Punto a distancia d a lo largo de t y n en perpendicular."""
    return cx + t[0] * d - t[1] * n, cy + t[1] * d + t[0] * n


def cloud_cumulus(part, cx, cy, cz, size, t):
    """Cúmulo clásico: bola principal, 1-2 bolas más bajas a cada lado y a veces un bulto arriba."""
    _puff(part, cx, cy, cz, size)
    for side in (-1, 1):
        for k in range(crng.randint(1, 2)):
            d = side * size * (0.85 + 0.6 * k) * crng.uniform(0.9, 1.1)
            r = size * (0.68 - 0.18 * k) * crng.uniform(0.85, 1.1)
            _puff(part, *_at(cx, cy, t, d, crng.uniform(-0.15, 0.15) * size), cz - size * (0.18 + 0.1 * k), r)
    if crng.random() < 0.6:
        _puff(part, *_at(cx, cy, t, size * 0.4 * crng.choice((-1, 1))), cz + size * 0.45, size * 0.6)


def cloud_congestus(part, cx, cy, cz, size, t):
    """Torre: bolas apiladas hacia arriba y lados cortos."""
    _puff(part, cx, cy, cz, size)
    o = size * crng.uniform(0.15, 0.35) * crng.choice((-1, 1))
    _puff(part, *_at(cx, cy, t, o), cz + size * 0.55, size * 0.78)            # bolas superiores muy encimadas,
    _puff(part, *_at(cx, cy, t, -o * 0.8), cz + size * 0.25, size * 0.65)     # escalonadas a ambos lados
    if crng.random() < 0.6:
        _puff(part, *_at(cx, cy, t, o * 0.4), cz + size * 1.0, size * 0.5)
    for side in (-1, 1):
        _puff(part, *_at(cx, cy, t, side * size * 0.9), cz - size * 0.2, size * crng.uniform(0.55, 0.62))


def cloud_stratocumulus(part, cx, cy, cz, size, t):
    """Banda larga: 4-6 bolas medianas en fila, alturas casi iguales."""
    n = crng.randint(4, 6)
    radii = [size * crng.uniform(0.55, 0.8) for _ in range(n)]
    step = sum(radii) / n * 1.0               # muy encimadas para que no parezca una oruga
    for i, r in enumerate(radii):
        d = (i - (n - 1) / 2) * step
        _puff(part, *_at(cx, cy, t, d, crng.uniform(-0.12, 0.12) * size), cz + crng.uniform(-0.15, 0.15) * size, r, 0.8)
    for _ in range(crng.randint(1, 2)):        # bultos encima de la banda
        d = crng.uniform(-0.35, 0.35) * n * step
        _puff(part, *_at(cx, cy, t, d), cz + size * 0.4, size * crng.uniform(0.45, 0.6))


def cloud_lopsided(part, cx, cy, cz, size, t):
    """Asimétrica: una bola secundaria grande y una cola hacia un lado, un bulto pequeño al otro."""
    side = crng.choice((-1, 1))
    _puff(part, cx, cy, cz, size)
    _puff(part, *_at(cx, cy, t, side * size * 0.9), cz - size * 0.1, size * 0.8)
    _puff(part, *_at(cx, cy, t, side * size * 1.65), cz - size * 0.28, size * 0.5)
    _puff(part, *_at(cx, cy, t, -side * size * 0.8), cz - size * 0.3, size * 0.45)


def cloud_cluster(part, cx, cy, cz, size, t):
    """Altocúmulo: grupito esponjoso de 5-7 bolitas a alturas parecidas."""
    for _ in range(crng.randint(5, 7)):
        a = crng.uniform(0, math.tau)
        rr = math.sqrt(crng.random())
        _puff(part, *_at(cx, cy, t, math.cos(a) * rr * size * 1.5, math.sin(a) * rr * size * 0.8),
              cz + crng.uniform(-0.15, 0.2) * size, size * crng.uniform(0.45, 0.62))


def cloud_humilis(part, cx, cy, cz, size, t):
    """Nubecita plana de buen tiempo: 2-3 bolas aplastadas."""
    n = crng.randint(2, 3)
    for i in range(n):
        d = (i - (n - 1) / 2) * size * 0.8
        _puff(part, *_at(cx, cy, t, d), cz - abs(d) * 0.12, size * crng.uniform(0.6, 0.8), 0.6)


CLOUD_FAMILIES = {
    "cumulus": cloud_cumulus, "congestus": cloud_congestus, "stratocumulus": cloud_stratocumulus,
    "lopsided": cloud_lopsided, "cluster": cloud_cluster, "humilis": cloud_humilis,
}
# Pesos por lugar: el borde necesita nubes anchas que tapen la orilla; la transición, variedad.
RIM_MIX = {"cumulus": 3, "stratocumulus": 3, "lopsided": 2, "congestus": 1, "humilis": 1}
FADE_MIX = {"cumulus": 3, "lopsided": 2, "cluster": 2, "humilis": 2, "congestus": 1.5, "stratocumulus": 1}


def cloud(part, cx, cy, cz, size, tangent, mix):
    fam = crng.choices(list(mix), weights=list(mix.values()))[0]
    CLOUD_FAMILIES[fam](part, cx, cy, cz, size, tangent)


def build_clouds(suffix, coll):
    """Nubes del borde y de transición. Se reinicia la semilla: la versión lejana cae en los mismos lugares."""
    global crng
    crng = random.Random(SEED + 1)
    # Borde: formaciones que cuelgan sobre la orilla y tapan parte del costado del disco
    rim = Part("SF_Clouds_Rim" + suffix)
    n = 52
    for i in range(n):
        a = (i + crng.uniform(-0.25, 0.25)) / n * math.tau
        rr = R + 0.3 + crng.uniform(-0.35, 0.2)
        cloud(rim, rr * math.cos(a), rr * math.sin(a), TOP + crng.uniform(-0.6, -0.2), crng.uniform(1.0, 1.4),
              (-math.sin(a), math.cos(a)), RIM_MIX)
    rim.build(coll, ROOT)
    # Transición: formaciones más pequeñas, escasas y altas hacia el centro, solo sobre el océano
    fade = Part("SF_Clouds_Fade" + suffix)
    placed = 0
    for _ in range(800):
        if placed >= 28:
            break
        t = crng.random() ** 1.6                 # sesgo hacia el borde
        r = (R - 2.5) - t * 7.0                  # de r=21.5 hacia r=14.5
        a = crng.uniform(0, math.tau)
        x, y = r * math.cos(a), r * math.sin(a)
        if on_land((x, y), margin=1.5):
            continue                             # no tapar la tierra ni las zonas
        inner = (R - 2.5 - r) / 7.0              # 0 cerca del borde, 1 hacia el centro
        d = crng.uniform(0, math.tau)
        cloud(fade, x, y, TOP + 0.9 + inner * 1.3, 0.8 - inner * 0.35, (math.cos(d), math.sin(d)), FADE_MIX)
        placed += 1
    fade.build(coll, ROOT)


build_clouds("", C_RIM)
# Nivel de detalle lejano: el motor cambia a estas nubes cuando el planeta está lejos. Ocultas en Blender.
C_LOD = collection("LOD_Far", scene.collection)
CLOUD_LOD = True
build_clouds("_Far", C_LOD)
CLOUD_LOD = False
C_LOD.hide_viewport = C_LOD.hide_render = True


# --- Cara inferior (todo cuelga hacia -Z) ------------------------------------------
# Por debajo el disco es una masa sólida: una montaña de hielo invertida que se hace más profunda hacia el
# centro, con picos de hielo que salen de ella, y el infierno como un cráter al fondo. Semilla propia.
HELL_R = 9.0             # Radio del cráter del infierno
UD_MAX = 4.5             # Profundidad de la masa de hielo junto al cráter
hrng = random.Random(SEED + 7)
ICE = [mat("UnderIce", "#d8eefc"), mat("UnderIce2", "#a9d6f5"), mat("UnderIce3", "#c4e3f7")]
ICE_EDGE = [mat("UnderIceBlue", "#6fb2ec"), mat("UnderIceBlue2", "#4f9be0"), mat("UnderIceBlue3", "#86c2f0")]


def under_depth(r):
    """Profundidad de la masa inferior bajo la cara de abajo del disco, según el radio."""
    if r >= HELL_R:
        k = (R - r) / (R - HELL_R)                   # 0 en el borde, 1 en el borde del cráter
        return UD_MAX * (1 - (1 - k) ** 2)
    return UD_MAX - 1.6 * (1 - (r / HELL_R) ** 2)    # Cráter: hundido hacia adentro de la masa


UNDER_STOPS = [(0.0, "#2f72bd"), (0.25, "#2a5f9e"), (0.5, "#a9cde6"), (0.8, "#7d8fa8"), (1.0, "#5a3a40")]   # Océano profundo en el borde que pasa a hielo


def under_color(r):
    if r < HELL_R:
        return M["hellfloor"]                         # Piso del infierno: rojo encendido
    t = round((R - r) / (R - HELL_R) * 10) / 10
    for (s0, c0), (s1, c1) in zip(UNDER_STOPS, UNDER_STOPS[1:]):
        if s0 <= t <= s1:
            return mat("Underside_%02d" % round(t * 10), hex_lerp(c0, c1, (t - s0) / (s1 - s0)))


under = Part("SF_Underside_Mass")
NA, NR = 96, 28
radii = [R * (1 - i / NR) for i in range(NR)]          # De R hacia el centro (sin el 0)
grid = []
for r in radii:
    row = []
    for j in range(NA):
        t = j / NA * math.tau
        k = (R - r) / (R - HELL_R)
        noise = hrng.uniform(-0.35, 0.35) * min(1.0, k * 2.5) if r < R - 0.01 and r > HELL_R + 0.3 else 0.0
        row.append(under.bm.verts.new((r * math.cos(t), r * math.sin(t), -TOP - under_depth(r) - noise)))
    grid.append(row)
center = under.bm.verts.new((0, 0, -TOP - under_depth(0)))
faces = []
for i in range(NR - 1):
    rm = (radii[i] + radii[i + 1]) / 2
    for j in range(NA):
        f = under.bm.faces.new((grid[i][j], grid[i][(j + 1) % NA], grid[i + 1][(j + 1) % NA], grid[i + 1][j]))
        f.material_index = under.mi(under_color(rm))
        faces.append(f)
for j in range(NA):
    f = under.bm.faces.new((grid[-1][j], grid[-1][(j + 1) % NA], center))
    f.material_index = under.mi(M["hellrock"])
    faces.append(f)
for f in faces:
    f.normal_update()
    if f.normal.z > 0:
        f.normal_flip()
# Picos de hielo que salen de la masa: más azules y cortos hacia el borde, más claros y largos hacia adentro
for x, y in scatter(hrng, 190, R - 0.4, 0.95):
    r = math.hypot(x, y)
    if r < HELL_R + 0.6:
        continue
    k = (R - r) / (R - HELL_R)
    h = (1.4 + 3.2 * k ** 1.3) * hrng.uniform(0.75, 1.2)
    w = (0.5 + 0.55 * k) * hrng.uniform(0.8, 1.2)
    z0 = -TOP - under_depth(r) + 0.35                 # Nace dentro de la masa: no queda flotando
    pal = ICE_EDGE if k < 0.45 else ICE
    under.cyl(pal[hrng.randrange(3)], 0.0, w, h, (x, y, z0 - h / 2), seg=6, rot_z=hrng.uniform(0, 1))
# Cráter del infierno: llamas, picos de roca roja y brillo en el borde
FIRE = [mat("FireRed", "#ff3a10", emit=0.9), mat("FireOrange", "#ff8a00", emit=1.3), mat("FireYellow", "#ffd000", emit=1.8)]
for x, y in scatter(hrng, 46, HELL_R - 0.6, 1.0):          # Llamas: tres conos (rojo, naranja, amarillo) por llama
    r = math.hypot(x, y)
    if r < 3.6:
        continue                                      # Espacio para el escritorio de Satanás
    z0 = -TOP - under_depth(r) + 0.1
    h = hrng.uniform(0.9, 1.9)
    w = hrng.uniform(0.35, 0.6)
    tilt = hrng.uniform(0, math.tau)
    for k, (fw, fh) in enumerate(((1.0, 1.0), (0.66, 0.75), (0.36, 0.5))):
        under.cyl(FIRE[k], 0.0, w * fw, h * fh, (x + 0.04 * k * math.cos(tilt), y + 0.04 * k * math.sin(tilt),
                                                   z0 - h * fh / 2 - 0.02 * k), seg=5, rot_z=hrng.uniform(0, 1))
for x, y in scatter(hrng, 14, HELL_R - 0.4, 2.2):
    r = math.hypot(x, y)
    if r < 3.6:
        continue                                      # Espacio para el escritorio de Satanás
    h = hrng.uniform(1.2, 2.8)
    under.cyl(M["hellrock"], 0.0, hrng.uniform(0.5, 1.0), h, (x, y, -TOP - under_depth(r) + 0.2 - h / 2), seg=6)
under.build(C_HELL, ROOT)


# ---------------------------------------------------------------------------
# Calle principal: Smiling Friends Inc. y Frowning Friends frente a frente (T1 EP7)
# ---------------------------------------------------------------------------

def merge(dst, src):
    """Copia la geometría de src a dst conservando materiales."""
    remap = [dst.mi(m) for m in src.mats]
    me = bpy.data.meshes.new("_merge")
    src.bm.to_mesh(me)
    src.bm.free()
    before = set(dst.bm.faces)
    dst.bm.from_mesh(me)
    for f in dst.bm.faces:
        if f not in before:
            f.material_index = remap[f.material_index]
    bpy.data.meshes.remove(me)


def text(p, m, body, size, loc, extrude=0.06):
    """Letrero 3D convertido a malla (no depende de una fuente instalada). Mira hacia -Y."""
    cu = bpy.data.curves.new("_txt", "FONT")
    cu.body = body
    cu.size = size
    cu.extrude = extrude
    cu.align_x = "CENTER"
    cu.align_y = "CENTER"
    cu.resolution_u = 2
    ob = bpy.data.objects.new("_txt", cu)
    scene.collection.objects.link(ob)
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    me.transform(mtx(loc, rot_x=math.radians(90)))
    i = p.mi(m)
    before = set(p.bm.faces)
    p.bm.from_mesh(me)
    dl = p.bm.faces.layers.int.get("detail")
    for f in p.bm.faces:
        if f not in before:
            f.material_index = i
            f[dl] = 1                 # Letras: detalle, sin contorno propio
    bpy.data.objects.remove(ob)
    bpy.data.curves.remove(cu)
    bpy.data.meshes.remove(me)


def on_sphere(R, cz, x, z, lift=0.03):
    """Punto sobre el hemisferio frontal (-Y) de una esfera de radio R centrada en (0, 0, cz)."""
    return (x, -math.sqrt(max(R * R - x * x - (z - cz) ** 2, 0.0)) - lift, z)


def sphere_decal(p, m, R, cz, pts_fn, nu, nv, lift=0.05):
    """Rejilla nu x nv pegada a la superficie de la esfera. pts_fn(u, v) -> (x, z) con u, v en [0, 1].

    lift separa capas apiladas (contorno, blanco del ojo, pupila) lo suficiente para que no parpadeen
    de lejos (z-fighting).
    """
    bm = p.bm
    grid = [[bm.verts.new(on_sphere(R, cz, *pts_fn(u / nu, v / nv), lift)) for u in range(nu + 1)]
            for v in range(nv + 1)]
    faces = []
    for v in range(nv):
        for u in range(nu):
            faces.append(bm.faces.new((grid[v][u], grid[v][u + 1], grid[v + 1][u + 1], grid[v + 1][u])))
    faces[-1].normal_update()
    if faces[-1].normal.y > 0:
        bmesh.ops.reverse_faces(bm, faces=faces)
    i = p.mi(m)
    for f in faces:
        f.material_index = i
        f.smooth = True


def ellipse(cx, cz, a, b, start=0.0, sweep=math.tau):
    """Elipse (o arco relleno) para sphere_decal: u recorre el ángulo y v el radio."""
    return lambda u, v: (cx + a * v * math.cos(start + sweep * u), cz + b * v * math.sin(start + sweep * u))


def face_sphere(name, R, body, smile, look=(0.0, 0.0)):
    """Edificio-esfera con la cara pintada hacia -Y y base en z=0 (se hunde un poco en el suelo).

    Como en la serie, ojos, pupilas, boca y puerta son planos sobre la esfera, no volúmenes.
    """
    p = Part(name)
    cz = R * 0.9
    p.sphere(body, R, (0, 0, cz), seg=32, rings=18)
    ea, eb, ez, line = R * 0.36, R * 0.29, cz + R * 0.22, R * 0.035
    for sx in (-1, 1):
        ex = sx * R * 0.43
        sphere_decal(p, M["black"], R, cz, ellipse(ex, ez, ea + line, eb + line), 28, 2, lift=0.05)  # contorno
        sphere_decal(p, M["white"], R, cz, ellipse(ex, ez, ea, eb), 28, 2, lift=0.10)
        sphere_decal(p, M["black"], R, cz, ellipse(ex + look[0] * ea * 0.5, ez + look[1] * eb * 0.5,
                                                   R * 0.1, R * 0.09), 14, 1, lift=0.15)
    # Boca: media elipse baja y ancha hacia abajo (sonrisa) o hacia arriba (ceño fruncido)
    if smile:
        sphere_decal(p, M["black"], R, cz, ellipse(0, cz - R * 0.2, R * 0.7, R * 0.3, math.pi, math.pi), 20, 4)
        sphere_decal(p, M["orange"], R, cz, lambda u, v: (-R * 0.08 + R * 0.16 * u, R * 0.12 + R * 0.26 * v), 2, 4)
    else:
        sphere_decal(p, M["black"], R, cz, ellipse(0, cz - R * 0.56, R * 0.5, R * 0.25, 0.0, math.pi), 20, 4)
    return p


def storefront(name, w, d, h, wall, win, rows, cols, arched=False, trim=None):
    """Edificio de calle con fachada hacia -Y y base en z=0."""
    p = Part(name)
    p.box(wall, (w, d, h), (0, 0, h / 2))
    p.box(trim or wall, (w + 0.12, d + 0.12, 0.14), (0, 0, h + 0.05))
    fy = -d / 2 - 0.03
    ww = w / cols * 0.48
    wh = min((h - 1.6) / max(rows, 1) * 0.55, 0.55)
    p.detail = True
    for c in range(cols):
        for r in range(rows):
            x = -w / 2 + w * (c + 0.5) / cols
            z = 1.35 + (h - 1.8) * (r + 0.5) / rows
            for sy in (1, -1):                      # Frente y ventanas traseras (se ven al llegar desde fuera)
                p.box(M["black"], (ww + 0.07, 0.05, wh + 0.07), (x, sy * (fy + 0.015), z))   # marco
                p.box(win, (ww, 0.08, wh), (x, sy * fy, z))
            if arched:
                p.cyl(M["black"], ww / 2 + 0.035, ww / 2 + 0.035, 0.05, (x, fy + 0.015, z + wh / 2), seg=8,
                      rot_x=math.radians(90))
                p.cyl(win, ww / 2, ww / 2, 0.08, (x, fy, z + wh / 2), seg=8, rot_x=math.radians(90))
    p.box(M["black"], (w * 0.62 + 0.07, 0.05, 0.62), (0, fy + 0.015, 0.5))
    p.box(M["glass"], (w * 0.62, 0.08, 0.55), (0, fy, 0.5))
    p.detail = False
    return p


C_SF = collection("Zone_SmilingFriends", C_ZONES)
C_FF = collection("Zone_FrowningFriends", C_ZONES)
city_top = TOP + 0.35

ROAD_Y, ROAD_W = -3.6, 2.2
SF_FRONT, FF_FRONT = ROAD_Y + ROAD_W / 2 + 0.6, ROAD_Y - ROAD_W / 2 - 0.6

# Marco de la ciudad: la calle corre por el eje X; Smiling Friends al norte (+Y) mirando a Frowning (-Y).
# El origen se ubica para que la calle y la esfera amarilla caigan donde están en el boceto.
_rx, _ry = sk(1022, 712)
CITY = mtx((_rx, _ry - ROAD_Y, city_top))

g = Part("SF_City_Ground")
# Isla de la ciudad: rectángulo redondeado (superelipse |x/a|^4 + |y/b|^4 = 1) ceñido a los edificios.
# Como en la serie, el concreto solo cubre el lado de Smiling Friends; del lado de Frowning Friends
# queda una banqueta junto a la calle y detrás, pasto (la tierra del continente).
ISLAND_C, ISLAND_A, ISLAND_B = (0, -3.65), 9.8, 5.1
FF_GROUND = LAND - city_top   # Altura local del pasto: ahí se apoya todo el lado de Frowning Friends
island = [(ISLAND_C[0] + ISLAND_A * math.copysign(abs(math.cos(t)) ** 0.6, math.cos(t)),
           ISLAND_C[1] + ISLAND_B * math.copysign(abs(math.sin(t)) ** 0.6, math.sin(t)))
          for t in (i / 64 * math.tau for i in range(64))]
y_cut = ROAD_Y - ROAD_W / 2 - 0.6   # Borde exterior de la banqueta de Frowning Friends
clipped = []
for a, b in zip(island, island[1:] + island[:1]):   # Recorte de la superelipse (convexa) con y >= y_cut
    if a[1] >= y_cut:
        clipped.append(a)
    if (a[1] >= y_cut) != (b[1] >= y_cut):
        t = (y_cut - a[1]) / (b[1] - a[1])
        clipped.append((a[0] + (b[0] - a[0]) * t, y_cut))
g.prism(M["sidewalk"], [(x, y, -0.35) for x, y in clipped], (0, 0, 0.35))
g.place(CITY)
g.build(C_SF, ROOT)

sf = face_sphere("SF_SmilingFriends_Building", 2.8, M["yellow"], smile=True, look=(-0.3, 0.05))
sf.place(Matrix.Translation((0, SF_FRONT + 2.2, 0)))
sf.place(CITY)
sf.build(C_SF, ROOT)

ff = face_sphere("SF_FrowningFriends_Building", 1.6, M["frown_face"], smile=False, look=(0.35, 0.5))
ff.place(Matrix.Rotation(math.pi, 4, "Z"))
ff.place(Matrix.Translation((-0.4, FF_FRONT - 1.55, FF_GROUND)))
ff.place(CITY)
ff.build(C_FF, ROOT)

# Como en el boceto: a cada lado de cada esfera, tres edificios cada vez más pequeños hacia fuera.
# Lado de Smiling Friends (fachadas hacia -Y local, mirando a la calle)
SF_SIDE = [
    ("RonsDeli", 2.0, 2.2, "ronsdeli", "white", 0, 3, False, "RON'S\nDELI", "ronsdeli_sign", 0.32, -6.3),
    ("Asylum", 2.2, 4.0, "asylum", "asylum_win", 4, 4, False, "ASYLUM", "asylum_sign", 0.45, -4.15),
    ("ToiletEmporium", 2.4, 3.4, "toilet", "toilet_win", 2, 5, True, "TOILET EMPORIUM", "toilet_sign", 0.22, 4.3),
    ("AntiqueStore", 1.4, 2.2, "antique", "window", 0, 2, False, "ANTIQUE\nSTORE", "antique_sign", 0.2, 6.25),
]
# Lado de Frowning Friends (fachadas hacia +Y local)
FF_SIDE = [
    ("Lilac_West", 1.8, 2.3, "lilac", "window", 1, 3, False, None, None, 0, -3.7),
    ("Brick_Frown", 2.0, 2.8, "brick", "white", 2, 3, False, None, None, 0, 2.4),
]
SIGN_Z = {"RonsDeli": 1.6, "Asylum": 3.75, "ToiletEmporium": 1.1, "AntiqueStore": 1.6}
D = 1.8  # Fondo de los edificios de la calle
# Los vecinos de la esfera llegan hasta el muro de ladrillo para que no queden huecos entre ellos
DEPTH = {"Asylum": 3.0, "ToiletEmporium": 3.0}
for side, rows_, front, coll in ((1, SF_SIDE, SF_FRONT, C_SF), (-1, FF_SIDE, FF_FRONT, C_FF)):
    for nm, w, h, wall, win, rows, cols, arched, sign, sign_m, sign_s, x in rows_:
        d = DEPTH.get(nm, D)
        b = storefront("SF_City_" + nm, w, d, h, M[wall], M[win], rows, cols, arched, trim=M["trim"])
        if sign:
            text(b, M[sign_m], sign, sign_s, (0, -d / 2 - 0.08, SIGN_Z[nm]))
        if nm == "RonsDeli":  # Toldo a rayas
            b.detail = True
            for k in range(8):
                b.box(M["awning"] if k % 2 == 0 else M["white"], (w / 8, 0.6, 0.06),
                      (-w / 2 + w * (k + 0.5) / 8, -d / 2 - 0.25, 1.05))
        if side == -1:
            b.place(Matrix.Rotation(math.pi, 4, "Z"))
        b.place(Matrix.Translation((x, front + side * d / 2, 0 if side > 0 else FF_GROUND)))
        b.place(CITY)
        b.build(coll, ROOT)

# Edificios pequeños de los extremos: se construyen a tamaño normal y se escalan enteros
# (lado, x del centro, escala, pared)
SMALL = [(1, -7.95, 0.55, "grayshop"), (1, 7.55, 0.6, "pinkbrick"),
         (-1, -5.35, 0.6, "yellowshop"), (-1, -6.75, 0.42, "grayshop"),
         (-1, 4.0, 0.6, "lilac"), (-1, 5.35, 0.42, "yellowshop")]
for i, (side, x, f, wall) in enumerate(SMALL):
    b = storefront("SF_City_Small_%d" % (i + 1), 1.8, D, 3.4, M[wall], M["window"], 3, 3, trim=M["trim"])
    b.place(Matrix.Diagonal((f, f, f, 1)))
    if side == -1:
        b.place(Matrix.Rotation(math.pi, 4, "Z"))
    front = SF_FRONT if side > 0 else FF_FRONT
    b.place(Matrix.Translation((x, front + side * D * f / 2, 0 if side > 0 else FF_GROUND)))
    b.place(CITY)
    b.build(C_SF if side > 0 else C_FF, ROOT)

props = Part("SF_City_Props")
# Muro delgado al fondo, de ASYLUM (borde x=-3.05) a TOILET EMPORIUM (borde x=3.1)
props.box(M["brick"], (6.15, 0.35, 3.2), (0.025, SF_FRONT + 2.8, 1.6))
props.box(M["dumpster"], (0.75, 0.45, 0.5), (-2.65, SF_FRONT + 0.4, 0.25))
props.box(M["black"], (1.0, 0.45, 0.05), (5.3, SF_FRONT - 0.3, 1.05))
props.box(M["black"], (1.0, 0.04, 0.95), (5.3, SF_FRONT - 0.08, 0.55))
props.box(M["crust"], (0.8, 0.22, 0.08), (5.3, SF_FRONT - 0.25, 0.35))
props.place(CITY)
props.build(C_SF, ROOT)

# Siluetas moradas de la ciudad detrás del lado de Smiling Friends, como el fondo de la serie:
# bloques lisos sin detalles en dos capas, la lejana más clara y alta, para dar sensación de profundidad.
# Tienen grosor para que desde un costado se lean como volúmenes y no como hojas de papel.
brng = random.Random(SEED + 2)
BACKDROP_X, BACKDROP_Y = (-9.0, 9.0), (3.3, 4.6)  # Detrás de todos los edificios (el fondo máximo es 3.0)
BACKDROP_T = 1.2                                   # Grosor de cada capa


def skyline(part, m, y, hmin, hmax, flat=4.5, thick=BACKDROP_T):
    """Silueta de bloques. Hasta |x| = flat la altura es completa; de ahí a los extremos baja hasta un 30 %,
    para que los lados se lean como edificios cada vez más pequeños y no como el corte de un muro."""
    x, base = BACKDROP_X[0], LAND - city_top
    end = BACKDROP_X[1]

    def taper(xm):
        k = max(0.0, (abs(xm) - flat) / (end - flat))
        return 1.0 - 0.7 * min(k, 1.0)

    pts = [(x, y, base)]
    while x < end:
        w = min(brng.uniform(0.8, 1.8), end - x)
        t = taper(x + w / 2)
        h = base + (brng.uniform(hmin, hmax) - base) * t
        pts.append((x, y, h))
        if brng.random() < 0.25 and w > 1.0:      # remate en punta
            pts.append((x + w / 2, y, h + w * 0.45 * t))
        elif brng.random() < 0.35:                # escalón
            pts += [(x + w * 0.5, y, h), (x + w * 0.5, y, h + brng.uniform(0.3, 0.7) * t)]
            h = pts[-1][2]
        pts.append((x + w, y, h))
        if brng.random() < 0.3 and t > 0.6:       # antena con bolita (no en los extremos bajos)
            ax = x + w * brng.uniform(0.3, 0.7)
            part.box(m, (0.08, 0.08, 0.9), (ax, y + thick / 2, h + 0.45))
            part.cyl(m, 0.17, 0.17, 0.12, (ax, y + thick / 2, h + 0.95), seg=12, rot_x=math.radians(90))
        x += w
    pts.append((end, y, base))
    part.prism(m, pts, (0, thick, 0))


backdrop = Part("SF_City_Skyline")
skyline(backdrop, mat("SkylineFar", "#dcb6e0"), SF_FRONT + BACKDROP_Y[1], 4.0, 5.6, flat=4.0)
skyline(backdrop, mat("SkylineNear", "#b783c6"), SF_FRONT + BACKDROP_Y[0], 3.1, 4.4, flat=5.0)
backdrop.place(CITY)
backdrop.build(C_SF, ROOT)

st = Part("SF_City_StreetTrees")
for x, y in ((8.6, SF_FRONT - 0.3), (-8.9, SF_FRONT - 0.3), (1.3, FF_FRONT + 0.3), (-7.6, FF_FRONT + 0.3)):
    st.cyl(M["crust"], 0.07, 0.07, 0.9, (x, y, 0.45), seg=6)
    st.ico(M["leaf"], 0.42, (x, y, 1.15), subdiv=1, scale=(1, 1, 1.15))
st.place(CITY)
st.build(C_SF, ROOT)


def in_city(p, margin=0.0):
    lx, ly, _ = CITY.inverted() @ Vector((p[0], p[1], city_top))
    return ((abs(lx - ISLAND_C[0]) / (ISLAND_A + margin)) ** 4 + (abs(ly - ISLAND_C[1]) / (ISLAND_B + margin)) ** 4) < 1


def in_autumn(p):
    """Franja de pasto detrás del lado de Frowning Friends, donde va el bosque de otoño (como en la serie)."""
    lx, ly, _ = CITY.inverted() @ Vector((p[0], p[1], city_top))
    return abs(lx) < ISLAND_A + 0.8 and -12.0 < ly < y_cut - 0.3


def behind_city(p):
    """Franja de las siluetas moradas (con margen), donde no van árboles."""
    lx, ly, _ = CITY.inverted() @ Vector((p[0], p[1], city_top))
    return BACKDROP_X[0] - 0.8 < lx < BACKDROP_X[1] + 0.8 and SF_FRONT + BACKDROP_Y[0] - 0.8 < ly < SF_FRONT + BACKDROP_Y[1] + BACKDROP_T + 0.8


# ---------------------------------------------------------------------------
# Caminos, puentes y túnel
# ---------------------------------------------------------------------------

C_ROADS = collection("Roads", C_ZONES)
ROAD = resample(smooth([sk(*p) for p in ROAD_SK], 8), 0.55)
# Distancia a lo largo de la carretera, medida desde el extremo oeste de la ciudad (0) hacia Spamtopia.
ROAD_S = [0.0]
for a, b in zip(ROAD, ROAD[1:]):
    ROAD_S.append(ROAD_S[-1] + math.dist(a, b))
_west = next(i for i, q in enumerate(ROAD) if (CITY.inverted() @ Vector((q[0], q[1], city_top))).x <= -ISLAND_A)
ROAD_S = [v - ROAD_S[_west] for v in ROAD_S]
RAINBOW_S = 9.5      # Desde aquí el asfalto se vuelve arcoíris (después del desierto)
RAINBOW_BLEND = 3.0  # Largo de la transición asfalto -> arcoíris


def road_s(p):
    """Posición a lo largo de la carretera del punto de la carretera más cercano a p."""
    return ROAD_S[min(range(len(ROAD)), key=lambda i: (ROAD[i][0] - p[0]) ** 2 + (ROAD[i][1] - p[1]) ** 2)]


def rainbow_mat(i):
    """Asfalto hasta RAINBOW_S; luego arcoíris a lo largo de la carretera (morado, azul, cian, verde, amarillo, naranja)."""
    sv = (ROAD_S[i] + ROAD_S[min(i + 1, len(ROAD_S) - 1)]) / 2
    if sv < RAINBOW_S:
        return M["road"]
    mix = round(min(1.0, (sv - RAINBOW_S) / RAINBOW_BLEND) * 3) / 3     # 3 pasos de mezcla
    hue = round(((0.8 - (sv - RAINBOW_S) / 9.0) % 1.0) * 24) % 24      # 24 tonos
    r, g, b = colorsys.hsv_to_rgb(hue / 24, 0.75, 1.0)
    col = hex_lerp("#77748a", "#%02x%02x%02x" % (round(r * 255), round(g * 255), round(b * 255)), mix)
    return mat("Rainbow_%02d_%d" % (hue, round(mix * 3)), col)


road = Part("SF_Road")
road.ribbon(rainbow_mat, ROAD, ROAD_W, city_top + 0.08, LAND - 0.02, dash=M["white"])
road.build(C_ROADS, ROOT)


def bridge(name, coll, a, b, z0, z1, width, plank, post, arch=0.0):
    """Puente de tablones entre a y b (x, y), con postes a los lados. arch levanta el centro (puente arqueado)."""
    p = Part(name)
    ln = math.dist(a, b)
    ang = math.atan2(b[1] - a[1], b[0] - a[0])
    n = max(2, int(ln / 0.32))
    for i in range(n):
        t = (i + 0.5) / n
        x, y = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
        p.box(plank, (ln / n * 0.8, width, 0.08), (x, y, z0 + (z1 - z0) * t + arch * math.sin(math.pi * t)),
              rot_z=ang)
    for i in range(0, n + 1, 3):
        t = i / n
        x, y = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
        for s in (-1, 1):
            p.box(post, (0.08, 0.08, 0.45), (x - math.sin(ang) * s * width / 2, y + math.cos(ang) * s * width / 2,
                                            z0 + (z1 - z0) * t + arch * math.sin(math.pi * t) + 0.2))
    p.build(coll, ROOT)


def arched_strip(part, m, a, b, n_off, w, z0, h, arch, segs=16):
    """Bloque largo que sigue un arco de a a b: n_off lo corre de lado, w es su ancho y h su alto."""
    ang = math.atan2(b[1] - a[1], b[0] - a[0])
    px, py = -math.sin(ang), math.cos(ang)
    bm, rows = part.bm, []
    for i in range(segs + 1):
        t = i / segs
        cx, cy = a[0] + (b[0] - a[0]) * t + px * n_off, a[1] + (b[1] - a[1]) * t + py * n_off
        zb = z0 + arch * math.sin(math.pi * t)
        rows.append([bm.verts.new((cx + px * sx * w / 2, cy + py * sx * w / 2, zb + dz))
                     for sx, dz in ((-1, 0), (1, 0), (1, h), (-1, h))])
    faces = []
    for r0, r1 in zip(rows, rows[1:]):
        for k in range(4):
            faces.append(bm.faces.new((r0[k], r1[k], r1[(k + 1) % 4], r0[(k + 1) % 4])))
    faces += [bm.faces.new(rows[0][::-1]), bm.faces.new(rows[-1])]
    bmesh.ops.recalc_face_normals(bm, faces=faces)
    i = part.mi(m)
    for f in faces:
        f.material_index = i


def slab_bridge(name, coll, a, b, z, width, arch, deck, rail):
    """Puente de una sola pieza de madera, arqueado, con barandas macizas a los lados."""
    p = Part(name)
    arched_strip(p, deck, a, b, 0.0, width, z - 0.12, 0.12, arch)
    for sx in (-1, 1):
        arched_strip(p, rail, a, b, sx * (width / 2 - 0.05), 0.1, z, 0.28, arch)
    p.build(coll, ROOT)


DEMON_BRIDGE = (sk(1008, 952), sk(1010, 1004))
slab_bridge("SF_Bridge_DemonForest", C_ROADS, *DEMON_BRIDGE, LAND + 0.05, 1.0, 0.45, M["wood"], M["post"])


def off_bridge(p, clear=1.2):
    """Lejos del puente del bosque del demonio, incluidas sus dos llegadas."""
    (ax, ay), (bx, by) = DEMON_BRIDGE
    ln = math.hypot(bx - ax, by - ay)
    ux, uy = (bx - ax) / ln, (by - ay) / ln
    return dist_to_path(p, [(ax - ux * 1.0, ay - uy * 1.0), (bx + ux * 1.0, by + uy * 1.0)]) > clear

# Árboles del continente central: alrededor de la ciudad, sin pisar la calle, la carretera ni la colina
def seg_cyl(part, m, p0, p1, r0, r1, seg=5):
    """Cilindro cónico abierto (sin tapas) de p0 a p1."""
    p0, p1 = Vector(p0), Vector(p1)
    d = p1 - p0
    rot = d.to_track_quat("Z", "Y").to_matrix().to_4x4()
    part.op(m, "create_cone", cap_ends=False, segments=seg, radius1=r0, radius2=r1, depth=d.length or 1e-4,
            matrix=Matrix.Translation((p0 + p1) / 2) @ rot)


def puffy_tree(part, x, y, z, h, leaf, trunk=None):
    """Árbol de caricatura como los de la serie: tronco visible que se abre en dos ramas y una copa de bultos
    sobre cada rama. Variación propia por posición (no consume el azar de la distribución)."""
    tr = trunk or M["crust"]
    v = random.Random(round(x * 1000) * 7919 + round(y * 1000))
    a = v.uniform(0, math.tau)
    ca, sa = math.cos(a), math.sin(a)
    fork = (x + ca * 0.04 * h, y + sa * 0.04 * h, z + 0.45 * h)
    b1 = (fork[0] + ca * 0.22 * h, fork[1] + sa * 0.22 * h, fork[2] + 0.26 * h)
    b2 = (fork[0] - ca * 0.17 * h, fork[1] - sa * 0.17 * h, fork[2] + 0.3 * h)
    seg_cyl(part, tr, (x, y, z), fork, 0.075 * h, 0.05 * h, 5)
    seg_cyl(part, tr, fork, b1, 0.045 * h, 0.022 * h, 4)
    seg_cyl(part, tr, fork, b2, 0.045 * h, 0.022 * h, 4)
    part.sphere(leaf, 0.3 * h, (x + ca * 0.03 * h, y + sa * 0.03 * h, z + 0.82 * h), seg=6, rings=3, scale=(1, 1, 0.85))
    part.sphere(leaf, 0.23 * h, (b1[0], b1[1], b1[2] + 0.08 * h), seg=6, rings=3, scale=(1, 1, 0.85))
    part.sphere(leaf, 0.21 * h, (b2[0], b2[1], b2[2] + 0.07 * h), seg=6, rings=3, scale=(1, 1, 0.85))


central_trees = Part("SF_Central_Trees")
HILL_S = (2.5, 2.5, 3.3)   # Colina redonda y alta (radio, radio, alto), como una casa Shmaloogle
HILLS = (HILL_C, HILL2_C)
for x, y in scatter_poly(rng, 32, LAND_LOGIC["Central"], 0.95,
                         ok=lambda p: not in_city(p, 0.6) and not behind_city(p) and not in_autumn(p)
                         and dist_to_path(p, ROAD) > 1.8 and off_bridge(p) and dist_to_path(p, EAST_ROAD_PTS) > 1.8
                         and ((p[0] - HILL_C[0]) / HILL_S[0]) ** 2 + ((p[1] - HILL_C[1]) / HILL_S[1]) ** 2 > 1.2):
    h = rng.uniform(1.0, 1.5)
    leaf = M["leaf"] if rng.random() < 0.6 else M["leaf2"]
    puffy_tree(central_trees, x, y, LAND, h, leaf)
central_trees.build(C_SF, ROOT)

# ---------------------------------------------------------------------------
# Zonas
# ---------------------------------------------------------------------------

# Bosque de otoño detrás de Frowning Friends: rojos, naranjas y amarillos, sin pisar edificios ni la esfera
arng = random.Random(SEED + 3)
AUTUMN = [mat("AutumnRed", "#d9453a"), mat("AutumnOrange", "#ef8a2c"), mat("AutumnYellow", "#e8c13a"),
          mat("AutumnCrimson", "#b8323a")]


def autumn_free(p):
    lx, ly, _ = CITY.inverted() @ Vector((p[0], p[1], city_top))
    if not in_autumn(p) or ly > FF_FRONT - D - 0.4:             # detrás de la fila de edificios
        return False
    if -2.6 < lx < 1.8 and ly > FF_FRONT - 3.7:                 # la esfera azul
        return False
    return (dist_to_path(p, ROAD) > 1.8 and off_bridge(p)
            and ((p[0] - HILL_C[0]) / HILL_S[0]) ** 2 + ((p[1] - HILL_C[1]) / HILL_S[1]) ** 2 > 1.2)


# Degradado del borde al centro: verde de siempre, lima, amarillo, naranja, rojo y carmesí
GRADIENT = [M["leaf"], M["leaf2"], mat("AutumnLime", "#a9c23f"), AUTUMN[2], AUTUMN[1], AUTUMN[0], AUTUMN[3]]
AUTUMN_CY, AUTUMN_HY = (-12.0 + y_cut - 0.3) / 2, (y_cut - 0.3 + 12.0) / 2   # Centro y semialto de la franja


def warmth(p):
    """1 en el centro de la franja de otoño y 0 en los lados. Hacia el fondo (lejos de la calle) casi no
    se enfría: el centro sigue cálido hasta la orilla y solo los lados pasan a verde."""
    lx, ly, _ = CITY.inverted() @ Vector((p[0], p[1], city_top))
    dy = (ly - AUTUMN_CY) / AUTUMN_HY
    if dy < 0:
        dy *= 0.3
    return max(0.0, 1.0 - math.hypot(lx / (ISLAND_A + 0.8), dy))


autumn = Part("SF_FrowningSide_AutumnTrees")
for x, y in scatter_poly(arng, 45, LAND_LOGIC["Central"], 0.85, ok=autumn_free):
    # El centro apunta entre naranja y rojo (4.5) y el ruido amplio lo mezcla con amarillo y carmesí;
    # en los bordes el mismo ruido mezcla los dos verdes con la lima.
    k = round(warmth((x, y)) * 4.5 + arng.uniform(-1.6, 1.6))
    puffy_tree(autumn, x, y, LAND, arng.uniform(1.1, 1.5), GRADIENT[max(0, min(len(GRADIENT) - 1, k))])
autumn.build(C_FF, ROOT)


def mountains(part, cx, cy, z, body, n, size, snow=None):
    for i in range(n):
        x = cx + (i - (n - 1) / 2) * size * 0.8 + rng.uniform(-0.15, 0.15) * size
        y = cy + rng.uniform(-0.3, 0.3) * size
        h = size * rng.uniform(1.0, 1.6)
        part.cyl(body, size * 0.55, 0.0, h, (x, y, z + h / 2), seg=6, rot_z=rng.uniform(0, 1))
        if snow:
            part.cyl(snow, size * 0.55 * 0.3, 0.0, h * 0.3, (x, y, z + h * 0.85), seg=6, rot_z=rng.uniform(0, 1))


# --- Spamtopia (T2 EP7): desierto, letrero, rey con la multitud y figuras geométricas junto a la carretera ---
C = collection("Zone_Spamtopia", C_ZONES)

# Suelo del noroeste con degradado a lo largo de la carretera: pasto de la ciudad -> arena del desierto ->
# verde brillante de Spamtopia. Cada triángulo toma el color según la posición en la carretera más cercana.
GROUND_STOPS = [(1.5, "#7bc062"), (4.0, "#f2c46b"), (8.5, "#f2c46b"), (11.5, "#8ef08c")]


def ground_mat(sv):
    sv = round(max(GROUND_STOPS[0][0], min(GROUND_STOPS[-1][0], sv)) * 2) / 2   # pasos de 0.5
    for (s0, c0), (s1, c1) in zip(GROUND_STOPS, GROUND_STOPS[1:]):
        if s0 <= sv <= s1:
            return mat("Ground_%04.1f" % sv, hex_lerp(c0, c1, (sv - s0) / (s1 - s0)))


nw = Part("SF_Land_NorthWest")
poly = LANDW["NorthWest"]
vt = [nw.bm.verts.new((x, y, LAND + 0.02)) for x, y in poly]   # sobre la tierra unida
for a, b, c in tessellate_polygon([[Vector((x, y, 0)) for x, y in poly]]):
    f = nw.bm.faces.new((vt[a], vt[b], vt[c]))
    f.normal_update()
    if f.normal.z < 0:
        f.normal_flip()
# Triángulos pequeños para que el degradado sea suave (malla conforme: sin grietas entre triángulos)
bmesh.ops.subdivide_edges(nw.bm, edges=list(nw.bm.edges), cuts=4, use_grid_fill=True)   # la orilla redondeada ya trae más puntos
bmesh.ops.triangulate(nw.bm, faces=list(nw.bm.faces))
# Costados (la orilla), hacia afuera. Usan vértices propios: la subdivisión ya reemplazó los de la tapa.
shore(nw.bm, poly, LAND + 0.02, TOP)
for f in nw.bm.faces:
    c = f.calc_center_median()
    f.material_index = nw.mi(ground_mat(road_s((c.x, c.y))))
nw.build(C_MAP, ROOT)

sign = Part("SF_Spamtopia_Sign")  # Letrero de frente para quien mira el planeta desde la vista por defecto
sign.box(M["white"], (2.2, 0.12, 0.7), (0, 0, 1.5))
sign.box(M["black"], (2.3, 0.1, 0.8), (0, 0.04, 1.5))
for sx in (-1, 1):
    sign.box(M["post"], (0.12, 0.12, 1.15), (sx * 0.75, 0.05, 0.57))
text(sign, M["black"], "SPAMTOPIA", 0.32, (0, -0.07, 1.5))
sx_, sy_ = sk(500, 520)
VIEWER = (0.0, -40.0)   # Punto de vista por defecto: desde el sur, mirando la cara de Smiling Friends
sign.place(mtx((sx_, sy_, LAND), math.atan2(VIEWER[1] - sy_, VIEWER[0] - sx_) + math.pi / 2))
sign.build(C, ROOT)

# Spamtopia (T2 EP7): lo que importa es la sensación de un lugar lleno de edificios psicodélicos, no el
# detalle. Unos pocos edificios fijos de referencia y muchos más repartidos, todos con formas simples y
# colores neón. Semilla propia.
srng = random.Random(SEED + 4)
SP = {k: mat("Spam_" + k, v) for k, v in (("pink", "#ff7ac0"), ("coral", "#ff6b6b"), ("cyan", "#5ee6e0"),
                                          ("yellow", "#ffe066"), ("green", "#5be37a"), ("blue", "#3f8cff"),
                                          ("lamp", "#f6c6e8"), ("ball", "#ff8fd8"), ("mint", "#6be0b0"),
                                          ("lilac", "#c9a3ff"), ("purple", "#b06ae0"), ("lime", "#c6f56a"))}
_central_edge = LAND_LOGIC["Central"] + LAND_LOGIC["Central"][:1]


def spam_only(q, margin=1.3):
    """Solo en Spamtopia: fuera de la tierra central (ciudad, fondo morado y bosque) con un margen."""
    if in_city(q, 1.0) or _near_skyline(q):
        return False                                       # Ni en la ciudad ni bajo el fondo morado
    return not inside(q, LAND_LOGIC["Central"]) and dist_to_path(q, _central_edge) > margin


def _near_skyline(q, pad=1.4):
    lx, ly, _ = CITY.inverted() @ Vector((q[0], q[1], city_top))
    return (BACKDROP_X[0] - pad < lx < BACKDROP_X[1] + pad
            and SF_FRONT + BACKDROP_Y[0] - pad < ly < SF_FRONT + BACKDROP_Y[1] + BACKDROP_T + pad)


sp = Part("SF_Spamtopia_Buildings")
x, y = sk(545, 375)                                   # Triángulo invertido rosa con un triángulo cian al pie
sp.cyl(SP["pink"], 0.35, 1.7, 2.8, (x, y, LAND + 1.4), seg=3, rot_z=0.4)
sp.cyl(SP["cyan"], 1.0, 0.0, 1.3, (x + 0.9, y - 0.7, LAND + 0.65), seg=3, rot_z=1.2)
sp.cyl(SP["mint"], 0.32, 0.32, 0.06, (x - 0.3, y - 0.95, LAND + 1.9), seg=12, rot_x=math.radians(90))  # ventana
x, y = sk(595, 445)                                   # Pirámide amarilla con puerta redonda verde
sp.cyl(SP["yellow"], 1.4, 0.0, 1.9, (x, y, LAND + 0.95), seg=4, rot_z=math.radians(45))
sp.cyl(SP["green"], 0.33, 0.33, 0.08, (x, y - 0.72, LAND + 0.42), seg=14, rot_x=math.radians(90))
x, y = sk(500, 440)                                   # Iglú cian
sp.dome(SP["cyan"], 0.9, (x, y, LAND), seg=14, rings=8, scale=(1, 1, 0.85))
sp.cyl(SP["cyan"], 0.35, 0.35, 0.6, (x, y - 0.8, LAND), seg=10, rot_x=math.radians(90))
x, y = sk(510, 305)                                   # Torre-paleta: poste azul, bola rosa y espiral amarilla
sp.cyl(SP["blue"], 0.1, 0.1, 2.4, (x, y, LAND + 1.2), seg=8)
sp.sphere(SP["pink"], 0.7, (x, y, LAND + 2.9), seg=14, rings=8)
for k in range(14):
    a = k * 0.9
    sp.sphere(SP["yellow"], 0.09, (x + 0.28 * math.cos(a), y + 0.28 * math.sin(a), LAND + 0.3 + k * 0.13), seg=6, rings=4)
x, y = sk(850, 335)                                   # Tubos verdes ondulados
for dx, n in (((-0.3, 5), (0.35, 4)) if spam_only((x, y), 0.9) else ()):
    for k in range(n):
        sp.cyl(SP["green"], 0.2, 0.2, 0.42, (x + dx + 0.12 * math.sin(k * 1.6), y, LAND + 0.21 + k * 0.4), seg=10)
x, y = sk(855, 410)                                   # Picos azules
for dx, dy, h in (((0, 0, 2.6), (0.45, 0.2, 1.9), (-0.4, 0.25, 2.1)) if spam_only((x, y), 0.9) else ()):
    sp.cyl(SP["blue"], 0.26, 0.0, h, (x + dx, y + dy, LAND + h / 2), seg=6)

# Relleno: muchos edificios simples repartidos por el verde de Spamtopia, sin pisar la carretera ni los fijos
NEON_KEYS = ["pink", "coral", "cyan", "yellow", "green", "blue", "lilac", "purple", "lime", "mint"]
SPAM_SPOTS = [sk(*q) for q in ((545, 375), (595, 445), (500, 440), (510, 305), (850, 335), (855, 410))]


def spam_building(part, x, y, f):
    """Edificio psicodélico simple de un tipo al azar; f escala el tamaño."""
    m, m2 = SP[srng.choice(NEON_KEYS)], SP[srng.choice(NEON_KEYS)]
    kind = srng.choice(("pyramid", "inverted", "triangle", "dome", "lollipop", "slab", "spikes", "tower", "tubes",
                        "block", "block", "drum"))
    if kind == "pyramid":
        part.cyl(m, 0.9 * f, 0.0, 1.3 * f, (x, y, LAND + 0.65 * f), seg=4, rot_z=srng.uniform(0, 1.6))
    elif kind == "inverted":
        part.cyl(m, 0.2 * f, 0.7 * f, 1.8 * f, (x, y, LAND + 0.9 * f), seg=srng.choice((3, 4)), rot_z=srng.uniform(0, 2))
    elif kind == "triangle":
        part.cyl(m, 0.9 * f, 0.0, 1.2 * f, (x, y, LAND + 0.6 * f), seg=3, rot_z=srng.uniform(0, 2))
    elif kind == "dome":
        part.dome(m, 0.7 * f, (x, y, LAND), seg=12, rings=8, scale=(1, 1, srng.uniform(0.8, 1.4)))
    elif kind == "lollipop":
        h = srng.uniform(1.4, 2.2) * f
        part.cyl(m2, 0.08 * f, 0.08 * f, h, (x, y, LAND + h / 2), seg=6)
        part.sphere(m, 0.45 * f, (x, y, LAND + h + 0.3 * f), seg=10, rings=6)
    elif kind == "slab":
        h = srng.uniform(1.4, 2.6) * f
        part.box(m, (0.7 * f, 0.5 * f, h), (x, y, LAND + h / 2), rot_z=srng.uniform(0, 1.6))
    elif kind == "spikes":
        for dx, dy in ((0, 0), (0.3, 0.15), (-0.28, 0.2)):
            h = srng.uniform(1.0, 2.0) * f
            part.cyl(m, 0.18 * f, 0.0, h, (x + dx * f, y + dy * f, LAND + h / 2), seg=6)
    elif kind == "tower":
        h = srng.uniform(1.2, 2.0) * f
        part.cyl(m, 0.35 * f, 0.35 * f, h, (x, y, LAND + h / 2), seg=10)
        part.sphere(m2, 0.4 * f, (x, y, LAND + h), seg=10, rings=6)
    elif kind == "block":
        w, d, h = srng.uniform(0.7, 1.3) * f, srng.uniform(0.6, 1.1) * f, srng.uniform(0.5, 1.3) * f
        part.box(m, (w, d, h), (x, y, LAND + h / 2), rot_z=srng.uniform(0, 1.6))
    elif kind == "drum":
        h = srng.uniform(0.4, 0.9) * f
        part.cyl(m, 0.5 * f, 0.5 * f, h, (x, y, LAND + h / 2), seg=12)
        part.cyl(m2, 0.25 * f, 0.25 * f, 0.05, (x, y, LAND + h + 0.02), seg=10)
    else:
        for k in range(srng.randint(3, 5)):
            part.cyl(m, 0.17 * f, 0.17 * f, 0.36 * f, (x + 0.1 * f * math.sin(k * 1.6), y, LAND + (0.18 + k * 0.34) * f),
                     seg=8)


for x, y in scatter_poly(srng, 140, LANDW["NorthWest"], 0.78,
                         ok=lambda q: road_s(q) > 10.0 and dist_to_path(q, ROAD) > 1.4 and spam_only(q)
                         and all(math.dist(q, o) > 1.4 for o in SPAM_SPOTS), tries=300):
    f = srng.uniform(0.45, 0.95)
    if dist_to_path((x, y), ROAD) < ROAD_W / 2 + 0.7 * f:    # los más grandes necesitan más espacio
        continue
    spam_building(sp, x, y, f)
    SPAM_SPOTS.append((x, y))
sp.build(C, ROOT)

# Faroles a lo largo de la carretera arcoíris: poste lila, bola rosa y hojas verdes, alternando lados
lamps = Part("SF_Spamtopia_Lamps")
last, side = -99.0, 1
for i, (x, y) in enumerate(ROAD):
    if ROAD_S[i] < RAINBOW_S + 2.0 or ROAD_S[i] - last < 3.2:
        continue
    j = min(i + 1, len(ROAD) - 1)
    tx, ty = ROAD[j][0] - ROAD[i - 1][0], ROAD[j][1] - ROAD[i - 1][1]
    ln = math.hypot(tx, ty) or 1.0
    px, py = x - ty / ln * 1.7 * side, y + tx / ln * 1.7 * side
    side = -side
    if not inside((px, py), LANDW["NorthWest"]) or not spam_only((px, py))             or any(math.dist((px, py), q) < 1.2 for q in SPAM_SPOTS):
        continue
    last = ROAD_S[i]
    lamps.cyl(SP["lamp"], 0.09, 0.07, 1.3, (px, py, LAND + 0.65), seg=8)
    lamps.sphere(SP["ball"], 0.22, (px, py, LAND + 1.45), seg=10, rings=6)
    for k in range(4):
        a = k * math.pi / 2 + 0.4
        lamps.sphere(SP["green"], 0.3, (px + 0.22 * math.cos(a), py + 0.22 * math.sin(a), LAND + 1.3), seg=6, rings=4,
                     scale=(1, 0.35, 0.18), rot_z=a)
lamps.build(C, ROOT)

# --- Casa de Mr. Frog (T1 EP2): península con un sendero entre montañas y pirámides hasta la cabaña ---
C = collection("Zone_MrFrog", C_ZONES)
# El chiste del episodio: para llegar a su casa, Mr. Frog cruza montañas rocosas, un bosque, montañas de hielo
# y un desierto. La península está dividida en bandas rectas de paisaje, de la ciudad (sur) a la casa (norte).
frng = random.Random(SEED + 11)
FPATH = resample(smooth(FROG_PATH_PTS, 6), 0.3)
_ax0, _ax1 = Vector(FROG_PATH_PTS[0]), Vector(FROG_PATH_PTS[-1])
_axd = (_ax1 - _ax0).normalized()
_axl = (_ax1 - _ax0).length


def frog_t(q):
    """Avance (0..1) a lo largo del eje de la península, de la calle a la casa."""
    return max(0.0, min(1.0, (Vector(q) - _ax0).dot(_axd) / _axl))


# Paisajes (inicio, fin, color); entre uno y otro hay una transición corta en franjas finas
BANDS = [(0.0, 0.47, "#8a8580"), (0.5, 0.61, "#5f9e4a"), (0.64, 0.77, "#eaf2f8"), (0.8, 0.88, "#e8c873"),
         (0.91, 1.0, "#2f6b5e")]


def band_color(t):
    for (a0, a1, c), (b0, b1, d) in zip(BANDS, BANDS[1:]):
        if t <= a1:
            return c
        if t < b0:
            return hex_lerp(c, d, (t - a1) / (b0 - a1))
    return BANDS[-1][2]


fp = Part("SF_Land_FrogPeninsula")
poly = LANDW["FrogPeninsula"]
vt = [fp.bm.verts.new((x, y, LAND + 0.02)) for x, y in poly]   # sobre la tierra unida
for a_, b_, c_ in tessellate_polygon([[Vector((x, y, 0)) for x, y in poly]]):
    f = fp.bm.faces.new((vt[a_], vt[b_], vt[c_]))
    f.normal_update()
    if f.normal.z < 0:
        f.normal_flip()
# Cortes rectos perpendiculares al eje: límites limpios entre paisajes y franjas finas en las transiciones
cuts_t = []
for (a0, a1, _), (b0, b1, _) in zip(BANDS, BANDS[1:]):
    cuts_t += [a1 + (b0 - a1) * k / 6 for k in range(7)]
for t in cuts_t:
    co = _ax0 + _axd * (t * _axl)
    bmesh.ops.bisect_plane(fp.bm, geom=list(fp.bm.verts) + list(fp.bm.edges) + list(fp.bm.faces),
                           plane_co=(co.x, co.y, 0), plane_no=(_axd.x, _axd.y, 0))
bmesh.ops.triangulate(fp.bm, faces=list(fp.bm.faces))
shore(fp.bm, poly, LAND + 0.02, TOP)
for f in fp.bm.faces:
    c = f.calc_center_median()
    col = band_color(frog_t((c.x, c.y)))
    f.material_index = fp.mi(mat("FrogGround_%s" % col[1:], col))
fp.build(C_MAP, ROOT)

# El sendero de tierra empieza en el primer bosque: antes, Mr. Frog tiene que trepar las rocas sin camino
_first = next(i for i, q in enumerate(FPATH) if frog_t(q) >= 0.49)
FPATH_VIS = FPATH[_first:]
path = Part("SF_MrFrog_Path")
path.ribbon(mat("FrogPath", "#8b5a3c"), FPATH_VIS, 0.45, LAND + 0.07)
path.build(C, ROOT)

# Letrero al inicio del camino, de frente a la vista por defecto
sign = Part("SF_MrFrog_Sign")
sign.box(mat("FrogSignWood", "#b07a4a"), (1.5, 0.1, 0.45), (0, 0, 1.0))
sign.box(M["post"], (0.1, 0.1, 1.2), (-0.55, 0.05, 0.6))
sign.box(M["post"], (0.1, 0.1, 1.2), (0.55, 0.05, 0.6))
text(sign, M["black"], "MR. FROG", 0.26, (0, -0.06, 1.0))
_hc = (FPATH[-1][0] + 0.3, FPATH[-1][1] + 0.5)            # Junto a la casa, al lado del camino
_sx, _sy = _hc[0] - 1.9, _hc[1] - 2.0
sign.place(mtx((_sx, _sy, LAND), math.atan2(VIEWER[1] - _sy, VIEWER[0] - _sx) + math.pi / 2))
sign.build(C, ROOT)

fz = Part("SF_MrFrog_Landscape")
ROCK, ROCK2 = mat("FrogRock", "#77726c"), mat("FrogRockDark", "#5d5954")
ICE_M, ICE_M2 = mat("FrogIce", "#a9d6f5"), mat("FrogIce2", "#7fb8e6")
SAND_P, SAND_D = mat("PyramidYellow", "#f2c94c"), mat("FrogDune", "#ddb85e")
CACT = mat("FrogCactus", "#4caf50")
spots = scatter_poly(frng, 115, poly, 0.62, ok=lambda q: dist_to_path(q, FPATH_VIS) > 0.6, tries=200)
n_trees = 0
for x, y in spots:
    t = frog_t((x, y))
    if t < 0.47:                                          # Montañas rocosas: picos grises y peñascos
        if frng.random() < 0.55:
            h = frng.uniform(0.9, 1.8)
            fz.cyl(frng.choice((ROCK, ROCK2)), frng.uniform(0.35, 0.6), 0.05, h, (x, y, LAND + h / 2), seg=5,
                   rot_z=frng.uniform(0, 1))
        else:
            fz.ico(frng.choice((ROCK, ROCK2)), frng.uniform(0.25, 0.4), (x, y, LAND + 0.12), subdiv=1,
                   scale=(1.2, 1, 0.7))
    elif 0.51 < t < 0.6:                                  # Bosque normal: cinco árboles y arbustos
        if n_trees < 5:
            puffy_tree(fz, x, y, LAND, frng.uniform(1.0, 1.3), M["leaf"] if n_trees % 2 else M["leaf2"])
            n_trees += 1
        elif frng.random() < 0.5:
            fz.sphere(mat("Bush", "#6cc04a"), frng.uniform(0.18, 0.28), (x, y, LAND + 0.12), seg=6, rings=4)
    elif 0.65 < t < 0.77:                                 # Montañas de hielo con cima nevada y montículos de nieve
        if frng.random() < 0.65:
            h = frng.uniform(1.1, 2.0)
            r = frng.uniform(0.4, 0.6)
            fz.cyl(frng.choice((ICE_M, ICE_M2)), r, 0.0, h, (x, y, LAND + h / 2), seg=6, rot_z=frng.uniform(0, 1))
            fz.cyl(M["white"], r * 0.42, 0.0, h * 0.42, (x, y, LAND + h * 0.79), seg=6, rot_z=frng.uniform(0, 1))
        else:
            fz.dome(M["white"], frng.uniform(0.25, 0.4), (x, y, LAND), seg=8, rings=4, scale=(1.3, 1, 0.6))
    elif 0.81 < t < 0.88:                                 # Desierto: pirámides, dunas y algún cactus
        roll = frng.random()
        if roll < 0.45:
            sz = frng.uniform(0.4, 0.65)
            fz.cyl(SAND_P, sz, 0.0, sz * 1.4, (x, y, LAND + sz * 0.7), seg=4, rot_z=math.radians(45))
        elif roll < 0.8:
            fz.dome(SAND_D, frng.uniform(0.3, 0.45), (x, y, LAND), seg=8, rings=4, scale=(1.6, 1, 0.45))
        else:
            fz.cyl(CACT, 0.08, 0.07, 0.6, (x, y, LAND + 0.3), seg=6)
            fz.cyl(CACT, 0.05, 0.05, 0.25, (x + 0.12, y, LAND + 0.4), seg=5)
# Bosque del pantano alrededor de la casa: árboles del mismo verde oscuro del suelo
_hc = (FPATH[-1][0] + 0.3, FPATH[-1][1] + 0.5)
SWAMP_TREE = [mat("SwampTree", "#2f6b5e"), mat("SwampTree2", "#285c51")]
for x, y in scatter_poly(frng, 16, poly, 0.75, tries=300,
                         ok=lambda q: frog_t(q) > 0.9 and math.dist(q, _hc) > 2.1 and dist_to_path(q, FPATH) > 0.55
                         and math.dist(q, (_sx, _sy)) > 1.0):
    puffy_tree(fz, x, y, LAND, frng.uniform(1.0, 1.4), SWAMP_TREE[frng.randrange(2)], M["crust"])
fz.build(C, ROOT)

# Casa de Mr. Frog (como en la serie): base redonda color crema, gran techo de escamas verde azulado con una
# planta enorme que le brota encima, ventanas redondas, puerta de arco, raíces de árbol, sobre un estanque
# con nenúfares y luciérnagas.
hut = Part("SF_MrFrog_House")
hx, hy = FPATH[-1][0] + 0.3, FPATH[-1][1] + 0.5
door_dir = math.atan2(FPATH[-6][1] - hy, FPATH[-6][0] - hx)       # La puerta mira al camino que llega
HOUSE = Matrix.Translation((hx, hy, LAND)) @ Matrix.Rotation(door_dir + math.pi / 2, 4, "Z")
h = Part("_house")                                        # Modelo local: puerta hacia -Y, base en z=0
CREAM, ROOF, ROOF2 = mat("FrogHouseCream", "#e6dcc0"), mat("FrogRoof", "#2f8f86"), mat("FrogRoofDark", "#226f69")
LEAF, WOOD, LILY = mat("FrogLeaf", "#3fa38f"), mat("FrogDoor", "#7a5236"), mat("LilyPad", "#3f8f55")
POND, PINK = mat("FrogPond", "#2c6f7a"), mat("LilyFlower", "#f08fb7")
h.cyl(POND, 1.7, 1.7, 0.04, (0, 0, 0.02), seg=20)
for k in range(9):                                        # Nenúfares alrededor
    a_ = k / 9 * math.tau + 0.3
    lx, ly = 1.35 * math.cos(a_), 1.35 * math.sin(a_)
    h.cyl(LILY, 0.18, 0.18, 0.02, (lx, ly, 0.05), seg=8)
    if k % 3 == 0:
        h.sphere(PINK, 0.06, (lx, ly, 0.09), seg=6, rings=3)
h.cyl(CREAM, 0.8, 0.8, 0.65, (0, 0, 0.33), seg=16)       # Base
h.dome(ROOF2, 1.12, (0, 0, 0.62), seg=16, rings=6, scale=(1, 1, 0.55))   # Alero del techo
h.dome(ROOF, 1.0, (0, 0, 0.72), seg=16, rings=6, scale=(1, 1, 0.75))     # Techo de escamas
h.dome(ROOF2, 0.62, (0, 0, 1.15), seg=12, rings=5, scale=(1, 1, 0.65))
h.cyl(LEAF, 0.05, 0.04, 0.8, (0, 0, 1.75), seg=6)         # Planta que brota del techo
for k, (ang, tilt) in enumerate(((0.3, 0.5), (2.4, 0.6), (4.4, 0.45), (1.3, 0.2))):
    h.sphere(LEAF, 0.42 - 0.06 * k, (0.35 * math.cos(ang), 0.35 * math.sin(ang), 1.85 + 0.15 * k), seg=8, rings=4,
             scale=(1.3, 0.55, 0.18), rot_z=ang)
h.cyl(M["metal"], 0.07, 0.07, 0.6, (-0.55, 0.3, 1.35), seg=6)   # Chimenea de tubo
h.detail = True                                           # Puerta de arco y ventanas redondas en cruz
h.box(WOOD, (0.32, 0.06, 0.34), (0, -0.8, 0.17))
h.cyl(WOOD, 0.16, 0.16, 0.06, (0, -0.8, 0.34), seg=10, rot_x=math.radians(90))
for wx_, wz in ((-0.5, 0.36), (0.48, 0.42)):
    h.cyl(M["white"], 0.17, 0.17, 0.06, (wx_, -0.66, wz), seg=12, rot_x=math.radians(90), rot_z=-wx_ * 0.9)
    h.box(WOOD, (0.26, 0.05, 0.03), (wx_, -0.7, wz), rot_z=-wx_ * 0.9)
    h.box(WOOD, (0.03, 0.05, 0.26), (wx_, -0.7, wz), rot_z=-wx_ * 0.9)
h.detail = False
for sx in (-1, 1):                                        # Raíces de árbol abrazando la casa
    seg_cyl(h, M["crust"], (sx * 0.95, -0.25, 0.0), (sx * 0.78, -0.4, 0.7), 0.1, 0.06, 6)
    seg_cyl(h, M["crust"], (sx * 0.78, -0.4, 0.7), (sx * 0.55, -0.55, 0.95), 0.06, 0.03, 5)
h.sphere(mat("FrogGreen", "#6abf3a"), 0.3, (1.15, -0.95, 0.3), seg=10, rings=6)   # Mr. Frog (marcador), al lado de la puerta
h.place(HOUSE)
merge(hut, h)
for k in range(6):                                        # Luciérnagas
    a_ = frng.uniform(0, math.tau)
    hut.sphere(mat("Firefly", "#ffe36a", emit=5.0), 0.05,
               (hx + 1.6 * math.cos(a_), hy + 1.6 * math.sin(a_), LAND + frng.uniform(1.0, 2.2)), seg=6, rings=3)
hut.build(C, ROOT)

# --- Colina, Shmaloogleville (T3 EP7) y castillo del mago ---
C = collection("Zone_Shmaloogleville", C_ZONES)


def hill_z(x, y):
    """Altura de la colina: cúpula redondeada que baja suave hasta el suelo (la carretera puede subirla)."""
    d2 = min(((x - hc[0]) / HILL_S[0]) ** 2 + ((y - hc[1]) / HILL_S[1]) ** 2 for hc in HILLS)
    return LAND + HILL_S[2] * max(0.0, 1 - d2) ** 1.2


# La carretera continúa sobre las dos colinas y entra al pueblo por la puerta del noreste
def on_wall(t, dr=0.0):
    return SH_C[0] + (SH_A + dr) * math.cos(t), SH_C[1] + (SH_B + dr) * math.sin(t)


gate = on_wall(GATE_T, -0.6)
hill_road = resample(EAST_ROAD_PTS, 0.25)
for _ in range(4):                                       # Redondea solo las esquinas (los tramos siguen rectos)
    hill_road = [hill_road[0]] + [((a[0] + 2 * b[0] + c[0]) / 4, (a[1] + 2 * b[1] + c[1]) / 4)
                                  for a, b, c in zip(hill_road, hill_road[1:], hill_road[2:])] + [hill_road[-1]]
HR_W = 1.3
_hs = [0.0]
for a, b in zip(hill_road, hill_road[1:]):
    _hs.append(_hs[-1] + math.dist(a, b))
HR_WIDTHS = [ROAD_W + (HR_W - ROAD_W) * min(1.0, max(0.0, (v - 1.0) / 1.2)) for v in _hs]   # Empalme con la principal


def deck_road(part, m, wall_m, dash, pts, widths):
    """Camino sobre las colinas sin deformar nada: cada tramo es plano de lado a lado, apoyado en lo más alto de
    la colina bajo su ancho, con costados que bajan hasta el terreno (como un terraplén). La colina queda lisa."""
    bm, rows = part.bm, []
    for i, (x, y) in enumerate(pts):
        a, b = pts[max(i - 1, 0)], pts[min(i + 1, len(pts) - 1)]
        ln = math.dist(a, b) or 1.0
        nx, ny = -(b[1] - a[1]) / ln, (b[0] - a[0]) / ln
        w = widths[i]
        top = max(hill_z(x + nx * o, y + ny * o) for o in (-w / 2, -w / 4, 0, w / 4, w / 2)) + 0.04
        rows.append([x, y, nx, ny, w, max(top, city_top + 0.08)])
    zs = [r[5] for r in rows]
    for _ in range(3):                                   # Suaviza sin bajar nunca de lo necesario
        zs = [max(zs[i], (zs[max(i - 1, 0)] + zs[i] + zs[min(i + 1, len(zs) - 1)]) / 3) for i in range(len(zs))]
    verts = []
    for (x, y, nx, ny, w, _), z in zip(rows, zs):
        L, Rr = (x + nx * w / 2, y + ny * w / 2), (x - nx * w / 2, y - ny * w / 2)
        verts.append((bm.verts.new((*L, z)), bm.verts.new((*Rr, z)),
                      bm.verts.new((*L, min(z - 0.05, hill_z(*L) - 0.06))),
                      bm.verts.new((*Rr, min(z - 0.05, hill_z(*Rr) - 0.06)))))
    for i in range(len(rows) - 1):
        (lt0, rt0, lb0, rb0), (lt1, rt1, lb1, rb1) = verts[i], verts[i + 1]
        nx, ny = rows[i][2], rows[i][3]
        for quad, m_, want in (((lt0, rt0, rt1, lt1), m, (0, 0, 1)), ((lb0, lt0, lt1, lb1), wall_m, (nx, ny, 0)),
                               ((rt0, rb0, rb1, rt1), wall_m, (-nx, -ny, 0))):
            f = bm.faces.new(quad)
            f.normal_update()
            if f.normal.dot(Vector(want)) < 0:
                f.normal_flip()
            f.material_index = part.mi(m_)
    for i in range(0, len(rows) - 1, 2):                 # Línea central discontinua
        x0, y0, nx, ny = rows[i][:4]
        x1, y1 = rows[i + 1][:2]
        q = [(x0 - nx * 0.06, y0 - ny * 0.06, zs[i]), (x1 - nx * 0.06, y1 - ny * 0.06, zs[i + 1]),
             (x1 + nx * 0.06, y1 + ny * 0.06, zs[i + 1]), (x0 + nx * 0.06, y0 + ny * 0.06, zs[i])]
        f = bm.faces.new([bm.verts.new((qx, qy, qz + 0.03)) for qx, qy, qz in q])
        f.normal_update()
        if f.normal.z < 0:
            f.normal_flip()
        f.material_index = part.mi(dash)


def cut_z(x, y, base=False):
    """Colina lisa: ya no la cruza ningún camino."""
    return hill_z(x, y)


def terrain_ribbon(part, m, dash, pts, width, zf, lift=0.07, across=4, flat=False):
    """Camino que sigue el terreno también a lo ancho: rejilla de `across` divisiones, cada vértice a zf(x, y)."""
    bm, rows = part.bm, []
    for i, (x, y) in enumerate(pts):
        a, b = pts[max(i - 1, 0)], pts[min(i + 1, len(pts) - 1)]
        tx, ty = b[0] - a[0], b[1] - a[1]
        ln = math.hypot(tx, ty) or 1.0
        nx, ny = -ty / ln, tx / ln
        row = []
        for k in range(across + 1):
            o = (k / across - 0.5) * (width[i] if isinstance(width, list) else width)
            qx, qy = x + nx * o, y + ny * o
            row.append(bm.verts.new((qx, qy, zf(i) if flat else zf(qx, qy) + lift)))
        rows.append((row, nx, ny))
    faces = [bm.faces.new((r0[k], r0[k + 1], r1[k + 1], r1[k]))
             for (r0, _, _), (r1, _, _) in zip(rows, rows[1:]) for k in range(across)]
    for i in range(0, len(pts) - 1, 2):                    # Línea central discontinua
        (x0, y0), (x1, y1) = pts[i], pts[i + 1]
        _, nx, ny = rows[i]
        q = [(x0 - nx * 0.06, y0 - ny * 0.06), (x1 - nx * 0.06, y1 - ny * 0.06), (x1 + nx * 0.06, y1 + ny * 0.06),
             (x0 + nx * 0.06, y0 + ny * 0.06)]
        f = bm.faces.new([bm.verts.new((qx, qy, (zf(i + (j in (1, 2))) if flat else zf(qx, qy) + lift) + 0.012))
                          for j, (qx, qy) in enumerate(q)])
        f.material_index = part.mi(dash)
    for f in faces:
        f.material_index = part.mi(m)
    for f in bm.faces:
        f.normal_update()
        if f.normal.z < 0:
            f.normal_flip()


for hc, hname in ((HILL_C, "SF_Shmaloogle_Hill"), (HILL2_C, "SF_Shmaloogle_Hill2")):
    hill = Part(hname)
    NH, RH = 40, 14   # Cúpula lisa (sombreado suave): no necesita más resolución
    rings_ = []
    for k in range(RH):
        f_ = 1 - k / RH
        rings_.append([hill.bm.verts.new((hc[0] + HILL_S[0] * f_ * math.cos(t), hc[1] + HILL_S[1] * f_ * math.sin(t),
                                          cut_z(hc[0] + HILL_S[0] * f_ * math.cos(t),
                                                hc[1] + HILL_S[1] * f_ * math.sin(t), base=(k == 0))))
                       for t in (i / NH * math.tau for i in range(NH))])
    top_v = hill.bm.verts.new((*hc, cut_z(*hc)))
    hf = []
    for k in range(RH - 1):
        for i in range(NH):
            hf.append(hill.bm.faces.new((rings_[k][i], rings_[k][(i + 1) % NH], rings_[k + 1][(i + 1) % NH],
                                         rings_[k + 1][i])))
    for i in range(NH):
        hf.append(hill.bm.faces.new((rings_[-1][i], rings_[-1][(i + 1) % NH], top_v)))
    for f in hf:
        f.normal_update()
        if f.normal.z < 0:
            f.normal_flip()
        f.material_index = hill.mi(M["hill"])
        f.smooth = True
    hill.build(C, ROOT)
hr = Part("SF_Road_East")
deck_road(hr, M["road"], mat("RoadSide", "#5d5a6e"), M["white"], hill_road, HR_WIDTHS)
hr.build(C_ROADS, ROOT)

village = Part("SF_Shmaloogle_Village")
village.prism(mat("ShmaloogleGrass", "#9be05a"),
              [(SH_C[0] + SH_A * math.cos(t), SH_C[1] + SH_B * math.sin(t), LAND)
               for t in (i / 40 * math.tau for i in range(40))], (0, 0, 0.04), bottom=False)
# Muro color ciruela alrededor del pueblo
wall_m, NW = village.mi(mat("PlumWall", "#8e4585")), 56
GATE_DT = (HR_W / 2 + 0.35) / math.hypot(SH_A * math.sin(GATE_T), SH_B * math.cos(GATE_T))
vw = []
for i in range(NW):
    t = i / NW * math.tau
    c, s = math.cos(t), math.sin(t)
    vw.append([village.bm.verts.new((SH_C[0] + (SH_A + dr) * c, SH_C[1] + (SH_B + dr) * s, z))
               for dr, z in ((-0.18, LAND), (-0.18, LAND + 0.45), (0.18, LAND + 0.45), (0.18, LAND))])   # muro bajo
wall_faces = []
for i in range(NW):
    if abs(((i + 0.5) / NW * math.tau - GATE_T + math.pi) % math.tau - math.pi) < GATE_DT:
        continue                                          # Entrada del pueblo, donde llega la carretera
    a, b = vw[i], vw[(i + 1) % NW]
    for k in range(3):
        wall_faces.append(village.bm.faces.new((a[k], b[k], b[k + 1], a[k + 1])))
for f in wall_faces:
    f.material_index = wall_m
bmesh.ops.recalc_face_normals(village.bm, faces=wall_faces)
# Arco de entrada: dos pilares y un medio arco sobre la carretera
plum = mat("PlumWall", "#8e4585")
ga, gb = on_wall(GATE_T - GATE_DT), on_wall(GATE_T + GATE_DT)
for q in (ga, gb):
    village.cyl(plum, 0.22, 0.22, 0.8, (q[0], q[1], LAND + 0.4), seg=8)
half = math.dist(ga, gb) / 2
mid = ((ga[0] + gb[0]) / 2, (ga[1] + gb[1]) / 2)
ux, uy = (gb[0] - ga[0]) / (2 * half), (gb[1] - ga[1]) / (2 * half)
arc = [(mid[0] - ux * half * math.cos(th), mid[1] - uy * half * math.cos(th), LAND + 0.8 + half * 0.85 * math.sin(th))
       for th in (k / 10 * math.pi for k in range(11))]
for a_, b_ in zip(arc, arc[1:]):
    seg_cyl(village, plum, a_, b_, 0.15, 0.15, 8)
# Pueblo como en T3 EP7: casas altas redondeadas (gomitas) en colores pastel con ventanas redondas en cruz y
# puertas de arco, tallos morados en espiral, camino de arena amarilla con manchas naranjas y Shmaloogles.
vrng = random.Random(SEED + 9)
HOUSE_COLS = [mat("ShHouseGreen", "#86e07a"), mat("ShHouseOrange", "#f3a640"), mat("ShHousePink", "#f293bd"),
              mat("ShHouseLime", "#cde65c"), mat("ShHouseCoral", "#f2735e")]
WIN_RIM, WIN_IN = mat("ShWindowRim", "#8e4fb5"), mat("ShWindowGlass", "#c9a7e6")
DOOR, STALK = mat("ShDoor", "#e2506c"), mat("ShStalk", "#9b59c9")


def facing(x, y):
    return math.atan2(VIEWER[1] - y, VIEWER[0] - x) + math.pi / 2   # Fachada hacia la vista por defecto


def gumdrop(part, x, y, r, h, m):
    """Casa en forma de gomita: cilindro y cúpula alargada encima. Devuelve la altura de la cima."""
    part.cyl(m, r, r, h, (x, y, LAND + h / 2), seg=14)
    part.dome(m, r, (x, y, LAND + h), seg=14, rings=8, scale=(1, 1, 0.9))
    return LAND + h + r * 0.9


def spiral_stalk(part, base, height, size, turn):
    """Tallo morado que sube y termina en un rizo (espiral), como en la serie."""
    pts = [Vector(base), Vector(base) + Vector((0, 0, height))]
    top = pts[-1]
    for k in range(1, 13):
        a = k / 12 * math.tau * 1.15
        rad = size * (1 - k / 15)
        pts.append(top + Vector((turn * (rad - rad * math.cos(a)), 0, rad * math.sin(a))))
    part.detail = True                                    # Sin contorno: deformaba los rizos
    for a_, b_ in zip(pts, pts[1:]):
        seg_cyl(part, STALK, a_, b_, 0.07, 0.06, 5)
    part.detail = False


houses_xy = []
for x, y in scatter_ellipse(vrng, 30, SH_C[0], SH_C[1], SH_A - 1.0, SH_B - 1.0, 1.55):
    if dist_to_path((x, y), [gate, SH_C]) < 1.05 or len(houses_xy) >= 7:
        continue                                          # Deja libre el camino de la entrada
    houses_xy.append((x, y))
for i, (x, y) in enumerate(houses_xy):
    m = HOUSE_COLS[i % len(HOUSE_COLS)]
    r, h = vrng.uniform(0.5, 0.65), vrng.uniform(0.9, 1.5)
    top = gumdrop(village, x, y, r, h, m)
    rot = facing(x, y)
    fw = Matrix.Translation((x, y, 0)) @ Matrix.Rotation(rot, 4, "Z")
    if vrng.random() < 0.5:                               # Segundo bulto más bajo al costado (casa doble)
        sx, sy, _ = fw @ Vector((r * 0.9, 0.1, 0))
        gumdrop(village, sx, sy, r * 0.75, h * 0.6, m)
    village.detail = True
    # Puerta de arco y ventana redonda en cruz, en la cara que mira al espectador
    door = Part("_d")
    door.box(DOOR, (0.3, 0.06, 0.32), (0, -r - 0.01, LAND + 0.16))
    door.cyl(DOOR, 0.15, 0.15, 0.06, (0, -r - 0.01, LAND + 0.32), seg=10, rot_x=math.radians(90))
    wz = LAND + h * 0.85
    door.cyl(WIN_RIM, 0.17, 0.17, 0.06, (0, -r - 0.01, wz), seg=12, rot_x=math.radians(90))
    door.cyl(WIN_IN, 0.12, 0.12, 0.06, (0, -r - 0.03, wz), seg=12, rot_x=math.radians(90))
    door.box(WIN_RIM, (0.24, 0.05, 0.035), (0, -r - 0.05, wz))
    door.box(WIN_RIM, (0.035, 0.05, 0.24), (0, -r - 0.05, wz))
    door.place(fw)
    door.detail = True
    merge(village, door)
    village.detail = False
    if i % 2 == 0:                                        # Tallo en espiral que sale de la cima
        spiral_stalk(village, (x, y, top - 0.1), vrng.uniform(0.4, 0.7), vrng.uniform(0.25, 0.35),
                     vrng.choice((-1, 1)))
for k in range(3):                                        # Tallos sueltos junto al muro
    t = GATE_T + math.pi * (0.55 + 0.45 * k)
    bx, by = on_wall(t, -0.6)
    spiral_stalk(village, (bx, by, LAND), vrng.uniform(0.9, 1.3), 0.3, vrng.choice((-1, 1)))

# Camino de arena amarilla desde la entrada hasta el centro, con manchas naranjas (sin contorno)
SAND, SPOT = mat("ShSand", "#f6dc7c"), mat("ShSpot", "#f1a25c")
path = Part("SF_Shmaloogle_Path")
path.ribbon(SAND, resample(smooth([on_wall(GATE_T, 0.1), ((gate[0] + SH_C[0]) / 2 + 0.3, (gate[1] + SH_C[1]) / 2), SH_C], 6), 0.3),
            1.0, LAND + 0.1)
path.cyl(SAND, 1.0, 1.0, 0.01, (SH_C[0], SH_C[1], LAND + 0.1), seg=20)
for _ in range(9):
    a = vrng.uniform(0, math.tau)
    d = vrng.uniform(0, 0.8)
    path.cyl(SPOT, vrng.uniform(0.12, 0.25), 0.2, 0.01, (SH_C[0] + d * math.cos(a), SH_C[1] + d * math.sin(a),
                                                         LAND + 0.13), seg=8, scale=(1.4, 1, 1))
path.build(C, ROOT)

# Shmaloogles: cuerpo azul redondo, panza blanca, hocico grande y ojitos (todos miran a la vista por defecto)
BLUE, BELLY = mat("ShmaloogleBlue", "#5fc3f4"), mat("ShmaloogleBelly", "#f4fbff")
crowd = Part("SF_Shmaloogles")
spots_ = [q for q in scatter_ellipse(vrng, 60, SH_C[0], SH_C[1], SH_A - 0.45, SH_B - 0.45, 0.6)
          if all(math.dist(q, hq) > 0.95 for hq in houses_xy)][:16]
for x, y in spots_:
    sc = vrng.uniform(0.85, 1.15)
    b = Part("_s")
    b.sphere(BLUE, 0.22, (0, 0, 0.21), seg=8, rings=5, scale=(1, 0.9, 0.95))
    b.sphere(BLUE, 0.13, (0, -0.2, 0.3), seg=6, rings=4, scale=(1.4, 1.1, 0.8))       # hocico
    b.sphere(BELLY, 0.1, (0, -0.17, 0.13), seg=6, rings=4, scale=(1, 0.5, 0.9))       # panza
    for ex in (-0.07, 0.07):
        b.sphere(M["white"], 0.045, (ex, -0.17, 0.42), seg=6, rings=3)
        b.sphere(M["black"], 0.02, (ex, -0.21, 0.42), seg=4, rings=3)
    b.place(Matrix.Translation((x, y, LAND + 0.04)) @ Matrix.Rotation(facing(x, y) + vrng.uniform(-0.6, 0.6), 4, "Z")
            @ Matrix.Diagonal((sc, sc, sc, 1)))
    merge(crowd, b)
crowd.build(C, ROOT)
village.build(C, ROOT)


C = collection("Zone_WizardCastle", C_ZONES)
castle = Part("SF_WizardCastle")
wx, wy = CASTLE_C
cwall, croof, crock = mat("CastleWall", "#3d3548"), mat("CastleRoof", "#211c29"), mat("CastleRock", "#4a4a52")
cglow = mat("CastleGlow", "#b46bff", emit=2.5)
crng = random.Random(SEED + 10)
_jx, _jy = br_rot(sk(1325 + 11 * BR_PX, 1200 + 8.5 * BR_PX))                               # Cristo Redentor (Río de Janeiro)
face = math.atan2(_jy - wy, _jx - wx)                   # Dirección hacia la estatua
fx, fy = math.cos(face), math.sin(face)
for k in range(9):                                      # Base de roca dentada
    a = k / 9 * math.tau + crng.uniform(-0.2, 0.2)
    d = crng.uniform(0.9, 1.4)
    h = crng.uniform(0.5, 1.1)
    castle.cyl(crock, crng.uniform(0.35, 0.55), 0.0, h, (wx + d * math.cos(a), wy + d * math.sin(a), LAND + h / 2), seg=5)
castle.cyl(cwall, 0.95, 0.75, 1.1, (wx, wy, LAND + 0.55), seg=7)          # Cuerpo
castle.cyl(croof, 0.5, 0.0, 4.2, (wx, wy, LAND + 1.1 + 2.1), seg=6)      # Aguja central, muy alta
for k in range(8):                                      # Agujas alrededor, inclinadas hacia fuera
    a = k / 8 * math.tau + 0.3
    d = 0.7
    h = crng.uniform(1.6, 3.0)
    castle.cyl(croof if k % 2 else cwall, crng.uniform(0.2, 0.3), 0.0, h,
               (wx + d * math.cos(a), wy + d * math.sin(a), LAND + 0.9 + h / 2), seg=5,
               rot_z=a + math.pi / 2, rot_x=0.18)
for k in range(5):                                      # Púas pequeñas en el muro
    a = face + (k - 2) * 0.55
    castle.cyl(croof, 0.12, 0.0, 0.6, (wx + 0.9 * math.cos(a), wy + 0.9 * math.sin(a), LAND + 1.3), seg=4,
               rot_z=a + math.pi / 2, rot_x=0.5)
castle.detail = True                                    # Puerta en arco y ventanas brillantes hacia la estatua
door_c = (wx + fx * 0.9, wy + fy * 0.9)
castle.box(cglow, (0.4, 0.06, 0.55), (door_c[0], door_c[1], LAND + 0.28), rot_z=face + math.pi / 2)
castle.cyl(cglow, 0.2, 0.2, 0.06, (door_c[0], door_c[1], LAND + 0.55), seg=8, rot_z=face + math.pi / 2,
           rot_x=math.radians(90))
for z in (1.9, 2.6, 3.2):
    castle.box(cglow, (0.12, 0.05, 0.22), (wx + fx * (0.5 - 0.1 * (z - 1.9)), wy + fy * (0.5 - 0.1 * (z - 1.9)),
                                            LAND + z), rot_z=face + math.pi / 2)
castle.detail = False
castle.build(C, ROOT)

# --- Enchanted Forest (T1 EP6): isla con árboles verdes y magenta y el castillo amarillo ---
C = collection("Zone_EnchantedForest", C_ZONES)
# Enchanted Forest (T1 EP6), como en la serie: árboles grandes de troncos rosas, lilas y azul marino con copas
# moradas, magenta, verde oscuro y azul; arbustos rosas, hongos rojos con puntos y hongos morados, flores
# rosas. Es un parque de la tierra central, separado solo por una cerca de alambre.
erng = random.Random(SEED + 12)
EPOLY = LAND_LOGIC["Enchanted"]
ef = Part("SF_Enchanted_Forest")
ef.prism(mat("EnchantedGround", "#93e36a"), [(x, y, LAND + 0.03) for x, y in EPOLY], (0, 0, 0.03), bottom=False)
mx, my = sk(590, 965)
TRUNKS = [mat("EnchTrunkPink", "#e98ac4"), mat("EnchTrunkLilac", "#9a86d8"), mat("EnchTrunkNavy", "#2c3e6b"),
          mat("EnchTrunkCoral", "#e88a7a")]
CANOPY = [mat("EnchPurple", "#8a3fb5"), mat("EnchMagenta", "#c94fc0"), mat("EnchDarkGreen", "#2f7a52"),
          mat("EnchNavy", "#2f4f8f"), mat("EnchTeal", "#3fa59a"), mat("EnchPlum", "#6b2f5a")]
for x, y in scatter_poly(erng, 30, EPOLY, 0.8, tries=300,
                         ok=lambda q: math.dist(q, (mx, my)) > 1.6 and dist_to_path(q, EPOLY + EPOLY[:1]) > 0.45):
    puffy_tree(ef, x, y, LAND + 0.03, erng.uniform(1.25, 1.75), CANOPY[erng.randrange(len(CANOPY))],
               TRUNKS[erng.randrange(len(TRUNKS))])
MUSH_RED, MUSH_DOT = mat("MushRed", "#e0443e"), M["white"]
MUSH_PURPLE, STEM = mat("MushPurple", "#9a3fc0"), mat("MushStem", "#f2e6d8")
BUSHES = [mat("EnchBushPink", "#f08ad0"), mat("EnchBushTeal", "#46c9b5"), mat("EnchBushBlue", "#4f78d6"),
          mat("EnchBushLilac", "#b58ae8"), mat("EnchBushMint", "#7fe0a0"), mat("EnchBushCoral", "#f2877a")]
# Borde encantado: una franja continua de arbustos de colores y plantas altas azules rodea el bosque
for i, (x, y) in enumerate(resample(EPOLY + EPOLY[:1], 0.42)):
    cxp = sum(q[0] for q in EPOLY) / len(EPOLY)
    cyp = sum(q[1] for q in EPOLY) / len(EPOLY)
    d = math.hypot(cxp - x, cyp - y) or 1.0
    x, y = x + (cxp - x) / d * 0.35, y + (cyp - y) / d * 0.35          # Un poco hacia adentro
    if i % 5 == 2:                                        # Planta alta de hojas azules (sin contorno: hojas finas)
        ef.detail = True
        for k in range(4):
            a_ = k * 1.6
            seg_cyl(ef, mat("EnchLeafBlue", "#3f6fc9"), (x, y, LAND + 0.03),
                    (x + 0.18 * math.cos(a_), y + 0.18 * math.sin(a_), LAND + 0.55), 0.05, 0.0, 4)
        ef.detail = False
    else:
        ef.sphere(BUSHES[(i * 7 + erng.randrange(3)) % len(BUSHES)], erng.uniform(0.22, 0.32),
                  (x, y, LAND + 0.13), seg=6, rings=3, scale=(1.2, 1.2, 0.85))
for x, y in scatter_poly(erng, 30, EPOLY, 0.45, tries=300,
                         ok=lambda q: math.dist(q, (mx, my)) > 1.4 and dist_to_path(q, EPOLY + EPOLY[:1]) > 0.3):
    z = LAND + 0.03
    kind = erng.random()
    if kind < 0.35:                                       # Hongo rojo con puntos blancos
        hh = erng.uniform(0.18, 0.3)
        ef.cyl(STEM, 0.04, 0.04, hh, (x, y, z + hh / 2), seg=6)
        ef.dome(MUSH_RED, 0.13, (x, y, z + hh), seg=8, rings=3, scale=(1, 1, 0.7))
        ef.sphere(MUSH_DOT, 0.025, (x + 0.05, y, z + hh + 0.07), seg=4, rings=2)
    elif kind < 0.5:                                      # Hongo morado grande
        hh = erng.uniform(0.3, 0.45)
        ef.cyl(STEM, 0.06, 0.05, hh, (x, y, z + hh / 2), seg=6)
        ef.dome(MUSH_PURPLE, 0.22, (x, y, z + hh), seg=8, rings=3, scale=(1, 1, 0.6))
    else:                                                 # Arbustos de colores
        ef.sphere(BUSHES[erng.randrange(len(BUSHES))], erng.uniform(0.18, 0.3), (x, y, z + 0.1), seg=6, rings=3,
                  scale=(1.2, 1, 0.8))
gold = mat("MipCastle", "#f0cf5a")
ef.box(gold, (1.6, 1.0, 1.0), (mx, my, LAND + 0.5))
for dx, h in ((-0.75, 1.5), (0.0, 2.1), (0.75, 1.3)):
    ef.box(gold, (0.45, 0.45, h), (mx + dx, my, LAND + h / 2))
    for cx_ in (-0.15, 0.15):
        ef.box(gold, (0.12, 0.45, 0.16), (mx + dx + cx_, my, LAND + h + 0.08))  # almenas
# Cerca de alambre del parque: sigue el borde del bosque solo donde hay tierra del otro lado (no en la costa)
fence = Part("SF_Enchanted_Fence")
fence.detail = True
FENCE_M = mat("FenceMetal", "#9aa0a8")
_ring = resample(EPOLY + EPOLY[:1], 0.6)
_ccw = sum(a_[0] * b_[1] - b_[0] * a_[1] for a_, b_ in zip(EPOLY, EPOLY[1:] + EPOLY[:1])) > 0
_fence_pts = []
for i in range(len(_ring) - 1):
    (ax, ay), (bx, by) = _ring[i], _ring[i + 1]
    ln = math.hypot(bx - ax, by - ay) or 1.0
    ox, oy = ((by - ay) / ln, -(bx - ax) / ln) if _ccw else (-(by - ay) / ln, (bx - ax) / ln)   # Hacia afuera
    mid = ((ax + bx) / 2 + ox * 0.6, (ay + by) / 2 + oy * 0.6)
    _fence_pts.append(inside(mid, LANDW["Central"]))
for i, on in enumerate(_fence_pts):
    if not on:
        continue
    a_, b_ = _ring[i], _ring[i + 1]
    fence.cyl(FENCE_M, 0.035, 0.035, 0.6, (a_[0], a_[1], LAND + 0.33), seg=5)          # Poste
    for zw in (0.25, 0.45, 0.6):                                                       # Alambres
        seg_cyl(fence, FENCE_M, (a_[0], a_[1], LAND + zw), (b_[0], b_[1], LAND + zw), 0.012, 0.012, 4)
    if i + 1 < len(_fence_pts) and not _fence_pts[i + 1]:
        fence.cyl(FENCE_M, 0.035, 0.035, 0.6, (b_[0], b_[1], LAND + 0.33), seg=5)
ef.build(C, ROOT)
fence.build(C, ROOT)

# --- Bosque del Forest Demon (T1 EP4): isla de árboles morados con el demonio al centro ---
C = collection("Zone_DemonForest", C_ZONES)


def desaturate(hexcol, k=0.55, dark=0.85):
    """Mezcla el color con su gris (k) y lo oscurece un poco: tonos apagados del bosque del demonio."""
    r, g, b = [int(hexcol[i:i + 2], 16) for i in (1, 3, 5)]
    lum = 0.3 * r + 0.59 * g + 0.11 * b
    return "#%02x%02x%02x" % tuple(round((c + (lum - c) * k) * dark) for c in (r, g, b))


# Suelo: pasto gris en el interior y arena gris en la orilla, con un degradado corto entre ambos
DEMON_GRASS, DEMON_SAND = desaturate("#7bc062", 0.6, 0.9), "#b3aa98"
di = Part("SF_Land_DemonIsland")
poly = LANDW["DemonIsland"]
vt = [di.bm.verts.new((x, y, LAND)) for x, y in poly]
for a, b, c in tessellate_polygon([[Vector((x, y, 0)) for x, y in poly]]):
    f = di.bm.faces.new((vt[a], vt[b], vt[c]))
    f.normal_update()
    if f.normal.z < 0:
        f.normal_flip()
bmesh.ops.subdivide_edges(di.bm, edges=list(di.bm.edges), cuts=4, use_grid_fill=True)
bmesh.ops.triangulate(di.bm, faces=list(di.bm.faces))
shore(di.bm, poly, LAND, TOP)
edge_path = poly + poly[:1]
def demon_gloom(q):
    """0 junto al puente (continúa la tierra central), 1 lejos de él (bosque tenebroso)."""
    return max(0.0, min(1.0, (math.dist(q, DEMON_BRIDGE[1]) - 1.0) / 5.5))


for f in di.bm.faces:                                       # Pasto que se vuelve gris verdoso al alejarse del puente
    c = f.calc_center_median()
    t = round(demon_gloom((c.x, c.y)) * 5) / 5
    f.material_index = di.mi(mat("DemonIslandGrass_%d" % round(t * 5), hex_lerp("#7bc062", DEMON_GRASS, t)))
di.build(C_MAP, ROOT)

# Árboles: la paleta del bosque de otoño (verdes, lima, amarillo, naranja, rojo, carmesí), apagada y gris
DEMON_TREES = [mat("DemonTree_%d" % i, desaturate(h)) for i, h in
               enumerate(("#5bb14a", "#4c9a3f", "#a9c23f", "#e8c13a", "#ef8a2c", "#d9453a", "#b8323a"))]
DEMON_DARK = [mat("DemonDark_%d" % i, h) for i, h in enumerate(("#2f2b30", "#3b302b", "#2b3330", "#3a2a2f"))]
TWIG = mat("Twig", "#3b2e28")


def limb(part, m, p0, p1, r0, r1, seg=5):
    """Cilindro cónico de p0 a p1 (ramas, brazos y patas)."""
    d = Vector(p1) - Vector(p0)
    ln = d.length or 1e-4
    part.cyl(m, r0, r1, ln, tuple((Vector(p0) + Vector(p1)) / 2), seg=seg,
             rot_z=math.atan2(d.y, d.x) + math.pi / 2, rot_x=math.acos(max(-1.0, min(1.0, d.z / ln))))


def dead_tree(part, x, y, z, h, rr):
    """Árbol seco: tronco y ramas retorcidas sin hojas."""
    top = (x, y, z + h)
    limb(part, TWIG, (x, y, z), top, 0.09, 0.05)
    for k in range(5):
        a = k * 1.3 + rr.uniform(-0.3, 0.3)
        start = (x, y, z + h * rr.uniform(0.55, 0.95))
        end = (start[0] + math.cos(a) * 0.55, start[1] + math.sin(a) * 0.55, start[2] + rr.uniform(0.2, 0.5))
        limb(part, TWIG, start, end, 0.04, 0.015, seg=4)
        limb(part, TWIG, end, (end[0] + math.cos(a + 0.6) * 0.25, end[1] + math.sin(a + 0.6) * 0.25, end[2] + 0.2),
             0.015, 0.005, seg=3)


dfz = Part("SF_DemonForest")
_poly = LANDW["DemonIsland"]                                       # Centroide de la isla: ahí va el demonio
_a = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(_poly, _poly[1:] + _poly[:1])) / 2
dx_ = sum((a[0] + b[0]) * (a[0] * b[1] - b[0] * a[1]) for a, b in zip(_poly, _poly[1:] + _poly[:1])) / (6 * _a)
dy_ = sum((a[1] + b[1]) * (a[1] * b[0] - b[1] * a[0]) for a, b in zip(_poly, _poly[1:] + _poly[:1])) / (-6 * _a)
free = lambda p: math.dist(p, (dx_, dy_)) > 1.4 and dist_to_path(p, edge_path) > 0.5 and off_bridge(p, 1.0)
# Primero los árboles tenebrosos con separación amplia (quedan repartidos parejo por toda la isla),
# después los de colores apagados rellenan el resto.
# Árboles normales en tonos fríos y grisáceos (verde azulado, gris azulado, pizarra, lavanda). Junto al
# puente empiezan en el verde de la tierra central y se enfrían al alejarse.
COLD_SRC = ("#5f8f84", "#6b8a9c", "#7a86a6", "#8a9a90", "#6d7f8e", "#9aa5b4", "#7f9488", "#8e8aa8")
WARM_START = ("#5bb14a", "#4c9a3f")
for x, y in scatter_poly(rng, 46, _poly, 0.75, ok=free):
    g = round(demon_gloom((x, y)) * 3) / 3                  # 0: verde de la tierra central; 1: frío y gris
    cold = COLD_SRC[rng.randrange(len(COLD_SRC))]
    col = hex_lerp(WARM_START[rng.randrange(2)], desaturate(cold, 0.3, 0.95), min(1.0, g * 1.4))
    puffy_tree(dfz, x, y, LAND, rng.uniform(1.0, 1.4), mat("DemonTreeCold_%s" % col[1:], col))
dfz.build(C, ROOT)
# Forest Demon: modelo propio en su propio archivo (tools/blender/smilingfriends/build_forest_demon.py).
# Se importa el objeto SF_ForestDemon y se coloca de pie en el centro de la isla.
DEMON_FILE = os.path.join(REPO, "assets", "models", "characters", "smilingfriends", "forest_demon.blend")
DEMON_SCALE = 0.6
with bpy.data.libraries.load(DEMON_FILE, link=False) as (src, dst):
    dst.objects = ["SF_ForestDemon"]
demon_ob = dst.objects[0]
C.objects.link(demon_ob)
demon_ob.parent = ROOT
demon_ob.location = (dx_, dy_, LAND)
demon_ob.scale = (DEMON_SCALE,) * 3

# --- Brasil: isla con forma de Brasil, el Corcovado con el Cristo Redentor y Río de Janeiro a sus pies ---
C = collection("Zone_Brazil", C_ZONES)
bz = Part("SF_Brazil")
brng = random.Random(SEED + 8)
rio = br_rot(sk(1325 + 11 * BR_PX, 1200 + 8.5 * BR_PX))                  # Río de Janeiro: longitud -43, latitud -23
_bz_edge = LANDW["Brazil"] + LANDW["Brazil"][:1]
_bz_c = (sum(q[0] for q in LANDW["Brazil"]) / len(LANDW["Brazil"]), sum(q[1] for q in LANDW["Brazil"]) / len(LANDW["Brazil"]))
for _t in (i / 20 for i in range(21)):                     # Corcovado: de Río hacia adentro, hasta tener margen
    cx_, cy_ = rio[0] + (_bz_c[0] - rio[0]) * _t, rio[1] + (_bz_c[1] - rio[1]) * _t
    if dist_to_path((cx_, cy_), _bz_edge) > 1.7:
        break
rock, jungle = mat("CorcovadoRock", "#7d8572"), M["land"]   # Base del cerro: mismo pasto que la isla
# Corcovado: cerro empinado y rocoso, con la cumbre corrida a un lado y la base cubierta de selva
bz.cyl(jungle, 1.7, 0.9, 1.0, (cx_, cy_, LAND + 0.5), seg=10)
bz.cyl(rock, 1.0, 0.32, 2.6, (cx_ + 0.15, cy_ - 0.1, LAND + 1.0 + 1.3), seg=8, rot_z=0.3)   # cumbre plana
bz.cyl(rock, 0.7, 0.0, 1.8, (cx_ - 0.55, cy_ + 0.35, LAND + 1.0 + 0.9), seg=7, rot_z=1.1)
bz.cyl(jungle, 1.15, 0.7, 0.6, (cx_ + 0.1, cy_, LAND + 1.3), seg=9, rot_z=0.5)
peak = (cx_ + 0.15, cy_ - 0.1, LAND + 1.0 + 2.6 - 0.05)    # Sobre la cumbre plana
# Cristo Redentor, más grande, de frente a la ciudad
stone = mat("Statue", "#e3e1d8")
k = 1.15
bz.box(stone, (0.4 * k, 0.4 * k, 0.3 * k), (peak[0], peak[1], peak[2] + 0.1 * k))
bz.cyl(stone, 0.26 * k, 0.14 * k, 1.2 * k, (peak[0], peak[1], peak[2] + 0.85 * k), seg=10)
bz.box(stone, (1.4 * k, 0.14 * k, 0.14 * k), (peak[0], peak[1], peak[2] + 1.3 * k))
bz.sphere(stone, 0.13 * k, (peak[0], peak[1], peak[2] + 1.58 * k), seg=8, rings=5)
# Ciudad: edificios claros alrededor del Corcovado, sin subir al cerro
city_cols = [mat("RioWhite", "#f1eee6"), mat("RioCream", "#e8d9b8"), mat("RioPeach", "#efb99a"),
             mat("RioBlue", "#a9c8e0"), mat("RioPink", "#e7a8b4")]
for x, y in scatter_poly(brng, 30, LANDW["Brazil"], 0.5,
                         ok=lambda q: math.dist(q, rio) < 3.6 and math.dist(q, (cx_, cy_)) > 1.9
                         and dist_to_path(q, LANDW["Brazil"] + LANDW["Brazil"][:1]) > 0.35):
    w, h = brng.uniform(0.28, 0.45), brng.uniform(0.35, 1.3)
    bz.box(city_cols[brng.randrange(len(city_cols))], (w, w * brng.uniform(0.8, 1.2), h), (x, y, LAND + h / 2),
           rot_z=brng.uniform(0, 0.4))
# Selva amazónica en el norte de la isla
for x, y in scatter_poly(brng, 70, LANDW["Brazil"], 0.72,
                         ok=lambda q: math.dist(q, rio) > 3.7 and math.dist(q, (cx_, cy_)) > 2.0
                         and dist_to_path(q, LANDW["Brazil"] + LANDW["Brazil"][:1]) > 0.5):
    puffy_tree(bz, x, y, LAND, brng.uniform(0.9, 1.2), M["leaf2"] if brng.random() < 0.5 else M["leaf"])
bz.build(C, ROOT)

# --- Satanás (T1 EP8) en el centro del infierno, colgando de la cara inferior ---------------------------
# Se modela de pie (z hacia arriba, mirando a -Y) y al final se da vuelta para que cuelgue del disco.
C = collection("Zone_Hell_Satan", C_HELL)
sat = Part("SF_Hell_Satan")
SKIN, FUR, HORN = mat("SatanSkin", "#b8322a"), mat("SatanFur", "#231c1c"), mat("SatanHorn", "#1a1414")
WING, EYE, FANG = mat("SatanWing", "#5a3a33"), mat("SatanEyes", "#ffd23f", emit=3.0), mat("SatanTeeth", "#d8cf9a")
UP = 1.3                                                     # El torso baja 1.3 al sentarse
for sx in (-1, 1):                                           # patas peludas: muslos al frente, pezuñas en el piso
    hip_, knee_, ankle_ = (sx * 0.45, 0, 1.8), (sx * 0.55, -1.3, 1.8), (sx * 0.55, -1.35, 0.3)
    sat.sphere(FUR, 0.34, hip_, seg=8, rings=5)
    limb(sat, FUR, hip_, knee_, 0.34, 0.27, seg=8)
    limb(sat, FUR, knee_, ankle_, 0.27, 0.18, seg=8)
    sat.sphere(FUR, 0.27, knee_, seg=8, rings=5)
    sat.box(HORN, (0.4, 0.55, 0.3), (sx * 0.55, -1.45, 0.15))
sat.sphere(FUR, 0.6, (0, 0, 3.1 - UP), seg=10, rings=6, scale=(1.1, 0.8, 0.7))     # cadera
limb(sat, SKIN, (0, 0, 3.2 - UP), (0, -0.25, 5.0 - UP), 0.45, 0.85, seg=10)     # torso en V, inclinado al frente
sat.sphere(SKIN, 0.8, (0, -0.3, 4.7 - UP), seg=12, rings=8, scale=(1.35, 0.75, 0.85))
for sx in (-1, 1):                                           # brazos hacia la laptop, garras sobre el teclado
    sh, el, ha = (sx * 1.05, -0.2, 5.0 - UP), (sx * 1.25, -1.2, 2.95), (sx * 0.45, -1.75, 2.95)
    sat.sphere(SKIN, 0.48, sh, seg=10, rings=6)
    limb(sat, SKIN, sh, el, 0.34, 0.26, seg=8)
    sat.sphere(SKIN, 0.26, el, seg=8, rings=5)
    limb(sat, SKIN, el, ha, 0.26, 0.2, seg=8)
    sat.sphere(SKIN, 0.24, ha, seg=8, rings=5, scale=(1, 1.2, 0.7))
    for k in range(4):
        base = (ha[0] + (k - 1.5) * 0.1, ha[1] - 0.15, ha[2])
        limb(sat, HORN, base, (base[0], base[1] - 0.35, base[2] - 0.08), 0.05, 0.0, seg=4)
neck0, head = (0, -0.3, 5.2 - UP), (0, -0.75, 6.0 - UP)      # cabeza agachada mirando la pantalla
limb(sat, SKIN, neck0, (0, -0.6, 5.65 - UP), 0.32, 0.28, seg=8)
sat.sphere(SKIN, 0.55, head, seg=12, rings=8, scale=(0.85, 0.95, 1.1))
H = lambda x, y, z: (head[0] + x, head[1] + y + 0.18, head[2] + z - 6.05)   # Detalles de la cabeza de antes
for sx in (-1, 1):
    sat.sphere(EYE, 0.08, H(sx * 0.2, -0.66, 6.1), seg=6, rings=4)            # ojos amarillos
    limb(sat, SKIN, H(sx * 0.42, -0.2, 6.15), H(sx * 0.8, -0.1, 6.45), 0.12, 0.0, seg=5)   # orejas
    pts = [H(sx * 0.3, -0.1, 6.45), H(sx * 0.65, -0.05, 6.95), H(sx * 1.15, 0.0, 7.2), H(sx * 1.55, 0.05, 7.0),
           H(sx * 1.7, 0.1, 6.55)]                      # cuernos curvos
    for k, (p0, p1) in enumerate(zip(pts, pts[1:])):
        limb(sat, HORN, p0, p1, 0.17 * (1 - k * 0.22), 0.17 * (1 - (k + 1) * 0.22), seg=6)
sat.box(FANG, (0.42, 0.08, 0.16), H(0, -0.66, 5.78))                            # dientes
for sx in (-1, 1):                                           # alas de murciélago, plegadas detrás de la silla
    root, top = (sx * 0.6, 0.55, 5.0 - UP), (sx * 2.6, 1.1, 6.6 - UP)
    fingers = [(sx * 3.3, 1.2, 3.6 - UP), (sx * 2.8, 1.1, 2.9 - UP), (sx * 2.0, 0.95, 2.8 - UP)]
    limb(sat, SKIN, root, top, 0.16, 0.1, seg=6)
    for f_ in fingers:
        limb(sat, SKIN, top, f_, 0.08, 0.04, seg=5)
    sat.prism(WING, [root, top] + fingers + [(sx * 1.0, 0.75, 3.8 - UP)], (0, 0.06, 0))
# Silla de oficina negra
CHAIR, DESK, METAL = mat("OfficeChair", "#1f1f24"), mat("HellDesk", "#5a3a26"), mat("LaptopGray", "#9aa0a8")
SCREEN = mat("LaptopScreen", "#7fd7ff", emit=3.0)
sat.box(CHAIR, (1.7, 1.5, 0.25), (0, 0.05, 1.45))
sat.box(CHAIR, (1.6, 0.22, 1.7), (0, 0.85, 2.35))
sat.cyl(METAL, 0.09, 0.09, 1.25, (0, 0.1, 0.7), seg=8)
for k in range(5):
    ang = k / 5 * math.tau
    sat.box(METAL, (0.75, 0.12, 0.1), (math.cos(ang) * 0.35, 0.1 + math.sin(ang) * 0.35, 0.08), rot_z=ang)
# Escritorio y laptop abierta (la pantalla mira hacia Satanás)
sat.box(DESK, (3.2, 1.7, 0.16), (0, -2.15, 2.75))
for dx in (-1.45, 1.45):
    for dy in (-1.45, -2.85):
        sat.box(DESK, (0.14, 0.14, 2.67), (dx, dy, 1.34))
# PC gamer: monitor grande, teclado y mouse con luces RGB y torre con ventana de colores
RGB = [mat("RGB_Red", "#ff2a4a", emit=3.0), mat("RGB_Green", "#2aff7a", emit=3.0), mat("RGB_Blue", "#2a8cff", emit=3.0),
       mat("RGB_Purple", "#c42aff", emit=3.0)]
sat.box(CHAIR, (1.3, 0.42, 0.06), (-0.1, -1.8, 2.87))                            # teclado
for k, m_ in enumerate(RGB):
    sat.box(m_, (0.3, 0.42, 0.015), (-0.6 + k * 0.33, -1.8, 2.905))
sat.box(CHAIR, (0.18, 0.28, 0.06), (0.85, -1.8, 2.87))                           # mouse
sat.box(RGB[3], (0.04, 0.2, 0.015), (0.85, -1.8, 2.905))
sat.box(METAL, (0.5, 0.35, 0.05), (-0.1, -2.6, 2.86))                            # base del monitor
sat.box(METAL, (0.12, 0.08, 0.6), (-0.1, -2.62, 3.15))
sat.box(CHAIR, (2.0, 0.1, 1.15), (-0.1, -2.6, 3.75))                             # monitor
sat.box(SCREEN, (1.85, 0.02, 1.0), (-0.1, -2.54, 3.75))                          # pantalla encendida
sat.box(CHAIR, (0.55, 1.1, 1.25), (1.25, -2.35, 3.46))                           # torre
sat.box(RGB[2], (0.02, 0.85, 0.95), (0.97, -2.35, 3.46))                         # ventana lateral
for k, m_ in enumerate(RGB[:3]):                                                 # ventiladores al frente
    sat.cyl(m_, 0.14, 0.14, 0.03, (1.25, -2.91, 3.0 + k * 0.38), seg=10, rot_x=math.radians(90))
# Dar vuelta: de pie -> colgando bajo el disco, sin dejar de mirar hacia -Y
sat.place(Matrix.Translation((0, 0, -TOP - under_depth(0))) @ Matrix.Rotation(math.pi, 4, "Y"))
sat.build(C, ROOT)

# ---------------------------------------------------------------------------
# Presentación (excluir de la exportación)
# ---------------------------------------------------------------------------

world = bpy.data.worlds.new("SmilingFriends_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#1b2a3a"), 1)
scene.world = world


def light(name, loc, energy, size=12.0):
    ld = bpy.data.lights.new(name, "AREA")
    ld.energy = energy
    ld.size = size
    ob = bpy.data.objects.new(name, ld)
    C_PRES.objects.link(ob)
    ob.location = loc
    ob.rotation_euler = (Vector((0, 0, 0)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()


light("SF_Key", (-30, -42, 60), 10500)
light("SF_Fill", (35, -28, 12), 8500)
light("SF_Rim", (10, 30, 45), 11000)
light("SF_Front", (-40, -42, -10), 6000)
light("SF_Under", (0, 0, -50), 9000)

cam_data = bpy.data.cameras.new("SF_Overview")
cam_data.type = "ORTHO"
cam = bpy.data.objects.new("SF_Overview", cam_data)
C_PRES.objects.link(cam)
scene.camera = cam

for eng in ("BLENDER_EEVEE", "BLENDER_EEVEE_NEXT"):
    try:
        scene.render.engine = eng
        break
    except TypeError:
        pass
scene.render.resolution_x = scene.render.resolution_y = 1100
try:
    scene.eevee.taa_render_samples = 32
except AttributeError:
    pass


def zone_view(px, py, scale, z=1.5):
    x, y = sk(px, py)
    return (x, y - 22, z + 20), (x, y, z), scale


VIEWS = {
    "three_quarter": ((-40, -66, 44), (0, 0, 0), 60),
    "top": ((0, 0.01, 90), (0, 0, 0), 56),
    "side": ((0, -90, 6), (0, 0, 0), 56),
    "bottom": ((0, 0.01, -90), (0, 0, 0), 56),
    "hell": ((0, -30, -26), (0, 0, -7), 30),
    "main_street": (tuple(CITY @ Vector((26, ROAD_Y, 16))), tuple(CITY @ Vector((0, ROAD_Y, 1.5))), 19),
    "smiling_friends": (tuple(CITY @ Vector((0, -30, 19))), tuple(CITY @ Vector((0, 0.3, 2.2))), 15),
    "frowning_friends": (tuple(CITY @ Vector((6, 16, 24))), tuple(CITY @ Vector((-0.4, FF_FRONT - 1.55, 1.4))), 12),
    "spamtopia": zone_view(640, 480, 18),
    "mr_frog": zone_view(1200, 280, 19),
    "shmaloogleville": zone_view(1560, 760, 16),
    "islands": zone_view(820, 1050, 22),
    "brazil": zone_view(1330, 1215, 15),
    "forest_demon": zone_view(1015, 1050, 7, z=1.8),
    "default": ((*VIEWER, 16.0), (0.0, 2.0, 2.0), 0),   # Perspectiva como la vista por defecto de Cami
}


def render(view):
    loc, target, scale = VIEWS[view]
    cam.location = loc
    cam.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    if view == "top":  # Mirando hacia abajo con +Y arriba: misma orientación que el boceto
        cam.rotation_euler = (0, 0, 0)
    cam_data.type = "PERSP" if scale == 0 else "ORTHO"
    cam_data.lens = 30
    cam_data.ortho_scale = scale or 1.0
    scene.render.filepath = os.path.join(REF_DIR, "smilingfriends_%s_%s.png" % (VERSION, view))
    bpy.ops.render.render(write_still=True)


# ---------------------------------------------------------------------------
# Auditoría, guardado y capturas
# ---------------------------------------------------------------------------

dg = bpy.context.evaluated_depsgraph_get()
total = 0
for coll in [C_BASE, C_MAP, C_RIM, C_HELL] + list(C_ZONES.children):
    n = 0
    for ob in coll.objects:
        me = ob.evaluated_get(dg).data
        me.calc_loop_triangles()
        n += len(me.loop_triangles)
    total += n
    print("TRIS %-24s %6d" % (coll.name, n))
print("TRIS TOTAL %d  objects %d  materials %d" % (total, sum(1 for o in scene.objects if o.type == "MESH"), len(MATS)))
_lod = 0
for ob in C_LOD.objects:
    _me = ob.evaluated_get(dg).data
    _me.calc_loop_triangles()
    _lod += len(_me.loop_triangles)
_rim = sum(len(o.evaluated_get(dg).data.loop_triangles) for o in C_RIM.objects if o.name.startswith("SF_Clouds"))
print("TRIS LOD nubes lejanas %d (cerca %d) -> planeta lejos ~%d" % (_lod, _rim, total - _rim + _lod))

# Contorno negro de caricatura, guardado en el .blend para verlo en Blender. Técnica de "casco invertido":
# un modificador Solidify infla cada objeto con las normales al revés y solo se ven sus caras traseras, en
# negro. Se agrega después del conteo de triángulos: no forma parte del modelo, y en el motor propio se hará
# como shader de contorno. Al exportar, omitir los modificadores "Outline". SF_OUTLINE=0 lo desactiva.
if os.environ.get("SF_OUTLINE") != "0":
    OUTLINE_THIN = {"SF_ForestDemon": 0.012}   # Personajes de ramas finas: contorno delgado (en su escala local)
    outline = bpy.data.materials.new("SF_Outline")
    outline.diffuse_color = (0, 0, 0, 1)
    outline.use_nodes = True
    _ob = outline.node_tree.nodes["Principled BSDF"]
    _ob.inputs["Base Color"].default_value = (0, 0, 0, 1)
    _ob.inputs["Roughness"].default_value = 1.0
    outline.use_backface_culling = True
    for ob in list(scene.objects):
        if ob.type != "MESH" or ob.name in ("SF_Disc", "SF_ForestDemon", "SF_Shmaloogle_Path", "SF_Road_East"):   # Demonio: plastilina; camino de la colina: cinta sin costados
            continue
        ob.data.materials.append(outline)
        mod = ob.modifiers.new("Outline", "SOLIDIFY")
        det = ob.data.attributes.get("detail")
        if det and any(v.value for v in det.data):
            shell = set()
            for poly, v in zip(ob.data.polygons, det.data):
                if not v.value:
                    shell.update(poly.vertices)
            vg = ob.vertex_groups.new(name="OutlineShell")
            vg.add(list(shell), 1.0, "REPLACE")
            mod.vertex_group, mod.thickness_vertex_group = "OutlineShell", 0.0
        mod.thickness = OUTLINE_THIN.get(ob.name, 0.05)
        mod.offset = 1.0
        mod.use_flip_normals = True
        mod.use_even_offset = True
        mod.use_rim = False
        mod.material_offset = 100      # Fuera de rango: Blender usa el último material (el contorno)
    # Vista 3D de Blender: ocultar caras traseras y usar el color de los materiales, para que el contorno se vea
    for screen in bpy.data.screens:
        for area in screen.areas:
            for space in area.spaces:
                if space.type == "VIEW_3D":
                    space.shading.show_backface_culling = True
                    space.shading.color_type = "MATERIAL"

os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
os.makedirs(REF_DIR, exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if os.environ.get("SF_NO_RENDER") != "1":
    for v in VIEWS:
        render(v)

    # Catálogo de familias de nubes (después de guardar: no queda en el .blend).
    # Dos ejemplos por familia, en columnas, lejos del planeta.
    cat = collection("Cloud_Catalog", scene.collection)
    for col, fam in enumerate(CLOUD_FAMILIES):
        for row in range(2):
            p = Part("Catalog_" + fam)
            CLOUD_FAMILIES[fam](p, (col - 2.5) * 5.0, 300.0, row * 4.2, 1.0, (1.0, 0.0))
            p.build(cat, None)
    sun = bpy.data.objects.new("Catalog_Sun", bpy.data.lights.new("Catalog_Sun", "SUN"))
    sun.data.energy = 3.5
    sun.rotation_euler = (math.radians(50), 0, math.radians(-30))
    cat.objects.link(sun)
    VIEWS["cloud_families"] = ((0, 260, 8), (0, 300, 2.1), 30)
    render("cloud_families")
print("DONE", BLEND_OUT)
