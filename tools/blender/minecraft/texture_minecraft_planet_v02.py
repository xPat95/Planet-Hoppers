"""Aplicar recursos locales al modelo existente: solo materiales y UV.

Ejecutar con __file__ definido. Requiere el planeta v01 abierto y una copia previa.
No reconstruye mallas, no exporta y no guarda el .blend automáticamente.
"""
import bpy
import hashlib
import json
import struct
from pathlib import Path
from mathutils import Vector

REPOSITORY = Path(__file__).resolve().parents[3]
MODEL = REPOSITORY / 'assets' / 'models' / 'planets' / 'minecraft'
TEXTURES = REPOSITORY / 'assets' / 'textures' / 'planets' / 'minecraft'
DIAGNOSTICS = REPOSITORY / 'docs' / 'references' / 'minecraft' / 'diagnostics'
root = bpy.data.objects.get('Minecraft_Planet_ROOT')
if root is None:
    raise RuntimeError('No existe Minecraft_Planet_ROOT')
objects = [o for o in root.children_recursive if o.type == 'MESH']

def geometry_signature():
    result = {}
    for obj in objects:
        digest = hashlib.sha256()
        for vertex in obj.data.vertices:
            digest.update(struct.pack('<3f', *vertex.co))
        for poly in obj.data.polygons:
            digest.update(struct.pack('<%di' % len(poly.vertices), *poly.vertices))
        result[obj.name] = {
            'geometry': digest.hexdigest(),
            'matrix': [list(row) for row in obj.matrix_local],
            'parent': obj.parent.name if obj.parent else None,
            'collections': sorted(c.name for c in obj.users_collection),
        }
    return result

before = geometry_signature()
original_hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in TEXTURES.glob('*.png') if p.name != 'water_static_frame_00.png'}
images = {}
for path in TEXTURES.glob('*.png'):
    images[path.stem] = bpy.data.images.load(str(path), check_existing=True)

# Blender almacena filas desde abajo; el primer frame de la tira está arriba.
water_source = images.get('water_still') or images.get('water')
if water_source:
    width, height = water_source.size
    if height % width != 0:
        raise RuntimeError('La imagen de agua no es una tira de cuadros cuadrados')
    pixels = list(water_source.pixels)
    frame_path = TEXTURES / 'water_static_frame_00.png'
    if frame_path.exists():
        water_frame = bpy.data.images.load(str(frame_path), check_existing=True)
    else:
        water_frame = bpy.data.images.new('water_static_frame_00', width=width, height=width, alpha=True)
        start = (height-width)*width*4
        water_frame.pixels[:] = pixels[start:start+width*width*4]
        water_frame.filepath_raw = str(frame_path)
        water_frame.file_format = 'PNG'
        water_frame.save()
    images['water_frame'] = water_frame

def linear_hex(value):
    channels = [int(value[i:i+2],16)/255 for i in (0,2,4)]
    return tuple(v/12.92 if v <= 0.04045 else ((v+0.055)/1.055)**2.4 for v in channels) + (1,)

created = {}
missing = []
def make(name, image_key, tint=None, alpha=False, roughness=0.85):
    image = images.get(image_key)
    if image is None:
        missing.append(image_key)
        return None
    mat = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    nodes.clear()
    output = nodes.new('ShaderNodeOutputMaterial')
    output.location = (600,40)
    shader = nodes.new('ShaderNodeBsdfPrincipled')
    shader.location = (330,40)
    shader.inputs['Roughness'].default_value = roughness
    tex = nodes.new('ShaderNodeTexImage')
    tex.location = (-500,40)
    tex.image = image
    tex.interpolation = 'Closest'
    tex.extension = 'REPEAT'
    uv = nodes.new('ShaderNodeUVMap')
    uv.uv_map = 'MC_BlockUV'
    uv.location = (-730,40)
    links = mat.node_tree.links
    links.new(uv.outputs['UV'], tex.inputs['Vector'])
    color = tex.outputs['Color']
    if tint:
        multiply = nodes.new('ShaderNodeMixRGB')
        multiply.blend_type = 'MULTIPLY'
        multiply.inputs[0].default_value = 1
        multiply.inputs[2].default_value = linear_hex(tint)
        multiply.label = 'Tinte de bioma ajustable #' + tint
        multiply.location = (-100,40)
        links.new(color,multiply.inputs[1])
        color = multiply.outputs['Color']
        mat.diffuse_color = linear_hex(tint)
    links.new(color,shader.inputs['Base Color'])
    if alpha:
        links.new(tex.outputs['Alpha'],shader.inputs['Alpha'])
        mat['alpha'] = 'Canal alfa original conectado; transparencia del material conservada.'
    links.new(shader.outputs['BSDF'],output.inputs['Surface'])
    mat['texture_filter'] = 'Closest; UV repetidas a escala de bloque'
    created[name] = mat
    return mat

