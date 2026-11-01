"""Prepara la Ciudadela aprobada: fuente portable, GLB near/far, OBJ y auditoria.

blender -b --python tools/blender/spaceStation/prepare_citadel_asset.py
Opera exclusivamente sobre el paquete del repositorio. No usa ProyectoFinal.
"""
from pathlib import Path
import bpy
import bmesh
import hashlib
import json
import re
import numpy as np
from mathutils import Matrix, Vector

REPO=Path(__file__).resolve().parents[3]
MANIFEST=REPO/'assets/models/environment/spaceStation/asset.json'
asset=json.loads(MANIFEST.read_text(encoding='utf-8'))
SOURCE=REPO/asset['source']
report={'asset_id':asset['id'],'role':asset['role'],
        'approved_source_sha256':asset['approved_source_sha256'],
        'exports':{},'scale_factor':1.0,'runtime_placement':'pending'}


def signature(objects):
    h=hashlib.sha256()
    for ob in sorted(objects,key=lambda o:o.name):
        h.update(ob.name.encode('utf-8'))
        h.update(str([tuple(row) for row in ob.matrix_world]).encode())
        coords=np.empty(len(ob.data.vertices)*3,dtype=np.float32)
        ob.data.vertices.foreach_get('co',coords);h.update(coords.tobytes())
        indices=np.empty(len(ob.data.loops),dtype=np.int32)
        ob.data.loops.foreach_get('vertex_index',indices);h.update(indices.tobytes())
        h.update(str([len(p.vertices) for p in ob.data.polygons]).encode())
        h.update(str([m.name if m else None for m in ob.data.materials]).encode())
    return h.hexdigest()


def clean(mesh):
    bm=bmesh.new();bm.from_mesh(mesh)
    bmesh.ops.triangulate(bm,faces=list(bm.faces))
    bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=1e-6)
    for face in list(bm.faces):
        if face.calc_area()<1e-12:bm.faces.remove(face)
    for edge in list(bm.edges):
        if not edge.link_faces:bm.edges.remove(edge)
    for vertex in list(bm.verts):
        if not vertex.link_faces:bm.verts.remove(vertex)
    bm.to_mesh(mesh);bm.free()
    mesh.validate(verbose=False,clean_customdata=True);mesh.update()


def stats(meshes):
    points=[];triangles=0;zero=0
    for ob in meshes:
        me=ob.data;me.calc_loop_triangles();triangles+=len(me.loop_triangles)
        xyz=np.empty(len(me.vertices)*3,dtype=np.float64)
        me.vertices.foreach_get('co',xyz);xyz=xyz.reshape(-1,3)
        ids=np.empty(len(me.loop_triangles)*3,dtype=np.int32)
        me.loop_triangles.foreach_get('vertices',ids);ids=ids.reshape(-1,3)
        cross=np.cross(xyz[ids[:,1]]-xyz[ids[:,0]],xyz[ids[:,2]]-xyz[ids[:,0]])
        zero+=int(np.count_nonzero(np.einsum('ij,ij->i',cross,cross)<4e-24))
        points.extend(ob.matrix_world@Vector(p) for p in ob.bound_box)
    return {'meshes':len(meshes),'triangles':triangles,'degenerate_triangles':zero,
            'bounds':[[min(p[i] for p in points),max(p[i] for p in points)] for i in range(3)]}


