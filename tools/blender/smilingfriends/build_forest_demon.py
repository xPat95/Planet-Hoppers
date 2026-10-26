"""Generador del Forest Demon (T1 EP4) como modelo propio, separado del planeta.

Estilo plastilina (claymation): todo con sombreado suave y sin contorno negro.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/smilingfriends/build_forest_demon.py

Guarda assets/models/characters/smilingfriends/forest_demon.blend (objeto SF_ForestDemon, de pie sobre z=0 y
mirando hacia -Y, ~2.6 unidades de alto) y capturas en docs/references/smilingfriends/forest_demon_*.png.
El planeta lo importa desde ese archivo (build_smilingfriends_planet.py).

Referencia: criatura de ramas, flaca y encorvada, con brazos y patas largos de nudos, dedos de ramas,
cabeza redonda con una corona radial de ramas, ojos azules brillantes y saltones y una boca enorme
abierta con dientes amarillos.
"""

import bpy
import bmesh
import math
import os
import random
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "characters", "smilingfriends", "forest_demon.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "smilingfriends")
SEED = 4404

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "Forest_Demon"
rng = random.Random(SEED)


def srgb(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(((v + 0.055) / 1.055) ** 2.4 if v > 0.04045 else v / 12.92 for v in c)


def mat(name, hexcol, emit=0.0, rough=0.9):
    m = bpy.data.materials.get("SF_" + name) or bpy.data.materials.new("SF_" + name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Roughness"].default_value = rough
    if emit:
        b.inputs["Emission Color"].default_value = (*srgb(hexcol), 1)
        b.inputs["Emission Strength"].default_value = emit
    m.diffuse_color = (*srgb(hexcol), 1)
    return m


BARK, BARK2 = mat("DemonBark", "#3b2e28", rough=0.55), mat("DemonBarkDark", "#2a201c", rough=0.55)   # brillo de plastilina
EYE, PUPIL = mat("DemonEyes", "#3fb4ff", emit=4.0, rough=0.3), mat("DemonPupil", "#0a0a0a")
MOUTH, TEETH = mat("DemonMouth", "#a8321f"), mat("DemonTeeth", "#e8b24a")

bm = bmesh.new()
mats = []


def mi(m):
    if m not in mats:
        mats.append(m)
    return mats.index(m)


def add(m, op, smooth=True, **kw):
    before = set(bm.faces)
    getattr(bmesh.ops, op)(bm, **kw)
    for f in bm.faces:
        if f not in before:
            f.material_index = mi(m)
            f.smooth = smooth


def ball(m, r, c, seg=10, rings=6, scale=(1, 1, 1)):
    add(m, "create_uvsphere", u_segments=seg, v_segments=rings, radius=r,
        matrix=Matrix.Translation(c) @ Matrix.Diagonal((*scale, 1)))


def stick(m, p0, p1, r0, r1, seg=10):
    """Cilindro cónico de p0 a p1. La corteza tiene grosor mínimo (puntas romas, como plastilina)."""
    if m is BARK or m is BARK2:
        r0, r1 = max(r0, 0.016), max(r1, 0.013)
    p0, p1 = Vector(p0), Vector(p1)
    d = p1 - p0
    ln = d.length or 1e-4
    rot = d.to_track_quat("Z", "Y").to_matrix().to_4x4()
    add(m, "create_cone", cap_ends=True, cap_tris=False, segments=seg, radius1=r0, radius2=r1, depth=ln,
        matrix=Matrix.Translation((p0 + p1) / 2) @ rot)


def gnarled(m, pts, r0, r1, wobble=0.04, knots=True, seg=10):
    """Miembro nudoso: polilínea con ruido, grosor decreciente y nudos en las uniones (sin huecos)."""
    pts = [Vector(p) for p in pts]
    out = [pts[0]]
    for a, b in zip(pts, pts[1:]):
        for k in (1, 2):
            q = a.lerp(b, k / 2)
            if k == 1:
                q += Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * wobble
            out.append(q)
    n = len(out) - 1
    for i, (a, b) in enumerate(zip(out, out[1:])):
        ra, rb = r0 + (r1 - r0) * i / n, r0 + (r1 - r0) * (i + 1) / n
        stick(m, a, b, ra, rb, seg)
        if knots:
            ball(m, rb * 1.05, b, seg=seg, rings=6)   # nudo sutil: tapa la unión sin parecer cuenta
    return out


def twig(m, base, direction, length, r, depth=2):
    """Rama con ramitas laterales (recursiva)."""
    d = Vector(direction).normalized()
    tip = Vector(base) + d * length
    mid = Vector(base).lerp(tip, 0.5) + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1))) * length * 0.08
    stick(m, base, mid, r, r * 0.7, 7)
    stick(m, mid, tip, r * 0.7, r * 0.15, 6)
    if depth > 0:
        for k in range(rng.randint(1, 2)):
            side = d.cross(Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-1, 1)))).normalized()
            twig(m, Vector(base).lerp(tip, rng.uniform(0.45, 0.8)), (d + side * 0.9).normalized(),
                 length * rng.uniform(0.35, 0.5), r * 0.5, depth - 1)


