"""Complementary Presidium service module. Never saves the approved source.
Run with Blender -b --python tools/blender/spaceStation/build_hangar.py.
Geometry below uses game coordinates (X right, Y up, -Z launch).
"""
from pathlib import Path
from math import sin, cos, pi
import bpy, bmesh, json, hashlib, sys, types
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT/'assets/models/environment/spaceStation'
OUT = ROOT/'docs/references/hangar'
OUT.mkdir(parents=True, exist_ok=True)
source = BASE/'citadel_space_station.blend'
source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
bpy.ops.wm.open_mainfile(filepath=str(source))
materials = {m.name.removeprefix('CI | '): m for m in bpy.data.materials if m.name.startswith('CI | ')}
scene = bpy.data.scenes.new('Presidium | Hangar interior')
bpy.context.window.scene = scene
# Remove original scene only in memory; preserve its material datablocks for reuse.
for s in list(bpy.data.scenes):
    if s != scene: bpy.data.scenes.remove(s)
for o in list(bpy.data.objects): bpy.data.objects.remove(o, do_unlink=True)

def p(v): return (v[0], -v[2], v[1])

class Mesh:
    def __init__(self, name): self.name, self.v, self.f, self.m = name, [], [], []
    def add(self, verts, faces, mat):
        n = len(self.v)
        self.v.extend(p(v) for v in verts)
        self.f.extend(tuple(n+i for i in face) for face in faces)
        self.m.extend([mat]*len(faces))
    def box(self, c, d, mat):
        v = [(c[0]+a*d[0]/2,c[1]+b*d[1]/2,c[2]+e*d[2]/2)
             for e in [-1,1] for b in [-1,1] for a in [-1,1]]
        self.add(v, [(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)], mat)
    def beam(self, a, b, width, depth, mat):
        a,b = Vector(a),Vector(b)
        t = (b-a).normalized()
        ref = Vector((0,0,1)) if abs(t.z)<.9 else Vector((0,1,0))
        u=t.cross(ref).normalized()*width/2
        v=t.cross(u).normalized()*depth/2
        verts=[q+u*i+v*j for q in [a,b] for j in [-1,1] for i in [-1,1]]
        self.add(verts,[(0,1,3,2),(4,6,7,5),(0,4,5,1),(2,3,7,6),(0,2,6,4),(1,5,7,3)],mat)
    def arch(self, z, r, thick, depth, mat, start=0, end=pi, count=24, cy=-2):
        for i in range(count):
            a=start+(end-start)*i/count
            b=start+(end-start)*(i+1)/count
            verts=[(radius*cos(t),cy+radius*sin(t),zz)
                   for zz in [z-depth/2,z+depth/2] for radius in [r,r+thick] for t in [a,b]]
            self.add(verts,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],mat)
    def finish(self, bevel=0):
        me=bpy.data.meshes.new(self.name)
        me.from_pydata(self.v,[],self.f)
        names=list(materials)
        for name in names: me.materials.append(materials[name])
        for poly,name in zip(me.polygons,self.m): poly.material_index=names.index(name)
        bm=bmesh.new();bm.from_mesh(me)
        bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free()
        ob=bpy.data.objects.new(self.name,me);scene.collection.objects.link(ob)
        if bevel:
            mod=ob.modifiers.new('Armor edge', 'BEVEL');mod.width=bevel;mod.segments=1
        return ob

# Layered barrel shell; facet spacing and silver edge bands continue the ring armor.
shell=Mesh('01 | Faceted Presidium pressure shell')
armor=Mesh('02 | Staggered armor cassettes')
ribs=Mesh('03 | Concentric ring frames and keel')
lamps=Mesh('04 | Recessed architectural luminaires')
for z in range(-26,28,6):
    shell.arch(z,18.1,.5,5.96,'Blindaje grafito',count=20)
    for k in range(20):
        a=k*pi/20+.012;b=(k+1)*pi/20-.012
        armor.arch(z,17.94,.13,5.6,'Placas acero' if (k+z)%3 else 'Titanio azulado',a,b,1)
        if 3<=k<=16:
            # Small inset plate on each roof cassette, leaving visible seams.
            armor.arch(z+.2,17.88,.05,3.4,'Placas plata' if k%4==0 else 'Titanio azulado',a+.02,b-.02,1)
