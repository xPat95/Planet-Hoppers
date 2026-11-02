"""DIAMOND RUSH phase 3. Deterministic finite mine; only writes new mine derivatives.

Run Blender --background --factory-startup --python-exit-code 1 --python this_file.
Reuses the phase-1 block mesh/UV/material helpers without running its generator.
"""
import importlib.util
from pathlib import Path
from collections import defaultdict, deque
import hashlib
import json
import math
import random
import struct
import sys
import types
import bpy
from mathutils import Vector
sys.dont_write_bytecode=True

REPO=Path(__file__).resolve().parents[3]
OUT=REPO/'assets/models/minigames/minecraft/mine'
REPORT=REPO/'docs/references/minecraft/phase3'
spec=importlib.util.spec_from_file_location('village_helpers',Path(__file__).with_name('build_explorable_village.py'))
v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
OFFSETS=[(-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)]
ROOMS=[
    ('A_entry','A',[-4,-3,5,6],4),('A_junction','A',[-5,12,5,21],4),
    ('A_supply','A',[-19,13,-12,20],3),('B_hub','B',[-5,28,6,38],5),
    ('B_east','B',[13,26,22,35],4),('B_secret','B',[-21,29,-13,37],4),
    ('C_lava','C',[-6,46,6,58],6),('C_vault','C',[13,46,24,57],5),
    ('C_secret','C',[-22,47,-14,55],4),
]
TUNNELS=[
    ('A_entry_junction','A_entry','A_junction',[[-2,6,2,12]],[[0,4],[0,16]],3,None),
    ('A_supply_branch','A_junction','A_supply',[[-12,15,-5,18]],[[0,16.5],[-15.5,16.5]],3,None),
    ('A_B_main','A_junction','B_hub',[[-2,21,2,28]],[[0,16],[0,33]],4,None),
    ('A_B_east_loop','A_junction','B_east',[[5,15,19,18],[16,16,19,26]],[[0,16.5],[17.5,16.5],[17.5,30.5]],3,None),
    ('B_east_link','B_hub','B_east',[[6,30,13,34]],[[0,32],[17.5,32]],4,None),
    ('B_hidden_branch','B_hub','B_secret',[[-13,32,-5,34]],[[0,33],[-17,33]],3,'B_gate'),
    ('B_C_main','B_hub','C_lava',[[-2,38,2,46]],[[0,33],[0,48]],4,None),
    ('B_C_east','B_east','C_vault',[[16,35,20,46]],[[18,30],[18,47]],4,None),
    ('C_return_link','C_lava','C_vault',[[6,50,13,53]],[[3.5,51.5],[18,51.5]],3,None),
    ('C_hidden_branch','C_lava','C_secret',[[-14,50,-6,52]],[[-5,51],[-18,51]],3,'C_gate'),
]
DIAMONDS=[
    ('A_junction',(-4,21,1)),('A_supply',(-20,16,1)),
    ('B_hub',(-6,30,1)),('B_hub',(4,38,1)),('B_east',(22,28,1)),
    ('B_secret',(-22,31,1)),('B_secret',(-17,37,1)),
    ('C_lava',(-7,47,1)),('C_lava',(-3,58,1)),('C_lava',(6,55,1)),
    ('C_vault',(24,48,1)),('C_vault',(20,57,1)),
    ('C_secret',(-23,49,1)),('C_secret',(-18,55,1)),
]
COLLIDERS=[]
INTERACTIVE=[]
LIGHTS=[]
GROUPS={}
ORE_SPEC=[('coal_ore',14,-3,28),('iron_ore',14,6,40),('gold_ore',10,28,58),('redstone_ore',12,28,58),('lapis_ore',10,26,58)]

def layout_bounds(height):
    return min(x for x,y in height)-4,min(y for x,y in height)-4,max(x for x,y in height)+5,max(y for x,y in height)+5


def rect_cells(rect):
    x0,y0,x1,y1=rect
    return {(x,y) for x in range(x0,x1) for y in range(y0,y1)}