def export_glb(path):
    path.parent.mkdir(parents=True,exist_ok=True)
    props=bpy.ops.export_scene.gltf.get_rna_type().properties
    formats=[i.identifier for i in props['export_format'].enum_items]
    if not formats:
        # Algunas propiedades dinamicas no exponen enum_items en segundo plano.
        try:bpy.ops.export_scene.gltf(export_format='__INSPECT_ENUM__')
        except TypeError as exc:
            formats=re.findall(r"'([^']+)'",str(exc).split('not found in')[-1])
    fmt=next(value for value in formats if value=='GLB')
    bpy.ops.export_scene.gltf(filepath=str(path),export_format=fmt,use_selection=True,
        export_cameras=False,export_lights=False,export_animations=False,
        export_extras=True,export_apply=True)
    return {'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}


bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
scene=bpy.data.scenes[asset['scene']];bpy.context.window.scene=scene
source_collection=bpy.data.collections[asset['collection']]
source_meshes=[o for o in source_collection.all_objects if o.type=='MESH']
bpy.context.view_layer.update()
before=signature(source_meshes)
report['source_raw']=stats(source_meshes)
root=bpy.data.objects[asset['root']]
root['approval_status']='Aprobado por el usuario; recurso integrado en Planet-Hoppers'
root['asset_id']=asset['id'];root['asset_role']=asset['role']
scene.render.filepath='//../../../../docs/references/spaceStation/Ciudadela_Principal.png'
text=bpy.data.texts.get('LEEME | Ciudadela de Mass Effect')
if text:
    text.clear();text.write((SOURCE.parent/'README.md').read_text(encoding='utf-8'))
after=signature(source_meshes)
report['source_geometry_signature']=after
report['source_geometry_preserved']=before==after
assert report['source_geometry_preserved']
# El archivo editable conserva geometria, materiales, presentacion y coordenadas.
bpy.context.preferences.filepaths.save_version=0
bpy.ops.wm.save_as_mainfile(filepath=str(SOURCE))
source_hash=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
report['integrated_source_sha256']=source_hash

# Copias evaluadas, centradas y trianguladas para exportar.
points=[o.matrix_world@Vector(p) for o in source_meshes for p in o.bound_box]
lo=Vector([min(p[i] for p in points) for i in range(3)])
hi=Vector([max(p[i] for p in points) for i in range(3)])
origin=(lo+hi)/2;T=Matrix.Translation(-origin)
report['origin_shift_blender']=list(origin)
old_objects=list(bpy.data.objects)
col=bpy.data.collections.new('CIUDADELA | Exportacion')
scene.collection.children.link(col)
empties=[o for o in source_collection.all_objects if o.type=='EMPTY']
mapped={};names={}
for old in empties:
    new=bpy.data.objects.new('__COPY_'+old.name,None);col.objects.link(new)
    mapped[old]=new;names[new]=old.name
    for key in old.keys():new[key]=old[key]
new_root=mapped[root];new_root.matrix_world=Matrix.Identity(4)
pending=[o for o in empties if o!=root]
while pending:
    progressed=False
    for old in list(pending):
        if old.parent not in pending:
            new=mapped[old];new.parent=mapped.get(old.parent,new_root)
            new.matrix_world=T@old.matrix_world;pending.remove(old);progressed=True
    assert progressed,'Jerarquia ciclica'
deps=bpy.context.evaluated_depsgraph_get()
meshes=[]
for old in source_meshes:
    data=bpy.data.meshes.new_from_object(old.evaluated_get(deps))
    clean(data)
    new=bpy.data.objects.new('__COPY_'+old.name,data);col.objects.link(new)
    new.parent=mapped.get(old.parent,new_root);new.matrix_world=T@old.matrix_world
    names[new]=old.name;meshes.append(new)
for old in old_objects:bpy.data.objects.remove(old,do_unlink=True)
for ob,name in names.items():ob.name=name
bpy.context.view_layer.update()
for ob in bpy.context.selected_objects:ob.select_set(False)
for ob in list(mapped.values())+meshes:ob.select_set(True)
bpy.context.view_layer.objects.active=new_root
report['exports']['near']=stats(meshes)
assert report['exports']['near']['degenerate_triangles']==0
report['exports']['near'].update(export_glb(REPO/asset['near']))

obj=REPO/asset['obj'];obj.parent.mkdir(parents=True,exist_ok=True)
props=bpy.ops.wm.obj_export.get_rna_type().properties
def option(key,value):return next(i.identifier for i in props[key].enum_items if i.identifier==value)
bpy.ops.wm.obj_export(filepath=str(obj),export_selected_objects=True,
    export_materials=True,export_normals=True,export_uv=True,
    export_triangulated_mesh=True,export_object_groups=True,
    forward_axis=option('forward_axis','NEGATIVE_Z'),up_axis=option('up_axis','Y'),
    path_mode=option('path_mode','RELATIVE'))

for ob in meshes:
    if len(ob.data.polygons)<100:continue
    mod=ob.modifiers.new('LOD lejano','DECIMATE');mod.ratio=.24
    bpy.context.view_layer.objects.active=ob
    bpy.ops.object.modifier_apply(modifier=mod.name);clean(ob.data)
report['exports']['far']=stats(meshes)
assert report['exports']['far']['degenerate_triangles']==0
assert report['exports']['far']['triangles']<report['exports']['near']['triangles']
report['exports']['far'].update(export_glb(REPO/asset['far']))

for level in ('near','far'):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(REPO/asset[level]))
    observed=stats([o for o in bpy.context.scene.objects if o.type=='MESH'])
    expected=report['exports'][level]
    assert observed['triangles']==expected['triangles'],(level,'triangulos')
    assert observed['degenerate_triangles']==0
    assert asset['root'] in bpy.data.objects
    checkroot=bpy.data.objects[asset['root']]
    assert checkroot.get('asset_role')=='space_station'
    assert sum(o.name.startswith('CI | BRAZO ') for o in bpy.data.objects)==5
    assert not any(o.type in ('LIGHT','CAMERA') for o in bpy.context.scene.objects)
    assert not any(i.source=='FILE' and not i.packed_file for i in bpy.data.images)
    for a,b in zip(observed['bounds'],expected['bounds']):
        assert max(abs(a[i]-b[i]) for i in range(2))<.001,(level,a,b)
    expected['reimport']='OK'
report['source_unchanged_during_export']=hashlib.sha256(SOURCE.read_bytes()).hexdigest()==source_hash
assert report['source_unchanged_during_export']
(REPO/asset['audit']).write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding='utf-8')
print('CITADEL_AUDIT '+json.dumps(report,ensure_ascii=False),flush=True)
