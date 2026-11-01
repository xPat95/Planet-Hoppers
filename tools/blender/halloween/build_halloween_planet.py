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
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "planets", "halloween", "halloween_planet.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "halloween")

SEED = 666
R = 22.0             # Radio del cráneo (South Park: 22)
H = 0.013            # Tamaño de celda de la rejilla (en unidades de R)
SHELL = 0.2          # Grosor de la cáscara (en unidades de R, ~4.5 unidades)
TARGET_TRIS = 40000  # Triángulos tras simplificar (afuera y adentro)

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


# Paleta: estratos de roca roja quemada en tonos cercanos, roca oscura en bordes y adentro, y lava roja profunda.
STRATA = [mat("Strata%d" % i, h) for i, h in
          enumerate(("#a3352a", "#ad3d30", "#b74536", "#9e3127", "#b04034", "#ba4b3b"))]   # Rojos cercanos
RUST_D = mat("RustDark", "#6e2019")
CHAR = mat("Char", "#3b1c13")
INNER = mat("Inner", "#0f1a10")                         # Adentro: roca negra verdosa
TEETH = mat("Teeth", "#c87a5e")
LAVA = mat("Lava", "#b50d0b", emit=2.5, rough=0.5)              # Grietas rojo profundo
LAVA_HOT = mat("LavaHot", "#e0261a", emit=3.5, rough=0.5)
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
    d = sd_rbox(p, (0, -0.76, -0.31), (0.012, 0.09, 0.21), 0.011)                      # Tabique: de arriba a abajo,
    for zc in (-0.13, -0.49):                                                           # metido en el hueso y con las
        d = smin(d, sd_ell(p, (0, -0.76, zc), (0.035, 0.09, 0.04)), 0.03)               # puntas ensanchadas
    d = np.minimum(d, sd_ell(pm, (0.07, -0.64, -0.36), (0.016, 0.06, 0.04)))            # Cornetes, chicos y al fondo
    d = np.minimum(d, sd_ell(p, (0, -0.83, -0.44), (0.02, 0.03, 0.014)))                # Espina nasal, chica
    return np.maximum(d, body(p) + 0.03)                  # Siempre por dentro de la cara: nada sobresale


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
    d = smin(d, nose_bones(p), 0.04)                     # Unión suave: el tabique sale del hueso
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
hole_wall = (np.abs(hole_d) < 0.025) & (brow(p) > 0.03) & (b_f < -0.04)   # Paredes oscuras solo adentro de la órbita (no en el borde ni la ceja)
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

# Grieta de la frente: arriba a la izquierda (vista de frente), sube curvándose hacia la coronilla con una rama.
# Más ancha y más honda que las otras: de ella salen los zombies. Se define vista de frente (x, z) y se lleva a
# direcciones sobre el cráneo.
FOREHEAD_PATHS = [[(-0.16, 0.46), (-0.27, 0.56), (-0.23, 0.68), (-0.35, 0.79), (-0.47, 0.85)],
                  [(-0.27, 0.56), (-0.12, 0.64), (-0.06, 0.76)]]


def front_dir(x, z):
    return np.array([x, -math.sqrt(max(0.0, 1 - x * x - z * z)), z])


def forehead_dist(dd):
    """Distancia (sobre la esfera unidad) de cada dirección a la grieta de la frente, con un poco de zigzag."""
    warp = dd + 0.012 * (np.stack([vnoise(dd, 14, o) for o in (3.0, 23.0, 43.0)], axis=1) - 0.5)
    best = np.full(len(dd), 9.0)
    for path in FOREHEAD_PATHS:
        pts = [front_dir(*q) for q in path]
        for a, b in zip(pts, pts[1:]):
            ab = b - a
            t = np.clip(((warp - a) @ ab) / (ab @ ab), 0, 1)
            best = np.minimum(best, np.linalg.norm(warp - (a + t[:, None] * ab), axis=1))
    return best



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
fh = forehead_dist(pd)                                   # La grieta de la frente (de donde salen los zombies)
fh_lava = (fh < 0.016) & ~inner
is_lava |= fh_lava
idx = np.where((fh < 0.034) & ~inner, M[RUST_D], idx)



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
deep = np.zeros(len(me.vertices), bool)                 # La de la frente, más honda
for f in np.nonzero(fh_lava)[0]:
    deep[lv[ls[f]:ls[f] + lt[f]]] = True
vco[sink] *= ((ln[sink] - 0.3) / ln[sink])[:, None]
vco[deep] *= ((ln[deep] - 0.15) / ln[deep])[:, None]
me.vertices.foreach_set("co", vco.ravel())
me.update()

