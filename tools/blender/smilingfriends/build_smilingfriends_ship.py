"""Generador de la nave de Smiling Friends: el OVNI de T2 EP6 "Charlie, Pim, and Bill vs. the Alien", con Pim y
Charlie adentro.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/smilingfriends/build_smilingfriends_ship.py

Guarda assets/models/ships/smilingfriends/smilingfriends_ship.blend y capturas en
docs/references/smilingfriends/ship_*.png. Diseño: docs/cami-guidelines.md, sección 1.7.

Platillo plateado con cúpula de vidrio transparente. Arriba, líneas de paneles y placas más oscuras; abajo, un
centro con ocho gajos, tres luces grandes amarillo verdosas y placas rectangulares. Adentro: piso verde azulado,
dos asientos y un tablero de mando chico. Pim y Charlie son sus dibujos convertidos en "stickers inflados",
parados delante de las sillas y mirando al frente. El haz verde
de abducción es un objeto aparte (SF_Ship_Beam) para encenderlo o apagarlo.
"""

import bpy
import bmesh
import math
import os
import random
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BLEND_OUT = os.path.join(REPO, "assets", "models", "ships", "smilingfriends", "smilingfriends_ship.blend")
REF_DIR = os.path.join(REPO, "docs", "references", "smilingfriends")

# El casco y sus calcomanías se diseñan con radio 2 y se agrandan S veces (la cabina necesita lugar para dos)
R = 2.0          # Radio del platillo (unidades de diseño)
S = 1.5          # Escala final: radio 3
SEG = 64         # Segmentos alrededor
DR = 0.9         # Radio de la cúpula (unidades de diseño): adentro el casco es el piso plano de la cabina

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


SILVER = mat("ShipSilver", "#c9ccd2", rough=0.45)
SILVER_D = mat("ShipSilverDark", "#a3a7ae", rough=0.5)
LINE = mat("ShipLine", "#5d6168", rough=0.6)
GLASS = mat("ShipGlass", "#a9cfe0", rough=0.03, alpha=0.07)    # Cúpula: vidrio transparente
LIGHT = mat("ShipLight", "#f1f5d2", emit=4.0)
GLOW = mat("ShipGlow", "#a9dc3c", emit=3.0)
BEAM = mat("ShipBeam", "#7dff7a", emit=2.0, alpha=0.3)
# Cabina (colores de la escena de la serie)
CAB = mat("CabinFloor", "#24403f", rough=0.7)
CAB_D = mat("CabinDark", "#172b2c", rough=0.8)
CAB_L = mat("CabinRim", "#86c2bb", rough=0.5)
def outline_material():
    ol = bpy.data.materials.get("SF_Outline")
    if ol is None:                                       # Negro, sin sombra propia (no oscurece lo de adentro)
        ol = mat("Outline", "#000000", rough=1.0)
        ol.use_backface_culling = True
        nt = ol.node_tree
        lp, tr, mix = nt.nodes.new("ShaderNodeLightPath"), nt.nodes.new("ShaderNodeBsdfTransparent"), \
            nt.nodes.new("ShaderNodeMixShader")
        out = nt.nodes["Material Output"]
        nt.links.new(lp.outputs["Is Shadow Ray"], mix.inputs[0])
        nt.links.new(nt.nodes["Principled BSDF"].outputs[0], mix.inputs[1])
        nt.links.new(tr.outputs[0], mix.inputs[2])
        nt.links.new(mix.outputs[0], out.inputs["Surface"])
        try:
            ol.use_transparent_shadow = True
        except AttributeError:
            pass
    return ol


def add_outline(ob, thickness):
    """Contorno de caricatura: casco invertido (Solidify) con el material negro en la última ranura."""
    ob.data.materials.append(outline_material())
    mod = ob.modifiers.new("Outline", "SOLIDIFY")
    mod.thickness, mod.offset, mod.use_flip_normals, mod.use_rim = thickness, 1.0, True, False
    mod.material_offset = 100


def new_part():
    return bmesh.new(), []


def mi(mats, m):
    if m not in mats:
        mats.append(m)
    return mats.index(m)


