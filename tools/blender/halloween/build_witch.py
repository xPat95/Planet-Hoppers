"""Generador de la bruja del planeta Halloween (modelo base para todas las brujas de las órbitas).

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/halloween/build_witch.py

Guarda assets/models/characters/halloween/witch.blend (un modelo por pose: HW_Witch_Solo_A, Solo_B, Solo_C, Pair,
Crawl, Climb, Peek y Sit) y capturas en docs/references/halloween/witches_*.png. El planeta las importa.
Diseño: docs/cami-guidelines.md, sección 2.

La bruja se arma sobre un esqueleto simple: cada pose es solo un juego de ángulos (cadera, torso, brazos, piernas y
cabeza), así las variantes (bailando sola, en pareja, gateando, colgada del borde) salen del mismo modelo. Las piezas
son tubos de pocos lados que siguen los huesos: piernas y brazos de piel verde pálida, vestido negro corto con el
borde en picos, botas de tacón, pelo largo en mechones y sombrero negro con el ala ancha y la punta doblada.
Estilo de ilustración plana de moda: cabeza chica, piernas muy largas y colores planos sin brillo. La cara casi
no tiene detalle: dos ojitos verdes que brillan. Lleva botas negras de tacón hasta media pierna y
medias a rayas blancas y negras hasta arriba de la rodilla. El pelo es un solo panel que cae ondulando.
"""

import bpy
import math
import os
import sys
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from figure import Builder, SmoothBody, make_mesh, mat, rx, ry, rz, srgb  # noqa: E402  (piezas compartidas)

REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "characters", "halloween", "witch.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "halloween")

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "Halloween_Witch"


SKIN = mat("WitchSkin", "#6fbf4a")                       # Verde
BLACK = mat("WitchBlack", "#141014")                     # Vestido, sombrero y tacones
HAIR = mat("WitchHair", "#1d1424")
BAND = mat("WitchHatBand", "#4a2a5a")
EYES = mat("WitchEyes", "#b6ff3a", emit=6.0)
WHITE = mat("WitchTightsWhite", "#ebe6ef")              # Medias a rayas: blanco y negro
BEAD = mat("WitchBeadEyes", "#050405", emit=0.01, rough=0.12)   # Ojos de cuenta: negros y brillantes
BLUSH = mat("WitchBlush", "#ff8fb4")                     # Cachetes rosados
HAIR_JET = mat("WitchHairJet", "#0d1430")
BEAD_RED = mat("WitchBeadEyesRed", "#c8102a", emit=0.6, rough=0.08)   # Ojos de cuenta rojos y brillantes
SHINE = mat("WitchEyeShine", "#ffffff", emit=3.0)        # El brillito de los ojos                # Azul azabache: muy oscuro, pero se distingue del negro
BAND_EDGE = mat("WitchBandEdge", "#75518b")
MATS = [SKIN, BLACK, HAIR, BAND, EYES, WHITE, BEAD, BLUSH, HAIR_JET, BEAD_RED, SHINE,
        BAND_EDGE]


HS = 0.78                    # Escala de la cabeza (cabeza chica, como en ilustración de moda)
THIGH, SHIN = 0.34, 0.35     # Piernas muy largas
STRIPE = 0.04                # Ancho de cada raya de las medias


# Pose base (ángulos en grados). Frente hacia -Y. Flexión negativa = hacia adelante.
POSES = {
    # Bailando sola (cadera de lado): cadera muy ladeada y el torso al revés, rodilla doblada cruzando hacia
    # adelante en punta de pie y los dos brazos doblados arriba, cerca de la cabeza
    "solo_a": {
        "pelvis": (0, 14, 15),          # Adelante/atrás, cadera ladeada, giro
        "torso": (-6, -18, -12),
        "head": (-5, 12, 15),
        "leg_R": (-2, 12, 4),           # Flexión de cadera, apertura, rodilla
        "leg_L": (-40, -20, 75),
        "arm_R": (140, -10, -95, 30),   # Apertura (levantar al lado), flexión, codo, giro del brazo
        "arm_L": (140, 0, -110, 70),
    },
    # Bailando sola (fiesta): brazos de cactus (codos doblados, antebrazos hacia arriba) y una rodilla levantada
    "solo_b": {
        "pelvis": (0, -8, -8), "torso": (2, 10, 8), "head": (-6, -10, 0),
        "leg_R": (-3, -8, 6), "leg_L": (-55, -12, 85),
        "arm_R": (95, 0, -95, 90), "arm_L": (95, 0, -95, 90),
    },
    # Bailando sola (disco): estocada ancha, torso girado, un brazo señalando arriba en diagonal y el otro cruzado
    "solo_c": {
        "pelvis": (8, -8, 25), "torso": (10, -14, 12), "head": (-10, 12, 20),
        "leg_R": (-35, 20, 35), "leg_L": (20, 20, 10),
        "arm_R": (150, -20, 0, 0), "arm_L": (85, -25, -10, 0),
    },
    # En pareja (cada una): un brazo hacia la otra para tomarse de las manos y el otro arriba
    "pair": {
        "pelvis": (0, 4, 0), "torso": (2, -6, 0), "head": (-4, 6, 0),
        "leg_R": (-10, 4, 12), "leg_L": (-22, -4, 30),
        "arm_R": (150, 0, -30, 0), "arm_L": (12, -75, -10, 0),
    },
    # Gateando (saliendo por arriba de la órbita): en cuatro patas, cabeza levantada, un brazo estirándose
    "crawl": {
        "pelvis": (82, 0, 0), "torso": (6, 0, 0), "head": (-70, 0, 0),
        "leg_R": (-80, 8, 95), "leg_L": (-70, 6, 100),
        "arm_R": (6, -92, 0, 0), "arm_L": (6, -115, 25, 0),
    },
    # Asomándose, como por una ventana: acostada boca abajo, cabeza y hombros más allá del borde y los brazos
    # apoyados en él. En el planeta va de cabeza en el techo de la órbita, asomándose hacia abajo
    "peek": {
        "pelvis": (90, 0, 0), "torso": (-4, 0, 0), "head": (-75, 10, 0),
        "leg_R": (-4, 3, 25), "leg_L": (0, -3, 50),
        "arm_R": (22, -80, -60, 0), "arm_L": (18, -70, -75, 0),
    },
    # Sentada en el borde de la órbita, columpiando las piernas: muslos hacia adelante, una pierna patea hacia
    # adelante y la otra se va hacia atrás; las manos apoyadas atrás en el borde
    "sit": {
        "pelvis": (-8, 0, 0), "torso": (14, 0, 0), "head": (-14, 8, 0),
        "leg_R": (-88, 6, 45), "leg_L": (-82, -6, 125),
        "arm_R": (25, 40, -15, 0), "arm_L": (25, 40, -15, 0),
    },
    # Sentada de costado en la escoba (la nave), como amazona: la cadera girada hacia un lado con las dos piernas
    # juntas colgando de ese lado, y el torso girado de vuelta hacia adelante, inclinado como volando rápido. Una mano
    # en el palo y la otra saludando
    "ride": {
        # Suelta, nada de robot: la cadera apenas ladeada, el torso curvado de costado y más inclinado, la cabeza
        # ladeada mirando un poco hacia quien mira, una pierna más doblada que la otra
        "pelvis": (-4, 4, -42), "torso": (21, -10, 29), "head": (-13, 12, 17),
        "leg_R": (-84, 4, 80), "leg_L": (-71, -5, 106),
        "arm_R": (24, -58, -25, 0), "arm_L": (100, 10, -72, 60),   # Saludando al costado: codo suave, muñeca girada
    },
    # Colgada del borde de la órbita: brazos estirados arriba y piernas colgando, una pateando
    "climb": {
        "pelvis": (-6, 0, 0), "torso": (-4, 0, 0), "head": (-12, 0, 0),
        "leg_R": (-30, 6, 50), "leg_L": (8, -4, 12),
        "arm_R": (172, 0, -8, 0), "arm_L": (172, 0, -8, 0),
    },
}


