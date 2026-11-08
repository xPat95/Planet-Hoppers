"""Criaturas del minijuego de Halloween: diablito (cuerpo entero), fantasma y demonio de parálisis del sueño.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/halloween/build_game_creatures.py

Guarda assets/models/minigames/halloween/creatures.blend y capturas en docs/references/halloween/game_creatures.png.
Diseño: docs/minigames/halloween-plan.md.

Cada criatura tiene varias poses (caminar/flotar y bailar) con la misma malla, para que el juego las mezcle suave.
Cada pose es un modelo aparte con su raíz (Halloween_Game_<criatura>_<pose>_ROOT), origen en el piso.
- Diablo: rojo carmesí, alto como los fantasmas y muy flaco (silueta larga, brazos que llegan a las rodillas,
  dedos larguísimos, cintura finita, apenas encorvado), con huesos que se marcan, patas de cabra, cuernos rojos
  enroscados y una cara tallada en la cabeza como una calabaza (dibujo de Cami en
  docs/references/halloween/devil_face_reference.png). Soldado y con huesos, como el demonio.
- Fantasma: una persona de verdad debajo de una sábana fina llena de pliegues. La tela tiene recortados dos agujeros
  chiquitos para los ojos (azules, con los párpados a media asta, mirada muerta) y uno para la boca, que sonríe con
  todos los dientes. Soldado y con huesos, como el demonio.
- Demonio de parálisis del sueño: un esqueleto negro demasiado alto, con cuatro brazos y el detalle en las costillas.
  Es una sola pieza soldada (weld.py): los huesos se funden unos con otros y las poses doblan esa misma malla.
"""

import bmesh
import bpy
import math
import os
import sys
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from figure import Builder, make_mesh, mat, rx, ry, rz, srgb  # noqa: E402
import numpy as np  # noqa: E402
from weld import Weld, carve, decimate, rigid, skin  # noqa: E402

REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "minigames", "halloween", "creatures.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "halloween")

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "Halloween_Game_Creatures"

DEVIL = mat("GameDevilSkin", "#ff1744", emit=0.35)    # Menos luz propia que antes: que se lean músculos y huesos
DEVIL_EYES = mat("GameDevilEyes", "#2b0208")           # Lo tallado en la cara (adentro oscuro)
GHOST = mat("GameGhost", "#e4e2dc", emit=0.3, rough=0.75)   # Sábana vieja, apenas amarillenta
HOLE = mat("GameGhostHole", "#050407")
DEMON = mat("GameDemon", "#07060a", rough=0.5)
DEMON_EYES = mat("GameDemonEyes", "#f4f4ff", emit=5.0)     # Puntitos blancos al fondo de las cuencas
TEETH = mat("GameDemonTeeth", "#e8e2d4", emit=0.3)
DEVIL_HORN = mat("GameDevilHorn", "#1c1012", rough=0.4)          # Pezuñas y pupilas
DEVIL_GLOW = mat("GameDevilGlow", "#ffc21a", emit=5.0)            # Ojos de cabra
GHOST_IN = mat("GameGhostInside", "#8e8c86", rough=0.85)         # El revés de la tela, en sombra
GHOST_SKIN = mat("GameGhostSkin", "#9c7764", rough=0.6)            # Piel a la sombra de la sábana
GHOST_LIPS = mat("GameGhostLips", "#a0635a", rough=0.5)
GHOST_SCLERA = mat("GameGhostSclera", "#f4f1ea", emit=1.2, rough=0.25)    # Blanco que se ve aun en la sombra
GHOST_IRIS = mat("GameGhostIris", "#2b74d6", emit=1.0, rough=0.25)
GHOST_PUPIL = mat("GameGhostPupil", "#040305", rough=0.2)
GHOST_CUT = mat("GameGhostCut", "#ff00ff")                          # Solo para recortar la tela; no queda en la malla
MATS = [DEVIL, DEVIL_EYES, GHOST, HOLE, DEMON, DEMON_EYES, TEETH, DEVIL_HORN, DEVIL_GLOW,
        GHOST_IN, GHOST_SKIN, GHOST_LIPS, GHOST_SCLERA, GHOST_IRIS, GHOST_PUPIL]


# --- Diablo -----------------------------------------------------------------------------------------------------
# Alto como los fantasmas (~1.2 con los cuernos) y con la silueta de una figura larguísima y flaca (referencia de
# Cami en docs/references/halloween/devil_body_reference.png): hombros angostos y cuadrados, cintura finita, brazos
# larguísimos que llegan a las rodillas, dedos muy largos con garras, el cuello y la cabeza apenas hacia adelante.
# Liso, sin músculos marcados; se notan los huesos, sobre todo la caja de costillas (y clavículas, esternón,
# columna, omóplatos, codos, muñecas y rodillas). Cintura, cadera y espalda normales. Patas de cabra: la articulación del medio dobla para atrás y terminan en pezuñas partidas.
# Cuernos rojos enroscados. La cara está tallada en la cabeza, como una calabaza, siguiendo el dibujo de Cami: dos
# ojos enormes con un contorno roto alrededor, dos fosas de la nariz en gota y una sonrisa en V de mejilla a mejilla
# que baja hasta el mentón, con un ganchito en cada punta.
# Como el demonio: se suelda una vez en la pose neutra, se talla la cara y cada pose dobla esa malla con los huesos.
DEVIL_POSES = {
    # Acechando, doblado del todo hacia adelante sobre las patas de cabra: los brazos larguísimos cuelgan y las
    # garras se arrastran por el piso; la cabeza, ladeada muy fuerte siempre para el mismo lado, mira fija a la
    # bruja; pasos largos
    "walk_a": {"torso": (75, 4, 5), "head": (-88, 6, 42), "arm_R": (10, -82, -8, 0), "arm_L": (10, -66, -4, 0),
               "leg_R": (24, 6, 38), "leg_L": (-40, 6, 14)},
    "walk_b": {"torso": (75, -4, -5), "head": (-84, -2, 48), "arm_R": (10, -66, -4, 0), "arm_L": (10, -82, -8, 0),
               "leg_R": (-40, 6, 14), "leg_L": (24, 6, 38)},
    # Bailando: los de los planetas
    "dance_a": {"torso": (4, 14, 10), "head": (-6, -12, -8), "arm_R": (160, 0, -25, 0), "arm_L": (155, 0, -35, 0),
                "leg_R": (-4, 10, 0), "leg_L": (-35, -10, 35)},
    "dance_b": {"torso": (2, -12, -8), "head": (-4, 10, 6), "arm_R": (95, 0, -95, 90), "arm_L": (95, 0, -95, 90),
                "leg_R": (-35, 10, 35), "leg_L": (-4, -10, 0)},
}
DEVIL_REST = {"torso": (0, 0, 0), "head": (0, 0, 0), "arm_R": (25, 0, -15, 0), "arm_L": (25, 0, -15, 0),
              "leg_R": (0, 6, 0), "leg_L": (0, 6, 0)}
DEVIL_WELD_H, DEVIL_TRIS = 0.002, 8000
DEVIL_STALK_BACK = 0.3                                  # Cuánto se corre para atrás acechando (ver el armado)
DEVIL_HS = 1.25                                          # La cabeza (con la cara tallada y los cuernos), más grande


