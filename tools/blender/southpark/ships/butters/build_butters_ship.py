"""Native editable Butters ship. Front -Y, up +Z; metres. Only creates its own scene."""
from pathlib import Path
import bpy, math, random, json, bmesh
from mathutils import Vector, Matrix
REPO=Path(__file__).resolve().parents[5]
OUT=REPO/'assets/models/ships/southpark/butters'
REF=REPO/'docs/references/southpark/ships/butters'
TEX=REPO/'assets/textures/ships/southpark/butters'
for p in (OUT,REF,TEX): p.mkdir(parents=True,exist_ok=True)
NAME='SouthPark_Butters_Ship'
old=bpy.data.scenes.get(NAME)
scene=bpy.data.scenes.new(NAME+'_BUILD'); bpy.context.window.scene=scene
if old:
    oldcols=[]
    def remember(col):
        for child in col.children:
            remember(child); oldcols.append(child)
    remember(old.collection)
    for ob in list(old.objects): bpy.data.objects.remove(ob,do_unlink=True)
    bpy.data.scenes.remove(old)
    for col in oldcols:
        if col.name in bpy.data.collections: bpy.data.collections.remove(col)
    bpy.data.orphans_purge(do_recursive=True)
scene.name=NAME; bpy.context.window.scene=scene
scene.unit_settings.system='METRIC'; scene.unit_settings.scale_length=1
asset=bpy.data.collections.new('NAVE SOUTH PARK | Butters'); scene.collection.children.link(asset)
stage=bpy.data.collections.new('PRESENTATION | exclude from game'); scene.collection.children.link(stage)
root=bpy.data.objects.new('SouthPark_Butters_ROOT',None); asset.objects.link(root)
root['asset_role']='ship'; root['front']='-Y'; root['up']='+Z'; root['units']='metres'
groups={}
for name in ('Hull','Cabin','Face','Tape','Engines','Fins','Doodles'):
    col=bpy.data.collections.new('BUTTERS | '+name); asset.children.link(col); groups[name]=col
current=groups['Hull']
def link(ob):
    for c in list(ob.users_collection): c.objects.unlink(ob)
    current.objects.link(ob)
    if current!=stage: ob.parent=root
    return ob
def material(name,color,rough=.8,metal=0):
    m=bpy.data.materials.new('BUTTERS | '+name); m.use_nodes=True; m.diffuse_color=(*color,1)
    bs=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    bs.inputs['Base Color'].default_value=(*color,1); bs.inputs['Roughness'].default_value=rough; bs.inputs['Metallic'].default_value=metal
    return m
