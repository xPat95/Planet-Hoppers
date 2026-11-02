"""Exporta a GLB los planetas y naves que todavía no tenían versión para el juego (sistema solar, fase 3).

Las naves de South Park, Un Show Más y Star Wars ya tienen sus exportaciones (scripts de cada franquicia); aquí no se
tocan.

Uso (desde la raíz del repositorio):
    blender -b --factory-startup --python tools/blender/solarsystem/export_solar_models.py
    blender -b --factory-startup --python tools/blender/solarsystem/export_solar_models.py -- halloween smilingfriends_ship

Abre cada .blend, toma todo lo que cuelga de su raíz (`*_ROOT`) y lo exporta, sin guardar nunca el .blend: los
archivos originales no cambian. Los tintes que glTF no soporta (Minecraft) se hornean en copias de las imágenes. Se dejan afuera la cámara y las luces (`Presentation`) y las versiones lejanas
(`LOD_Far`), salvo en la exportación lejana, que usa solo eso. El contorno de caricatura (modificador `Outline`) se
apaga, salvo en la nave y el planeta de Smiling Friends, que lo llevan como geometría (`SF_Outline`) y el juego lo dibuja. Ejes: Y arriba, como las demás exportaciones del repositorio.
"""

import bpy
import numpy as np
import os
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))

# id: (archivo .blend, raíz, archivo de salida, colecciones que se usan: "near" = todo menos LOD_Far, "far" = solo
# LOD_Far; y si conserva el contorno de caricatura como geometría)
EXPORTS = {
    "minecraft": ("assets/models/planets/minecraft/minecraft_planet_textured_v02.blend", "Minecraft_Planet_ROOT",
                  "assets/models/planets/minecraft/exports/minecraft_planet.glb", "near"),
    "southpark": ("assets/models/planets/southpark/southpark_planet.blend", "SouthPark_Planet_ROOT",
                  "assets/models/planets/southpark/exports/southpark_planet.glb", "near"),
    "smilingfriends": ("assets/models/planets/smilingfriends/smilingfriends_planet.blend",
                       "SmilingFriends_Planet_ROOT",
                       "assets/models/planets/smilingfriends/exports/smilingfriends_planet.glb", "near", True),
    "halloween": ("assets/models/planets/halloween/halloween_planet.blend", "Halloween_Planet_ROOT",
                  "assets/models/planets/halloween/exports/halloween_planet.glb", "near"),
    "halloween_far": ("assets/models/planets/halloween/halloween_planet.blend", "Halloween_Planet_ROOT",
                      "assets/models/planets/halloween/exports/halloween_planet_far.glb", "far"),
    "smilingfriends_ship": ("assets/models/ships/smilingfriends/smilingfriends_ship.blend",
                            "SmilingFriends_Ship_ROOT",
                            "assets/models/ships/smilingfriends/exports/smilingfriends_ship.glb", "near", True),
    "minecraft_ship": ("assets/models/ships/minecraft/end_ship.blend", "EndShip_ROOT",
                       "assets/models/ships/minecraft/exports/end_ship.glb", "near"),
}


def under(ob, root):
    while ob:
        if ob == root:
            return True
        ob = ob.parent
    return False


def in_collection(ob, word):
    """¿Está en una colección cuyo nombre contiene `word` (sin importar mayúsculas)?"""
    return any(word.lower() in c.name.lower() for c in ob.users_collection)


def bake_multiply_tints():
    """Minecraft tiñe sus texturas grises (pasto, hojas, agua) con un nodo Mezclar en modo Multiplicar, que glTF no
    puede guardar: sin esto se exportan grises. Se hornea el tinte en una copia de cada imagen (solo en memoria)."""
    for mat in bpy.data.materials:
        if not mat.use_nodes:
            continue
        nodes, links = mat.node_tree.nodes, mat.node_tree.links
        for mix in [n for n in nodes if n.type == "MIX_RGB" and n.blend_type == "MULTIPLY"]:
            fac, a, b = mix.inputs[0], mix.inputs[1], mix.inputs[2]
            tex_in, color_in = (a, b) if a.is_linked else (b, a)
            src = tex_in.links[0].from_node if tex_in.is_linked else None
            if src is None or src.type != "TEX_IMAGE" or color_in.is_linked or fac.is_linked or not src.image:
                continue
            img = src.image
            tint = np.array(color_in.default_value[:3], dtype=np.float32)
            px = np.empty(len(img.pixels), dtype=np.float32)
            img.pixels.foreach_get(px)
            px = px.reshape(-1, 4)
            srgb = img.colorspace_settings.name == "sRGB"
            lin = np.where(px[:, :3] <= 0.04045, px[:, :3] / 12.92, ((px[:, :3] + 0.055) / 1.055) ** 2.4) if srgb else px[:, :3]
            lin = lin * (1 - fac.default_value) + lin * tint * fac.default_value      # Multiplicar en espacio lineal
            out = np.where(lin <= 0.0031308, lin * 12.92, 1.055 * np.power(lin, 1 / 2.4) - 0.055) if srgb else lin
            px[:, :3] = np.clip(out, 0, 1)
            baked = bpy.data.images.new(img.name + "_tinted", img.size[0], img.size[1], alpha=True)
            baked.colorspace_settings.name = img.colorspace_settings.name
            baked.pixels.foreach_set(px.ravel())
            baked.pack()
            src.image = baked
            for link in list(mix.outputs[0].links):              # La textura va directo adonde iba la mezcla
                links.new(src.outputs["Color"], link.to_socket)
            nodes.remove(mix)


