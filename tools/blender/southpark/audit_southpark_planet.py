"""Auditoria no destructiva del planeta aprobado. No guarda ni exporta el blend.

Ejecutar desde disco en Blender para disponer de __file__. Escribe un unico JSON
de diagnostico y restaura la raiz despues de probar sus transformaciones.
"""
import bpy
import bmesh
import numpy as np
import math
import json
import hashlib
from pathlib import Path
from collections import defaultdict, Counter
from mathutils import Vector, Matrix

REPO=Path(__file__).resolve().parents[3]
OUTPUT=REPO/'docs/references/southpark/southpark_technical_review.json'
scene=bpy.data.scenes['SouthPark_Planet']
assert bpy.context.scene==scene,'Activar primero la escena SouthPark_Planet'
root=bpy.data.objects['SouthPark_Planet_ROOT']
assert Path(bpy.data.filepath).resolve()==(REPO/'assets/models/planets/southpark/southpark_planet.blend').resolve()
bpy.context.view_layer.update()
planet=[root]+list(root.children_recursive)
meshes=[o for o in planet if o.type=='MESH']
unique={o.data for o in meshes}

def plain(v):
    if hasattr(v,'to_dict'):return {k:plain(x) for k,x in v.to_dict().items()}
    if hasattr(v,'to_list'):return [plain(x) for x in v.to_list()]
    if isinstance(v,(list,tuple)):return [plain(x) for x in v]
    if isinstance(v,(str,int,float,bool,type(None))):return v
    return repr(v)

def geometry_hash(me):
    """Incluye indices, orden, UV, color, suavizado, aristas y materiales."""
    h=hashlib.sha256()
    def add(v):h.update(repr(v).encode('utf-8'))
    for v in me.vertices:add(tuple(v.co))
    for e in me.edges:add((tuple(e.vertices),e.use_seam,e.use_edge_sharp))
    for p in me.polygons:add((tuple(p.vertices),p.material_index,p.use_smooth))
    for a in me.attributes:
        add((a.name,a.domain,a.data_type))
        if a.name=='position':continue
        for d in a.data:
            for prop in ('value','vector','color','uv'):
                if hasattr(d,prop):
                    v=getattr(d,prop);add(tuple(v) if hasattr(v,'__len__') else v)
    add(tuple(m.name if m else None for m in me.materials))
    add(plain(dict(me.items())))
    return h.hexdigest()

def material_record(mat):
    nodes=[];links=[]
    if mat.use_nodes:
        for n in mat.node_tree.nodes:
            inputs={}
            for s in n.inputs:
                if hasattr(s,'default_value'):
                    v=s.default_value
                    inputs[s.identifier]=list(v) if hasattr(v,'__len__') else v
            nodes.append({'name':n.name,'type':n.type,'inputs':inputs})
        links=[(l.from_node.name,l.from_socket.identifier,l.to_node.name,l.to_socket.identifier)
               for l in mat.node_tree.links]
    return {'name':mat.name,'diffuse_color':list(mat.diffuse_color),
            'backface_culling':mat.use_backface_culling,'nodes':nodes,'links':links}

def snapshot():
    mesh_hashes={m:geometry_hash(m) for m in unique}
    objects={o.name:{'world':[list(row) for row in o.matrix_world],
                     'parent':o.parent.name if o.parent else None,
                     'collections':sorted(c.name for c in o.users_collection),
                     'properties':{k:plain(v) for k,v in o.items()},
                     'geometry':mesh_hashes[o.data] if o.type=='MESH' else None}
             for o in planet}
    mats=sorted({m for o in meshes for m in o.data.materials if m},key=lambda m:m.name)
    return {'objects':objects,'materials':[material_record(m) for m in mats]}

before=snapshot()
before_hash=hashlib.sha256(json.dumps(before,sort_keys=True).encode()).hexdigest()
transforms={o.name:o.matrix_world.copy() for o in planet}
original_world=root.matrix_world.copy()
original_basis=root.matrix_basis.copy()
original_lrs=(root.location.copy(),root.rotation_euler.copy(),root.scale.copy(),root.rotation_quaternion.copy())
tests=[]
try:
    for name,delta in [('translation',Matrix.Translation((4.25,-2.5,3.75))),
                       ('rotation',Matrix.Rotation(.63,4,'Z') @ Matrix.Rotation(-.24,4,'X')),
                       ('uniform_scale',Matrix.Diagonal((1.23,1.23,1.23,1.)))]:
        root.matrix_world=delta @ original_world
        bpy.context.view_layer.update()
        errors={o.name:max(abs(o.matrix_world[i][j]-(delta @ transforms[o.name])[i][j])
                          for i in range(4) for j in range(4)) for o in planet}
        tests.append({'test':name,'max_error':max(errors.values()),
                      'failed':[n for n,e in errors.items() if e>2e-5]})