card=material('Kraft cardboard',(.58,.34,.15))
edge=material('Exposed corrugation dark',(.20,.105,.037))
liner=material('Exposed paper liner',(.70,.45,.23))
tape=material('Ivory cloth tape',(.72,.70,.59),.92)
ink=material('Charcoal felt tip',(.022,.019,.017))
purple=material('Purple marker',(.40,.13,.48))
yellow=material('Butters sunshine hair',(1,.76,.025))
gold=material('Yellow crayon',(.91,.61,.07))
white=material('Paper white eyes',(.94,.93,.87))
red=material('Recycled red cans',(.67,.045,.035),.64)
steel=material('Dull recycled collars',(.29,.31,.31),.7,.18)
inside=material('Nozzle soot',(.055,.045,.04))
plastic=material('Transparent smoky plastic',(.22,.29,.32),.17)
bs=next(n for n in plastic.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
bs.inputs['Transmission Weight'].default_value=.62; bs.inputs['IOR'].default_value=1.46
glint=material('Plastic reflection strokes',(.65,.73,.73),.27)
seatmat=material('Teal seat cushion',(.11,.31,.28))
rng=random.Random(221)
im=bpy.data.images.new('butters_kraft_paper',width=256,height=256)
pixels=[]
for y in range(256):
    for x in range(256):
        v=rng.uniform(-.022,.022)+.012*math.sin(x*.12)*math.cos(y*.09)
        if rng.random()<.035: v-=.038
        pixels.extend((.65+v,.411+v*.8,.211+v*.5,1))
im.pixels.foreach_set(pixels); im.filepath_raw=str(TEX/'butters_kraft_paper.png')
im.file_format=next(i.identifier for i in im.bl_rna.properties['file_format'].enum_items if i.identifier=='PNG')
im.save(); im.pack()
tex=card.node_tree.nodes.new('ShaderNodeTexImage'); tex.image=im
bs=next(n for n in card.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
card.node_tree.links.new(tex.outputs['Color'],bs.inputs['Base Color'])
def finish(ob,name,mat,bevel=0):
    ob.name='BUTTERS | '+name; link(ob)
    if mat: ob.data.materials.append(mat)
    if bevel:
        mod=ob.modifiers.new('Tiny folded paper edges','BEVEL'); mod.width=bevel; mod.segments=1
    return ob
def mesh(name,verts,faces,mat,bevel=0):
    me=bpy.data.meshes.new(name); me.from_pydata(verts,[],faces); me.update()
    ob=bpy.data.objects.new(name,me); current.objects.link(ob); finish(ob,name,mat,bevel)
    uv=me.uv_layers.new(name='PaperUV')
    for poly in me.polygons:
        axis=max(range(3),key=lambda a:abs(poly.normal[a])); axes=[a for a in range(3) if a!=axis]
        for li in poly.loop_indices:
            v=me.vertices[me.loops[li].vertex_index].co; uv.data[li].uv=(v[axes[0]]*.6,v[axes[1]]*.6)
    return ob
def box(name,loc,size,mat=card,bevel=.008):
    bpy.ops.mesh.primitive_cube_add(size=1,location=loc)
    ob=bpy.context.object; ob.scale=size
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    return finish(ob,name,mat,bevel)
def line(name,points,mat=ink,radius=.008,cyclic=False):
    cu=bpy.data.curves.new(name,'CURVE'); cu.dimensions='3D'; cu.resolution_u=1; cu.bevel_depth=radius; cu.bevel_resolution=0; cu.use_fill_caps=True
    sp=cu.splines.new('POLY'); sp.points.add(len(points)-1)
    for p,co in zip(sp.points,points): p.co=(*co,1)
    sp.use_cyclic_u=cyclic
    ob=bpy.data.objects.new(name,cu); current.objects.link(ob)
    return finish(ob,name,mat)
def cylinder(name,loc,radius,depth,mat,axis=(0,1,0),vertices=24):
    if name.startswith(('Red recycled thruster','Dull can collar')):
        # True hollow cans and annular straps; no hidden cap across the nozzle.
        rinner=radius-.026
        c=Vector(loc); q=Vector(axis).to_track_quat('Z','Y')
        vs=[]
        for z,r in ((-depth/2,radius),(depth/2,radius),(-depth/2,rinner),(depth/2,rinner)):
            vs += [tuple(c+q@Vector((r*math.cos(i*math.tau/vertices),r*math.sin(i*math.tau/vertices),z))) for i in range(vertices)]
        fs=[]
        for a,b in ((0,1),(1,3),(3,2),(2,0)):
            fs += [(a*vertices+i,a*vertices+(i+1)%vertices,b*vertices+(i+1)%vertices,b*vertices+i) for i in range(vertices)]
        return mesh(name,vs,fs,mat)
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices,radius=radius,depth=depth,location=loc)
    ob=bpy.context.object; ob.rotation_euler=Vector(axis).to_track_quat('Z','Y').to_euler()
    return finish(ob,name,mat,.005)
def slab(name,points,normal,thickness,mat=card):
    n=Vector(normal)*thickness; count=len(points)
    vs=[tuple(Vector(p)+n*d) for d in (0,1) for p in points]
    fs=[tuple(reversed(range(count))),tuple(range(count,count*2))]
    fs += [(i,(i+1)%count,(i+1)%count+count,i+count) for i in range(count)]
    return mesh(name,vs,fs,mat)
def ellipse(name,center,rx,rz,mat,normal=(0,-1,0)):
    side=Vector((1,0,0)) if abs(normal[1]) else Vector((0,1,0))
    c=Vector(center)
    return slab(name,[tuple(c+side*(rx*math.cos(i*math.tau/32))+Vector((0,0,rz*math.sin(i*math.tau/32)))) for i in range(32)],normal,.012,mat)
# Single external shell: omit hidden top underneath the raised cabin.
cross=[(-2,.18),(2,.18),(2,1.88),(1.62,1.88),(1.20,2.62),(-.76,2.62),(-1.46,1.88),(-2,1.88)]
verts=[(x,y,z) for x in (-1.23,1.23) for y,z in cross]
faces=[]
for i in (0,1,2,6,7):
    j=(i+1)%8; faces.append((i,j,j+8,i+8))
faces += [(0,7,6,3,2,1),(8,9,10,11,14,15)]
mesh('Closed box lower shell and hood',verts,faces,card,.009)
current=groups['Cabin']
box('Sealed cabin roof',(0,.22,2.635),(2.47,1.98,.035))
def window(name,outer,inner,normal):
    n=Vector(normal); vs=[tuple(Vector(p)+n*.008) for p in outer+inner]
    mesh(name+' cardboard frame',vs,[(i,(i+1)%4,(i+1)%4+4,i+4) for i in range(4)],card)
    line(name+' dark gasket',[tuple(Vector(p)+n*.012) for p in inner],edge,.016,True)
    pane=slab(name+' plastic pane',inner,normal,.006,plastic); pane['closed_window']=True
    a,b,c,d=[Vector(p)+n*.022 for p in inner]
    line(name+' long reflection',[tuple(a.lerp(b,.11).lerp(d.lerp(c,.11),.18)),tuple(a.lerp(b,.26).lerp(d.lerp(c,.26),.86))],glint,.014)
    line(name+' short reflection',[tuple(a.lerp(b,.20).lerp(d.lerp(c,.20),.13)),tuple(a.lerp(b,.28).lerp(d.lerp(c,.28),.48))],glint,.005)
outer=[(-1.23,-1.46,1.88),(1.23,-1.46,1.88),(1.23,-.76,2.62),(-1.23,-.76,2.62)]
inner=[(-1.07,-1.373,1.972),(1.07,-1.373,1.972),(1.07,-.865,2.509),(-1.07,-.865,2.509)]
window('Windshield',outer,inner,(0,-.726,.687))
for s in (-1,1):
    x=s*1.237
    outer=[(x,-1.46,1.88),(x,1.62,1.88),(x,1.20,2.62),(x,-.76,2.62)]
    inner=[(x,-.81,2.005),(x,.68,2.005),(x,.48,2.47),(x,-.69,2.47)]
    window(('Port' if s<0 else 'Starboard')+' side window',outer,inner,(s,0,0))
slab('Rear sealed cabin panel',[(-1.23,1.62,1.88),(1.23,1.62,1.88),(1.23,1.2,2.62),(-1.23,1.2,2.62)],(0,.87,.49),.025)
box('Dashboard',(0,-1.23,1.95),(2,.22,.12),edge)
box('Seat back',(0,.55,2.08),(.68,.16,.53),seatmat,.04)
cylinder('Joystick stem',(.25,-.98,2.07),.025,.22,ink,(0,0,1),12)
bpy.ops.mesh.primitive_uv_sphere_add(segments=12,ring_count=6,radius=.075,location=(.25,-.98,2.21))
finish(bpy.context.object,'Red joystick knob',red)
current=groups['Hull']
for s in (-1,1):
    line('Box folded side edge',[(s*1.238,-2,.20),(s*1.238,2,.20),(s*1.238,2,1.87)],edge,.012)
    line('Side hood crease',[(s*1.242,-2,1.885),(s*1.242,-1.46,1.885)],edge,.01)
    line('Side panel joint',[(s*1.246,-.95,.20),(s*1.246,-.95,1.88)],edge,.009)
    line('Rear box joint',[(s*1.247,1.52,.20),(s*1.247,1.52,1.88)],edge,.009)
for z in (.20,1.88): line('Front folded liner',[(-1.22,-2.012,z),(1.22,-2.012,z)],edge,.013)
for i in range(44):
    x=-1.2+i*2.4/43
    line('Front corrugated flute',[(x,-2.018,1.878),(x+.019,-2.022,1.845),(x+.039,-2.018,1.878)],liner,.005)
box('Long roof seam tape',(.36,.22,2.668),(.18,1.98,.008),tape,0)
current=groups['Face']
for s in (-1,1):
    ellipse('White eye '+str(s),(s*.405,-2.022,1.06),.385,.49,white)
    ellipse('Black pupil '+str(s),(s*.22,-2.047,1.065),.054,.071,ink)
line('Small happy smile',[(.24*math.cos(a),-2.047,.47+.09*math.sin(a)) for a in [math.pi+i*math.pi/20 for i in range(21)]],ink,.018)
hair=[(-1.10,2.01),(-.62,2.21),(-.85,2.28),(-.35,2.29),(-.39,2.64),(-.05,2.34),(.50,2.48),(.38,2.27),(1.09,2.25),(.69,2.04),(.91,1.79),(.38,1.97),(.39,1.66),(.05,1.98),(-.16,1.75),(-.30,2.02),(-.86,1.80)]
def hairpoint(x,z):
    rise=max(0,z-1.9)
    return (x,-2.055+rise*.9,z-rise*.35)
slab('Spiky yellow paper hair',[hairpoint(x,z) for x,z in hair],(0,-1,0),.035,yellow)
line('Hair cut edge',[hairpoint(x,z) for x,z in hair],gold,.008,True)
current=groups['Tape']
def sticker(name,center,u,v,width,height):
    c=Vector(center); u=Vector(u); v=Vector(v)
    coords=[(-.5,-.5),(.5,-.5),(.48,-.39),(.53,-.30),(.48,-.22),(.52,-.12),(.49,.5),(-.5,.5),(-.47,.39),(-.53,.31),(-.49,.21)]
    pts=[tuple(c+u*x*width+v*y*height) for x,y in coords]
    slab(name,pts,u.cross(v).normalized(),.004,tape)
for y in (-1.1,.95):
    box('Tape across roof',(0,y if y>0 else -.59,2.661),(2.49,.19,.008),tape,0)
    for s in (-1,1):
        sticker('Side vertical reinforcement',(s*1.263,y,1.16),(0,1,0),(0,0,1),.22,1.38)
        sticker('Side tape cross patch',(s*1.27,y,.57),(0,1,0),(0,0,1),.58,.20)
for s in (-1,1):
    for z in (.43,1.60):
        sticker('Front corner tape',(s*1.13,-2.035,z),(1,0,0),(0,0,1),.29,.25)
        sticker('Side corner tape',(s*1.267,-1.83,z),(0,1,0),(0,0,1),.37,.25)
    box('Tape over hood',(s*.89,-1.74,1.899),(.22,.52,.012),tape,0)
current=groups['Fins']
for s in (-1,1):
    pts=[(s*1.23,.48,.79),(s*1.23,1.65,.79),(s*2.07,1.95,.42),(s*2.08,.96,.42)]
    slab('Small cardboard stabilizer '+str(s),pts,(0,0,-1),.055)
    line('Wing corrugated edge '+str(s),[pts[1],pts[2],pts[3],pts[0]],edge,.012)
    slab('Purple wing stripe '+str(s),[(s*1.40,.83,.728),(s*1.45,1.02,.706),(s*1.94,1.72,.484),(s*2.0,1.57,.457)],(0,0,1),.008,purple)
    for j in range(15):
        t=(j+.3)/15; p=Vector(pts[3]).lerp(Vector(pts[2]),t)
        line('Wing exposed flute',[(p.x,p.y,p.z-.006),(p.x-s*.025,p.y+.012,p.z-.043),(p.x,p.y+.03,p.z-.006)],liner,.005)
fin=[(-.045,.93,2.63),(-.045,1.69,3.55),(-.045,2.16,3.55),(-.045,2.12,2.30),(-.045,1.52,2.30)]
slab('Rear vertical cardboard fin',fin,(1,0,0),.09)
for s in (-1,1):
    slab('Fin purple streak',[(s*.049,1.38,2.64),(s*.049,1.86,3.40),(s*.049,2.04,3.44),(s*.049,1.71,2.63)],(s,0,0),.004,purple)
line('Fin cut perimeter',[(0,y,z) for x,y,z in fin],edge,.014,True)
for s in (-1,1):
    pts=[]
    for i in range(10):
        a=i*math.pi/5; r=.105 if i%2==0 else .043
        x=s*(1.89+math.sin(a)*r); y=1.79+math.cos(a)*r
        z=.79-(abs(x)-1.23)*(.37/.85)+.014
        pts.append((x,y,z))
    slab('Purple wingtip star',pts,(0,0,1),.003,purple)
current=groups['Engines']
for s in (-1,1):
    x=s*.88
    box('Taped cardboard engine mount '+str(s),(x,2.10,1.18),(.75,.55,.95))
    cylinder('Red recycled thruster '+str(s),(x,2.52,1.36),.43,.88,red)
    for y in (2.22,2.40,2.87): cylinder('Dull can collar',(x,y,1.36),.448,.105,steel)
    count=24; vs=[]
    for y,r in ((2.98,.43),(2.98,.32),(2.77,.29)):
        vs += [(x+r*math.cos(i*math.tau/count),y,1.36+r*math.sin(i*math.tau/count)) for i in range(count)]
    fs=[]
    for k in range(2):
        fs += [(k*count+i,k*count+(i+1)%count,(k+1)*count+(i+1)%count,(k+1)*count+i) for i in range(count)]
    mesh('Rolled exhaust rim '+str(s),vs,fs,steel)
    cylinder('Recessed soot inside nozzle',(x,2.765,1.36),.291,.015,inside)
    for a in (0,math.pi): box('Engine strap clip',(x+.46*math.cos(a),2.40,1.36+.46*math.sin(a)),(.06,.18,.10),steel)
    sticker('Engine mount tape',(x,2.03,1.664),(1,0,0),(0,1,0),.22,.45)
current=groups['Doodles']
def sidepoint(s,u,v,offset=.275): return (s*(1+offset),u,v)
def doodle(s,name,coords,mat=gold,r=.009,closed=False): return line(name,[sidepoint(s,u,v) for u,v in coords],mat,r,closed)
def star(s,u,v,r,mat=gold):
    pts=[(u+math.sin(i*math.pi/5)*r*(1 if i%2==0 else .40),v+math.cos(i*math.pi/5)*r*(1 if i%2==0 else .40)) for i in range(10)]
    doodle(s,'Hand drawn star',pts,mat,.009,True)
for s in (-1,1):
    u=-.28; v=1.13
    pts=[sidepoint(s,u+math.sin(i*math.pi/8)*(.49 if i%2==0 else .31),v+math.cos(i*math.pi/8)*(.49 if i%2==0 else .31),.269) for i in range(16)]
    slab('Purple SP burst '+str(s),pts,(s,0,0),.008,purple)
    pts=[sidepoint(s,u+math.sin(i*math.pi/8)*(.355 if i%2==0 else .235),v+math.cos(i*math.pi/8)*(.355 if i%2==0 else .235),.279) for i in range(16)]
    slab('Kraft SP burst center '+str(s),pts,(s,0,0),.004,card)
    cu=bpy.data.curves.new('SP lettering','FONT'); cu.body='SP'; cu.size=.40; cu.align_x='CENTER'; cu.align_y='CENTER'; cu.extrude=.001
    ob=bpy.data.objects.new('SP',cu); current.objects.link(ob); finish(ob,'Hand painted SP '+str(s),purple)
    ob.matrix_world=Matrix(((0,0,s,s*1.291),(s,0,0,u),(0,1,0,v),(0,0,0,1)))
    for a,b,r in ((-.79,1.57,.09),(.30,1.67,.11),(.64,.69,.085),(-.69,.59,.085),(1.24,1.59,.07)): star(s,a,b,r,ink if a==.30 else gold)
    u=.63; v=1.18
    doodle(s,'Butters doodle head',[(u+.20*math.cos(a),v+.23*math.sin(a)) for a in [i*math.tau/24 for i in range(24)]],ink,.007,True)
    doodle(s,'Doodle yellow hair',[(u-.21,v+.18),(u-.11,v+.30),(u-.09,v+.18),(u+.01,v+.32),(u+.04,v+.19),(u+.17,v+.28),(u+.21,v+.17)],gold,.012)
    for ey in (-.06,.06): doodle(s,'Doodle dot eye',[(u+ey,v+.025),(u+ey,v+.045)],ink,.013)
    doodle(s,'Doodle smile',[(u-.05,v-.075),(u,v-.09),(u+.05,v-.07)],ink,.006)
    doodle(s,'Doodle body',[(u,v-.24),(u,v-.49)],ink,.009)
    for a in (-1,1):
        doodle(s,'Doodle arm',[(u,v-.28),(u+a*.13,v-.36)],ink,.008)
        doodle(s,'Doodle leg',[(u,v-.49),(u+a*.10,v-.68)],ink,.009)
    u=1.25; v=1.07
    doodle(s,'Little ringed planet',[(u+.08*math.cos(a),v+.11*math.sin(a)) for a in [i*math.tau/20 for i in range(20)]],purple,.012,True)
    doodle(s,'Planet orbit',[(u+.15*math.cos(a),v+.06*math.sin(a)+.09*math.cos(a)) for a in [i*math.tau/24 for i in range(24)]],ink,.007,True)
# Bake code-native strokes into meshes for interchange.
for ob in list(asset.all_objects):
    if ob.type in {'CURVE','FONT'}:
        bpy.ops.object.select_all(action='DESELECT'); ob.select_set(True); bpy.context.view_layer.objects.active=ob
        bpy.ops.object.convert(target=next(i.identifier for i in bpy.ops.object.convert.get_rna_type().properties['target'].enum_items if i.identifier=='MESH'))
for ob in asset.all_objects:
    if ob.type=='MESH':
        bm=bmesh.new(); bm.from_mesh(ob.data); bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces)); bm.to_mesh(ob.data); bm.free()
