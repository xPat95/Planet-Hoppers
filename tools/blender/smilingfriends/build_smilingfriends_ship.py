"""Generador de la nave de Smiling Friends: el OVNI de T2 EP6 "Charlie, Pim, and Bill vs. the Alien".

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/smilingfriends/build_smilingfriends_ship.py

Guarda assets/models/ships/smilingfriends/smilingfriends_ship.blend y capturas en
docs/references/smilingfriends/ship_*.png. Diseño: docs/cami-guidelines.md, sección 1.7.

Platillo plateado con cúpula negra brillante. Arriba, líneas de paneles y placas más oscuras; abajo, un
centro con ocho gajos, tres luces grandes amarillo verdosas y placas rectangulares. El haz verde de
abducción es un objeto aparte (SF_Ship_Beam) para encenderlo o apagarlo en el juego.
"""

import bpy
import bmesh
import math
import os
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "ships", "smilingfriends", "smilingfriends_ship.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "smilingfriends")

R = 2.0          # Radio del platillo
SEG = 48         # Segmentos alrededor

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.name = "SmilingFriends_Ship"


def srgb(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(((v + 0.055) / 1.055) ** 2.4 if v > 0.04045 else v / 12.92 for v in c)


def mat(name, hexcol, emit=0.0, rough=0.5, metal=0.0, alpha=1.0):
    m = bpy.data.materials.new("SF_" + name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Roughness"].default_value = rough
    b.inputs["Metallic"].default_value = metal
    if emit:
        b.inputs["Emission Color"].default_value = (*srgb(hexcol), 1)
        b.inputs["Emission Strength"].default_value = emit
    if alpha < 1.0:
        b.inputs["Alpha"].default_value = alpha
        try:
            m.surface_render_method = "BLENDED"
        except AttributeError:
            m.blend_method = "BLEND"
        m.use_backface_culling = False
    m.diffuse_color = (*srgb(hexcol), alpha)
    return m


SILVER = mat("ShipSilver", "#c9ccd2", rough=0.45, metal=0.0)
SILVER_D = mat("ShipSilverDark", "#a3a7ae", rough=0.5, metal=0.0)
LINE = mat("ShipLine", "#5d6168", rough=0.6)
DOME = mat("ShipDome", "#0c0c0f", rough=0.08)
LIGHT = mat("ShipLight", "#f1f5d2", emit=4.0)
GLOW = mat("ShipGlow", "#a9dc3c", emit=3.0)
BEAM = mat("ShipBeam", "#7dff7a", emit=2.0, alpha=0.3)


def f_top(r):
    """Altura de la cara superior: sube suave del borde a la cúpula."""
    t = max(0.0, min(1.0, r / R))
    return 0.05 + 0.42 * (1 - t ** 1.6)


def f_bot(r):
    """Altura de la cara inferior: más plana, apenas convexa."""
    t = max(0.0, min(1.0, r / R))
    return -0.05 - 0.2 * (1 - t ** 2.2)


def new_part():
    return bmesh.new(), []


def mi(mats, m):
    if m not in mats:
        mats.append(m)
    return mats.index(m)


def build(name, bm, mats, outline=True):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for m in mats:
        me.materials.append(m)
    ob = bpy.data.objects.new(name, me)
    scene.collection.objects.link(ob)
    ob.parent = ROOT
    if outline:                                           # Contorno de caricatura (solo para ver en Blender)
        ol = bpy.data.materials.get("SF_Outline")
        if ol is None:                                    # Negro, sin sombra propia (el casco no se oscurece)
            ol = mat("Outline", "#000000", rough=1.0)
            ol.use_backface_culling = True
            nt = ol.node_tree
            lp, tr, mix = nt.nodes.new("ShaderNodeLightPath"), nt.nodes.new("ShaderNodeBsdfTransparent"),                 nt.nodes.new("ShaderNodeMixShader")
            out = nt.nodes["Material Output"]
            nt.links.new(lp.outputs["Is Shadow Ray"], mix.inputs[0])
            nt.links.new(nt.nodes["Principled BSDF"].outputs[0], mix.inputs[1])
            nt.links.new(tr.outputs[0], mix.inputs[2])
            nt.links.new(mix.outputs[0], out.inputs["Surface"])
            try:
                ol.use_transparent_shadow = True
            except AttributeError:
                pass
        me.materials.append(ol)
        mod = ob.modifiers.new("Outline", "SOLIDIFY")
        mod.thickness, mod.offset, mod.use_flip_normals, mod.use_rim = 0.025, 1.0, True, False
        mod.material_offset = 100
    return ob


ROOT = bpy.data.objects.new("SmilingFriends_Ship_ROOT", None)
scene.collection.objects.link(ROOT)

# --- Casco: perfil girado (arriba y abajo) con el borde afilado -------------------------------------------
bm, mats = new_part()
profile = ([(R * (1 - k / 14) * 0.999 + 0.0, f_top(R * (1 - k / 14))) for k in range(0, 13)]
           + [(0.0, f_top(0.0))])                        # De borde a centro (arriba)
under = [(R * (1 - k / 14), f_bot(R * (1 - k / 14))) for k in range(0, 13)] + [(0.0, f_bot(0.0))]
rim = [(R + 0.06, 0.0)]
rings = []
pts = list(reversed(profile)) + rim + under              # centro arriba -> borde -> centro abajo
for r, z in pts:
    if r < 1e-6:
        rings.append([bm.verts.new((0, 0, z))])
    else:
        rings.append([bm.verts.new((r * math.cos(a), r * math.sin(a), z))
                      for a in (i / SEG * math.tau for i in range(SEG))])
for r0, r1 in zip(rings, rings[1:]):
    for i in range(SEG):
        j = (i + 1) % SEG
        if len(r0) == 1:
            f = bm.faces.new((r0[0], r1[i], r1[j]))
        elif len(r1) == 1:
            f = bm.faces.new((r0[i], r1[0], r0[j]))
        else:
            f = bm.faces.new((r0[i], r1[i], r1[j], r0[j]))
        f.smooth = True
        f.material_index = mi(mats, SILVER)
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
build("SF_Ship_Hull", bm, mats)

# --- Cúpula negra brillante --------------------------------------------------------------------------------
bm, mats = new_part()
bmesh.ops.create_uvsphere(bm, u_segments=32, v_segments=16, radius=0.62,
                          matrix=Matrix.Translation((0, 0, f_top(0.6) - 0.05)) @ Matrix.Diagonal((1, 1, 0.78, 1)))
bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < f_top(0.6) - 0.06], context="VERTS")
for f in bm.faces:
    f.smooth = True
    f.material_index = mi(mats, DOME)
