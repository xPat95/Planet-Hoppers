"""Piezas compartidas por los personajes del planeta Halloween (brujas y zombies).

Estilo de ilustración plana: materiales sin brillo con un poco de luz propia, y cuerpos armados con tubos de pocos
lados que siguen los huesos (Builder), o con cajas para los personajes cuadrados. Lo usan build_witch.py y build_zombie.py.
"""

import bmesh
import bpy
import math
import numpy as np
from mathutils import Matrix, Vector


def srgb(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(((v + 0.055) / 1.055) ** 2.4 if v > 0.04045 else v / 12.92 for v in c)


def mat(name, hexcol, emit=0.0, rough=1.0):
    """Material plano: sin brillo y con un poco de luz propia, para que cada color se lea como un relleno parejo."""
    m = bpy.data.materials.new("HW_" + name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Roughness"].default_value = rough
    b.inputs["Specular IOR Level"].default_value = 0.0
    b.inputs["Emission Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Emission Strength"].default_value = emit if emit else 0.35
    m.diffuse_color = (*srgb(hexcol), 1)
    return m


def rx(a):
    return Matrix.Rotation(math.radians(a), 3, "X")


def ry(a):
    return Matrix.Rotation(math.radians(a), 3, "Y")


def rz(a):
    return Matrix.Rotation(math.radians(a), 3, "Z")


class Builder:
    """Junta vértices y caras de todas las piezas en una sola malla (mats: lista de materiales del modelo)."""

    def __init__(self, mats):
        self.mats = mats
        self.v, self.f, self.m = [], [], []
        self.hands = []                                   # Puntas de las manos (para colgarse)
        self.chest = Vector()                             # Pecho (para asomarse por un borde)
        self.seat = Vector()                              # Donde se sienta (para sentarla en un borde)

    def ring(self, c, a, b, r, n, sx=1.0, sy=1.0):
        start = len(self.v)
        for k in range(n):
            t = math.tau * k / n
            self.v.append(c + r * (math.cos(t) * sx * a + math.sin(t) * sy * b))
        return start

    def tube(self, pts, radii, n, m, sx=1.0, sy=1.0, cap0=False, cap1=False, ref=Vector((0, -1, 0)), split=None):
        """Tubo por una polilínea (anillos perpendiculares, marco que no gira de golpe). m puede ser una lista con
        un material por tramo; split=(m1, m2) pinta una mitad de cada color a lo largo (traje de arlequín)."""
        rings = []
        a_prev = None
        for i, p in enumerate(pts):
            tan = (pts[min(i + 1, len(pts) - 1)] - pts[max(i - 1, 0)]).normalized()
            a = (a_prev if a_prev is not None else ref).cross(tan)
            if a.length < 1e-6:
                a = Vector((1, 0, 0)).cross(tan)
            a.normalize()
            b = tan.cross(a)
            a_prev = a.cross(tan) * -1
            rings.append(self.ring(p, a, b, radii[i], n, sx, sy))
        for i in range(len(rings) - 1):
            r0, r1 = rings[i], rings[i + 1]
            for k in range(n):
                k1 = (k + 1) % n
                mk = m[i] if isinstance(m, list) else m
                if split:
                    mk = split[0] if math.cos(math.tau * (k + 0.5) / n) > 0 else split[1]
                self.add((r0 + k, r0 + k1, r1 + k1, r1 + k), mk)
        mc = m[-1] if isinstance(m, list) else m
        for cap, idx, p in ((cap0, rings[0], pts[0]), (cap1, rings[-1], pts[-1])):
            if cap:
                c = len(self.v)
                self.v.append(p.copy())
                for k in range(n):
                    self.add((idx + k, idx + (k + 1) % n, c) if cap is cap1 else (idx + (k + 1) % n, idx + k, c), mc)

    def ellipsoid(self, c, rot, r, m, seg=8, rings=6):
        top = len(self.v)
        self.v.append(c + rot @ Vector((0, 0, r[2])))
        for i in range(1, rings):
            th = math.pi * i / rings
            for k in range(seg):
                ph = math.tau * k / seg
                self.v.append(c + rot @ Vector((r[0] * math.sin(th) * math.cos(ph), r[1] * math.sin(th) * math.sin(ph),
                                                r[2] * math.cos(th))))
        bot = len(self.v)
        self.v.append(c + rot @ Vector((0, 0, -r[2])))
        for k in range(seg):
            k1 = (k + 1) % seg
            self.add((top, top + 1 + k1, top + 1 + k), m)
            self.add((bot, top + 1 + (rings - 2) * seg + k, top + 1 + (rings - 2) * seg + k1), m)
        for i in range(rings - 2):
            for k in range(seg):
                k1 = (k + 1) % seg
                a0, a1 = top + 1 + i * seg, top + 1 + (i + 1) * seg
                self.add((a0 + k, a0 + k1, a1 + k1, a1 + k), m)

    def box(self, c, rot, half, m):
        """Caja (cubo estirado) con centro c, giro rot y medios lados half."""
        base = len(self.v)
        for sx in (-1, 1):
            for sy in (-1, 1):
                for sz in (-1, 1):
                    self.v.append(c + rot @ Vector((sx * half[0], sy * half[1], sz * half[2])))
        for q in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
            self.add(tuple(base + i for i in q), m)

    def bone_box(self, a, b, w, d, side, m):
        """Caja a lo largo del hueso de a a b, de ancho w y fondo d (side: eje de referencia para el ancho)."""
        z = (b - a).normalized()
        x = (side - z * side.dot(z)).normalized()
        y = z.cross(x)
        rot = Matrix((x, y, z)).transposed()
        self.box((a + b) / 2, rot, (w, d, (b - a).length / 2), m)

    def add(self, face, m):
        self.f.append(face)
        self.m.append(self.mats.index(m))


def _smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0, 1)
    return b + (a - b) * h - k * h * (1 - h)


def _surface_nets(F, lo, h):
    """Un vértice por celda que cruza la superficie y un cuadrilátero por arista de la rejilla que la cruza."""
    ins = F < 0
    nx, ny, nz = F.shape
    corners = [(a, b, c) for a in (0, 1) for b in (0, 1) for c in (0, 1)]
    cnt = sum(ins[a:nx - 1 + a, b:ny - 1 + b, c:nz - 1 + c].astype(np.int8) for a, b, c in corners)
    act = (cnt > 0) & (cnt < 8)
    ci = np.argwhere(act)
    vid = np.full(act.shape, -1, np.int64)
    vid[act] = np.arange(len(ci))
    acc = np.zeros((len(ci), 3))
    n = np.zeros(len(ci))
    for a in corners:
        for ax in range(3):
            if a[ax]:
                continue
            b = list(a)
            b[ax] = 1
            ia, ib = ci + a, ci + b
            fa, fb = F[ia[:, 0], ia[:, 1], ia[:, 2]], F[ib[:, 0], ib[:, 1], ib[:, 2]]
            m = (fa < 0) != (fb < 0)
            t = fa[m] / (fa[m] - fb[m])
            acc[m] += ia[m] + t[:, None] * (np.array(b) - np.array(a))
            n[m] += 1
    verts = lo + h * acc / n[:, None]
    quads = []
    for ax in range(3):
        bx, cx = (ax + 1) % 3, (ax + 2) % 3
        s0, s1 = [slice(None)] * 3, [slice(None)] * 3
        s0[ax], s1[ax] = slice(0, -1), slice(1, None)
        low = ins[tuple(s0)]
        P = np.argwhere(low != ins[tuple(s1)])
        P = P[(P[:, bx] >= 1) & (P[:, cx] >= 1) & (P[:, bx] <= F.shape[bx] - 2) & (P[:, cx] <= F.shape[cx] - 2)]

        def cell(db, dc):
            Q = P.copy()
            Q[:, bx] += db
            Q[:, cx] += dc
            return vid[Q[:, 0], Q[:, 1], Q[:, 2]]
        q = np.stack([cell(-1, -1), cell(0, -1), cell(0, 0), cell(-1, 0)], axis=1)
        flip = ~low[P[:, 0], P[:, 1], P[:, 2]]           # Normal hacia afuera del volumen
        q[flip] = q[flip][:, ::-1]
        quads.append(q)
    return verts, np.concatenate(quads)


class SmoothBody:
    """Cuerpo de una sola pieza: huesos (tubos) y elipsoides unidos en suave, sin costuras en hombros, codos,
    cadera ni cuello. Cada pieza guarda su material (o una función punto -> material, para las medias a rayas), y
    cada cara de la malla final toma el de la pieza más cercana."""

    def __init__(self, h=0.0055, k=0.016):
        self.h, self.k = h, k
        self.parts = []                                   # (función de distancia, caja, material)
        self.starts = {}                                  # Pieza -> plano donde empieza su color (punto, dirección)
        self.cuts = []                                    # Planos donde se corta la malla: bordes de color limpios

    def cut(self, p, n, reach):
        """Corta la malla por el plano (p, n), solo cerca de p (hasta reach): ahí cambia de color."""
        self.cuts.append((Vector(p), Vector(n).normalized(), reach))

    def chain(self, pts, radii, m, frame=None, start=None, claim=0.3):
        """Tubo por la polilínea; frame=(R, sx, sy) lo hace de sección elíptica (más ancho que hondo). start=(p, n):
        su color empieza en ese plano (el resto, aunque esté más cerca, es de la pieza vecina) y la malla se corta
        ahí, para que el borde quede derecho. claim: pasado el plano, hasta qué distancia de su superficie (en radios)
        es suyo aunque el vecino esté igual de cerca."""
        if start:
            self.starts[len(self.parts)] = (np.array(tuple(start[0])), np.array(tuple(Vector(start[1]).normalized())),
                                            claim * max(radii))
            self.cut(start[0], start[1], max(radii) * 2.5)
        P0 = np.array([tuple(p) for p in pts])
        rr = np.array(radii, float)
        R, sx, sy = frame if frame else (Matrix.Identity(3), 1.0, 1.0)
        Rn = np.array([list(r) for r in R])               # Filas de R (para pasar al espacio de la pieza)
        sc = np.array([sx, sy, 1.0])

        def dist(P):
            L = (P @ Rn) / sc                             # Columnas de R = ejes de la pieza
            A = (P0 @ Rn) / sc
            d = np.full(len(P), 9.0)
            for i in range(len(A) - 1):
                a, b = A[i], A[i + 1]
                ba = b - a
                t = np.clip((L - a) @ ba / max(ba @ ba, 1e-12), 0, 1)
                d = np.minimum(d, np.linalg.norm(L - a - t[:, None] * ba, axis=1) - (rr[i] + (rr[i + 1] - rr[i]) * t))
            return d * min(sx, sy, 1.0)
        pad = rr.max() * max(sx, sy, 1.0) + 2 * self.h
        self.parts.append((dist, (P0.min(0) - pad, P0.max(0) + pad), m))

    def ellipsoid(self, c, R, r, m):
        c0, r0 = np.array(tuple(c)), np.array(r, float)
        Rn = np.array([list(row) for row in R])

        def dist(P):
            q = ((P - c0) @ Rn) / r0
            k0 = np.linalg.norm(q, axis=1)
            k1 = np.linalg.norm(q / r0, axis=1)
            return k0 * (k0 - 1.0) / np.maximum(k1, 1e-9)
        pad = r0.max() + 2 * self.h
        self.parts.append((dist, (c0 - pad, c0 + pad), m))

    def build(self, B):
        """Malla de la superficie, agregada al Builder B con su material por cara."""
        h = self.h
        lo = np.min([b[0] for _, b, _ in self.parts], axis=0) - 2 * h
        hi = np.max([b[1] for _, b, _ in self.parts], axis=0) + 2 * h
        axes = [np.arange(lo[i], hi[i] + h, h) for i in range(3)]
        F = np.full([len(a) for a in axes], 9.0)
        D = np.full(F.shape, 9.0)                         # Distancia sin unir (para saber de qué pieza es)
        own = np.full(F.shape, -1, np.int32)
        for pi, (dist, (blo, bhi), _) in enumerate(self.parts):
            i0 = np.clip(np.floor((blo - lo) / h).astype(int), 0, None)
            i1 = np.minimum(np.ceil((bhi - lo) / h).astype(int) + 1, F.shape)
            sl = tuple(slice(i0[i], i1[i]) for i in range(3))
            g = np.meshgrid(*[axes[i][sl[i]] for i in range(3)], indexing="ij")
            d = dist(np.stack([gi.ravel() for gi in g], axis=1)).reshape(F[sl].shape)
            F[sl] = _smin(F[sl], d, self.k)
            closer = d < D[sl]
            D[sl] = np.where(closer, d, D[sl])
            own[sl] = np.where(closer, pi, own[sl])
        verts, quads = _surface_nets(F, lo, h)
        for _ in range(3):                                # Un poco de suavizado: sin escalones de la rejilla
            acc = np.zeros_like(verts)
            cnt = np.zeros(len(verts))
            for k in range(4):
                a, b = quads[:, k], quads[:, (k + 1) % 4]
                np.add.at(acc, a, verts[b])
                np.add.at(acc, b, verts[a])
                np.add.at(cnt, a, 1)
                np.add.at(cnt, b, 1)
            verts = verts * 0.5 + 0.5 * acc / np.maximum(cnt, 1)[:, None]
        # Cortes por los bordes de color (rayas de las medias, hombros, cuello), solo cerca de cada borde
        bm = bmesh.new()
        bv = [bm.verts.new(tuple(v)) for v in verts]
        for q in quads:
            try:
                bm.faces.new([bv[i] for i in q])
            except ValueError:
                pass
        for p, n, reach in self.cuts:
            geom = [f for f in bm.faces if (f.calc_center_median() - p).length < reach]
            if not geom:
                continue
            edges = list({e for f in geom for e in f.edges})
            verts_ = list({v for f in geom for v in f.verts})
            bmesh.ops.bisect_plane(bm, geom=verts_ + edges + geom, plane_co=p, plane_no=n)
        # Cada vértice es de la pieza más cercana (con la distancia de verdad, no la de la rejilla; una pieza con plano
        # de inicio no reclama lo que queda antes de él). Donde un triángulo tiene vértices de dos piezas se parte
        # justo por la línea donde las dos están igual de cerca: el borde de color sigue esa línea, sin serrucho
        bmesh.ops.triangulate(bm, faces=bm.faces[:])
        bm.verts.index_update()
        V = np.array([tuple(v.co) for v in bm.verts])
        G = np.stack([self._score(pi, V) for pi in range(len(self.parts))], axis=1)
        own = G.argmin(axis=1)
        base = len(B.v)
        B.v += [Vector(v) for v in V]
        cross = {}

        def border(i, j, a, b):
            """Punto de la arista i-j donde las piezas a y b están igual de cerca (compartido por los dos lados)."""
            i, j = min(i, j), max(i, j)
            key = (i, j, min(a, b), max(a, b))
            if key not in cross:
                fi, fj = G[i, a] - G[i, b], G[j, a] - G[j, b]
                t = min(max(fi / (fi - fj), 0.0), 1.0) if fi != fj else 0.5
                cross[key] = len(B.v)
                B.v.append(Vector(V[i] + t * (V[j] - V[i])))
            return cross[key]

        def paint(vids, pi):
            m = self.parts[pi][2]
            c = sum((B.v[k] for k in vids), Vector()) / len(vids)
            B.add(tuple(vids), m(c) if callable(m) else m)

        for f in bm.faces:
            ids = [v.index for v in f.verts]
            o = [int(own[i]) for i in ids]
            if o[0] == o[1] == o[2]:
                paint([base + i for i in ids], o[0])
            elif len(set(o)) == 2:
                odd = next(k for k in range(3) if o[k] != o[(k + 1) % 3] and o[k] != o[(k + 2) % 3])
                i, j, l = ids[odd], ids[(odd + 1) % 3], ids[(odd + 2) % 3]
                a, b = o[odd], o[(odd + 1) % 3]
                pj, pl = border(i, j, a, b), border(i, l, a, b)
                paint([base + i, pj, pl], a)
                paint([pj, base + j, base + l, pl], b)
            else:                                         # Tres piezas en un triángulo (casi nunca): la del centro
                paint([base + i for i in ids], int(G[ids].mean(axis=0).argmin()))
        bm.free()

    def _score(self, pi, P):
        d = self.parts[pi][0](P)
        if pi in self.starts:
            sp, sn, _ = self.starts[pi]
            d = np.where((P - sp) @ sn < -1e-6, d + 1.0, d)
        return d


def make_mesh(name, B, mats, shift=Vector(), smooth=True):
    """Malla a partir de un Builder, con sus materiales por cara (suave, o plana para piezas de caja)."""
    me = bpy.data.meshes.new(name)
    me.from_pydata([tuple(v - shift) for v in B.v], [], B.f)
    me.validate()
    for m in mats:
        me.materials.append(m)
    me.polygons.foreach_set("material_index", B.m)
    me.polygons.foreach_set("use_smooth", [smooth] * len(me.polygons))
    return me
