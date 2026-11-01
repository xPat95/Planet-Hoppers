"""Derived low-poly phase-4A assets. Blender --background --python this_file.

Never reads or saves over any existing town, planet, ship or character resource.
Blender: Z up, bus +X forward. Export: metres, Y up, same scale.
"""
import bpy
import bmesh
import math
import json
import runpy
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'assets/models/minigames/southpark/threats'
export_static_glb = runpy.run_path(str(Path(__file__).with_name('build_explorable_town.py')))['export_static_glb']
OUT.mkdir(parents=True, exist_ok=True)
bpy.context.preferences.filepaths.save_version = 0
materials = {}
for name, rgb in {
    'Yellow':(.95,.57,.035),'Black':(.025,.03,.04),'Glass':(.045,.12,.18),
    'White':(.96,.94,.82),'Red':(.72,.045,.025),'Metal':(.36,.42,.48),
    'Dome':(.19,.51,.59),'Rim':(.18,.23,.27),'Hole':(.012,.016,.02),
}.items():
    mat=bpy.data.materials.new('SPT_Threat_'+name)
    mat.use_nodes=True
    node=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    node.inputs['Base Color'].default_value=(*rgb,1)
    node.inputs['Roughness'].default_value=.8
    mat.diffuse_color=(*rgb,1)
    materials[name]=mat

def scene(name):
    result=bpy.data.scenes.new('SouthPark_Threat_'+name)
    bpy.context.window.scene=result
    result.unit_settings.system='METRIC'
    result['units']='meters; Blender Z up; glTF Y up'
    return result

def finish(obj,name,material):
    obj.name=name
    obj.data.materials.append(materials[material])
    return obj

def box(name,location,size,material):
    bpy.ops.mesh.primitive_cube_add(size=1,location=location)
    obj=bpy.context.object
    obj.scale=size
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    return finish(obj,name,material)

def sphere(name,location,size,material):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16,ring_count=8,location=location)
    obj=bpy.context.object;obj.scale=size
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    return finish(obj,name,material)

def cylinder(name,location,radius,depth,material,rotation=(0,0,0)):
    bpy.ops.mesh.primitive_cylinder_add(vertices=16,radius=radius,depth=depth,location=location,rotation=rotation)
    return finish(bpy.context.object,name,material)

def label(text,location,size,rotation):
    bpy.ops.object.text_add(location=location,rotation=rotation)
    obj=bpy.context.object;obj.data.body=text;obj.data.align_x='CENTER';obj.data.size=size;obj.data.extrude=.003
    obj.data.materials.append(materials['Black'])
    bpy.ops.object.convert(target='MESH')
    obj.name='Label_'+text