def devil_skeleton(pose):
    """Puntos y huesos del diablo en una pose ({nombre: (origen, rotación)})."""
    Rt = rz(pose["torso"][2]) @ ry(pose["torso"][1]) @ rx(pose["torso"][0])
    Rh = Rt @ rz(pose["head"][2]) @ ry(pose["head"][1]) @ rx(pose["head"][0])
    hip = Vector((0, 0, 0.58))
    neck = hip + Rt @ Vector((0, -0.014, 0.43))          # Cuello apenas hacia adelante
    head = neck + Rh @ Vector((0, 0, 0.06 * DEVIL_HS))
    bones = {"pelvis": (hip, rx(0)), "torso": (hip, Rt), "head": (neck, Rh)}
    arms, legs = [], []
    for side, key in ((1, "arm_R"), (-1, "arm_L")):
        abd, flex, elbow, twist = pose[key]
        sh = hip + Rt @ Vector((side * 0.078, 0, 0.355))
        Ru = Rt @ ry(-side * abd) @ rx(flex) @ rz(side * twist)
        el = sh + Ru @ Vector((0, 0, -0.29))
        Rf = Ru @ rx(elbow)
        wr = el + Rf @ Vector((0, 0, -0.27))
        bones[key + "_up"], bones[key + "_fore"] = (sh, Ru), (el, Rf)
        arms.append((key, side, sh, Ru, el, Rf, wr))
    for side, key in ((1, "leg_R"), (-1, "leg_L")):
        flex, abd, knee = pose[key]
        hp = hip + Vector((side * 0.042, 0, 0.0))
        R1 = ry(-side * abd) @ rx(flex)
        Rth, Rsh, Rca = R1 @ rx(-28), R1 @ rx(62 + knee), R1 @ rx(-18 - knee * 0.5)
        kn = hp + Rth @ Vector((0, 0, -0.25))
        hock = kn + Rsh @ Vector((0, 0, -0.21))
        foot = hock + Rca @ Vector((0, 0, -0.2))
        bones[key + "_thigh"], bones[key + "_shin"], bones[key + "_cannon"] = (hp, Rth), (kn, Rsh), (hock, Rca)
        legs.append((key, side, hp, Rth, kn, Rsh, hock, Rca, foot))
    return bones, dict(Rt=Rt, Rh=Rh, hip=hip, neck=neck, head=head, arms=arms, legs=legs)


def devil_parts(S):
    """Piezas soldadas del diablo y las piezas finas pegadas a un hueso (cuernos, garras, pezuñas)."""
    W = Weld()
    A = Builder(MATS)
    abone = []
    hs = DEVIL_HS

    def tag(bone, n0):
        abone.extend([bone] * (len(A.v) - n0))
    Rt, Rh, hip, neck, head = S["Rt"], S["Rh"], S["hip"], S["neck"], S["head"]

    def up(h, x=0.0, y=0.0):
        return hip + Rt @ Vector((x, y, h))
    # Torso largo y flaco: cadera, cintura finita, pecho angosto y hombros cuadrados
    W.ellipsoid("pelvis", up(0.0), Rt, (0.054, 0.037, 0.06), k=0.025)
    W.ellipsoid("torso", up(0.11), Rt, (0.052, 0.037, 0.09), k=0.035)                                # Cintura normal
    W.ellipsoid("torso", up(0.25), Rt, (0.068, 0.046, 0.11), k=0.03)
    W.ellipsoid("torso", up(0.33, 0, -0.004), Rt, (0.074, 0.04, 0.045), k=0.02)                      # Arriba del pecho
    for sd in (-1, 1):
        W.ellipsoid("torso", up(0.3, sd * 0.04, 0.036), Rt, (0.022, 0.006, 0.03), k=0.012)            # Omóplato
        W.capsule("torso", up(0.4, 0, 0.004), up(0.36, sd * 0.075, 0.0), 0.016, 0.012, k=0.015)       # Trapecio
        W.capsule("torso", up(0.362, sd * 0.012, -0.037), up(0.358, sd * 0.075, -0.012), 0.005, k=0.008)  # Clavícula
        # Caja de costillas marcada bajo la piel: seis pares que salen de la columna, rodean el pecho bajando y
        # llegan al esternón; las de abajo se quedan cortas y dibujan el arco debajo del pecho
        for i in range(6):
            h0 = 0.335 - 0.03 * i
            reach = 0.93 if i < 3 else 0.93 - 0.07 * (i - 2)
            pts = []
            for a in np.linspace(0.12, math.pi * reach, 10):
                h = h0 - 0.055 * (1 - math.cos(a)) / 2
                q = math.sqrt(max(0.0, 1 - ((h - 0.25) / 0.11) ** 2))   # Ancho del pecho a esa altura
                pts.append(up(h, sd * 0.068 * q * math.sin(a), 0.046 * q * math.cos(a)))
            W.chain("torso", pts, [0.0025] + [0.0048] * 7 + [0.004, 0.002], k=0.003)
    W.capsule("torso", up(0.35, 0, -0.04), up(0.24, 0, -0.044), 0.0035, k=0.01)                       # Esternón
    for i in range(7):                                   # Columna en la espalda
        h = 0.06 + 0.045 * i
        W.ellipsoid("torso", up(h, 0, 0.033), Rt, (0.006, 0.005, 0.006), k=0.006)
    # Cuello largo con los tendones marcados, y la cabeza (todo lo de la cabeza escalado por hs)
    W.capsule("torso", up(0.35), neck, 0.019, 0.017, k=0.015)
    W.capsule("head", neck, neck + Rh @ Vector((0, 0, 0.03 * hs)), 0.017, k=0.015)
    for sd in (-1, 1):
        W.capsule("head", neck + Rh @ Vector((sd * 0.022, 0.006, 0.035)) * 1, up(0.36, sd * 0.008, -0.036), 0.005,
                  k=0.008)

    def hd(x, y, z):
        return head + Rh @ (Vector((x, y, z)) * hs)
    W.ellipsoid("head", hd(0, 0.004, 0.008), Rh, tuple(r * hs for r in (0.044, 0.048, 0.05)), k=0.015)    # Cráneo
    W.ellipsoid("head", hd(0, -0.008, -0.018), Rh, tuple(r * hs for r in (0.038, 0.042, 0.045)), k=0.015)  # Cara
    W.ellipsoid("head", hd(0, -0.03, -0.058), Rh, tuple(r * hs for r in (0.013, 0.012, 0.012)), k=0.012)   # Mentón
    for sd in (-1, 1):
        W.ellipsoid("head", hd(sd * 0.03, -0.034, -0.006), Rh, tuple(r * hs for r in (0.014, 0.012, 0.01)),
                    k=0.01)                              # Pómulo
        W.ellipsoid("head", hd(sd * 0.046, 0.004, 0.006), Rh @ ry(sd * 35), tuple(r * hs for r in (0.006, 0.01, 0.022)),
                    k=0.006)                             # Oreja puntiaguda
    for sd in (-1, 1):                                   # Cuernos rojos enroscados
        n0 = len(A.v)
        c = hd(0, 0.025, 0.04)
        horn = []
        for k in range(14):
            t = k / 13
            phi = math.radians(320 * t)
            r = 0.03 + 0.016 * t
            horn.append(c + Rh @ (Vector((sd * (0.022 + 0.05 * t), -r * math.cos(phi), r * math.sin(phi))) * hs))
        A.tube(horn, [(0.019 - 0.0175 * (k / 13) ** 0.8) * hs for k in range(14)], 8, DEVIL, cap0=True, cap1=True)
        tag("head", n0)

    for key, side, sh, Ru, el, Rf, wr in S["arms"]:      # Brazos larguísimos y flacos, con dedos muy largos
        up_, fore = key + "_up", key + "_fore"
        W.ellipsoid(up_, sh, Ru, (0.022, 0.02, 0.022), k=0.015)                                    # Hombro
        W.capsule(up_, sh, el, 0.017, 0.012, k=0.012)
        W.ellipsoid(up_, sh.lerp(el, 0.45), Ru, (0.012, 0.011, 0.06), k=0.01)
        W.ellipsoid(up_, el + Ru @ Vector((0, 0.005, 0)), Ru, (0.01, 0.01, 0.011), k=0.006)         # Codo
        W.capsule(fore, el, wr, 0.012, 0.008, k=0.012)
        W.ellipsoid(fore, el.lerp(wr, 0.3), Rf, (0.013, 0.011, 0.06), k=0.012)
        W.ellipsoid(fore, wr, Rf, (0.009, 0.007, 0.007), k=0.006)                                  # Muñeca
        palm = wr + Rf @ Vector((0, 0, -0.022))
        W.ellipsoid(fore, palm, Rf, (0.012, 0.006, 0.02), k=0.008)
        n0 = len(A.v)
        for f in range(4):                               # Dedos larguísimos con garras negras
            Rd = Rf @ ry((f - 1.5) * 9)
            base = palm + Rf @ Vector(((f - 1.5) * 0.0065, 0, -0.016))
            k1 = base + Rd @ rx(-10) @ Vector((0, 0, -0.04))
            k2 = k1 + Rd @ rx(-25) @ Vector((0, 0, -0.032))
            W.chain(fore, [base, k1, k2], [0.0048, 0.0042, 0.0036], k=0.003)
            A.tube([k2, k2 + Rd @ rx(-55) @ Vector((0, 0, -0.022))], [0.0036, 0.0004], 5, DEVIL_HORN, cap1=True)
        tag(fore, n0)
    for key, side, hp, Rth, kn, Rsh, hock, Rca, foot in S["legs"]:   # Patas de cabra, largas y flacas
        th_, sh_, ca_ = key + "_thigh", key + "_shin", key + "_cannon"
        W.capsule(th_, hp + Vector((0, 0, 0.03)), kn, 0.028, 0.018, k=0.025)
        W.ellipsoid(th_, hp.lerp(kn, 0.45) + Rth @ Vector((0, -0.006, 0)), Rth, (0.019, 0.017, 0.08), k=0.015)
        W.ellipsoid(th_, kn + Rth @ Vector((0, -0.006, 0)), Rth, (0.014, 0.013, 0.016), k=0.006)     # Rodilla
        W.capsule(sh_, kn, hock, 0.017, 0.01, k=0.012)
        W.ellipsoid(sh_, kn.lerp(hock, 0.3) + Rsh @ Vector((0, 0.008, 0)), Rsh, (0.016, 0.015, 0.05), k=0.012)
        W.ellipsoid(ca_, hock, Rca, (0.011, 0.012, 0.012), k=0.006)                                # Corvejón
        W.capsule(ca_, hock, foot, 0.0095, 0.0085, k=0.008)
        n0 = len(A.v)
        for o in (-1, 1):
            A.ellipsoid(foot + Rca @ Vector((o * 0.0105, -0.008, -0.013)), Rca, (0.012, 0.02, 0.017), DEVIL_HORN,
                        seg=7, rings=4)
        tag(ca_, n0)
    return W, A, abone


