"""Nave del planeta Halloween: una de las brujas montada en una escoba, con una estela de estrellitas verdes.

La bruja es HW_Witch_Ride de witch.blend (build_witch.py se ejecuta antes): a horcajadas, inclinada hacia adelante,
feliz, con una mano en el palo y la otra saludando. Acá se le agrega la escoba (palo de madera, atado y paja en
mechones) y, detrás de la paja, una estela corta y ondulada de estrellitas verdes que brillan, cada vez más chicas.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/halloween/build_witch_broom.py

Guarda assets/models/ships/halloween/witch_broom.blend y las capturas en docs/references/halloween/broom_*.png (con
HW_NO_RENDER=1 no renderiza). Después: exportar con export_solar_models.py -- halloween_ship.
"""

import bpy
import math
import os
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
STARS_MATS = [mat("BroomStarGreen", "#3fbf1a", emit=0.9, rough=0.22),    # Verde veneno
              mat("BroomStarPurple", "#5e1a9e", emit=0.9, rough=0.22),   # Morado oscuro
              mat("BroomStarOrange", "#e8560c", emit=0.9, rough=0.22),   # Naranja calabaza
              mat("BroomStarRed", "#a8091a", emit=0.9, rough=0.22),      # Rojo sangre
              mat("BroomStarLime", "#a8d40f", emit=0.9, rough=0.22)]    # Verde lima enfermizo
#                                                          Colores de Halloween: dan un poco de miedo
STAR = STARS_MATS[0]                                      # La de siempre: verde
MATS = [WOOD, WOOD_DARK, STRAW, STRAW_DARK, TIE] + STARS_MATS

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
B.ellipsoid(tip, Matrix.Identity(3), (0.02, 0.02, 0.02), WOOD_DARK, seg=8, rings=5)   # Punta redondeada
# Atado y paja: mechones que salen del atado y se abren hacia atrás, en dos tonos
side = Vector((1, 0, 0))
up = side.cross(fwd).normalized() * -1 if side.cross(fwd).z < 0 else side.cross(fwd).normalized()
B.tube([tail + fwd * 0.03, tail - fwd * 0.03], [0.034, 0.034], 12, TIE)
STRANDS = 80                                             # En tres capas, de adentro hacia afuera
for k in range(STRANDS):
    layer = k % 3
    a = math.tau * k / STRANDS * 3 + 0.37 * layer
    r0 = 0.008 + 0.009 * layer
    spread = (0.04 + 0.04 * layer) + 0.03 * math.sin(k * 2.3)
    length = 0.42 + 0.06 * math.sin(k * 1.7)
    off = side * math.cos(a) + up * math.sin(a)
    p0 = tail + off * r0
    p1 = tail - fwd * length * 0.5 + off * (r0 + spread * 0.55)
    p2 = tail - fwd * length + off * (r0 + spread) + up * 0.015 * math.sin(k)
    B.tube([p0, p1, p2], [0.012, 0.011, 0.006], 5, STRAW if k % 3 else STRAW_DARK, cap1=True)
broom = bpy.data.objects.new("HW_Broom", make_mesh("HW_Broom", B, MATS))

# La falda no atraviesa el palo: adelante y detrás del asiento, la tela que está cerca del palo se levanta y queda
# colgando encima, como una carpa sobre una barra (más alta justo arriba del palo, cayendo hacia los costados)
black = {i for i, m in enumerate(witch_me.materials) if m.name == "HW_WitchBlack"}
skirt_v = {v for p in witch_me.polygons if p.material_index in black for v in p.vertices}
ROD_R, ROD_W = 0.034, 0.15                               # Radio del palo con la tela, y ancho de la carpa (ancha:
#                                                          redondeada, sin hacer una cresta en punta)
for i in skirt_v:
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
    top = axis.z + ROD_R * (1 - (dl / ROD_W) ** 2) - 0.03 * (dl / ROD_W) ** 2
    if v.co.z < top:
        v.co.z = top
witch_me.update()
ship.objects.link(broom)
broom.parent = ROOT

# --- Estela de estrellitas verdes ---------------------------------------------------------------------------------
# Detrás de la paja, en una curva corta que ondula, cada vez más chicas y girando un poco
S = Builder(MATS)