def skirt_long(B, pose, pel, Rp, waist):
    """Falda de tela asimétrica (corta adelante, larga atrás): anillos desde la cintura que se abren, con pliegues que
    crecen hacia abajo y el borde ondulado. Adelante se apoya sobre el regazo y termina a mitad del muslo (las rodillas
    se ven); atrás es larga y el viento la lleva hacia atrás (+Y, porque vuela hacia -Y). No atraviesa los muslos (se
    empuja afuera de ellos). Tiene dos caras (afuera y adentro), así se ve bien también desde abajo."""
    thighs = []
    for side, key in ((1, "leg_R"), (-1, "leg_L")):
        flex, abd, _ = pose[key]
        hip = pel + Rp @ Vector((side * 0.05, 0, -0.02))
        thighs.append((hip, hip + (Rp @ ry(-side * abd) @ rx(flex)) @ Vector((0, 0, -THIGH))))
    lap = sum(((b_ - a_) for a_, b_ in thighs), Vector())   # Hacia dónde van las rodillas (el frente de la falda)
    lap.z = 0.0
    lap.normalize()
    n, rows = 64, 12
    rings = []
    for j in range(rows):
        t = j / (rows - 1)
        r = 0.048 + 0.21 * t ** 0.9 - 0.015 * max(0.0, 1 - t / 0.3)  # Tuck early rings inside the hip.
        #                                                  escalón) y se abre por encima de la cadera
        ring = []
        for k in range(n):
            a = math.tau * k / n
            fold = 1 + 0.18 * t * math.sin(7 * a + 1.3) + 0.06 * t * math.sin(13 * a)   # Pliegues hondos
            out = Rp @ Vector((math.cos(a), math.sin(a), 0))
            f_ = min(max((out.dot(lap) + 0.3) / 1.3, 0.0), 1.0)
            front = f_ * f_ * (3 - 2 * f_)               # 1 hacia las rodillas, 0 atrás (sin quiebre: sin puntas)
            drop = 0.076 * front + 0.35 * (1 - front)   # Largo: corta adelante, más larga atrás
            p = pel + Rp @ Vector((r * math.cos(a) * 1.15 * fold, r * math.sin(a) * fold,
                                   0.07 - drop * t + 0.05 * t * math.sin(5 * a + 0.7)))   # Borde ondulado
            p += lap * 0.07 * t * front                  # Adelante se tiende un poco sobre el regazo
            p += Vector((0, 0.16, 0.05)) * t ** 1.6 * (1 - front)   # Viento: atrás, y un poco hacia arriba
            side_ = Rp @ Vector((-math.sin(a), math.cos(a), 0))
            p += side_ * 0.022 * t * math.sin(3 * a + 4.5 * t)  # Vaivén de la tela: los pliegues se mecen
            # Atrás, el final de la falda se enrosca hacia afuera y arriba: deja ver un poco el palo de la escoba
            # (solo el borde; más arriba la falda sigue tapando)
            backness = max(0.0, out.dot(Vector((0, 1, 0)))) ** 0.8   # Ancho: se enrosca redondo, sin pico
            r_ = min(max((t - 0.35) / 0.65, 0.0), 1.0)
            roll = r_ * r_ * (3 - 2 * r_) * backness     # Entra de a poco: se curva, sin quiebre
            p += out * 0.04 * roll + Vector((0, 0, 0.145)) * roll
            for a_, b_ in thighs:                        # Cae sobre los muslos, sin atravesarlos
                ab = b_ - a_
                h = max(0.0, min(1.0, (p - a_).dot(ab) / ab.length_squared))
                q = a_ + ab * h
                dv = p - q
                if dv.length < 0.058:                    # Pegada a los muslos (sin hacer carpa)
                    p = q + (dv.normalized() if dv.length > 1e-6 else Vector((0, 0, 1))) * 0.058
            ring.append(p)
        rings.append(ring)
    for j in range(1, rows - 1):                         # Suavizar un poco entre anillos (tela, no escalones)
        for k in range(n):
            rings[j][k] = rings[j][k] * 0.6 + (rings[j - 1][k] + rings[j + 1][k]) * 0.2
    for j in range(1, rows):                             # Y a lo largo de cada anillo: el borde queda redondeado
        for _ in range(3 if j < rows - 3 else 6):       # El borde, más redondeado todavía
            rings[j] = [rings[j][k] * 0.5 + (rings[j][k - 1] + rings[j][(k + 1) % n]) * 0.25 for k in range(n)]
    outer = []
    for ring in rings:
        outer.append(len(B.v))
        B.v.extend(ring)
    inner = []
    for j, ring in enumerate(rings):                     # Cara de adentro, apenas metida
        c = sum(ring, Vector()) / n
        inner.append(len(B.v))
        B.v.extend([p + (c - p).normalized() * 0.004 for p in ring])
    for j in range(rows - 1):
        for k in range(n):
            k1 = (k + 1) % n
            B.add((outer[j] + k, outer[j] + k1, outer[j + 1] + k1, outer[j + 1] + k), BLACK)
            B.add((inner[j] + k, inner[j + 1] + k, inner[j + 1] + k1, inner[j] + k1), BLACK)
    j = rows - 1
    for k in range(n):                                   # El borde, cerrado
        k1 = (k + 1) % n
        B.add((outer[j] + k, outer[j] + k1, inner[j] + k1, inner[j] + k), BLACK)