def rectangles(cells):
    """Greedy exact 2D covering. No collider crosses an air cell."""
    remaining=set(cells)
    while remaining:
        x,y=min(remaining,key=lambda p:(p[1],p[0]));xx=x+1
        while (xx,y) in remaining:xx+=1
        yy=y+1
        while all((i,yy) in remaining for i in range(x,xx)):yy+=1
        for j in range(y,yy):
            for i in range(x,xx):remaining.remove((i,j))
        yield [x,y,xx,yy]


def collider(name,lo,hi,category):
    COLLIDERS.append(dict(name=name,shape='aabb',role='solid',category=category,min=list(lo),max=list(hi)))


def block(name,position,material,room,kind,approach=None,gate=None):
    b=v.Batch(name,kind);x,y,z=position
    b.box((x,y,z),(x+1,y+1,z+1),material)
    obj=b.finish(GROUPS['Interactive']);obj['interactive_id']=name;obj['kind']=kind
    collider(name,position,(x+1,y+1,z+1),kind)
    entry=dict(id=name,object=name,collider_id=name,kind=kind,room=room,sector=room[0],
               min=list(position),max=[x+1,y+1,z+1],initial_visible=True,initial_solid=True,
               reset=dict(visible=True,solid=True),gate=gate)
    if approach:entry['approach']=list(approach)
    INTERACTIVE.append(entry)


def lava_material():
    """Pack first square of the 16x320 animation in memory; never alter source PNG."""
    path=v.TEXTURES/'lava_still.png';source=bpy.data.images.load(str(path))
    w,h=source.size;check=h%w==0
    assert check and w==16,(w,h)
    pixels=list(source.pixels);image=bpy.data.images.new('DR_lava_frame_00',width=w,height=w,alpha=True)
    image.pixels[:]=pixels[(h-w)*w*4:h*w*4];image.pack()
    mat=bpy.data.materials.new('DR_lava_still');mat.use_nodes=True
    node=mat.node_tree.nodes.new('ShaderNodeTexImage');node.image=image;node.interpolation='Closest';node.extension='REPEAT'
    bsdf=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    mat.node_tree.links.new(node.outputs['Color'],bsdf.inputs['Base Color'])
    bsdf.inputs['Emission Color'].default_value=(1,.2,.015,1);bsdf.inputs['Emission Strength'].default_value=.8
    v.MATS['lava_still']=mat
    v.TEXTURE_AUDIT.append(dict(file=str(path.relative_to(REPO)).replace('\\','/'),size=[w,h],
                              sha256=hashlib.sha256(path.read_bytes()).hexdigest(),frame=0,frame_count=h//w,
                              note='First top frame packed in memory, no external derivative texture'))


