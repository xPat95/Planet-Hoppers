"""Run with Blender --background --python this_file. Creates only End Ship files."""
from pathlib import Path
import bpy, json, zipfile, math, collections
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT/'assets/models/ships/minecraft'
REF = ROOT/'docs/references/minecraft/end_ship'
TEX = ROOT/'assets/textures/ships'
SOURCE = Path.home()/'AppData/Roaming/.minecraft/versions/1.21.11/1.21.11.jar'
OUT.mkdir(parents=True, exist_ok=True)
REF.mkdir(parents=True, exist_ok=True)
TARGET = OUT/'end_ship.blend'
if TARGET.exists():
    raise RuntimeError('Output already exists; preserve it before rebuilding.')
data = json.loads((REF/'ship_source.json').read_text(encoding='utf8'))
jar = zipfile.ZipFile(SOURCE)

def resource(path):
    return json.loads(jar.read('assets/minecraft/'+path))

def model(name):
    name=name.replace('minecraft:','')
    raw=resource('models/'+name+'.json')
    parent=model(raw['parent']) if 'parent' in raw and not raw['parent'].startswith('builtin') else {}
    return {**parent, **raw, 'textures':{**parent.get('textures',{}),**raw.get('textures',{})}}

def variant(state):
    name=state['Name'].split(':')[1]
    variants=resource('blockstates/'+name+'.json')['variants']
    props=state.get('Properties',{})
    for key,value in variants.items():
        if all(props.get(k)==v for k,v in (s.split('=') for s in key.split(',') if s)):
            return value[0] if isinstance(value,list) else value
    raise ValueError(state)

# Independent scene: preserve anything already open in interactive Blender.
scene=bpy.data.scenes.new('Minecraft_End_Ship')
bpy.context.window.scene=scene
scene.unit_settings.system='METRIC'
scene.unit_settings.scale_length=1
root=bpy.data.objects.new('EndShip_ROOT',None);scene.collection.objects.link(root)
root['forward']='+Y (dragon head / bow)';root['up']='+Z'
root['block_size']=1.0;root['source']='Minecraft Java 1.21.11 / end_city/ship.nbt'
root['origin']='Centerline at main deck level; MC (6.5, 8, 14.5)'
groups={}
for name in ['Hull','Bow','Deck','Mast','Cabin','Crown','End_Rods','Dragon_Head','Interior','Presentation']:
    c=bpy.data.collections.new('EndShip_'+name);scene.collection.children.link(c);groups[name]=c

materials={}
def material(key,filename=None,alpha=1,emission=0,color=None):
    mat=bpy.data.materials.new('EndShip_'+key);mat.use_nodes=True
    bs=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    bs.inputs['Roughness'].default_value=.85
    if filename:
        im=bpy.data.images.load(str(filename),check_existing=True);im.pack()
        im.filepath=bpy.path.relpath(str(filename),start=str(OUT))
        node=mat.node_tree.nodes.new('ShaderNodeTexImage');node.image=im;node.interpolation='Closest';node.extension='REPEAT'
        mat.node_tree.links.new(node.outputs['Color'],bs.inputs['Base Color'])
        if key=='Ladder':
            mat.node_tree.links.new(node.outputs['Alpha'],bs.inputs['Alpha'])
        if emission:
            mat.node_tree.links.new(node.outputs['Color'],bs.inputs['Emission Color'])
    if color: bs.inputs['Base Color'].default_value=(*color,1)
    if alpha<1:
        bs.inputs['Alpha'].default_value=alpha
        bs.inputs['Roughness'].default_value=.25
        bs.inputs['Transmission Weight'].default_value=.18
    if emission:
        bs.inputs['Emission Strength'].default_value=emission
        if color:bs.inputs['Emission Color'].default_value=(*color,1)
    mat.diffuse_color=(*color,alpha) if color else (.55,.32,.58,alpha)
    materials[key]=mat
    return key

for key,f in [('Purpur','purpur_block'),('Pillar_Side','purpur_pillar_side'),('Pillar_Top','purpur_pillar_top'),('End_Stone','end_stone_bricks'),('Obsidian','obsidian')]:material(key,TEX/(f+'.png'))
material('Glass',TEX/'purple_stained_glass.png',alpha=.38)
material('Rod',TEX/'end_rod.png',emission=.6)
material('Dragon_Eyes',color=(.65,.08,.8),emission=.7)
extra=TEX/'minecraft_end_ship';extra.mkdir(exist_ok=True)
for file,key,res in [('ladder.png','Ladder','block/ladder.png'),('chest.png','Chest','entity/chest/normal.png'),('brewing_stand.png','Brewing','block/brewing_stand.png'),('brewing_stand_base.png','Brewing_Base','block/brewing_stand_base.png')]:
    p=extra/file
    if not p.exists():p.write_bytes(jar.read('assets/minecraft/textures/'+res))
    material(key,p)