# Cara tallada, del dibujo de Cami (devil_face_reference.png): coordenadas en píxeles del dibujo. La cara del
# dibujo ocupa el cuadrado (520..1338, 173..970)
def devil_px(px, py):
    return (px - 929) / 410 * 0.036 * DEVIL_HS, (0.045 - (py - 173) / 797 * 0.105) * DEVIL_HS


def devil_face_cuts(W, head):
    """Sólidos que se restan de la cabeza (en la pose neutra): ojos, contornos rotos, nariz, sonrisa y ganchitos."""
    def surf_y(x, z):
        ys = head.y + np.arange(-0.1, 0.0, 0.0004)
        f = W.field(np.stack([np.full(len(ys), head.x + x), ys, np.full(len(ys), head.z + z)], 1))
        j = int(np.argmax(f < 0))
        return ys[j] - 0.0004 * f[j] / (f[j] - f[j - 1])

    def prism(outline, depth):
        """Contorno (x, z) relativo a la cabeza, cortado hacia adentro `depth` desde la superficie."""
        V, n = [], len(outline)
        for front in (True, False):
            for x, z in outline:
                y = surf_y(x, z)
                V.append(Vector((head.x + x, y - 0.02 if front else y + depth * DEVIL_HS, head.z + z)))
        F = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
        F += [(k, (k + 1) % n, n + (k + 1) % n, n + k) for k in range(n)]
        return V, F

    def stroke(pts, width, depth):
        """Una raya del dibujo (polilínea en píxeles) con su ancho en píxeles."""
        P = [np.array(devil_px(*q)) for q in pts]
        w = width / 2 / 410 * 0.036 * DEVIL_HS
        left, right = [], []
        for i, q in enumerate(P):
            t = P[min(i + 1, len(P) - 1)] - P[max(i - 1, 0)]
            nrm = np.array([-t[1], t[0]]) / np.linalg.norm(t)
            left.append(tuple(q + nrm * w))
            right.append(tuple(q - nrm * w))
        return prism(left + right[::-1], depth)

    def ellipse(cx, cy, rx_, ry_, rot, wob, n=28):
        out = []
        for k in range(n):
            a = 2 * math.pi * k / n
            r = 1 + wob * math.sin(3 * a + cx) + 0.6 * wob * math.sin(5 * a + cy)
            x, y = rx_ * r * math.cos(a), ry_ * r * math.sin(a)
            c, s_ = math.cos(math.radians(rot)), math.sin(math.radians(rot))
            out.append((cx + c * x - s_ * y, cy + s_ * x + c * y))
        return out
    cuts = []
    for (cx, cy, rx_, ry_, rot), arcs in (((703, 380, 140, 165, -15), ((20, 80), (100, 165), (185, 245), (265, 340))),
                                         ((1163, 405, 145, 140, 10), ((15, 75), (95, 160), (180, 250), (270, 345)))):
        cuts.append(prism([devil_px(*q) for q in ellipse(cx, cy, rx_, ry_, rot, 0.03)], 0.012))      # Ojo
        for a0, a1 in arcs:                              # Contorno roto alrededor del ojo
            pts = []
            for a in np.linspace(math.radians(a0), math.radians(a1), 8):
                r = 1.17 + 0.03 * math.sin(4 * a + cx)
                x, y = rx_ * r * math.cos(a), ry_ * r * math.sin(a)
                c, s_ = math.cos(math.radians(rot)), math.sin(math.radians(rot))
                pts.append((cx + c * x - s_ * y, cy + s_ * x + c * y))
            cuts.append(stroke(pts, 18, 0.0028))
    for (tx, ty, r), (bx, by) in (((876, 585, 27), (897, 708)), ((984, 565, 24), (942, 716))):   # Nariz en gota
        d = math.atan2(by - ty, bx - tx)
        out = [(bx, by)]
        for a in np.linspace(d + math.pi / 2, d + 3 * math.pi / 2, 10):
            out.append((tx + r * math.cos(a), ty + r * math.sin(a)))
        cuts.append(prism([devil_px(*q) for q in out], 0.006))
    # Sonrisa en V: finita en las puntas, gruesa abajo en el mentón; los lados apenas curvados
    bottom, corners = np.array([912.0, 922.0]), (np.array([565.0, 557.0]), np.array([1270.0, 578.0]))
    up_edge, lo_edge = [], []
    for t in np.linspace(-1, 1, 31):
        corner = corners[0] if t < 0 else corners[1]
        q = bottom + (corner - bottom) * abs(t)
        dirv = (corner - bottom) / np.linalg.norm(corner - bottom)
        nrm = np.array([dirv[1], -dirv[0]]) * (1 if t < 0 else -1)   # Hacia afuera de la V (abajo)
        q = q + nrm * 18 * math.sin(math.pi * abs(t))
        ht = 5 + 30 * (1 - abs(t)) ** 1.6
        up_edge.append(tuple(q - nrm * ht))
        lo_edge.append(tuple(q + nrm * ht))
    cuts.append(prism([devil_px(*q) for q in up_edge + lo_edge[::-1]], 0.007))
    cuts.append(stroke([(558, 577), (530, 540), (538, 512), (575, 515), (607, 538)], 20, 0.0035))     # Ganchitos
    cuts.append(stroke([(1297, 615), (1320, 580), (1300, 557), (1260, 560), (1233, 580)], 20, 0.0035))
    return cuts


_DEVIL = {}


def build_devil(pose):
    if not _DEVIL:                                       # Se suelda y se talla una sola vez, en la pose neutra
        rest, S = devil_skeleton(DEVIL_REST)
        W, A, abone = devil_parts(S)
        V, F = W.surface(DEVIL_WELD_H)
        V, F = decimate(V, F, DEVIL_TRIS)
        V, F, M = carve(V, F, [DEVIL] * len(F), devil_face_cuts(W, S["head"]), DEVIL_EYES)
        bones, Wt = W.weights(V, 0.006)
        _DEVIL.update(rest=rest, A=A, abone=abone, V=V, F=F, M=M, bones=bones, Wt=Wt)
    D = _DEVIL
    posed, _ = devil_skeleton(pose)
    B = Builder(MATS)
    B.v = [Vector(v) for v in skin(D["V"], D["bones"], D["Wt"], D["rest"], posed)]
    for f, m in zip(D["F"], D["M"]):
        B.add(f, m)
    off = len(B.v)
    B.v += [rigid(v, b, D["rest"], posed) for v, b in zip(D["A"].v, D["abone"])]
    B.f += [tuple(i + off for i in f) for f in D["A"].f]
    B.m += D["A"].m
    return B


