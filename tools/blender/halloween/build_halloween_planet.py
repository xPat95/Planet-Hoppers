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

import bmesh
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
H = 0.01             # Tamaño de celda de la rejilla (en unidades de R)
SHELL = 0.2          # Grosor de la cáscara (en unidades de R, ~4.5 unidades)
TARGET_TRIS = 75000  # Triángulos tras simplificar (afuera y adentro)

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
# El cráneo es rojo sangre y brillante, como mojado (el mismo rojo que tenía la sangre); los estratos quedan todos
# del mismo color, así que la superficie se lee lisa y lustrosa, no como roca
STRATA = [mat("Strata%d" % i, "#c4001f", emit=0.25, rough=0.08) for i in range(6)]
RUST_D = mat("RustDark", "#7a0012", emit=0.15, rough=0.12)   # Alrededor de las grietas: rojo más oscuro, también mojado
CHAR = mat("Char", "#060406")                            # Paredes de las órbitas y la nariz: negras
INNER = mat("Inner", "#0f1a10")                         # Adentro, en las zonas verdes: roca negra verdosa
INNER_CRUST = mat("InnerCrust", "#2e0905")               # Adentro: costra de lava enfriada, rojo casi negro
INNER_LAVA = mat("InnerLava", "#c22a0c", emit=1.6, rough=0.5)   # Lava que corre entre las costras
TEETH = mat("Teeth", "#c4001f", emit=0.25, rough=0.08)   # Del mismo rojo brillante que el cráneo
LAVA = mat("Lava", "#0b0709", rough=0.06)                       # Grietas: negras y brillantes, como la sangre
LAVA_HOT = mat("LavaHot", "#e0261a", emit=3.5, rough=0.5)
GLOW = mat("InnerGlow", "#8fd61e", emit=1.4, rough=0.5)  # Lava verde (en pocas zonas de adentro)
GLOW_HOT = mat("InnerGlowHot", "#ff7a14", emit=2.5, rough=0.5)  # El centro de la lava: naranja que quema
MATS = STRATA + [RUST_D, CHAR, INNER, TEETH, LAVA, LAVA_HOT, GLOW, GLOW_HOT, INNER_CRUST, INNER_LAVA]
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


FOREHEAD_PATHS = [[(-0.16, 0.46), (-0.27, 0.56), (-0.23, 0.68), (-0.35, 0.79), (-0.47, 0.85)],
                  [(-0.27, 0.56), (-0.12, 0.64), (-0.06, 0.76)]]


def front_dir(x, z):
    return np.array([x, -math.sqrt(max(0.0, 1 - x * x - z * z)), z])


def forehead_dist(dd, paths=FOREHEAD_PATHS):
    """Distancia (sobre la esfera unidad) de cada dirección a una grieta dibujada de frente, con un poco de zigzag."""
    warp = dd + 0.012 * (np.stack([vnoise(dd, 14, o) for o in (3.0, 23.0, 43.0)], axis=1) - 0.5)
    best = np.full(len(dd), 9.0)
    for path in paths:
        pts = [front_dir(*q) for q in path]
        for a, b in zip(pts, pts[1:]):
            ab = b - a
            t = np.clip(((warp - a) @ ab) / (ab @ ab), 0, 1)
            best = np.minimum(best, np.linalg.norm(warp - (a + t[:, None] * ab), axis=1))
    return best


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


def surface_nets(F, lo=LO, h=H):
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
    verts = lo + h * acc / n[:, None]
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

# Relieve por ruido y cráteres (los cráteres solo afuera y lejos de la cara). Liso, como cubierto de líquido: solo
# ondas grandes y suaves. Es una función porque el líquido que se escurre (más abajo) se apoya en esta superficie
craters = rng.normal(size=(40, 3))
craters /= np.linalg.norm(craters, axis=1)[:, None]
craters = craters[(craters[:, 1] > -0.3) & (craters[:, 2] > -0.5)][:12]
crater_rho = [rng.uniform(0.07, 0.15) for _ in craters]      # Radio angular de cada uno


def relief(pts):
    dirs_ = pts / np.linalg.norm(pts, axis=1)[:, None]
    outer_w = np.clip((body(pts) + SHELL) / SHELL, 0, 1)     # 1 en la cara exterior, 0 en la interior
    d = 0.022 * (fbm(dirs_, 3.0, 2, 1.7) - 0.5) + 0.004 * (vnoise(dirs_, 9.0, 5.1) - 0.5)
    for c, rho in zip(craters[:0], crater_rho):         # Sin cráteres: la superficie queda lisa, como mojada
        a_ = np.arccos(np.clip(dirs_ @ c, -1, 1)) / rho
        d += outer_w * np.where(a_ < 1, -0.2 * rho * (1 - a_ * a_), 0.07 * rho * np.exp(-((a_ - 1) / 0.3) ** 2))
    return d * np.clip(holes(pts) / 0.1, 0.15, 1)           # Casi sin relieve en los bordes de las aberturas


dirs = verts / np.linalg.norm(verts, axis=1)[:, None]
verts = verts + dirs * relief(verts)[:, None]

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

# Borde liso entre el hueso rojo y las paredes negras de las órbitas y la nariz: la malla se corta justo por la línea
# donde empieza la pared (si no, cada cara es toda roja o toda negra y el borde sale serruchado)
def wall_field(q):
    b_ = body(q)
    hd = np.maximum(holes(q), np.minimum(b_ + 0.07, q[:, 1] + 0.7))
    return np.maximum.reduce([np.abs(hd) - 0.025, 0.03 - brow(q), b_ + 0.04])   # Negativo: pared


bm_w = bmesh.new()
bm_w.from_mesh(me)
wf = wall_field(np.array([tuple(v.co) for v in bm_w.verts]) / R)
cuts = []
for e in list(bm_w.edges):
    va, vb = e.verts
    fa_, fb_ = wf[va.index], wf[vb.index]
    if fa_ * fb_ < 0:
        cuts.append((e, va, vb, fa_ / (fa_ - fb_)))
new_vs = []
for e, va, vb, t in cuts:
    pa, pb = va.co.copy(), vb.co.copy()
    _, nv = bmesh.utils.edge_split(e, va, t)
    nv.co = pa.lerp(pb, t)
    new_vs.append(nv)
bmesh.ops.connect_verts(bm_w, verts=new_vs)
bmesh.ops.triangulate(bm_w, faces=[f for f in bm_w.faces if len(f.verts) > 4])
bm_w.to_mesh(me)
bm_w.free()
me.update()
print("BORDE PAREDES: %d cortes" % len(cuts))

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



