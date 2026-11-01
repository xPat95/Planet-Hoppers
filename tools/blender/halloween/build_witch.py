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
from figure import Builder, make_mesh, mat, rx, ry, rz, srgb  # noqa: E402  (piezas compartidas con el zombie)

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
MATS = [SKIN, BLACK, HAIR, BAND, EYES, WHITE]


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
    # Colgada del borde de la órbita: brazos estirados arriba y piernas colgando, una pateando
    "climb": {
        "pelvis": (-6, 0, 0), "torso": (-4, 0, 0), "head": (-12, 0, 0),
        "leg_R": (-30, 6, 50), "leg_L": (8, -4, 12),
        "arm_R": (172, 0, -8, 0), "arm_L": (172, 0, -8, 0),
    },
}


def build_witch(pose):
    B = Builder(MATS)
    pel = Vector((0, 0, 0.5))
    Rp = rz(pose["pelvis"][2]) @ ry(pose["pelvis"][1]) @ rx(pose["pelvis"][0])
    Rt = Rp @ rz(pose["torso"][2]) @ ry(pose["torso"][1]) @ rx(pose["torso"][0])
    Rh = Rt @ rz(pose["head"][2]) @ ry(pose["head"][1]) @ rx(pose["head"][0])
    waist = pel + Rp @ Vector((0, 0, 0.07))
    chest = waist + Rt @ Vector((0, 0, 0.12))
    neck = chest + Rt @ Vector((0, 0, 0.07))
    head = neck + Rh @ Vector((0, 0, 0.085 * HS))
    B.chest = chest.copy()
    B.seat = pel + Rp @ Vector((0, 0.02, -0.06))

    # Vestido: torso ajustado (más ancho que hondo) y falda corta con el borde en picos
    B.tube([pel + Rp @ Vector((0, 0, -0.02)), waist, chest - Rt @ Vector((0, 0, 0.03)), chest,
            neck - Rt @ Vector((0, 0, 0.035))],
           [0.08, 0.05, 0.066, 0.07, 0.056], 10, BLACK, sx=1.3, sy=0.9, ref=Rt @ Vector((0, -1, 0)), cap1=True)
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
    B.tube([neck - Rt @ Vector((0, 0, 0.04)), neck + Rh @ Vector((0, 0, 0.03))], [0.024, 0.022], 8, SKIN)
    B.ellipsoid(head, Rh, (0.058 * HS, 0.064 * HS, 0.075 * HS), SKIN, seg=10, rings=7)
    for s in (-1, 1):
        B.ellipsoid(head + Rh @ Vector((s * 0.024, -0.052, 0.012)) * HS, Rh, (0.009 * HS, 0.006 * HS, 0.006 * HS), EYES,
                    seg=5, rings=3)

    # Pelo: un solo panel. Arriba envuelve toda la cabeza hasta la coronilla (sin huecos de cuero cabelludo),
    # enmarca la cara a los lados y cae ondulando por la espalda con el borde en ondas. Va con las caras para los
    # dos lados, así se ve de frente y de atrás.
    cols, rows = 13, 10
    grid = []
    for j in range(rows):
        v = -0.25 + 1.25 * j / (rows - 1)                # v < 0: la parte que cubre la cabeza
        reach = 165 - 25 * min(max((v + 0.25) / 0.25, 0), 1) - 30 * min(max((v - 0.05) / 0.25, 0), 1)
        if v < 0:
            r, z = 0.066 + 0.07 * v, 0.03 - 0.18 * v
        else:
            r, z = 0.066 + 0.05 * v, 0.03 - 0.36 * v
        row = []
        for i in range(cols):
            u = i / (cols - 1)
            phi = math.radians(-reach + 2 * reach * u)
            vv = max(v, 0)
            wave = 0.012 * math.sin(3 * vv + 5 * u) * vv
            hem = 0.025 * math.sin(u * math.tau * 2) if j == rows - 1 else 0.0
            row.append(len(B.v))
            B.v.append(head + Rh @ (Vector((r * math.sin(phi) + wave, r * math.cos(phi) * 0.9 + 0.03 * vv + 0.05 * vv * vv,
                                             z + hem)) * HS))
        grid.append(row)
    back = len(B.v)                                      # Cara de atrás con sus propios vértices (si no, se
    B.v.extend([B.v[k].copy() for row in grid for k in row])   # toman por caras repetidas y se borran)
    for j in range(rows - 1):
        for i in range(cols - 1):
            q = (grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i])
            B.add(q, HAIR)
            B.add(tuple(back + (k - grid[0][0]) for k in q[::-1]), HAIR)

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
            B.v[idx + k] = B.v[idx + k] - up * 0.018 * abs(math.cos(t))
    cone = [hb + up * 0.004, hb + up * 0.07, hb + up * 0.135 + hbv * 0.015, hb + up * 0.185 + hbv * 0.05,
            hb + up * 0.21 + hbv * 0.1, hb + up * 0.2 + hbv * 0.15, hb + up * 0.17 + hbv * 0.18 - ha * 0.01]
    B.tube(cone, [0.066, 0.05, 0.036, 0.024, 0.015, 0.008, 0.002], 12, BLACK, cap1=True, ref=hbv)
    B.tube([hb + up * 0.006, hb + up * 0.026], [0.068, 0.062], 12, BAND, ref=hbv)

    # Brazos (piel) con manos pequeñas
    for side, key in ((1, "arm_R"), (-1, "arm_L")):
        abd, flex, elbow, twist = pose[key]
        sh = chest + Rt @ Vector((side * 0.085, 0, 0.045))
        Ru = Rt @ ry(-side * abd) @ rx(flex) @ rz(side * twist)
        el = sh + Ru @ Vector((0, 0, -0.165))
        Rf = Ru @ rx(elbow)
        wr = el + Rf @ Vector((0, 0, -0.155))
        B.tube([sh - Rt @ Vector((side * 0.02, 0, 0)), sh, sh.lerp(el, 0.5), el, el.lerp(wr, 0.5), wr],
               [0.024, 0.024, 0.021, 0.016, 0.015, 0.011], 8, SKIN, cap0=True)
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
