"""Prepare a compact GLB from evaluated copies, audit reimport; never save source."""
from pathlib import Path
import bpy,bmesh,json,hashlib
# Alpha 5.4 can ship without this unused animation module. Never called for static assets.
import sys, types, io_scene_gltf2
missing = 'io_scene_gltf2.blender.exp.animation.animations'
modulefile = Path(io_scene_gltf2.__file__).parent/'blender/exp/animation/sampled/shapekeys/channel_target.py'
if not modulefile.exists():
    stub=types.ModuleType(missing)
    def no_animation(*args,**kwargs):
        raise RuntimeError('Static asset only: this Blender build lacks shape-key animation export')
    stub.gather_animations=no_animation
    sys.modules[missing]=stub
from mathutils import Vector
REPO=Path(__file__).resolve().parents[5]
OUT=REPO/'assets/models/ships/southpark/butters'
REF=REPO/'docs/references/southpark/ships/butters'
SOURCE=OUT/'southpark_butters_ship.blend'
sha=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
source_scene=bpy.data.scenes['SouthPark_Butters_Ship']; bpy.context.window.scene=source_scene
asset=bpy.data.collections['NAVE SOUTH PARK | Butters']
originals=[o for o in asset.all_objects if o.type=='MESH']
panes=[o for o in originals if o.get('closed_window')]
assert len(panes)==3
assert any('Sealed cabin roof' in o.name for o in originals)
assert any('Rear sealed cabin panel' in o.name for o in originals)
assert len([o for o in originals if 'Red recycled thruster' in o.name])==2
deps=bpy.context.evaluated_depsgraph_get()
export_scene=bpy.data.scenes.new('BUTTERS_GAME_EXPORT')
newroot=bpy.data.objects.new('SouthPark_Butters_ROOT_export',None)
export_scene.collection.objects.link(newroot)
for key,value in bpy.data.objects['SouthPark_Butters_ROOT'].items(): newroot[key]=value
groups={}
for original in originals:
    me=bpy.data.meshes.new_from_object(original.evaluated_get(deps))
    bm=bmesh.new(); bm.from_mesh(me)
    bmesh.ops.triangulate(bm,faces=list(bm.faces))
    bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=1e-6)
    for face in list(bm.faces):
        if face.calc_area()<1e-12: bm.faces.remove(face)
    for edge in list(bm.edges):
        if not edge.link_faces: bm.edges.remove(edge)
    for vert in list(bm.verts):
        if not vert.link_faces: bm.verts.remove(vert)
    bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
    bm.to_mesh(me); bm.free()
    ob=bpy.data.objects.new('__GAME_'+original.name,me); export_scene.collection.objects.link(ob)
    ob.parent=newroot; ob.matrix_world=original.matrix_world.copy()
    key=me.materials[0].name
    groups.setdefault(key,[]).append(ob)
bpy.context.window.scene=export_scene
for name,objects in groups.items():
    bpy.ops.object.select_all(action='DESELECT')
    for ob in objects: ob.select_set(True)
    bpy.context.view_layer.objects.active=objects[0]
    if len(objects)>1: bpy.ops.object.join()
    ob=bpy.context.object; ob.name='Butters_'+name.split(' | ')[-1].replace(' ','_'); ob.data.name=ob.name
    bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
# Remove source scene objects in this disposable process only, freeing original root name.
for ob in list(bpy.data.objects):
    if ob.name not in export_scene.objects: bpy.data.objects.remove(ob,do_unlink=True)
newroot.name='SouthPark_Butters_ROOT'
bpy.context.view_layer.update()
def stats():
    meshes=[o for o in bpy.context.scene.objects if o.type=='MESH']
    pts=[o.matrix_world@Vector(c) for o in meshes for c in o.bound_box]
    tris=0; degenerate=0
    for ob in meshes:
        ob.data.calc_loop_triangles(); tris+=len(ob.data.loop_triangles)
        degenerate+=sum(t.area<1e-12 for t in ob.data.loop_triangles)
    return {'meshes':len(meshes),'triangles':tris,'degenerate_triangles':degenerate,
            'bounds':[[min(p[i] for p in pts),max(p[i] for p in pts)] for i in range(3)]}
expected=stats()
assert expected['degenerate_triangles']==0
bpy.ops.object.select_all(action='SELECT')
dest=OUT/'exports/southpark_butters_ship.glb'; dest.parent.mkdir(parents=True,exist_ok=True)
bpy.ops.export_scene.gltf(filepath=str(dest),export_format='GLB',use_selection=True,
    export_cameras=False,export_lights=False,export_animations=False,export_extras=True,export_apply=True,use_active_scene=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(dest))
observed=stats()
assert observed['triangles']==expected['triangles'],(observed,expected)
assert observed['degenerate_triangles']==0
assert not any(o.type in {'LIGHT','CAMERA'} for o in bpy.context.scene.objects)
assert 'SouthPark_Butters_ROOT' in bpy.data.objects
assert not any(i.source=='FILE' and not i.packed_file for i in bpy.data.images)
for a,b in zip(observed['bounds'],expected['bounds']):
    assert max(abs(a[i]-b[i]) for i in range(2))<.001,(a,b)
unchanged=hashlib.sha256(SOURCE.read_bytes()).hexdigest()==sha
assert unchanged
report={'asset_id':'southpark_butters','role':'ship','franchise':'southpark','variant':'butters',
 'source':str(SOURCE.relative_to(REPO)),'source_sha256':sha,'source_unchanged':unchanged,
 'source_meshes':len(originals),'export':str(dest.relative_to(REPO)),'export_stats':expected,
 'reimport':'OK','reimport_stats':observed,'packed_texture':True,'closed_cabin_windows':3,
 'front_blender':'-Y','up_blender':'+Z','units':'metres','root':'SouthPark_Butters_ROOT',
 'pivot':'Hull centre; glTF exporter converts axes to Y up',
 'limits':'No engine, physics, collision, animation or runtime performance verification.'}
REF.mkdir(parents=True,exist_ok=True)
(REF/'integration_audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('BUTTERS_AUDIT',json.dumps(report,ensure_ascii=False))
