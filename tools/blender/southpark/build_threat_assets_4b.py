"""Standalone derived phase-4B models. Run with Blender --background --python.
Never opens or rewrites original town/planet resources or phase-4A models.
Metres, Blender Z up; GLB Y up. Front of heaters/snowman is Blender +Y.
"""
import bpy
import bmesh
import json
import math
import runpy
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'assets/models/minigames/southpark/threats/phase4b'
OUT.mkdir(parents=True, exist_ok=True)
export_glb = runpy.run_path(str(Path(__file__).with_name('build_explorable_town.py')))['export_static_glb']
bpy.context.preferences.filepaths.save_version = 0
materials = {}
for name, rgb in {
    'Ebony': (.025,.018,.025), 'Wood': (.13,.055,.025),
    'Ivory': (.96,.95,.87), 'Metal': (.22,.27,.32),
    'Yellow': (.97,.65,.035), 'Red': (.75,.035,.025),
    'Snow': (.92,.96,.99), 'Coal': (.015,.025,.035),
    'Carrot': (.98,.26,.025), 'Branch': (.19,.085,.035),
    'Heat': (.65,.13,.025), 'Cable': (.10,.12,.15),
}.items():
    mat = bpy.data.materials.new('SPT_4B_'+name)
    mat.use_nodes = True
    shader = next(n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    shader.inputs['Base Color'].default_value = (*rgb,1)
    shader.inputs['Roughness'].default_value = .78
    mat.diffuse_color = (*rgb,1)
    materials[name] = mat

def scene(name):
    s = bpy.data.scenes.new('SouthPark_4B_'+name)
    bpy.context.window.scene = s
    return s

def finish(o, name, mat):
    o.name = name
    o.data.materials.append(materials[mat])
    return o

def box(name, p, size, mat):
    bpy.ops.mesh.primitive_cube_add(size=1, location=p)
    o = bpy.context.object
    o.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    return finish(o,name,mat)

def ball(name,p,size,mat):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=6, location=p)
    o=bpy.context.object
    o.scale=size
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    return finish(o,name,mat)

def cylinder(name,p,r,depth,mat,rotation=(0,0,0)):
    bpy.ops.mesh.primitive_cylinder_add(vertices=12,radius=r,depth=depth,location=p,rotation=rotation)
    return finish(bpy.context.object,name,mat)

def rod(name,a,b,r,mat):
    a,b=Vector(a),Vector(b)
    o=cylinder(name,(a+b)/2,r,(b-a).length,mat)
    o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
    return o