# --- Brujas en las órbitas ---------------------------------------------------------------------------------
# Modelos de tools/blender/halloween/build_witch.py (se ejecuta antes). Cada bruja es una copia que comparte la
# malla de su pose. Las que bailan se paran en el piso de cada órbita (a la vista desde el frente); las que trepan
# se cuelgan del borde de arriba. El piso y el borde se buscan con rayos dentro de la órbita.
WITCH_BLEND = os.path.join(REPO, "assets", "models", "characters", "halloween", "witch.blend")
WITCH_SCALE = 1.9                                        # La bruja mide ~1.25: en el planeta, ~2.4 unidades
with bpy.data.libraries.load(WITCH_BLEND) as (src, dst):
    dst.meshes = [n for n in src.meshes if n.startswith("HW_Witch_")]
WMESH = {m.name.replace("HW_Witch_", ""): m for m in dst.meshes}
witches = bpy.data.collections.new("Witches")
zones = bpy.data.collections.new("Zones")
scene.collection.children.link(zones)
zones.children.link(witches)
ORBIT_HALF_W = ORBIT_B[0] * 1.08                         # Medio ancho de la órbita en su boca


def orbit_point(s, u, dz=0.0, y=-0.82):
    """Punto dentro de la órbita (lado s = ±1), a lo ancho u ∈ [-1, 1] siguiendo su inclinación."""
    c = ORBIT_C * np.array([s, 1, 1])
    ca, sa = math.cos(ORBIT_TILT), math.sin(ORBIT_TILT)
    x = u * ORBIT_HALF_W
    return Vector((c[0] + s * x * ca, y, c[2] - x * sa + dz)) * R      # Esquina de adentro más baja


def floor_spot(s, u, y0=-0.73):
    """Piso de la órbita lo más al frente posible (para que se vea): avanza hacia adentro y se queda con el primer
    golpe que mira hacia arriba y está dentro de la órbita (no en la mejilla ni por la abertura al interior)."""
    for y in np.linspace(y0, y0 + 0.13, 8):
        hit, loc, nrm, _ = ob.ray_cast(orbit_point(s, u, 0.03, y), Vector((0, 0, -1)), distance=0.3 * R)
        if hit and nrm.z > 0.35 and loc.z > (ORBIT_C[2] - 0.22) * R:
            return loc, nrm
    return None


def put(kind, loc, yaw):
    w = bpy.data.objects.new("HW_Witch_%s_%02d" % (kind, len(witches.objects)), WMESH[kind])
    witches.objects.link(w)
    w.parent = ROOT
    w.location = loc
    w.rotation_euler = (0, 0, math.radians(yaw))
    w.scale = (WITCH_SCALE,) * 3


# Cada órbita con su propio arreglo (no en espejo): quién baila dónde, cuántas cuelgan y cuántas salen gateando
LAYOUTS = {
    1: dict(floor=(("Solo_A", -0.55), ("Pair", 0.05), ("Solo_C", 0.62)), hang=(0.35,), creep=(0.05,), wall=(-1,),
            back=(("Solo_B", -0.3), ("Pair", 0.35), ("Solo_A", -0.75)),
            peek=(-0.15,)),
    -1: dict(floor=(("Solo_B", -0.45), ("Solo_A", 0.2), ("Pair", 0.62)), hang=(-0.5, 0.15), creep=(0.55,), wall=(),
             back=(("Solo_C", -0.15), ("Solo_B", 0.42), ("Solo_A", -0.68)),
             peek=(0.4,)),
}
SHADOW = mat("ContactShadow", "#0c0705")                  # Sombra de contacto bajo los pies (las ancla al piso)
shadow_v, shadow_f = [], []


def contact_shadow(loc, nrm, rx_, ry_):
    """Óvalo oscuro sobre el piso, siguiendo su inclinación, apenas levantado."""
    t = Vector((1, 0, 0)) - nrm * nrm.x                  # A lo ancho de la cara del cráneo
    t.normalize()
    b2 = nrm.cross(t)
    c = len(shadow_v)
    shadow_v.append(loc + nrm * 0.04)
    for k in range(10):
        a_ = math.tau * k / 10
        shadow_v.append(loc + nrm * 0.04 + t * rx_ * math.cos(a_) + b2 * ry_ * math.sin(a_))
    shadow_f.extend((c, c + 1 + k, c + 1 + (k + 1) % 10) for k in range(10))


