"""Piezas compartidas por los personajes del planeta Halloween (brujas y zombies).

Estilo de ilustración plana: materiales sin brillo con un poco de luz propia, y cuerpos armados con tubos de pocos
lados que siguen los huesos (Builder), o con cajas para los personajes cuadrados. Lo usan build_witch.py y build_zombie.py.
"""

import bpy
import math
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