# Adentro: un mar de lava que se ve brillar por los ojos, la nariz y la boca. Costras oscuras flotando sobre lava
# roja que corre entre ellas, con el centro naranja; en unas pocas zonas la lava es verde
vein, vkeep = crack_net(60, 200.0, 0.02, 0.3)
# Pocas zonas verdes, puestas a mano: una chica que se asoma por el ojo izquierdo y las otras fuera de la vista
GREEN_SPOTS = [((0.3, 0.2, 0.93), 0.45), ((0.7, 0.1, -0.7), 0.4)]
# La que se asoma: donde llega la vista de frente por la parte de arriba del ojo izquierdo (la primera cara de adentro)
o_ = Vector((-0.27 * R, -2.0 * R, -0.03 * R))
for _ in range(12):
    hit_, loc_, nrm_, fi_ = ob.ray_cast(o_, Vector((0, 1, 0)))
    if not hit_:
        break
    if inner[fi_]:
        GREEN_SPOTS.append((tuple(loc_ / R), 0.2))
        break
    o_ = loc_ + Vector((0, 0.01 * R, 0))
gd = np.full(len(pd), 9.0)
for c_, r_ in GREEN_SPOTS:
    c_ = np.array(c_) / np.linalg.norm(c_)
    gd = np.minimum(gd, np.linalg.norm(pd - c_, axis=1) / r_)
green = inner & (gd + 0.35 * (vnoise(pd, 6.0, 131.0) - 0.5) < 1.0)   # Borde disparejo
idx = np.where(inner, np.where(green, M[INNER], M[INNER_CRUST]), idx)
flow = inner & (hole_d > 0.05)                           # Sin lava pegada a las aberturas
idx = np.where(flow & (vein < 0.05), np.where(green, M[GLOW], M[INNER_LAVA]), idx)
idx = np.where(flow & (vein < 0.02), np.where(green, M[GLOW], M[GLOW_HOT]), idx)
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
# Sombreado suave: la superficie se lee lisa y mojada (las grietas y lo de adentro quedan facetados)
red = {M[m] for m in STRATA + [RUST_D]}
me.polygons.foreach_set("use_smooth", np.isin(idx, list(red)))
me.update()



def surf(p):
    """Punto de la superficie más cercano (en unidades de R), su normal y el material de esa cara."""
    ok, loc, nrm, fi = ob.closest_point_on_mesh(Vector(p * R))
    return np.array(loc) / R, np.array(nrm), me.polygons[fi].material_index

# --- Versión lejana (LOD) -----------------------------------------------------------------------------------
# El mismo cráneo, ya pintado, con muchas menos caras; va oculto en LOD_Far y el motor lo usa cuando el planeta está
# lejos. A esa distancia los personajes, las notas y la sangre no se ven, así que la versión lejana no los lleva.
FAR_TRIS = 22000
lod = bpy.data.collections.new("LOD_Far")
scene.collection.children.link(lod)
fob = bpy.data.objects.new("HW_Planet_Surface_Far", me.copy())
lod.objects.link(fob)
fob.parent = ROOT
me.calc_loop_triangles()
fmod = fob.modifiers.new("Decimate", "DECIMATE")
fmod.ratio = FAR_TRIS / len(me.loop_triangles)
dg = bpy.context.evaluated_depsgraph_get()
fme = bpy.data.meshes.new_from_object(fob.evaluated_get(dg))
fob.modifiers.remove(fmod)
fob.data = fme
fme.name = "HW_Planet_Surface_Far"
fob.hide_viewport = fob.hide_render = True
fme.calc_loop_triangles()
print("TRIS %-18s %6d (lejos)" % (fob.name, len(fme.loop_triangles)))

# --- Brujas en las órbitas ---------------------------------------------------------------------------------
# Modelos de tools/blender/halloween/build_witch.py (se ejecuta antes). Cada bruja es una copia que comparte la
# malla de su pose. Las que bailan se paran en el piso de cada órbita (a la vista desde el frente); las que trepan
# se cuelgan del borde de arriba. El piso y el borde se buscan con rayos dentro de la órbita.
WITCH_BLEND = os.path.join(REPO, "assets", "models", "characters", "halloween", "witch.blend")
WITCH_SCALE = 3.4                                        # La bruja mide ~1.25: en el planeta, ~4.3 unidades
SWING_WITCH_SCALE = 2.6                                  # Las de las hamacas de sangre, a la medida de la curva
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


def put(kind, loc, yaw, scale=WITCH_SCALE):
    w = bpy.data.objects.new("HW_Witch_%s_%02d" % (kind, len(witches.objects)), WMESH[kind])
    witches.objects.link(w)
    w.parent = ROOT
    w.location = loc
    w.rotation_euler = (0, 0, math.radians(yaw))
    w.scale = (scale,) * 3