def build_witch(pose, happy=False, smooth=False, nose=False, beads=False, long_skirt=False, blush=False,
                flowy_hair=False, hair_mat=None, bead_mat=None, ride_detail=False):
    """happy: ojos cerrados de felicidad (arcos ∩) y una sonrisa grande, en vez de los ojitos verdes. smooth: el
    cuerpo (torso, cuello, cabeza, brazos, manos y piernas) de una sola pieza, con las uniones redondeadas en vez de
    tubos que se cruzan; más triángulos, para la bruja que se ve de cerca (la de la escoba). nose: naricita en punta.
    beads: ojos de cuenta negros y brillantes (en vez de los ojitos o los arcos). blush: dos círculos rosados en los
    cachetes. hair_mat: material del pelo (si no, el de siempre). bead_mat: material de los ojos de cuenta (con un
    brillito blanco). flowy_hair: pelo más largo, con ondas grandes, que el viento lleva hacia atrás. long_skirt: falda larga de tela, con
    pliegues y el borde ondulado, que el viento lleva hacia atrás y que cae sobre los muslos."""
    B = Builder(MATS)
    SB = SmoothBody() if smooth else None
    hair_m = hair_mat or HAIR
    pel = Vector((0, 0, 0.5))
    Rp = rz(pose["pelvis"][2]) @ ry(pose["pelvis"][1]) @ rx(pose["pelvis"][0])
    Rt = Rp @ rz(pose["torso"][2]) @ ry(pose["torso"][1]) @ rx(pose["torso"][0])
    Rh = Rt @ rz(pose["head"][2]) @ ry(pose["head"][1]) @ rx(pose["head"][0])
    waist = pel + Rp @ Vector((0, 0, 0.07))
    chest = waist + Rt @ Vector((0, 0, 0.12))
    neck = chest + Rt @ Vector((0, 0, 0.095 if smooth else 0.07))   # La de cerca, con un cuello que se note
    head = neck + Rh @ Vector((0, 0, 0.085 * HS))
    B.chest = chest.copy()
    B.seat = pel + Rp @ Vector((0, 0.02, -0.06))

    # Vestido: torso ajustado (más ancho que hondo) y falda corta con el borde en picos
    torso = [pel + Rp @ Vector((0, 0, -0.02)), waist, chest - Rt @ Vector((0, 0, 0.03)), chest,
             neck - Rt @ Vector((0, 0, 0.035))]
    if SB:
        torso[-1] = chest + Rt @ Vector((0, 0, 0.035))     # El vestido termina más abajo y más angosto: se ve el cuello
        SB.chain(torso, [0.08, 0.05, 0.066, 0.07, 0.042], BLACK, frame=(Rt, 1.3, 0.9))
    else:
        B.tube(torso, [0.08, 0.05, 0.066, 0.07, 0.056], 10, BLACK, sx=1.3, sy=0.9, ref=Rt @ Vector((0, -1, 0)),
               cap1=True)
    if long_skirt:
        skirt_start = len(B.v)
        skirt_long(B, pose, pel, Rp, waist)
        B.skirt_range = (skirt_start, len(B.v))
    else:
        hem = []
        n = 14
        a, b = Rp @ Vector((1, 0, 0)), Rp @ Vector((0, 1, 0))
        top = B.ring(waist, a, b, 0.054, n, 1.3, 0.9)
        mid = B.ring(pel + Rp @ Vector((0, 0, -0.03)), a, b, 0.1, n, 1.25, 0.95)
        hem = len(B.v)
        for k in range(n):
            t = math.tau * k / n
            drop = 0.14 if k % 2 == 0 else 0.105             # Picos en el borde
            B.v.append(pel + Rp @ Vector((0.14 * math.cos(t) * 1.15, 0.14 * math.sin(t), -drop)))
        for r0, r1 in ((top, mid), (mid, hem)):
            for k in range(n):
                k1 = (k + 1) % n
                B.add((r0 + k, r0 + k1, r1 + k1, r1 + k), BLACK)

    # Cuello y cabeza (cara casi sin detalle: solo dos ojitos verdes)
    if SB:
        SB.chain([chest + Rt @ Vector((0, 0, 0.02)), neck + Rh @ Vector((0, 0, 0.03))], [0.026, 0.022], SKIN,
                 start=(chest + Rt @ Vector((0, 0, 0.04)), Rt @ Vector((0, 0, 1))))   # Escote derecho
        SB.ellipsoid(head, Rh, (0.058 * HS, 0.064 * HS, 0.075 * HS), SKIN)
    else:
        B.tube([neck - Rt @ Vector((0, 0, 0.04)), neck + Rh @ Vector((0, 0, 0.03))], [0.024, 0.022], 8, SKIN)
        B.ellipsoid(head, Rh, (0.058 * HS, 0.064 * HS, 0.075 * HS), SKIN, seg=10, rings=7)
    face = lambda x, z: head + Rh @ Vector((x, -0.064 * HS * (1 - (x / (0.058 * HS)) ** 2 - (z / (0.075 * HS)) ** 2) ** 0.5 - 0.002, z))   # noqa: E731
    if beads:
        for s in (-1, 1):                                # Ojos de cuenta: bolitas negras que asoman de la cara
            eye_c = face(s * 0.03 * HS, 0.012 * HS)
            eye_start = len(B.v)
            eye_shape = ((0.0095 * HS, 0.006 * HS, 0.0095 * HS) if ride_detail
                         else (0.0075 * HS, 0.006 * HS, 0.0085 * HS))
            B.ellipsoid(eye_c, Rh, eye_shape, bead_mat or BEAD,
                        seg=10, rings=6)
            if ride_detail:
                # Lower each eye's inner top corner toward the nose. The eye itself
                # supplies the angry diagonal; there is no separate eyebrow mesh.
                inv = Rh.inverted()
                for i in range(eye_start, len(B.v)):
                    local = inv @ (B.v[i] - eye_c)
                    local.z = min(local.z, 0.001 * HS + 0.85 * s * local.x)
                    B.v[i] = eye_c + Rh @ local
            if bead_mat:                                 # Brillito arriba al costado: ojo brillante
                glint = (eye_c + Rh @ Vector((s * 0.002 * HS, -0.0055 * HS, -0.001 * HS))
                         if ride_detail else face(s * 0.03 * HS - 0.0025 * HS, 0.0155 * HS)
                         + Rh @ Vector((0, -0.0045 * HS, 0)))
                B.ellipsoid(glint, Rh,
                            (0.0022 * HS, 0.0012 * HS, 0.0022 * HS), SHINE, seg=6, rings=4)
    if blush:
        for s in (-1, 1):                                # Círculos rosados chatos, pegados a los cachetes
            B.ellipsoid(face(s * 0.038 * HS, -0.012 * HS), Rh @ Matrix.Rotation(math.radians(s * 32), 3, "Z"),
                        (0.012 * HS, 0.0025 * HS, 0.01 * HS), BLUSH, seg=12, rings=5)
    if nose:                                             # Naricita en punta, apenas caída
        n0 = face(0, -0.004 * HS)
        B.tube([n0 + Rh @ Vector((0, 0.006, 0.004)), n0 + Rh @ Vector((0, -0.014, -0.004)) * HS,
                n0 + Rh @ Vector((0, -0.026, -0.012)) * HS], [0.011 * HS, 0.007 * HS, 0.0015], 8, SKIN, cap1=True)
    if happy and not beads:
        for s in (-1, 1):                                # Ojos cerrados de felicidad: arcos ∩ brillantes
            arc = [face(s * 0.024 * HS + t * 0.011 * HS, (0.012 + 0.008 * (1 - t * t)) * HS) for t in (-1.0, -0.5, 0.0, 0.5, 1.0)]
            B.tube(arc, [0.0028 * HS] * 5, 4, EYES)
    if happy:
        smile = [face(t * 0.03 * HS, (-0.022 - 0.012 * (1 - t * t)) * HS) for t in (-1.0, -0.6, -0.2, 0.2, 0.6, 1.0)]
        B.tube(smile, [0.003 * HS] * 6, 4, BLACK)        # Sonrisa grande (U)
    if not happy and not beads:
        for s in (-1, 1):
            B.ellipsoid(head + Rh @ Vector((s * 0.024, -0.052, 0.012)) * HS, Rh, (0.009 * HS, 0.006 * HS, 0.006 * HS), EYES,
                        seg=5, rings=3)

    # Pelo: un solo panel. Arriba envuelve toda la cabeza hasta la coronilla (sin huecos de cuero cabelludo),
    # enmarca la cara a los lados y cae ondulando por la espalda con el borde en ondas. Va con las caras para los
    # dos lados, así se ve de frente y de atrás. Con flowy_hair el panel solo cubre la cabeza y la nuca, y lo que
    # cae son mechones en dos capas (ver más abajo).
    def hair_pt(u, v, ph=0.0, curl=1.0, rs=1.0, v_end=1.0, out=0.0):
        """Punto del pelo: u de un costado de la cara al otro (pasando por la nuca), v de la coronilla hacia abajo."""
        reach = 165 - 25 * min(max((v + 0.25) / 0.25, 0), 1) - 30 * min(max((v - 0.05) / 0.25, 0), 1)
        if v < 0:
            r, z = 0.066 + 0.07 * v, 0.03 - 0.18 * v
        else:
            r, z = 0.066 + 0.05 * v, 0.03 - 0.36 * v
        r *= rs
        if ride_detail and v > 0:
            r += 0.027 * min(v / 0.7, 1.0)  # Fuller hair mass below the crown, clear of the face.
        phi = math.radians(-reach + 2 * reach * u)
        vv = max(v, 0)
        wave = 0.012 * math.sin(3 * vv + 5 * u) * vv
        back, lift = 0.0, 0.0
        if flowy_hair and vv > 0:                        # Ondas grandes que se mueven y el viento hacia atrás
            wave = 0.03 * math.sin(2.6 * vv + 4.5 * u + ph) * vv + 0.012 * math.sin(7 * vv - 3 * u + ph) * vv
            back = 0.11 * vv ** 1.5 + 0.02 * math.sin(3 * vv + 6 * u + ph) * vv
            lift = 0.075 * vv ** 1.4                     # Cae hacia atrás en diagonal, no derecho para abajo
            ct = min(max((vv - 0.7 * v_end) / (0.3 * v_end), 0.0), 1.0) * curl   # Puntas enroscadas hacia afuera
            back += 0.07 * math.sin(ct * math.pi * 0.9)
            lift += 0.11 * (1 - math.cos(ct * math.pi * 0.9))
        return head + Rh @ (Vector((r * math.sin(phi) + wave, r * math.cos(phi) * 0.9 + 0.03 * vv + 0.05 * vv * vv
                                    + back + out * vv, z + lift)) * HS)

    def hair_sheet(pts_grid):
        """Lámina de pelo con cara de adelante y de atrás (vértices propios cada una)."""
        rows_, cols_ = len(pts_grid), len(pts_grid[0])
        front = len(B.v)
        B.v.extend(p for row in pts_grid for p in row)
        backv = len(B.v)
        if ride_detail and not flowy_hair:
            for row in pts_grid:
                for i, p in enumerate(row):
                    phi = math.radians(-110 + 220 * i / (cols_ - 1))
                    inward = Rh @ Vector((math.sin(phi), math.cos(phi) * 0.9, 0))
                    B.v.append(p - inward * (0.014 * HS))
        else:
            B.v.extend(p.copy() for row in pts_grid for p in row)
        for j in range(rows_ - 1):
            for i in range(cols_ - 1):
                q = (j * cols_ + i, j * cols_ + i + 1, (j + 1) * cols_ + i + 1, (j + 1) * cols_ + i)
                B.add(tuple(front + k for k in q), hair_m)
                B.add(tuple(backv + k for k in q[::-1]), hair_m)
        if ride_detail and not flowy_hair:
            # Close the edge so the hair reads as a thick sheet from the side and back.
            for j in range(rows_ - 1):
                for i in (0, cols_ - 1):
                    a, b = j * cols_ + i, (j + 1) * cols_ + i
                    B.add((front + a, front + b, backv + b, backv + a), hair_m)
            for i in range(cols_ - 1):
                a, b = (rows_ - 1) * cols_ + i, (rows_ - 1) * cols_ + i + 1
                B.add((front + a, front + b, backv + b, backv + a), hair_m)

    cols, rows = (61, 18) if smooth else (13, 10)       # De cerca: más columnas y filas, para que el borde ondule suave
    v_end = 0.12 if flowy_hair else 1.0                  # Con mechones, el panel termina en la nuca
    grid = []
    for j in range(rows):
        v = -0.25 + (v_end + 0.25) * j / (rows - 1)      # v < 0: la parte que cubre la cabeza
        row = []
        for i in range(cols):
            u = i / (cols - 1)
            p = hair_pt(u, v, out=0.05 if smooth else 0.0)   # El cuerpo de una pieza es más ancho atrás
            if smooth and not flowy_hair and j >= rows - 8:  # Borde en ondas parejas y redondas (como olas): la
                fade = (j - (rows - 8)) / 7                      # onda entra de a poco en las últimas filas
                p = p + Rh @ Vector((0, 0, 0.03 * fade * math.sin(u * math.tau * 4.5))) * HS
            elif j == rows - 1 and not flowy_hair:
                p = p + Rh @ Vector((0, 0, 0.025 * math.sin(u * math.tau * 2))) * HS   # Borde en ondas
            row.append(p)
        grid.append(row)
    hair_sheet(grid)
    if flowy_hair:
        # Capas, como un corte en capas: paneles enteros uno sobre otro desde la raíz; la de arriba es la más corta
        # y las de abajo, cada vez más largas, asoman por debajo. Cada una con su onda y su rulo en la punta
        for rs, vend, ph, curl in ((1.0, 0.8, 0.0, 0.35), (0.975, 1.1, 1.3, 0.5), (0.95, 1.4, 2.6, 1.0)):
            layer = []
            for j in range(14):
                v = 0.0 + vend * j / 13
                layer.append([hair_pt(i / 14, v, ph, curl, rs, vend)
                              + (Rh @ Vector((0, 0, 0.012 * math.sin((i / 14) * math.tau * 2.5 + ph))) * HS
                                 if j == 13 else Vector()) for i in range(15)])   # Borde en ondas
            hair_sheet(layer)

    if ride_detail:
        # Flequillo: una lámina de pelo pegada a la frente, de sien a sien, bajo el ala del sombrero, con el borde
        # de abajo en mechones redondeados que llegan justo arriba de los ojos (un poco más largo a los costados)
        def brow_pt(x, z, out):
            k = max(0.06, 1 - (x / (0.06 * HS)) ** 2 - (z / (0.077 * HS)) ** 2)
            return head + Rh @ Vector((x * 1.04, -0.066 * HS * k ** 0.5 - out, z))
        bc, br = 61, 9
        fr = []
        for j in range(br):
            row = []
            for i in range(bc):
                u = i / (bc - 1) * 2 - 1                     # -1 a 1, de sien a sien
                x = u * 0.058 * HS
                # Tres mechones grandes y redondeados: cada uno baja en una curva redonda y entre mechón y mechón
                # sube en una muesca chica; a los costados bajan más, enmarcando la cara
                ph = (u + 1) / 2 * 3                          # 0..3: en qué mechón está
                f = ph - math.floor(ph)                       # 0..1 dentro del mechón
                bulge = math.sqrt(max(0.0, 1 - (2 * f - 1) ** 2))   # Medio círculo: punta redonda abajo
                side_drop = max(0.0, (abs(u) - 0.62) / 0.38) ** 1.5
                fringe = 0.034 - 0.013 * bulge - 0.042 * side_drop
                z = (0.064 - (0.064 - fringe) * j / (br - 1)) * HS
                row.append(brow_pt(x, z, 0.004 * HS * (1 + 0.8 * j / (br - 1))))
            fr.append(row)
        f0 = len(B.v)
        B.v.extend(p for row in fr for p in row)
        f1 = len(B.v)
        B.v.extend(p + Rh @ Vector((0, 0.003 * HS, 0)) for row in fr for p in row)   # Cara de atrás, hacia la frente
        for j in range(br - 1):
            for i in range(bc - 1):
                q = (j * bc + i, j * bc + i + 1, (j + 1) * bc + i + 1, (j + 1) * bc + i)
                B.add(tuple(f0 + k for k in q[::-1]), hair_m)
                B.add(tuple(f1 + k for k in q), hair_m)
        for i in range(bc - 1):                              # Borde de abajo, con grosor
            a_, b_ = (br - 1) * bc + i, (br - 1) * bc + i + 1
            B.add((f0 + a_, f0 + b_, f1 + b_, f1 + a_), hair_m)

    # Sombrero: ala ancha (con grosor), cinta y cono con la punta doblada hacia atrás
    hb = head + Rh @ Vector((0, 0.005, 0.05)) * HS
    hr = Rh @ rx(-8)
    up, ha, hbv = hr @ Vector((0, 0, 1)), hr @ Vector((1, 0, 0)), hr @ Vector((0, 1, 0))
    seg = 20
    outer_t = B.ring(hb + up * 0.004, ha, hbv, 0.15, seg)
    outer_b = B.ring(hb - up * 0.004, ha, hbv, 0.15, seg)
    inner_t = B.ring(hb + up * 0.004, ha, hbv, 0.066, seg)
    inner_b = B.ring(hb - up * 0.004, ha, hbv, 0.066, seg)
    for k in range(seg):
        k1 = (k + 1) % seg
        B.add((inner_t + k, inner_t + k1, outer_t + k1, outer_t + k), BLACK)
        B.add((outer_b + k, outer_b + k1, inner_b + k1, inner_b + k), BLACK)
        B.add((outer_t + k, outer_t + k1, outer_b + k1, outer_b + k), BLACK)
    for k in range(seg):                                 # El ala cae un poco hacia los lados
        t = math.tau * k / seg
        for idx in (outer_t, outer_b):
            p = B.v[idx + k] - up * 0.018 * abs(math.cos(t))
            if ride_detail:
                p += (p - hb).normalized() * (0.006 * math.sin(5 * t + 0.4))
                p += up * (0.005 * math.sin(3 * t - 0.5))
            B.v[idx + k] = p
    cone = [hb + up * 0.004, hb + up * 0.07, hb + up * 0.135 + hbv * 0.015, hb + up * 0.185 + hbv * 0.05,
            hb + up * 0.21 + hbv * 0.1, hb + up * 0.2 + hbv * 0.15, hb + up * 0.17 + hbv * 0.18 - ha * 0.01]
    B.tube(cone, [0.066, 0.05, 0.036, 0.024, 0.015, 0.008, 0.002], 12, BLACK, cap1=True, ref=hbv)
    B.tube([hb + up * 0.006, hb + up * 0.026], [0.068, 0.062], 12, BAND, ref=hbv)
    if ride_detail:
        band_trim = [hb + up * 0.024 + (ha * math.cos(math.tau * i / 32)
                    + hbv * math.sin(math.tau * i / 32)) * 0.063 for i in range(33)]
        B.tube(band_trim, [0.002] * len(band_trim), 5, BAND_EDGE)

    # Brazos (piel) con manos pequeñas
    for side, key in ((1, "arm_R"), (-1, "arm_L")):
        abd, flex, elbow, twist = pose[key]
        sh = chest + Rt @ Vector((side * 0.085, 0, 0.045))
        Ru = Rt @ ry(-side * abd) @ rx(flex) @ rz(side * twist)
        el = sh + Ru @ Vector((0, 0, -0.165))
        Rf = Ru @ rx(elbow)
        wr = el + Rf @ Vector((0, 0, -0.155))
        arm = [sh - Rt @ Vector((side * 0.02, 0, 0)), sh, sh.lerp(el, 0.5), el, el.lerp(wr, 0.5), wr]
        if SB:
            # Sisa: cruzada al brazo, apenas pasado el hombro (el brazo de la escoba va un poco separado del cuerpo,
            # así nada del costado queda pegado al brazo y el borde sale derecho)
            SB.chain(arm, [0.024, 0.024, 0.021, 0.016, 0.015, 0.011], SKIN,
                     start=(sh + (el - sh).normalized() * 0.01, el - sh), claim=1.0)
            SB.ellipsoid(wr + Rf @ Vector((0, 0, -0.036)), Rf, (0.016, 0.023, 0.04), SKIN)   # Mano más grande
            tb = wr + Rf @ Vector((0, -0.014, -0.018))  # Pulgar: sale del costado de la palma, hacia adelante
            SB.chain([tb, tb + Rf @ Vector((0, -0.022, -0.016)), tb + Rf @ Vector((0, -0.028, -0.04))],
                     [0.0105, 0.0095, 0.008], SKIN)
        else:
            B.tube(arm, [0.024, 0.024, 0.021, 0.016, 0.015, 0.011], 8, SKIN, cap0=True)
            B.ellipsoid(wr + Rf @ Vector((0, 0, -0.03)), Rf, (0.012, 0.018, 0.032), SKIN, seg=8, rings=5)
        B.hands.append(wr + Rf @ Vector((0, 0, -0.062)))

    # Piernas (piel) y tacones de aguja negros
    for side, key in ((1, "leg_R"), (-1, "leg_L")):
        flex, abd, knee = pose[key]
        hip = pel + Rp @ Vector((side * 0.05, 0, -0.02))
        Rth = Rp @ ry(-side * abd) @ rx(flex)
        kn = hip + Rth @ Vector((0, 0, -THIGH))
        Rs = Rth @ rx(knee)
        an = kn + Rs @ Vector((0, 0, -SHIN))
        calf = kn + Rs @ Vector((0, 0.008, -0.09))
        # Medias a rayas blancas y negras, hasta arriba de la rodilla (más arriba, piel): la pierna se parte en
        # tramos de una raya de ancho, alternando el color; arriba empiezan con una raya negra
        knots = [hip + Rp @ Vector((0, 0, 0.04)), hip, hip.lerp(kn, 0.45), kn, calf, kn.lerp(an, 0.75), an]
        krad = [0.045, 0.048, 0.038, 0.025, 0.028, 0.018, 0.015]
        pts, rads = [knots[0]], [krad[0]]
        for i in range(len(knots) - 1):
            n_ = max(1, round((knots[i + 1] - knots[i]).length / STRIPE))
            for j in range(1, n_ + 1):
                pts.append(knots[i].lerp(knots[i + 1], j / n_))
                rads.append(krad[i] + (krad[i + 1] - krad[i]) * j / n_)
        top = hip.lerp(kn, 0.72)                          # Borde de la media, arriba de la rodilla
        mats, k = [], 0
        for i in range(len(pts) - 1):
            mid = pts[i].lerp(pts[i + 1], 0.5)
            if (mid - hip).length < (top - hip).length and (mid - hip).dot(kn - hip) > 0 or i == 0:
                mats.append(SKIN)
            else:
                mats.append(WHITE if k % 2 else BLACK)
                k += 1
        if SB:
            # En la rodilla, una banda negra: el frente de la rodilla está igual de cerca del muslo que de la
            # canilla, y si ahí cambia el color el borde sale serruchado
            ki = min(range(len(pts)), key=lambda i: (pts[i] - kn).length)
            for i in range(max(1, ki - 1), min(len(mats), ki + 1)):
                mats[i] = BLACK
            first = [0]                                   # Desde qué tramo (lo de antes lo tapa el vestido)

            def stripe(c, pts=pts, mats=mats, first=first):   # El material del tramo de pierna más cercano
                best, bi = 9.0, first[0]
                for i in range(first[0], len(pts) - 1):
                    a, b = pts[i], pts[i + 1]
                    t = max(0.0, min(1.0, (c - a).dot(b - a) / max((b - a).length_squared, 1e-12)))
                    dd = (c - a.lerp(b, t)).length
                    if dd < best:
                        best, bi = dd, i
                return mats[bi]
            # El vestido baja por el muslo y termina en un borde derecho (con la falda larga, hasta la rodilla: le
            # tapa el regazo); de ahí para abajo, la pierna
            hem = hip.lerp(kn, 0.3)                      # Debajo de la falda
            SB.chain(pts, rads, stripe, start=(hem, kn - hip))
            for i in range(1, len(mats)):                # Cada borde de raya, derecho
                if mats[i] != mats[i - 1]:
                    SB.cut(pts[i], pts[i + 1] - pts[i - 1], rads[i] * 3)
        else:
            B.tube(pts, rads, 8, mats)
        # Botas negras de tacón: caña hasta media pierna, pie en punta y tacón de aguja
        Rfoot = Rs @ rx(-55)
        toe = an + Rfoot @ Vector((0, 0, -0.105))
        B.tube([kn.lerp(an, 0.45), kn.lerp(an, 0.75), an + Rs @ Vector((0, 0, 0.02))], [0.032, 0.025, 0.023], 10,
               BLACK)
        B.tube([an + Rs @ Vector((0, 0, 0.02)), an, an.lerp(toe, 0.6), toe], [0.023, 0.027, 0.022, 0.006], 10, BLACK,
               cap1=True)
        heel = an + Rs @ Vector((0, 0.016, -0.01))
        B.tube([heel, heel + Rs @ Vector((0, 0.008, -0.085))], [0.013, 0.004], 6, BLACK, cap1=True)

    if SB:
        SB.build(B)
    return B