def brow_spot(s, u, dz0=0.9):
    """Borde de arriba de la órbita visto de frente: el hueso justo encima de la abertura (no adentro). dz0 más
    bajo deja buscar más cerca de la esquina de adentro, donde el borde baja."""
    for dz in np.linspace(dz0, 1.3, 13):
        p = orbit_point(s, u, ORBIT_B[2] * dz)
        hit, loc, nrm, _ = ob.ray_cast(Vector((p.x, -1.4 * R, p.z)), Vector((0, 1, 0)), distance=0.8 * R)
        if hit and loc.y < -0.78 * R:
            return loc, nrm
    return None


wrng = np.random.default_rng(SEED + 1)
for s, lay in LAYOUTS.items():
    # Bailando en el piso, un poco más adentro de la órbita
    for kind, u in lay["floor"]:
        spot = floor_spot(s, u)
        if spot:
            put(kind, spot[0], wrng.uniform(-20, 20))
            w = 2.4 if kind == "Pair" else 1.0
            contact_shadow(spot[0], spot[1], 0.32 * WITCH_SCALE * w, 0.22 * WITCH_SCALE)
    # Una fila de atrás, más adentro y entre las de adelante, para que se vea lleno
    for kind, u in lay["back"]:
        spot = floor_spot(s, u, -0.66)
        if spot:
            put(kind, spot[0], wrng.uniform(-30, 30))
            w = 2.4 if kind == "Pair" else 1.0
            contact_shadow(spot[0], spot[1], 0.32 * WITCH_SCALE * w, 0.22 * WITCH_SCALE)
    # Colgadas del borde de arriba (las manos justo debajo del borde)
    for u in lay["hang"]:
        hit, loc, _, _ = ob.ray_cast(orbit_point(s, u, 0.0, -0.73), Vector((0, 0, 1)), distance=0.4 * R)
        if hit:
            put("Climb", loc - Vector((0, 0, 0.15)), wrng.uniform(-10, 10))
    # Saliendo por arriba: gateando sobre el borde de arriba de la órbita, hacia arriba, con las piernas todavía
    # adentro, como trepando para salir
    for u in lay["creep"]:
        spot = brow_spot(s, *u) if isinstance(u, tuple) else brow_spot(s, u)
        if spot:
            loc, nrm = spot
            face = Vector((wrng.uniform(-0.3, 0.3), 0.0, 1.0))
            face = (face - nrm * face.dot(nrm)).normalized()        # Hacia el borde, sobre la superficie
            yax = -face                                             # El modelo mira hacia -Y
            xax = yax.cross(nrm).normalized()
            put("Crawl", loc, 0)
            w_ob = witches.objects[-1]
            w_ob.rotation_euler = Matrix((xax, yax, nrm)).transposed().to_euler()
            contact_shadow(loc, nrm, 0.36 * WITCH_SCALE, 0.16 * WITCH_SCALE)
    # Asomándose de cabeza desde el techo de la órbita (como por una ventana, pero al revés): acostada en el
    # techo, con la cabeza y los hombros afuera del borde
    for u in lay["peek"]:
        for y in np.linspace(-0.82, -0.7, 8):
            hit, loc, nrm, _ = ob.ray_cast(orbit_point(s, u, 0.0, y), Vector((0, 0, 1)), distance=0.4 * R)
            if hit and nrm.z < -0.3:
                put("Peek", loc, 0)
                witches.objects[-1].rotation_euler = (0, math.pi, math.radians(wrng.uniform(-15, 15)))
                break
    # Gateando bien adentro por la pared de arriba de la órbita (hacia el lado d: -1 = la nariz), saliendo
    for d in lay["wall"]:
        o = orbit_point(s, 0.0, 0.0, -0.74)
        for ang in (25, 35, 45, 55, 15):                 # Ángulo sobre la horizontal; la pared, no la abertura
            a_ = math.radians(ang)
            hit, loc, nrm, _ = ob.ray_cast(o, Vector((d * s * math.cos(a_), 0, math.sin(a_))), distance=0.35 * R)
            if hit and loc.y < -0.64 * R:
                break
        if hit and loc.y < -0.64 * R:
            face = Vector((0, -1, 0))
            face = (face - nrm * face.dot(nrm)).normalized()        # Hacia la boca de la órbita
            yax = -face
            xax = yax.cross(nrm).normalized()
            put("Crawl", loc, 0)
            witches.objects[-1].rotation_euler = Matrix((xax, yax, nrm)).transposed().to_euler()