def claw(base, direction, n, length, r):
    """Mano o pie: dedos largos de ramas abiertos en abanico, con nudillo."""
    d = Vector(direction).normalized()
    side = d.cross(Vector((0, 0, 1)))
    if side.length < 0.1:
        side = d.cross(Vector((1, 0, 0)))
    side.normalize()
    up = side.cross(d).normalized()
    for k in range(n):
        t = (k - (n - 1) / 2) / max(n - 1, 1)
        fd = (d + side * t * 1.4 + up * rng.uniform(-0.15, 0.15)).normalized()
        knuckle = Vector(base) + fd * length * 0.45
        tip = knuckle + (fd + up * -0.35).normalized() * length * 0.6
        stick(BARK, base, knuckle, r, r * 0.75, 8)
        ball(BARK, r * 0.85, knuckle, seg=8, rings=5)
        stick(BARK, knuckle, tip, r * 0.75, r * 0.1, 6)


# --- Cuerpo: encorvado hacia adelante, cadera estrecha, pecho y hombros más anchos -----------------------
pelvis, chest, neck = Vector((0, 0.1, 1.15)), Vector((0, -0.2, 1.75)), Vector((0, -0.4, 2.05))
spine = gnarled(BARK, [pelvis, Vector((0, -0.05, 1.45)), chest, neck], 0.13, 0.11, wobble=0.03)
ball(BARK, 0.17, pelvis, seg=10, rings=6, scale=(1.3, 0.9, 0.8))
ball(BARK, 0.24, chest, seg=10, rings=6, scale=(1.6, 0.8, 1.0))                     # pecho en triángulo
for k in range(4):                                                                  # costillas de ramas
    z = 1.5 + k * 0.08
    for sx in (-1, 1):
        stick(BARK2, (0, -0.25 - k * 0.02, z), (sx * 0.24, -0.12, z - 0.05), 0.025, 0.012, 4)
shoulders = [Vector((sx * 0.38, -0.22, 1.85)) for sx in (-1, 1)]
gnarled(BARK, [shoulders[0], chest, shoulders[1]], 0.08, 0.08, wobble=0.0)

# --- Patas: flacas, dobladas, rodillas abiertas, pies grandes de ramas ---------------------------------------
for sx in (-1, 1):
    hip = pelvis + Vector((sx * 0.14, 0, -0.05))
    knee = Vector((sx * 0.42, -0.25, 0.62))
    ankle = Vector((sx * 0.38, 0.05, 0.12))
    gnarled(BARK, [hip, knee, ankle], 0.075, 0.05, wobble=0.035)
    claw(ankle, (sx * 0.15, -1, -0.25), 4, 0.42, 0.035)
    stick(BARK, ankle, ankle + Vector((0, 0.22, -0.08)), 0.035, 0.01, 4)            # talón

# --- Brazos: largos y nudosos; el derecho alzado con la mano abierta, el izquierdo bajo con garras ---------
arms = [
    (shoulders[1], [Vector((0.85, -0.35, 2.05)), Vector((1.15, -0.55, 2.55))], (0.4, -0.5, 1)),     # alzado
    (shoulders[0], [Vector((-0.8, -0.45, 1.55)), Vector((-1.2, -0.7, 1.2))], (-0.6, -0.6, -0.4)),   # bajo
]
for sh, (elbow, wrist), hand_dir in arms:
    gnarled(BARK, [sh, elbow, wrist], 0.06, 0.045, wobble=0.035)
    ball(BARK, 0.08, wrist, seg=8, rings=5)
    claw(wrist, hand_dir, 5, 0.5, 0.03)
for sh in shoulders:                                                                # ramitas en los hombros
    twig(BARK, sh, (sh.x * 0.6, 0.6, 0.8), 0.45, 0.03, 1)

# --- Cabeza: redonda, ojos azules saltones, boca enorme abierta, corona radial de ramas -------------------
head = Vector((0, -0.62, 2.3))
ball(BARK, 0.36, head, seg=14, rings=10, scale=(1.0, 0.92, 1.0))
ball(BARK2, 0.2, head + Vector((0, -0.2, -0.2)), seg=10, rings=6, scale=(1.5, 1.0, 1.2))  # hocico
for sx in (-1, 1):
    brow = head + Vector((sx * 0.15, -0.27, 0.13))
    ball(BARK2, 0.12, brow + Vector((0, 0.02, 0.06)), seg=8, rings=5, scale=(1.3, 0.8, 0.6))   # ceja
    ball(EYE, 0.11, brow, seg=10, rings=6)
    ball(PUPIL, 0.05, brow + Vector((0, -0.095, 0)), seg=8, rings=4)