# Cada órbita con su propio arreglo (no en espejo): pocas y grandes, tres bailando cerca del borde. En una, la pareja
# y una sola; en la otra, tres solas (una de cada pose). La que está sentada va en los dientes de abajo (ver la sangre)
LAYOUTS = {
    1: dict(floor=(("Solo_B", -0.42, 25), ("Pair", 0.38, None)), hang=(), creep=(), wall=(), back=(), peek=(),
            sit=()),
    -1: dict(floor=(("Solo_A", -0.55, None), ("Solo_C", -0.05, None), ("Solo_B", 0.45, -25)), hang=(), creep=(),
             wall=(), back=(), peek=(), sit=()),
}
def lift_feet(w):
    """El piso de la órbita es inclinado: si algún pie queda hundido, sube a la bruja hasta que ninguno lo esté."""
    bpy.context.view_layer.update()
    mw = w.matrix_world
    co = [mw @ v.co for v in w.data.vertices]
    floor_z = min(c.z for c in co)
    feet = [c for c in co if c.z < floor_z + 0.06 * WITCH_SCALE]   # Las suelas
    sink = 0.0
    for c in feet[::3]:
        hit, loc, _, _ = ob.ray_cast(c + Vector((0, 0, 0.5 * WITCH_SCALE)), Vector((0, 0, -1)),
                                     distance=0.8 * WITCH_SCALE)
        if hit:
            sink = max(sink, loc.z - c.z)
    w.location.z += sink


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
    # Bailando en el piso, cerca del borde de la órbita
    for kind, u, yaw in lay["floor"]:                     # yaw: giro a mano (None = un poco al azar)
        spot = floor_spot(s, u, -0.78)
        if spot:
            put(kind, spot[0], yaw if yaw is not None else wrng.uniform(-20, 20))
            lift_feet(witches.objects[-1])
    # Una fila de atrás, más adentro y entre las de adelante, para que se vea lleno
    for kind, u in lay["back"]:
        spot = floor_spot(s, u, -0.66)
        if spot:
            put(kind, spot[0], wrng.uniform(-30, 30))
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
    # Sentadas en el borde de abajo de la órbita, de frente, columpiando las piernas por fuera, sobre la mejilla
    for u, yaw in lay.get("sit", ()):                    # yaw: giro a mano (None = un poco al azar)
        # La cresta del borde de abajo: rayos hacia abajo desde la boca de la órbita, del frente hacia adentro; el
        # punto más alto es el filo entre la mejilla (afuera) y el piso de la órbita (adentro)
        hits = []
        o = orbit_point(s, u, 0.0)
        for yy in np.linspace(-1.0, -0.74, 53):
            hit, loc, _, _ = ob.ray_cast(Vector((o.x, yy * R, o.z)), Vector((0, 0, -1)), distance=0.35 * R)
            if hit:
                hits.append(loc)
        if hits:
            # El filo: el primer punto (desde el frente) que ya llega a la altura del borde; ahí se sienta, con las
            # piernas colgando por fuera, sobre la mejilla
            top = max(h.z for h in hits)
            edge = next(h for h in hits if h.z > top - 0.025 * R)
            # Un poco más afuera, sobre el labio del borde, para que las caderas queden en el filo
            hit, out, _, _ = ob.ray_cast(Vector((edge.x, edge.y - 0.025 * R, edge.z + 0.1 * R)), Vector((0, 0, -1)),
                                         distance=0.3 * R)
            seat = out if hit and out.z > edge.z - 0.03 * R else edge
            put("Sit", seat + Vector((0, 0, 0.05)), yaw if yaw is not None else wrng.uniform(-15, 15))
# --- Diablitos bailando en las grietas -----------------------------------------------------------------------
# Modelos de tools/blender/halloween/build_devil.py (se ejecuta antes): solo la mitad de arriba, con el origen en la
# cintura, que queda hundida en la grieta. Van repartidos, bailando, en todas las grietas de afuera (también la de la
# frente), con notas musicales alrededor (ver más abajo).
ZOMBIE_BLEND = os.path.join(REPO, "assets", "models", "characters", "halloween", "devil.blend")   # Diablitos
ZOMBIE_SCALE = 5.7
DEVIL_COUNT = 38                                         # Repartidos por todas las grietas
with bpy.data.libraries.load(ZOMBIE_BLEND) as (src, dst):
    dst.meshes = [n for n in src.meshes if n.startswith("HW_Devil_")]
ZMESH = {m.name.replace("HW_Devil_", ""): m for m in dst.meshes}
zombies = bpy.data.collections.new("Devils")
zones.children.link(zombies)
zrng = np.random.default_rng(SEED + 3)
kinds = ("ArmsUp", "Cactus", "Point", "Sway")
# Repartidos parejo a lo largo de todas las grietas de afuera: cada uno va en la cara de lava más lejana de los
# que ya están (así no se amontonan y ninguna grieta queda vacía)
cand = np.nonzero(is_lava & ~inner)[0]
cdir = pd[cand]
spots = [int(zrng.integers(len(cand)))]
dmin = np.linalg.norm(cdir - cdir[spots[0]], axis=1)
while len(spots) < DEVIL_COUNT:
    spots.append(int(dmin.argmax()))
    dmin = np.minimum(dmin, np.linalg.norm(cdir - cdir[spots[-1]], axis=1))
n_z = 0
for j in spots:
    d = cdir[j]
    hit, loc, nrm, _ = ob.ray_cast(Vector(d * 1.6 * R), Vector(-d), distance=1.2 * R)
    if not hit:
        continue
    side = nrm.orthogonal().normalized()
    face = (Matrix.Rotation(zrng.uniform(0, math.tau), 3, nrm) @ side).normalized()
    yax = -face                                           # El modelo mira hacia -Y
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
    if m.name.startswith("HW_Witch") and "Eyes" not in m.name and m.node_tree:
        m.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 0.18

# --- Sangre que sale de la boca --------------------------------------------------------------------------------
# Un solo líquido, espeso y brillante (mojado), distinto del rojo mate del cráneo, que sale solo por los dientes de
# abajo. Pensado como líquido de verdad: pesa, así que se junta en un charco grueso sobre los dientes (llena los
# huecos entre ellos) y se derrama por delante como una capa pegada al hueso. La capa se abre en regueros chatos y
# anchos (no tubos), que engordan y se angostan al bajar; donde el mentón dobla hacia abajo el líquido se junta en
# gotas pesadas que cuelgan. De las más cargadas cae un hilo que se afina y se cuenta en cuentas, y abajo se curva
# en una J gorda donde se sientan brujas felices, como en una hamaca. Todo es una sola forma (distancia con signo,
# unida en suave) para que se lea como líquido y no como piezas pegadas.
BLOOD = mat("Blood", "#0b0709", rough=0.06)              # Sangre negra y brillante: se lee sobre el cráneo rojo
BLOOD_H = 0.0035                                         # Rejilla de la sangre (más fina que la del cráneo)
BLOOD_TRIS = 14000
BLOOD_K = 0.014                                          # Qué tanto se funden las partes entre sí
brng = np.random.default_rng(SEED + 5)
chains, drops = [], []                                   # Partes del líquido (en unidades de R)


def jaw_open(q):
    """Lo contrario de jaw_closed: de la mandíbula cerrada a su posición abierta en el mundo."""
    q = q - JAW_PIVOT
    c, s_ = math.cos(JAW_OPEN), math.sin(JAW_OPEN)
    return np.stack([q[:, 0], q[:, 1] * c - q[:, 2] * s_, q[:, 1] * s_ + q[:, 2] * c], axis=1) + JAW_PIVOT + JAW_SLIDE


def sdf_grad(q, e=0.004):
    g = np.stack([(planet(q + e * ax) - planet(q - e * ax)) / (2 * e) for ax in np.eye(3)], axis=1)
    return g / np.maximum(np.linalg.norm(g, axis=1), 1e-9)[:, None]


def blood_tube(pts, radii):
    """Un hilo de líquido por los puntos, con el grosor de cada punto."""
    chains.append((np.asarray(pts, float), np.asarray(radii, float)))


def drop(c, r, stretch=1.6):
    """Gota con forma de lágrima, la punta hacia arriba."""
    drops.append((np.asarray(c, float), r, stretch))