sme = bpy.data.meshes.new("HW_Witch_Shadows")
sme.from_pydata([tuple(v) for v in shadow_v], [], shadow_f)
sme.materials.append(SHADOW)
sob = bpy.data.objects.new("HW_Witch_Shadows", sme)
witches.objects.link(sob)
sob.parent = ROOT
# --- Diablitos bailando en las grietas -----------------------------------------------------------------------
# Modelos de tools/blender/halloween/build_devil.py (se ejecuta antes): solo la mitad de arriba, con el origen en la
# cintura, que queda hundida en la grieta. Van en grupos, bailando, en todas las grietas de afuera (también la de la
# frente), con notas musicales alrededor (ver más abajo).
ZOMBIE_BLEND = os.path.join(REPO, "assets", "models", "characters", "halloween", "devil.blend")   # Diablitos
ZOMBIE_SCALE = 3.5
ZOMBIE_GROUPS = 18                                       # Grupos repartidos por todas las grietas
with bpy.data.libraries.load(ZOMBIE_BLEND) as (src, dst):
    dst.meshes = [n for n in src.meshes if n.startswith("HW_Devil_")]
ZMESH = {m.name.replace("HW_Devil_", ""): m for m in dst.meshes}
zombies = bpy.data.collections.new("Devils")
zones.children.link(zombies)
zrng = np.random.default_rng(SEED + 3)
kinds = ("ArmsUp", "Cactus", "Point", "Sway")
# Grupos en todas las grietas de afuera: centros bien repartidos entre las caras de lava (el más lejano de los ya
# elegidos, cada vez) y de 3 a 5 diablitos alrededor de cada centro, sobre la misma grieta
cand = np.nonzero(is_lava & ~inner)[0]
cdir = pd[cand]
centers = [int(zrng.integers(len(cand)))]
dmin = np.linalg.norm(cdir - cdir[centers[0]], axis=1)
while len(centers) < ZOMBIE_GROUPS and dmin.max() > 0.12:
    centers.append(int(dmin.argmax()))
    dmin = np.minimum(dmin, np.linalg.norm(cdir - cdir[centers[-1]], axis=1))
n_z = 0
for c in centers:
    near = np.nonzero(np.linalg.norm(cdir - cdir[c], axis=1) < 0.11)[0]
    zrng.shuffle(near)
    taken, want = [], int(zrng.integers(3, 6))
    for j in near:
        if len(taken) >= want:
            break
        if any(np.linalg.norm(cdir[j] - cdir[k]) < 0.014 for k in taken):
            continue
        taken.append(j)
        d = cdir[j]
        hit, loc, nrm, _ = ob.ray_cast(Vector(d * 1.6 * R), Vector(-d), distance=1.2 * R)
        if not hit:
            continue
        side = nrm.orthogonal().normalized()
        face = (Matrix.Rotation(zrng.uniform(0, math.tau), 3, nrm) @ side).normalized()
        yax = -face                                       # El modelo mira hacia -Y
        xax = yax.cross(nrm).normalized()
        kind = kinds[n_z % len(kinds)]
        z = bpy.data.objects.new("HW_Devil_%s_%02d" % (kind, n_z), ZMESH[kind])
        zombies.objects.link(z)
        z.parent = ROOT
        z.location = loc
        z.rotation_euler = Matrix((xax, yax, nrm)).transposed().to_euler()
        z.scale = (ZOMBIE_SCALE * zrng.uniform(0.9, 1.1),) * 3
        n_z += 1
print("DEVILS %d" % len(zombies.objects))

# En el planeta las brujas toman la luz de la escena (los diablitos guardan su brillo para destacar sobre el cráneo) (con su luz propia de ilustración se veían pegadas encima)
for m in bpy.data.materials:
    if m.name.startswith("HW_Witch") and "Eyes" not in m.name and m.node_tree:   # (no la sombra)
        m.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 0.18
print("WITCHES %d" % (len(witches.objects) - 1))

# --- Serpientes que salen de la boca -----------------------------------------------------------------------
# Pocas, finas y con escamas, cabeza redonda con ojitos y bailando (la parte de adelante se menea), con la cola
# apoyada en el cráneo: casi todas gatean desde adentro de la boca por la
# mandíbula de abajo y levantan la cabeza hacia quien mira; otras cuelgan de los dientes de abajo o de arriba. Verde oscuro mate, en
# anillos alternados: escamas un poco más gruesas y oscuras entre bandas más claras.
SNAKE = mat("Snake", "#3d7a2e", rough=0.8)
SNAKE_D = mat("SnakeScales", "#24521f", rough=0.8)
SNAKE_SIDES, SNAKE_SEGS = 5, 22
SNAKE_EYE = mat("SnakeEyes", "#0a0a0a", rough=0.3)
snake_heads = []                                         # Dónde quedan las cabezas (para las notas)

snrng = np.random.default_rng(SEED + 5)