def merge(parts):
    """Une varias brujas (cada una con su giro y su posición) en un solo Builder."""
    out = Builder(MATS)
    for b, rot, off in parts:
        base = len(out.v)
        out.v += [rot @ v + off for v in b.v]
        out.f += [tuple(i + base for i in f) for f in b.f]
        out.m += b.m
        out.hands += [rot @ h + off for h in b.hands]
    return out


def make(name, B, anchor, x):
    """Crea la malla. anchor="feet": el origen va en el suelo, bajo los tacones; anchor="hands": el origen va en las
    manos (para colgarla del borde). x solo separa los modelos en la presentación."""
    if anchor == "seat":                                 # El origen va bajo la cola: se sienta en el borde
        shift = B.seat.copy()
    elif anchor == "chest":                                # El borde va bajo el pecho: la cabeza queda afuera
        shift = B.chest.copy()
        shift.z = min(v.z for v in B.v)
    elif anchor == "feet":
        lo = min(v.z for v in B.v)
        shift = Vector((sum(v.x for v in B.v) / len(B.v), sum(v.y for v in B.v) / len(B.v), lo))
        shift.x = shift.y = 0.0
    else:
        shift = sum(B.hands, Vector()) / len(B.hands)
        shift.z = max(h.z for h in B.hands)
    me = make_mesh(name, B, MATS, shift)
    ob = bpy.data.objects.new(name, me)
    scene.collection.objects.link(ob)
    ob.parent = ROOT
    ob.location = (x, 0, 1.25 if anchor == "hands" else 0)
    me.calc_loop_triangles()
    print("TRIS %-18s %d" % (name, len(me.loop_triangles)))
    return ob