audit={}
def export(name,objects=None):
    objects=list(objects or [o for o in bpy.context.scene.objects if o.type=='MESH'])
    # Selection is stored per view layer. The glTF exporter can inspect every
    # scene, so deselect globally, including the startup cube and earlier assets.
    for scn in bpy.data.scenes:
        for layer in scn.view_layers:
            for obj in layer.objects:obj.select_set(False,view_layer=layer)
    for obj in objects:
        obj.select_set(True)
        bm=bmesh.new();bm.from_mesh(obj.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(obj.data);bm.free()
    bpy.context.view_layer.objects.active=objects[0]
    bpy.context.view_layer.update()
    export_static_glb(OUT/(name+'.glb'))
    points=[o.matrix_world@Vector(v) for o in objects for v in o.bound_box]
    triangles=0
    for o in objects:o.data.calc_loop_triangles();triangles+=len(o.data.loop_triangles)
    audit[name]={'triangles':triangles,'objects':len(objects),'blender_bounds':[[min(v[i] for v in points) for i in range(3)],[max(v[i] for v in points) for i in range(3)]]}

bus=scene('Bus')
box('Yellow_body',(0,0,1.65),(5.4,2.1,2.0),'Yellow')
box('Hood',(2.55,0,1.15),(.9,1.95,.7),'Yellow')
box('Front_windshield',(2.72,0,2.3),(.035,1.87,.72),'Glass')
box('Rear_window',(-2.72,0,2.3),(.035,1.8,.7),'Glass')
for y in (-1.066,1.066):
    for x in (-2.0,-1.25,-.5,.25,1,1.75):box('Side_window',(x,y,2.25),(.59,.025,.72),'Glass')
    for z in (1.25,1.43):box('Safety_stripe',(0,y,z),(5.35,.03,.055),'Black')
for x in (-1.9,1.9):
    for y in (-1.09,1.09):
        cylinder('Wheel',(x,y,.48),.48,.22,'Black',(math.pi/2,0,0))
        cylinder('Hub',(x,y*1.105,.48),.19,.03,'Metal',(math.pi/2,0,0))
for y in (-.7,.7):box('Headlight',(3.01,y,1.18),(.03,.24,.2),'White')
box('Front_bumper',(3.04,0,.75),(.12,2.13,.14),'Black')
label('SCHOOL BUS',(0,-1.09,1.78),.25,(math.pi/2,0,0))
export('bus')

ufo=scene('UFO')
sphere('Saucer',(0,0,0),(2.7,2.7,.48),'Metal')
cylinder('Outer_rim',(0,0,0),2.72,.15,'Rim')
sphere('Central_dome',(0,0,.5),(1.12,1.12,.76),'Dome')
cylinder('Tractor_emitter',(0,0,-.38),.65,.2,'Dome')
lights=[]
for i in range(3):
    angle=math.radians(30+i*120)
    x,y=2.25*math.cos(angle),-2.25*math.sin(angle)
    cylinder('Light_holder_'+str(i),(x,y,-.32),.33,.14,'Black')
    lights.append(sphere('Runtime_light_'+str(i),(x,y,-.45),(.25,.25,.25),'White'))
# Bulbs stay editable in Blender. Rust draws these exact three 0.25 m spheres
# to change each light independently without duplicating the saucer mesh.
export('ufo',[o for o in ufo.objects if o.type=='MESH' and o not in lights])

sewer=scene('Sewer')
cylinder('Dark_opening',(0,0,.035),.86,.04,'Hole')
bpy.ops.mesh.primitive_torus_add(major_segments=24,minor_segments=6,location=(0,0,.07),major_radius=.89,minor_radius=.085)
finish(bpy.context.object,'Metal_opening_rim','Rim')
opening=list(sewer.objects)
export('sewer',opening)
lid=cylinder('Separate_metal_lid',(0,0,.08),.86,.10,'Metal')
lid_parts=[lid]
for y in (-.45,-.15,.15,.45):
    width=2*math.sqrt(.78**2-y*y)
    lid_parts.append(box('Cover_ridge',(0,y,.14),(width,.035,.025),'Rim'))
export('lid',lid_parts)
# Keep the cover separate in the editable presentation, not over the hole.
for obj in lid_parts:obj.location.x+=2.4

panel=scene('Brake_Panel')
box('Stand',(0,0,.65),(.15,.15,1.3),'Metal')
box('Emergency_red_panel',(0,0,1.5),(.65,.22,.65),'Red')
cylinder('Emergency_button',(0,-.15,1.5),.17,.10,'Yellow',(math.pi/2,0,0))
label('STOP',(0,-.121,1.67),.18,(math.pi/2,0,0))
export('brake_panel')

# One full editable file with four named scenes, proper UI state, shared palette.
# Exclude the startup scene; no existing project file was ever opened.
bpy.context.window.scene=bus
for scn in list(bpy.data.scenes):
    if scn not in (bus,ufo,sewer,panel):
        for obj in list(scn.objects):bpy.data.objects.remove(obj,do_unlink=True)
        bpy.data.scenes.remove(scn)
bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'southpark_threats.blend'),compress=True)
(OUT/'asset-audit.json').write_text(json.dumps(audit,indent=2),encoding='utf-8')
print('THREAT_ASSETS_OK',json.dumps(audit))