def export(key):
    blend, root_name, out, part, *flags = EXPORTS[key]
    keep_outline = bool(flags and flags[0])
    bpy.ops.wm.open_mainfile(filepath=os.path.join(REPO, blend))
    root = bpy.data.objects[root_name]
    # Algunos archivos traen otra escena con el cubo por defecto: se borra (el .blend no se guarda) para que el
    # exportador solo vea la escena del modelo
    home = next(sc for sc in bpy.data.scenes if root.name in sc.objects)
    for sc in list(bpy.data.scenes):
        if sc != home:
            bpy.data.scenes.remove(sc)
    bpy.context.window_manager.windows[0].scene = home
    keep = []
    for ob in bpy.context.scene.objects:
        if not under(ob, root) or ob.type in {"CAMERA", "LIGHT"} or in_collection(ob, "presentation"):
            continue
        # The abduction beam is an optional effect, not part of the ship's visible hull.
        # Including it also makes fit-to-bounds previews tiny and lifts the saucer off its platform.
        if key == "smilingfriends_ship" and ob.name == "SF_Ship_Beam":
            continue
        far = in_collection(ob, "lod_far")
        if (part == "far") != far and ob != root:
            continue
        if part == "far" and ob.type == "MESH" or part == "near":
            keep.append(ob)
    bake_multiply_tints()
    for ob in bpy.context.scene.objects:
        ob.hide_set(False)
        ob.hide_viewport = False
        ob.select_set(False)
    for c in bpy.data.collections:                        # Lo oculto (LOD_Far) también tiene que poder exportarse
        c.hide_viewport = False
    for layer_col in bpy.context.view_layer.layer_collection.children:
        layer_col.exclude = False
        layer_col.hide_viewport = False
    for ob in keep:
        ob.select_set(True)
        if ob.name in ("SF_Pim", "SF_Charlie"):           # Figuras de caras parejas (para pintar): más livianas en el juego
            dec = ob.modifiers.new("Decimate", "DECIMATE")
            dec.ratio = 0.4
            bpy.context.view_layer.objects.active = ob
            with bpy.context.temp_override(object=ob):
                bpy.ops.object.modifier_move_to_index(modifier=dec.name, index=0)
        for m in getattr(ob, "modifiers", []):
            if m.name == "Outline" and not keep_outline:  # Contorno solo de vista (salvo donde el juego lo dibuja)
                m.show_viewport = m.show_render = False
    if keep_outline:
        # El contorno va como geometría: casco invertido negro. El juego lo dibuja descartando las caras de
        # adelante, así solo se ve el borde. Su material (con nodos que glTF no entiende) pasa a negro liso.
        ol = bpy.data.materials.get("SF_Outline")
        if ol and ol.use_nodes:
            nt = ol.node_tree
            bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
            nt.links.new(bsdf.outputs[0], next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL").inputs["Surface"])
    path = os.path.join(REPO, out)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True, export_apply=True,
                              export_yup=True, export_cameras=False, export_lights=False, use_active_scene=True, export_animations=False,
                              export_extras=False)
    tris = 0
    deps = bpy.context.evaluated_depsgraph_get()
    for ob in keep:
        if ob.type == "MESH":
            me = ob.evaluated_get(deps).to_mesh()
            me.calc_loop_triangles()
            tris += len(me.loop_triangles)
            ob.evaluated_get(deps).to_mesh_clear()
    print("EXPORT %-20s %8d tris  %6.1f MB  %s" % (key, tris, os.path.getsize(path) / 1e6, out))


wanted = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else list(EXPORTS)
for key in wanted:
    export(key)