meshes=collections.defaultdict(lambda: [[],[],[],[]])
def world(p):
    x,y,z=p
    return (x-6.5,14.5-z,y-8)

def face(group,key,points,uv=None):
    verts,faces,uvs,mats=meshes[group]
    idx=len(verts);verts.extend(world(p) for p in points);faces.append(tuple(range(idx,idx+len(points))))
    uvs.append(uv or [(0,0),(1,0),(1,1),(0,1)]);mats.append(key)

def box(group,key,lo,hi,uv=None):
    x,y,z=lo;X,Y,Z=hi
    for i,points in enumerate([[(x,y,z),(x,y,Z),(x,Y,Z),(x,Y,z)],[(X,y,Z),(X,y,z),(X,Y,z),(X,Y,Z)],[(x,y,Z),(x,y,z),(X,y,z),(X,y,Z)],[(x,Y,z),(x,Y,Z),(X,Y,Z),(X,Y,z)],[(X,y,z),(x,y,z),(x,Y,z),(X,Y,z)],[(x,y,Z),(X,y,Z),(X,Y,Z),(x,Y,Z)]]):
        face(group,key,points,uv.get(i) if isinstance(uv,dict) else uv)

def group_for(p):
    x,y,z=p
    if y>=20:return 'Crown'
    if x==6 and z==14 and y>=8:return 'Mast'
    if z>=21 and y>=6:return 'Cabin'
    if z<11 and y>=8:return 'Bow'
    if y>=8:return 'Deck'
    return 'Hull'

