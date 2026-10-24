"""Blockout editable de Planet Hoppers. Ejecutar desde el editor de texto de Blender.

Crea una escena nueva y conserva las escenas existentes. No instala dependencias.
El guardado se realiza por separado para no sobrescribir archivos al reutilizarlo.
"""
import math
import random
import bpy
from mathutils import Vector

SEED = 41
GRID = 24
HALF = GRID / 2
rng = random.Random(SEED)
scene = bpy.data.scenes.new('Minecraft_Planet_Concept_v01')
bpy.context.window.scene = scene

def collection(name):
    result = bpy.data.collections.new(name)
    scene.collection.children.link(result)
    return result

groups = {name: collection(name) for name in
          ('Planet_Base', 'Terrain', 'Water', 'Trees', 'Mountain', 'Presentation')}
root = bpy.data.objects.new('Minecraft_Planet_ROOT', None)
groups['Planet_Base'].objects.link(root)
root['concept'] = 'Tierra cubica voxel / Planet Hoppers / primera version editable'
root['seed'] = SEED
root['export'] = 'Exportar jerarquia de este objeto; excluir Presentation.'

def material(name, color, roughness=0.85):
    mat = bpy.data.materials.new('MC_' + name)
    mat.diffuse_color = (*color, 1)
    mat.use_nodes = True
    node = next(n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    node.inputs['Base Color'].default_value = (*color, 1)
    node.inputs['Roughness'].default_value = roughness
    return mat

mats = {
    'Grass': material('Pasto', (0.24, 0.52, 0.065)),
    'GrassLight': material('Pasto_Claro', (0.34, 0.64, 0.085)),
    'GrassDark': material('Pasto_Oscuro', (0.16, 0.39, 0.045)),
    'Dirt': material('Tierra', (0.30, 0.145, 0.06)),
    'Stone': material('Piedra', (0.34, 0.39, 0.42)),
    'StoneLight': material('Piedra_Clara', (0.46, 0.50, 0.52)),
    'Water': material('Agua', (0.018, 0.24, 0.50), 0.3),
    'WaterLight': material('Agua_Clara', (0.025, 0.33, 0.59), 0.3),
    'WaterDark': material('Agua_Profunda', (0.018, 0.19, 0.40), 0.3),
    'Sand': material('Arena', (0.77, 0.65, 0.36)),
    'Snow': material('Nieve', (0.87, 0.94, 0.98)),
    'Wood': material('Madera', (0.24, 0.105, 0.035)),
    'Leaves': material('Hojas', (0.12, 0.35, 0.035)),
    'LeavesLight': material('Hojas_Claras', (0.24, 0.48, 0.045)),
}

# Cada grupo contiene bloques reales editables. No hay imagenes ni displacement.
# Las bases locales son dextrógiras: u x v = normal exterior.
frames = {
    'Top': ((1,0,0), (0,1,0), (0,0,1)),
    'Front': ((1,0,0), (0,0,1), (0,-1,0)),
    'Right': ((0,1,0), (0,0,1), (1,0,0)),
    'Back': ((-1,0,0), (0,0,1), (0,1,0)),
    'Left': ((0,-1,0), (0,0,1), (-1,0,0)),
    'Bottom': ((1,0,0), (0,-1,0), (0,0,-1)),
}
batches = {}

def block(group, name, mat, center, size, frame=None):
    key = (group, name, mat)
    verts, faces = batches.setdefault(key, ([], []))
    offset = len(verts)
    u, v, n = (Vector(axis) for axis in frames[frame]) if frame else (
        Vector((1,0,0)), Vector((0,1,0)), Vector((0,0,1)))
    center = Vector(center)
    for x,y,z in ((-1,-1,-1), (1,-1,-1), (1,1,-1), (-1,1,-1),
                  (-1,-1,1), (1,-1,1), (1,1,1), (-1,1,1)):
        verts.append(tuple(center + u*x*size[0]/2 + v*y*size[1]/2 + n*z*size[2]/2))
    for indices in ((3,2,1,0), (0,1,5,4), (1,2,6,5), (2,3,7,6), (3,0,4,7), (4,5,6,7)):
        faces.append(tuple(offset + i for i in indices))

def surface_block(group, name, mat, face, a, b, height, depth, width=1, length=1):
    u,v,n = (Vector(axis) for axis in frames[face])
    center = u*a + v*b + n*(HALF + height - depth/2)
    block(group, name, mat, center, (width, length, depth), face)

def ellipse(x, y, cx, cy, rx, ry):
    return 1 - ((x-cx)/rx)**2 - ((y-cy)/ry)**2

def continent(face, x, y):
    if face == 'Top':
        shapes = [(-7,5,7.5,8), (7,5,6,8), (-8,-7,4.6,4), (3,-8,3.3,2.2)]
    elif face == 'Front':
        shapes = [(-6,5,5,6), (-4,-5,4.3,4), (8,2,4.7,8)]
    elif face == 'Right':
        shapes = [(-4,5,7,5), (4,-2,6,5), (6,-8,3,3)]
    elif face == 'Back':
        shapes = [(-6,-1,4,8), (5,4,6,5), (6,-7,3,2)]
    elif face == 'Left':
        shapes = [(-5,5,5,4), (3,-1,6,6), (-5,-7,3,3)]
    else:
        shapes = [(-5,1,5,7), (6,3,4,5), (3,-7,4,2)]
    value = max(ellipse(x,y,*shape) for shape in shapes)
    return value + 0.10*math.sin(x*1.3+y*0.7) + 0.06*math.cos(y*1.9-x)

heights = {}
block('Planet_Base', 'Planet_Core_Cube', 'Stone', (0,0,0), (23.90,23.90,23.90))
for face in frames:
    heights[face] = {}
    for i in range(GRID):
        for j in range(GRID):
            x,y = i-HALF+0.5, j-HALF+0.5
            value = continent(face,x,y)
            water_mat = rng.choices(['Water','WaterLight','WaterDark'], [8,1,1])[0]
            surface_block('Water', 'Ocean_' + face, water_mat, face, x,y,0,0.12)
            if value <= 0:
                continue
            height = 0.55 + 0.5*min(3, int(value*3.3))
            heights[face][(i,j)] = height
            surface_block('Terrain', 'Soil_' + face, 'Dirt', face,x,y,height-0.18,height-0.18)
            grass = rng.choices(['Grass','GrassLight','GrassDark'],[7,2,1])[0]
            cap = 'Sand' if value < 0.16 else grass
            surface_block('Terrain','Surface_' + face,cap,face,x,y,height,0.18)
            if face == 'Top' and y > 3 and -3 < x < 6:
                mountain = max(0, int(5 - max(abs(x-1)*0.85,abs(y-7)*0.9)))
                if mountain:
                    surface_block('Mountain','Rock_Terraces','Stone' if (i+j)%3 else 'StoneLight',face,x,y,height+mountain,mountain)
                    if mountain >= 3:
                        surface_block('Mountain','Snow_Caps','Snow',face,x,y,height+mountain+0.2,0.2)
                    heights[face][(i,j)] = -1  # reservar montaña sin árboles

# Arboles por cara y numero; cada tronco y copa se puede editar por separado.
tree_count = 0
for face in frames:
    if face == 'Bottom':
        continue
    candidates = [(ij,h) for ij,h in heights[face].items() if h >= 1.05 and
                  abs(ij[0]-11.5) < 10 and abs(ij[1]-11.5) < 10]
    rng.shuffle(candidates)
    placed = []
    for (i,j),h in candidates:
        x,y = i-HALF+0.5,j-HALF+0.5
        if any((x-a)**2+(y-b)**2 < 13 for a,b in placed):
            continue
        if len(placed) >= (13 if face == 'Top' else 4):
            break
        placed.append((x,y))
        tree_count += 1
        label = 'Tree_%s_%02d' % (face,len(placed))
        surface_block('Trees',label+'_Trunk','Wood',face,x,y,h+1.8,1.8,0.45,0.45)
        surface_block('Trees',label+'_Canopy','Leaves',face,x,y,h+2.7,1.4,1.9,1.9)
        surface_block('Trees',label+'_Canopy','LeavesLight',face,x-0.3,y+0.2,h+3.3,0.7,1.3,1.3)

# Cascada escalonada desde el borde del océano superior hasta la cara frontal.
for step in range(9):
    z = 11.5-step
    block('Water','Waterfall_Front','WaterLight', (1.5,-12.18-step*0.035,z), (1.7,0.30,1.02))
block('Water','Waterfall_Pool','WaterLight', (1.5,-12.48,2.95), (3.0,0.48,0.65))

for (group,name,mat), (verts,faces) in batches.items():
    mesh = bpy.data.meshes.new(name + '_' + mat + '_Mesh')
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name + '_' + mat, mesh)
    groups[group].objects.link(obj)
    obj.data.materials.append(mats[mat])
    obj.parent = root
    obj['component'] = group
    obj['editable'] = 'Bloques reales; modo Edicion para modificar bloques individuales.'