finally:
    root.matrix_basis=original_basis
    root.location,root.rotation_euler,root.scale,root.rotation_quaternion=original_lrs
    bpy.context.view_layer.update()
after=snapshot()
assert before==after,'La prueba temporal no restauro el estado exacto'
assert not any(t['failed'] for t in tests),tests

mesh_records=[];hash_groups=defaultdict(list);material_slots=0
duplicate_world_groups=defaultdict(list)
for me in sorted(unique,key=lambda m:m.name):
    me.calc_loop_triangles()
    objects=[o for o in meshes if o.data==me]
    digest=geometry_hash(me)
    hash_groups[digest].append(me.name)
    bm=bmesh.new();bm.from_mesh(me)
    boundary=sum(e.is_boundary for e in bm.edges)
    nonmanifold=sum(not e.is_manifold for e in bm.edges)
    noncontiguous=sum(e.is_manifold and not e.is_contiguous for e in bm.edges)
    closed=bool(bm.edges) and all(e.is_manifold for e in bm.edges)
    signed_volume=bm.calc_volume(signed=True) if closed else None
    bm.free()
    facekeys=Counter((tuple(sorted(tuple(me.vertices[i].co) for i in p.vertices)),p.material_index)
                     for p in me.polygons)
    used={i for p in me.polygons for i in p.vertices}
    signed_inner=0
    # Estimacion conservadora: todos los vertices del triangulo bajo la superficie analitica.
    obj=objects[0]
    for tri in me.loop_triangles:
        buried=True
        for vi in tri.vertices:
            p=obj.matrix_world @ me.vertices[vi].co;r=p.length
            if r<1e-9:continue
            lat=math.asin(max(-1,min(1,p.z/r)));lon=math.atan2(p.x,-p.y)
            radius=22+.10*math.sin(3*lon)*math.cos(lat)**2+.08*math.sin(lat*3)*math.cos(lon*4)
            if r>=radius-.025:buried=False;break
        signed_inner+=buried
    mesh_records.append({'mesh':me.name,'objects':[o.name for o in objects],
                         'vertices':len(me.vertices),'triangles':len(me.loop_triangles),
                         'materials':len(me.materials),'uv_layers':[u.name for u in me.uv_layers],
                         'shape_keys':bool(me.shape_keys),'custom_normals':me.has_custom_normals,
                         'boundary_edges':boundary,'nonmanifold_edges':nonmanifold,
                         'inconsistent_winding_edges':noncontiguous,
                         'closed_mesh':closed,'signed_volume_if_closed':signed_volume,
                         'zero_area_faces':sum(p.area<1e-10 for p in me.polygons),
                         'loose_vertices':len(me.vertices)-len(used),
                         'coincident_face_candidates':sum(n-1 for n in facekeys.values() if n>1),
                         'buried_triangle_estimate_first_instance':signed_inner,
                         'digest':digest})
    for o in objects:
        duplicate_world_groups[(digest,tuple(tuple(row) for row in o.matrix_world))].append(o.name)
        material_slots+=len({p.material_index for p in me.polygons})

mats=sorted({m for o in meshes for m in o.data.materials if m},key=lambda m:m.name)
material_records=[material_record(m) for m in mats]
mat_groups=defaultdict(list)
for rec in material_records:
    key={k:v for k,v in rec.items() if k!='name'}
    mat_groups[json.dumps(key,sort_keys=True)].append(rec['name'])
assign_errors=[]
for obj in meshes:
    for p in obj.data.polygons:
        if p.material_index>=len(obj.material_slots) or obj.material_slots[p.material_index].material is None:
            assign_errors.append((obj.name,p.index,p.material_index))

road_normals=[]
for obj in bpy.data.collections['Roads'].objects:
    if obj.type!='MESH':continue
    normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
    outward=inward=zero=0
    for p in obj.data.polygons:
        center=obj.matrix_world @ p.center
        d=(normal_matrix @ p.normal).normalized().dot(center.normalized())
        if p.area<1e-10:zero+=1
        elif d<-.2:inward+=1
        elif d>.2:outward+=1
    road_normals.append({'object':obj.name,'outward_faces':outward,'inward_faces':inward,'zero_area_faces':zero})

sphere=bpy.data.objects['Planet_Surface_Snow']
points=np.array([tuple(sphere.matrix_world @ v.co) for v in sphere.data.vertices])
# Ajuste por minimos cuadrados: centro de la esfera base, sin tomar edificios/montanas.
fit=np.linalg.lstsq(np.column_stack((2*points,np.ones(len(points)))),np.sum(points*points,axis=1),rcond=None)[0][:3]
centroid=points.mean(axis=0)
origin=np.array(tuple(root.matrix_world.translation))
deps=[]
for kind,items in [('images',bpy.data.images),('fonts',bpy.data.fonts),('libraries',bpy.data.libraries),('texts',bpy.data.texts)]:
    for item in items:
        path=getattr(item,'filepath','')
        if not path:continue
        packed=bool(getattr(item,'packed_file',None))
        deps.append({'type':kind,'name':item.name,'path':path,'relative':path.startswith('//'),
                     'packed':packed,'exists':Path(bpy.path.abspath(path)).exists(),
                     'needed_to_display_model':kind in {'images','fonts','libraries'}})