def build(name, bm, mats, outline=0.0, parent=None, scale=1.0, smooth=None):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    if scale != 1.0:
        me.transform(Matrix.Scale(scale, 4))
    for m in mats:
        me.materials.append(m)
    if smooth is not None:
        for p in me.polygons:
            p.use_smooth = smooth
    ob = bpy.data.objects.new(name, me)
    scene.collection.objects.link(ob)
    ob.parent = parent or ROOT
    if outline:
        add_outline(ob, outline)
    return ob


ROOT = bpy.data.objects.new("SmilingFriends_Ship_ROOT", None)
scene.collection.objects.link(ROOT)


def f_top(r):
    """Altura de la cara superior (diseño): sube suave del borde a la cúpula; adentro de la cúpula es el piso."""
    t = max(DR / R, min(1.0, r / R))
    return 0.05 + 0.42 * (1 - t ** 1.6)


def f_bot(r):
    """Altura de la cara inferior: más plana, apenas convexa."""
    t = max(0.0, min(1.0, r / R))
    return -0.05 - 0.2 * (1 - t ** 2.2)


FLOOR = f_top(0) * S                                     # Altura del piso de la cabina (unidades finales)
DOME_R = DR * S                                          # Radio de la cúpula (unidades finales)
DOME_H = 1.15                                            # Alto de la cúpula / su radio

# --- Casco: perfil girado (arriba, piso de la cabina y abajo) con el borde afilado -------------------------
bm, mats = new_part()
top = [(R - (R - DR) * k / 12, None) for k in range(13)]                # Del borde a la cúpula
floor = [(DR * (1 - k / 4), None) for k in range(1, 4)] + [(0.0, None)]  # Piso plano de la cabina
under = [(R * (1 - k / 16), f_bot(R * (1 - k / 16))) for k in range(16)] + [(0.0, f_bot(0.0))]
pts = [(r, f_top(r)) for r, _ in reversed(floor)] + [(r, f_top(r)) for r, _ in reversed(top)] \
    + [(R + 0.06, 0.0)] + under                          # centro arriba -> borde -> centro abajo
rings = []
for r, z in pts:
    if r < 1e-6:
        rings.append([bm.verts.new((0, 0, z))])
    else:
        rings.append([bm.verts.new((r * math.cos(a), r * math.sin(a), z))
                      for a in (i / SEG * math.tau for i in range(SEG))])
for k, (r0, r1) in enumerate(zip(rings, rings[1:])):
    inside = k < len(floor)                              # Los primeros anillos son el piso de la cabina
    for i in range(SEG):
        j = (i + 1) % SEG
        if len(r0) == 1:
            f = bm.faces.new((r0[0], r1[i], r1[j]))
        elif len(r1) == 1:
            f = bm.faces.new((r0[i], r1[0], r0[j]))
        else:
            f = bm.faces.new((r0[i], r1[i], r1[j], r0[j]))
        f.smooth = not inside
        f.material_index = mi(mats, CAB if inside else SILVER)
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
build("SF_Ship_Hull", bm, mats, outline=0.035, scale=S)

# --- Cúpula de vidrio y su anillo de base ------------------------------------------------------------------
bm, mats = new_part()
bmesh.ops.create_uvsphere(bm, u_segments=48, v_segments=24, radius=DOME_R,
                          matrix=Matrix.Translation((0, 0, FLOOR)) @ Matrix.Diagonal((1, 1, DOME_H, 1)))
bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < FLOOR - 1e-4], context="VERTS")
for f in bm.faces:
    f.smooth = True
    f.material_index = mi(mats, GLASS)
dome = build("SF_Ship_Dome", bm, mats)

bm, mats = new_part()                                    # Anillo plateado donde apoya la cúpula
bmesh.ops.create_cone(bm, cap_ends=False, segments=SEG, radius1=DOME_R + 0.1, radius2=DOME_R + 0.02, depth=0.1,
                      matrix=Matrix.Translation((0, 0, FLOOR + 0.03)))
for f in bm.faces:
    f.smooth = True
    f.material_index = mi(mats, SILVER_D)
build("SF_Ship_DomeRing", bm, mats, outline=0.02)

# --- Calcomanías (paneles y luces), apenas separadas del casco; sin contorno -------------------------------
bm, mats = new_part()
RIM_R = R + 0.06