# --- Fantasma ---------------------------------------------------------------------------------------------------
# Una persona de verdad debajo de una sábana (cabeza con forma de cráneo, frente, nariz, pómulos, mentón y orejas;
# cuello, trapecios, hombros, torso y brazos colgando). La sábana es fina: se pega a la cabeza, cruza los huecos
# (de la nariz al labio, del mentón al pecho), cae de los hombros y los brazos, y se llena de pliegues que salen en
# abanico de los hombros. Flota: no hay piernas y el borde ondula sobre el piso.
# La cara: dos agujeros redondos recortados en la tela, con ojos humanos atrás bien abiertos, sin párpados, con un
# puntito de pupila (mirada fija y muerta), y un recorte para la boca: una sonrisa muy ancha con todos los dientes.
# Como el demonio: se arma una vez en la pose neutra y cada pose dobla esa malla con los huesos.
GHOST_POSES = {
    # Flotando quieto, la tela se mece (dos fases), la cabeza ladeada
    "float_a": {"tilt": 14, "turn": 6, "nod": 0, "lean": 3, "sway": 0.0, "hem": 0.0,
                "arm_R": (10, 0, -10), "arm_L": (10, 0, -10)},
    "float_b": {"tilt": 18, "turn": -4, "nod": 6, "lean": 5, "sway": 0.03, "hem": 1.7,
                "arm_R": (12, 5, -15), "arm_L": (8, -5, -8)},
    # "Bailando": se balancea lento, un brazo se levanta abajo de la tela, la cabeza se va para el otro lado
    "dance_a": {"tilt": -20, "turn": 10, "nod": -4, "lean": 0, "sway": 0.08, "hem": 0.9,
                "arm_R": (28, -20, -40), "arm_L": (14, 10, -20)},
    "dance_b": {"tilt": 24, "turn": -10, "nod": 4, "lean": 0, "sway": -0.08, "hem": 2.5,
                "arm_R": (14, 10, -20), "arm_L": (28, -20, -40)},
}
GHOST_REST = {"tilt": 0, "turn": 0, "nod": 0, "lean": 0, "sway": 0.0, "hem": 0.0,
              "arm_R": (10, 0, -10), "arm_L": (10, 0, -10)}
GH = Vector((0, 0, 1.1))                                 # Centro de la cabeza en la pose neutra (mide ~1.2)
G_SEG = 128                                              # Divisiones alrededor de la sábana
G_TRIS, G_FACE_TRIS = 9000, 3500


def ghost_skeleton(pose):
    """Huesos del fantasma en una pose: pecho, cabeza y los brazos (brazo y antebrazo)."""
    Rc = rx(pose["lean"])
    chest = Vector((0, 0, 0.6))
    bones = {"chest": (chest, Rc)}
    Rh = Rc @ rz(pose["turn"]) @ ry(pose["tilt"]) @ rx(pose["nod"])
    bones["head"] = (chest + Rc @ Vector((0, 0, 0.38)), Rh)
    arms = []
    for side in (1, -1):
        name = "arm_R" if side > 0 else "arm_L"
        abd, flex, elbow = pose[name]
        sh = chest + Rc @ Vector((side * 0.155, 0.0, 0.285))
        Ru = Rc @ ry(-side * abd) @ rx(flex)
        el = sh + Ru @ Vector((0, 0, -0.27))
        Rf = Ru @ rx(elbow)
        bones[name + "_up"] = (sh, Ru)
        bones[name + "_fore"] = (el, Rf)
        arms.append((name, side, sh, el, Rf, el + Rf @ Vector((0, 0, -0.24))))
    return bones, arms


G_MOUTH_W = 0.042                                        # Media boca: una sonrisa muy ancha, de mejilla a mejilla


def g_mouth_z(x):
    """Altura de la línea de la boca (entre los dientes): sonríe, las comisuras suben."""
    return -0.062 + 0.016 * (x / G_MOUTH_W) ** 2


def g_face_y(x, z):
    """Frente de la cara (la masa de la cara, un elipsoide) en x, z, relativo al centro de la cabeza."""
    return -0.022 - 0.07 * math.sqrt(max(0.0, 1 - (x / 0.062) ** 2 - ((z + 0.038) / 0.066) ** 2))


G_EYES = [GH + Vector((s * 0.03, -0.066, 0.004)) for s in (-1, 1)]


def ghost_head(W, detail):
    """Cabeza con forma de cráneo de verdad. detail: cuencas, boca abierta y labios (la cara que se ve)."""
    c = GH
    W.ellipsoid("head", c + Vector((0, 0.012, 0.012)), rx(0), (0.073, 0.09, 0.088), k=0.02)     # Cráneo
    W.ellipsoid("head", c + Vector((0, -0.022, -0.038)), rx(0), (0.062, 0.07, 0.066), k=0.02)   # Cara
    W.ellipsoid("head", c + Vector((0, -0.07, -0.09)), rx(0), (0.024, 0.018, 0.02), k=0.015)    # Mentón
    W.ellipsoid("head", c + Vector((0, -0.077, 0.03)), rx(0), (0.056, 0.013, 0.014), k=0.015)   # Frente
    W.capsule("head", c + Vector((0, -0.084, 0.018)), c + Vector((0, -0.1, -0.028)), 0.008, 0.012, k=0.01)  # Nariz
    W.ellipsoid("head", c + Vector((0, -0.099, -0.03)), rx(0), (0.012, 0.012, 0.011), k=0.008)
    for sd in (-1, 1):
        W.ellipsoid("head", c + Vector((sd * 0.05, -0.008, -0.07)), rx(0), (0.018, 0.03, 0.022), k=0.02)  # Mandíbula
        W.ellipsoid("head", c + Vector((sd * 0.046, -0.062, -0.008)), rx(0), (0.022, 0.02, 0.016), k=0.015)  # Pómulo
        W.ellipsoid("head", c + Vector((sd * 0.013, -0.09, -0.036)), rx(0), (0.009, 0.008, 0.007), k=0.006)  # Ala nariz
        W.ellipsoid("head", c + Vector((sd * 0.075, 0.006, -0.005)), rx(0), (0.011, 0.02, 0.03), k=0.01)   # Oreja
    xs = list(np.linspace(-G_MOUTH_W, G_MOUTH_W, 11))
    for dz, rad in ((0.0105, 0.0038), (-0.011, 0.0045)):    # Labios: el de abajo más lleno
        W.chain("head", [c + Vector((x, g_face_y(x, g_mouth_z(x) + dz) - 0.001, g_mouth_z(x) + dz)) for x in xs],
                [rad * (0.55 + 0.45 * math.cos(x / G_MOUTH_W * math.pi / 2)) for x in xs], k=0.006, m=GHOST_LIPS)
    if detail:
        for e in G_EYES:                                 # Cuencas para los ojos
            W.ellipsoid("head", e + Vector((0, -0.004, 0)), rx(0), (0.017, 0.014, 0.012), k=0.006, sub=True)
        W.chain("head", [c + Vector((x, g_face_y(x, g_mouth_z(x)), g_mouth_z(x))) for x in xs],
                [0.004, 0.006, 0.0075, 0.0085, 0.0085, 0.0085, 0.0085, 0.0085, 0.0075, 0.006, 0.004], k=0.003,
                sub=True)


def ghost_torso(W, arms):
    W.capsule("head", Vector((0, 0.006, 1.0)), Vector((0, 0.006, 0.95)), 0.045, k=0.02)        # Cuello
    W.capsule("chest", Vector((0, 0.006, 0.95)), Vector((0, 0.01, 0.88)), 0.046, k=0.02)
    W.ellipsoid("chest", Vector((0, 0.005, 0.78)), rx(0), (0.155, 0.095, 0.15), k=0.04)         # Pecho
    W.ellipsoid("chest", Vector((0, 0.0, 0.6)), rx(0), (0.132, 0.088, 0.14), k=0.04)            # Panza
    W.ellipsoid("chest", Vector((0, 0.005, 0.45)), rx(0), (0.15, 0.1, 0.12), k=0.04)            # Cadera
    for name, side, sh, el, Rf, wr in arms:
        W.capsule("chest", Vector((0, 0.02, 0.95)), Vector((side * 0.14, 0.01, 0.895)), 0.04, 0.034, k=0.03)  # Trapecio
        W.ellipsoid(name + "_up", sh + Vector((side * 0.008, 0, -0.012)), rx(0), (0.045, 0.048, 0.05), k=0.03)  # Hombro
        W.capsule(name + "_up", sh, el, 0.042, 0.034, k=0.02)
        W.capsule(name + "_fore", el, wr, 0.032, 0.025, k=0.02)
        W.ellipsoid(name + "_fore", wr + Rf @ Vector((0, 0, -0.05)), Rf, (0.024, 0.012, 0.05), k=0.015)  # Mano