# Boca: hueco ovalado grande, hundido en el hocico, con labio y dientes alrededor del borde
mouth_c = head + Vector((0, -0.33, -0.22))
ball(MOUTH, 0.17, mouth_c, seg=12, rings=8, scale=(1.25, 0.45, 1.0))
for k in range(16):                                                                 # labio
    a0, a1 = k / 16 * math.tau, (k + 1) / 16 * math.tau
    p0 = mouth_c + Vector((math.cos(a0) * 0.22, -0.04, math.sin(a0) * 0.18))
    p1 = mouth_c + Vector((math.cos(a1) * 0.22, -0.04, math.sin(a1) * 0.18))
    stick(BARK2, p0, p1, 0.03, 0.03, 5)
for k in range(7):                                                                  # dientes de arriba y abajo
    t = (k - 3) / 3
    for zs in (1, -1):
        x = t * 0.17
        base = mouth_c + Vector((x, -0.06, zs * 0.17 * math.sqrt(max(1 - t * t * 0.8, 0.1))))
        stick(TEETH, base, base + Vector((x * 0.1, -0.01, -zs * rng.uniform(0.07, 0.11))), 0.03, 0.0, 4)
for k in range(14):                                                                 # corona radial de ramas
    th = math.radians(-40 + 260 * k / 13) + rng.uniform(-0.08, 0.08)
    d = Vector((math.cos(th), rng.uniform(0.0, 0.5), math.sin(th))).normalized()
    twig(BARK, head + d * 0.3, d, rng.uniform(0.6, 1.0), 0.035, 2)

# --- Malla, objeto y guardado -------------------------------------------------------------------------------
# El cuerpo (corteza) se funde en una sola superficie continua: remallado por vóxeles, suavizado y reducido.
# Ojos, pupilas, boca, dientes y detalles oscuros quedan como piezas aparte para conservar sus colores.
BODY_TRIS = 4500
body_bm, feat_bm = bm.copy(), bm.copy()
bm.free()
bark_i = mats.index(BARK)
bmesh.ops.delete(body_bm, geom=[f for f in body_bm.faces if f.material_index != bark_i], context="FACES")
bmesh.ops.delete(feat_bm, geom=[f for f in feat_bm.faces if f.material_index == bark_i], context="FACES")

body = bpy.data.objects.new("_body", bpy.data.meshes.new("_body"))
body_bm.to_mesh(body.data)
body_bm.free()
scene.collection.objects.link(body)
rm = body.modifiers.new("Remesh", "REMESH")
rm.mode, rm.voxel_size, rm.adaptivity = "VOXEL", 0.012, 0.0
sm = body.modifiers.new("Smooth", "SMOOTH")
sm.factor, sm.iterations = 0.5, 4
dg = bpy.context.evaluated_depsgraph_get()
dense = body.evaluated_get(dg).to_mesh()
dense.calc_loop_triangles()
dec = body.modifiers.new("Decimate", "DECIMATE")
dec.ratio = min(1.0, BODY_TRIS / max(len(dense.loop_triangles), 1))
body.evaluated_get(dg).to_mesh_clear()
dg = bpy.context.evaluated_depsgraph_get()
body_me = bpy.data.meshes.new_from_object(body.evaluated_get(dg))
bpy.data.objects.remove(body)

out = bmesh.new()
out.from_mesh(body_me)
for f in out.faces:
    f.material_index = 0
    f.smooth = True
fm = bpy.data.meshes.new("_feat")
feat_bm.to_mesh(fm)
feat_bm.free()
before = set(out.faces)
out.from_mesh(fm)
final_mats = [BARK] + [m for m in mats if m is not BARK]
for f in out.faces:
    if f not in before:
        f.material_index = final_mats.index(mats[f.material_index])
me = bpy.data.meshes.new("SF_ForestDemon")
out.to_mesh(me)
out.free()
for m in final_mats:
    me.materials.append(m)
demon = bpy.data.objects.new("SF_ForestDemon", me)
scene.collection.objects.link(demon)
me.calc_loop_triangles()
print("TRIS ForestDemon %d" % len(me.loop_triangles))

# Presentación: luces y cámara de revisión (no forman parte del modelo)
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("Demon_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#7d8a7a"), 1)
scene.world = world
for name, loc, e in (("Key", (-4, -6, 6), 2500), ("Fill", (5, -4, 3), 1200), ("Rim", (0, 5, 5), 1800)):
    ld = bpy.data.lights.new(name, "AREA")
    ld.energy, ld.size = e, 4
    ob = bpy.data.objects.new(name, ld)
    pres.objects.link(ob)
    ob.location = loc
    ob.rotation_euler = (Vector((0, 0, 1.5)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x = scene.render.resolution_y = 900
os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
for name, loc in (("front", (0, -6.5, 2.0)), ("three_quarter", (4.0, -5.0, 2.8))):
    cam.location = loc
    cam.rotation_euler = (Vector((0, -0.3, 1.5)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    scene.render.filepath = os.path.join(REF_DIR, "forest_demon_%s.png" % name)
    bpy.ops.render.render(write_still=True)
print("DONE", BLEND_OUT)