def on_surface(x, y, top_side, lift):
    r = math.hypot(x, y)
    if r > R:                                            # Sobre el borde afilado: baja hasta el filo
        k = min(1.0, (r - R) / (RIM_R - R))
        z = (f_top(R) if top_side else f_bot(R)) * (1 - k)
        return (x, y, z + (lift if top_side else -lift))
    return (x, y, (f_top(r) + lift) if top_side else (f_bot(r) - lift))


def decal(m, pts2d, top_side, lift=0.012):
    vs = [bm.verts.new(on_surface(x, y, top_side, lift)) for x, y in pts2d]
    f = bm.faces.new(vs if top_side else list(reversed(vs)))
    f.material_index = mi(mats, m)
    return f


def arc_band(m, r0, r1, a0, a1, top_side, lift=0.012, n=None):
    """Banda curva entre radios r0..r1 y ángulos a0..a1 (líneas y placas que siguen la curva)."""
    n = n or max(2, int(abs(a1 - a0) / math.tau * SEG))
    for k in range(n):
        t0, t1 = a0 + (a1 - a0) * k / n, a0 + (a1 - a0) * (k + 1) / n
        decal(m, [(r0 * math.cos(t0), r0 * math.sin(t0)), (r1 * math.cos(t0), r1 * math.sin(t0)),
                  (r1 * math.cos(t1), r1 * math.sin(t1)), (r0 * math.cos(t1), r0 * math.sin(t1))], top_side, lift)


def radial(m, a, r0, r1, top_side, w=0.008, lift=0.013):
    c, s = math.cos(a), math.sin(a)
    nx, ny = -s * w, c * w
    n = max(1, int(abs(r1 - r0) / 0.08))                 # Tramos cortos: la línea sigue la curva del casco
    for k in range(n):
        ra, rb = r0 + (r1 - r0) * k / n, r0 + (r1 - r0) * (k + 1) / n
        decal(m, [(ra * c - nx, ra * s - ny), (rb * c - nx, rb * s - ny), (rb * c + nx, rb * s + ny),
                  (ra * c + nx, ra * s + ny)], top_side, lift)


# Arriba: como en la serie, un anillo alrededor de la cúpula y paneles irregulares: cada sector tiene su propio
# arco a distinto radio, divisiones que no se alinean y líneas que llegan hasta el filo. Placas oscuras con borde.
lrng = random.Random(77)
LW = 0.008                                               # Medio ancho de las líneas (finas)
INNER = DR + 0.12                                        # Anillo alrededor de la cúpula
arc_band(LINE, INNER - LW, INNER + LW, 0, math.tau, True, 0.013)
cuts = [0.15, 0.85, 1.35, 2.2, 2.75, 3.5, 4.3, 4.85, 5.6]
arcs = [lrng.uniform(1.3, 1.7) for _ in cuts]            # Arco propio de cada sector
for i, a0 in enumerate(cuts):
    a1 = cuts[(i + 1) % len(cuts)] + (math.tau if i == len(cuts) - 1 else 0)
    rr = arcs[i]
    arc_band(LINE, rr - LW, rr + LW, a0, a1, True, 0.013)
    radial(LINE, a0, INNER, max(rr, arcs[i - 1]), True)  # Une el anillo interior con los dos arcos vecinos
    for k in range(lrng.randint(1, 2)):                  # Del arco al filo, en ángulos que no coinciden
        radial(LINE, a0 + (a1 - a0) * lrng.uniform(0.2, 0.8), rr, RIM_R, True)


def plate(a, r, da, dr):
    """Placa oscura con borde de línea, siguiendo la curva del casco."""
    arc_band(LINE, r - dr / 2 - 0.02, r + dr / 2 + 0.02, a - da / 2 - 0.02 / r, a + da / 2 + 0.02 / r, True, 0.012, n=3)
    arc_band(SILVER_D, r - dr / 2, r + dr / 2, a - da / 2, a + da / 2, True, 0.015, n=3)


for a, r, da, dr in ((-1.45, 1.2, 0.26, 0.18), (-0.35, 1.8, 0.18, 0.2), (-2.75, 1.78, 0.2, 0.2),
                     (0.9, 1.45, 0.24, 0.18), (2.3, 1.82, 0.16, 0.2)):
    plate(a, r, da, dr)