def convex_hull(P):
    """Envolvente convexa (cadena monótona), en sentido antihorario."""
    P = sorted(map(tuple, P))

    def half(pts):
        h = []
        for q in pts:
            while len(h) >= 2 and ((h[-1][0] - h[-2][0]) * (q[1] - h[-2][1]) -
                                   (h[-1][1] - h[-2][1]) * (q[0] - h[-2][0])) <= 0:
                h.pop()
            h.append(q)
        return h
    lo, hi = half(P), half(P[::-1])
    return np.array(lo[:-1] + hi[:-1])


def hull_radius(th, r):
    """La tela cruza los huecos de cada vuelta: radio de la envolvente convexa en cada dirección."""
    m = r > 1e-6
    if m.sum() < 3:
        return r
    Hl = convex_hull(np.stack([r[m] * np.cos(th[m]), r[m] * np.sin(th[m])], 1))
    a, e = Hl, np.roll(Hl, -1, 0) - Hl
    d = np.stack([np.cos(th), np.sin(th)], 1)

    def cross(u, v):
        return u[..., 0] * v[..., 1] - u[..., 1] * v[..., 0]
    den = cross(d[:, None], e[None])
    den = np.where(np.abs(den) < 1e-12, 1e-12, den)
    sd, t = cross(a[None], e[None]) / den, cross(a[None], d[:, None]) / den
    ok = (t > -1e-6) & (t < 1 + 1e-6) & (sd > 0)
    return np.where(ok, sd, 0).max(1)


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


def ghost_sheet(body):
    """La sábana sobre el cuerpo, en la pose neutra: vértices, caras y material de cada cara."""
    th = np.linspace(0, 2 * np.pi, G_SEG, endpoint=False)
    ct, st = np.cos(th), np.sin(th)
    zs = np.linspace(1.32, 1.0, 400)
    top = zs[np.argmax(body.field(np.stack([zs * 0, zs * 0, zs], 1)) < 0)]
    z_rows = np.concatenate([np.linspace(top - 0.002, 0.88, 80), np.linspace(0.868, 0.17, 59)])
    rs = np.arange(0.46, 0.0, -0.0015)
    R = np.zeros((len(z_rows), G_SEG))
    for i, z in enumerate(z_rows):                       # Lo más afuera del cuerpo en cada dirección, más la tela
        P = np.stack([np.outer(ct, rs).ravel(), np.outer(st, rs).ravel(), np.full(G_SEG * len(rs), z)], 1)
        f = body.field(P).reshape(G_SEG, len(rs))
        neg = f < 0
        j = np.argmax(neg, 1)
        fi, fo = f[np.arange(G_SEG), j], f[np.arange(G_SEG), np.maximum(j - 1, 0)]
        R[i] = np.where(neg.any(1), rs[j] + 0.0015 * -fi / np.maximum(fo - fi, 1e-9) + (0.009 if z > 0.97 else 0.006),
                        0.0)
    skin_r = R.copy()
    for i in range(1, len(z_rows)):                      # Cuelga: en la cara se pega (cae rápido), del cuerpo cae casi recta
        c = 0.35 if z_rows[i - 1] > 0.97 else 0.12
        R[i] = np.maximum(R[i], R[i - 1] - c * (z_rows[i - 1] - z_rows[i]))
    for i in range(len(z_rows)):
        R[i] = hull_radius(th, R[i])
    for it in range(8):                                  # A lo largo de la vuelta (en la cabeza, más: sin aristas)
        Rs = 0.25 * np.roll(R, 1, 1) + 0.5 * R + 0.25 * np.roll(R, -1, 1)
        R = Rs if it < 3 else np.where((z_rows > 0.95)[:, None], Rs, R)
    low = z_rows < 0.975                                 # Hombros redondos: se suaviza a lo alto debajo de la cabeza
    for _ in range(20):
        Rz = R.copy()
        Rz[1:-1] = 0.25 * R[:-2] + 0.5 * R[1:-1] + 0.25 * R[2:]
        R[low] = Rz[low]
    # Borde de abajo: ondula y queda flotando apenas sobre el piso
    z_hem = 0.035 + 0.02 * (0.5 + 0.5 * np.sin(3 * th + 1.0)) + 0.01 * np.sin(7 * th + 2.0)
    Z = np.repeat(z_rows[:, None], G_SEG, 1)
    hem_t = np.arange(1, 9)[:, None] / 8
    Zh = 0.17 + hem_t * (z_hem[None] - 0.17)
    Z = np.concatenate([Z, Zh])
    R = np.concatenate([R, R[-1][None] - 0.12 * (0.17 - Zh)])
    skin_r = np.concatenate([skin_r, np.zeros_like(Zh)])
    R += 0.035 * np.clip((0.4 - Z) / 0.36, 0, 1) ** 2   # Se abre un poco al llegar abajo
    # Pliegues: verticales, desparejos, más hondos y anchos abajo; arriba se juntan hacia los hombros
    drop = np.clip(0.92 - Z, 0, None)
    amp = (0.0015 + 0.02 * smoothstep(0, 0.6, drop)) * (drop > 0)
    thw = th + 0.3 * (1 - smoothstep(0, 0.7, drop)) * np.sin(2 * th)
    x = 14 * thw + 2.2 * np.sin(3 * thw + 1.0) + 1.2 * np.sin(5 * thw + 2.0)
    fold = (np.sin(x) + 0.35 * np.sin(2.1 * x + 0.7)) * (0.6 + 0.4 * np.sin(4 * th + 1.0))
    # Arrugas suaves que bajan de la mandíbula y la nuca a los hombros
    neck = smoothstep(0.88, 0.93, Z) * (1 - smoothstep(0.98, 1.03, Z))
    fold = fold + neck * 0.0035 / np.maximum(amp, 1e-4) * np.sin(9 * th + 1.5 * np.sin(2 * th) + 0.5)
    R = np.maximum(R + amp * fold, np.where(skin_r > 0, skin_r - 0.003, 0))

    V = [(0.0, 0.0, top + 0.004)]
    for i in range(len(Z)):
        V += list(zip(R[i] * ct, R[i] * st, Z[i]))
    V = np.array(V)
    n = len(Z)

    def vid(i, k):
        return 1 + i * G_SEG + k % G_SEG
    F, M = [], []
    for k in range(G_SEG):
        F.append((0, vid(0, k), vid(0, k + 1)))
        M.append(GHOST)
    for i in range(n - 1):
        for k in range(G_SEG):
            F.append((vid(i, k), vid(i + 1, k), vid(i + 1, k + 1), vid(i, k + 1)))
            M.append(GHOST)
    return V, F, M


def g_prism(outline, y0, y1):
    """Prisma a lo largo de y con el contorno (x, z) dado, relativo al centro de la cabeza."""
    n = len(outline)
    V = [GH + Vector((x, y, z)) for y in (y0, y1) for x, z in outline]
    F = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    F += [(k, (k + 1) % n, n + (k + 1) % n, n + k) for k in range(n)]
    return V, F


def ghost_cut(V, F, M):
    """Recorta en la tela los dos agujeritos de los ojos (a tijera: no son círculos perfectos) y la boca."""
    cuts = []
    for i, e in enumerate(G_EYES):
        o = e - GH
        ring = []
        for k in range(16):
            a = 2 * math.pi * k / 16
            r = 1 + 0.07 * math.sin(3 * a + 1.3 * i) + 0.05 * math.sin(5 * a + 0.4 + i)
            r = 1 + 0.4 * (r - 1)                         # Casi redondos: dejan ver todo el ojo
            ring.append((o.x + 0.0118 * r * math.cos(a), o.z + 0.0118 * r * math.sin(a)))
        cuts.append(g_prism(ring, -0.2, -0.072))
    mw = G_MOUTH_W + 0.006
    xs = np.linspace(-mw, mw, 19)
    g = np.sqrt(np.maximum(0, 1 - (xs / mw) ** 2))
    mouth = [(x, g_mouth_z(x) + 0.0125 * gg) for x, gg in zip(xs, g)]
    mouth += [(x, g_mouth_z(x) - 0.0145 * gg) for x, gg in list(zip(xs, g))[-2:0:-1]]
    cuts.append(g_prism(mouth, -0.2, -0.062))
    return carve(V, F, M, cuts, GHOST_CUT, keep_cut=False)