# Use vanilla model boxes for stair corners, upside-down stairs, and slabs.
cells={};special=[];counts=collections.Counter()
for b in data['blocks']:
    state=data['palette'][b['state']];name=state['Name'].split(':')[1];pos=b['pos']
    if name in ('air','structure_block'):continue
    counts[name]+=1
    if name in ('end_rod','ladder','dragon_wall_head','chest','brewing_stand'):
        special.append((name,pos,state));continue
    v=variant(state);definition=model(v['model'])
    def transform(p):
        x,y,z=[q-8 for q in p]
        for axis,deg in [('x',v.get('x',0)),('y',v.get('y',0))]:
            a=math.radians(-deg);c=round(math.cos(a));s=round(math.sin(a))
            if axis=='x':y,z=y*c-z*s,y*s+z*c
            else:x,z=x*c+z*s,-x*s+z*c
        return [round(x+8),round(y+8),round(z+8)]
    for e in definition['elements']:
        a=transform(e['from']);c=transform(e['to']);lo=[min(q,r)//8 for q,r in zip(a,c)];hi=[max(q,r)//8 for q,r in zip(a,c)]
        for x in range(lo[0],hi[0]):
            for y in range(lo[1],hi[1]):
                for z in range(lo[2],hi[2]):
                    cells[(pos[0]*2+x,pos[1]*2+y,pos[2]*2+z)]=(name,tuple(pos),state.get('Properties',{}).get('axis'),group_for(pos))

# Cull all hidden half-block faces, including contacts between stair components.
directions=[(-1,0,0),(1,0,0),(0,-1,0),(0,1,0),(0,0,-1),(0,0,1)]
for p,(name,origin,axis,group) in cells.items():
    x,y,z=[q*.5 for q in p];X,Y,Z=x+.5,y+.5,z+.5
    sides=[[(x,y,z),(x,y,Z),(x,Y,Z),(x,Y,z)],[(X,y,Z),(X,y,z),(X,Y,z),(X,Y,Z)],[(x,y,Z),(x,y,z),(X,y,z),(X,y,Z)],[(x,Y,z),(x,Y,Z),(X,Y,Z),(X,Y,z)],[(X,y,z),(x,y,z),(x,Y,z),(X,Y,z)],[(x,y,Z),(X,y,Z),(X,Y,Z),(x,Y,Z)]]
    for direction,points in zip(directions,sides):
        neighbor=cells.get(tuple(a+b for a,b in zip(p,direction)))
        if neighbor and (neighbor[0]!='magenta_stained_glass' or name=='magenta_stained_glass'):continue
        key={'end_stone_bricks':'End_Stone','obsidian':'Obsidian','magenta_stained_glass':'Glass'}.get(name,'Purpur')
        normal_axis=next(i for i,d in enumerate(direction) if d)
        if name=='purpur_pillar':key='Pillar_Top' if normal_axis=={'x':0,'y':1,'z':2}[axis] else 'Pillar_Side'
        # Face-aligned, one texture repetition per block, pillar grooves follow axis.
        if normal_axis==1:uv=[(q[0]-origin[0],q[2]-origin[2]) for q in points]
        elif normal_axis==0:uv=[(q[2]-origin[2],q[1]-origin[1]) for q in points]
        else:uv=[(q[0]-origin[0],q[1]-origin[1]) for q in points]
        if name=='purpur_pillar' and axis=='z' and normal_axis!=2:uv=[(v,u) for u,v in uv]
        face(group,key,points,uv)

for name,p,state in special:
    x,y,z=p
    if name=='dragon_wall_head':
        # Vanilla dragon head silhouette: skull, long muzzle, lower jaw, two horns.
        box('Dragon_Head','Obsidian',(x,y,z-.35),(x+1,y+1,z+1))
        box('Dragon_Head','Obsidian',(x+.125,y+.18,z-1.15),(x+.875,y+.5,z-.25))
        box('Dragon_Head','Obsidian',(x+.125,y+.02,z-1.15),(x+.875,y+.21,z-.15))
        for a in [x+.125,x+.75]:box('Dragon_Head','Obsidian',(a,y+.95,z+.25),(a+.125,y+1.35,z+.5))
        for a in [x-.002,x+1.002]:
            box('Dragon_Head','Dragon_Eyes',(a-.002,y+.61,z-.22),(a+.002,y+.74,z+.03))
    elif name=='ladder':
        # South-facing plane, alpha-cut texture from the same local game installation.
        face('Mast','Ladder',[(x,y,z+.06),(x+1,y,z+.06),(x+1,y+1,z+.06),(x,y+1,z+.06)])
    elif name=='end_rod':
        v=variant(state);m=model(v['model'])
        for e in m['elements']:
            a=[q/16 for q in e['from']];b=[q/16 for q in e['to']]
            if state['Properties']['facing']=='south':
                a=[a[0],1-a[2],a[1]];b=[b[0],1-b[2],b[1]]
            atlas={}
            for i,side in enumerate(['west','east','down','up','north','south']):
                u0,v0,u1,v1=e['faces'].get(side,e['faces']['up'])['uv']
                atlas[i]=[(u0/16,1-v1/16),(u1/16,1-v1/16),(u1/16,1-v0/16),(u0/16,1-v0/16)]
            box('End_Rods','Rod',tuple(p[i]+min(a[i],b[i]) for i in range(3)),tuple(p[i]+max(a[i],b[i]) for i in range(3)),atlas)
    elif name=='chest':
        # Vanilla chest atlas: body, lid, and latch, original two interior positions.
        uv=[(14/64,1-33/64),(28/64,1-33/64),(28/64,1-19/64),(14/64,1-19/64)]
        box('Interior','Chest',(x+.0625,y,z+.0625),(x+.9375,y+.625,z+.9375),uv)
        box('Interior','Chest',(x+.0625,y+.625,z+.0625),(x+.9375,y+.875,z+.9375),[(14/64,1-19/64),(28/64,1-19/64),(28/64,1-14/64),(14/64,1-14/64)])
        box('Interior','Brewing_Base',(x+.4375,y+.45,z+.925),(x+.5625,y+.7,z+.99))
    elif name=='brewing_stand':
        box('Interior','Brewing_Base',(x+.12,y,z+.12),(x+.88,y+.125,z+.88))
        box('Interior','Brewing',(x+.4375,y+.125,z+.4375),(x+.5625,y+.875,z+.5625))

for group,(verts,faces,uvs,keys) in meshes.items():
    mesh=bpy.data.meshes.new('EndShip_'+group+'_Mesh');mesh.from_pydata(verts,[],faces);mesh.update()
    ob=bpy.data.objects.new('EndShip_'+group,mesh);groups[group].objects.link(ob);ob.parent=root
    unique=list(dict.fromkeys(keys))
    for key in unique:mesh.materials.append(materials[key])
    layer=mesh.uv_layers.new(name='Minecraft_Block_UV')
    for poly,uv,key in zip(mesh.polygons,uvs,keys):
        poly.material_index=unique.index(key)
        for loop,co in zip(poly.loop_indices,uv):layer.data[loop].uv=co
    # Weld coincident positions without merging UV seams.
    import bmesh
    bm=bmesh.new();bm.from_mesh(mesh);bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=.00001);bm.to_mesh(mesh);bm.free();mesh.update()

