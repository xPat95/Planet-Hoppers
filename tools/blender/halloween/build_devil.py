"""Generador de los diablitos que salen de las grietas del planeta Halloween (junto con los payasos).

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/halloween/build_devil.py

Guarda assets/models/characters/halloween/devil.blend (un modelo por pose de baile: HW_Devil_ArmsUp, Cactus, Point y Sway)
y capturas en docs/references/halloween/devils_*.png. El planeta los importa. Diseño: docs/cami-guidelines.md,
sección 2.

Mismo estilo suave y simple que las brujas (piezas de tools/blender/halloween/figure.py): todo rojo, con los ojos
cerrados (rayitas felices) y dos cuernitos del mismo rojo, bailando. Solo la mitad de arriba: salen de las grietas con la cintura hundida (el origen va en la
cintura). Pocas caras, porque van muchos.
"""

import bpy
import math
import os
import sys
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from figure import Builder, make_mesh, mat, rx, ry, rz, srgb  # noqa: E402

REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "characters", "halloween", "devil.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "halloween")

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "Halloween_Devil"

SKIN = mat("DevilSkin", "#ff1744", emit=0.7)             # Rojo carmesí vivo, distinto del rojo ladrillo del cráneo
EYES = mat("DevilEyes", "#2b0208")                       # Ojos cerrados: rayitas oscuras
MATS = [SKIN, EYES]

# Poses (grados). Frente hacia -Y; flexión negativa = hacia adelante. Brazos: apertura, flexión, codo, giro.
POSES = {
    # Bailando con los dos brazos arriba y la cadera de lado (el torso ladeado)
    "arms_up": {"torso": (4, 14, 10), "head": (-6, -12, -8),
                "arm_R": (160, 0, -25, 0), "arm_L": (155, 0, -35, 0)},
    # Brazos de cactus (codos doblados, antebrazos hacia arriba), torso ladeado al otro lado
    "cactus": {"torso": (2, -12, -8), "head": (-4, 10, 6),
               "arm_R": (95, 0, -95, 90), "arm_L": (95, 0, -95, 90)},
    # Disco: un brazo señalando arriba en diagonal y el otro cruzado, torso girado
    "point": {"torso": (8, -10, 22), "head": (-10, 12, 18),
              "arm_R": (150, -20, 0, 0), "arm_L": (80, -30, -15, 0)},
    # Meneándose: inclinado hacia un lado, un brazo arriba doblado y la otra mano en la cintura
    "sway": {"torso": (6, 18, -12), "head": (-8, -14, 10),
             "arm_R": (140, -10, -90, 30), "arm_L": (45, 0, -105, -80)},
}