# Abajo (como en la serie): centro de ocho gajos con doble anillo; tres luces ovaladas anchas, cada una en su
# hueco con borde; entre ellas, paneles rectangulares oscuros divididos en tres, que llegan a un anillo cerca
# del filo. Todas las líneas empiezan y terminan sobre otra línea.


def ell_pts(cx, cy, a, rr, rt, n=32):
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
build("SF_Ship_Details", bm, mats, scale=S)

# --- Haz de abducción (objeto aparte) ---------------------------------------------------------------------
bm, mats = new_part()
bmesh.ops.create_cone(bm, cap_ends=False, segments=48, radius1=2.1 * S, radius2=0.45 * S, depth=3.2 * S,
                      matrix=Matrix.Translation((0, 0, (f_bot(0) - 1.6) * S)))
for f in bm.faces:
    f.smooth = True
    f.material_index = mi(mats, BEAM)
beam = build("SF_Ship_Beam", bm, mats)

# --- Cabina ------------------------------------------------------------------------------------------------
# Como en la escena de la serie: cada asiento es un cojín redondo en un hueco del piso, con un respaldo alto y
# delgado en forma de óvalo detrás (no los envuelve: se los ve enteros). Adelante, un tablero de mando chico.
cabin = bpy.data.objects.new("SF_Ship_Cabin", None)
scene.collection.objects.link(cabin)
cabin.parent = ROOT
# Lugar, tamaño y alto del respaldo de cada asiento (Pim y Charlie están parados delante, del susto)
PODS = {"pim": (Vector((-0.5, 0.12, FLOOR)), 0.34, 0.2), "charlie": (Vector((0.52, 0.14, FLOOR)), 0.42, 0.26)}


def pod(name, base, size, seat_h):
    """Asiento: hueco redondo del piso con borde claro y respaldo ovalado detrás."""
    # Respaldo: óvalo alto y fino, apenas inclinado hacia atrás, con el borde claro
    bm, mats = new_part()
    back_w, back_h, back_d = size * 0.72, size * 1.58, size * 0.075
    lean = Matrix.Rotation(math.radians(-8), 3, "X")
    back_c = Vector((0, size * 0.72, seat_h + back_h * 0.55))
    rounded_box(bm, mats, CAB, back_c, (back_w, back_d, back_h * 0.5), lean, bevel=size * 0.12)
    inset_c = back_c + lean @ Vector((0, -back_d - size * 0.018, 0))
    rounded_box(bm, mats, CAB_L, inset_c,
                (back_w * 0.82, size * 0.018, back_h * 0.39), lean, bevel=size * 0.07)
    back = build(name + "_Back", bm, mats, parent=cabin, outline=0.012)
    back.location = base
    bm, mats = new_part()
    rounded_box(bm, mats, CAB_D, Vector((0, -size * 0.06, seat_h * 0.38)),
                (size * 0.72, size * 0.56, size * 0.10), Matrix(), bevel=size * 0.10)
    cushion = build(name + "_Cushion", bm, mats, parent=cabin, outline=0.008)
    cushion.location = base
    # Hueco del piso con su borde
    bm, mats = new_part()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=48, radius1=size * 1.3, radius2=size * 1.3, depth=0.02)
    for f in bm.faces:
        f.material_index = mi(mats, CAB_D)
    hole = build(name + "_Floor", bm, mats, parent=cabin)
    hole.location = base + Vector((0, 0, 0.012))
    bm, mats = new_part()
    bmesh.ops.create_cone(bm, cap_ends=False, segments=48, radius1=size * 1.35, radius2=size * 1.35, depth=0.03)
    for f in bm.faces:
        f.smooth = True
        f.material_index = mi(mats, CAB_L)
    rim = build(name + "_FloorRim", bm, mats, parent=cabin, outline=0.01)
    rim.location = base + Vector((0, 0, 0.015))
    return base + Vector((0, -0.08, 0.012))               # Donde se para el personaje (sobre el hueco del piso)