for z in [-27,-19,-11,-3,5,13,21,27]:
    ribs.arch(z,17.25,.65,.6,'Placas plata',count=20)
    ribs.arch(z,17.08,.17,1.0,'Juntas oscuras',count=20)
    ribs.arch(z,17.0,.1,.12,'Marcas de panel',count=20)
for x in [-9,0,9]:
    y=(18**2-x*x)**.5-2
    ribs.box((x,y-.5,0),(.55,.7,55),'Blindaje grafito')
    for z in range(-24,26,6):
        lamps.box((x,y-.88,z),(.22,.05,3.1),'Luz hielo')

# Floor panels, radial collar around the independently rendered turntable.
deck=Mesh('05 | Deck plates and turntable foundation')
deck.box((0,-.35,0),(36,.6,56),'Juntas oscuras')
for x in range(-16,18,2):
    for z in range(-26,28,4):
        deck.box((x,-.025,z),(1.94,.08,3.92),'Placas acero' if (x+z)%6 else 'Titanio azulado')
for i in range(80):
    a=2*pi*i/80;b=2*pi*(i+1)/80
    deck.beam((5.0*cos(a),.06,5.0*sin(a)),(5.0*cos(b),.06,5.0*sin(b)),.22,.1,'Placas plata')
    if i%2==0:
        deck.beam((5.35*cos(a),.055,5.35*sin(a)),(5.75*cos(a),.055,5.75*sin(a)),.12,.05,'Luz marfil')
for side in [-1,1]:
    deck.box((side*6.3,.04,-12),(.08,.03,23),'Marcas de panel')
    for z in range(-24,-6,3):
        lamps.box((side*6.3,.07,z),(.09,.02,.7),'Luz hielo')
    for z in range(-24,27,2):
        for x in [side*10,side*15.7]:
            deck.box((x,.04,z),(.5,.04,.05),'Juntas oscuras')

# Occupied side galleries: structural buttresses, layered walls, warm office windows,
# secondary access doors, service racks and open-railed maintenance mezzanines.
walls=Mesh('06 | Deep service walls and five district buttresses')
gallery=Mesh('07 | Elevated service galleries and stairs')
details=Mesh('08 | Maintenance racks conduits and consoles')
for side in [-1,1]:
    walls.box((side*17.4,1.4,0),(1.2,3.2,56),'Blindaje grafito')
    gallery.box((side*14.7,3.6,0),(3.7,.36,52),'Placas acero')
    gallery.box((side*12.8,3.5,0),(.22,.7,52),'Placas plata')
    for z in range(-24,27,4):
        walls.box((side*16.7,5.5,z),(.8,7,3.7),'Titanio azulado')
        walls.box((side*16.23,6,z),(.18,2.5,3.25),'Juntas oscuras')
        for j in range(5):
            lamps.box((side*16.1,6,z-1.25+j*.6),(.04,1.6,.3),'Luz marfil' if j%3 else 'Luz ambar')
        # Below gallery: door with deep frame, paired mechanical cabinets.
        walls.box((side*16.4,1.55,z),(.6,2.8,2),'Placas plata')
        walls.box((side*16.05,1.5,z),(.1,2.4,1.45),'Juntas oscuras')
        walls.box((side*15.97,1.5,z),(.08,2.2,.05),'Marcas de panel')
        lamps.box((side*15.98,2.8,z),(.05,.08,1.3),'Luz hielo')
        gallery.box((side*12.8,4.25,z),(.1,1.3,.1),'Placas plata')
        for y in [4.4,4.85]: gallery.box((side*12.8,y,z),(.08,.08,4),'Placas plata')
    for z in [-23,-13,-3,7,17]:
        walls.beam((side*12.4,.1,z),(side*14.5,8.4,z),1.1,1.35,'Placas plata')
        walls.beam((side*14.5,8.4,z),(side*10.5,12.4,z),.8,1.35,'Titanio azulado')
        walls.beam((side*12.9,1,z+.72),(side*14.65,7.8,z+.72),.22,.08,'Blindaje grafito')
        for y in [1.5,2,2.5]:
            details.box((side*13.5,y,z+.78),(.7,.09,.05),'Juntas oscuras')
    for z in [-18,10]:
        for k in range(12):
            gallery.box((side*14.5,.15+k*.3,z+k*.36),(2.3,.3,.42),'Placas acero')
        for xoff in [-1.25,1.25]:
            gallery.beam((side*14.5+xoff,1,z),(side*14.5+xoff,4.6,z+4.1),.08,.08,'Placas plata')
    for z in [-16,-8,8,16]:
        details.box((side*10.9,.7,z),(1.25,1.4,2),'Blindaje grafito')
        details.box((side*10.9,1.45,z),(1.35,.16,2.1),'Placas plata')
        lamps.box((side*10.9,1.55,z),(.9,.05,1.35),'Canales')
        for j in range(5):
            details.box((side*11.57,.3+j*.21,z),(.04,.08,1.7),'Marcas de panel')
    for y in [.6,1.0,1.4]:
        details.box((side*16,y,0),(.18,.18,54),'Placas plata')