def jaw_open(q):
    """Lo contrario de jaw_closed: de la mandíbula cerrada a su posición abierta en el mundo."""
    q = q - JAW_PIVOT
    c, s_ = math.cos(JAW_OPEN), math.sin(JAW_OPEN)
    return np.stack([q[:, 0], q[:, 1] * c - q[:, 2] * s_, q[:, 1] * s_ + q[:, 2] * c], axis=1) + JAW_PIVOT + JAW_SLIDE


def surf(p, dz):
    """Punto de la superficie del cráneo bajo (dz < 0) o sobre (dz > 0) p, buscando con un rayo vertical."""
    hit, loc, _, _ = ob.ray_cast(Vector(p * R), Vector((0, 0, np.sign(dz))), distance=abs(dz) * R)
    return np.array(loc) / R if hit else None


def snake_path(kind, x):
    """Camino de una serpiente: la cola apoyada en el cráneo (dientes y encía) y la parte de adelante mirando hacia
    quien mira. kind: "crawl" (gatea sobre la mandíbula de abajo y levanta la cabeza), "hang_low" (cuelga por
    delante de los dientes de abajo) o "hang_up" (cuelga de los dientes de arriba)."""
    pts, r = [], 0.007
    if kind == "hang_up":                                 # Cola pegada bajo los dientes de arriba, hacia adentro
        for u in np.linspace(0.1, 0.0, 6):
            q = surf(np.array([x, arch(abs(x), -0.84) + u, -0.75]), 0.2)
            if q is not None:
                pts.append(q - np.array([0, 0, r]))
        if not pts:
            return None
        end = pts[-1]
        hang = snrng.uniform(0.05, 0.08)
        for k in range(1, 6):                             # Cuelga y la cabeza gira hacia adelante
            t = k / 5
            pts.append(end + np.array([0, -0.03 * t * t, -hang * math.sin(t * math.pi / 2)]))
        for k in range(1, 4):
            pts.append(pts[-1] + np.array([0, -0.02, 0.005 * k]))
        return pts
    g = jaw_open(np.array([[x, arch(abs(x), -0.8), -0.74]]))[0]
    z = None
    for u in np.linspace(0.14, -0.03, 10):                # De adentro de la boca hasta delante de los dientes
        q = surf(g + np.array([0, u, 0.1]), -0.25)
        if q is None:
            if z is None:
                continue
            q = np.array([g[0], g[1] + u, z - 0.01])      # Por el hueco de la U: la cola cae hacia adentro
        z = q[2]
        pts.append(q + np.array([0, 0, r]))
    if len(pts) < 3:
        return None
    end = pts[-1]
    drift = snrng.uniform(-0.15, 0.15)
    d = np.array([x * 0.8 + drift, -1.0, 0.0])
    d /= np.linalg.norm(d)
    sd_ = np.array([-d[1], d[0], 0.0])
    if kind == "hang_low":                                # Por delante de los dientes, colgando por la mandíbula
        for k in range(1, 4):
            pts.append(end + np.array([0.01 * drift * k, -0.01 * k, -0.02 * k]))
        end = pts[-1]
        for k in range(1, 4):
            pts.append(end + np.array([0.02 * drift * k, -0.02 * k, 0.008 * k * k / 3]))
    else:                                                 # Hacia adelante, hacia quien mira: cae un poco por su
        L = snrng.uniform(0.06, 0.11)                     # peso y levanta apenas la cabeza (solo se asoman)
        amp, ph = snrng.uniform(0.012, 0.02), snrng.uniform(0, math.tau)
        for k in range(1, 9):
            t = k / 8
            pts.append(end + d * L * t + sd_ * amp * math.sin(math.tau * 1.6 * t + ph) * t
                       + np.array([0, 0, L * (-0.12 * math.sin(math.pi * t) + 0.1 * t ** 3)]))
    return pts


