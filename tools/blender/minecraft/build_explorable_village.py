"""DIAMOND RUSH phase 1. Run with Blender --background --python-exit-code 1 --python.

Only derived Minecraft minigame files are written. Original assets are read-only.
"""
from pathlib import Path
from collections import defaultdict
import hashlib
import json
import math
import random
import struct
import sys
import types
import bpy
from mathutils import Vector

REPO = Path(__file__).resolve().parents[3]
OUT = REPO/'assets/models/minigames/minecraft'
REPORT = REPO/'docs/references/minecraft/phase1'
TEXTURES = REPO/'assets/textures/minigames/Minecraft'
MATS = {}
GROUPS = {}
COLLIDERS = []
ROADS = []
LANDMARKS = []
TEXTURE_AUDIT = []
FACE_INDICES = [(0,4,6,2),(1,3,7,5),(0,1,5,4),(2,6,7,3),(0,2,3,1),(4,5,7,6)]
# Face order: -X,+X,-Y,+Y,-Z,+Z. Outward winding, no smooth shading.


class Batch:
    def __init__(self, name, category):
        self.name, self.category = name, category
        self.vertices, self.faces, self.materials, self.uvs = [], [], [], []

    def box(self, lo, hi, side, top=None, bottom=None, front=None, visible=None):
        corners = [(x,y,z) for z in (lo[2],hi[2]) for y in (lo[1],hi[1]) for x in (lo[0],hi[0])]
        textures = [side,side,front or side,side,bottom or side,top or side]
        for i,indices in enumerate(FACE_INDICES):
            if visible is not None and i not in visible:
                continue
            pts = [corners[j] for j in indices]
            start = len(self.vertices)
            self.vertices.extend(pts)
            self.faces.append(tuple(range(start,start+4)))
            self.materials.append(textures[i])
            # Unit-scale planar mapping: one square texture per world block.
            # Side faces keep the texture's top at the top of each block.
            if i < 2:
                uv = [(p[1]-lo[1],p[2]-lo[2]) for p in pts]
            elif i < 4:
                uv = [(p[0]-lo[0],p[2]-lo[2]) for p in pts]
            else:
                uv = [(p[0]-lo[0],p[1]-lo[1]) for p in pts]
            self.uvs.append(uv)

    def finish(self, collection):
        mesh = bpy.data.meshes.new(self.name)
        mesh.from_pydata(self.vertices, [], self.faces)
        palette = list(dict.fromkeys(self.materials))
        for name in palette:
            mesh.materials.append(MATS[name])
        uv = mesh.uv_layers.new(name='MC_BlockUV')
        for face, mat, coords in zip(mesh.polygons,self.materials,self.uvs):
            face.material_index = palette.index(mat)
            for loop, value in zip(face.loop_indices,coords):
                uv.data[loop].uv = value
        mesh.update()
        obj = bpy.data.objects.new(self.name,mesh)
        collection.objects.link(obj)
        obj['category'] = self.category
        return obj


def material(name, color=None, tint=None, alpha=False):
    mat = bpy.data.materials.new('DR_'+name)
    mat.use_nodes = True
    bsdf = next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    bsdf.inputs['Roughness'].default_value = .88
    if color:
        bsdf.inputs['Base Color'].default_value = (*color,1)
    else:
        path = TEXTURES/(name+'.png')
        image = bpy.data.images.load(str(path),check_existing=True)
        TEXTURE_AUDIT.append({'file':str(path.relative_to(REPO)).replace('\\','/'),
                              'size':list(image.size),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                              'tint':tint,'alpha':'MASK' if alpha else 'OPAQUE'})
        if tint:
            import numpy as np
            pixels = np.array(image.pixels[:],dtype=np.float32).reshape(-1,4)
            pixels[:,:3] *= tint
            tinted = bpy.data.images.new('DR_'+name+'_tinted',width=image.size[0],height=image.size[1],alpha=True)
            tinted.pixels.foreach_set(pixels.ravel())
            tinted.pack()
            image = tinted
        else:
            image.pack()
        node = mat.node_tree.nodes.new('ShaderNodeTexImage')
        node.image, node.interpolation, node.extension = image,'Closest','REPEAT'
        mat.node_tree.links.new(node.outputs['Color'],bsdf.inputs['Base Color'])
        if alpha:
            mat.node_tree.links.new(node.outputs['Alpha'],bsdf.inputs['Alpha'])
            # Blender 5 uses dithered surfaces; GLB explicitly uses alpha MASK.
            if hasattr(mat,'surface_render_method'):
                prop=mat.bl_rna.properties['surface_render_method']
                choices=[i.identifier for i in prop.enum_items]
                if 'DITHERED' in choices: mat.surface_render_method='DITHERED'
            mat.use_backface_culling = False
    MATS[name]=mat