def ragged(x, f, off):
    """Borde disparejo del líquido, entre 0 y 1."""
    return np.clip(0.5 + 0.35 * np.sin(x * f + off) + 0.25 * np.sin(x * f * 2.3 + off * 3.1), 0, 1)


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


POOL_X = 0.27                                            # Hasta dónde llega el charco hacia los costados


def lower_coat(p):
    """Charco sobre las coronas de los dientes de abajo: grueso (el líquido pesa y se junta) y metido en los huecos
    entre dientes; por el frente baja solo un poco, con borde disparejo. Los dientes giran con la mandíbula."""
    q = jaw_closed(p)
    d = sd_rbox(q, *LOWER_TEETH[0], 0.02)
    for c, b in LOWER_TEETH[1:]:
        d = smin(d, sd_rbox(q, c, b, 0.02), 0.02)         # Unidos en suave: el líquido tapa los huecos
    thick = 0.006 + 0.008 * smoothstep(-0.66, -0.61, q[:, 2])   # Más grueso arriba
    # Hacia los costados se va afinando hasta desaparecer (sin cortes: un corte deja puntas)
    thick *= smoothstep(POOL_X, POOL_X - 0.07, np.abs(q[:, 0]) + 0.02 * ragged(q[:, 0], 17.0, 0.9))
    thick *= smoothstep(-0.56, -0.66, q[:, 1])          # Y hacia el fondo de la boca también
    d = np.where(thick > 0.0015, d - thick, 1.0)
    low = -0.643 - 0.025 * ragged(q[:, 0], 27.0, 0.3)    # Hasta dónde baja por el frente de los dientes
    mask = np.maximum(low - q[:, 2], q[:, 1] - arch(np.abs(q[:, 0]), -0.815) - 0.025)
    return -smin(-d, -mask, 0.015)                       # Bordes redondeados, como líquido


def surface_walk(p, min_steps=6):
    """Baja por la superficie desde p hasta que queda debajo de algo (de ahí cae). Devuelve el camino suavizado."""
    path = []
    down = np.array([0.0, 0.0, -1.0])
    for _ in range(120):
        d = planet(p[None])[0]
        n = sdf_grad(p[None])[0]
        p = p - n * (d - 0.012)                          # Sobre la superficie, apenas afuera
        path.append(p.copy())
        if n[2] < -0.55 and len(path) > min_steps:       # Ya está debajo: de acá cae
            break
        t = down - n * down.dot(n)
        p = p + t / max(np.linalg.norm(t), 1e-6) * 0.012
    path = np.array(path)
    for _ in range(3):                                   # Suavizar el tramo pegado al hueso
        path[1:-1] = (path[:-2] + 2 * path[1:-1] + path[2:]) / 4
    return path


# Gotas que cuelgan del borde de la capa (x en el mentón, largo, tipo): "seat" = baja hasta donde termina el mentón y
# ahí se curva en una J con una bruja sentada; "drip" = baja hasta el borde del mentón y cuelga una gota pesada (y
# otra ya se soltó); "short" = se frena a mitad del mentón con la cabeza gorda
STREAMS = [(-0.15, 0.0, "seat"), (-0.075, 0.0, "drip"), (0.0, 0.0, "seat"), (0.075, 0.0, "drip"),
           (0.15, 0.0, "seat")]
SHORT = [(-0.215, 0.27), (0.215, 0.3)]
# Borde de abajo de la capa: en cada gota baja (ahí empieza la gota) y entre gota y gota sube en un arco de media
# luna, como en un dibujo de sangre que chorrea
DRIP_X = np.array(sorted([x_ for x_, _, _ in STREAMS] + [x_ for x_, _ in SHORT]))
DRIP_Z = -0.8 + brng.uniform(-0.012, 0.012, len(DRIP_X))   # En coordenadas de la mandíbula cerrada
ARCH_H = np.array([0.03, 0.062, 0.036, 0.07, 0.028, 0.052])   # Cada arco a su altura (no todos iguales)


def sheet_edge(x):
    """Altura del borde de abajo de la capa en cada x (coordenadas de la mandíbula cerrada)."""
    i = np.clip(np.searchsorted(DRIP_X, x) - 1, 0, len(DRIP_X) - 2)
    t = np.clip((x - DRIP_X[i]) / (DRIP_X[i + 1] - DRIP_X[i]), 0, 1)
    e = DRIP_Z[i] + (DRIP_Z[i + 1] - DRIP_Z[i]) * t + ARCH_H[i] * np.sin(math.pi * t) ** 0.8
    # Más allá de las gotas de los costados el borde sube de a poco hasta los dientes
    tl = smoothstep(DRIP_X[0], -POOL_X, x)
    tr = smoothstep(DRIP_X[-1], POOL_X, x)
    e = np.where(x < DRIP_X[0], DRIP_Z[0] + (-0.7 - DRIP_Z[0]) * tl, e)
    return np.where(x > DRIP_X[-1], DRIP_Z[-1] + (-0.7 - DRIP_Z[-1]) * tr, e)
rivulets = []                                            # Regueros sobre el mentón: (camino, alto, ancho)
seats = []                                               # Dónde se sientan las brujas (punto y giro)
down = np.array([0.0, 0.0, -1.0])
for x, fall, kind in STREAMS + [(x_, l_, "short") for x_, l_ in SHORT]:
    # Arranca en el charco de arriba de los dientes y baja pegado a los dientes, la encía y el mentón
    path = surface_walk(jaw_open(np.array([[x, arch(abs(x), -0.815) - 0.05, -0.63]]))[0])
    seg = np.r_[0, np.cumsum(np.linalg.norm(np.diff(path, axis=0), axis=1))]
    if kind == "short":                                  # Se frena antes: se corta el camino
        keep = max(3, int(np.searchsorted(seg, fall)))
        path, seg = path[:keep], seg[:keep]
    u = seg / seg[-1]
    ph = brng.uniform(0, math.tau)
    wob = 1 + 0.3 * np.sin(seg * 55 + ph) * np.sin(seg * 23 + 2 * ph)    # Engorda y se angosta al bajar
    if kind == "short":                                  # Cabeza gorda y redonda donde se frenó
        gather = 1 + 0.9 * smoothstep(0.6, 1.0, u)
    else:
        gather = 1 + 0.45 * smoothstep(0.65, 1.0, u)    # Se junta antes de colgar
    rivulets.append((path, 0.011 * wob * gather, 0.019 * (0.85 + 0.3 * wob) * gather))
    if kind == "short":
        continue
    end = path[-1]
    if kind == "drip":
        # Gota pesada colgando donde el mentón dobla hacia abajo, unida al reguero por un cuello
        rp = brng.uniform(0.019, 0.023)
        pend = end + down * rp * 1.1
        drop(pend, rp, 1.5)
        drop(pend + down * (0.07 + brng.uniform(0, 0.04)), rp * 0.6, 1.8)   # Una que ya se soltó
        continue
    # J gorda de costado justo donde termina el mentón (para que la bruja se vea de frente): baja, pasa por abajo
    # (el asiento) y sube un poco del otro lado; más pesada abajo, donde se junta el líquido
    rc = brng.uniform(0.044, 0.05)
    side = -1.0 if x < 0 else 1.0
    sdir = np.array([side, 0.0, 0.0])
    c = end + sdir * rc
    angs = np.linspace(0, 0.8 * math.pi, 16)            # Solo hasta sostener a la bruja: un labio corto del otro lado
    curl = [c + rc * (-sdir * math.cos(a) + down * math.sin(a)) for a in angs]
    curl_r = 0.011 + 0.005 * np.sin(np.clip(angs / (0.75 * math.pi), 0, 1) * math.pi / 2)
    curl_r[-4:] *= np.linspace(1.0, 0.85, 4)
    blood_tube([path[-3], path[-2]] + curl, np.r_[0.011, 0.011, curl_r])
    drop(curl[-1] + np.array([0, 0, 0.003]), curl_r[-1] * 1.15, 1.0)   # La punta termina en una bola
    seat_pt = c + down * rc                              # El fondo de la curva
    seats.append((seat_pt, 0.0))
    drop(seat_pt + down * 0.03, 0.017, 1.6)              # Y abajo del asiento se está juntando otra gota