ROOT = bpy.data.objects.new("Halloween_Witch_ROOT", None)
scene.collection.objects.link(ROOT)
# Un modelo por pose; en el planeta se usan como copias que comparten la malla
make("HW_Witch_Solo_A", build_witch(POSES["solo_a"]), "feet", -1.5)
make("HW_Witch_Solo_B", build_witch(POSES["solo_b"]), "feet", -0.9)
make("HW_Witch_Solo_C", build_witch(POSES["solo_c"]), "feet", -2.2)
# Pareja: frente a frente y de la mano. La segunda usa la pose al revés (brazos y piernas cambiados de lado), no
# un espejo de la malla, que dejaría las caras al revés
pair_b = dict(POSES["pair"], arm_R=POSES["pair"]["arm_L"], arm_L=POSES["pair"]["arm_R"],
              leg_R=POSES["pair"]["leg_L"], leg_L=POSES["pair"]["leg_R"])
make("HW_Witch_Pair", merge([(build_witch(POSES["pair"]), rz(90), Vector((-0.27, 0, 0))),
                             (build_witch(pair_b), rz(-90), Vector((0.27, 0, 0)))]), "feet", -0.1)
make("HW_Witch_Crawl", build_witch(POSES["crawl"]), "feet", 0.7)
make("HW_Witch_Climb", build_witch(POSES["climb"]), "hands", 1.4)
make("HW_Witch_Peek", build_witch(POSES["peek"]), "chest", 2.2)
make("HW_Witch_Sit", build_witch(POSES["sit"]), "seat", 3.0)
# Sentada y feliz, saludando con los dos brazos: va sentada en las gotas de sangre que caen del mentón
make("HW_Witch_SitHappy", build_witch(dict(POSES["sit"], head=(-4, 0, 0), arm_R=(150, 20, -20, 0), arm_L=(140, -15, -30, 0)),
                                      happy=True), "seat", 3.7)
