"""Pim y Charlie a partir de las figuras descargadas por Cami (STL, sin colores): las aliviana, las pinta por zonas y
guarda un archivo por personaje.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/smilingfriends/import_figures.py

Lee assets/models/characters/smilingfriends/source/Pim.stl y Charlie.stl y guarda pim.blend y charlie.blend (objetos
SF_Pim y SF_Charlie, parados sobre z = 0) en assets/models/characters/smilingfriends/, con capturas en
docs/references/smilingfriends/pim_*.png y charlie_*.png. La nave los importa desde esos archivos.

Las figuras traen una sola malla gris. Se rehacen con caras parejas (remallado por vóxeles) para que se puedan
retocar a mano, y se pintan con bordes medidos en vistas ortogonales con grilla: las costuras de la escultura se cortan
con planos y los ojos y botones se buscan como bultos esféricos. Los pedacitos sueltos de un color se funden con lo que
los rodea. Si se retocan a mano en Blender, no volver a correr este script: pisa los archivos. Las pupilas de Pim son piezas sueltas (espirales) y van negras.
El interior de las bocas abiertas se reconoce por lo encerrado que está cada punto (rayos que chocan enseguida).
Las coordenadas de las zonas son las del STL original, antes de escalarlo.
"""

import bpy
import bmesh
import math
import os
import numpy as np
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CHAR_DIR = os.path.join(REPO, "assets", "models", "characters", "smilingfriends")
SRC = os.path.join(CHAR_DIR, "source")
REF_DIR = os.path.join(REPO, "docs", "references", "smilingfriends")


def srgb(h):
    c = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(((v + 0.055) / 1.055) ** 2.4 if v > 0.04045 else v / 12.92 for v in c)