grass = make('MC_Grass_Top','grass_block_top','7CBD6B')
# Este recurso ya tiene la franja de césped coloreada: conservarla y no teñir tierra.
side = make('MC_Grass_Side','grass_block_side')
dirt = make('MC_Dirt','dirt')
stone = make('MC_Stone','stone')
wood = make('MC_Oak_Log','oak_log')
leaves = make('MC_Oak_Leaves','oak_leaves','58A345',alpha=True)
sand = make('MC_Sand','sand')
snow = make('MC_Snow','snow')
water = make('MC_Water','water_frame','3F95D0',roughness=0.48)
deep = make('MC_Water_Deep','water_frame','2669B1',roughness=0.48)
shallow = make('MC_Water_Shallow','water_frame','4BA9D7',roughness=0.48)

axes = {
    'Top': ((1,0,0),(0,1,0),(0,0,1)),
    'Front': ((1,0,0),(0,0,1),(0,-1,0)),
    'Right': ((0,1,0),(0,0,1),(1,0,0)),
    'Back': ((-1,0,0),(0,0,1),(0,1,0)),
    'Left': ((0,-1,0),(0,0,1),(-1,0,0)),
    'Bottom': ((1,0,0),(0,-1,0),(0,0,-1)),
}

for obj in objects:
    mesh = obj.data
    previous = mesh.materials[0] if mesh.materials else None
    old = previous.name if previous else ''
    is_grass = old.startswith('MC_Pasto')
    mapping = {'MC_Tierra': dirt, 'MC_Piedra': stone, 'MC_Piedra_Clara': stone,
               'MC_Madera': wood, 'MC_Hojas': leaves, 'MC_Hojas_Claras': leaves,
               'MC_Arena': sand, 'MC_Nieve': snow, 'MC_Agua': water,
               'MC_Agua_Profunda': deep, 'MC_Agua_Clara': shallow}
    replacement = grass if is_grass else mapping.get(old)
    if not replacement:
        continue
    mesh.materials.clear()
    mesh.materials.append(replacement)
    if is_grass:
        mesh.materials.append(side or previous)
        mesh.materials.append(dirt or previous)
    face_name = next((key for key in axes if key in obj.name.split('_')), 'Top')
    # Cascada se construyó en ejes globales; sus UV usan ese mismo marco.
    if obj.name.startswith('Waterfall'):
        face_name = 'Top'
    u,v,n = [Vector(a) for a in axes[face_name]]
    uv = mesh.uv_layers.get('MC_BlockUV') or mesh.uv_layers.new(name='MC_BlockUV')
    for poly in mesh.polygons:
        coords = [mesh.vertices[index].co for index in poly.vertices]
        normal = poly.normal
        dn,du,dv = normal.dot(n), normal.dot(u), normal.dot(v)
        if abs(dn)>0.5:
            horizontal,vertical = u,v
        elif abs(du)>0.5:
            horizontal,vertical = v,n
        else:
            horizontal,vertical = u,n
        low_u = min(p.dot(horizontal) for p in coords)
        low_v = min(p.dot(vertical) for p in coords)
        high_n = max(p.dot(n) for p in coords)
        if is_grass:
            poly.material_index = 0 if dn>0.5 else (2 if dn < -0.5 else 1)
        for loop_index in poly.loop_indices:
            p = mesh.vertices[mesh.loops[loop_index].vertex_index].co
            tx,ty = p.dot(horizontal)-low_u,p.dot(vertical)-low_v
            if is_grass and abs(dn)<0.5:
                ty = 1 - (high_n-p.dot(n))
            # Tronco: una anchura de corteza; vertical paralela a su eje local.
            if old == 'MC_Madera' and abs(dn)<0.5:
                width = max(q.dot(horizontal) for q in coords)-low_u
                tx /= width
            uv.data[loop_index].uv = (tx,ty)
    mesh.update()

after = geometry_signature()
assert before == after, 'Se detecto un cambio de geometria, jerarquia o transformaciones'
for name,digest in original_hashes.items():
    assert hashlib.sha256((TEXTURES/name).read_bytes()).hexdigest() == digest, name

report = {
    'geometry_and_hierarchy_unchanged': before == after,
    'mesh_count': len(objects),
    'vertices': sum(len(o.data.vertices) for o in objects),
    'faces': sum(len(o.data.polygons) for o in objects),
    'materials': sorted(created),
    'missing_textures': missing,
    'water_source': [water_source.name, list(water_source.size)] if water_source else None,
    'water_frame': 'Primer fotograma superior, 16x16; sin animacion; alpha del shader 1.',
    'grass_side': 'Textura ya coloreada; se conserva sin tinte para no alterar tierra.',
    'missing_log_end': 'No existe oak_log_top; se reutiliza oak_log en los extremos.',
    'original_texture_sha256': original_hashes,
    'geometry_signatures': after,
}
DIAGNOSTICS.mkdir(parents=True, exist_ok=True)
(DIAGNOSTICS/'texturing_v02_verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
for image in set(node.image for mat in created.values() for node in mat.node_tree.nodes
                 if node.type == 'TEX_IMAGE' and node.image):
    image.pack()
print('Texturizado completo:',len(objects),'mallas;',len(created),'materiales; geometria intacta.')
print('Texturas faltantes:',missing)