world_data=bpy.data.worlds.new('EndShip_Studio');scene.world=world_data;world_data.use_nodes=True
next(n for n in world_data.node_tree.nodes if n.type=='BACKGROUND').inputs[0].default_value=(.085,.075,.12,1)
next(n for n in world_data.node_tree.nodes if n.type=='BACKGROUND').inputs[1].default_value=.8
def aim(ob,target):ob.rotation_euler=(Vector(target)-ob.location).to_track_quat('-Z','Y').to_euler()
def light(name,loc,power,size):
    dat=bpy.data.lights.new(name,'AREA');dat.energy=power;dat.shape='DISK';dat.size=size
    ob=bpy.data.objects.new(name,dat);groups['Presentation'].objects.link(ob);ob.location=loc;aim(ob,(0,0,2))
light('EndShip_Key',(12,15,28),6500,18)
light('EndShip_Fill',(-16,8,15),4500,20)
light('EndShip_Rim',(5,-18,22),5500,15)
camdata=bpy.data.cameras.new('EndShip_Camera');camdata.type='ORTHO'
cam=bpy.data.objects.new('EndShip_Camera',camdata);groups['Presentation'].objects.link(cam);scene.camera=cam
try:scene.render.engine='CYCLES'
except TypeError:pass
scene.cycles.samples=32
scene.cycles.use_denoising=True
scene.render.resolution_x=1400;scene.render.resolution_y=1200;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG';scene.render.film_transparent=False
scene.view_settings.view_transform='Standard'
scene.render.image_settings.color_mode='RGBA'
views=[('three_quarter',(-36,43,29),(0,0,3),38),('side',(-40,0,3.5),(0,0,3.5),35),('top',(0,0,60),(0,0,0),39)]
for name,loc,target,scale in views:
    cam.location=loc;aim(cam,target);camdata.ortho_scale=scale
    scene.render.filepath=str(REF/('end_ship_'+name+'.png'))
    bpy.ops.render.render(write_still=True)
cam.location=views[0][1];aim(cam,views[0][2]);camdata.ortho_scale=views[0][3]
scene.render.filepath='//../../../../docs/references/minecraft/end_ship/end_ship_three_quarter.png'
bpy.ops.object.select_all(action='DESELECT');root.select_set(True);bpy.context.view_layer.objects.active=root
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type=='VIEW_3D':
            area.spaces.active.region_3d.view_distance=44
            area.spaces.active.region_3d.view_location=(0,0,3)
            area.spaces.active.region_3d.view_rotation=cam.rotation_euler.to_quaternion()
            area.spaces.active.shading.type='MATERIAL'
scene['notes']='Exact original template geometry. Purple glass replaces magenta as requested. Obsidian provisional dragon head. No game logic.'
# Remove the untouched startup scene from this new asset file only.
for s in list(bpy.data.scenes):
    if s!=scene:bpy.data.scenes.remove(s)
bpy.ops.wm.save_as_mainfile(filepath=str(TARGET))
report={'source_size_blocks':data['size'],'source_block_counts':dict(counts),'half_cells':len(cells),'objects':len(scene.objects),'mesh_objects':len(meshes),'faces':sum(len(o.data.polygons) for o in scene.objects if o.type=='MESH'),'vertices':sum(len(o.data.vertices) for o in scene.objects if o.type=='MESH'),'packed_images':[im.name for im in bpy.data.images if im.packed_file],'forward':'+Y','up':'+Z','origin':'Main deck centerline','adaptations':['Purple glass supplied by user replaces vanilla magenta glass','Dragon head obsidian placeholder with cubic silhouette','Interior chest UVs and brewing stand simplified; no loot, mobs, item frames or bottles','Structure data markers excluded']}
(REF/'validation.json').write_text(json.dumps(report,indent=2),encoding='utf8')
print(json.dumps(report,indent=2))
