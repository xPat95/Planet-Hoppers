"""Blender: genera copias near/far y personajes a partir de asset.json.
Uso: blender -b --python prepare_ship_asset.py -- --repo RUTA --asset RUTA_RELATIVA
No guarda ni modifica la fuente editable.
"""
from pathlib import Path
import bpy,bmesh,json,sys,argparse,hashlib
from mathutils import Vector,Matrix

cli=argparse.ArgumentParser();cli.add_argument('--repo',type=Path,default=Path(__file__).resolve().parents[3]);cli.add_argument('--asset',required=True)
args=cli.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])
REPO=args.repo.resolve();manifest=json.loads((REPO/args.asset).read_text(encoding='utf-8'))
SOURCE=REPO/manifest['source'];sha=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
report={'asset_id':manifest['id'],'role':'ship','franchise':manifest['franchise'],'source_sha256':sha,'exports':{},'characters':{},'scale_factor':1.0,'source_unchanged':False}

def clean(mesh):
    bm=bmesh.new();bm.from_mesh(mesh);bmesh.ops.triangulate(bm,faces=list(bm.faces));bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=1e-6)
    for face in list(bm.faces):
        if face.calc_area()<1e-12:bm.faces.remove(face)
    for edge in list(bm.edges):
        if not edge.link_faces:bm.edges.remove(edge)
    for vertex in list(bm.verts):
        if not vertex.link_faces:bm.verts.remove(vertex)
    bm.to_mesh(mesh);bm.free();mesh.validate(verbose=False,clean_customdata=True);mesh.update()

def build_copy(character=None):
    bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
    main=bpy.data.objects[character['root'] if character else manifest['root']]
    allowed=set(bpy.data.collections[manifest['collection']].all_objects)
    objects=[o for o in allowed if o.type=='MESH' and not o.get('render_only')]
    def under(ob,parent):
        while ob:
            if ob==parent:return True
            ob=ob.parent
        return False
    if character:objects=[o for o in objects if under(o,main)]
    points=[o.matrix_world@Vector(p) for o in objects for p in o.bound_box]
    lower=Vector(tuple(min(p[i] for p in points) for i in range(3)));upper=Vector(tuple(max(p[i] for p in points) for i in range(3)))
    origin=Vector((main.matrix_world.translation.x,main.matrix_world.translation.y,lower.z)) if character else (lower+upper)/2
    T=Matrix.Translation(-origin);old_objects=list(bpy.data.objects);deps=bpy.context.evaluated_depsgraph_get()
    empties=[main]+[o for o in allowed if o.type=='EMPTY' and o!=main and under(o,main)]
    mapped={};names={};col=bpy.data.collections.new('__EXPORT');bpy.context.scene.collection.children.link(col)
    for old in empties:
        new=bpy.data.objects.new('__COPY_'+old.name,None);col.objects.link(new);mapped[old]=new;names[new]=old.name
        for key in old.keys():new[key]=old[key]
    mapped[main].matrix_world=Matrix.Identity(4);mapped[main]['asset_role']='character' if character else 'ship'
    pending=[o for o in empties if o!=main]
    while pending:
        for old in list(pending):
            if old.parent==main or old.parent not in pending:
                new=mapped[old];new.parent=mapped.get(old.parent,mapped[main]);new.matrix_world=T@old.matrix_world;pending.remove(old)
    meshes=[]
    for old in objects:
        data=bpy.data.meshes.new_from_object(old.evaluated_get(deps));clean(data)
        new=bpy.data.objects.new('__COPY_'+old.name,data);col.objects.link(new);new.parent=mapped.get(old.parent,mapped[main]);new.matrix_world=T@old.matrix_world;names[new]=old.name;meshes.append(new)
    export_root=mapped[main]
    for old in old_objects:bpy.data.objects.remove(old,do_unlink=True)
    for ob,name in names.items():ob.name=name
    bpy.context.view_layer.update()
    for ob in list(bpy.context.selected_objects):ob.select_set(False)
    for ob in list(mapped.values())+meshes:ob.select_set(True)
    bpy.context.view_layer.objects.active=export_root
    return meshes,origin