def chin_film(p):
    """Capa pegada a los dientes de abajo y al mentón, con el borde en arcos, y los regueros (las gotas) que bajan
    de ella: chatos y anchos, con los bordes apoyados en el hueso."""
    ds = planet(p)
    q = jaw_closed(p)
    edge = sheet_edge(q[:, 0])
    rim = 0.008 + 0.006 * np.exp(-((q[:, 2] - edge - 0.012) / 0.01) ** 2)   # El borde se engorda (tensión)
    th = rim * smoothstep(edge, edge + 0.01, q[:, 2]) * smoothstep(POOL_X, POOL_X - 0.04, np.abs(q[:, 0]))
    th *= (q[:, 2] < -0.6) & (q[:, 1] < arch(np.abs(q[:, 0]), -0.815))      # Solo el frente, hasta los dientes
    th *= smoothstep(-0.56, -0.66, q[:, 1])             # Sin colarse hacia el fondo por los dientes de los costados
    for path, hh, ww in rivulets:
        best = np.full(len(p), 9.0)
        hb, wb = np.zeros(len(p)), np.ones(len(p))
        for j in range(len(path) - 1):
            a_, b_ = path[j], path[j + 1]
            ba = b_ - a_
            h_ = np.clip((p - a_) @ ba / max(ba @ ba, 1e-12), 0, 1)
            dd = np.linalg.norm(p - a_ - h_[:, None] * ba, axis=1)
            m = dd < best
            best[m] = dd[m]
            hb[m] = hh[j] + (hh[j + 1] - hh[j]) * h_[m]
            wb[m] = ww[j] + (ww[j + 1] - ww[j]) * h_[m]
        # Corte de un reguero: chato y ancho, con los bordes que se apoyan en el hueso
        th = np.maximum(th, hb * np.clip(1 - (best / wb) ** 2, 0, 1))
    th = np.where(th > 0.0015, th, -0.05)
    return np.maximum(ds - th, -ds - 0.02)               # Solo una cáscara por fuera del hueso


# Todo en una rejilla fina alrededor de la boca y el mentón; cada parte se calcula solo en su caja y se une en suave
jaw_box = jaw_open(np.array([(x, y, z) for x in (-0.33, 0.33) for y in (-0.95, -0.6) for z in (-0.76, -0.56)]))
chin_box = jaw_open(np.array([(x, y, z) for x in (-0.33, 0.33) for y in (-1.0, -0.6) for z in (-1.0, -0.58)]))
riv_pts = np.concatenate([r[0] for r in rivulets])
lo_b = [jaw_box.min(0), np.minimum(riv_pts.min(0) - 0.06, chin_box.min(0))]
hi_b = [jaw_box.max(0), np.maximum(riv_pts.max(0) + 0.06, chin_box.max(0))]
for pts, radii in chains:
    lo_b.append(pts.min(0) - radii.max())
    hi_b.append(pts.max(0) + radii.max())
for c, r, st in drops:
    lo_b.append(c - r * 2)
    hi_b.append(c + r * 2.5)
B_LO = np.min(lo_b, axis=0) - 0.03
B_HI = np.max(hi_b, axis=0) + 0.03
b_axes = [np.arange(B_LO[i], B_HI[i] + BLOOD_H, BLOOD_H) for i in range(3)]
BF = np.full([len(ax) for ax in b_axes], 9.0, np.float32)


def box(lo, hi):
    """Los índices de la rejilla dentro de la caja, y los puntos que les tocan."""
    i0 = np.clip(np.floor((np.asarray(lo) - B_LO) / BLOOD_H).astype(int), 0, None)
    i1 = np.minimum(np.ceil((np.asarray(hi) - B_LO) / BLOOD_H).astype(int) + 1, BF.shape)
    sl = tuple(slice(i0[i], i1[i]) for i in range(3))
    g = np.meshgrid(*[b_axes[i][sl[i]] for i in range(3)], indexing="ij")
    return sl, np.stack([gi.ravel() for gi in g], axis=1)


def blend(sl, d, k=BLOOD_K):
    BF[sl] = smin(BF[sl], d.reshape(BF[sl].shape), k)


sl, pp = box(lo_b[0], hi_b[0])
blend(sl, lower_coat(pp))
sl, pp = box(lo_b[1], hi_b[1])
blend(sl, chin_film(pp), 0.02)
for pts, radii in chains:
    sl, _ = box(pts.min(0) - radii.max() - BLOOD_H, pts.max(0) + radii.max() + BLOOD_H)
    loc = np.full(BF[sl].shape, 9.0, np.float32)        # El hilo entero primero (sin hincharse en las uniones)
    off = np.array([x.start for x in sl])
    for a_, b_, ra, rb in zip(pts, pts[1:], radii, radii[1:]):
        ssl, pp = box(np.minimum(a_, b_) - max(ra, rb) - BLOOD_H, np.maximum(a_, b_) + max(ra, rb) + BLOOD_H)
        lsl = tuple(slice(ssl[i].start - off[i], ssl[i].stop - off[i]) for i in range(3))
        ba = b_ - a_
        h_ = np.clip((pp - a_) @ ba / max(ba @ ba, 1e-12), 0, 1)
        d = np.linalg.norm(pp - a_ - h_[:, None] * ba, axis=1) - (ra + (rb - ra) * h_)
        loc[lsl] = np.minimum(loc[lsl], d.reshape(loc[lsl].shape))
    BF[sl] = smin(BF[sl], loc, BLOOD_K)