def ghost_face_parts():
    """Lo que se ve por los agujeros: ojos, dientes y el fondo oscuro de la boca (pegado a la cabeza)."""
    A = Builder(MATS)
    for e in G_EYES:
        # Mirada muerta: bien abiertos, sin párpados, blancos redondos con un puntito de pupila en el centro (apenas
        # un anillo azul alrededor) y un aro oscuro alrededor del ojo
        A.ellipsoid(e + Vector((0, 0.003, 0)), rx(0), (0.0138, 0.0138, 0.0138), HOLE, seg=16, rings=10)
        A.ellipsoid(e, rx(0), (0.0115, 0.0115, 0.0115), GHOST_SCLERA, seg=16, rings=12)
        A.ellipsoid(e + Vector((0, -0.0108, 0)), rx(0), (0.0032, 0.0012, 0.0032), GHOST_IRIS, seg=12, rings=5)
        A.ellipsoid(e + Vector((0, -0.0119, 0)), rx(0), (0.0016, 0.0008, 0.0016), GHOST_PUPIL, seg=8, rings=4)
    c = GH
    xs = list(np.linspace(-G_MOUTH_W - 0.002, G_MOUTH_W + 0.002, 11))
    A.tube([c + Vector((x, g_face_y(x, g_mouth_z(x)) + 0.013, g_mouth_z(x))) for x in xs],
           [0.003, 0.0045] + [0.0055] * 7 + [0.0045, 0.003], 8, HOLE, cap0=True, cap1=True)
    n = 14                                               # Dientes de verdad, parejos: catorce arriba y catorce abajo
    for i in range(n):
        x = (i - (n - 1) / 2) * 0.0062
        w = 0.0033 + 0.0008 * math.exp(-(x / 0.008) ** 2)    # Los de adelante más anchos; sin huecos entre ellos
        zt = g_mouth_z(x)
        y = g_face_y(x, zt) - 0.0005 + 0.25 * x * x / 0.03
        ref = rz(-math.degrees(x / 0.05)) @ Vector((0, -1, 0))   # De frente, siguiendo la curva de la arcada
        # Cuadrados, como dientes de verdad: un bloque que se afina un poco hacia el borde que muerde
        top = c + Vector((x, y, zt + 0.0095))
        A.tube([top, top + Vector((0, 0, -0.008)), top + Vector((0, 0, -0.0105))], [1.0, 0.95, 0.75], 8, TEETH,
               sx=w * 0.9, sy=0.0018, cap0=True, cap1=True, ref=ref)
        bot = c + Vector((x * 0.94, y + 0.003, zt - 0.0085))
        A.tube([bot, bot + Vector((0, 0, 0.0065)), bot + Vector((0, 0, 0.0085))], [1.0, 0.95, 0.8], 8, TEETH,
               sx=w * 0.75, sy=0.0016, cap0=True, cap1=True, ref=ref)
    return A


_GHOST = {}


def build_ghost(pose):
    if not _GHOST:                                       # Todo se arma una sola vez, en la pose neutra
        rest, arms = ghost_skeleton(GHOST_REST)
        body = Weld()
        ghost_head(body, detail=False)
        ghost_torso(body, arms)
        V, F, M = ghost_cut(*ghost_sheet(body))
        V, F, M = decimate(V, F, G_TRIS, M)
        bones, Wt = body.weights(V, 0.02)
        face = Weld()
        ghost_head(face, detail=True)
        FV, FQ = face.surface(0.0015)
        C = FV[FQ].mean(1)
        FQ = FQ[(C[:, 1] < GH.y - 0.035) & (C[:, 2] > GH.z - 0.115)]   # Solo el frente de la cara
        used = np.unique(FQ)
        FV, FQ = FV[used], np.searchsorted(used, FQ)
        FV, FF = decimate(FV, FQ, G_FACE_TRIS)
        FM = face.face_materials(FV, FF, GHOST_SKIN)
        A = ghost_face_parts()
        _GHOST.update(rest=rest, V=V, F=F, M=M, bones=bones, Wt=Wt, FV=FV, FF=FF, FM=FM, A=A,
                      AV=np.array([tuple(v) for v in A.v]))
    G = _GHOST
    posed, _ = ghost_skeleton(pose)
    one = np.ones((1, 1))
    parts = [(skin(G["V"], G["bones"], G["Wt"], G["rest"], posed), G["V"], G["F"], G["M"]),
             (skin(G["FV"], ["head"], np.ones((len(G["FV"]), 1)), G["rest"], posed), G["FV"], G["FF"], G["FM"]),
             (skin(G["AV"], ["head"], np.ones((len(G["AV"]), 1)), G["rest"], posed), G["AV"], G["A"].f,
              [MATS[i] for i in G["A"].m])]
    del one
    B = Builder(MATS)
    for P, P0, F, M in parts:
        # Se mece de lado (más arriba, más) y el borde de abajo ondula, según dónde estaba cada punto
        P = P.copy()
        P[:, 0] += pose["sway"] * (np.maximum(P[:, 2], 0) / 1.2) ** 1.5
        w = np.clip((0.25 - P0[:, 2]) / 0.22, 0, 1)
        u = np.arctan2(P0[:, 0], -P0[:, 1])
        P[:, 2] -= 0.03 * w * (0.5 + 0.5 * np.cos(5 * u + pose["hem"]))
        off = len(B.v)
        B.v += [Vector(v) for v in P]
        for f, m in zip(F, M):
            B.add(tuple(int(i) + off for i in f), m)
    return B


# --- Demonio de parálisis del sueño -------------------------------------------------------------------------------
# Una sombra demasiado alta, hecha de huesos negros: columna encorvada, una caja de costillas abierta (el detalle va
# ahí: costillas curvas con huecos entre ellas, esternón y la columna que se ve por detrás), cintura de pura columna
# y pelvis. Cuello largo doblado hacia adelante y una cara de calavera que grita: cuencas enormes con un puntito
# blanco al fondo y la boca abierta de par en par, llena de colmillos. Cuatro brazos larguísimos de hueso, con
# articulaciones nudosas y garras, y piernas dobladas para atrás.
# Soldado: se modela una vez en la pose neutra (DEMON_REST) y cada pose dobla esa malla con los huesos.
DEMON_POSES = {
    # Acechando (dos pasos), muy encorvado pero con la cara levantada, mirando hacia abajo a quien tiene adelante; los
    # brazos de abajo cuelgan, los de arriba adelante queriendo agarrar
    "walk_a": {"hunch": 38, "neck": 35, "head": (-45, 0, 22),
               "arm_R": (25, -75, -25, 0), "arm_L": (12, -20, -30, 0),
               "arm2_R": (20, -10, -40, 0), "arm2_L": (30, -60, -20, 0), "leg_R": (-30, 6), "leg_L": (10, -4)},
    "walk_b": {"hunch": 38, "neck": 35, "head": (-45, 0, -18),
               "arm_R": (12, -20, -30, 0), "arm_L": (25, -75, -25, 0),
               "arm2_R": (30, -60, -20, 0), "arm2_L": (20, -10, -40, 0), "leg_R": (10, 6), "leg_L": (-30, -4)},
    # "Baila": se retuerce, brazos abiertos en abanico con las garras, la cabeza al revés de lo normal
    "dance_a": {"hunch": 20, "neck": 20, "head": (-30, 0, 40),
                "arm_R": (160, 10, -50, 0), "arm_L": (70, -30, -100, 0),
                "arm2_R": (80, 20, -60, 0), "arm2_L": (120, -10, -40, 0), "leg_R": (-40, 10), "leg_L": (0, -8)},
    "dance_b": {"hunch": 26, "neck": 25, "head": (-30, 0, -40),
                "arm_R": (70, -30, -100, 0), "arm_L": (160, 10, -50, 0),
                "arm2_R": (120, -10, -40, 0), "arm2_L": (80, 20, -60, 0), "leg_R": (0, 8), "leg_L": (-40, -10)},
}
# Pose neutra donde se suelda: las extremidades separadas, para que no se peguen entre sí
DEMON_REST = {"hunch": 28, "neck": 28, "head": (0, 0, 0),
              "arm_R": (30, 0, -10, 0), "arm_L": (30, 0, -10, 0),
              "arm2_R": (15, -45, -10, 0), "arm2_L": (15, -45, -10, 0), "leg_R": (0, 5), "leg_L": (0, 5)}