current=stage
floor=box('Studio floor',(0,0,.06),(200,200,.12),material('Studio warm white',(.80,.78,.72)),0); floor['render_only']=True
world=bpy.data.worlds.new('BUTTERS | warm studio'); world.use_nodes=True
next(n for n in world.node_tree.nodes if n.type=='BACKGROUND').inputs['Color'].default_value=(.67,.73,.80,1)
next(n for n in world.node_tree.nodes if n.type=='BACKGROUND').inputs['Strength'].default_value=.35
scene.world=world
def area(name,loc,power,size):
    data=bpy.data.lights.new(name,'AREA'); data.energy=power; data.shape='DISK'; data.size=size
    ob=bpy.data.objects.new(name,data); stage.objects.link(ob); ob.location=loc
    ob.rotation_euler=(Vector((0,0,1))-ob.location).to_track_quat('-Z','Y').to_euler()
area('Studio key',(1,-5,8),1000,6); area('Studio fill',(-5,-2,4),600,5); area('Studio rim',(2,5,7),900,4)
camdata=bpy.data.cameras.new('Review camera'); cam=bpy.data.objects.new('Review camera',camdata); stage.objects.link(cam)
scene.camera=cam; camdata.type='ORTHO'; camdata.ortho_scale=7.6
cam.location=(7,-9,6); cam.rotation_euler=(Vector((0,.30,1.65))-cam.location).to_track_quat('-Z','Y').to_euler()
try: scene.render.engine='CYCLES'
except TypeError: pass
scene.cycles.samples=32; scene.cycles.use_denoising=True
scene.render.resolution_x=1280; scene.render.resolution_y=960; scene.render.resolution_percentage=100
scene.render.image_settings.file_format=next(i.identifier for i in scene.render.image_settings.bl_rna.properties['file_format'].enum_items if i.identifier=='PNG')
try: scene.view_settings.view_transform='Standard'
except TypeError: pass
scene.render.filepath=str(REF/'butters_ship_three_quarter.png')
# Centre source pivot at the hull. Whole presentation moves with it.
shift=Vector((0,.35,1.65))
for ob in asset.all_objects:
    if ob!=root: ob.location-=shift
for ob in stage.objects: ob.location-=shift
root['pivot']='Hull centre; front -Y, up +Z'
im.filepath='//../../../../textures/ships/southpark/butters/butters_kraft_paper.png'
for screen in bpy.data.screens:
    for ar in screen.areas:
        if ar.type=='VIEW_3D':
            ar.spaces.active.region_3d.view_perspective='CAMERA'; ar.spaces.active.shading.type='MATERIAL'
bpy.ops.object.select_all(action='DESELECT'); root.select_set(True); bpy.context.view_layer.objects.active=root
# Remove only the untouched startup scene identified during initial inspection.
startup=bpy.data.scenes.get('Scene')
if startup and startup!=scene and set(o.name for o in startup.objects)=={'Cube','Camera','Light'}:
    for ob in list(startup.objects): bpy.data.objects.remove(ob,do_unlink=True)
    bpy.data.scenes.remove(startup)
bpy.context.preferences.filepaths.save_version=0
bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'southpark_butters_ship.blend'))
print('BUTTERS_BUILT',len(asset.all_objects),'objects',str(OUT))