def stats(meshes):
    points=[];triangles=0;zero=0
    for ob in meshes:
        ob.data.calc_loop_triangles();triangles+=len(ob.data.loop_triangles)
        zero+=sum(t.area<1e-12 for t in ob.data.loop_triangles);points.extend(ob.matrix_world@Vector(p) for p in ob.bound_box)
    return {'meshes':len(meshes),'triangles':triangles,'degenerate_triangles':zero,'bounds':[[min(p[i] for p in points),max(p[i] for p in points)] for i in range(3)]}

def glb(path):
    path.parent.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(path),export_format='GLB',use_selection=True,export_cameras=False,export_lights=False,export_animations=False,export_extras=True,export_apply=True)

meshes,origin=build_copy();report['origin_shift']=list(origin);report['exports']['near']=stats(meshes)
assert report['exports']['near']['degenerate_triangles']==0
glb(REPO/manifest['near'])
objpath=REPO/manifest['obj'];objpath.parent.mkdir(parents=True,exist_ok=True)
props=bpy.ops.wm.obj_export.get_rna_type().properties
def option(name,value):return next(i.identifier for i in props[name].enum_items if i.identifier==value)
bpy.ops.wm.obj_export(filepath=str(objpath),export_selected_objects=True,export_materials=True,export_normals=True,export_uv=True,export_triangulated_mesh=True,export_object_groups=True,forward_axis=option('forward_axis','NEGATIVE_Z'),up_axis=option('up_axis','Y'),path_mode=option('path_mode','RELATIVE'))
for ob in meshes:
    if len(ob.data.polygons)<100 or any(word.casefold() in ob.name.casefold() for word in ('Cristal','Ojos','Visor','Pupilas')):continue
    mod=ob.modifiers.new('LOD lejano','DECIMATE');mod.ratio=.28
    bpy.context.view_layer.objects.active=ob;bpy.ops.object.modifier_apply(modifier=mod.name);clean(ob.data)
report['exports']['far']=stats(meshes);assert report['exports']['far']['degenerate_triangles']==0
assert report['exports']['far']['triangles']<report['exports']['near']['triangles']
glb(REPO/manifest['far'])
for character in manifest.get('characters',[]):
    meshes,origin=build_copy(character);character_report=stats(meshes);character_report['origin_shift']=list(origin);assert character_report['degenerate_triangles']==0
    glb(REPO/character['glb']);report['characters'][character['id']]=character_report

# Reimportación independiente del estado de exportación, con comprobación de raíces.
checks=[('near',manifest['near'],manifest['root']),('far',manifest['far'],manifest['root'])]
checks.extend((c['id'],c['glb'],c['root']) for c in manifest.get('characters',[]))
for key,path,rootname in checks:
    bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(REPO/path))
    meshes=[o for o in bpy.context.scene.objects if o.type=='MESH'];observed=stats(meshes)
    expected=report['exports'].get(key) or report['characters'][key]
    assert observed['triangles']==expected['triangles'],(key,observed['triangles'],expected['triangles'])
    assert observed['degenerate_triangles']==0
    assert rootname in bpy.data.objects,(key,rootname)
    assert not any(o.type in {'LIGHT','CAMERA'} for o in bpy.context.scene.objects)
    assert not any(o.name.startswith(('INK |','N1 STAGE |','RS | Estrellas')) for o in meshes)
    assert not any(i.source=='FILE' and not i.packed_file for i in bpy.data.images)
    for a,b in zip(observed['bounds'],expected['bounds']):assert max(abs(a[i]-b[i]) for i in range(2))<.001,(key,a,b)
    expected['reimport']='OK'
report['source_unchanged']=hashlib.sha256(SOURCE.read_bytes()).hexdigest()==sha;assert report['source_unchanged']
output=REPO/manifest['audit'];output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('SHIP_AUDIT',json.dumps(report,ensure_ascii=False))