DEMON_WELD_H = 0.0045                                    # Celda de la rejilla
DEMON_TRIS = 12000                                       # Triángulos después de simplificar


def demon_skeleton(pose):
    """Puntos del demonio en una pose y sus huesos ({nombre: (origen, rotación)})."""
    S = {"pose": pose}
    hunch = pose["hunch"]
    hip = Vector((0, 0, 1.0))
    spine = [hip]
    for i in range(1, 9):
        spine.append(spine[-1] + rx(hunch * i / 8) @ Vector((0, 0, 0.075)))
    bones = {"pelvis": (hip, rx(0))}
    for i in range(8):
        bones["spine%d" % i] = (spine[i], rx(hunch * (i + 1) / 8))
    Rc = rx(hunch * 0.75)                                # Orientación del pecho
    chest = spine[8]
    bones["cage"] = (spine[6], Rc)
    Rn = rx(hunch + pose["neck"])
    neck = [chest + Rn @ Vector((0, 0, 0.05 * i)) for i in range(5)]
    bones["neck"] = (chest, Rn)
    hx, hy, hz = pose["head"]
    Rh = Rn @ rz(hy) @ ry(hz) @ rx(hx)
    bones["head"] = (neck[-1], Rh)
    arms = []
    for side in (-1, 1):                                 # Cuatro brazos: los de arriba y otro par más abajo
        for key, sh, R0, length in (
                ("arm", chest + Rc @ Vector((side * 0.16, -0.02, -0.03)), Rc, 0.44),
                ("arm2", spine[5] + Rc @ Vector((side * 0.13, -0.03, 0.0)), rx(hunch * 0.6), 0.36)):
            name = "%s_%s" % (key, "R" if side > 0 else "L")
            abd, flex, elbow, twist = pose[name]
            Ru = R0 @ ry(-side * abd) @ rx(flex) @ rz(side * twist)
            el = sh + Ru @ Vector((0, 0, -length))
            Rf = Ru @ rx(elbow)
            wr = el + Rf @ Vector((0, 0, -length * 0.95))
            bones[name + "_up"] = (sh, Ru)
            bones[name + "_fore"] = (el, Rf)
            arms.append((name, side, sh, Ru, el, Rf, wr))
    legs = []
    for side, key in ((1, "leg_R"), (-1, "leg_L")):
        flex, abd = pose[key]
        hp = hip + Vector((side * 0.08, -0.02, -0.03))
        R1 = ry(-side * abd) @ rx(flex)
        bones[key] = (hp, R1)
        legs.append((key, hp, R1))
    S.update(hunch=hunch, hip=hip, spine=spine, Rc=Rc, chest=chest, Rn=Rn, neck=neck, Rh=Rh,
             head=neck[-1] + Rh @ Vector((0, 0, 0.09)), arms=arms, legs=legs)
    return S, bones


def demon_parts(S):
    """Piezas soldadas (Weld) y piezas finas pegadas a un hueso (Builder + hueso de cada vértice), en la pose S."""
    W = Weld()
    A = Builder(MATS)
    abone = []

    def tag(bone, n0):
        abone.extend([bone] * (len(A.v) - n0))
    hunch, hip, spine, Rc, chest = S["hunch"], S["hip"], S["spine"], S["Rc"], S["chest"]
    # Columna: vértebras encadenadas con apófisis que salen para atrás
    for i in range(8):
        W.capsule("spine%d" % i, spine[i], spine[i + 1], 0.028, k=0.01)
    for i, q in enumerate(spine):
        W.ellipsoid("spine%d" % min(i, 7), q + rx(hunch * i / 8) @ Vector((0, 0.024, 0)), rx(hunch * i / 8),
                    (0.022, 0.032, 0.02), k=0.008)
    # Costillas: pares curvos que salen de la columna, rodean el pecho y casi se juntan adelante en el esternón,
    # más largas en el medio, con hueco entre una y otra (ahí se ve lo oscuro de adentro)
    for i in range(7):
        t = i / 6
        q = spine[3].lerp(spine[7], 1 - t)              # De abajo (cortas, flotantes) hacia arriba
        Rq = rx(hunch * (0.35 + 0.45 * (1 - t)))
        size = 0.09 + 0.05 * math.sin(math.pi * min(1.0, (1 - t) * 1.15))
        reach = 0.82 if i > 1 else 0.55                  # Las dos de abajo no llegan al esternón
        for side in (-1, 1):
            pts = [q + Rq @ Vector((side * size * math.sin(a) * 1.25, size * math.cos(a) * 0.9, -0.03 * math.sin(a)))
                   for a in (math.pi * reach * k / 8 for k in range(9))]
            W.chain("cage", pts, [0.013, 0.012, 0.011, 0.011, 0.011, 0.01, 0.0095, 0.009, 0.008], k=0.008)
    st_top = spine[7] + Rc @ Vector((0, -0.13, -0.02))   # Esternón
    st_bot = spine[5] + Rc @ Vector((0, -0.12, 0.0))
    W.capsule("cage", st_top, st_bot, 0.018, 0.012, k=0.01)
    for side in (-1, 1):                                 # Clavículas hasta los hombros y pelvis de dos alas
        W.capsule("cage", st_top, chest + Rc @ Vector((side * 0.15, -0.02, -0.02)), 0.012, 0.014, k=0.01)
        W.chain("pelvis", [hip + Vector((0, 0.02, 0.02)), hip + Vector((side * 0.1, -0.02, 0.05)),
                           hip + Vector((side * 0.09, -0.03, -0.05))], [0.02, 0.022, 0.012], k=0.012)
    # Cuello largo de vértebras y cabeza huesuda
    Rn, neck, Rh, head = S["Rn"], S["neck"], S["Rh"], S["head"]
    W.chain("neck", neck, [0.02] * 5, k=0.01)
    for q in neck[1:]:
        W.ellipsoid("neck", q, Rn, (0.022, 0.022, 0.016), k=0.008)
    # Cabeza de calavera que grita (referencia de Cami): cráneo pelado y abultado con la frente arrugada, cuencas
    # enormes y hondas con un puntito blanco al fondo, nariz de calavera y la boca abierta de par en par, hacia abajo,
    # rodeada de colmillos
    CRANIUM = (Vector((0, 0.01, 0.03)), (0.078, 0.085, 0.09))
    JAW = (Vector((0, -0.02, -0.06)), (0.055, 0.06, 0.085))
    W.ellipsoid("head", head + Rh @ CRANIUM[0], Rh, CRANIUM[1], k=0.02)
    W.ellipsoid("head", head + Rh @ JAW[0], Rh, JAW[1], k=0.025)

    def front(x, z, out=0.0):
        """Frente de la cara (lo más adelante entre el cráneo y la mandíbula) en x, z."""
        ys = []
        for c, r in (CRANIUM, JAW):
            k = 1 - (x / r[0]) ** 2 - ((z - c.z) / r[2]) ** 2
            if k > 0:
                ys.append(c.y - r[1] * math.sqrt(k))
        return head + Rh @ Vector((x, min(ys, default=-0.05) - out, z))
    for sd in (-1, 1):
        W.ellipsoid("head", head + Rh @ Vector((sd * 0.05, -0.05, -0.02)), Rh, (0.022, 0.02, 0.02), k=0.012)  # Pómulos
    for i, z in enumerate((0.058, 0.073, 0.087)):          # Arrugas de la frente
        W.chain("head", [front(x, z + 0.004 * math.sin(x * 90 + i), 0.001) for x in [j * 0.012 - 0.054 for j in range(10)]],
                [0.002] + [0.0035] * 8 + [0.002], k=0.004)
    for sd in (-1, 1):
        # Cuenca enorme y honda, y al fondo el puntito blanco que mira
        W.ellipsoid("head", front(sd * 0.031, 0.02) + Rh @ Vector((0, 0.008, 0)), Rh @ ry(sd * -12), (0.034, 0.036, 0.033),
                    k=0.008, sub=True)
        n0 = len(A.v)
        A.ellipsoid(front(sd * 0.031, 0.02) + Rh @ Vector((0, 0.03, 0)), Rh, (0.0065, 0.0065, 0.0065), DEMON_EYES,
                    seg=8, rings=6)
        tag("head", n0)
        # Nariz de calavera: dos huecos que se juntan abajo
        W.ellipsoid("head", front(sd * 0.008, -0.025) + Rh @ Vector((0, 0.006, 0)), Rh @ ry(sd * 20),
                    (0.008, 0.014, 0.015), k=0.004, sub=True)
    # La boca: un óvalo enorme hacia abajo, abierta en un grito
    mouth_z, mw, mh = -0.095, 0.03, 0.064
    W.ellipsoid("head", front(0, mouth_z) + Rh @ Vector((0, 0.02, 0)), Rh, (mw, 0.05, mh), k=0.006, sub=True)
    n0 = len(A.v)
    for a_deg in range(15, 360, 15):                       # Colmillos todo alrededor, apuntando adentro
        a = math.radians(a_deg)
        if abs(math.sin(a)) < 0.45:                        # Arriba y abajo, no a los costados (como un grito)
            continue
        x, z = mw * 0.95 * math.cos(a), mouth_z + mh * 0.92 * math.sin(a)
        base = front(x, z) + Rh @ Vector((0, 0.006, 0))
        canine = a_deg in (60, 120, 240, 300)
        length = 0.034 if canine else 0.017
        inward = (Vector((0, 0, mouth_z)) - Vector((x, 0, z))).normalized()
        tip = base + Rh @ (inward * length + Vector((0, 0.004, 0)))
        A.tube([base, tip], [0.006 if canine else 0.0045, 0.0003], 5, TEETH, cap1=True)
    tag("head", n0)

    for name, side, sh, Ru, el, Rf, wr in S["arms"]:    # Brazos de hueso: húmero, codo nudoso, antebrazo doble
        up, fore = name + "_up", name + "_fore"
        W.ellipsoid(up, sh, Ru, (0.03, 0.03, 0.03), k=0.012)
        W.chain(up, [sh, sh.lerp(el, 0.5), el], [0.016, 0.012, 0.015], k=0.012)
        W.ellipsoid(up, el, Ru, (0.024, 0.024, 0.028), k=0.01)
        for o in (-1, 1):
            off = Rf @ Vector((o * 0.009, 0, 0))
            W.chain(fore, [el + off, el.lerp(wr, 0.5) + off * 1.4, wr + off], [0.009, 0.007, 0.008], k=0.006)
        W.ellipsoid(fore, wr, Rf, (0.02, 0.015, 0.02), k=0.01)
        n0 = len(A.v)
        for f in range(5):                               # Cinco dedos largos de hueso, con garras curvas y afiladas
            Rd = Rf @ ry(side * (f - 2) * 12)
            base = wr + Rf @ Vector((side * (f - 2) * 0.012, 0, -0.025))
            k1 = base + Rd @ Vector((0, 0, -0.09))
            k2 = k1 + Rd @ rx(-25) @ Vector((0, 0, -0.08))
            claw = k2 + Rd @ rx(-55) @ Vector((0, 0, -0.06))
            W.chain(fore, [base, k1, k2], [0.0075, 0.0068, 0.0062], k=0.004)
            W.ellipsoid(fore, k1, Rd, (0.009, 0.009, 0.01), k=0.004)   # Nudillos
            A.tube([k2, k2.lerp(claw, 0.5), claw], [0.0062, 0.004, 0.0005], 5, DEMON, cap1=True)
        tag(fore, n0)
    for key, hp, R1 in S["legs"]:                        # Piernas de hueso dobladas para atrás
        kn = hp + R1 @ rx(-20) @ Vector((0, 0, -0.38))
        hock = kn + R1 @ rx(55) @ Vector((0, 0, -0.36))
        foot = hock + R1 @ rx(-10) @ Vector((0, 0, -0.3))
        toe = foot + R1 @ Vector((0, -0.12, -0.01))
        W.ellipsoid(key, hp, R1, (0.026, 0.026, 0.026), k=0.012)
        W.chain(key, [hp, hp.lerp(kn, 0.5), kn], [0.02, 0.014, 0.018], k=0.012)
        W.ellipsoid(key, kn, R1, (0.026, 0.026, 0.03), k=0.01)          # Rodilla
        W.chain(key, [kn, kn.lerp(hock, 0.5), hock], [0.014, 0.011, 0.013], k=0.01)
        W.ellipsoid(key, hock, R1, (0.02, 0.02, 0.022), k=0.01)
        W.chain(key, [hock, foot, toe], [0.011, 0.01, 0.005], k=0.008)
    return W, A, abone