# Presentación técnica separada del modelo; sin escenario ni elementos extra.
world = bpy.data.worlds.new('Minecraft_Deep_Space')
world.use_nodes = True
background = next(n for n in world.node_tree.nodes if n.type == 'BACKGROUND')
background.inputs['Color'].default_value = (0.018,0.027,0.05,1)
background.inputs['Strength'].default_value = 0.35
scene.world = world

def point_at(obj, target=(0,0,2)):
    obj.rotation_euler = (Vector(target)-obj.location).to_track_quat('-Z','Y').to_euler()

camera_data = bpy.data.cameras.new('Planet_Overview')
camera = bpy.data.objects.new('Planet_Overview',camera_data)
groups['Presentation'].objects.link(camera)
camera.location = (39,-49,38)
point_at(camera)
camera_data.type = 'ORTHO'
camera_data.ortho_scale = 48
scene.camera = camera
for name,location,power,size in (
    ('Key', (4,-20,38),6500,20),
    ('Fill', (28,-5,12),4200,18),
    ('Front_Fill', (-22,-32,8),7000,22),
    ('Rim', (-15,20,24),5500,15),
):
    data = bpy.data.lights.new('Planet_' + name,'AREA')
    data.energy, data.shape, data.size = power,'DISK',size
    obj = bpy.data.objects.new('Planet_' + name,data)
    groups['Presentation'].objects.link(obj)
    obj.location = location
    point_at(obj)

scene.render.resolution_x = 1100
scene.render.resolution_y = 1100
scene.render.resolution_percentage = 100
scene['asset_status'] = 'Concepto v01; sin texturas finales, colisiones ni LOD.'
for area in bpy.context.screen.areas:
    if area.type == 'VIEW_3D':
        space = area.spaces.active
        space.shading.type = 'MATERIAL'
        space.region_3d.view_rotation = camera.rotation_euler.to_quaternion()
        space.region_3d.view_distance = 57
        space.region_3d.view_location = (0,0,2)
        space.overlay.show_floor = False
        space.overlay.show_axis_x = False
        space.overlay.show_axis_y = False
bpy.context.view_layer.objects.active = root
root.select_set(True)
print('Creado:',scene.name,'| arboles:',tree_count,'| objetos:',len(scene.objects))