# Montada en la escoba, feliz: es la nave del planeta (build_witch_broom.py le pone la escoba). La mano que agarra el
# palo queda guardada en la malla (relativa al asiento) para apuntar la escoba hacia ella
BROOM_YAW = 14                                           # Grados que la escoba gira bajo ella: la punta hacia la mano que
#                                                          la agarra y la cola hacia el lado de las piernas (que cuelgan
#                                                          adelante); así, de atrás, el palo y la paja asoman al costado


def ride_fk(pose):
    """Asiento, mano que agarra el palo (relativa al asiento) y hacia qué lado (x) caen las piernas, sin armar la
    malla: las mismas cuentas que build_witch."""
    pel = Vector((0, 0, 0.5))
    Rp = rz(pose["pelvis"][2]) @ ry(pose["pelvis"][1]) @ rx(pose["pelvis"][0])
    Rt = Rp @ rz(pose["torso"][2]) @ ry(pose["torso"][1]) @ rx(pose["torso"][0])
    chest = pel + Rp @ Vector((0, 0, 0.07)) + Rt @ Vector((0, 0, 0.12))
    seat = pel + Rp @ Vector((0, 0.02, -0.06))
    abd, flex, elbow, twist = pose["arm_R"]
    sh = chest + Rt @ Vector((0.085, 0, 0.045))
    Ru = Rt @ ry(-abd) @ rx(flex) @ rz(twist)
    Rf = Ru @ rx(elbow)
    hand = sh + Ru @ Vector((0, 0, -0.165)) + Rf @ Vector((0, 0, -0.155 - 0.062))
    legs_x = sum((Rp @ ry(-sd * pose[k][1]) @ rx(pose[k][0]) @ Vector((0, 0, -1))).x
                 for sd, k in ((1, "leg_R"), (-1, "leg_L")))
    return seat, hand - seat, legs_x