def tube(bm, mats, m, path, r, sides=10, cap=False):
    """Tubo por una polilínea (anillos perpendiculares que no giran de golpe)."""
    rings_ = []
    prev = None
    for i, p in enumerate(path):
        t = (path[min(i + 1, len(path) - 1)] - path[max(i - 1, 0)]).normalized()
        a = (prev if prev is not None else (Vector((0, 0, 1)) if abs(t.z) < 0.9 else Vector((1, 0, 0)))).cross(t)
        if a.length < 1e-6:
            a = Vector((1, 0, 0)).cross(t)
        a.normalize()
        b = t.cross(a)
        prev = b
        rr = r[i] if isinstance(r, (list, tuple)) else r
        rings_.append([bm.verts.new(p + rr * (math.cos(k / sides * math.tau) * a + math.sin(k / sides * math.tau) * b))
                       for k in range(sides)])
    for r0, r1 in zip(rings_, rings_[1:]):
        for k in range(sides):
            f = bm.faces.new((r0[k], r0[(k + 1) % sides], r1[(k + 1) % sides], r1[k]))
            f.smooth = True
            f.material_index = mi(mats, m)
    if cap:
        for ring_, p in ((rings_[0], path[0]), (rings_[-1], path[-1])):
            c = bm.verts.new(p)
            for k in range(sides):
                f = bm.faces.new((ring_[k], ring_[(k + 1) % sides], c))
                f.smooth = True
                f.material_index = mi(mats, m)
    return rings_


def rounded_box(bm, mats, m, center, half, rot=Matrix(), bevel=0.02):
    """Caja de esquinas redondeadas (se arma aparte y se copia al bmesh)."""
    tmp = bmesh.new()
    bmesh.ops.create_cube(tmp, size=1.0)
    for v in tmp.verts:
        v.co = Vector((v.co.x * half[0] * 2, v.co.y * half[1] * 2, v.co.z * half[2] * 2))
    bmesh.ops.bevel(tmp, geom=list(tmp.edges), offset=bevel, segments=3, affect="EDGES", profile=0.5)
    vmap = {v: bm.verts.new(Vector(center) + rot @ v.co) for v in tmp.verts}
    for f in tmp.faces:
        nf = bm.faces.new([vmap[v] for v in f.verts])
        nf.smooth = True
        nf.material_index = mi(mats, m)
    tmp.free()


# Tablero de mando chico al frente, entre los dos asientos y más bajo que ellos: un pie y una tapa inclinada hacia
# Pim y Charlie, con pantallitas, botones de colores y dos palancas
BTN_R = mat("ButtonRed", "#ff4a3d", emit=1.5)
BTN_Y = mat("ButtonYellow", "#ffd23d", emit=1.5)
BTN_G = mat("ButtonGreen", "#57e86b", emit=1.5)
BTN_B = mat("ButtonBlue", "#4fb6ff", emit=1.5)
SCREEN = mat("Screen", "#7dff9a", emit=1.2)
bm, mats = new_part()
desk_c = Vector((0, -0.8, FLOOR + 0.36))
tilt = Matrix.Rotation(math.radians(-40), 3, "X")       # La tapa mira hacia ellos (hacia +Y y arriba)
tube(bm, mats, CAB_D, [Vector((0, -0.82, FLOOR)), Vector((0, -0.81, FLOOR + 0.18)), desk_c - Vector((0, 0, 0.04))],
     [0.11, 0.07, 0.07], sides=16, cap=True)
rounded_box(bm, mats, CAB, desk_c, (0.36, 0.13, 0.03), tilt, bevel=0.025)
rounded_box(bm, mats, CAB_L, desk_c - tilt @ Vector((0, 0, 0.01)), (0.375, 0.145, 0.022), tilt, bevel=0.02)
top = lambda x, y: desk_c + tilt @ Vector((x, y, 0.032))  # noqa: E731  Punto sobre la tapa
for x in (-0.2, 0.2):                                    # Pantallitas verdes, una frente a cada uno
    rounded_box(bm, mats, SCREEN, top(x, 0.035), (0.095, 0.05, 0.006), tilt, bevel=0.012)
for k, m in enumerate((BTN_R, BTN_Y, BTN_G, BTN_B, BTN_R, BTN_G)):  # Botones en fila
    c = top(-0.13 + k * 0.052, -0.075)
    ret = bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.018)
    for v in ret["verts"]:
        v.co = c + tilt @ Vector((v.co.x, v.co.y, v.co.z * 0.6))
    for f in {f for v in ret["verts"] for f in v.link_faces}:
        f.smooth = True
        f.material_index = mi(mats, m)