build("SF_Ship_Dome", bm, mats)

# --- Detalles pintados (paneles y luces), apenas separados del casco; sin contorno -------------------------
bm, mats = new_part()


RIM_R = R + 0.06


def on_surface(x, y, top, lift):
    r = math.hypot(x, y)
    if r > R:                                            # Sobre el borde afilado: baja hasta el filo
        k = min(1.0, (r - R) / (RIM_R - R))
        z = (f_top(R) if top else f_bot(R)) * (1 - k)
        return (x, y, z + (lift if top else -lift))
    return (x, y, (f_top(r) + lift) if top else (f_bot(r) - lift))


def decal(m, pts2d, top, lift=0.012):
    vs = [bm.verts.new(on_surface(x, y, top, lift)) for x, y in pts2d]
    f = bm.faces.new(vs if top else list(reversed(vs)))
    f.material_index = mi(mats, m)
    return f


def arc_band(m, r0, r1, a0, a1, top, lift=0.012, n=None):
    """Banda curva entre radios r0..r1 y ángulos a0..a1 (líneas y placas que siguen la curva)."""
    n = n or max(2, int(abs(a1 - a0) / math.tau * SEG))
    for k in range(n):
        t0, t1 = a0 + (a1 - a0) * k / n, a0 + (a1 - a0) * (k + 1) / n
        decal(m, [(r0 * math.cos(t0), r0 * math.sin(t0)), (r1 * math.cos(t0), r1 * math.sin(t0)),
                  (r1 * math.cos(t1), r1 * math.sin(t1)), (r0 * math.cos(t1), r0 * math.sin(t1))], top, lift)


def radial(m, a, r0, r1, top, w=0.008, lift=0.013):
    c, s = math.cos(a), math.sin(a)
    nx, ny = -s * w, c * w
    n = max(1, int(abs(r1 - r0) / 0.08))                 # Tramos cortos: la línea sigue la curva del casco
    for k in range(n):
        ra, rb = r0 + (r1 - r0) * k / n, r0 + (r1 - r0) * (k + 1) / n
        decal(m, [(ra * c - nx, ra * s - ny), (rb * c - nx, rb * s - ny), (rb * c + nx, rb * s + ny),
                  (ra * c + nx, ra * s + ny)], top, lift)