# La escoba se gira BROOM_YAW bajo ella (ella sigue mirando para adelante) y el brazo que agarra el palo se acomoda
# para que la mano quede sobre el palo girado: se busca la pose del brazo más parecida a la original que lo logra
ride_pose = dict(POSES["ride"])
RIDE_KW = dict(smooth=True, nose=True, beads=True, long_skirt=True, blush=False, hair_mat=HAIR_JET,
               bead_mat=BEAD_RED, ride_detail=True)
# Primero, dónde termina la cola: el palo va justo debajo (apoyada encima, sin atravesarla)
probe = build_witch(ride_pose, **RIDE_KW)
bum = min(v.z for v in probe.v if (v.x - probe.seat.x) ** 2 + (v.y - probe.seat.y) ** 2 < 0.045 ** 2
          and v.z < probe.seat.z) - probe.seat.z
stick_at = Vector((0, 0, bum - 0.022))                   # Centro del palo bajo el asiento (radio del palo: 0.021)
_, g0, legs_x = ride_fk(ride_pose)
d0 = Vector((0, g0.y, g0.z)).normalized()                # Dirección del palo sin girar (pasa por la mano)
broom_dir = rz(math.copysign(BROOM_YAW, -legs_x)) @ d0
a0 = ride_pose["arm_R"]
best = None
for abd in range(a0[0] - 30, a0[0] + 31, 3):
    for flex in range(a0[1] - 30, a0[1] + 31, 3):
        for elbow in range(a0[2] - 30, a0[2] + 31, 5):
            _, g, _ = ride_fk(dict(ride_pose, arm_R=(abd, flex, elbow, a0[3])))
            rel = g - stick_at
            miss = (rel - broom_dir * rel.dot(broom_dir)).length   # Qué tan lejos queda la mano del palo
            cost = miss * 50 + 0.0004 * ((abd - a0[0]) ** 2 + (flex - a0[1]) ** 2 + (elbow - a0[2]) ** 2) ** 0.5
            if best is None or cost < best[0]:
                best = (cost, (abd, flex, elbow, a0[3]), miss)