presentation=set(bpy.data.collections['Presentation'].all_objects)
unparented=[o.name for o in scene.objects if o.type in {'MESH','FONT','CURVE','SURFACE','META','VOLUME'}
            and o not in planet and o not in presentation]
report={
    'schema':1,'blender_version':bpy.app.version_string,'file':'assets/models/planets/southpark/southpark_planet.blend',
    'blend_sha256':hashlib.sha256(Path(bpy.data.filepath).read_bytes()).hexdigest(),
    'visual_snapshot_sha256':before_hash,'snapshot_unchanged_after_tests':True,
    'counts':{'planet_objects':len(planet),'planet_mesh_objects':len(meshes),
              'active_scene_objects':len(scene.objects),'all_file_objects':len(bpy.data.objects),
              'vertices_instanced':sum(len(o.data.vertices) for o in meshes),
              'triangles_instanced':sum(len(o.data.loop_triangles) for o in meshes),
              'unique_meshes':len(unique),'vertices_unique':sum(len(m.vertices) for m in unique),
              'triangles_unique':sum(len(m.loop_triangles) for m in unique),
              'materials':len(mats),'shared_meshes':sum(len(m['objects'])>1 for m in mesh_records),
              'objects_using_shared_meshes':sum(len(m['objects']) for m in mesh_records if len(m['objects'])>1),
              'object_material_batches_upper_estimate':material_slots},
    'collections':[{'name':c.name,'direct_objects':len(c.objects),'children':[x.name for x in c.children]}
                   for c in bpy.data.collections if c in scene.collection.children_recursive],
    'root':{'world':[list(row) for row in root.matrix_world],
            'sphere_fitted_center':fit.tolist(),'sphere_vertex_centroid':centroid.tolist(),
            'origin_distance_from_fit':float(np.linalg.norm(origin-fit)),
            'sphere_bounds_center':((points.min(axis=0)+points.max(axis=0))*.5).tolist(),
            'unparented_visual_objects':unparented,'transform_tests':tests},
    'mesh_records':mesh_records,
    'identical_mesh_data_groups':[names for names in hash_groups.values() if len(names)>1],
    'coincident_identical_objects':[names for names in duplicate_world_groups.values() if len(names)>1],
    'identical_material_candidates':[names for names in mat_groups.values() if len(names)>1],
    'material_assign_errors':assign_errors,'materials':material_records,'dependencies':deps,
    'road_normal_diagnostics':road_normals,
    'modifiers':[{'object':o.name,'type':m.type,'name':m.name,'viewport':m.show_viewport,'render':m.show_render}
                 for o in planet for m in o.modifiers],
    'constraints':[{'object':o.name,'type':c.type} for o in planet for c in o.constraints],
    'curves_or_text':[o.name for o in planet if o.type in {'FONT','CURVE','SURFACE'}],
    'sign':[{'object':o.name,'type':o.type,'parent':o.parent.name if o.parent else None}
            for o in bpy.data.collections['SouthPark_Sign'].all_objects],
    'nonunit_scales':[{'object':o.name,'scale':list(o.scale),'negative_determinant':o.matrix_world.determinant()<0}
                      for o in planet if any(abs(x-1)>1e-6 for x in o.scale)],
    'animation':[o.name for o in planet if o.animation_data],
    'unused_data_candidates':{'meshes':[m.name for m in bpy.data.meshes if m.users==0],
                              'materials':[m.name for m in bpy.data.materials if m.users==0]},
    'other_scenes':[{'scene':s.name,'objects':[o.name for o in s.objects]} for s in bpy.data.scenes if s!=scene],
    'hidden_objects':[o.name for o in planet if o.hide_render or o.hide_viewport],
    'uv_mesh_count':sum(bool(m.uv_layers) for m in unique),
    'negative_closed_meshes':[m['mesh'] for m in mesh_records
                             if m['signed_volume_if_closed'] is not None and m['signed_volume_if_closed']<-.00001],
    'optimizations_applied_to_model':[],
    'render_path':scene.render.filepath,
}
assert not unparented,unparented
assert not assign_errors,assign_errors
OUTPUT.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({k:report[k] for k in ['counts','root','identical_mesh_data_groups','coincident_identical_objects',
                                     'identical_material_candidates','modifiers','curves_or_text','unused_data_candidates']},ensure_ascii=False))
print('Road normals:',json.dumps(road_normals))
print('Topology summary:',json.dumps({k:sum(m[k] for m in mesh_records) for k in
      ['zero_area_faces','loose_vertices','inconsistent_winding_edges','coincident_face_candidates']}))