kinds_s = ["crawl"] * 10 + ["hang_low"] * 4 + ["hang_up"] * 4
sv, sf, sm = [], [], []
xs_s = np.linspace(-0.2, 0.2, len(kinds_s)) + snrng.uniform(-0.01, 0.01, len(kinds_s))   # Repartidas a lo ancho
snrng.shuffle(xs_s)
for i, kind in enumerate(kinds_s):
    path = snake_path(kind, xs_s[i])
    if path is None:
        continue
    raw = np.array(path)
    for _ in range(2):                                    # Suavizar el camino
        raw[1:-1] = (raw[:-2] + 2 * raw[1:-1] + raw[2:]) / 4
    def resample(path, n):
        sl = np.r_[0, np.cumsum(np.linalg.norm(np.diff(path, axis=0), axis=1))]
        tt = np.linspace(0, sl[-1], n)
        return np.stack([np.interp(tt, sl, path[:, c]) for c in range(3)], axis=1), tt

    def push_out(q, clear=0.03):
        """Que ningún punto quede dentro del cráneo (dientes incluidos): se empuja hacia afuera siguiendo la
        distancia a la superficie, con margen para el ruido de roca de la malla."""
        for _ in range(8):
            dist = planet(q)
            bad = dist < clear
            if not bad.any():
                break
            e = 0.004
            grad = np.stack([(planet(q + e * ax) - planet(q - e * ax)) / (2 * e) for ax in np.eye(3)], axis=1)
            grad /= np.maximum(np.linalg.norm(grad, axis=1), 1e-9)[:, None]
            q[bad] += grad[bad] * (clear - dist[bad])[:, None]
        return q
    dense, _ = resample(raw, 60)                          # Revisión fina: pasa por encima de los dientes
    dense = push_out(dense)
    for _ in range(2):
        dense[1:-1] = (dense[:-2] + 2 * dense[1:-1] + dense[2:]) / 4
    dense = push_out(dense)
    pts, t = resample(dense, SNAKE_SEGS)
    pts = push_out(pts)

    def pierces(q, rad):
        """Revisión con la malla de verdad (tiene ruido de roca que la forma ideal no tiene): rayos por el centro y
        por los bordes del cuerpo de la serpiente."""
        offs = [np.zeros(3)] + [rad * np.array(v) for v in ((1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1), (0, 1, 0))]
        for o in offs:
            for i0 in range(len(q) - 1):
                a0, b0 = (q[i0] + o) * R, (q[i0 + 1] + o) * R
                seg = Vector(b0 - a0)
                if seg.length < 1e-6:
                    continue
                hit, _, _, _ = ob.ray_cast(Vector(a0), seg.normalized(), distance=seg.length)
                if hit:
                    return True
        return False
    clear = 0.03
    while pierces(pts, 0.012) and clear < 0.08:          # Si todavía atraviesa, más margen
        clear += 0.012
        pts = push_out(pts, clear)
    if pierces(pts, 0.012):
        continue                                          # Si no hay forma, esa serpiente no va
    # Solo la cabeza: los últimos tramos giran de a poco hasta mirar de frente a quien mira (no hacia arriba)
    nh = 6
    head_pts = pts.copy()
    step = np.linalg.norm(pts[-1] - pts[-2])
    prev = pts[-nh - 1] - pts[-nh - 2]
    prev /= np.linalg.norm(prev)
    look = np.array([0.0, -1.0, -0.05])
    look /= np.linalg.norm(look)
    for k in range(1, nh + 1):
        tk = k / nh
        dk = (1 - tk) * prev + tk * look
        head_pts[-nh - 1 + k] = head_pts[-nh - 2 + k] + dk / np.linalg.norm(dk) * step
    if not pierces(head_pts, 0.012):
        pts = head_pts
    # Bailando: la parte de adelante se menea en S hacia un costado y sube y baja un poco (la cola no se mueve)
    nd = 9
    dance = pts.copy()
    fwd = pts[-1] - pts[-nd - 1]
    fwd /= np.linalg.norm(fwd)
    side_d = np.cross(fwd, (0.0, 0.0, 1.0))
    if np.linalg.norm(side_d) < 0.2:
        side_d = np.array([1.0, 0.0, 0.0])
    side_d /= np.linalg.norm(side_d)
    amp_d, ph_d = snrng.uniform(0.012, 0.02) * (1 if i % 2 else -1), snrng.uniform(0, math.pi)
    for k in range(1, nd + 1):
        tk = k / nd
        dance[-nd - 1 + k] += side_d * amp_d * math.sin(math.pi * 1.5 * tk) * tk             + np.array([0, 0, 0.008 * math.sin(math.pi * tk + ph_d) * tk])
    if not pierces(dance, 0.012):
        pts = dance
    # La cola tiene que tocar la boca: se busca la superficie más cercana (arriba para las que cuelgan del paladar,
    # abajo para las demás) y los primeros tramos se pegan a ella, con la punta un poco hundida en el hueso
    dz = 1.0 if kind == "hang_up" else -1.0
    hit, loc, _, _ = ob.ray_cast(Vector(pts[0] * R), Vector((0, 0, dz)), distance=0.15 * R)
    if hit:
        v = np.array(loc) / R - pts[0] + np.array([0, 0, dz * 0.006])
        for k, w_ in enumerate((1.0, 0.65, 0.35, 0.12)):
            pts[k] += v * w_
    up = np.array([0.0, 0.0, 1.0])
    r0 = snrng.uniform(0.005, 0.007)
    tq = t / t[-1]
    radius = r0 * (0.5 + 0.5 * np.minimum(tq * 3, 1)) * np.where(np.arange(SNAKE_SEGS) % 2 == 0, 1.15, 0.95)
    radius[-2] *= 1.15
    base = len(sv)
    for j in range(SNAKE_SEGS):                           # Anillos perpendiculares al camino
        tan = pts[min(j + 1, SNAKE_SEGS - 1)] - pts[max(j - 1, 0)]
        tan /= np.linalg.norm(tan)
        a_ = np.cross(tan, up)
        if np.linalg.norm(a_) < 0.2:                      # Tramo casi vertical (colgando)
            a_ = np.cross(tan, (0.0, 1.0, 0.0))
        a_ /= np.linalg.norm(a_)
        b_ = np.cross(a_, tan)
        for k in range(SNAKE_SIDES):
            ang = math.tau * k / SNAKE_SIDES + (j % 2) * math.pi / SNAKE_SIDES   # Anillos girados: más escamoso
            sv.append(pts[j] + radius[j] * (math.cos(ang) * a_ + math.sin(ang) * b_))
    for j in range(SNAKE_SEGS - 1):
        for k in range(SNAKE_SIDES):
            k1 = (k + 1) % SNAKE_SIDES
            r_, s2 = base + j * SNAKE_SIDES, base + (j + 1) * SNAKE_SIDES
            sf.append((r_ + k, r_ + k1, s2 + k1, s2 + k))
            sm.append(1 if j % 2 == 0 else 0)
    last = base + (SNAKE_SEGS - 1) * SNAKE_SIDES          # Tapa el final del cuerpo (queda dentro de la cabeza)
    tip = len(sv)
    sv.append(pts[-1].copy())
    for k in range(SNAKE_SIDES):
        sf.append((last + k, last + (k + 1) % SNAKE_SIDES, tip))
        sm.append(1)

    def blob(c, axes, radii, m, seg, rings):
        """Elipsoide con ejes propios (para la cabeza y los ojos)."""
        top = len(sv)
        sv.append(c + axes[2] * radii[2])
        for ii in range(1, rings):
            th = math.pi * ii / rings
            for kk in range(seg):
                ph2 = math.tau * kk / seg
                sv.append(c + axes[0] * radii[0] * math.sin(th) * math.cos(ph2)
                          + axes[1] * radii[1] * math.sin(th) * math.sin(ph2) + axes[2] * radii[2] * math.cos(th))
        bot = len(sv)
        sv.append(c - axes[2] * radii[2])
        for kk in range(seg):
            k1 = (kk + 1) % seg
            sf.append((top, top + 1 + k1, top + 1 + kk))
            sf.append((bot, top + 1 + (rings - 2) * seg + kk, top + 1 + (rings - 2) * seg + k1))
            sm.extend([m, m])
        for ii in range(rings - 2):
            for kk in range(seg):
                k1 = (kk + 1) % seg
                a0, a1 = top + 1 + ii * seg, top + 1 + (ii + 1) * seg
                sf.append((a0 + kk, a0 + k1, a1 + k1, a1 + kk))
                sm.append(m)
    # Cabeza redonda (un óvalo un poco más ancho que el cuerpo) y dos ojitos negros a los lados
    hc = pts[-1] + tan * r0 * 1.2
    blob(hc, (a_, b_, tan), (r0 * 1.9, r0 * 1.6, r0 * 2.3), 0, 5, 4)
    closed = i % 2 == 0                                   # La mitad, con los ojos cerrados (rayita)
    for sg in (-1, 1):
        eye_r = (r0 * 0.55, r0 * 0.12, r0 * 0.3) if closed else (r0 * 0.45,) * 3
        blob(hc + a_ * sg * r0 * 1.5 + b_ * r0 * 0.6 + tan * r0 * 0.9, (a_, b_, tan), eye_r, 2, 4, 3)
    snake_heads.append((hc.copy(), tan.copy()))