# Arriba: como en la serie, un anillo alrededor de la cúpula y paneles irregulares: cada sector tiene su propio
# arco a distinto radio, divisiones que no se alinean y líneas que llegan hasta el filo. Placas oscuras con borde.
import random
lrng = random.Random(77)
LW = 0.008                                               # Medio ancho de las líneas (finas)
arc_band(LINE, 0.74 - LW, 0.74 + LW, 0, math.tau, True, 0.013)
cuts = [0.15, 0.85, 1.35, 2.2, 2.75, 3.5, 4.3, 4.85, 5.6]
arcs = [lrng.uniform(1.1, 1.55) for _ in cuts]           # Arco propio de cada sector
for i, a0 in enumerate(cuts):
    a1 = cuts[(i + 1) % len(cuts)] + (math.tau if i == len(cuts) - 1 else 0)
    rr = arcs[i]
    arc_band(LINE, rr - LW, rr + LW, a0, a1, True, 0.013)
    radial(LINE, a0, 0.74, max(rr, arcs[i - 1]), True)   # Une el anillo interior con los dos arcos vecinos
    for k in range(lrng.randint(1, 2)):                  # Del arco al filo, en ángulos que no coinciden
        radial(LINE, a0 + (a1 - a0) * lrng.uniform(0.2, 0.8), rr, RIM_R, True)


def plate(a, r, da, dr):
    """Placa oscura con borde de línea, siguiendo la curva del casco."""
    arc_band(LINE, r - dr / 2 - 0.02, r + dr / 2 + 0.02, a - da / 2 - 0.02 / r, a + da / 2 + 0.02 / r, True, 0.012, n=3)
    arc_band(SILVER_D, r - dr / 2, r + dr / 2, a - da / 2, a + da / 2, True, 0.015, n=3)


# Como en la imagen: una al frente cerca de la cúpula, una a cada lado y una atrás a la derecha
for a, r, da, dr in ((-1.45, 1.05, 0.32, 0.22), (-0.35, 1.72, 0.2, 0.26), (-2.75, 1.68, 0.22, 0.24),
                     (0.9, 1.3, 0.28, 0.2), (2.3, 1.75, 0.18, 0.22)):
    plate(a, r, da, dr)

# Abajo (como en la serie): centro de ocho gajos con doble anillo; tres luces ovaladas anchas, cada una en su
# hueco con borde; entre ellas, paneles rectangulares oscuros divididos en tres, que llegan a un anillo cerca
# del filo. Todas las líneas empiezan y terminan sobre otra línea.


def ell_pts(cx, cy, a, rr, rt, n=24):
    """Óvalo con el eje rr hacia afuera (radial) y rt a lo ancho (tangencial)."""
    ca, sa = math.cos(a), math.sin(a)
    return [(cx + rr * math.cos(t) * ca - rt * math.sin(t) * sa, cy + rr * math.cos(t) * sa + rt * math.sin(t) * ca)
            for t in (i / n * math.tau for i in range(n))]


def ell_line(cx, cy, a, rr, rt, w=0.008, lift=0.013):
    inner, outer = ell_pts(cx, cy, a, rr - w, rt - w), ell_pts(cx, cy, a, rr + w, rt + w)
    for i in range(len(inner)):
        j = (i + 1) % len(inner)
        decal(LINE, [inner[i], outer[i], outer[j], inner[j]], False, lift)


def oval(m, cx, cy, a, rr, rt, lift, rings=3):
    """Óvalo en anillos concéntricos que siguen la curva del casco (un solo polígono plano quedaba hundido)."""
    prev = None
    for k in range(1, rings + 1):
        cur = [bm.verts.new(on_surface(x, y, False, lift)) for x, y in ell_pts(cx, cy, a, rr * k / rings, rt * k / rings)]
        n = len(cur)
        if prev is None:
            c = bm.verts.new(on_surface(cx, cy, False, lift))
            fs = [bm.faces.new((c, cur[i], cur[(i + 1) % n])) for i in range(n)]
        else:
            fs = [bm.faces.new((prev[i], cur[i], cur[(i + 1) % n], prev[(i + 1) % n])) for i in range(n)]
        for f in fs:
            f.normal_update()
            if f.normal.z > 0:
                f.normal_flip()
            f.material_index = mi(mats, m)
        prev = cur


def rbox(m, a, r0, r1, hw, lift):
    """Rectángulo a lo largo del radio a, de r0 a r1, medio ancho hw (en tramos que siguen la curva)."""
    c, s_ = math.cos(a), math.sin(a)
    n = max(1, int((r1 - r0) / 0.1))
    for k in range(n):
        ra, rb = r0 + (r1 - r0) * k / n, r0 + (r1 - r0) * (k + 1) / n
        decal(m, [(ra * c + s_ * hw, ra * s_ - c * hw), (rb * c + s_ * hw, rb * s_ - c * hw),
                  (rb * c - s_ * hw, rb * s_ + c * hw), (ra * c - s_ * hw, ra * s_ + c * hw)], False, lift)