def shape(name,outline,z,depth,mat):
    n=len(outline)
    verts=[(x,y,z+d) for d in (0,depth) for x,y in outline]
    faces=[tuple(reversed(range(n))),tuple(range(n,2*n))]
    faces += [(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    mesh=bpy.data.meshes.new(name)
    mesh.from_pydata(verts,[],faces)
    mesh.update()
    o=bpy.data.objects.new(name,mesh)
    bpy.context.collection.objects.link(o)
    return finish(o,name,mat)

audit={}
def export(name):
    objects=[o for o in bpy.context.scene.objects if o.type=='MESH']
    for s in bpy.data.scenes:
        for layer in s.view_layers:
            for o in layer.objects: o.select_set(False,view_layer=layer)
    for o in objects:
        o.select_set(True)
        bm=bmesh.new(); bm.from_mesh(o.data)
        bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
        bm.to_mesh(o.data); bm.free()
    bpy.context.view_layer.objects.active=objects[0]
    bpy.context.view_layer.update()
    export_glb(OUT/(name+'.glb'))
    points=[o.matrix_world@Vector(v) for o in objects for v in o.bound_box]
    tris=0
    for o in objects:
        o.data.calc_loop_triangles(); tris+=len(o.data.loop_triangles)
    audit[name]={'triangles':tris,'objects':len(objects),
        'blender_bounds':[[min(p[i] for p in points) for i in range(3)],
                          [max(p[i] for p in points) for i in range(3)]]}

piano=scene('Grand_Piano')
outline=[(-1.65,-1.16),(-.25,-1.16),(.35,-.98),(.8,-.55),
         (1.6,-.35),(1.85,.0),(1.7,.6),(1.25,1.03),(.5,1.16),(-1.65,1.16)]
shape('Grand_body',outline,1.05,.44,'Ebony')
shape('Raised_grand_lid',outline,1.6,.09,'Ebony')
for x,y in [(-1.3,-.92),(-1.3,.92),(1.3,.23)]:
    box('Piano_leg',(x,y,.52),(.18,.18,1.04),'Wood')
    cylinder('Brass_foot',(x,y,.05),.13,.1,'Yellow')
rod('Lid_prop',(.25,-.7,1.48),(.25,-.7,1.61),.025,'Wood')
box('Keyboard_bed',(-1.86,0,1.14),(.48,2.4,.15),'Wood')
for i in range(21):
    y=-1.12+i*.112
    box('White_key_%02d'%i,(-1.92,y,1.24),(.42,.101,.065),'Ivory')
    if i%7 not in (2,6) and i<20:
        box('Black_key_%02d'%i,(-1.79,y+.056,1.29),(.22,.055,.06),'Ebony')
box('Music_stand',(-1.4,0,1.66),(.06,.65,.3),'Ebony')
export('piano')

support=scene('Piano_Support')
box('Crane_base',(0,-2.2,.12),(.85,.85,.24),'Metal')
box('Crane_mast',(0,-2.2,4.7),(.28,.28,9.4),'Yellow')
box('Overhead_beam',(0,-1.1,9.25),(.28,2.8,.28),'Yellow')
rod('Support_brace',(0,-2.2,7.9),(0,-.8,9.25),.095,'Metal')
rod('Suspension_rope',(0,0,9.2),(0,0,8.3),.035,'Cable')
export('piano_support')

lock=scene('Piano_Safety_Lock')
box('Safety_base',(0,0,.05),(.65,.5,.1),'Metal')
box('Safety_post',(0,0,.65),(.13,.13,1.3),'Metal')
box('Yellow_lock_housing',(0,0,1.35),(.76,.35,.7),'Yellow')
cylinder('Red_safety_button',(0,.23,1.35),.18,.13,'Red',(math.pi/2,0,0))
box('Lock_symbol_bar',(0,.2,1.62),(.25,.05,.05),'Coal')
export('piano_lock')

snowman=scene('Killer_Snowman')
ball('Snow_base',(0,0,.68),(.75,.7,.68),'Snow')
ball('Snow_middle',(0,0,1.66),(.58,.53,.6),'Snow')
ball('Snow_head',(0,0,2.5),(.45,.43,.45),'Snow')
for x in (-.18,.18):
    ball('Coal_eye',(x,.393,2.57),(.072,.055,.075),'Coal')
    brow=box('Angry_eyebrow',(x,.406,2.7),(.19,.04,.045),'Coal')
    brow.rotation_euler.y=math.copysign(.32,x)
bpy.ops.mesh.primitive_cone_add(vertices=8,radius1=.09,radius2=.008,depth=.42,
    location=(0,.57,2.47),rotation=(-math.pi/2,0,0))
finish(bpy.context.object,'Carrot_nose','Carrot')
for i in range(7):
    x=(i-3)*.075
    ball('Sinister_mouth',(x,.39,2.29-abs(i-3)*.017),(.034,.04,.034),'Coal')
for z in (1.38,1.65,1.91): ball('Coal_button',(0,.52,z),(.06,.05,.06),'Coal')
for side in (-1,1):
    rod('Branch_arm',(side*.42,0,1.8),(side*1.02,0,2.1),.06,'Branch')
    rod('Branch_finger',(side*.85,0,2.0),(side*.96,.15,2.29),.035,'Branch')
cylinder('Hat_brim',(0,0,2.91),.58,.09,'Ebony')
cylinder('Hat_crown',(0,0,3.08),.34,.3,'Ebony')
cylinder('Hat_red_band',(0,0,2.97),.35,.08,'Red')
export('snowman')

heater=scene('Heater')
box('Heater_base',(0,0,.08),(.9,.7,.16),'Metal')
box('Heater_post',(0,0,.36),(.18,.18,.5),'Coal')
box('Heater_housing',(0,0,.92),(.78,.42,.9),'Metal')
box('Heat_panel',(0,.22,.92),(.64,.04,.7),'Heat')
for x in (-.24,-.12,0,.12,.24): box('Grille_bar',(x,.27,.92),(.025,.04,.74),'Coal')
box('Power_indicator',(0,.26,1.32),(.22,.04,.08),'Red')
export('heater')

bpy.context.window.scene=piano
keep=(piano,support,lock,snowman,heater)
for s in list(bpy.data.scenes):
    if s not in keep:
        for o in list(s.objects): bpy.data.objects.remove(o,do_unlink=True)
        bpy.data.scenes.remove(s)
bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'southpark_threats_4b.blend'),compress=True)
(OUT/'asset-audit.json').write_text(json.dumps(audit,indent=2),encoding='utf-8')
print('PHASE4B_ASSETS',json.dumps(audit))
