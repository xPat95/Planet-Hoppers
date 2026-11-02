"""Phase 1 art rebuild. Source station and the Rust platform are read-only.

blender -b --factory-startup --python tools/blender/spaceStation/rebuild_hangar_phase1.py
Only the independent phase1 .blend/.glb and review evidence are written.
Game coordinates: X right, Y up, -Z exit. Blender coordinates: X, -Z, Y.
"""
from pathlib import Path
from math import sin, cos, pi, tan, sqrt
import bpy, bmesh, json, hashlib, sys, types
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'assets/models/environment/spaceStation'
OUT=ROOT/'docs/references/hangar/phase1'
OUT.mkdir(parents=True,exist_ok=True)
source=BASE/'citadel_space_station.blend'
source_hash=hashlib.sha256(source.read_bytes()).hexdigest()
bpy.ops.wm.open_mainfile(filepath=str(source))
mats={m.name.removeprefix('CI | '):m for m in bpy.data.materials if m.name.startswith('CI | ')}
scene=bpy.data.scenes.new('PRESIDIUM | Phase 1 maintenance concourse')
bpy.context.window.scene=scene
for other in list(bpy.data.scenes):
    if other!=scene: bpy.data.scenes.remove(other)
for ob in list(bpy.data.objects): bpy.data.objects.remove(ob,do_unlink=True)
architecture=bpy.data.collections.new('ARCHITECTURE | export only');scene.collection.children.link(architecture)
preview=bpy.data.collections.new('REVIEW ONLY | platform ship lights cameras');scene.collection.children.link(preview)

def principled(mat): return next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
def variant(name,base,factor=1,emission=None):
    mat=mats[base].copy();mat.name='CIH | '+name
    node=principled(mat);col=node.inputs['Base Color'].default_value
    node.inputs['Base Color'].default_value=tuple(c*factor for c in col[:3])+(1,)
    if emission is not None: node.inputs['Emission Strength'].default_value=emission
    mats[name]=mat;return name
variant('Recess shadow','Juntas oscuras',.35)
variant('Oiled steel','Placas acero',.32)
variant('Brushed edge','Placas plata',.85)
variant('Louver interior','Edificios oscuros',.38)
variant('Low ice','Luz hielo',1,.45)
variant('Low ivory','Luz marfil',1,.32)
variant('Low amber','Luz ambar',1,.48)
variant('Status cyan','Luz cian',1,.6)
def p(v): return (v[0],-v[2],v[1])