for x in (-0.3, 0.3):                                    # Palancas con bola roja
    base = top(x, -0.07)
    stick = base + tilt @ Vector((0, 0.03, 0.09))
    tube(bm, mats, LINE, [base, stick], 0.008, sides=8)
    ret = bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=8, radius=0.022)
    for v in ret["verts"]:
        v.co = v.co + stick
    for f in {f for v in ret["verts"] for f in v.link_faces}:
        f.smooth = True
        f.material_index = mi(mats, BTN_R)
build("SF_Cabin_Console", bm, mats, parent=cabin, outline=0.01)

SEATS = {k: pod("SF_Cabin_Pod_" + k.capitalize(), *v) for k, v in PODS.items()}


# --- Personajes --------------------------------------------------------------------------------------------
# Pim y Charlie son sus dibujos convertidos en "stickers inflados": la silueta del PNG se recorta en una malla y se
# infla como un almohadón (gruesa en el medio, fina en los bordes), con el dibujo adelante y atrás. Son objetos
# de verdad: las sillas los tapan solo cuando están delante, y de costado se ve un cuerpo redondeado.
CHAR_DIR = os.path.join(REPO, "assets", "models", "characters", "smilingfriends")


def sticker_material(name, image):
    """Colores planos del dibujo (el juego los dibuja sin luz: material `*_Sticker`)."""
    m = bpy.data.materials.new("SF_" + name + "_Sticker")
    m.use_nodes = True
    nodes, links = m.node_tree.nodes, m.node_tree.links
    tex = nodes.new("ShaderNodeTexImage")
    tex.image = image
    tex.extension = "EXTEND"
    bsdf = nodes["Principled BSDF"]
    links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    links.new(tex.outputs["Color"], bsdf.inputs["Emission Color"])
    bsdf.inputs["Emission Strength"].default_value = 0.6
    bsdf.inputs["Roughness"].default_value = 1.0
    return m


