"""Generador del planeta Halloween: la Tierra quemada con forma de calavera gritando, hueca por dentro.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/halloween/build_halloween_planet.py

Guarda assets/models/planets/halloween/halloween_planet.blend y capturas en
docs/references/halloween/halloween_v01_*.png. Diseño: docs/cami-guidelines.md, sección 2.

La forma sale de una función de distancia con las proporciones de un cráneo real: bóveda, ceja de hueso, órbitas,
nariz en pera, pómulos, arcos cigomáticos, maxilar, mastoides, mandíbula abierta (cuerpo en U, rama, cóndilo y
coronoides) y dientes de adulto en sus arcos. El cráneo se vacía en una cáscara gruesa y las órbitas, la nariz y
el foramen magnum la atraviesan, para poder poner cosas adentro. La malla se arma con "surface nets" sobre una
rejilla, se le suma ruido de roca y se simplifica. Colores por cara: estratos de roca en tonos cercanos (como el
Gran Cañón) y pocas grietas de lava afuera, y roca negra verdosa con venas que brillan en verde adentro. Por ahora el planeta va
vacío: las zonas se agregan después.
"""

import bpy
import math
import os
import numpy as np
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "planets", "halloween", "halloween_planet.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "halloween")

SEED = 666
R = 22.0             # Radio del cráneo (South Park: 22)
H = 0.013            # Tamaño de celda de la rejilla (en unidades de R)
SHELL = 0.2          # Grosor de la cáscara (en unidades de R, ~4.5 unidades)
TARGET_TRIS = 50000  # Triángulos tras simplificar (afuera y adentro)

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "Halloween_Planet"
rng = np.random.default_rng(SEED)