ride_pose["arm_R"] = best[1]
print("RIDE_ARM", best[1], "miss %.4f" % best[2])
ride = build_witch(ride_pose, **RIDE_KW)                # Sin boca: ojos de cuenta rojos y cachetes
ride_ob = make("HW_Witch_Ride", ride, "seat", 4.5)
ride_ob.data["skirt_range"] = ride.skirt_range
ride_ob.data["grip"] = tuple(ride.hands[0] - ride.seat)
ride_ob.data["broom_dir"] = tuple(broom_dir)              # build_witch_broom.py pone el palo en esta dirección
ride_ob.data["stick_at"] = tuple(stick_at)                # ...pasando por acá (justo bajo la cola)

# --- Presentación ------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("Witch_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#0d0b12"), 1)
scene.world = world
for name, rot, col, e in (("Key", (math.radians(55), 0, math.radians(-35)), "#ffe2c8", 5.0),
                          ("Rim", (math.radians(70), 0, math.radians(160)), "#8fa0ff", 2.5),
                          ("Under", (math.radians(160), 0, math.radians(10)), "#9be04a", 1.0)):
    ld = bpy.data.lights.new(name, "SUN")
    ld.energy, ld.color = e, srgb(col)
    lo_ = bpy.data.objects.new(name, ld)
    pres.objects.link(lo_)
    lo_.rotation_euler = rot
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1600, 900
scene.view_settings.view_transform = "Standard"
for screen in bpy.data.screens:
    for area in screen.areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.color_type = "MATERIAL"

os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if not os.environ.get("HW_NO_RENDER"):
    for name, loc in (("front", (0.0, -8.0, 0.75)), ("three_quarter", (-4.8, -6.2, 1.3)), ("back", (0.5, 8.0, 0.9))):
        cam.location = loc
        cam.rotation_euler = (Vector((0.0, 0, 0.62)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = os.path.join(REF_DIR, "witches_%s.png" % name)
        bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