def puffy_sticker(name, feet, height, thick, rows=170, roundness=0.13):
    """Malla inflada a partir del PNG: `height` de alto con los pies en `feet`, mirando hacia -Y. `thick` es
    el medio grosor en el centro del cuerpo; `roundness` (fracción del alto) cuánto tarda en llegar a ese grosor
    desde el borde (las partes finas, como dedos y brazos, quedan más finas)."""
    import numpy as np
    from collections import deque
    path = os.path.join(CHAR_DIR, "cutouts", name.lower() + ".png")
    image = bpy.data.images.load(path, check_existing=True)
    image.pack()
    W, H = image.size
    alpha = np.array(image.pixels[:], dtype=np.float32).reshape(H, W, 4)[..., 3]   # Fila 0 = abajo
    ys, xs = np.nonzero(alpha > 0.5)
    y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    cell = (y1 - y0) / rows
    cols = int(math.ceil((x1 - x0) / cell))
    # Máscara por celda: adentro si la mayor parte de la celda es dibujo
    mask = np.zeros((rows, cols), bool)
    for r in range(rows):
        a0, a1 = y0 + int(r * cell), min(H, y0 + int((r + 1) * cell) + 1)
        for c in range(cols):
            b0, b1 = x0 + int(c * cell), min(W, x0 + int((c + 1) * cell) + 1)
            block = alpha[a0:a1, b0:b1]
            mask[r, c] = block.size and block.mean() > 0.45
    # Fuera las motitas sueltas (un pelo fino que quedó partido): solo piezas de un tamaño razonable
    label = -np.ones((rows, cols), int)
    sizes = []
    for r in range(rows):
        for c in range(cols):
            if mask[r, c] and label[r, c] < 0:
                k, q, n = len(sizes), deque([(r, c)]), 0
                label[r, c] = k
                while q:
                    a, b = q.popleft()
                    n += 1
                    for da, db in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        u, v = a + da, b + db
                        if 0 <= u < rows and 0 <= v < cols and mask[u, v] and label[u, v] < 0:
                            label[u, v] = k
                            q.append((u, v))
                sizes.append(n)
    keep = [k for k, n in enumerate(sizes) if n >= 40]
    mask = np.isin(label, keep)
    # Distancia de cada celda al borde (en celdas), con dos pasadas de chaflán
    big = 1e9
    dist = np.where(mask, big, 0.0)
    d2 = math.sqrt(2)
    for r in range(rows):
        for c in range(cols):
            if dist[r, c]:
                best = dist[r, c]
                for dr, dc, w in ((-1, 0, 1), (0, -1, 1), (-1, -1, d2), (-1, 1, d2)):
                    u, v = r + dr, c + dc
                    best = min(best, (dist[u, v] if 0 <= u < rows and 0 <= v < cols else 0.0) + w)
                dist[r, c] = best
    for r in range(rows - 1, -1, -1):
        for c in range(cols - 1, -1, -1):
            if dist[r, c]:
                best = dist[r, c]
                for dr, dc, w in ((1, 0, 1), (0, 1, 1), (1, 1, d2), (1, -1, d2)):
                    u, v = r + dr, c + dc
                    best = min(best, (dist[u, v] if 0 <= u < rows and 0 <= v < cols else 0.0) + w)
                dist[r, c] = best
    s = height / rows                                    # Unidades por celda
    reach = roundness * height / s                       # En celdas
    inside = lambda r, c: 0 <= r < rows and 0 <= c < cols and mask[r, c]   # noqa: E731

    def corner_depth(r, c):
        """Profundidad de una esquina de la grilla: 0 en el borde de la silueta."""
        cells = [(r - 1, c - 1), (r - 1, c), (r, c - 1), (r, c)]
        if not all(inside(a, b) for a, b in cells):
            return 0.0
        t = min(1.0, (min(dist[a, b] for a, b in cells) - 0.5) / reach)
        return thick * (1 - (1 - t) ** 2) ** 0.75          # Borde inclinado (no vertical): el contorno queda parejo

    bm = bmesh.new()
    uv_layer = bm.loops.layers.uv.new()
    front, back, edge = {}, {}, set()

    def vert(r, c, side):
        key = (r, c)
        if key not in front:
            h = corner_depth(r, c)
            x, z = (c - cols / 2) * s, r * s
            if h == 0.0:                                 # Borde: un solo vértice compartido por los dos lados
                front[key] = back[key] = bm.verts.new((x, 0, z))
                edge.add(front[key])
            else:
                front[key] = bm.verts.new((x, -h, z))
                back[key] = bm.verts.new((x, h, z))
        return front[key] if side < 0 else back[key]

    def uv(r, c):
        return ((x0 + c * cell) / W, (y0 + r * cell) / H)

    for r in range(rows):
        for c in range(cols):
            if not mask[r, c]:
                continue
            quad = [(r, c), (r, c + 1), (r + 1, c + 1), (r + 1, c)]
            fv = [vert(a, b, -1) for a, b in quad]       # Adelante (mira a -Y)
            bv = [vert(a, b, 1) for a, b in reversed(quad)]
            for verts, keys in ((fv, quad), (bv, list(reversed(quad)))):
                if len(set(verts)) < 4 or (verts is bv and set(bv) == set(fv)):
                    continue
                try:
                    f = bm.faces.new(verts)
                except ValueError:
                    continue
                f.smooth = True
                f.material_index = 1 if verts is bv else 0  # Atrás: negro, como la espalda de un recorte
                for loop, (a, b) in zip(f.loops, keys):
                    loop[uv_layer].uv = uv(a, b)
    bm.verts.ensure_lookup_table()
    # Alisar la silueta (la grilla deja escalones): cada vértice del borde se acerca a sus vecinos del borde
    for _ in range(6):
        moved = {}
        for v in edge:
            ns = [e.other_vert(v) for e in v.link_edges if e.other_vert(v) in edge]
            if len(ns) >= 2:
                avg = sum((n.co for n in ns), Vector()) / len(ns)
                moved[v] = Vector((v.co.x * 0.5 + avg.x * 0.5, 0, v.co.z * 0.5 + avg.z * 0.5))
        for v, p in moved.items():
            v.co = p
    # La imagen sigue a la silueta alisada (si no, el contorno negro del dibujo queda en zigzag en el borde)
    for f in bm.faces:
        for loop in f.loops:
            loop[uv_layer].uv = uv(loop.vert.co.z / s, loop.vert.co.x / s + cols / 2)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new("SF_" + name)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(sticker_material(name, image))
    back = bpy.data.materials.new("SF_" + name + "_Back")
    back.use_nodes = True
    back.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.012, 0.012, 0.014, 1)
    back.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
    back.diffuse_color = (0.012, 0.012, 0.014, 1)
    me.materials.append(back)
    ob = bpy.data.objects.new("SF_" + name, me)
    scene.collection.objects.link(ob)
    ob.parent = cabin
    ob.location = feet
    add_outline(ob, 0.01)
    return ob