def star(c, r, spin, n_axis, m=STAR, seg=40, rings=5):
    """Estrella inflada, como un almohadón: puntas redondeadas y gordita en el medio, de frente hacia n_axis."""
    a_ = Vector((1, 0, 0)) if abs(n_axis.x) < 0.9 else Vector((0, 1, 0))
    u = (a_ - n_axis * a_.dot(n_axis)).normalized()
    v = n_axis.cross(u)

    def edge(ang):                                       # Contorno: cinco puntas suaves y redondas
        k = (1 + math.cos(5 * (ang - spin))) / 2
        return r * (0.6 + 0.4 * k)                      # Gordita: puntas cortas, anchas y redondas
    base = len(S.v)
    S.v += [c + n_axis * r * 0.5, c - n_axis * r * 0.5]     # Centro de adelante y de atrás: bien inflada
    for i in range(1, rings + 1):
        f = i / rings
        for side in (1, -1):
            for k in range(seg):
                ang = math.tau * k / seg
                tip = (1 + math.cos(5 * (ang - spin))) / 2  # Hacia las puntas se afina (más gordita en el medio)
                h = r * 0.5 * math.sqrt(max(0.0, 1 - f ** 2)) * (1 - 0.35 * tip * f)
                S.v.append(c + (u * math.cos(ang) + v * math.sin(ang)) * edge(ang) * f + n_axis * h * side)

    def ring(i, side):                                   # Índice del primer vértice del anillo i (1..rings)
        return base + 2 + ((i - 1) * 2 + (0 if side > 0 else 1)) * seg

    for side, center in ((1, base), (-1, base + 1)):
        r1 = ring(1, side)
        for k in range(seg):
            k1 = (k + 1) % seg
            S.add((center, r1 + k, r1 + k1) if side > 0 else (center, r1 + k1, r1 + k), m)
        for i in range(1, rings):
            a0, a1 = ring(i, side), ring(i + 1, side)
            for k in range(seg):
                k1 = (k + 1) % seg
                q = (a0 + k, a1 + k, a1 + k1, a0 + k1)
                S.add(q if side > 0 else q[::-1], m)
    # El borde de afuera: la cara de adelante y la de atrás se tocan (el último anillo de cada una es el mismo borde)
    fa, fb = ring(rings, 1), ring(rings, -1)
    for k in range(seg):
        k1 = (k + 1) % seg
        S.add((fa + k, fb + k, fb + k1, fa + k1), m)


start = tail - fwd * 0.5
STARS = 16
for i in range(STARS):                                   # Estrellas en la curva, de colores que se van turnando
    t = i / (STARS - 1)
    wob = 0.5 * math.sin(i * 2.3)                        # Unas arriba y otras abajo de la curva
    c = start - fwd * (0.95 * t) + up * (0.09 * math.sin(t * 5.5) + 0.04 * wob) + side * (0.05 * math.sin(t * 3.7 + 1) + 0.04 * math.cos(i * 1.9))
    r = 0.07 * (1 - 0.65 * t) * (1.0 if i % 3 == 0 else 0.65)
    star(c, r, i * 0.7, Vector((0.25 * math.sin(i), 0.3, 1)).normalized() if i % 2 else -fwd,
         STARS_MATS[(i * 2) % len(STARS_MATS)])
for i in range(30):                                      # Chispitas sueltas alrededor de la estela
    t = (i + 0.5) / 30
    c = start - fwd * (0.95 * t) + up * (0.16 * math.sin(i * 2.1)) + side * (0.13 * math.cos(i * 1.7))
    star(c, 0.022 * (1 - 0.5 * t), i, Vector((0, 0.2, 1)).normalized(), STARS_MATS[i % len(STARS_MATS)],
         seg=15, rings=2)                                 # Las chiquitas, con menos caras
trail = bpy.data.objects.new("HW_StarTrail", make_mesh("HW_StarTrail", S, MATS))   # Suaves: infladas
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
                            ("close", Vector((-0.75, -0.6, 0.25)), 1.9)):
        c = Vector((0, -0.05, 0.4)) if tag == "close" else Vector((0, 0.35, 0.35))
        cam.location = c + dvec.normalized() * dist
        cam.rotation_euler = (c - cam.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = os.path.join(REF_DIR, "broom_%s.png" % tag)
        bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