# Full rear wall, secondary airlock, layered launch vestibule and five couplings.
gate=Mesh('09 | Launch aperture layered pressure jambs')
for side in [-1,1]:
    gate.box((side*14,6,-27.8),(8,12,1),'Blindaje grafito')
    for z in [-27,-29,-31]:
        gate.box((side*10.3,5.9,z),(.6,11.8,.55),'Placas plata')
        gate.box((0,12,z),(21.2,.6,.55),'Placas plata')
    gate.box((side*12.1,5,-26.8),(2.7,10,1.3),'Titanio azulado')
    for y in range(1,10):
        gate.box((side*12.1,y,-26.05),(1.8,.12,.08),'Juntas oscuras')
    lamps.box((side*9.92,6,-27),(.08,11.3,.1),'Luz hielo')
gate.box((0,14.1,-28),(21.2,3.6,1),'Blindaje grafito')
for i in range(5):
    gate.box(((i-2)*3.5,13.7,-27.2),(2.6,1.7,.8),'Placas plata')
    lamps.box(((i-2)*3.5,13.7,-26.76),(.14,1.1,.06),'Luz cian')
# Short external threshold remains open to the sky; nothing spans the hatch.
gate.box((0,-.2,-29.4),(20,.4,4),'Placas acero')
gate.box((0,6,27.9),(36,12,.5),'Blindaje grafito')
gate.box((0,2.8,27.5),(4.6,5.6,.5),'Placas plata')
gate.box((0,2.7,27.17),(3.7,5.2,.1),'Juntas oscuras')
for m in [shell,armor,ribs,lamps,deck,walls,gallery,details,gate]: m.finish(.025 if m not in [lamps,deck] else 0)

# Discreet physical signage; converted to geometry for raylib.
def label(text,loc,size):
    cu=bpy.data.curves.new(text,'FONT');cu.body=text;cu.size=size;cu.extrude=.002;cu.align_x='CENTER'
    ob=bpy.data.objects.new(text,cu);scene.collection.objects.link(ob);ob.location=p(loc)
    ob.rotation_euler=(pi/2,0,0);cu.materials.append(materials['Marcas de panel'])
    bpy.context.view_layer.objects.active=ob;ob.select_set(True);bpy.ops.object.convert(target='MESH');ob.select_set(False)
label('P R E S I D I U M   /   0 5',(0,12.55,-26.7),.4)
label('SERVICE  /  TRANSIT',(0,5.8,27.1),.26)
architecture=[o for o in scene.objects if o.type=='MESH']