for c, r, st in drops:
    sl, pp = box(c - r * 2, c + r * (st * 1.4 + 1))
    tip = c + np.array([0, 0, r * st * 1.3])             # Lágrima: bola abajo que se afina hacia la punta
    ba = tip - c
    h_ = np.clip((pp - c) @ ba / (ba @ ba), 0, 1)
    cone = np.linalg.norm(pp - c - h_[:, None] * ba, axis=1) - r * (1 - h_) ** 1.5
    blend(sl, smin(sd_sphere(pp, c, r), cone, r * 0.5), r * 0.6)
bverts, bquads = surface_nets(BF, B_LO, BLOOD_H)
del BF
bme = bpy.data.meshes.new("HW_Chin_Blood")
bme.from_pydata((bverts * R).tolist(), [], bquads.tolist())
bme.validate()
blood_ob = bpy.data.objects.new("HW_Chin_Blood", bme)
zones.objects.link(blood_ob)
blood_ob.parent = ROOT
sm = blood_ob.modifiers.new("Smooth", "SMOOTH")         # Superficie de líquido: lisa, sin escalones de la rejilla
sm.factor, sm.iterations = 0.6, 4
dm = blood_ob.modifiers.new("Decimate", "DECIMATE")
dm.ratio = min(1.0, BLOOD_TRIS / (2 * len(bquads)))
dg = bpy.context.evaluated_depsgraph_get()
low_b = bpy.data.meshes.new_from_object(blood_ob.evaluated_get(dg))
blood_ob.modifiers.clear()
blood_ob.data = low_b
bpy.data.meshes.remove(bme)
bme = low_b
bme.name = "HW_Chin_Blood"
bme.materials.append(BLOOD)
bme.polygons.foreach_set("use_smooth", np.ones(len(bme.polygons), bool))
bme.calc_loop_triangles()
print("TRIS %-18s %6d" % (blood_ob.name, len(bme.loop_triangles)))

# Sangre negra debajo de los ojos, con la misma lógica que la de la boca: desde el borde de abajo de cada órbita la
# sangre cubre el hueso en una capa cuyo borde de abajo es una fila de arcos de media luna (el borde se engorda, porque
# ahí se junta). Los arcos son bajos. De donde se juntan dos arcos baja un reguero que sigue la curva de la cara: fino
# casi todo el largo y abajo se junta en una bola gorda y redonda, como la gota del mentón. Cada ojo con la suya (no en
# espejo): 4 regueros en uno y 3 en el otro, de largos distintos. Igual en la nariz, hasta los dientes
UNDER_EYES = [
    # (x donde empieza y termina la capa, y para cada reguero: x, largo)
    # Solo del borde de abajo de la órbita, lejos del rincón de adentro (nada corre hacia la nariz)
    ((-0.44, -0.13), [(-0.365, 0.075), (-0.3, 0.05), (-0.245, 0.085)]),     # Largo, corto, mediano
    ((0.13, 0.44), [(0.245, 0.06), (0.285, 0.065), (0.37, 0.04)]),         # Corto, el más largo, cortito
    # La nariz: dos regueros debajo de cada abertura, que llegan casi hasta los dientes de arriba (sin tocarlos)
    ((-0.105, -0.02), [(-0.085, 0.045), (-0.042, 0.12)]),    # Uno cortito y uno largo
    ((0.018, 0.105), [(0.045, 0.085), (0.082, 0.11)]),      # Dos medianos, de largos distintos
]
UE_H = 0.0035
down = np.array([0.0, 0.0, -1.0])
tv, tf = [], []


def rim_point(x, z0=-0.1):
    """El borde de abajo de la órbita en x: rayos de frente, de arriba hacia abajo; el primer hueso rojo que
    tocan después de pasar por la abertura."""
    seen_hole = False
    for z in np.arange(z0, -0.5, -0.004):
        hit, loc, nrm, fi = ob.ray_cast(Vector((x * R, -2.0 * R, z * R)), Vector((0, 1, 0)))
        frontal = hit and loc.y / R < -0.77 and me.polygons[fi].material_index in red   # La cara de afuera (no los
        if not frontal:                                                                     # huesitos de adentro)
            seen_hole = True
        elif seen_hole:
            return np.array(loc) / R
    return None


def on_face(x, z):
    hit, loc, nrm, fi = ob.ray_cast(Vector((x * R, -2.0 * R, z * R)), Vector((0, 1, 0)))
    return (np.array(loc) / R, np.array(nrm)) if hit else (None, None)


def walk_down(p, n, length, stick=False):
    """Baja por la cara con la gravedad hasta el largo pedido, o hasta que la cara dobla hacia abajo (ahí cuelga)."""
    path = [p]
    run = 0.0
    while run < length:
        t = down - n * n.dot(down)
        if not stick and (n[2] < -0.8 or np.linalg.norm(t) < 0.12):   # Solo se corta en un borde que dobla
            return np.array(path), True
        if stick:
            # Con inercia: donde el pómulo dobla para abajo la gravedad casi no tira sobre la superficie, así que
            # sigue en la dirección que traía, pegada al hueso (rodea la curva en vez de caerse)
            prev = (path[-1] - path[-2]) if len(path) > 1 else t
            dir_ = t + prev / max(np.linalg.norm(prev), 1e-9) * 0.6
            dir_ = dir_ - n * n.dot(dir_)
            if np.linalg.norm(dir_) < 1e-6:
                return np.array(path), True
            t = dir_
        q, n, mi = surf(path[-1] + t / np.linalg.norm(t) * 0.008)
        if mi not in red:                                # Ni dientes ni aberturas: se frena antes
            return np.array(path), False
        run += np.linalg.norm(q - path[-1])
        path.append(q)
    return np.array(path), False


def ss(a_, b_, x):
    t = np.clip((x - a_) / (b_ - a_), 0, 1)
    return t * t * (3 - 2 * t)