def build_geometry():
    height={}
    for _,_,rect,h in ROOMS:
        for p in rect_cells(rect):height[p]=max(h,height.get(p,0))
    for _,_,_,rects,_,h,_ in TUNNELS:
        for rect in rects:
            for p in rect_cells(rect):height[p]=max(h,height.get(p,0))
    air={(x,y,z) for (x,y),h in height.items() for z in range(h)}
    gates={}
    for name,x,ys,room in [('B_gate',-9,[32,33],'B_secret'),('C_gate',-10,[50,51],'C_secret')]:
        gates[name]=[]
        for y in ys:
            for z in range(3):
                ident='BREAK_%s_%s_%s'%(name,y,z)
                block(ident,(x,y,z),'cobblestone' if z==1 else 'stone',room,'breakable',gate=name)
                gates[name].append(ident)
    diamond_cells={p for _,p in DIAMONDS}
    for index,(room,p) in enumerate(DIAMONDS,1):
        x,y,z=p
        assert p not in air
        neighbours=[(x+dx,y+dy) for dx,dy,dz in OFFSETS[:4] if (x+dx,y+dy,1) in air]
        assert neighbours,('Unexposed diamond',room,p)
        ax,ay=neighbours[0]
        block('ORE_DIAMOND_%02d'%index,p,'diamond_ore',room,'diamond',(ax+.5,ay+.5,0))
    reserved={tuple(i['min']) for i in INTERACTIVE}
    void=air|reserved
    candidates=set()
    for x,y,z in void:
        for dx,dy,dz in OFFSETS:
            p=x+dx,y+dy,z+dz
            if p not in void:candidates.add(p)
    rng=random.Random(9307)
    ore_materials={}
    for material,count,low,high in ORE_SPEC:
        choices=sorted(p for p in candidates if p[2] in (1,2) and low<=p[1]<=high and p not in ore_materials)
        for p in rng.sample(choices,count):ore_materials[p]=material
    batches={}
    for x,y,z in sorted(candidates):
        visible=[i for i,(dx,dy,dz) in enumerate(OFFSETS) if (x+dx,y+dy,z+dz) in void]
        if not visible:continue
        sector='A' if y<25 else 'B' if y<43 else 'C'
        part='Floor' if z==-1 else 'Ceiling' if (x,y) in height and z>=height[x,y] else 'Walls'
        name='Shell_%s_%s'%(sector,part)
        b=batches.setdefault(name,v.Batch(name,'static_'+part.lower()))
        material=ore_materials.get((x,y,z))
        if not material:
            material='gravel' if z==-1 and (x*13+y*7)%17<3 else (
                'granite' if 28<=y<34 and x<-2 else 'diorite' if 34<=y<=40 else
                'andesite' if (x+y)//4%5==0 else 'cobblestone' if sector=='A' and z==0 else 'stone')
        b.box((x,y,z),(x+1,y+1,z+1),material,visible=visible)
    for b in batches.values():b.finish(GROUPS['Static'])
    # Exact solid complement of the floor footprint, merged into rectangular wall volumes.
    ore_columns={(p[0],p[1]) for p in diamond_cells}
    x0,y0,x1,y1=layout_bounds(height);wall_top=max(8,max(height.values())+1)
    wall_cells={(x,y) for x in range(x0,x1) for y in range(y0,y1)}-set(height)-ore_columns
    for i,(x,y,xx,yy) in enumerate(rectangles(wall_cells)):
        collider('Wall_%03d'%i,(x,y,0),(xx,yy,wall_top),'wall')
    for index,(_,p) in enumerate(DIAMONDS):
        x,y,z=p
        collider('Ore_column_lower_%02d'%index,(x,y,0),(x+1,y+1,z),'wall')
        collider('Ore_column_upper_%02d'%index,(x,y,z+1),(x+1,y+1,wall_top),'wall')
    by_height=defaultdict(set)
    for p,h in height.items():by_height[h].add(p)
    for h,cells in by_height.items():
        for i,(x,y,xx,yy) in enumerate(rectangles(cells)):
            collider('Ceiling_%s_%03d'%(h,i),(x,y,h),(xx,yy,h+1),'ceiling')
    return height,gates,ore_materials


def decorations(height):
    b=v.Batch('Timber_Supports','support')
    for index,(cx,y) in enumerate([(0,7),(0,11),(0,23),(0,27),(18,38),(18,43)]):
        for x in (cx-1.9,cx+1.65):
            b.box((x,y,0),(x+.25,y+.35,2.8),'oak_log',top='oak_log_top',bottom='oak_log_top')
            collider('Support_%s_%s'%(index,x),(x,y,0),(x+.25,y+.35,2.8),'support')
        b.box((cx-1.9,y,2.8),(cx+1.9,y+.35,3),'oak_log',top='oak_log_top',bottom='oak_log_top')
        collider('Support_beam_%s'%index,(cx-1.9,y,2.8),(cx+1.9,y+.35,3),'support')
    b.finish(GROUPS['Static'])
    rails=v.Batch('Upper_Rails','decoration')
    for x in (-.65,.55):rails.box((x,2,.01),(x+.1,15,.08),'metal')
    for y in range(2,15):rails.box((-.9,y,.01),(.9,y+.18,.04),'oak_planks')
    rails.finish(GROUPS['Static'])
    lamps=[(0,-2,2.8),(3.5,3.5,2.7),(-1.6,9,2.5),(3.5,17,2.8),(-16,14,2.4),
           (17,17,2.5),(1.5,24,2.7),(-3.5,29,3),(4.5,36,3),(-11,33,2.5),
           (14.5,28,2.7),(20.5,33,2.7),(17,40,2.7),(-18,30,2.7),
           (-4.5,47.5,3),(4.5,56,3),(11,51,2.5),(14.5,47,3),(22.5,55,3),(-19,48,2.7)]
    torches=v.Batch('Torch_Props','light_fixture')
    for i,(x,y,z) in enumerate(lamps):
        torches.box((x-.07,y-.07,z-.6),(x+.07,y+.07,z),'oak_planks')
        torches.box((x-.12,y-.12,z-.05),(x+.12,y+.12,z+.2),'torch_flame')
        ceiling=height[math.floor(x),math.floor(y)]
        if ceiling>z+.2:torches.box((x-.035,y-.035,z+.2),(x+.035,y+.035,ceiling),'metal')
        LIGHTS.append(dict(id='Torch_%02d'%i,position=[x,y,z],color=[1,.66,.32],radius=9 if y<25 else 8))
    torches.finish(GROUPS['Static'])
    regions=[]
    for name,rect in [('Lava_West',[-4,50,-1,55]),('Lava_Vault',[19,49,22,52])]:
        x,y,xx,yy=rect;b=v.Batch(name,'lava')
        b.box((x,y,-.08),(xx,yy,.025),'lava_still')
        b.finish(GROUPS['Lava'])
        regions.append(dict(id=name,min=[x,y,-.1],max=[xx,yy,.5],damage_enabled=False,
                            animation=dict(source='lava_still.png',frames=20,active=False)))
        LIGHTS.append(dict(id=name,position=[(x+xx)/2,(y+yy)/2,.7],color=[1,.22,.035],radius=10))
    for title,pos,rotation in [('A / MINA SUPERIOR',(0,20.95,3.25),math.pi/2),
                               ('B / GALERIAS',(0,37.95,4.25),math.pi/2),
                               ('C / PROFUNDIDADES',(0,57.95,4.5),math.pi/2),
                               ('SALIDA / OVERWORLD',(0,-2.95,2.7),-math.pi/2)]:
        curve=bpy.data.curves.new(title,'FONT');curve.body=title;curve.align_x='CENTER';curve.size=.31;curve.extrude=.004
        obj=bpy.data.objects.new('Sign_'+title,curve);GROUPS['Static'].objects.link(obj)
        obj.location=pos;obj.rotation_euler=(rotation,0,0)
        curve.materials.append(v.MATS['sign_green' if 'SALIDA' in title else 'sign_cream'])
        bpy.context.view_layer.objects.active=obj;obj.select_set(True);bpy.ops.object.convert(target='MESH');obj.select_set(False)
        for p in obj.data.polygons:p.use_smooth=False
    return regions


def preview():
    scene=bpy.context.scene;scene.world=bpy.data.worlds.new('Underground_Ambient');scene.world.use_nodes=True
    bg=next(n for n in scene.world.node_tree.nodes if n.type=='BACKGROUND');bg.inputs[0].default_value=(.12,.16,.22,1);bg.inputs[1].default_value=.12
    for item in LIGHTS:
        light=bpy.data.lights.new(item['id'],'POINT');light.energy=110 if item['id'].startswith('Torch') else 210
        light.color=item['color'];light.shadow_soft_size=.25
        obj=bpy.data.objects.new(light.name,light);GROUPS['Presentation'].objects.link(obj);obj.location=item['position']
    scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=True
    scene.render.resolution_x=1280;scene.render.resolution_y=800;scene.render.resolution_percentage=100
    scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='Standard'
    poses={'entrance':((0,-.5,1.7),(0,15,1.7)),
           'galleries':((0,31,1.7),(20,31,1.6)),
           'depths':((0,48,1.7),(-2,54,1)),
           'diamonds':((21,47.5,1.7),(24.4,48.5,1.5)),
           'overview':((50,-42,76),(0,29,0))}
    for name,(pos,target) in poses.items():
        data=bpy.data.cameras.new('Mine_Camera_'+name);data.lens=23 if name!='overview' else 43;data.clip_start=.05
        obj=bpy.data.objects.new(data.name,data);GROUPS['Presentation'].objects.link(obj)
        obj.location=pos;obj.rotation_euler=(Vector(target)-Vector(pos)).to_track_quat('-Z','Y').to_euler()
        if name=='overview':data.type='ORTHO';data.ortho_scale=92
    scene.camera=bpy.data.objects['Mine_Camera_entrance']
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type=='VIEW_3D':area.spaces.active.region_3d.view_perspective='CAMERA';area.spaces.active.shading.type='MATERIAL'
    return poses


def export_glb():
    import io_scene_gltf2
    if not (Path(io_scene_gltf2.__file__).parent/'blender/exp/animation/sampled/shapekeys/channel_target.py').exists():
        stub=types.ModuleType('io_scene_gltf2.blender.exp.animation.animations')
        def disabled(*args,**kwargs):raise RuntimeError('Static mine export only')
        stub.gather_animations=disabled;sys.modules[stub.__name__]=stub
    bpy.ops.object.select_all(action='DESELECT')
    for name in ('Static','Interactive','Lava'):
        for obj in GROUPS[name].objects:obj.select_set(True)
    target=OUT/'minecraft_mine.glb'
    bpy.ops.export_scene.gltf(filepath=str(target),export_format='GLB',use_selection=True,export_yup=True,
                              export_animations=False,export_cameras=False,export_lights=False,export_extras=True)
    blob=target.read_bytes();size=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+size]);tail=blob[20+size:]
    for sampler in doc.get('samplers',[]):sampler.update(magFilter=9728,minFilter=9728)
    for mat in doc['materials']:mat.setdefault('pbrMetallicRoughness',{}).setdefault('baseColorFactor',[1,1,1,1])
    raw=json.dumps(doc,separators=(',',':')).encode();raw+=b' '*((-len(raw))%4)
    target.write_bytes(struct.pack('<4sII',b'glTF',2,20+len(raw)+len(tail))+struct.pack('<I4s',len(raw),b'JSON')+raw+tail)


