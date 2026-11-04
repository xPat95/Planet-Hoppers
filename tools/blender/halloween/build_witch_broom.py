"""Nave del planeta Halloween: una de las brujas montada en una escoba.

La bruja es HW_Witch_Ride de witch.blend (build_witch.py se ejecuta antes): a horcajadas, inclinada hacia adelante,
con una mano en el palo y la otra saludando. Acá se le agrega la escoba (palo de madera, atado y paja en
mechones) y, detrás de la paja, una estela suelta y tupida: estrellas infladas rojas con el borde naranja que brilla, de
tamaños distintos, hilos de humo de luz naranja que serpentean y chispas, que se abren hacia atrás.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/halloween/build_witch_broom.py

Guarda assets/models/ships/halloween/witch_broom.blend y las capturas en docs/references/halloween/broom_*.png (con
HW_NO_RENDER=1 no renderiza). Después: exportar con export_solar_models.py -- halloween_ship.
"""

import bpy
import math
import os
import random
import sys
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from figure import Builder, make_mesh, mat, srgb  # noqa: E402

REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
WITCH_BLEND = os.path.join(REPO, "assets", "models", "characters", "halloween", "witch.blend")
BLEND_OUT = os.path.join(REPO, "assets", "models", "ships", "halloween", "witch_broom.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "halloween")

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "Halloween_Witch_Broom"

WOOD = mat("BroomWood", "#6b4423")
WOOD_DARK = mat("BroomWoodDark", "#4a2d17")
STRAW = mat("BroomStraw", "#c9a24a")
STRAW_DARK = mat("BroomStrawDark", "#a07d32")
TIE = mat("BroomTie", "#4a2a5a")                         # El atado, del mismo morado que la cinta del sombrero
TIE_WRAP = mat("BroomTieWrap", "#79528a")
STAR_DARK = mat("BroomStarDark", "#b0101e", emit=0.35, rough=0.25)      # Estrellas: rojo sangre...
STAR_DARKER = mat("BroomStarDarker", "#6e0812", emit=0.25, rough=0.25)    # ...y rojo oscuro
STAR_RIM = mat("BroomStarRim", "#ff7a1a", emit=4.0)                       # ...con el borde naranja que brilla
EMBER = mat("BroomEmber", "#ffb03a", emit=5.0)                            # Chispas: naranja amarillento
GLITTER = mat("BroomGlitter", "#ffe45c", emit=6.0)                        # Brillitos amarillos sobre los hilos
MATS = [WOOD, WOOD_DARK, STRAW, STRAW_DARK, TIE, TIE_WRAP, STAR_DARK, STAR_DARKER, STAR_RIM, EMBER, GLITTER]

ROOT = bpy.data.objects.new("Halloween_Ship_ROOT", None)
scene.collection.objects.link(ROOT)
ship = bpy.data.collections.new("Ship")
scene.collection.children.link(ship)

# --- La bruja ----------------------------------------------------------------------------------------------------
# Origen en el asiento; mira hacia -Y (en vuelo se gira 180° con config/ships.json)
with bpy.data.libraries.load(WITCH_BLEND) as (src, dst):
    dst.meshes = ["HW_Witch_Ride"]
witch_me = dst.meshes[0]
witch = bpy.data.objects.new("HW_Witch_Ride", witch_me)
ship.objects.link(witch)
witch.parent = ROOT
grip = Vector(witch_me["grip"])                          # La mano que agarra el palo, relativa al asiento

# --- La escoba ---------------------------------------------------------------------------------------------------
# El palo pasa justo debajo del asiento y por la mano que lo agarra; la paja va atrás
seat = Vector(witch_me["stick_at"])                     # Justo bajo la cola (ver build_witch.py)
fwd = Vector(witch_me["broom_dir"]).normalized()        # Girada un poco bajo ella (ver BROOM_YAW en build_witch.py)
B = Builder(MATS)
tip = seat + fwd * 0.78
tail = seat - fwd * 0.42
B.tube([tail, seat, seat + fwd * 0.4, tip], [0.02, 0.021, 0.019, 0.016], 10, WOOD, cap0=True, cap1=True)
B.ellipsoid(tip, Matrix.Identity(3), (0.02, 0.02, 0.02), WOOD_DARK, seg=8, rings=5)
side = Vector((1, 0, 0))
up = side.cross(fwd).normalized() * -1 if side.cross(fwd).z < 0 else side.cross(fwd).normalized()
# Dense, layered straw with softly swept tips. All strands start behind the skirt.
B.tube([tail + fwd * 0.03, tail - fwd * 0.03], [0.034, 0.034], 12, TIE)
wrap = [tail + fwd * (0.028 - 0.056 * i / 40)
        + (side * math.cos(math.tau * 2.2 * i / 40) + up * math.sin(math.tau * 2.2 * i / 40)) * 0.036
        for i in range(41)]
B.tube(wrap, [0.0025] * len(wrap), 5, TIE_WRAP)
STRANDS = 240
for k in range(STRANDS):
    layer = k % 4
    a = math.tau * (k * 0.61803398875 % 1.0)
    off = side * math.cos(a) + up * math.sin(a)
    length = 0.39 + 0.035 * layer + 0.035 * math.sin(k * 2.37)
    root_radius = 0.009 + 0.005 * layer
    spread = 0.042 + 0.024 * layer
    sweep = side * (0.018 * math.sin(k * 1.9)) - up * 0.014
    path, radii = [], []
    for step in range(9):
        t = step / 8
        # Each strand flares smoothly and levels out at its tip; no sharp return bend.
        flare = spread * (1 - (1 - t) ** 2)
        path.append(tail - fwd * length * t + off * (root_radius + flare) + sweep * t * t)
        radii.append(0.002 + 0.008 * (1 - t ** 3))
    B.tube(path, radii, 5, STRAW if k % 3 else STRAW_DARK, cap1=True)
broom = bpy.data.objects.new("HW_Broom", make_mesh("HW_Broom", B, MATS))

# La falda no atraviesa el palo: adelante y detrás del asiento, la tela que está cerca del palo se levanta y queda
# colgando encima, como una carpa sobre una barra (más alta justo arriba del palo, cayendo hacia los costados)
skirt_start, skirt_end = witch_me["skirt_range"]
# La tela de afuera (filas 4..11) y la de adentro (filas 16..23, va 0.004 por debajo para no cruzarse con la de afuera)
skirt_v = [(i, 0.0) for i in range(skirt_start + 4 * 64, skirt_start + 12 * 64)] +           [(i, 0.004) for i in range(skirt_start + 16 * 64, skirt_start + 24 * 64)]
ROD_R, ROD_W = 0.034, 0.15                               # Radio del palo con la tela, y ancho de la carpa (ancha:
#                                                          redondeada, sin hacer una cresta en punta)
for i, under in skirt_v:
    v = witch_me.vertices[i]
    rel = v.co - seat
    along = rel.dot(fwd)
    if abs(along) < 0.04:                                # Donde se sienta, no (ahí está ella)
        continue
    axis = seat + fwd * along
    lateral = (v.co - axis)
    lateral.z = 0.0
    dl = lateral.length
    if dl > ROD_W or v.co.z < axis.z - 0.12:
        continue
    top = axis.z + ROD_R * (1 - (dl / ROD_W) ** 2) - 0.03 * (dl / ROD_W) ** 2 - under
    if v.co.z < top:
        v.co.z = top
witch_me.update()
ship.objects.link(broom)
broom.parent = ROOT

# --- Estela: estrellas oscuras con borde verde, en espiral ----------------------------------------------------------
S = Builder(MATS)


def star(c, r, spin, n_axis, m, seg=30, rings=4):
    """Estrella inflada (gordita, puntas cortas y redondas) de frente hacia n_axis, con un borde verde que brilla."""
    a_ = Vector((1, 0, 0)) if abs(n_axis.x) < 0.9 else Vector((0, 1, 0))
    u = (a_ - n_axis * a_.dot(n_axis)).normalized()
    v = n_axis.cross(u)

    def edge(ang):
        return r * (0.6 + 0.4 * (1 + math.cos(5 * (ang - spin))) / 2)
    base = len(S.v)
    S.v += [c + n_axis * r * 0.5, c - n_axis * r * 0.5]
    for i in range(1, rings + 1):
        f = i / rings
        for sd in (1, -1):
            for k in range(seg):
                ang = math.tau * k / seg
                tip = (1 + math.cos(5 * (ang - spin))) / 2
                h = r * 0.5 * math.sqrt(max(0.0, 1 - f ** 2)) * (1 - 0.35 * tip * f)
                S.v.append(c + (u * math.cos(ang) + v * math.sin(ang)) * edge(ang) * f + n_axis * h * sd)

    def ring(i, sd):
        return base + 2 + ((i - 1) * 2 + (0 if sd > 0 else 1)) * seg

    for sd, center in ((1, base), (-1, base + 1)):
        r1 = ring(1, sd)
        for k in range(seg):
            k1 = (k + 1) % seg
            S.add((center, r1 + k, r1 + k1) if sd > 0 else (center, r1 + k1, r1 + k), m)
        for i in range(1, rings):
            a0, a1 = ring(i, sd), ring(i + 1, sd)
            for k in range(seg):
                k1 = (k + 1) % seg
                q = (a0 + k, a1 + k, a1 + k1, a0 + k1)
                S.add(q if sd > 0 else q[::-1], m)
    fa, fb = ring(rings, 1), ring(rings, -1)
    for k in range(seg):
        k1 = (k + 1) % seg
        S.add((fa + k, fb + k, fb + k1, fa + k1), m)
    # Borde que brilla: un tubito fino por el contorno
    loop = [c + (u * math.cos(a) + v * math.sin(a)) * edge(a) * 1.02
            for a in (math.tau * k / seg for k in range(seg + 1))]
    S.tube(loop, [r * 0.07] * len(loop), 6, STAR_RIM)


rng = random.Random(1031)                                # Semilla fija: misma estela siempre
start = tail - fwd * 0.5
TRAIL_LEN = 1.0


def path(t):
    """Centro de la estela: sale de la paja y serpentea suelto (sin espiral perfecta), subiendo apenas."""
    return (start - fwd * TRAIL_LEN * t
            + side * (0.07 * math.sin(t * 7.3 + 0.4) + 0.035 * math.sin(t * 15.1 + 1.7)) * (0.3 + t)
            + up * (0.06 * math.sin(t * 5.9 + 2.1) + 0.03 * math.sin(t * 12.7) + 0.05 * t) * (0.3 + t))


def scatter(t, spread):
    """Un punto al azar alrededor del centro: la nube se abre hacia atrás."""
    rad = spread * (0.35 + 0.9 * t) * math.sqrt(rng.random())
    ang = rng.uniform(0, math.tau)
    return path(t) + side * math.cos(ang) * rad + up * math.sin(ang) * rad * 0.8


# Hilos de humo de luz naranja: varios, de largos distintos, que serpentean y se afinan (no cintas parejas)
for w in range(5):
    t_end = rng.uniform(0.45, 1.0)
    ph1, ph2 = rng.uniform(0, math.tau), rng.uniform(0, math.tau)
    amp = rng.uniform(0.03, 0.08)
    f1, f2 = rng.uniform(5, 9), rng.uniform(5, 9)       # Cada hilo con su propia onda (una sola, suave)
    n = 50
    pts = []
    for k in range(n + 1):
        t = t_end * k / n
        pts.append(path(t) + side * amp * math.sin(t * f1 + ph1) * (0.4 + t)
                   + up * amp * math.cos(t * f2 + ph2) * (0.4 + t))
    r0 = rng.uniform(0.0025, 0.0045)
    S.tube(pts, [r0 * (1 - k / n) ** 0.7 + 0.0008 for k in range(n + 1)], 6, STAR_RIM, cap0=True, cap1=True)
    for _ in range(40):                                  # Brillitos amarillos pegados al hilo: se lee como brillo
        k = rng.randrange(n + 1)
        jit = Vector((rng.gauss(0, 1), rng.gauss(0, 1), rng.gauss(0, 1))) * 0.009 * (0.6 + k / n)
        S.ellipsoid(pts[k] + jit, Matrix.Identity(3), (rng.uniform(0.0018, 0.0045),) * 3, GLITTER, seg=5, rings=3)
# Estrellas: muchas, de tamaños distintos, amontonadas cerca de la paja y más sueltas hacia atrás
STARS = 60                                               # Muchas y chiquitas: brillo, no objetos
for i in range(STARS):
    t = (i / (STARS - 1)) ** 1.25
    c = scatter(t, 0.12)
    r = rng.uniform(0.011, 0.03) * (1 - 0.45 * t)
    face = (Vector((rng.uniform(-1, 1), rng.uniform(-0.6, 0.2), rng.uniform(-1, 1))) + up * 0.6).normalized()
    big = r > 0.022
    star(c, r, rng.uniform(0, math.tau), face, STAR_DARK if rng.random() < 0.6 else STAR_DARKER,
         seg=20 if big else 15, rings=2)
for i in range(120):                                     # Chispas sueltas por toda la nube
    t = rng.random() ** 0.9
    c = scatter(t, 0.2)
    S.ellipsoid(c, Matrix.Identity(3), (rng.uniform(0.0025, 0.007) * (1 - 0.4 * t),) * 3,
                EMBER if rng.random() < 0.6 else STAR_RIM, seg=5, rings=3)
trail = bpy.data.objects.new("HW_StarTrail", make_mesh("HW_StarTrail", S, MATS))
ship.objects.link(trail)
trail.parent = ROOT

tris = 0
for o in ship.objects:
    o.data.calc_loop_triangles()
    tris += len(o.data.loop_triangles)
print("BROOM TRIS %d" % tris)

# --- Presentación -----------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#0d0b12"), 1)
scene.world = world
for name, rot, col, e in (("Key", (math.radians(55), 0, math.radians(-35)), "#ffe2c8", 5.0),
                          ("Rim", (math.radians(70), 0, math.radians(160)), "#8fa0ff", 2.5),
                          ("Under", (math.radians(160), 0, math.radians(10)), "#9be04a", 1.0)):
    ld = bpy.data.lights.new(name, "SUN")
    ld.energy, ld.color = e, srgb(col)
    lo = bpy.data.objects.new(name, ld)
    pres.objects.link(lo)
    lo.rotation_euler = rot
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1200, 800
scene.view_settings.view_transform = "Standard"
os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if not os.environ.get("HW_NO_RENDER"):
    for tag, dvec, dist in (("three_quarter", Vector((-0.9, -0.8, 0.35)), 4.2), ("side", Vector((-1, 0, 0.05)), 4.0),
                            ("front", Vector((0, -1, 0.1)), 3.6), ("back", Vector((0.5, 1, 0.3)), 4.2),
                            ("close", Vector((-0.75, -0.6, 0.25)), 1.9),
                            ("face", Vector((0, -1, 0.1)), 0.7),
                            ("hip", Vector((0.9, -0.15, 0.35)), 0.9)):
        c = (Vector((0, -0.05, 0.4)) if tag in ("close", "face") else
             Vector((0, 0, 0.04)) if tag == "hip" else Vector((0, 0.35, 0.35)))
        cam.location = c + dvec.normalized() * dist
        cam.rotation_euler = (c - cam.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = os.path.join(REF_DIR, "broom_%s.png" % tag)
        bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