sme3 = bpy.data.meshes.new("HW_Mouth_Snakes")
sme3.from_pydata((np.array(sv) * R).tolist(), [], sf)
sme3.validate()
sme3.materials.append(SNAKE)
sme3.materials.append(SNAKE_D)
sme3.materials.append(SNAKE_EYE)
sme3.polygons.foreach_set("material_index", sm)
sme3.polygons.foreach_set("use_smooth", np.ones(len(sme3.polygons), bool))
snake_ob = bpy.data.objects.new("HW_Mouth_Snakes", sme3)
zones.objects.link(snake_ob)
snake_ob.parent = ROOT
snake_ob.visible_shadow = False                          # Sin sombras rayando la cara
sme3.calc_loop_triangles()
print("TRIS %-18s %6d" % (snake_ob.name, len(sme3.loop_triangles)))

# --- Notas musicales alrededor de las brujas y los diablitos que bailan --------------------------------------------------
# Notas chicas que brillan, flotando a distintas alturas alrededor de cada bailarina, de frente a quien mira:
# corcheas (cabeza, palito y bandera) y algunas dobles unidas por una barra.
NOTE = mat("MusicNote", "#ffe58a", emit=4.0, rough=0.5)
nrng = np.random.default_rng(SEED + 6)
nv, nf_ = [], []