ue_rng = np.random.default_rng(SEED + 31)
long_rng = np.random.default_rng(SEED + 41)
for (xa, xb), drips in UNDER_EYES:
    xs = np.linspace(xa, xb, 24)
    rims = [rim_point(x, -0.22 if (xb - xa) < 0.12 else -0.1) for x in xs]   # La nariz: debajo de los ojos
    ok = [r_ is not None for r_ in rims]
    xs = xs[ok]
    rim_z = np.array([r_[2] for r_ in rims if r_ is not None])
    gap = float(np.median([planet(r_[None])[0] - relief(r_[None])[0] for r_ in rims if r_ is not None]))
    dx = np.array(sorted(d[0] for d in drips))
    nose = (xb - xa) < 0.12
    depth = 0.032 if nose else 0.068                     # Cuánto baja la capa (como en el mentón: una capa de verdad)
    arch_h = ue_rng.uniform(0.012, 0.042, len(dx) + 1) * (0.55 if nose else 1.0)   # Arcos de alturas bien distintas

    def rim_at(x):
        return np.interp(x, xs, rim_z)

    inner_a = inner_b = False                            # (El rincón de adentro se afina de a poco, ver más abajo)

    def edge_at(x):
        """Borde de abajo de la capa: baja en cada reguero y sube en un arco de media luna entre uno y otro."""
        pts_x = np.r_[xa, dx, xb]
        # Hacia el rincón de adentro (lado de la nariz) la capa sigue llena hasta la pared negra; afuera se afina
        pts_d = np.r_[depth * (0.75 if inner_a else 0.35), np.full(len(dx), depth), depth * (0.75 if inner_b else 0.35)]
        i = np.clip(np.searchsorted(pts_x, x) - 1, 0, len(pts_x) - 2)
        t = np.clip((x - pts_x[i]) / (pts_x[i + 1] - pts_x[i]), 0, 1)
        d_ = pts_d[i] + (pts_d[i + 1] - pts_d[i]) * t - arch_h[np.minimum(i, len(arch_h) - 1)] * np.sin(math.pi * t) ** 0.8
        # Tramo del rincón de adentro: el borde sube redondo (un cuarto de círculo) hasta la pared, sin punta
        last = len(pts_x) - 2
        if inner_b:
            d_ = np.where(i == last, depth * np.sqrt(np.clip(1 - t ** 2, 0, 1)), d_)
        if inner_a:
            d_ = np.where(i == 0, depth * np.sqrt(np.clip(1 - (1 - t) ** 2, 0, 1)), d_)
        return rim_at(x) - np.maximum(d_, 0.004)

    # Regueros: arrancan donde se juntan los arcos y siguen la cara para abajo
    rivs, pend, strands = [], [], []
    # Además, en la punta de afuera de cada ojo, un reguero mucho más largo que sigue la curva del pómulo casi hasta
    # los dientes (con su
    # propio azar, así los demás no cambian)
    extra = {-0.44: [(-0.425, 0.36)], 0.13: [(0.425, 0.33)]}.get(round(xa, 2), [])
    # Recorrido de los largos (de frente, x y z): bajan por el frente del pómulo, rodean su borde de abajo hacia
    # adentro (donde el pómulo se termina no hay hueso) y siguen por la cara hasta casi los dientes
    # (pasan por fuera y por debajo de los otros regueros, sin tocarlos)
    GUIDE = {-0.425: [(-0.425, None), (-0.42, -0.31), (-0.405, -0.355), (-0.37, -0.39), (-0.335, -0.415),
                      (-0.3, -0.45), (-0.276, -0.495), (-0.26, -0.545), (-0.252, -0.585), (-0.248, -0.615)],
             0.425: [(0.425, None), (0.418, -0.31), (0.402, -0.355), (0.365, -0.39), (0.325, -0.415),
                     (0.292, -0.45), (0.276, -0.5), (0.262, -0.55), (0.252, -0.6)]}
    for x0, L in list(drips) + extra:
        rng_ = ue_rng if (x0, L) in drips else long_rng
        q, n = on_face(x0, float(edge_at(np.array([x0]))[0]) + 0.02)   # Desde adentro de la capa
        if q is None:
            continue
        if x0 in GUIDE:                                  # Por el recorrido, pegado al hueso
            g = [(gx, q[2] if gz is None else gz) for gx, gz in GUIDE[x0]]
            pts_ = []
            for (ax, az), (bx, bz) in zip(g, g[1:]):
                n_ = max(2, int(math.hypot(bx - ax, bz - az) / 0.006))
                for k in range(n_):
                    t_ = k / n_
                    hq, _ = on_face(ax + (bx - ax) * t_, az + (bz - az) * t_)
                    if hq is not None:
                        pts_.append(hq)
            path, hangs = np.array(pts_), False
        else:
            path, hangs = walk_down(q, n, L, stick=L > 0.2)
        seg = np.r_[0, np.cumsum(np.linalg.norm(np.diff(path, axis=0), axis=1))]
        u = seg / max(seg[-1], 1e-6)
        ph_ = rng_.uniform(0, math.tau)
        # Igual que los regueros del mentón: el mismo alto y ancho, engordan y se angostan al bajar y abajo se
        # juntan en la cabeza gorda donde se frenaron
        wob = 1 + 0.12 * np.sin(seg * 40 + ph_)          # Apenas desparejo (sin estrangularse)
        k_ = (0.34 if nose else 0.85) * rng_.uniform(0.9, 1.1)   # Cada uno con su grosor; los de la nariz, finos
        top_k = 0.65 if nose else 1.4                    # La cabeza de abajo (en la nariz, más chica)
        head = ss(0.7, 1.0, u)
        hh_ = 0.01 * (k_ + (top_k - k_) * head) * wob
        ww_ = 0.016 * (k_ + (top_k - k_) * head) * wob
        rivs.append((path, hh_, ww_))
    allp = np.vstack([r_[0] for r_ in rivs] + [np.array([[xa, -0.9, rim_z.max()], [xb, -0.9, rim_z.min()]])])
    low_z = min([allp[:, 2].min()] + [b_[2] for _, b_, _ in strands])
    lo = np.array([xa - 0.04, -1.05, low_z - 0.07])
    hi = np.array([xb + 0.04, -0.55, rim_z.max() + 0.04])
    axes = [np.arange(lo[i], hi[i] + UE_H, UE_H) for i in range(3)]
    F_ = np.empty([len(x) for x in axes])
    gy, gz = np.meshgrid(axes[1], axes[2], indexing="ij")
    for i, x in enumerate(axes[0]):
        P = np.stack([np.full(gy.size, x), gy.ravel(), gz.ravel()], axis=1)
        dp = planet(P) - relief(P) - gap
        z = P[:, 2]
        xx = np.full(len(P), x)
        edge = edge_at(xx)
        fade = min(0.03, (xb - xa) / 5)                  # Las puntas de la capa se afinan (no la del rincón)
        inner_side = (not nose) and abs(xb) < abs(xa)
        # Hacia el rincón de adentro (lado de la nariz) la capa se afina de a poco hasta desaparecer: un poco de
        # líquido sale también de ahí, sin gota y sin cortarse de golpe
        in_fade = 0.06
        fa_ = (x - xa) / (in_fade if (not nose and abs(xa) < abs(xb)) else fade)
        fb_ = (xb - x) / (in_fade if (not nose and abs(xb) < abs(xa)) else fade)
        side = float(np.clip(min(fa_, fb_), 0, 1))
        # Capa: chata y fina, con el borde de abajo engordado (tensión) y sin pasar del borde de la órbita
        # El borde de abajo redondeado como una gota que se junta (sin escalón): sube suave desde cero
        rim_t = 0.006 + 0.005 * np.exp(-((z - edge - 0.01) / 0.009) ** 2)
        lip = np.sqrt(np.clip((z - edge) / 0.012, 0, 1))
        inside_rim = 1.0 if nose else ss(-0.6, -0.7, P[:, 1])   # Sale de adentro de la órbita: sigue por su piso
        top_ = 0.03 if nose else 0.09
        th = rim_t * lip * ss(rim_at(xx) + top_, rim_at(xx) + top_ - 0.02, z) * inside_rim * side
        for path, hh, ww in rivs:                        # Regueros chatos y anchos, con los bordes apoyados
            best = np.full(len(P), 9.0)
            hb, wb = np.zeros(len(P)), np.ones(len(P))
            for j in range(len(path) - 1):
                a_, b_ = path[j], path[j + 1]
                ba = b_ - a_
                h_ = np.clip((P - a_) @ ba / max(ba @ ba, 1e-12), 0, 1)
                dd = np.linalg.norm(P - a_ - h_[:, None] * ba, axis=1)
                mk = dd < best
                best[mk] = dd[mk]
                hb[mk] = hh[j] + (hh[j + 1] - hh[j]) * h_[mk]
                wb[mk] = ww[j] + (ww[j + 1] - ww[j]) * h_[mk]
            th = np.maximum(th, hb * np.sqrt(np.clip(1 - (best / wb) ** 2, 0, 1)))   # Redondo, lleno
        film = np.where(th > 0.0012, np.maximum(dp - th, -dp - 0.03), 1.0)
        d_ = np.full(len(P), 9.0)
        for a_, b_, rs in strands:                       # Hilo que cuelga: se afina al medio, gota pesada abajo
            ba = b_ - a_
            h_ = np.clip((P - a_) @ ba / (ba @ ba), 0, 1)
            rad = rs * (1.0 - 0.35 * np.sin(math.pi * h_))
            d_ = np.minimum(d_, smin(np.linalg.norm(P - a_ - h_[:, None] * ba, axis=1) - rad,
                                     np.linalg.norm(((P - b_) / np.array([1, 1, 1.3])), axis=1) - rs * 2.0, rs * 1.5))
        for c, rp in pend:                               # La bola de abajo del reguero (redonda, un poco chata)
            q_ = (P - c) / np.array([1.0, 1.0, 1.15])
            d_ = np.minimum(d_, np.linalg.norm(q_, axis=1) - rp)
        F_[i] = smin(film, d_, 0.01).reshape(gy.shape)
    v_, q_ = surface_nets(F_, lo, UE_H)
    tf.extend((q_ + len(tv)).tolist())
    tv.extend(v_.tolist())
    print("SANGRE OJO: %d regueros" % len(rivs))