def solid(name,lo,hi,category):
    COLLIDERS.append({'name':name,'shape':'aabb','role':'solid','category':category,'min':list(lo),'max':list(hi)})


def path(name,points,width=3):
    ROADS.append({'name':name,'points':[[x,y,0] for x,y in points],'width':width})


def path_cells():
    result=set()
    for road in ROADS:
        for a,b in zip(road['points'],road['points'][1:]):
            for x in range(-32,32):
                for y in range(-26,34):
                    if min(a[0],b[0])-road['width']/2 <= x+.5 <= max(a[0],b[0])+road['width']/2 and min(a[1],b[1])-road['width']/2 <= y+.5 <= max(a[1],b[1])+road['width']/2:
                        result.add((x,y))
    return result


def house(name,x,y,w,d,height):
    b=Batch(name,'building')
    door_x=x+w//2
    for xx in range(x,x+w):
        for yy in range(y,y+d):
            edge=xx in (x,x+w-1) or yy in (y,y+d-1)
            if not edge: continue
            corner=xx in (x,x+w-1) and yy in (y,y+d-1)
            for z in range(height):
                if yy==y and xx==door_x and z<2:
                    continue
                window=not corner and z==2 and ((yy in (y,y+d-1) and xx in (x+1,x+w-2)) or (xx in (x,x+w-1) and yy==y+d//2))
                mat='oak_log' if corner else 'glass' if window else 'cobblestone' if z==0 else 'oak_planks'
                b.box((xx,yy,z),(xx+1,yy+1,z+1),mat,top='oak_log_top' if corner else None)
    # Two separate door halves, each with full 0..1 UVs on its visible faces.
    for z in range(2):
        b.box((door_x,y+.03,z),(door_x+1,y+.15,z+1),'oak_door_bottom' if z==0 else 'oak_door_top')
    # Stepped gabled roof, half-block risers preserve a recognizable stair silhouette.
    for step in range((w+2)//2+1):
        left,right=x-1+step,x+w+1-step
        if right<=left: break
        b.box((left,y-1,height+step*.5),(left+1,y+d+1,height+(step+1)*.5),'oak_planks')
        if right-1>left:
            b.box((right-1,y-1,height+step*.5),(right,y+d+1,height+(step+1)*.5),'oak_planks')
        if right-left>2:
            for yy in (y,y+d-1):
                b.box((left+1,yy,height+step*.5),(right-1,yy+1,height+(step+1)*.5),'oak_planks')
    b.finish(GROUPS['Buildings'])
    solid(name,(x,y,0),(x+w,y+d,height+1),'building')
    approach=(door_x+.5,y-2)
    start_x=(-4 if x<0 else 4) if -3<=approach[1]<=7 else 0
    path(name+'_approach',[(start_x,approach[1]),approach])
    LANDMARKS.append({'name':name,'category':'building','door':[door_x+.5,y,0],
                      'approach':[*approach,0],'size':[w,d,height]})


def terrain(cells):
    blocks={}
    heights={}
    for x in range(-32,32):
        for y in range(-26,34):
            edge=min(x+32,31-x,y+26,33-y)
            h=max(0,4-edge) if edge<4 else 0
            if y>=26 and 1<=x<=17:
                h=max(h,min(7,6+(y-26)//3,1+min(x-1,17-x)))
            if -30<=x<=-24 and -3<=y<=20: h=max(h,2+(int(y/4)%2))
            heights[x,y]=h
            for z in range(-2,h):
                # Short entrance only: five blocks wide, four high, six deep.
                if 5<=x<=11 and 26<=y<=32 and 0<=z<5: continue
                blocks[x,y,z]='stone' if z<h-2 else 'dirt' if z<h-1 else 'grass'
    chunks={}
    offsets=[(-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)]
    for (x,y,z),kind in blocks.items():
        visible=[i for i,(dx,dy,dz) in enumerate(offsets) if (x+dx,y+dy,z+dz) not in blocks]
        if not visible: continue
        key=(x//8,y//8)
        b=chunks.setdefault(key,Batch('Terrain_%s_%s'%key,'terrain'))
        if kind=='grass':
            is_path=(x,y) in cells and heights[x,y]==0
            b.box((x,y,z),(x+1,y+1,z+1),'dirt_path_side' if is_path else 'grass_block_side',
                  top='dirt_path_top' if is_path else 'grass_block_top',bottom='dirt',visible=visible)
        else:
            b.box((x,y,z),(x+1,y+1,z+1),kind,visible=visible)
    for b in chunks.values(): b.finish(GROUPS['Terrain'])
    # Merge equal-height hill runs per row; do not collide across the mine opening.
    for y in range(-26,34):
        x=-32
        while x<32:
            h=heights[x,y]
            if h==0 or (6<=x<=10 and 25<=y<=31): x+=1; continue
            end=x+1
            while end<32 and heights[end,y]==h and not (6<=end<=10 and 25<=y<=31): end+=1
            solid('Hill_%s_%s'%(x,y),(x,y,0),(end,y+1,h),'hill_boundary')
            x=end
    # Merge matching neighbouring strips into larger conservative boxes.
    strips=defaultdict(list)
    for c in COLLIDERS:
        if c['category']=='hill_boundary':
            strips[(c['min'][0],c['max'][0],c['max'][2])].append(c)
    COLLIDERS[:]=[c for c in COLLIDERS if c['category']!='hill_boundary']
    for entries in strips.values():
        current=None
        for c in sorted(entries,key=lambda c:c['min'][1]):
            if current is not None and current['max'][1]==c['min'][1]:
                current['max'][1]=c['max'][1]
            else:
                current=c
                COLLIDERS.append(current)
    COLLIDERS.insert(0,{'name':'Overworld_floor','shape':'plane','role':'walkable','category':'terrain',
                        'min':[-32,-26],'max':[32,34],'z':0})
    # Roof/back of the short transition; ceiling doesn't intersect a 1.8-high actor.
    return heights


def tree(index,x,y,z):
    b=Batch('Oak_%02d'%index,'tree')
    for zz in range(4): b.box((x,y,z+zz),(x+1,y+1,z+zz+1),'oak_log',top='oak_log_top',bottom='oak_log_top')
    leaves=set()
    for zz,rad in [(3,2),(4,2),(5,1),(6,0)]:
        for dx in range(-rad,rad+1):
            for dy in range(-rad,rad+1):
                if abs(dx)==rad and abs(dy)==rad and rad==2: continue
                if dx==dy==0 and zz==3: continue
                leaves.add((x+dx,y+dy,z+zz))
    offsets=[(-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)]
    for xx,yy,zz in leaves:
        visible=[i for i,(dx,dy,dz) in enumerate(offsets) if (xx+dx,yy+dy,zz+dz) not in leaves]
        b.box((xx,yy,zz),(xx+1,yy+1,zz+1),'oak_leaves',visible=visible)
    b.finish(GROUPS['Trees'])
    solid('Oak_%02d'%index,(x,y,z),(x+1,y+1,z+4),'tree_trunk')
    LANDMARKS.append({'name':'Oak_%02d'%index,'category':'tree','position':[x,y,z]})


def center_and_farm():
    b=Batch('Village_Well','well')
    for x in range(-2,2):
        for y in range(0,4):
            if x in (-2,1) or y in (0,3): b.box((x,y,0),(x+1,y+1,1),'cobblestone')
    b.box((-1,1,.35),(1,3,.45),'water_static_frame_00')
    for x in (-2,1):
        for y in (0,3): b.box((x,y,1),(x+.3,y+.3,3),'oak_log',top='oak_log_top')
    b.box((-2.5,-.5,3),(2.5,4.5,3.5),'oak_planks')
    b.finish(GROUPS['Village'])
    solid('Well',(-2,0,0),(2,4,3.5),'decoration')
    b=Batch('Farm','farm')
    for x in range(14,24):
        for y in range(15,24):
            if x in (14,23) or y in (15,23):
                b.box((x,y,0),(x+1,y+1,.2),'oak_log',top='oak_log_top')
            elif x in (18,19): b.box((x,y,.01),(x+1,y+1,.05),'water_static_frame_00')
            else:
                b.box((x,y,.01),(x+1,y+1,.08),'dirt',top='farmland_moist')
                for dx,dy in ((.25,.25),(.7,.65)):
                    b.box((x+dx,y+dy,.08),(x+dx+.07,y+dy+.07,.6),'crop_green')
                    b.box((x+dx-.1,y+dy-.08,.45),(x+dx+.17,y+dy+.15,.7),'crop_gold')
    for x,y,z in [(12,19,0),(12,20,0),(12,20,1),(24,16,0)]:
        b.box((x,y,z),(x+1,y+1,z+1),'hay_block_side',top='hay_block_top',bottom='hay_block_top')
        solid('Hay_%s_%s_%s'%(x,y,z),(x,y,z),(x+1,y+1,z+1),'decoration')
    b.finish(GROUPS['Village'])
    solid('Farm_beds',(14,15,0),(24,24,.7),'decoration')
    LANDMARKS.extend([{'name':'Well','category':'center','position':[0,2,0]},
                      {'name':'Farm','category':'farm','approach':[18,13,0]}])


def mine():
    b=Batch('Mine_Entrance','mine')
    for x in (5,11):
        b.box((x,25.5,0),(x+1,26.5,4),'oak_log',top='oak_log_top')
        solid('Mine_post_%s'%x,(x,25.5,0),(x+1,26.5,4),'mine_frame')
    for x in range(5,12): b.box((x,25.5,4),(x+1,26.5,5),'oak_log',top='oak_log_top')
    # Stone transition lining is explicit, even where the hill is only one block tall.
    for y in range(26,33):
        for x in range(5,12):
            b.box((x,y,4),(x+1,y+1,5),'stone')
            if x in (5,11):
                for z in range(4): b.box((x,y,z),(x+1,y+1,z+1),'andesite' if (y+z)%3==0 else 'stone')
    for x in (5,11): solid('Tunnel_wall_%s'%x,(x,26,0),(x+1,33,4),'mine_wall')
    solid('Tunnel_ceiling',(5,26,4),(12,33,5),'mine_ceiling')
    b.box((6,32,0),(11,33,4),'stone')
    solid('Transition_end',(6,32,0),(11,33,4),'mine_end')
    for x,y,mat in [(3,25,'crafting_table_side'),(12,24,'furnace_side')]:
        prefix='crafting_table' if 'crafting' in mat else 'furnace'
        b.box((x,y,0),(x+1,y+1,1),mat,top=prefix+'_top',front=prefix+'_front',bottom='oak_planks' if prefix=='crafting_table' else 'furnace_top')
        solid(prefix,(x,y,0),(x+1,y+1,1),'decoration')
    for x,y,mat in [(4,25,'granite'),(12,25,'diorite')]:
        b.box((x,y,0),(x+1,y+1,.5),mat)
        solid('Rubble_'+mat,(x,y,0),(x+1,y+1,.5),'decoration')
    for x in (5,10): b.box((x,23,.002),(x+2,25,.02),'gravel')
    for x in (6.4,9.4): b.box((x,27,.01),(x+.12,31,.1),'iron_gray')
    for y in range(27,31): b.box((6.2,y,.01),(9.8,y+.2,.05),'oak_planks')
    b.box((5,25.2,4.65),(12,25.4,5.5),'oak_planks')
    b.finish(GROUPS['Mine'])
    curve=bpy.data.curves.new('DIAMOND RUSH','FONT')
    curve.body='DIAMOND RUSH'; curve.align_x='CENTER'; curve.size=.57; curve.extrude=.008
    sign=bpy.data.objects.new('Diamond_Rush_Sign',curve)
    GROUPS['Mine'].objects.link(sign)
    sign.location=(8.5,25.16,4.85); sign.rotation_euler=(math.pi/2,0,0)
    curve.materials.append(MATS['sign_cream'])
    bpy.context.view_layer.objects.active=sign; sign.select_set(True)
    bpy.ops.object.convert(target='MESH'); sign.select_set(False)
    for polygon in sign.data.polygons: polygon.use_smooth=False
    LANDMARKS.extend([{'name':'Mine_interaction_reserved','category':'future_interaction',
                      'position':[8.5,25,1.4],'approach':[8.5,23,0],'radius':2,'implemented':False},
                     {'name':'Mine_transition_reserved','category':'future_transition','position':[8.5,30,0],'implemented':False},
                     {'name':'Solar_return_reserved','category':'future_exit','position':[0,-21,0],'implemented':False}])


def presentation():
    scene=bpy.context.scene
    scene.world=bpy.data.worlds.new('Overworld_Sky')
    scene.world.color=(.5,.65,.8)
    scene.world.use_nodes=True
    bg=next(n for n in scene.world.node_tree.nodes if n.type=='BACKGROUND')
    bg.inputs[0].default_value=(.48,.67,.88,1); bg.inputs[1].default_value=.65
    light=bpy.data.lights.new('Overworld_Sun','SUN'); light.energy=2.4; light.angle=.12
    sun=bpy.data.objects.new(light.name,light); GROUPS['Presentation'].objects.link(sun)
    sun.rotation_euler=(.45,-.5,-.5)
    try: scene.render.engine='CYCLES'
    except TypeError: pass
    scene.cycles.samples=24
    scene.cycles.use_denoising=True
    scene.render.resolution_x=1280; scene.render.resolution_y=800; scene.render.resolution_percentage=100
    scene.view_settings.view_transform='Standard'
    scene.render.image_settings.file_format='PNG'
    cameras={
        'overview':((53,-65,52),(0,4,0),48),
        'player':((0,-20,1.7),(0,6,2),23),
        'mine':((8.5,16,2.3),(8.5,28,2.5),27),
        'farm':((28,7,11),(18,19,0),43),
        'center':((8,-7,5),(0,3,1),25),
    }
    for name,(pos,target,lens) in cameras.items():
        data=bpy.data.cameras.new('DR_Camera_'+name); data.lens=lens; data.clip_end=300
        camera=bpy.data.objects.new(data.name,data); GROUPS['Presentation'].objects.link(camera)
        camera.location=pos; camera.rotation_euler=(Vector(target)-Vector(pos)).to_track_quat('-Z','Y').to_euler()
    scene.camera=bpy.data.objects['DR_Camera_player']
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type=='VIEW_3D':
                area.spaces.active.region_3d.view_perspective='CAMERA'
                area.spaces.active.shading.type='MATERIAL'


def export():
    import io_scene_gltf2
    missing=Path(io_scene_gltf2.__file__).parent/'blender/exp/animation/sampled/shapekeys/channel_target.py'
    if not missing.exists():
        stub=types.ModuleType('io_scene_gltf2.blender.exp.animation.animations')
        def no_animation(*args,**kwargs): raise RuntimeError('Static export only')
        stub.gather_animations=no_animation; sys.modules[stub.__name__]=stub
    bpy.ops.object.select_all(action='DESELECT')
    for key,group in GROUPS.items():
        if key not in ('Presentation','Colliders','Markers'):
            for obj in group.objects: obj.select_set(True)
    dest=OUT/'minecraft_village.glb'
    bpy.ops.export_scene.gltf(filepath=str(dest),export_format='GLB',use_selection=True,
                              export_yup=True,export_animations=False,export_cameras=False,export_lights=False)
    # Make cutout and sampler contracts explicit, independently of exporter version.
    blob=dest.read_bytes(); size=struct.unpack_from('<I',blob,12)[0]
    doc=json.loads(blob[20:20+size]); tail=blob[20+size:]
    for sampler in doc.get('samplers',[]): sampler.update(magFilter=9728,minFilter=9728)
    for mat in doc['materials']:
        mat.setdefault('pbrMetallicRoughness',{}).setdefault('baseColorFactor',[1,1,1,1])
        if mat['name'] in {'DR_'+n for n in ('glass','oak_leaves','oak_door_top','oak_door_bottom')}:
            mat.update(alphaMode='MASK',alphaCutoff=.5,doubleSided=True)
    raw=json.dumps(doc,separators=(',',':')).encode(); raw+=b' '*((-len(raw))%4)
    dest.write_bytes(struct.pack('<4sII',b'glTF',2,20+len(raw)+len(tail))+struct.pack('<I4s',len(raw),b'JSON')+raw+tail)


def main():
    OUT.mkdir(parents=True,exist_ok=True); REPORT.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.preferences.filepaths.save_version=0
    scene=bpy.context.scene; scene.name='Minecraft_Overworld_Village'
    scene.unit_settings.system='METRIC'; scene.unit_settings.scale_length=1
    for name in ('Terrain','Buildings','Village','Trees','Vegetation','Mine','Colliders','Markers','Presentation'):
        c=bpy.data.collections.new('DR_'+name); scene.collection.children.link(c); GROUPS[name]=c
    names=['grass_block_top','grass_block_side','dirt','stone','cobblestone','gravel','oak_planks','oak_log',
           'oak_log_top','oak_leaves','oak_door_top','oak_door_bottom','glass','farmland_moist','hay_block_top',
           'hay_block_side','crafting_table_front','crafting_table_side','crafting_table_top','furnace_front',
           'furnace_side','furnace_top','andesite','granite','diorite','dirt_path_top','dirt_path_side','water_static_frame_00']
    for name in names:
        tint={'grass_block_top':(.48,.76,.27),'oak_leaves':(.34,.64,.18),
              'water_static_frame_00':(.22,.48,.92)}.get(name)
        material(name,tint=tint,alpha=name in ('glass','oak_leaves','oak_door_top','oak_door_bottom'))
    for name,color in [('crop_green',(.22,.48,.04)),('crop_gold',(.69,.53,.08)),('iron_gray',(.32,.34,.35)),
                       ('sign_cream',(.94,.85,.56)),('flower_yellow',(1,.65,.02))]: material(name,color=color)
    house('House_Small_West',-20,-13,5,5,3)
    house('House_Small_East',13,-13,5,5,3)
    house('House_Medium_West',-21,3,7,6,4)
    house('House_Medium_East',13,4,7,6,4)
    house('Village_Hall',-11,16,9,7,4)
    path('South_entry',[(0,-21),(0,-3)])
    path('West_square',[(-4,-3),(-4,7),(0,7)])
    path('East_square',[(0,-3),(4,-3),(4,7),(0,7)])
    path('North_road',[(0,7),(0,23),(8.5,23),(8.5,30)])
    path('Farm_approach',[(0,13),(19,13)])
    # Connect southern approach streets to both sides of the well square.
    path('Square_south',[(-4,-3),(4,-3)])
    cells=path_cells(); heights=terrain(cells)
    center_and_farm(); mine()
    tree_positions=[(-26,-19),(-10,-21),(10,-21),(25,-20),(26,-7),(-27,-8),(-27,9),(-26,19),
                    (-19,24),(-16,28),(-3,28),(21,28),(27,24),(28,9),(-9,8),(23,0)]
    for i,(x,y) in enumerate(tree_positions): tree(i+1,x,y,heights[x,y])
    b=Batch('Meadow_Vegetation','decoration'); rng=random.Random(4801)
    for _ in range(240):
        x,y=rng.randrange(-30,30),rng.randrange(-24,31)
        if (x,y) in cells or any(c['min'][0]-1<x<c['max'][0]+1 and c['min'][1]-1<y<c['max'][1]+1 for c in COLLIDERS if c['role']=='solid'): continue
        z=heights[x,y]
        b.box((x+.4,y+.4,z),(x+.5,y+.5,z+.35),'crop_green')
        if rng.random()<.3: b.box((x+.32,y+.32,z+.3),(x+.58,y+.58,z+.46),'flower_yellow')
    b.finish(GROUPS['Vegetation'])
    for c in COLLIDERS:
        if c['shape']!='aabb': continue
        proxy=Batch('COL_'+c['name'],c['category']); proxy.box(c['min'],c['max'],'stone')
        obj=proxy.finish(GROUPS['Colliders']); obj.display_type='WIRE'; obj.hide_render=True; obj.hide_set(True)
    for landmark in LANDMARKS:
        if 'position' in landmark:
            obj=bpy.data.objects.new(landmark['name'],None); GROUPS['Markers'].objects.link(obj)
            obj.location=landmark['position']; obj.empty_display_size=.5
    presentation()
    scene.render.film_transparent=False
    scene['phase']='1 - environment only; no gameplay'
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'minecraft_village.blend'))
    export()
    metadata={'schema_version':1,'id':'minecraft.diamond-rush.environment','blend':'minecraft_village.blend',
              'visual_asset':'minecraft_village.glb','coordinates':{'units':'meters','up':'Z','gltf_up':'Y','to_gltf':'(x,y,z) -> (x,z,-y)'},
              'spawn':{'position':[0,-21,1.7],'forward':[0,1,0],'eye_height':1.7},
              'player_reference':{'height':1.8,'radius':.35},'roads':ROADS,'colliders':COLLIDERS,
              'landmarks':LANDMARKS,'textures':TEXTURE_AUDIT,'counts':{'buildings':5,'trees':16},
              'mine_clearance':{'width':5,'height':4,'depth':6},
              'rendering':{'texture_filter':'nearest','cutout':'MASK alphaCutoff=0.5','water':'static opaque'},
              'notes':['Closed houses; no interior traversal.','Hills are solid boundaries; all intended routes are at Z=0.',
                       'Reserved interaction/transition markers do not implement gameplay.']}
    (OUT/'minecraft_village.scene.json').write_text(json.dumps(metadata,indent=2,ensure_ascii=False),encoding='utf8')
    (REPORT/'texture_inventory.json').write_text(json.dumps([
        {'file':p.name,'size':list(struct.unpack('>II',p.read_bytes()[16:24]))} for p in sorted(TEXTURES.glob('*.png'))],indent=2),encoding='utf8')
    if '--skip-renders' not in sys.argv:
        for name in ('overview','player','mine','farm','center'):
            scene.camera=bpy.data.objects['DR_Camera_'+name]
            scene.render.filepath=str(REPORT/(name+'.png'))
            bpy.ops.render.render(write_still=True)
    print('DIAMOND RUSH phase 1 generated:',OUT)


if __name__=='__main__': main()