class Mesh:
    def __init__(self,name): self.name,self.v,self.f,self.m=name,[],[],[]
    def add(self,verts,faces,mat):
        n=len(self.v);self.v.extend(p(v) for v in verts)
        self.f.extend(tuple(n+i for i in face) for face in faces);self.m.extend([mat]*len(faces))
    def box(self,c,d,mat,angle=0):
        def pos(a,b,e):
            x,z=a*d[0]/2,e*d[2]/2
            return (c[0]+x*cos(angle)+z*sin(angle),c[1]+b*d[1]/2,c[2]+z*cos(angle)-x*sin(angle))
        self.add([pos(a,b,e) for e in [-1,1] for b in [-1,1] for a in [-1,1]],
            [(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],mat)
    def beam(self,a,b,width,depth,mat):
        a,b=Vector(a),Vector(b);t=(b-a).normalized()
        ref=Vector((0,0,1)) if abs(t.z)<.9 else Vector((0,1,0))
        u=t.cross(ref).normalized()*width/2;v=t.cross(u).normalized()*depth/2
        self.add([q+u*i+v*j for q in [a,b] for j in [-1,1] for i in [-1,1]],
            [(0,1,3,2),(4,6,7,5),(0,4,5,1),(2,3,7,6),(0,2,6,4),(1,5,7,3)],mat)
    def cassette(self,c,d,mat,axis='Z',cut=.16):
        # Chamfered armor volume; several nested cassettes form deep mechanical wells.
        a,b=d[0]/2,d[1]/2;k=min(cut,a*.35,b*.35)
        profile=[(-a+k,-b),(a-k,-b),(a,-b+k),(a,b-k),(a-k,b),(-a+k,b),(-a,b-k),(-a,-b+k)]
        def point(u,v,w):
            q=(w,v,u) if axis=='X' else (u,w,v) if axis=='Y' else (u,v,w)
            return tuple(c[i]+q[i] for i in range(3))
        vs=[point(u,v,w) for w in [-d[2]/2,d[2]/2] for u,v in profile]
        fs=[tuple(range(7,-1,-1)),tuple(range(8,16))]+[(i,(i+1)%8,(i+1)%8+8,i+8) for i in range(8)]
        self.add(vs,fs,mat)
    def tube(self,points,radius,mat,sides=8):
        for a,b in zip(points,points[1:]):
            a,b=Vector(a),Vector(b);t=(b-a).normalized();ref=Vector((0,1,0)) if abs(t.y)<.9 else Vector((1,0,0))
            u=t.cross(ref).normalized();v=t.cross(u).normalized()
            vs=[q+radius*(u*cos(i*2*pi/sides)+v*sin(i*2*pi/sides)) for q in [a,b] for i in range(sides)]
            self.add(vs,[tuple(range(sides-1,-1,-1)),tuple(range(sides,sides*2))]+
                [(i,(i+1)%sides,(i+1)%sides+sides,i+sides) for i in range(sides)],mat)
    def arch(self,z,r,t,depth,mat,a=0,b=pi,count=28,cy=-2):
        for i in range(count):
            aa=a+(b-a)*i/count;bb=a+(b-a)*(i+1)/count
            self.add([(rr*cos(v),cy+rr*sin(v),zz) for zz in [z-depth/2,z+depth/2] for rr in [r,r+t] for v in [aa,bb]],
                [(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],mat)
    def sector(self,r0,r1,a,b,y,height,mat,steps=3):
        for i in range(steps):
            aa=a+(b-a)*i/steps;bb=a+(b-a)*(i+1)/steps
            vs=[(rr*cos(v),yy,rr*sin(v)) for yy in [y-height,y] for rr in [r0,r1] for v in [aa,bb]]
            self.add(vs,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],mat)
    def finish(self,bevel=.02):
        me=bpy.data.meshes.new(self.name);me.from_pydata(self.v,[],self.f)
        names=list(dict.fromkeys(self.m))
        for name in names: me.materials.append(mats[name])
        lookup={name:i for i,name in enumerate(names)}
        for poly,name in zip(me.polygons,self.m): poly.material_index=lookup[name]
        bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free()
        ob=bpy.data.objects.new(self.name,me);architecture.objects.link(ob)
        if bevel:
            mod=ob.modifiers.new('Machined edges | real bevel','BEVEL');mod.width=bevel;mod.segments=1
        return ob

# MACRO: a five-bay annular pressure shell. Layered silver bands and graphite
# gaskets reproduce the original Presidium, with structural depth rather than wallpaper.
shell=Mesh('01 | Annular pressure hull and offset armor tiles')
crown=Mesh('02 | Five load couplings and laminated annular frames')
roof=Mesh('03 | Ceiling heat exchangers and mechanical keel')
lights=Mesh('04 | Recessed service lighting and occupied galleries')
for row,z in enumerate(range(-25,28,5)):
    shell.arch(z,18.0,.5,4.96,'Blindaje grafito',count=24)
    for k in range(24):
        a=k*pi/24+.008;b=(k+1)*pi/24-.008
        shell.arch(z,17.85,.13,4.55,'Titanio azulado' if (row+k)%4 else 'Placas acero',a,b,1)
        if k%3!=0: shell.arch(z+.3,17.77,.08,2.9,'Edificios pizarra',a+.01,b-.01,1)
for z in [-26,-15,-4,7,18,27]:
    for r,t,w,mat in [(17.0,.68,.75,'Placas plata'),(16.84,.16,1.12,'Juntas oscuras'),(16.72,.12,.15,'Brushed edge')]:
        crown.arch(z,r,t,w,mat,count=28)
    for k in range(1,28):
        a=k*pi/28
        # Raised attachment teeth and paired fasteners, the same rhythm as the original ring.
        crown.arch(z,16.58,.20,.44,'Placas acero',a-.012,a+.012,1)
for x in [-8.8,0,8.8]:
    y=sqrt(18**2-x*x)-2
    roof.cassette((x,y-.55,0),(.95,.8,55),'Blindaje grafito')
    for z in [-22,-11,0,11,22]:
        roof.cassette((x,y-.92,z),(.76,3.0,.18),'Recess shadow',axis='Y')
        for j in range(12): roof.box((x,y-1.04,z-1.25+j*.22),(.68,.08,.08),'Placas plata')
        lights.box((x-.43,y-.99,z),(.06,.05,2.9),'Low ice')
for side in [-1,1]:
    for z in [-20,-9,2,13,24]:
        y=11.2
        roof.cassette((side*11.3,y,z),(2.7,3.4,.55),'Placas acero',axis='Y',cut=.45)
        roof.cassette((side*11.3,y-.33,z),(2.25,2.9,.08),'Recess shadow',axis='Y')
        for j in range(10): roof.box((side*11.3,y-.42,z-1.2+j*.26),(2.15,.06,.10),'Brushed edge')
        roof.tube([(side*12.8,10.9,z-1.7),(side*10.2,11.1,z-1.7),(side*9.3,12,z-2.3)],.13,'Placas acero')

# MEDIUM: five distinct service sectors on each side. Observation galleries,
# recessed machinery bays, sloped root shields, conduit risers and deep louver racks.
for side in [-1,1]:
    walls=Mesh(('05' if side<0 else '06')+' | Layered district wall '+str(side))
    works=Mesh(('07' if side<0 else '08')+' | Pumps radiators and service conduits '+str(side))
    gallery=Mesh(('09' if side<0 else '10')+' | Mezzanine bridgework '+str(side))
    walls.box((side*17.7,3.8,0),(.7,8.0,55),'Juntas oscuras')
    gallery.box((side*14.9,4.35,0),(3.5,.35,53),'Placas acero')
    for z in [-22,-11,0,11,22]:
        walls.cassette((side*16.8,3.0,z),(8.5,5.0,1.1),'Blindaje grafito',axis='X',cut=.6)
        walls.cassette((side*16.15,3.0,z),(7.5,4.2,.35),'Brushed edge',axis='X',cut=.45)
        walls.cassette((side*15.90,3.0,z),(6.9,3.65,.25),'Recess shadow',axis='X',cut=.4)
        # Purposeful alternation: coolant plant / electrical distribution / access bay.
        sector=int((z+22)/11)
        if sector%2==0:
            for dz in [-1.8,0,1.8]:
                works.cassette((side*15.6,2.9,z+dz),(1.45,3.0,.45),'Oiled steel',axis='X')
                for j in range(14): works.box((side*15.29,1.7+j*.17,z+dz),(.12,.07,1.25),'Placas acero')
                works.tube([(side*15.2,1.2,z+dz-.6),(side*14.9,1.5,z+dz-.6),(side*14.9,4.0,z+dz-.6)],.10,'Brushed edge')
        else:
            for dz in [-2.2,2.2]:
                works.cassette((side*15.65,2.5,z+dz),(1.7,2.7,.45),'Titanio azulado',axis='X')
                works.cassette((side*15.35,2.7,z+dz),(1.25,1.5,.09),'Recess shadow',axis='X')
                for j in range(5): works.box((side*15.26,2.2+j*.2,z+dz),(.05,.06,1.05),'Marcas de panel')
                lights.box((side*15.20,3.65,z+dz),(.04,.06,.8),'Status cyan')
            walls.cassette((side*15.5,2.4,z),(2.0,3.1,.35),'Placas acero',axis='X')
            walls.cassette((side*15.26,2.4,z),(1.55,2.7,.08),'Louver interior',axis='X')
        # Glazed inhabited strip: dense small apertures, not oversized emissive bars.
        walls.cassette((side*16.6,7.1,z),(9.2,3.4,.8),'Placas acero',axis='X',cut=.4)
        walls.cassette((side*16.1,7.1,z),(8.6,2.8,.25),'Recess shadow',axis='X')
        for col in range(13):
            for row in range(3):
                lights.box((side*15.91,6.1+row*.62,z-3.8+col*.63),(.04,.32,.30),
                    'Low amber' if (col+row+sector)%7==0 else 'Low ivory' if (col+row)%3 else 'Low ice')
        # Monumental tapered buttress based on the exterior root shields.
        zz=z-5.0
        walls.beam((side*12.7,.15,zz),(side*14.3,7.8,zz),.9,1.3,'Placas plata')
        walls.beam((side*14.3,7.8,zz),(side*10.4,12.4,zz),.7,1.3,'Titanio azulado')
        walls.beam((side*13.2,.5,zz+.6),(side*14.7,7.6,zz+.6),.18,.13,'Juntas oscuras')
        for y in [1.2,2.0,2.8]:
            works.cassette((side*13.0,y,zz+.75),(.7,.45,.16),'Placas acero')
        for yy in [.7,1.1]: works.tube([(side*16,yy,z-5),(side*16,yy,z+5)],.13,'Placas acero')
        works.tube([(side*16,1.2,z+4.5),(side*15.2,1.2,z+4.5),(side*15.2,4.9,z+4.5)],.12,'Brushed edge')
    # Grated walkways, narrow railings and local landings, not solid gallery slabs.
    for z in range(-25,27,2):
        for j in range(8): gallery.box((side*14.8,4.55,z+j*.21),(3.1,.06,.07),'Brushed edge')
        gallery.box((side*13.1,5.1,z),(.08,1.2,.08),'Placas plata')
        gallery.box((side*13.1,5.65,z),(.08,.08,2.0),'Placas plata')
    for z in [-18,13]:
        for k in range(15): gallery.box((side*14.9,.16+k*.29,z+k*.32),(2.5,.17,.4),'Placas acero')
        for dx in [-1.3,1.3]: gallery.beam((side*14.9+dx,1,z),(side*14.9+dx,5.4,z+4.6),.08,.08,'Placas plata')
    walls.finish(.035);works.finish(.014);gallery.finish(.015)

# FLOOR: radial maintenance crown around the immutable 4.5 m turntable.
# Architecture above deck level never enters r < 4.85, so nothing covers the platform.
floor=Mesh('11 | Radial segmented deck and maintenance wells')
grates=Mesh('12 | Open service trench grating and fasteners')
floor.box((0,-.6,0),(36,.5,56),'Recess shadow')
for k in range(30):
    a=2*pi*k/30+.008;b=2*pi*(k+1)/30-.008
    floor.sector(4.85,7.4,a,b,.015,.18,'Placas acero' if k%3 else 'Titanio azulado')
    floor.sector(7.48,9.9,a,b,-.04,.14,'Blindaje grafito' if k%3 else 'Oiled steel')
    floor.sector(4.88,5.05,a+.015,b-.015,.035,.035,'Brushed edge')
    floor.sector(7.12,7.30,a+.015,b-.015,.035,.035,'Marcas de panel')
    for r in [5.4,6.6]:
        grates.tube([(r*cos(a+.025),.025,r*sin(a+.025)),(r*cos(a+.025),.075,r*sin(a+.025))],.055,'Brushed edge',6)
# Deliberate annular service trench: a recessed bed and radial grating connect the
# circular crown to the rectilinear service deck, rather than leaving black wedges.
floor.sector(9.94,11.85,0,2*pi,-.22,.18,'Recess shadow',steps=100)
for k in range(160):
    a=2*pi*k/160
    grates.beam((9.95*cos(a),.015,9.95*sin(a)),(11.75*cos(a),.015,11.75*sin(a)),.055,.075,'Placas acero')
for r in [10.1,11.5]:
    floor.sector(r,r+.07,0,2*pi,.025,.05,'Oiled steel',steps=100)
for x in range(-16,18,2):
    for z in range(-26,28,3):
        closest=max(abs(x)-.97,0)**2+max(abs(z)-1.45,0)**2
        if closest<100: continue
        mat='Placas acero' if (x+z)%5 else 'Blindaje grafito'
        floor.cassette((x,-.035,z),(1.93,2.92,.12),mat,axis='Y',cut=.16)
        # Inset non-slip strips and removable inspection panels, visible in foreground.
        if (x+z)%3==0:
            floor.cassette((x,.035,z),(1.45,2.15,.045),'Oiled steel',axis='Y')
            for j in range(5): grates.box((x,.065,z-.72+j*.36),(1.15,.025,.045),'Marcas de panel')
        else:
            floor.box((x,.035,z+1.0),(.65,.035,.14),'Juntas oscuras')
for side in [-1,1]:
    for x in [side*10.8,side*16.7]:
        floor.box((x,-.10,0),(.75,.2,53),'Recess shadow')
        for z in range(-26,27):
            for j in range(4): grates.box((x,.065,z+j*.24),(.66,.045,.06),'Placas acero')
    for z in range(-24,-10,2):
        floor.box((side*6.1,.06,z),(.08,.035,.8),'Marcas de panel')
        lights.box((side*6.1,.09,z),(.06,.02,.22),'Low ice')
    # Flush service pedestals stay outside every ship's exhibition radius.
    for z in [-15,10]:
        floor.cassette((side*11.6,.5,z),(1.15,1.0,.9),'Blindaje grafito')
        floor.cassette((side*11.6,1.05,z),(1.2,.85,.1),'Brushed edge',axis='Y')
        lights.box((side*11.6,1.12,z),(.72,.03,.48),'Status cyan')

# REAR: layered internal transit airlock, mezzanine bridge and service machinery.
rear=Mesh('13 | Internal transit lock and bridge abutments')
rear.box((0,6.0,27.8),(36,12,.6),'Blindaje grafito')
for x in [-11,11]:
    rear.cassette((x,3.0,27.25),(11.5,5.0,.6),'Titanio azulado',cut=.6)
    rear.cassette((x,3.0,26.85),(10.4,3.8,.22),'Recess shadow')
    for i in range(14): rear.box((x-4.5+i*.7,3,26.65),(.14,3.35,.15),'Placas acero')
for width,height,depth,mat in [(8.4,7,.8,'Placas plata'),(7.5,6.3,.55,'Juntas oscuras'),(6.6,5.8,.3,'Placas acero')]:
    rear.cassette((0,height*.5,26.8-depth*.5),(width,height,depth),mat,cut=.7)
rear.box((0,3,25.5),(.06,5.4,.06),'Juntas oscuras')
rear.box((0,7.4,26.2),(24,.5,2.4),'Placas plata')
for x in range(-11,12):
    rear.box((x,8.2,25.2),(.07,1.3,.07),'Placas plata')
    rear.box((x,8.8,25.2),(1.0,.06,.06),'Placas plata')
    lights.box((x,9.5,27.35),(.3,.22,.05),'Low ivory')

# EXIT: a deep octagonal pressure vestibule with retracted door packs, guides,
# hinge cartridges and hydraulic links. The opening remains genuinely empty.
gate=Mesh('14 | Octagonal pressure vestibule and retracted door mechanisms')
profile=[(-10,0),(-10,9.7),(-8.1,12),(8.1,12),(10,9.7),(10,0)]
for z,width,depth,mat in [(-27.0,.7,1.1,'Placas plata'),(-28.2,.28,.5,'Juntas oscuras'),(-29.3,.55,.8,'Placas acero'),(-31.0,.35,.45,'Brushed edge')]:
    for (x,y),(xx,yy) in zip(profile,profile[1:]): gate.beam((x,y,z),(xx,yy,z),width,depth,mat)
for side in [-1,1]:
    gate.cassette((side*14.1,5.8,-28),(7.8,11.8,1.4),'Blindaje grafito',cut=.7)
    for xoff in [0,.65,1.3]:
        gate.cassette((side*(12.3+xoff),5.7,-26.7+xoff*.12),(.50,10.4,.5),'Titanio azulado',cut=.15)
    for y in [1.2,3.6,6.0,8.4]:
        gate.cassette((side*10.8,y,-26.3),(1.15,1.4,.8),'Placas acero')
        gate.tube([(side*11.8,y-.3,-26.0),(side*10.9,y-.3,-25.7),(side*10.4,y+.5,-26.3)],.12,'Brushed edge')
        for dx in [-.32,.32]: grates.tube([(side*10.8+dx,y,-25.88),(side*10.8+dx,y,-25.78)],.06,'Oiled steel',6)
    lights.tube([(side*9.7,.4,-27.15),(side*9.7,9.5,-27.15),(side*7.95,11.6,-27.15)],.045,'Low ice',6)
gate.box((0,14.1,-28),(21.4,3.7,1.5),'Blindaje grafito')
for j in range(5):
    x=(j-2)*3.7
    gate.cassette((x,13.5,-26.8),(2.8,1.8,1.1),'Placas plata',cut=.3)
    gate.cassette((x,13.5,-26.16),(2.15,1.2,.12),'Recess shadow')
    for i in [-.55,0,.55]: lights.box((x+i,13.5,-26.06),(.12,.62,.05),'Status cyan')
gate.box((0,-.13,-29.5),(20,.25,5.5),'Placas acero')
for x in range(-9,10):
    grates.box((x,-.02,-29.3),(.10,.04,4.3),'Oiled steel')
for m,bevel in [(shell,.018),(crown,.025),(roof,.02),(lights,0),(floor,.015),(grates,0),(rear,.025),(gate,.035)]: m.finish(bevel)

def label(text,loc,size):
    cu=bpy.data.curves.new(text,'FONT');cu.body=text;cu.size=size;cu.extrude=.003;cu.align_x='CENTER'
    ob=bpy.data.objects.new(text,cu);architecture.objects.link(ob);ob.location=p(loc);ob.rotation_euler=(pi/2,0,0)
    cu.materials.append(mats['Marcas de panel']);bpy.context.view_layer.objects.active=ob
    ob.select_set(True);bpy.ops.object.convert(target='MESH');ob.select_set(False)
label('P R E S I D I U M   /   0 5',(0,12.4,-26.05),.32)
label('T R A N S I T   /   S E R V I C E',(0,6.7,25.45),.25)
# Static architecture only. The platform, ships, review lights and cameras never export.
try: from io_scene_gltf2.blender.exp.animation.animations import gather_animations
except ModuleNotFoundError:
    stub=types.ModuleType('io_scene_gltf2.blender.exp.animation.animations');stub.gather_animations=lambda *a,**k: []
    sys.modules[stub.__name__]=stub
bpy.ops.object.select_all(action='DESELECT')
for ob in architecture.objects: ob.select_set(True)
glb=BASE/'exports/citadel_hangar_phase1.glb'
bpy.ops.export_scene.gltf(filepath=str(glb),export_format='GLB',use_selection=True,export_apply=True,
    export_animations=False,export_cameras=False,export_lights=False)

def to_preview(ob):
    for col in list(ob.users_collection): col.objects.unlink(ob)
    preview.objects.link(ob)
# Render reference of the unchanged Rust platform, explicitly excluded from export.
for radius,y,height,mat in [(4.5,.36,.72,'Titanio azulado'),(4.54,.74,.04,'Status cyan'),(4.35,.78,.04,'Juntas oscuras')]:
    bpy.ops.mesh.primitive_cylinder_add(vertices=64,radius=radius,depth=height,location=p((0,y,0)))
    ob=bpy.context.object;ob.name='REVIEW ONLY | frozen Rust platform';ob.data.materials.append(mats[mat]);to_preview(ob)

# Actual ship as a review reference. Geometry and source materials remain untouched.
cfg=json.loads((ROOT/'config/ships.json').read_text(encoding='utf-8'))
ship=next(s for s in cfg['ships'] if s['id']=='starwars_mandalorian')
old=set(bpy.data.objects)
bpy.ops.import_scene.gltf(filepath=str(ROOT/ship['model']))
ship_objects=[o for o in bpy.data.objects if o not in old]
ship_meshes=[o for o in ship_objects if o.type=='MESH']
bpy.context.view_layer.update()
pts=[o.matrix_world@Vector(v) for o in ship_meshes for v in o.bound_box]
lo=Vector(tuple(min(v[i] for v in pts) for i in range(3)));hi=Vector(tuple(max(v[i] for v in pts) for i in range(3)))
center=(lo+hi)/2;half=(hi-lo)/2;scale=6.0/(2*max(half));half*=scale
for ob in ship_meshes:
    world=ob.matrix_world.copy();ob.parent=None;ob.matrix_world=world
    ob.location=(ob.location-center)*scale+Vector((0,0,.8+half.z));ob.scale*=scale
for ob in ship_objects: ob.name='REVIEW ONLY | '+ob.name;to_preview(ob)

world=bpy.data.worlds.new('Cold space');world.use_nodes=True
bg=next(n for n in world.node_tree.nodes if n.type=='BACKGROUND');bg.inputs[0].default_value=(.012,.019,.035,1);bg.inputs[1].default_value=.3;scene.world=world
def lamp(name,loc,target,power,size,color):
    data=bpy.data.lights.new(name,'AREA');data.energy=power;data.shape='DISK';data.size=size;data.color=color
    ob=bpy.data.objects.new(name,data);preview.objects.link(ob);ob.location=p(loc)
    ob.rotation_euler=(Vector(p(target))-ob.location).to_track_quat('-Z','Y').to_euler()
for z in [-20,-5,10,23]: lamp('Overhead service luminaire',(0,13,z),(0,0,z),1800,9,(.73,.85,1))
lamp('Key matching Rust',(-4,9,5),(0,1,0),2200,8,(.86,.91,1))
for side in [-1,1]: lamp('Gallery bounce',(side*12,7,3),(0,3,-3),950,8,(.73,.84,1))
lamp('Lock fill',(0,8,-27),(0,4,-12),900,9,(.50,.72,1))
def camera(name,loc,target,fov):
    data=bpy.data.cameras.new(name);data.sensor_fit='VERTICAL';data.sensor_height=24;data.lens=24/(2*tan(fov*pi/360))
    ob=bpy.data.objects.new(name,data);preview.objects.link(ob);ob.location=p(loc)
    ob.rotation_euler=(Vector(p(target))-ob.location).to_track_quat('-Z','Y').to_euler();return ob
# Exact raylib framing equation at 1280x800, including the unchanged 151/156 HUD insets.
gamehalf=Vector((half.x,half.z,half.y));target=Vector((0,.8+half.z,0))
angle=min(__import__('math').atan(tan(21*pi/180)*493/800),__import__('math').atan(tan(21*pi/180)*(1280/800)*(1100/1280)))
distance=max(11,gamehalf.length/sin(angle)*1.08)
eye=target+Vector((.9,.6,1.4)).normalized()*distance
cameras=[camera('01 | Same game camera',eye,target,42),
    camera('02 | Architecture',(11.88,10,22.96),(0,4.96,-10.08),65),
    camera('03 | Wall detail',(1.5,5.6,6),(-15,4.8,-4),49),
    camera('04 | Ceiling',(0,5,17),(0,12,-8),65),
    camera('05 | Pressure vestibule',(0,6,7.84),(0,6,-28),60),
    camera('06 | Rear transit',(8,8,-12),(0,5,26),60)]
scene.render.engine='CYCLES';scene.cycles.samples=32;scene.cycles.use_denoising=True
scene.render.resolution_x=1280;scene.render.resolution_y=800;scene.render.resolution_percentage=100
scene.camera=cameras[0]
bpy.ops.wm.save_as_mainfile(filepath=str(BASE/'citadel_hangar_phase1.blend'))
for cam,filename in zip(cameras,['blender-game-camera.png','blender-general.png','blender-wall-detail.png','blender-ceiling.png','blender-exit.png','blender-rear.png']):
    scene.camera=cam;scene.render.filepath=str(OUT/filename);bpy.ops.render.render(write_still=True)
assert hashlib.sha256(source.read_bytes()).hexdigest()==source_hash
report={'source_sha256':source_hash,'source_unchanged':True,'meshes':len(architecture.objects),
    'glb_bytes':glb.stat().st_size,'camera_game':{'eye':list(eye),'target':list(target),'fov_y':42,'viewport':[1280,800],'hud':[151,156]},
    'export_excludes':['platform','ship','lights','cameras'],'reference_materials':list(mats),
    'platform_clear_radius':4.85,'aperture':{'width':20,'height':12},'units':'game metres, Blender Z-up to GLB Y-up'}
(OUT/'build-manifest.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print('PHASE1_BUILD_PASS',flush=True)