# Static export, same units/origin as the Rust platform. Avoid exporter animation
# import bug in this local alpha distribution, as in the repository's other tools.
try:
    from io_scene_gltf2.blender.exp.animation.animations import gather_animations
except ModuleNotFoundError:
    stub=types.ModuleType('io_scene_gltf2.blender.exp.animation.animations')
    stub.gather_animations=lambda *args,**kwargs: []
    sys.modules[stub.__name__]=stub
bpy.ops.object.select_all(action='DESELECT')
for ob in architecture: ob.select_set(True)
glb=BASE/'exports/citadel_hangar_interior.glb'
bpy.ops.export_scene.gltf(filepath=str(glb),export_format='GLB',use_selection=True,
    export_apply=True,export_animations=False,export_cameras=False,export_lights=False)

# Preview platform only: Rust owns the real historical platform and its rotation.
bpy.ops.object.select_all(action='DESELECT')
bpy.ops.mesh.primitive_cylinder_add(vertices=80,radius=4.5,depth=.8,location=(0,0,.4))
ob=bpy.context.object;ob.name='PREVIEW ONLY | Rust turntable';ob.data.materials.append(materials['Titanio azulado'])
bpy.ops.mesh.primitive_cylinder_add(vertices=80,radius=4.35,depth=.02,location=(0,0,.81))
bpy.context.object.name='PREVIEW ONLY | Deck';bpy.context.object.data.materials.append(materials['Blindaje grafito'])

world=bpy.data.worlds.new('Space');world.use_nodes=True
bg=next((n for n in world.node_tree.nodes if n.type=='BACKGROUND'),None)
if bg is None:
    bg=world.node_tree.nodes.new('ShaderNodeBackground')
    output=world.node_tree.nodes.new('ShaderNodeOutputWorld')
    world.node_tree.links.new(bg.outputs[0],output.inputs[0])
bg.inputs[0].default_value=(.018,.026,.05,1)
bg.inputs[1].default_value=.35
scene.world=world
def light(loc,power,size):
    data=bpy.data.lights.new('Architectural fill','AREA');data.energy=power;data.shape='DISK';data.size=size
    ob=bpy.data.objects.new(data.name,data);scene.collection.objects.link(ob);ob.location=p(loc)
for z in [-20,-5,10,22]: light((0,13,z),2200,12)
def camera(name,loc,target,lens):
    data=bpy.data.cameras.new(name);data.lens=lens
    ob=bpy.data.objects.new(name,data);scene.collection.objects.link(ob);ob.location=p(loc)
    ob.rotation_euler=(Vector(p(target))-ob.location).to_track_quat('-Z','Y').to_euler();return ob
cams=[camera('Game approximate',(6.1,5.9,9.5),(0,1.7,0),27),
      camera('Architecture',(11,9,23),(0,6,-12),22),
      camera('Launch vestibule',(0,5,1),(0,6,-28),24)]
try: scene.render.engine='CYCLES'
except TypeError: pass
scene.cycles.samples=24
scene.render.resolution_x=1280;scene.render.resolution_y=800;scene.render.resolution_percentage=100
scene.camera=cams[0]
bpy.ops.wm.save_as_mainfile(filepath=str(BASE/'citadel_hangar_interior.blend'))
for cam,filename in zip(cams,['blender-game-camera.png','blender-interior.png','blender-aperture.png']):
    scene.camera=cam;scene.render.filepath=str(OUT/filename);bpy.ops.render.render(write_still=True)
assert hashlib.sha256(source.read_bytes()).hexdigest()==source_hash
(OUT/'build-manifest.json').write_text(json.dumps({'source_sha256':source_hash,
    'source_unchanged':True,'glb':str(glb.relative_to(ROOT)),'bytes':glb.stat().st_size,
    'architecture_meshes':len(architecture),'materials_from_source':list(materials),
    'units':'metres, Blender Z-up exported glTF Y-up; identity transform in Rust',
    'preview_platform_excluded_from_glb':True},indent=2),encoding='utf-8')
print('HANGAR_BUILD_PASS',flush=True)