def build_devil(pose):
    """Diablito rojo y suave, solo de la cintura para arriba."""
    B = Builder(MATS)
    Rt = rz(pose["torso"][2]) @ ry(pose["torso"][1]) @ rx(pose["torso"][0])
    Rh = Rt @ rz(pose["head"][2]) @ ry(pose["head"][1]) @ rx(pose["head"][0])
    waist = Vector((0, 0, -0.06))                         # Un poco por debajo del origen: queda en la grieta
    belly = waist + Rt @ Vector((0, 0, 0.11))
    chest = belly + Rt @ Vector((0, 0, 0.11))
    shoulders = chest + Rt @ Vector((0, 0, 0.05))
    neck = shoulders + Rt @ Vector((0, -0.005, 0.03))
    head = neck + Rh @ Vector((0, 0, 0.06))

    # Cuerpo redondeado, cuello corto y cabeza ovalada
    B.tube([waist, belly, chest, shoulders], [0.06, 0.065, 0.074, 0.066], 6, SKIN, sx=1.25, sy=0.85,
           ref=Rt @ Vector((0, -1, 0)), cap1=True)
    B.tube([shoulders - Rt @ Vector((0, 0, 0.02)), neck + Rh @ Vector((0, 0, 0.015))], [0.024, 0.022], 6, SKIN)
    HR = (0.045, 0.048, 0.055)
    B.ellipsoid(head, Rh, HR, SKIN, seg=7, rings=4)
    for s in (-1, 1):                                    # Ojos cerrados, felices (∩), porque están bailando
        arc = []
        for t in (-1.0, -0.5, 0.0, 0.5, 1.0):
            x, z = s * 0.018 + t * 0.01, 0.01 + 0.006 * (1 - t * t)
            y = -HR[1] * math.sqrt(max(0.0, 1 - (x / HR[0]) ** 2 - (z / HR[2]) ** 2)) - 0.001
            arc.append(head + Rh @ Vector((x, y, z)))
        B.tube(arc, [0.0022] * 5, 3, EYES)
    for s in (-1, 1):                                    # Dos cuernitos casi rectos, hacia arriba y un poco afuera
        base = head + Rh @ Vector((s * 0.026, -0.008, 0.042))
        B.tube([base - Rh @ Vector((s * 0.006, 0, 0.014)), base, base + Rh @ Vector((s * 0.01, 0, 0.028)),
                base + Rh @ Vector((s * 0.017, 0.003, 0.055))],
               [0.021, 0.015, 0.009, 0.001], 4, SKIN, cap1=True)   # Nace adentro de la cabeza, base ancha

    # Brazos suaves con manitos
    for side, key in ((1, "arm_R"), (-1, "arm_L")):
        abd, flex, elbow, twist = pose[key]
        sh = shoulders + Rt @ Vector((side * 0.07, 0, -0.015))      # Hombro pegado al cuerpo
        Ru = Rt @ ry(-side * abd) @ rx(flex) @ rz(side * twist)
        el = sh + Ru @ Vector((0, 0, -0.16))
        Rf = Ru @ rx(elbow)
        wr = el + Rf @ Vector((0, 0, -0.15))
        B.ellipsoid(sh - Rt @ Vector((side * 0.012, 0, 0)), Rt, (0.034, 0.03, 0.032), SKIN, seg=6, rings=4)  # Hombro
        B.tube([sh - Rt @ Vector((side * 0.035, 0, 0)), sh, el, wr], [0.027, 0.026, 0.019, 0.015], 5, SKIN,
               cap0=True)
        palm = wr + Rf @ Vector((0, 0, -0.025))
        B.ellipsoid(palm, Rf, (0.017, 0.011, 0.026), SKIN, seg=5, rings=3)
        B.hands.append(palm)
    return B


ROOT = bpy.data.objects.new("Halloween_Devil_ROOT", None)
scene.collection.objects.link(ROOT)
for i, (key, name) in enumerate((("arms_up", "ArmsUp"), ("cactus", "Cactus"), ("point", "Point"), ("sway", "Sway"))):
    B = build_devil(POSES[key])
    me = make_mesh("HW_Devil_" + name, B, MATS)
    ob = bpy.data.objects.new("HW_Devil_" + name, me)
    scene.collection.objects.link(ob)
    ob.parent = ROOT
    ob.location = ((i - 1.5) * 0.6, 0, 0)
    me.calc_loop_triangles()
    print("TRIS %-18s %d" % (ob.name, len(me.loop_triangles)))

# --- Presentación ------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
ground = bpy.data.meshes.new("Ground")                   # Suelo de referencia: la cintura queda hundida
ground.from_pydata([(-2, -1, 0), (2, -1, 0), (2, 1, 0), (-2, 1, 0)], [], [(0, 1, 2, 3)])
ground.materials.append(mat("PreviewGround", "#5a1e18"))
pres.objects.link(bpy.data.objects.new("Ground", ground))
world = bpy.data.worlds.new("Devil_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#0d0b12"), 1)
scene.world = world
for name, rot, col, e in (("Key", (math.radians(55), 0, math.radians(-35)), "#ffe2c8", 5.0),
                          ("Rim", (math.radians(70), 0, math.radians(160)), "#8fa0ff", 2.5)):
    ld = bpy.data.lights.new(name, "SUN")
    ld.energy, ld.color = e, srgb(col)
    lo_ = bpy.data.objects.new(name, ld)
    pres.objects.link(lo_)
    lo_.rotation_euler = rot
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1400, 800
scene.view_settings.view_transform = "Standard"
for screen in bpy.data.screens:
    for area in screen.areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.color_type = "MATERIAL"

os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if not os.environ.get("HW_NO_RENDER"):
    for name, loc in (("front", (0, -2.6, 0.45)), ("three_quarter", (-1.7, -1.9, 0.8))):
        cam.location = loc
        cam.rotation_euler = (Vector((0, 0, 0.2)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = os.path.join(REF_DIR, "devils_%s.png" % name)
        bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