def leaves_oval(r, t, cx, cy, a, rr, rt):
    """True si el punto a radio r y ángulo t queda fuera del óvalo."""
    x, y = r * math.cos(t) - cx, r * math.sin(t) - cy
    u = x * math.cos(a) + y * math.sin(a)
    v = -x * math.sin(a) + y * math.cos(a)
    return (u / rr) ** 2 + (v / rt) ** 2 >= 1


arc_band(SILVER_D, 0.0, 0.45, 0, math.tau, False, 0.011)
arc_band(LINE, 0.44, 0.46, 0, math.tau, False, 0.013)
arc_band(LINE, 0.6, 0.62, 0, math.tau, False, 0.013)
arc_band(LINE, 1.9, 1.92, 0, math.tau, False, 0.013)       # Anillo cerca del filo
for k in range(8):
    radial(LINE, k / 8 * math.tau, 0.0, 0.45, False, lift=0.014)
LIGHT_R, RR, RT = 1.2, 0.58, 0.78
for k in range(3):
    a = math.radians(90 + k * 120)
    cx, cy = LIGHT_R * math.cos(a), LIGHT_R * math.sin(a)
    ell_line(cx, cy, a, RR, RT)                           # Hueco de la luz, con borde
    oval(GLOW, cx, cy, a, 0.5, 0.68, 0.012)
    oval(LIGHT, cx, cy, a, 0.38, 0.54, 0.016)
    radial(LINE, a, LIGHT_R + RR, 1.91, False)            # Del hueco al anillo del filo
    for side in (-1, 1):                                  # Costados: nacen justo en el borde del hueco
        t = a + side * math.radians(30)
        lo, hi = 0.62, 1.9
        for _ in range(30):
            md = (lo + hi) / 2
            lo, hi = (lo, md) if leaves_oval(md, t, cx, cy, a, RR, RT) else (md, hi)
        radial(LINE, t, hi, 1.91, False)
    # Panel oscuro entre luces: borde, relleno y dos divisiones (tres celdas); toca el anillo central y el del filo
    pa = a + math.radians(60)
    radial(LINE, pa, 0.61, 0.95, False)
    rbox(LINE, pa, 0.95, 1.91, 0.2, 0.012)
    rbox(SILVER_D, pa, 0.97, 1.89, 0.18, 0.015)
    for rr in (1.27, 1.58):
        rbox(LINE, pa, rr - 0.01, rr + 0.01, 0.2, 0.017)
build("SF_Ship_Details", bm, mats, outline=False)

# --- Haz de abducción (objeto aparte) ---------------------------------------------------------------------
bm, mats = new_part()
bmesh.ops.create_cone(bm, cap_ends=False, segments=32, radius1=2.1, radius2=0.45, depth=3.2,
                      matrix=Matrix.Translation((0, 0, f_bot(0) - 1.6)))
for f in bm.faces:
    f.smooth = True
    f.material_index = mi(mats, BEAM)
beam = build("SF_Ship_Beam", bm, mats, outline=False)

# --- Presentación ------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("Ship_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#141a2e"), 1)
scene.world = world
for name, loc, e in (("Key", (-4, -5, 6), 1800), ("Fill", (5, -3, 2), 900), ("Under", (0, -2, -6), 900)):
    ld = bpy.data.lights.new(name, "AREA")
    ld.energy, ld.size = e, 5
    ob = bpy.data.objects.new(name, ld)
    pres.objects.link(ob)
    ob.location = loc
    ob.rotation_euler = (Vector((0, 0, 0)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1200, 800
for screen in bpy.data.screens:                          # Vista 3D: contorno visible y colores de material
    for area in screen.areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.show_backface_culling = True
                space.shading.color_type = "MATERIAL"

total = 0
dg = bpy.context.evaluated_depsgraph_get()
for ob in ROOT.children:
    me = ob.data
    me.calc_loop_triangles()
    total += len(me.loop_triangles)
    print("TRIS %-18s %5d" % (ob.name, len(me.loop_triangles)))
print("TRIS TOTAL %d" % total)

os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
for name, loc, target in (("ship_three_quarter", (0, -6.5, 3.2), (0, 0, -0.6)),
                          ("ship_bottom", (0.01, -3.5, -7.5), (0, 0, 0)),
                          ("ship_side", (0, -7.5, 0.4), (0, 0, -1.0))):
    cam.location = loc
    cam.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    beam.hide_render = name == "ship_bottom"
    scene.render.filepath = os.path.join(REF_DIR, name + ".png")
    bpy.ops.render.render(write_still=True)
beam.hide_render = False
print("DONE", BLEND_OUT)