def in_front_of_chair(seat_key, thick):
    """Pies del personaje: parado delante del cojín de su silla, sin tocarlo."""
    base, size, _ = PODS[seat_key]
    return base + Vector((0, -size * 0.62 - thick - 0.03, 0.012))


pim = puffy_sticker("Pim", in_front_of_chair("pim", 0.075), 0.84, 0.075)
charlie = puffy_sticker("Charlie", in_front_of_chair("charlie", 0.095), 1.15, 0.095)

# --- Presentación ------------------------------------------------------------------------------------------
pres = bpy.data.collections.new("Presentation")
scene.collection.children.link(pres)
world = bpy.data.worlds.new("Ship_World")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#141a2e"), 1)
scene.world = world
for name, loc, e in (("Key", (-6, -7, 9), 1500), ("Fill", (7, -4, 3), 700), ("Under", (0, -3, -9), 900)):
    ld = bpy.data.lights.new(name, "AREA")
    ld.energy, ld.size = e, 6
    ob = bpy.data.objects.new(name, ld)
    pres.objects.link(ob)
    ob.location = loc
    ob.rotation_euler = (Vector((0, 0, 0)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
pres.objects.link(cam)
scene.camera = cam
scene.render.engine = "BLENDER_EEVEE"
scene.render.resolution_x, scene.render.resolution_y = 1200, 800
scene.view_settings.view_transform = "Standard"
for screen in bpy.data.screens:                          # Vista 3D: contorno visible y colores de material
    for area in screen.areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                space.shading.show_backface_culling = True
                space.shading.color_type = "MATERIAL"

total = 0
dg = bpy.context.evaluated_depsgraph_get()
for ob in scene.objects:
    if ob.type != "MESH":
        continue
    # Sin contorno (en el juego el contorno no es geometría)
    outline = ob.modifiers.get("Outline")
    if outline:
        outline.show_viewport = False
    bpy.context.view_layer.update()
    me = ob.evaluated_get(bpy.context.evaluated_depsgraph_get()).to_mesh()
    me.calc_loop_triangles()
    total += len(me.loop_triangles)
    print("TRIS %-26s %6d" % (ob.name, len(me.loop_triangles)))
    ob.evaluated_get(bpy.context.evaluated_depsgraph_get()).to_mesh_clear()
    if outline:
        outline.show_viewport = True
print("TRIS TOTAL %d" % total)

os.makedirs(os.path.dirname(BLEND_OUT), exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=BLEND_OUT)
if not os.environ.get("SF_NO_RENDER"):
    for name, loc, target, lens in (("ship_three_quarter", (0, -9.5, 4.8), (0, 0, 0.2), 50),
                                    ("ship_bottom", (0.01, -5.0, -11.0), (0, 0, 0), 50),
                                    ("ship_side", (0, -12.5, 1.0), (0, 0, -0.6), 45),
                                    ("ship_cabin", (0, -3.6, FLOOR + 1.4), (0, 0.1, FLOOR + 0.75), 55),
                                    ("ship_figures", (0, -2.6, FLOOR + 0.9), (0, 0.05, FLOOR + 0.6), 50),
                                    ("ship_figures_back", (0, 2.8, FLOOR + 1.2), (0, 0.05, FLOOR + 0.6), 50),
                                    ("ship_figures_side", (2.7, -1.2, FLOOR + 1.0), (0, -0.1, FLOOR + 0.55), 50)):
        cam.location = loc
        cam.data.lens = lens
        cam.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        close = name in ("ship_figures", "ship_figures_back", "ship_figures_side")
        beam.hide_render = close or name in ("ship_bottom", "ship_cabin")
        dome.hide_render = close                         # De cerca, sin el vidrio para ver bien a los dos
        scene.render.filepath = os.path.join(REF_DIR, name + ".png")
        bpy.ops.render.render(write_still=True)
    beam.hide_render = dome.hide_render = False
print("DONE", BLEND_OUT)