_DEMON = {}


def build_demon(pose):
    if not _DEMON:                                       # La malla soldada se arma una sola vez
        S, rest = demon_skeleton(DEMON_REST)
        W, A, abone = demon_parts(S)
        V, F = W.surface(DEMON_WELD_H)
        print("WELD demonio: %d vértices antes de simplificar" % len(V))
        V, F = decimate(V, F, DEMON_TRIS)
        bones, Wt = W.weights(V, 0.008)
        _DEMON.update(rest=rest, A=A, abone=abone, V=V, F=F, bones=bones, Wt=Wt)
    D = _DEMON
    _, posed = demon_skeleton(pose)
    B = Builder(MATS)
    B.v = [Vector(v) for v in skin(D["V"], D["bones"], D["Wt"], D["rest"], posed)]
    for f in D["F"]:
        B.add(f, DEMON)
    off = len(B.v)
    B.v += [rigid(v, b, D["rest"], posed) for v, b in zip(D["A"].v, D["abone"])]
    B.f += [tuple(i + off for i in f) for f in D["A"].f]
    B.m += D["A"].m
    return B


# --- Armado: una raíz por pose, con el origen en el piso ----------------------------------------------------------
ROOTS = []
for ci, (cname, builder, poses) in enumerate((("Devil", build_devil, DEVIL_POSES),
                                             ("Ghost", build_ghost, GHOST_POSES),
                                             ("Demon", build_demon, DEMON_POSES))):
    counts = set()
    for pi, (pname, pose) in enumerate(poses.items()):
        B = builder(pose)
        lo = min(v.z for v in B.v)
        root = bpy.data.objects.new("Halloween_Game_%s_%s_ROOT" % (cname, pname), None)
        scene.collection.objects.link(root)
        row = {"Devil": 0, "Demon": 1, "Ghost": 2}[cname]          # Los fantasmas atrás: son altos y tapan
        root.location = (pi * 0.9, row * 1.6, 0) if cname != "Demon" else (pi * 1.1 - 0.2, row * 1.6, 0)
        shift = Vector((0, 0, lo)) if cname != "Ghost" else Vector()   # El fantasma flota: no se baja al piso
        if cname == "Devil" and pname.startswith("walk"):
            # Doblado así, la cara va bien adelante de la cadera: se corre todo para atrás, así la que llega a la
            # cara de la bruja es la cara y no la cadera
            shift += Vector((0, -DEVIL_STALK_BACK, 0))
        me = make_mesh("HWG_%s_%s" % (cname, pname), B, MATS, shift)
        ob = bpy.data.objects.new("HWG_%s_%s" % (cname, pname), me)
        scene.collection.objects.link(ob)
        ob.parent = root
        me.calc_loop_triangles()
        counts.add((len(me.vertices), len(me.polygons)))
        print("TRIS %-22s %d" % (ob.name, len(me.loop_triangles)))
        ROOTS.append(root)
    assert len(counts) == 1, "%s: las poses no tienen la misma malla: %s" % (cname, counts)

# --- Presentación -----------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#3a0a0e"), 1)
scene.world = world
for name, rot, col, e in (("Key", (math.radians(55), 0, math.radians(-35)), "#ffe2c8", 4.0),
                          ("Rim", (math.radians(70), 0, math.radians(160)), "#8fa0ff", 2.0)):
    ld = bpy.data.lights.new(name, "SUN")
    ld.energy, ld.color = e, srgb(col)
    lo_ = bpy.data.objects.new(name, ld)
    pres.objects.link(lo_)
    lo_.rotation_euler = rot
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1500, 1100
scene.view_settings.view_transform = "Standard"
os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if not os.environ.get("HW_NO_RENDER"):
    c = Vector((1.35, 1.6, 0.8))
    cam.location = c + Vector((0.1, -1, 0.3)).normalized() * 7.5
    cam.rotation_euler = (c - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = os.path.join(REF_DIR, "game_creatures.png")
    bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