def note_box(c, rot, half):
    base = len(nv)
    for sx in (-1, 1):
        for sy in (-1, 1):
            for sz in (-1, 1):
                nv.append(c + rot @ Vector((sx * half[0], sy * half[1], sz * half[2])))
    for q in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)):
        nf_.append(tuple(base + i for i in q))


NOTE_SCALE = 0.3                                         # ~0.2 de alto: mucho más chicas que las brujas (~2.4)


def music_note(c, rot, double):
    """Corchea (o dos unidas) en el plano XZ del giro rot; mide ~0.7 * NOTE_SCALE de alto."""
    rot = rot @ Matrix.Scale(NOTE_SCALE, 3)
    heads = [Vector((0, 0, 0))] + ([Vector((0.32, 0, 0.08))] if double else [])
    for h in heads:
        note_box(c + rot @ h, rot @ Matrix.Rotation(math.radians(-25), 3, "Y"), (0.11, 0.03, 0.075))   # Cabeza
        note_box(c + rot @ (h + Vector((0.09, 0, 0.3))), rot, (0.018, 0.02, 0.3))                      # Palito
    if double:                                            # Barra que une las dos
        note_box(c + rot @ Vector((0.25, 0, 0.62)), rot @ Matrix.Rotation(math.radians(-14), 3, "Y"),
                 (0.19, 0.022, 0.035))
    else:                                                 # Bandera
        note_box(c + rot @ Vector((0.17, 0, 0.5)), rot @ Matrix.Rotation(math.radians(40), 3, "Y"),
                 (0.11, 0.02, 0.03))


for w in list(witches.objects):
    if not any(k in w.name for k in ("Solo", "Pair")):
        continue
    for _ in range(int(nrng.integers(2, 4))):
        c = w.location + Vector((nrng.uniform(-0.9, 0.9), nrng.uniform(-0.3, 0.1), nrng.uniform(1.3, 2.7)))
        if planet(np.array([c]) / R)[0] < 0.03:          # Que no quede dentro del cráneo
            continue
        rot = Matrix.Rotation(math.radians(nrng.uniform(-25, 25)), 3, "Y") @             Matrix.Rotation(math.radians(nrng.uniform(-20, 20)), 3, "Z")
        music_note(c, rot, nrng.random() < 0.3)
for w in list(zombies.objects):                         # Y alrededor de los diablitos, siguiendo su inclinación
    drot = w.rotation_euler.to_matrix()
    sc = w.scale[0]
    for _ in range(int(nrng.integers(1, 3))):
        ang, rad = nrng.uniform(0, math.tau), nrng.uniform(0.32, 0.48)      # Más lejos: alrededor, no encima
        c = w.location + drot @ (Vector((rad * math.cos(ang), rad * math.sin(ang) * 0.6,
                                         nrng.uniform(0.4, 0.7))) * sc)
        if planet(np.array([c]) / R)[0] < 0.02:
            continue
        rot = drot @ Matrix.Rotation(math.radians(nrng.uniform(-25, 25)), 3, "Y") @             Matrix.Rotation(math.radians(nrng.uniform(-20, 20)), 3, "Z")
        music_note(c, rot, nrng.random() < 0.3)
for hc, tan in snake_heads:                              # Y una junto a cada serpiente, arriba de la cabeza
    c = Vector(hc * R) + Vector((nrng.uniform(-0.5, 0.5), nrng.uniform(-0.4, 0.0), nrng.uniform(0.4, 0.7)))
    if planet(np.array([c]) / R)[0] < 0.02:
        continue
    rot = Matrix.Rotation(math.radians(nrng.uniform(-25, 25)), 3, "Y")
    music_note(c, rot, False)
nme = bpy.data.meshes.new("HW_Music_Notes")
nme.from_pydata([tuple(v) for v in nv], [], nf_)
nme.validate()
nme.materials.append(NOTE)
note_ob = bpy.data.objects.new("HW_Music_Notes", nme)
witches.objects.link(note_ob)
note_ob.parent = ROOT
note_ob.visible_shadow = False
nme.calc_loop_triangles()
print("TRIS %-18s %6d" % (note_ob.name, len(nme.loop_triangles)))

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