def flat(name, hexcol, emit=0.25):
    """Material plano de caricatura: sin brillo y con un poco de luz propia."""
    m = bpy.data.materials.new("SF_" + name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Roughness"].default_value = 0.85
    b.inputs["Specular IOR Level"].default_value = 0.15
    b.inputs["Emission Color"].default_value = (*srgb(hexcol), 1)
    b.inputs["Emission Strength"].default_value = emit
    m.diffuse_color = (*srgb(hexcol), 1)
    return m


def outline_material():
    ol = bpy.data.materials.get("SF_Outline")
    if ol is None:                                       # Negro, sin sombra propia
        ol = bpy.data.materials.new("SF_Outline")
        ol.use_nodes = True
        nt = ol.node_tree
        bsdf = nt.nodes["Principled BSDF"]
        bsdf.inputs["Base Color"].default_value = (0, 0, 0, 1)
        bsdf.inputs["Roughness"].default_value = 1.0
        ol.diffuse_color = (0, 0, 0, 1)
        ol.use_backface_culling = True
        lp, tr, mix = nt.nodes.new("ShaderNodeLightPath"), nt.nodes.new("ShaderNodeBsdfTransparent"), \
            nt.nodes.new("ShaderNodeMixShader")
        nt.links.new(lp.outputs["Is Shadow Ray"], mix.inputs[0])
        nt.links.new(bsdf.outputs[0], mix.inputs[1])
        nt.links.new(tr.outputs[0], mix.inputs[2])
        nt.links.new(mix.outputs[0], nt.nodes["Material Output"].inputs["Surface"])
    return ol


def add_outline(ob, thickness):
    """Contorno de caricatura: casco invertido (Solidify) con el material negro en la última ranura."""
    ob.data.materials.append(outline_material())
    mod = ob.modifiers.new("Outline", "SOLIDIFY")
    mod.thickness, mod.offset, mod.use_flip_normals, mod.use_rim = thickness, 1.0, True, False
    mod.material_offset = 100


def load_even(path, voxel):
    """Importa el STL y lo rehace con caras parejas (cuadrados del mismo tamaño en toda la figura), para pintar a
    mano cómodo (Alt+clic selecciona anillos). Las piezas sueltas chicas (las espirales de las pupilas de Pim) no
    se rehacen: se alivianan aparte, porque el remallado las borraría."""
    bpy.ops.wm.stl_import(filepath=path)
    ob = bpy.context.selected_objects[0]
    global ORIGINAL
    ORIGINAL = original_surface(ob.data)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.separate(type="LOOSE")
    bpy.ops.object.mode_set(mode="OBJECT")
    parts = sorted(bpy.context.selected_objects, key=lambda o: len(o.data.polygons), reverse=True)
    body, small = parts[0], parts[1:]
    rm = body.modifiers.new("Remesh", "REMESH")
    rm.mode, rm.voxel_size = "VOXEL", voxel
    for o in small:
        dec = o.modifiers.new("Decimate", "DECIMATE")
        dec.ratio = 0.25
    for o in parts:
        bpy.context.view_layer.objects.active = o
        for m in list(o.modifiers):
            bpy.ops.object.modifier_apply(modifier=m.name)
        o["small"] = o is not body
    # Las piezas chicas se marcan con un atributo antes de unir todo
    for o in parts:
        attr = o.data.attributes.new("small", "BOOLEAN", "FACE")
        attr.data.foreach_set("value", [o is not body] * len(o.data.polygons))
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        o.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    for p in body.data.polygons:
        p.use_smooth = True
    return body


ORIGINAL = None

# Direcciones repartidas en una esfera (para medir lo encerrado de un punto)
RAYS = [Vector((math.cos(2.399963 * k) * math.sqrt(1 - z * z), math.sin(2.399963 * k) * math.sqrt(1 - z * z), z))
        for k, z in ((k, 1 - (k + 0.5) / 24 * 2) for k in range(24))]


def enclosed(bvh, p, n, reach=0.08):
    """Fracción de rayos (hacia afuera de la superficie) que chocan con la misma figura antes de reach: alta
    adentro de una boca abierta, baja en la piel de afuera."""
    hits = total = 0
    for d in RAYS:
        if d.dot(n) <= 0.05:
            continue
        total += 1
        if bvh.ray_cast(p + n * 0.002, d, reach)[0] is not None:
            hits += 1
    return hits / max(total, 1)


def original_surface(me):
    """La figura original con todo su detalle: superficie para proyectar y relieve de cada vértice (cuánto sobresale
    respecto de una versión suavizada de sí misma: positivo en pelos y bordes que sobresalen, negativo en surcos)."""
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    bm.verts.ensure_lookup_table()
    co = np.array([v.co for v in bm.verts])
    nrm = np.array([v.normal for v in bm.verts])
    ei = np.array([(e.verts[0].index, e.verts[1].index) for e in bm.edges])
    deg = np.maximum(np.bincount(ei.ravel(), minlength=len(co)), 1).astype(float)
    base = co.copy()
    for _ in range(25):
        acc = np.zeros_like(base)
        np.add.at(acc, ei[:, 0], base[ei[:, 1]])
        np.add.at(acc, ei[:, 1], base[ei[:, 0]])
        base = 0.5 * base + 0.5 * acc / deg[:, None]
    rel = np.einsum("ij,ij->i", co - base, nrm)
    faces = [tuple(v.index for v in f.verts) for f in bm.faces]
    bm.free()
    return {"co": co, "nrm": nrm, "rel": rel, "faces": faces,
            "bvh": BVHTree.FromPolygons([Vector(c) for c in co], faces)}


def strands(name, mat, box, thr=0.0005, lift=0.0012, min_faces=40):
    """Pelos de la figura original (no del remallado, que los borra): las caras que sobresalen dentro de `box`
    ((xmin, xmax), (ymin, ymax), (zmin, zmax)), un poco levantadas para que se vean enteras sobre la cabeza."""
    o = ORIGINAL
    co, rel = o["co"], o["rel"]
    inside = np.all([(co[:, k] > box[k][0]) & (co[:, k] < box[k][1]) for k in range(3)], axis=0) & (rel > thr)
    faces = [f for f in o["faces"] if all(inside[i] for i in f)]
    used = sorted({i for f in faces for i in f})
    remap = {i: k for k, i in enumerate(used)}
    me = bpy.data.meshes.new(name)
    me.from_pydata([Vector(co[i]) + Vector(o["nrm"][i]) * lift for i in used], [], [tuple(remap[i] for i in f) for f in faces])
    bm = bmesh.new()
    bm.from_mesh(me)
    for part in _islands(bm):                            # Fuera las motitas sueltas
        if len(part) < min_faces:
            bmesh.ops.delete(bm, geom=part, context="FACES")
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    for poly in me.polygons:
        poly.use_smooth = True
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def _islands(bm):
    seen, out = set(), []
    for f in bm.faces:
        if f in seen:
            continue
        part, st = [], [f]
        seen.add(f)
        while st:
            g = st.pop()
            part.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h not in seen:
                        seen.add(h)
                        st.append(h)
        out.append(part)
    return out


def smooth_borders(bm, passes=8):
    """Alisa los bordes entre colores: el remallado deja escalones (va por la grilla); cada vértice del borde se
    acerca al promedio de sus dos vecinos del borde y se vuelve a apoyar sobre la figura original."""
    bvh = ORIGINAL["bvh"]
    edges = [e for e in bm.edges if len(e.link_faces) == 2
             and e.link_faces[0].material_index != e.link_faces[1].material_index]
    nb = {}
    for e in edges:
        a, b = e.verts
        nb.setdefault(a, []).append(b)
        nb.setdefault(b, []).append(a)
    chain = [v for v, ns in nb.items() if len(ns) == 2]
    for _ in range(passes):
        target = {v: (nb[v][0].co + nb[v][1].co) * 0.5 for v in chain}
        for v in chain:
            q = v.co * 0.4 + target[v] * 0.6
            hit = bvh.find_nearest(q)
            v.co = hit[0] if hit[0] is not None else q


def paint(ob, mats, classify, cuts=(), keep_small=(), min_patch=240):
    """Pinta cara por cara según classify(centro, normal, pieza chica?, delgado()). Antes corta la malla por los
    planos `cuts` (las costuras de la escultura: puños, ruedo, botamangas, borde de la capucha, gorra), cada uno
    como una altura z o como (punto, normal): así esos bordes quedan rectos. Al final, los pedacitos sueltos de un
    color (menos de min_patch caras) toman el color que los rodea; los de `keep_small` (ojos, botones) se respetan."""
    me = ob.data
    for m in mats:
        me.materials.append(m)
    bm = bmesh.new()
    bm.from_mesh(me)
    small_layer = bm.faces.layers.bool.get("small")
    for cut in cuts:
        co, no = ((0, 0, cut), (0, 0, 1)) if isinstance(cut, (int, float)) else cut
        geom = bm.verts[:] + bm.edges[:] + [f for f in bm.faces if not (small_layer and f[small_layer])]
        bmesh.ops.bisect_plane(bm, geom=geom, plane_co=co, plane_no=no)
    bm.faces.ensure_lookup_table()
    bvh = BVHTree.FromBMesh(bm)

    for f in bm.faces:
        c, n = f.calc_center_median(), f.normal.copy()
        small = bool(small_layer and f[small_layer])
        f.material_index = mats.index(classify(c, n, small, lambda c=c, n=n: enclosed(bvh, c, n)))
    for _ in range(2):                                   # Cada cara toma el color que más se repite a su alrededor
        new = {}
        for f in bm.faces:
            votes = [f.material_index] * 2 + [h.material_index for e in f.edges for h in e.link_faces if h is not f]
            new[f] = max(set(votes), key=votes.count)
        for f, k in new.items():
            f.material_index = k
    keep = {mats.index(m) for m in keep_small}
    seen = set()
    for f in bm.faces:
        if f.index in seen:
            continue
        k = f.material_index
        stack, members, around = [f], [], {}
        seen.add(f.index)
        while stack:
            g = stack.pop()
            members.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h.material_index == k:
                        if h.index not in seen:
                            seen.add(h.index)
                            stack.append(h)
                    else:
                        around[h.material_index] = around.get(h.material_index, 0) + 1
        if len(members) < (4 if k in keep else min_patch) and around:
            fill = max(around, key=around.get)
            for g in members:
                g.material_index = fill
    # A voxel remesh can leave isolated islands that still read as speckles in the
    # final render. Repeat the cleanup after the first pass while protecting
    # deliberate small details such as mouths, stripes and ink marks.
    for _ in range(2):
        bm.faces.ensure_lookup_table()
        seen = set()
        for f in bm.faces:
            if f.index in seen or f.material_index in keep:
                continue
            k = f.material_index
            stack, members, around = [f], [], {}
            seen.add(f.index)
            while stack:
                g = stack.pop()
                members.append(g)
                for e in g.edges:
                    for h in e.link_faces:
                        if h.material_index == k:
                            if h.index not in seen:
                                seen.add(h.index)
                                stack.append(h)
                        else:
                            around[h.material_index] = around.get(h.material_index, 0) + 1
            if len(members) < min_patch and around:
                fill = max(around, key=around.get)
                for g in members:
                    g.material_index = fill
    smooth_borders(bm)
    bm.to_mesh(me)
    bm.free()


def ball_faces(ob, centers_xz, radius, axis, sign):
    """Caras de bultos esféricos (ojos de Charlie, botones de Pim): para cada (h, z), busca la parte de la
    superficie más adelantada en `axis` (y para Charlie, x para Pim) y toma lo que queda a menos de un radio de
    ese bulto. Devuelve un conjunto de centros redondeados para reconocerlas en classify."""
    pts = [p.co for p in ob.data.vertices]
    found = []
    other = 0 if axis == 1 else 1
    for h, z in centers_xz:
        col = [p for p in pts if abs(p[other] - h) < radius * 0.5 and abs(p.z - z) < radius * 0.5]
        if not col:
            continue
        front = max(sign * p[axis] for p in col)
        c = Vector((0, 0, z))
        c[other] = h
        c[axis] = sign * (front - radius)
        found.append(c)
    return found


def join(body, extra):
    """Une una pieza (los pelos) a la figura, con sus materiales."""
    bpy.ops.object.select_all(action="DESELECT")
    extra.select_set(True)
    body.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()


def in_ell(p, c, r):
    return sum(((p[k] - c[k]) / r[k]) ** 2 for k in range(3)) < 1.0


def in_balls(c, balls, radius):
    return any((c - b).length < radius * 1.18 for b in balls)


# --- Charlie ----------------------------------------------------------------------------------------------------
# Figura de la serie: cuerpo de frente (-Y), cabeza girada a -X con la nariz larga, saludando con la mano izquierda
# en alto y la boca abierta, gorra y pelitos atrás. Las medidas salen de vistas ortogonales con grilla de la figura
# (coordenadas del STL).
def build_charlie():
    SKIN, HOOD, PANTS, SHOE, CAP = flat("CharlieSkin", "#f6d62e"), flat("CharlieHoodie", "#f2662d"), \
        flat("CharliePants", "#3a3b44", 0.12), flat("CharlieShoe", "#eef0f3", 0.3), flat("CharlieCap", "#8a5a3c")
    INK, WHITE, STRIPE, MOUTH = flat("Ink", "#111014", 0.0), flat("CharlieCollar", "#f4f6f7", 0.3), \
        flat("CharlieStripe", "#3f7fd0"), flat("Mouth", "#5a1824", 0.1)
    ob = load_even(os.path.join(SRC, "Charlie.stl"), 0.0062)
    eyes = ball_faces(ob, [(-0.1246, 0.39), (-0.0325, 0.3655)], 0.0205, 1, -1)
    hand_n, hand_a = Vector((0, 0.697, 0.717)), Vector((0, -0.268, -0.155))      # Puño de la mano de abajo
    wave_n, wave_a = Vector((0.637, 0, -0.770)), Vector((-0.2265, 0, 0.1845))    # Puño de la mano que saluda
    # The hoodie/head seam is deliberately steeper toward the front: the face
    # drops lower over the collar on the front, while the back/shoulders must
    # stay orange instead of inheriting the yellow head color.
    hood_n = Vector((0, -0.20, 1)).normalized()                                # Borde de la capucha (cuello)

    def classify(c, n, small, thin):
        x, y, z = c
        # The collar is a narrow oval neckline in the sculpt.  Classifying
        # only the front-facing triangles left orange/yellow leaks whenever
        # the model was viewed from the side or back, so paint the complete
        # neckline ring first and put the blue stripe in its middle.
        collar_r = ((x - 0.05) / 0.13) ** 2 + ((y + 0.02) / 0.09) ** 2
        if 0.62 < collar_r < 1.38 and 0.04 < z < 0.10:
            return STRIPE if abs(z - 0.066) < 0.006 else WHITE
        if z < -0.462:                                    # Zapatillas (botamanga plana)
            return SHOE
        if x > 0.12 and -0.27 < z < -0.12 and y < -0.17 and hand_n.dot(c - hand_a) < 0:
            return SKIN                                   # Mano de abajo
        if z < -0.183:                                    # Pantalón (ruedo del buzo plano)
            return PANTS
        if x < -0.215 and z < 0.27 and wave_n.dot(c - wave_a) < 0:
            return SKIN                                   # Mano que saluda
        if x < -0.13 and z < 0.25:                        # Manga
            return HOOD
        if x > 0.15 and z < 0.25:                         # Manga del brazo derecho
            return HOOD
        if y > -0.015 and x > -0.13 and z > 0.455 - 0.28 * x:  # Gorra, sin invadir la frente
            return CAP
        if in_balls(c, eyes, 0.0205):                     # Ojos
            return INK
        if -0.15 < x < 0.01 and 0.205 < z < 0.275 and -0.04 < y < 0.2 and thin() > 0.5:
            return MOUTH                                  # Adentro de la boca (lo encerrado entre los labios)
        if y < 0.03 and -0.12 < x < 0.15 and 0.04 + 1.75 * x * x < z < 0.09 + 1.65 * x * x:
            mid = 0.066 + 1.9 * x * x                     # Cuello de la remera, con la raya al medio
            return STRIPE if abs(z - mid) < 0.0055 else WHITE
        if y < 0.03 and -0.12 < x < 0.15 and 0.09 + 1.65 * x * x <= z < 0.2:
            return SKIN                                   # La cabeza llega hasta el cuello de la remera
        neck_r = ((x - 0.05) / 0.16) ** 2 + ((y + 0.02) / 0.12) ** 2
        if neck_r < 1.0 and 0.10 < z < 0.20:
            return SKIN                                   # Cuello amarillo, separado de la manga
        # On the front, the head drops toward the collar.  On the side/back,
        # keep the seam higher so the yellow head cannot spill over the
        # shoulders.  A single diagonal plane was too permissive at oblique
        # angles and was the source of the collar complaints.
        if y < -0.16 and hood_n.dot(c - Vector((0, 0, 0.11))) > 0:
            return SKIN
        if y >= -0.16 and z > 0.18:
            return SKIN
        return HOOD

    cuts = (-0.462, -0.183, 0.04, 0.06, 0.07, 0.10,
            ((0, -0.268, -0.155), tuple(hand_n)), ((-0.2265, 0, 0.1845), tuple(wave_n)),
            ((0, 0, 0.44), (0.3, 0, 1)), ((0, 0, 0.11), tuple(hood_n)), ((-0.13, 0, 0), (1, 0, 0)))
    paint(ob, [SKIN, HOOD, PANTS, SHOE, CAP, INK, WHITE, STRIPE, MOUTH], classify, cuts,
          keep_small=(INK, WHITE, STRIPE, MOUTH))
    join(ob, strands("CharlieHair", INK, ((0.17, 0.4), (-0.2, 0.2), (0.24, 0.42))))
    # Parado sobre z = 0 y del alto de Charlie en la nave
    ob.data.transform(Matrix.Scale(1.12, 4) @ Matrix.Translation((0, 0, 0.53)))
    return ob, 0.007


# --- Pim --------------------------------------------------------------------------------------------------------
# Figura de Pim gritando: mira a +X, con los brazos arriba, la boca abierta enorme y los ojos de espiral arriba de
# la cabeza (las espirales son piezas sueltas).
def build_pim():
    SKIN, EYE, INK, SHIRT = flat("PimSkin", "#f6a8e2"), flat("EyeWhite", "#fbfbfb", 0.35), flat("Ink", "#111014", 0.0), \
        flat("PimShirt", "#f4f6f7", 0.3)
    PANTS, BUCKLE, SHOE, MOUTH = flat("PimPants", "#6b4fe0"), flat("PimBuckle", "#f6e27a"), \
        flat("Shoe", "#3a3236", 0.1), flat("Mouth", "#8f2146", 0.2)
    TONGUE = flat("Tongue", "#e87886", 0.25)
    ob = load_even(os.path.join(SRC, "Pim.stl"), 0.0052)
    buttons = ball_faces(ob, [(-0.22, -0.112), (-0.21, -0.162), (-0.205, -0.2)], 0.009, 0, 1)
    left_n, left_a = Vector((0, 0.625, -0.781)), Vector((0, -0.455, -0.05))      # Puño del brazo de atrás (-Y)
    right_n, right_a = Vector((0, 0.707, -0.707)), Vector((0, -0.1, -0.04))      # Puño del brazo de adelante

    def classify(c, n, small, thin):
        x, y, z = c
        if small:                                         # Espirales de las pupilas
            return INK
        if in_ell(c, (0.18, -0.27, 0.35), (0.13, 0.14, 0.10)) \
                or in_ell(c, (0.10, -0.10, 0.37), (0.11, 0.13, 0.10)):
            return EYE                                    # Ojos blancos arriba de la cabeza
        if z < -0.355:                                    # Zapatos
            return SHOE
        if z < -0.265:                                    # Pantalón corto
            return PANTS
        if z < -0.225:                                    # Cinturón y hebilla
            return BUCKLE if -0.24 < y < -0.155 and x > 0.1 else INK
        if y < -0.44 and left_n.dot(c - left_a) < 0:      # Mano de atrás
            return SKIN
        if y > -0.04 and right_n.dot(c - right_a) < 0:    # Mano de adelante (y la cabeza de ese lado)
            return SKIN
        if ((y + 0.22) / 0.13) ** 2 + ((z - 0.10) / 0.12) ** 2 < 1 and -0.08 < x < 0.08:
            # Adentro de la boca; la lengua es el óvalo de abajo, como en la escultura
            return TONGUE if ((y + 0.24) / 0.075) ** 2 + ((z - 0.035) / 0.05) ** 2 < 1 else MOUTH
        if in_balls(c, buttons, 0.009):                   # Botones
            return INK
        if -0.047 < z < 0.08 and (y < -0.33 or y > -0.12):
            return SHIRT                                  # Mangas blancas; las manos ya se resolvieron arriba
        if (-0.02 < z < 0.14 and x < -0.12 and -0.09 < y < 0.05) or \
                (0.02 < z < 0.18 and -0.115 < x < -0.07 and -0.25 < y < -0.13):
            return SHIRT                                  # Tramos interiores de las mangas
        if z > -0.047:                                    # Piel y brazos; la camiseta termina en el cuello
            return SKIN
        return SHIRT

    cuts = (-0.355, -0.265, -0.225, -0.047, ((0, -0.455, -0.05), tuple(left_n)), ((0, -0.1, -0.04), tuple(right_n)))
    # Keep the tongue as an intentional small detail, but merge isolated
    # maroon mouth specks into their surrounding cavity so the open mouth
    # remains one clean, bounded shape.
    paint(ob, [SKIN, EYE, INK, SHIRT, PANTS, BUCKLE, SHOE, MOUTH, TONGUE], classify, cuts,
          keep_small=(INK, BUCKLE, TONGUE))
    join(ob, strands("PimHair", INK, ((-0.4, -0.12), (-0.4, 0.2), (0.05, 0.23))))
    # Centrado, parado sobre z = 0 y más chico que Charlie
    ob.data.transform(Matrix.Scale(0.9, 4) @ Matrix.Translation((-0.01, 0.255, 0.45)))
    return ob, 0.006


def present(ob, name, height):
    """Guarda el archivo del personaje con luces y cámara, y capturas de frente, costado, atrás y tres cuartos."""
    sc = bpy.context.scene
    pres = bpy.data.collections.new("Presentation")
    sc.collection.children.link(pres)
    w = bpy.data.worlds.new(name + "_World")
    w.use_nodes = True
    w.node_tree.nodes["Background"].inputs["Color"].default_value = (*srgb("#7f7f7f"), 1)
    sc.world = w
    for lname, loc, e in (("Key", (-3, -4, 4), 700), ("Fill", (4, -2, 2), 300), ("Back", (0, 4, 3), 300)):
        ld = bpy.data.lights.new(lname, "AREA")
        ld.energy, ld.size = e, 4
        lo = bpy.data.objects.new(lname, ld)
        pres.objects.link(lo)
        lo.location = loc
        lo.rotation_euler = (Vector((0, 0, height / 2)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
    pres.objects.link(cam)
    sc.camera = cam
    sc.render.engine = "BLENDER_EEVEE"
    sc.render.resolution_x, sc.render.resolution_y = 900, 1000
    sc.view_settings.view_transform = "Standard"
    for screen in bpy.data.screens:
        for area in screen.areas:
            for space in area.spaces:
                if space.type == "VIEW_3D":
                    space.shading.show_backface_culling = True
                    space.shading.color_type = "MATERIAL"
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(CHAR_DIR, name + ".blend"))
    target = Vector((0, 0, height * 0.5))
    shots = (("front", (1, 0, 0.1)), ("side", (0, -1, 0.1)), ("back", (-1, 0, 0.1)),
             ("three_quarter", (0.75, -0.7, 0.3)), ("face", (0.9, -0.45, 0.25)),
             ("audience", (1, 0, 0.1))) if name == "pim" else \
            (("front", (0, -1, 0.1)), ("side", (1, 0, 0.1)), ("back", (0, 1, 0.1)),
             ("three_quarter", (-0.7, -0.75, 0.3)), ("face", (0.75, -0.65, 0.25)),
             ("audience", (1, 0, 0.1)))
    for shot, d in shots:
        cam.location = target + Vector(d).normalized() * height * 2.6
        cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
        sc.render.filepath = os.path.join(REF_DIR, "%s_%s.png" % (name, shot))
        bpy.ops.render.render(write_still=True)


for name, builder, height in (("charlie", build_charlie, 1.2), ("pim", build_pim, 0.81)):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.name = "SmilingFriends_" + name.capitalize()
    ob, outline = builder()
    ob.name = ob.data.name = "SF_" + name.capitalize()
    add_outline(ob, outline)
    ob.data.calc_loop_triangles()
    print("FIGURE %s tris %d" % (name, len(ob.data.loop_triangles)))
    present(ob, name, height)
print("DONE")