def srgb(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(((v + 0.055) / 1.055) ** 2.4 if v > 0.04045 else v / 12.92 for v in c)


def mat(name, hexcol, emit=0.0, rough=0.9):
    m = bpy.data.materials.new("HW_" + name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Roughness"].default_value = rough
    if emit:
        b.inputs["Emission Color"].default_value = (*srgb(hexcol), 1)
        b.inputs["Emission Strength"].default_value = emit
    m.diffuse_color = (*srgb(hexcol), 1)
    return m


# Paleta: estratos de arenisca quemada en tonos cercanos, roca oscura en bordes y adentro, y lava.
STRATA = [mat("Strata%d" % i, h) for i, h in
          enumerate(("#b45a3c", "#bf6644", "#c8724c", "#b0573a", "#c06a47", "#cb7a51"))]
RUST_D = mat("RustDark", "#8e432c")
CHAR = mat("Char", "#3b1c13")
INNER = mat("Inner", "#0f1a10")                         # Adentro: roca negra verdosa
TEETH = mat("Teeth", "#d08a62")
LAVA = mat("Lava", "#ff5a1a", emit=2.5, rough=0.5)
LAVA_HOT = mat("LavaHot", "#ff9a2e", emit=3.5, rough=0.5)
GLOW = mat("InnerGlow", "#8fd61e", emit=1.4, rough=0.5)  # Venas de adentro: verde cálido...
GLOW_HOT = mat("InnerGlowHot", "#ff7a14", emit=2.5, rough=0.5)  # ...que en el centro pasa a naranja
MATS = STRATA + [RUST_D, CHAR, INNER, TEETH, LAVA, LAVA_HOT, GLOW, GLOW_HOT]
M = {m: i for i, m in enumerate(MATS)}

# --- Ruido (valor 3D en una red periódica, con semilla) ----------------------------------------------------
LAT = rng.random((64, 64, 64)).astype(np.float32)


def vnoise(p, f, off=0.0):
    q = p * f + off
    i = np.floor(q).astype(np.int64)
    t = q - i
    t = t * t * (3 - 2 * t)
    i0, i1 = i % 64, (i + 1) % 64
    out = 0.0
    for cx in (0, 1):
        for cy in (0, 1):
            for cz in (0, 1):
                w = ((t[:, 0] if cx else 1 - t[:, 0]) * (t[:, 1] if cy else 1 - t[:, 1])
                     * (t[:, 2] if cz else 1 - t[:, 2]))
                out = out + w * LAT[(i1 if cx else i0)[:, 0], (i1 if cy else i0)[:, 1], (i1 if cz else i0)[:, 2]]
    return out


def fbm(p, f, octaves=3, off=0.0):
    s, a, tot = 0.0, 1.0, 0.0
    for k in range(octaves):
        s = s + a * vnoise(p, f * 2 ** k, off + 17.3 * k)
        tot += a
        a *= 0.5
    return s / tot


# --- Forma de la calavera (en unidades de R; frente hacia -Y, arriba +Z) -----------------------------------
def sd_sphere(p, c, r):
    return np.linalg.norm(p - c, axis=1) - r


def sd_ell(p, c, r):
    r = np.asarray(r)
    q = (p - c) / r
    k0 = np.linalg.norm(q, axis=1)
    k1 = np.linalg.norm(q / r, axis=1)
    return k0 * (k0 - 1.0) / np.maximum(k1, 1e-9)


def sd_rbox(p, c, b, rad):
    q = np.abs(p - c) - (np.asarray(b) - rad)
    return np.linalg.norm(np.maximum(q, 0), axis=1) + np.minimum(q.max(axis=1), 0) - rad


def smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0, 1)
    return b + (a - b) * h - k * h * (1 - h)


def ssub(a, b, k):
    """Resta suave: a menos b."""
    return -smin(-a, b, k)


def mirror(p):
    pm = p.copy()
    pm[:, 0] = np.abs(pm[:, 0])                          # Simetría izquierda/derecha
    return pm


# Proporciones de un cráneo real, con 1 unidad = 10 cm: bóveda de ~19 x 14 cm, cara de ~12 cm de alto, pómulos de
# ~13 cm de lado a lado, órbitas de ~4 x 3.5 cm. Para que dé miedo: órbitas un poco inclinadas en V (enojo), la
# ceja de hueso baja al centro y la mandíbula abierta, girada sobre la articulación (grito).
def sd_capsule(p, a, b, r):
    a, b = np.asarray(a), np.asarray(b)
    pa, ba = p - a, b - a
    h = np.clip(pa @ ba / (ba @ ba), 0, 1)
    return np.linalg.norm(pa - h[:, None] * ba, axis=1) - r


def chain(p, pts, r):
    return np.min(np.stack([sd_capsule(p, a, b, r) for a, b in zip(pts, pts[1:])]), axis=0)


ORBIT_C, ORBIT_B, ORBIT_TILT = np.array([0.255, -0.84, -0.12]), np.array([0.18, 0.42, 0.155]), 0.4


def orbit_local(pm):
    """Coordenadas en la órbita derecha (girada: la esquina de adentro más baja)."""
    q = pm - ORBIT_C
    ca, sa = math.cos(ORBIT_TILT), math.sin(ORBIT_TILT)
    return np.stack([q[:, 0] * ca + q[:, 2] * sa, q[:, 1], -q[:, 0] * sa + q[:, 2] * ca], axis=1)


# Arcos dentales: parábola vista desde arriba (incisivos al frente, muelas hacia atrás)
def arch(x, front, k=6.6):
    return front + k * x * x


def tooth_row(front, z, up):
    """Dientes de un lado a otro del arco: incisivos, canino, premolares y muelas (anchos de un adulto)."""
    widths = [0.085, 0.065, 0.075, 0.07, 0.07, 0.1, 0.1]          # Del centro hacia atrás
    heights = [0.055, 0.05, 0.06, 0.045, 0.045, 0.04, 0.04]       # Medio alto de la corona
    xs = np.linspace(0, 0.3, 400)
    ys = arch(xs, front)
    s = np.concatenate([[0], np.cumsum(np.hypot(np.diff(xs), np.diff(ys)))])
    out, at = [], 0.0
    for w, h in zip(widths, heights):
        x = np.interp(at + w / 2, s, xs)
        for sx in (-1, 1):
            out.append((np.array((sx * x, arch(x, front), z + (h if up else -h))), np.array((w * 0.45, w * 0.45, h))))
        at += w
    return out


UPPER_TEETH = tooth_row(-0.84, -0.6, up=False)
LOWER_TEETH = tooth_row(-0.815, -0.72, up=True)                    # Cerrada: muerde detrás de los de arriba
JAW_PIVOT = np.array([0.0, -0.13, -0.25])                          # Articulación (frente a la oreja)
JAW_OPEN, JAW_SLIDE = math.radians(26), np.array([0.0, -0.09, -0.02])   # Al abrir, también se desliza adelante


def jaw_closed(p):
    """Lleva un punto del mundo a la posición de mandíbula cerrada."""
    q = p - JAW_SLIDE - JAW_PIVOT
    c, s = math.cos(-JAW_OPEN), math.sin(-JAW_OPEN)
    return np.stack([q[:, 0], q[:, 1] * c - q[:, 2] * s, q[:, 1] * s + q[:, 2] * c], axis=1) + JAW_PIVOT


def body(p):
    """El volumen lleno del cráneo (sin mandíbula ni huecos)."""
    pm = mirror(p)
    d = sd_ell(p, (0, 0.1, 0.15), (0.7, 0.93, 0.72))                                    # Bóveda
    d = smin(d, sd_ell(p, (0, 0.88, 0.02), (0.3, 0.12, 0.2)), 0.2)                      # Occipital
    d = smin(d, sd_ell(p, (0, -0.68, -0.12), (0.5, 0.19, 0.22)), 0.16)                  # Marco de las órbitas
    d = smin(d, chain(pm, [(0, -0.86, -0.01), (0.25, -0.83, 0.1), (0.46, -0.72, 0.07)], 0.04), 0.12)  # Ceja: baja al centro
    d = smin(d, sd_ell(pm, (0.4, -0.66, -0.28), (0.16, 0.16, 0.11)), 0.14)              # Pómulos
    d = smin(d, chain(pm, [(0.4, -0.64, -0.25), (0.58, -0.42, -0.25), (0.6, -0.16, -0.22)], 0.05), 0.1)    # Arco cigomático
    d = smin(d, sd_ell(p, (0, -0.66, -0.45), (0.29, 0.22, 0.2)), 0.15)                  # Maxilar
    d = smin(d, chain(pm, [(x, arch(x, -0.82), -0.58) for x in np.linspace(0, 0.27, 6)], 0.06), 0.1)   # Encía de arriba
    d = smin(d, sd_capsule(p, (0, -0.875, -0.03), (0, -0.9, -0.17), 0.04), 0.08)        # Huesos de la nariz
    d = smin(d, sd_ell(pm, (0.5, 0.08, -0.45), (0.07, 0.08, 0.11)), 0.14)               # Mastoides
    d = ssub(d, sd_sphere(pm, (0.75, -0.35, -0.05), 0.2), 0.15)                         # Fosa temporal
    return ssub(d, sd_sphere(pm, (0.6, -0.02, -0.3), 0.045), 0.02)                      # Oído (poco hondo)


def holes(p):
    """Órbitas, abertura de la nariz y agujero de la nuca: atraviesan la cáscara."""
    pm = mirror(p)
    # Órbitas, cuadradas redondeadas, con una sola pared curva: se angosta suave y continua desde el borde hasta el
    # fondo, sin escalones (cada quiebre se leía como otro borde)
    q = orbit_local(pm)
    t = np.clip((q[:, 1] + 0.05) / 0.47, 0, 1)
    taper = 1.08 - 0.38 * t * t * (3 - 2 * t)
    q[:, 0] /= taper
    q[:, 2] /= taper
    d = sd_rbox(q, 0, ORBIT_B, 0.13)
    d = np.minimum(d, sd_ell(p, (0, -0.86, -0.36), (0.11, 0.4, 0.11)))                 # Nariz en pera...
    d = np.minimum(d, sd_ell(p, (0, -0.86, -0.24), (0.06, 0.4, 0.1)))                  # ...angosta arriba
    return np.minimum(d, sd_ell(p, (0, 0.32, -0.55), (0.08, 0.1, 0.3)))                # Foramen magnum


def jaw(p):
    """Mandíbula maciza: cuerpo en U, barbilla, ángulo y rama con su cóndilo y su apófisis coronoides."""
    q = jaw_closed(p)
    qm = mirror(q)
    body_pts = [(x, arch(x, -0.8), -0.86) for x in np.linspace(0, 0.27, 6)] + [(0.46, -0.16, -0.86)]
    qs = qm.copy()
    qs[:, 2] = -0.86 + (qs[:, 2] + 0.86) * 0.4                     # Sección mucho más alta que ancha
    d = chain(qs, body_pts, 0.055)
    d = smin(d, sd_ell(q, (0, -0.84, -0.9), (0.09, 0.05, 0.07)), 0.12)                  # Barbilla
    d = smin(d, sd_ell(qm, (0.47, -0.24, -0.55), (0.045, 0.12, 0.27)), 0.1)            # Rama
    d = smin(d, sd_capsule(qm, (0.46, -0.16, -0.86), (0.5, -0.13, -0.27), 0.045), 0.1)  # Borde de atrás
    d = smin(d, sd_capsule(qm, (0.4, -0.38, -0.72), (0.47, -0.3, -0.33), 0.035), 0.1)   # Coronoides
    d = smin(d, sd_ell(qm, (0.5, -0.13, -0.25), (0.06, 0.045, 0.04)), 0.08)            # Cóndilo
    return smin(d, chain(qm, [(x, arch(x, -0.8), -0.74) for x in np.linspace(0, 0.27, 6)], 0.05), 0.1)   # Encía


def teeth(p):
    """Dientes de arriba (fijos) y de abajo (giran con la mandíbula). Solo se evalúan cerca de la boca."""
    out = np.full(len(p), 9.0)
    near = (np.abs(p[:, 0]) < 0.4) & (p[:, 1] < 0.0) & (p[:, 2] < -0.45) & (p[:, 2] > -1.2)
    if near.any():
        pn = p[near]
        qn = jaw_closed(pn)
        up = np.min(np.stack([sd_rbox(pn, c, b, 0.02) for c, b in UPPER_TEETH]), axis=0)
        lo = np.min(np.stack([sd_rbox(qn, c, b, 0.02) for c, b in LOWER_TEETH]), axis=0)
        out[near] = np.minimum(up, lo)
    return out


def nose_bones(p):
    """Dentro de la abertura de la nariz: tabique al centro, cornetes en las paredes y la espina de abajo."""
    pm = mirror(p)
    d = sd_rbox(p, (0, -0.79, -0.31), (0.016, 0.08, 0.15), 0.01)                       # Tabique
    d = np.minimum(d, sd_ell(pm, (0.075, -0.71, -0.36), (0.025, 0.1, 0.055)))           # Cornetes
    return np.minimum(d, sd_capsule(p, (0, -0.84, -0.45), (0, -0.92, -0.44), 0.02))    # Espina nasal


def brow(p):
    """Ceño: ceja de hueso baja que sobresale sobre la parte de arriba de las órbitas, más baja al centro."""
    return chain(mirror(p), [(0, -0.885, -0.03), (0.2, -0.875, 0.03), (0.42, -0.78, 0.07)], 0.04)


def planet(p):
    b = body(p)
    d = np.maximum(b, -(b + SHELL))                      # Cáscara hueca
    # Las aberturas solo atraviesan la capa de afuera en la cara; en el resto del cráneo queda protegida (las
    # órbitas llegan hasta la sien hundida y la romperían)
    skin = np.maximum(-(b + 0.07), -(p[:, 1] + 0.7))    # Negativo en la capa exterior lejos de la cara
    d = ssub(d, np.maximum(holes(p), -skin), 0.07)       # Bordes de las aberturas redondeados
    d = smin(d, nose_bones(p), 0.02)
    # Ceño: la ceja de hueso baja y sobresale sobre la parte de arriba de las órbitas, más baja al centro (enojo).
    # Va después del corte para que tape el borde de arriba sin tocar la forma de la órbita
    d = smin(d, brow(p), 0.11)
    return np.minimum(np.minimum(d, jaw(p)), teeth(p))


# --- Malla: surface nets sobre la rejilla ------------------------------------------------------------------
LO = np.array([-0.85, -1.1, -1.4])
HI = np.array([0.85, 1.15, 0.95])
axes = [np.arange(LO[i], HI[i] + H, H) for i in range(3)]
F = np.empty([len(a) for a in axes], np.float32)
gy, gz = np.meshgrid(axes[1], axes[2], indexing="ij")
for i, x in enumerate(axes[0]):                          # Por rebanadas, para no llenar la memoria
    pts = np.stack([np.full(gy.size, x), gy.ravel(), gz.ravel()], axis=1)
    F[i] = planet(pts).reshape(gy.shape)


def surface_nets(F):
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
    for a in corners:                                    # Las 12 aristas de cada celda
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
    verts = LO + H * acc / n[:, None]
    quads = []
    for ax in range(3):
        bx, cx = (ax + 1) % 3, (ax + 2) % 3
        s0 = [slice(None)] * 3
        s1 = [slice(None)] * 3
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


verts, quads = surface_nets(F)
del F

# Roca: relieve por ruido y cráteres (los cráteres solo afuera y lejos de la cara)
dirs = verts / np.linalg.norm(verts, axis=1)[:, None]
outer_w = np.clip((body(verts) + SHELL) / SHELL, 0, 1)   # 1 en la cara exterior, 0 en la interior
disp = 0.025 * (fbm(dirs, 3.0, 2, 1.7) - 0.5) + 0.012 * (vnoise(dirs, 9.0, 5.1) - 0.5) \
    + 0.004 * (vnoise(dirs, 20.0, 9.3) - 0.5)
craters = rng.normal(size=(40, 3))
craters /= np.linalg.norm(craters, axis=1)[:, None]
craters = craters[(craters[:, 1] > -0.3) & (craters[:, 2] > -0.5)][:12]
for c in craters:
    rho = rng.uniform(0.07, 0.15)                        # Radio angular
    a = np.arccos(np.clip(dirs @ c, -1, 1)) / rho
    disp += outer_w * np.where(a < 1, -0.2 * rho * (1 - a * a), 0.07 * rho * np.exp(-((a - 1) / 0.3) ** 2))
disp *= np.clip(holes(verts) / 0.1, 0.15, 1)            # Casi sin ruido en los bordes de las aberturas
verts = verts + dirs * disp[:, None]

me = bpy.data.meshes.new("HW_Planet_Surface")
me.from_pydata((verts * R).tolist(), [], quads.tolist())
me.validate()

ROOT = bpy.data.objects.new("Halloween_Planet_ROOT", None)
ROOT["radius"] = R
scene.collection.objects.link(ROOT)
base = bpy.data.collections.new("Planet_Base")
scene.collection.children.link(base)
ob = bpy.data.objects.new("HW_Planet_Surface", me)
base.objects.link(ob)
ob.parent = ROOT

# Simplificar parejo: si se protege la cara, el interior queda en triángulos enormes que atraviesan la cáscara
mod = ob.modifiers.new("Decimate", "DECIMATE")
mod.ratio = min(1.0, TARGET_TRIS / (2 * len(quads)))
dg = bpy.context.evaluated_depsgraph_get()
low = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
ob.modifiers.remove(mod)
ob.data = low
bpy.data.meshes.remove(me)
me = low
me.name = "HW_Planet_Surface"
for m in MATS:
    me.materials.append(m)

# --- Colores por cara --------------------------------------------------------------------------------------
nf = len(me.polygons)
cen = np.empty(nf * 3, np.float32)
me.polygons.foreach_get("center", cen)
p = cen.reshape(-1, 3).astype(np.float64) / R
pm = mirror(p)
pd = p / np.linalg.norm(p, axis=1)[:, None]
b_f = body(p)
inner = b_f < -0.6 * SHELL
hole_d = np.maximum(holes(p), np.minimum(b_f + 0.07, p[:, 1] + 0.7))   # El mismo corte que en planet()
hole_wall = (np.abs(hole_d) < 0.025) & (brow(p) > 0.03) & (b_f < -0.03)   # Paredes oscuras solo adentro de la órbita (no en el borde ni la ceja)
tooth = teeth(p) < 0.012


def crack_net(n_cells, seed_off, width, keep_thr):
    """Grietas: bordes entre celdas al azar sobre la esfera, curvados en arcos por una deformación amplia y suave
    (más un poco de quiebre fino) y cortados por ruido para que no formen una red completa."""
    cells = rng.normal(size=(n_cells, 3))
    cells /= np.linalg.norm(cells, axis=1)[:, None]
    big = np.stack([fbm(pd, 2.0, 2, o + seed_off) for o in (0.0, 31.0, 57.0)], axis=1) - 0.5
    fine = np.stack([vnoise(pd, 9, o + seed_off) for o in (5.0, 41.0, 77.0)], axis=1) - 0.5
    warp = pd + 0.35 * big + 0.02 * fine
    dc = np.sort(np.linalg.norm(warp[:, None, :] - cells[None], axis=2), axis=1)
    keep = vnoise(pd, 2.5, 91.0 + seed_off) > keep_thr
    return dc[:, 1] - dc[:, 0], keep


# Afuera: estratos horizontales como en el Gran Cañón (bandas onduladas, cada una con un tono vecino)
h = p[:, 2] + 0.07 * (fbm(p, 3.0, 2, 7.0) - 0.5) + 0.02 * (vnoise(p, 14.0, 3.0) - 0.5)
band = np.floor(h / 0.045).astype(np.int64)
order = rng.permutation(np.resize(np.arange(len(STRATA)), 96))
idx = M[STRATA[0]] + order[band % 96]

# Pocas grietas largas en el cráneo; la cara queda limpia
front = pd[:, 1] < -0.55
crack, keep = crack_net(16, 0.0, 0.014, 0.55)
keep &= ~front & ~inner
is_lava = (crack < 0.014) & keep
idx = np.where((crack < 0.04) & keep, M[RUST_D], idx)



# Adentro: roca oscura con venas verdes, que se ven brillar por los ojos, la nariz y la boca
vein, vkeep = crack_net(60, 200.0, 0.02, 0.3)
idx = np.where(inner, M[INNER], idx)
veins = inner & vkeep & (hole_d > 0.08)                 # Sin venas cerca de las aberturas
idx = np.where(veins & (vein < 0.03), M[GLOW], idx)
idx = np.where(veins & (vein < 0.016), M[GLOW_HOT], idx)
idx = np.where(hole_wall, M[CHAR], idx)
is_lava &= ~hole_wall & ~tooth
idx = np.where(is_lava, M[LAVA], idx)
idx = np.where(tooth, M[TEETH], idx)
idx = np.where(nose_bones(p) < 0.012, M[STRATA[2]], idx)   # Hueso claro contra el hueco oscuro
me.polygons.foreach_set("material_index", idx.astype(np.int32))

# Las grietas de afuera se hunden un poco
nl = len(me.loops)
lv = np.empty(nl, np.int32)
me.loops.foreach_get("vertex_index", lv)
ls = np.empty(nf, np.int32)
me.polygons.foreach_get("loop_start", ls)
lt = np.empty(nf, np.int32)
me.polygons.foreach_get("loop_total", lt)
sink = np.zeros(len(me.vertices), bool)
for f in np.nonzero(is_lava)[0]:
    sink[lv[ls[f]:ls[f] + lt[f]]] = True
vco = np.empty(len(me.vertices) * 3, np.float32)
me.vertices.foreach_get("co", vco)
vco = vco.reshape(-1, 3)
ln = np.linalg.norm(vco, axis=1)
vco[sink] *= ((ln[sink] - 0.3) / ln[sink])[:, None]
me.vertices.foreach_set("co", vco.ravel())
me.update()

# --- Presentación ------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("Halloween_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#07060a"), 1)
scene.world = world
# Luz lateral y desde abajo (nunca plana de frente), un contraluz frío y el resplandor de adentro
for name, rot, col, e in (("Key", (math.radians(60), 0, math.radians(-120)), "#ffd2b0", 4.0),
                          ("Under", (math.radians(150), 0, math.radians(-20)), "#ff6a2a", 1.2),
                          ("Rim", (math.radians(75), 0, math.radians(150)), "#6f86ff", 2.0)):
    ld = bpy.data.lights.new(name, "SUN")
    ld.energy, ld.color = e, srgb(col)
    lo_ = bpy.data.objects.new(name, ld)
    pres.objects.link(lo_)
    lo_.rotation_euler = rot
ld = bpy.data.lights.new("InnerGlow", "POINT")
ld.energy, ld.color, ld.shadow_soft_size = 12000, srgb("#b4d83a"), 3.0
pres.objects.link(bpy.data.objects.new("InnerGlow", ld))
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1200, 800
scene.view_settings.view_transform = "Standard"         # AgX deslava la lava a rosa pálido
for screen in bpy.data.screens:
    for area in screen.areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.color_type = "MATERIAL"

me.calc_loop_triangles()
print("TRIS %-18s %6d (rejilla: %d cuadriláteros)" % (ob.name, len(me.loop_triangles), len(quads)))

os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
os.makedirs(REF_DIR, exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if not os.environ.get("HW_NO_RENDER"):
    for name, loc, tgt in (("front", (0, -105, -4), (0, 0, -4)), ("three_quarter", (-72, -72, 18), (0, 0, -4)),
                           ("side", (105, -10, 0), (0, 0, -4)), ("back", (20, 100, 30), (0, 0, -2)),
                           ("below", (10, -60, -85), (0, 0, -2)), ("mouth", (0, -48, -12), (0, -16, -13))):
        cam.location = loc
        cam.rotation_euler = (Vector(tgt) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = os.path.join(REF_DIR, "halloween_v01_%s.png" % name)
        bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
