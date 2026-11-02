"""Bake approved Cycles lighting and world; never save or remodel the source blend.

UVs and embedded radiance textures are an export correction. Positions, faces,
materials' original PBR factors and all 241 architecture object names are retained.
"""
from pathlib import Path
import bpy, bmesh, json, hashlib, math, sys, types, time, struct
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'assets/models/environment/spaceStation'
OUT=ROOT/'docs/references/hangar/presentation'
TEX=ROOT/'assets/textures/environment/hangar'
OUT.mkdir(parents=True,exist_ok=True);TEX.mkdir(parents=True,exist_ok=True)
SOURCE=BASE/'citadel_hangar_reconstructed.blend'
start=time.time();source_hash=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
scene=bpy.context.scene
scene.render.engine='CYCLES';scene.cycles.samples=64;scene.cycles.use_denoising=True
try:
    prefs=bpy.context.preferences.addons['cycles'].preferences
    prefs.compute_device_type='OPTIX';prefs.get_devices()
    for d in prefs.devices:d.use=d.type=='OPTIX'
    scene.cycles.device='GPU'
except Exception:
    scene.cycles.device='CPU'
architecture=sorted([o for o in scene.objects if o.type=='MESH' and o.get('exportable')==True],key=lambda o:o.name)
names=[o.name for o in architecture]
assert len(architecture)==241
source_materials={m.name:m for o in architecture for m in o.data.materials if m}
props={name:{'Base Color':list(next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED').inputs['Base Color'].default_value)} for name,m in source_materials.items()}
# Hide ships and diagnostic envelopes; retain the static scale-platform as an occluder.
for o in scene.objects:
    if o.type=='MESH' and o.get('exportable')!=True:
        o.hide_render='Plataforma' not in o.name
bpy.ops.object.select_all(action='DESELECT')
for o in architecture:o.hide_set(False);o.select_set(True)
bpy.context.view_layer.objects.active=architecture[0]
bpy.ops.object.convert(target='MESH')
architecture=sorted([o for o in scene.objects if o.type=='MESH' and o.get('exportable')==True],key=lambda o:o.name)
# Orient evaluated triangles only AFTER original modifiers have been applied.
# Thus normal corrections cannot change a Solidify displacement or silhouette.
for o in architecture:
    # Shield solids already have correct normals. Keep their original non-planar
    # polygon tessellation so the glTF matches the approved export exactly.
    if o.name.startswith('HOMBRO |'):
        continue
    original=o.data;original.calc_loop_triangles()
    tri=bpy.data.meshes.new(o.name+' | corrected export normals')
    tri.from_pydata([v.co for v in original.vertices],[],[tuple(t.vertices) for t in original.loop_triangles])
    for m in original.materials:tri.materials.append(m)
    for p,t in zip(tri.polygons,original.loop_triangles):
        p.material_index=original.polygons[t.polygon_index].material_index
        p.use_smooth=original.polygons[t.polygon_index].use_smooth
    bm=bmesh.new();bm.from_mesh(tri)
    bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(tri);bm.free()
    o.data=tri
# An integer face attribute lets us join for one bake and restore exact objects afterwards.
for i,o in enumerate(architecture):
    attr=o.data.attributes.new(name='ph_source_object',type='INT',domain='FACE')
    for d in attr.data:d.value=i
bpy.ops.object.join();joined=bpy.context.object
joined.name='TEMP | approved architecture lighting bake'
bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.uv.smart_project(angle_limit=math.radians(66),island_margin=.001,area_weight=1.0,scale_to_bounds=True)
bpy.ops.uv.pack_islands(rotate=True,margin=.002)
bpy.ops.object.mode_set(mode='OBJECT')
atlas=bpy.data.images.new('Hangar | Cycles radiance',width=4096,height=4096,alpha=False,float_buffer=True)
atlas.colorspace_settings.name='Linear Rec.709'
for m in joined.data.materials:
    if not m:continue
    node=m.node_tree.nodes.new('ShaderNodeTexImage');node.image=atlas
    m.node_tree.nodes.active=node
scene.render.bake.use_clear=False;scene.render.bake.margin=3
scene.render.bake.use_pass_direct=True;scene.render.bake.use_pass_indirect=True;scene.render.bake.use_pass_color=True
print('BAKE_BEGIN',len(joined.data.vertices),len(joined.data.polygons),flush=True)
bpy.ops.object.bake(type='COMBINED')
scene.render.image_settings.file_format='PNG';scene.render.image_settings.color_mode='RGB'
scene.render.image_settings.color_depth='8'
# save_render applies the same AgX view transform as the approved PNG renders.
atlas_path=TEX/'approved_cycles_radiance.png'
atlas.save_render(str(atlas_path),scene=scene)
print('BAKE_SAVED',str(atlas_path),time.time()-start,flush=True)
display_atlas=bpy.data.images.load(str(atlas_path),check_existing=False)
display_atlas.pack()
# Base-color slots transport display-ready radiance; the architecture-only shader reads
# it without multiplying the original base-color factor or adding emission a second time.
for m in joined.data.materials:
    if not m:continue
    bsdf=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    color=tuple(bsdf.inputs['Base Color'].default_value)
    tex=m.node_tree.nodes.new('ShaderNodeTexImage');tex.image=display_atlas
    m.node_tree.links.new(tex.outputs['Color'],bsdf.inputs['Base Color'])
    bsdf.inputs['Base Color'].default_value=color
me=joined.data;tag=me.attributes['ph_source_object'];uv=me.uv_layers.active
groups={i:[] for i in range(len(names))}
for p in me.polygons:groups[tag.data[p.index].value].append(p)
exports=[]
for i,name in enumerate(names):
    polygons=groups[i];ids=sorted({j for p in polygons for j in p.vertices});mapping={j:k for k,j in enumerate(ids)}
    mesh=bpy.data.meshes.new(name+' | export UV')
    mesh.from_pydata([me.vertices[j].co for j in ids],[],[tuple(mapping[j] for j in p.vertices) for p in polygons])
    for m in me.materials:mesh.materials.append(m)
    layer=mesh.uv_layers.new(name='Cycles radiance UV')
    for new,old in zip(mesh.polygons,polygons):
        new.material_index=old.material_index;new.use_smooth=old.use_smooth
        for a,b in zip(new.loop_indices,old.loop_indices):layer.data[a].uv=uv.data[b].uv
    o=bpy.data.objects.new(name,mesh);scene.collection.objects.link(o);exports.append(o)
bpy.data.objects.remove(joined,do_unlink=True)
bpy.ops.object.select_all(action='DESELECT')
for o in exports:o.select_set(True)
bpy.context.view_layer.objects.active=exports[0]
try:
    from io_scene_gltf2.blender.exp.animation.animations import gather_animations
except ModuleNotFoundError:
    stub=types.ModuleType('io_scene_gltf2.blender.exp.animation.animations');stub.gather_animations=lambda *a,**k:[]
    sys.modules[stub.__name__]=stub
target=BASE/'exports/citadel_hangar_reconstructed.glb'
bpy.ops.export_scene.gltf(filepath=str(target),export_format='GLB',use_selection=True,export_apply=True,
    export_animations=False,export_cameras=False,export_lights=False,export_extras=False)
# glTF's linked-image export defaults the color factor to white. Retain the original
# PBR factors as metadata; the runtime radiance branch deliberately does not multiply
# them a second time because they are already included in the lighting bake.
data=target.read_bytes();json_size=struct.unpack_from('<I',data,12)[0]
document=json.loads(data[20:20+json_size]);tail=data[20+json_size:]
for material in document['materials']:
    material['pbrMetallicRoughness']['baseColorFactor']=props[material['name']]['Base Color']
encoded=json.dumps(document,separators=(',',':'),ensure_ascii=False).encode('utf-8')
encoded+=b' '*((-len(encoded))%4)
target.write_bytes(struct.pack('<4sII',b'glTF',2,20+len(encoded)+len(tail))
    +struct.pack('<I4s',len(encoded),b'JSON')+encoded+tail)
print('LIT_GLB_SAVED',flush=True)
# Render the original procedural world into an equirectangular backdrop, with no geometry.
for o in scene.objects:
    if o.type in ['MESH','FONT']:o.hide_render=True
camdata=bpy.data.cameras.new('TEMP | original world panorama');camdata.type='PANO'
if hasattr(camdata,'panorama_type'):camdata.panorama_type='EQUIRECTANGULAR'
elif hasattr(camdata.cycles,'panorama_type'):camdata.cycles.panorama_type='EQUIRECTANGULAR'
cam=bpy.data.objects.new('TEMP | original world panorama',camdata);scene.collection.objects.link(cam)
cam.location=(0,0,0);cam.rotation_euler=(math.pi/2,0,0);scene.camera=cam
scene.render.resolution_x=2048;scene.render.resolution_y=1024;scene.render.resolution_percentage=100
scene.cycles.samples=32;scene.render.filepath=str(TEX/'approved_world.png')
bpy.ops.render.render(write_still=True)
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()==source_hash
report={'source_sha256':source_hash,'source_unchanged':True,'objects':len(exports),
    'source_object_names':names,'atlas':str(atlas_path.relative_to(ROOT)),
    'backdrop':str((TEX/'approved_world.png').relative_to(ROOT)),
    'radiance_resolution':[4096,4096],'world_resolution':[2048,1024],
    'bake':'Cycles COMBINED; original lights, materials, world and AgX display transform',
    'export':str(target.relative_to(ROOT)),'export_sha256':hashlib.sha256(target.read_bytes()).hexdigest(),
    'seconds':time.time()-start,'geometry_changes':'Positions and triangles retained; corrected reversed export normals and added UVs only',
    'limitations':['Static architectural lighting; ships keep their existing real-time shading.',
        'Specular radiance bake is an approximation across viewing angles; no dynamic ship shadows baked into the room.']}
(OUT/'bake-manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('PRESENTATION_EXPORT_PASS',report['seconds'],flush=True)