def main():
    OUT.mkdir(parents=True,exist_ok=True);REPORT.mkdir(parents=True,exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True);bpy.context.preferences.filepaths.save_version=0
    scene=bpy.context.scene;scene.name='Minecraft_Underground_Mine';scene.unit_settings.system='METRIC';scene.unit_settings.scale_length=1
    for name in ('Static','Interactive','Lava','Colliders','Markers','Presentation'):
        group=bpy.data.collections.new('DRM_'+name);scene.collection.children.link(group);GROUPS[name]=group
    for name in ('stone','cobblestone','granite','diorite','andesite','gravel','coal_ore','iron_ore','gold_ore',
                 'redstone_ore','lapis_ore','diamond_ore','oak_planks','oak_log','oak_log_top'):v.material(name)
    for name,color in [('metal',(.28,.3,.32)),('torch_flame',(1,.43,.05)),('sign_cream',(.9,.72,.39)),('sign_green',(.22,1,.35))]:
        v.material(name,color=color)
        if name in ('torch_flame','sign_green'):
            bsdf=next(n for n in v.MATS[name].node_tree.nodes if n.type=='BSDF_PRINCIPLED')
            bsdf.inputs['Emission Color'].default_value=(*color,1);bsdf.inputs['Emission Strength'].default_value=.8
    lava_material();height,gates,ore_materials=build_geometry();lava=decorations(height)
    x0,y0,x1,y1=layout_bounds(height)
    COLLIDERS.insert(0,dict(name='Mine_floor',shape='plane',role='walkable',category='floor',min=[x0,y0],max=[x1,y1],z=0))
    for c in COLLIDERS:
        if c['shape']!='aabb':continue
        b=v.Batch('COL_'+c['name'],c['category']);b.box(c['min'],c['max'],'stone')
        obj=b.finish(GROUPS['Colliders']);obj.hide_render=True;obj.hide_set(True);obj.display_type='WIRE'
    markers={'Mine_Spawn':[0,0,0],'Mine_Exit':[0,-2.7,1.4]}
    for name,position in markers.items():
        obj=bpy.data.objects.new(name,None);GROUPS['Markers'].objects.link(obj);obj.location=position
    poses=preview()
    scene['phase']='3: underground geometry and independent development preview; no mining'
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'minecraft_mine.blend'));export_glb()
    graph=dict(nodes=[dict(id=name,sector=sector,rect=rect,height=h,center=[(rect[0]+rect[2])/2,(rect[1]+rect[3])/2,0])
                      for name,sector,rect,h in ROOMS],
               edges=[dict(id=name,a=a,b=b,rects=rects,points=[[x,y,0] for x,y in points],height=h,
                           requires_break=gates.get(gate,[])) for name,a,b,rects,points,h,gate in TUNNELS])
    data=dict(schema_version=1,id='minecraft.diamond-rush.mine',blend='minecraft_mine.blend',visual_asset='minecraft_mine.glb',
              coordinates=dict(units='meters',up='Z',gltf_up='Y',to_gltf='(x,y,z) -> (x,z,-y)'),
              spawn=dict(position=[0,0,1.7],forward=[0,1,0],eye_height=1.7),
              colliders=COLLIDERS,roads=[dict(name=e['id'],points=e['points']) for e in graph['edges']],
              graph=graph,interactive_blocks=INTERACTIVE,lava_regions=lava,lights=LIGHTS,
              floor_cells=[dict(x=x,y=y,ceiling=h) for (x,y),h in sorted(height.items())],
              exit=dict(id='Mine_Exit',min=[-1.5,-3,0],max=[1.5,-2.5,2.8],forward=[0,-1,0],
                        surface_manifest='../minecraft_village.scene.json',surface_position=[8.5,23,1.7],
                        surface_forward=[0,-1,0],transition_implemented=False),
              texture_sources=v.TEXTURE_AUDIT,decorative_ores=dict((name,sum(m==name for m in ore_materials.values()))
                for name in ('coal_ore','iron_ore','gold_ore','redstone_ore','lapis_ore')),
              counts=dict(chambers=len(ROOMS),tunnels=len(TUNNELS),diamonds=len(DIAMONDS),breakables=12,lava_regions=2),
              notes=['Flat walkable floor Z=0; sector depth is thematic, no vertical descent.',
                     'No extraction, removal, collection, timer, damage or challenge transitions.',
                     'Two gated chambers require future removal of all six blocks per gate.',
                     'Lava has a solid development floor; sensor metadata is separate from solids.'])
    (OUT/'minecraft_mine.scene.json').write_text(json.dumps(data,indent=2),encoding='utf8')
    for name in ([] if '--skip-renders' in sys.argv else poses):
        scene.camera=bpy.data.objects['Mine_Camera_'+name]
        if name=='overview':
            for obj in GROUPS['Static'].objects:
                if obj.name.startswith('Shell_') and obj.name.endswith('Ceiling'):obj.hide_render=True
            # Technical cutaway only; stored blend always retains its ceilings.
            for light in bpy.data.lights:light.energy*=3
        scene.render.filepath=str(REPORT/('blender_'+name+'.png'));bpy.ops.render.render(write_still=True)
    print('MINE_GENERATED',json.dumps(data['counts']))


if __name__=='__main__':main()