tme = bpy.data.meshes.new("HW_Eye_Blood")
tme.from_pydata((np.array(tv) * R).tolist(), [], tf)
tme.validate()
tears_ob = bpy.data.objects.new("HW_Eye_Blood", tme)
zones.objects.link(tears_ob)
tears_ob.parent = ROOT
sm_ = tears_ob.modifiers.new("Smooth", "SMOOTH")
sm_.factor, sm_.iterations = 0.5, 3              # Poco: si no, los regueros se hunden
dm_ = tears_ob.modifiers.new("Decimate", "DECIMATE")
dm_.ratio = min(1.0, 24000 / (2 * max(len(tf), 1)))   # Sin achicar de más: queda liso, no a bloques
dg_ = bpy.context.evaluated_depsgraph_get()
low_ = bpy.data.meshes.new_from_object(tears_ob.evaluated_get(dg_))
tears_ob.modifiers.clear()
tears_ob.data = low_
bpy.data.meshes.remove(tme)
tme = low_
tme.name = "HW_Eye_Blood"
tme.materials.append(BLOOD)
tme.polygons.foreach_set("use_smooth", np.ones(len(tme.polygons), bool))
tme.calc_loop_triangles()
print("TRIS %-18s %6d" % (tears_ob.name, len(tme.loop_triangles)))
# Brujas sentadas en las curvas, felices, mirando hacia adelante (hacia quien mira)
for seat_pt, twist in seats:
    put("SitHappy", Vector(seat_pt * R) + Vector((0, 0, 0.018 * R)), math.degrees(twist), SWING_WITCH_SCALE)
# Una bruja sentada en el centro de los dientes de abajo, sobre el charco, de frente, con las piernas colgando por
# delante de los dientes
# Sentada en el filo de arriba del charco, sobre los dos dientes del centro, para que las piernas cuelguen por
# delante de la sangre (y no adentro)
bco = np.array([v.co[:] for v in blood_ob.data.vertices]) / R
center = jaw_open(np.array([[0.0, arch(0.0, -0.815), -0.6]]))[0]
near = bco[(np.abs(bco[:, 0]) < 0.03) & (np.linalg.norm(bco[:, 1:] - center[1:], axis=1) < 0.08)]
top_z = near[:, 2].max()
band = near[near[:, 2] > top_z - 0.035]                 # Lo de arriba del charco, con su panza de adelante
y_seat = band[:, 1].min() + 0.006                        # En el filo de adelante: las piernas cuelgan por fuera
under = near[np.abs(near[:, 1] - y_seat) < 0.008]
seat = np.array([0.0, y_seat, under[:, 2].max()])
put("Sit", Vector(seat * R) + Vector((0, 0, 0.004 * R)), 0, WITCH_SCALE * 1.5)   # Más grande que las de las órbitas
print("WITCHES %d" % len(witches.objects))

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
ld.energy, ld.color, ld.shadow_soft_size = 12000, srgb("#ff6a1e"), 3.0   # Resplandor de lava
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
