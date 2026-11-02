"""Validate source, editable scene and actual GLB, without saving either .blend.
Run using Blender -b --factory-startup --python this_file.py.
"""
from pathlib import Path
import bpy, json, struct, hashlib, math
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'assets/models/environment/spaceStation'
OUT=ROOT/'docs/references/hangar/phase1'
source=BASE/'citadel_space_station.blend'
manifest=json.loads((OUT/'build-manifest.json').read_text(encoding='utf-8'))
assert hashlib.sha256(source.read_bytes()).hexdigest()==manifest['source_sha256']

# Inspect the original transforms, modifiers and node inputs in the actual file.
bpy.ops.wm.open_mainfile(filepath=str(source))
reference=bpy.data.scenes['Ciudadela_Mass_Effect']
source_report={
    'sha256':manifest['source_sha256'],
    'objects':[{'name':o.name,'type':o.type,'location':list(o.location),
        'rotation_euler':list(o.rotation_euler),'scale':list(o.scale),
        'matrix_world':[list(row) for row in o.matrix_world],
        'dimensions':list(o.dimensions),'modifiers':[m.type for m in o.modifiers],
        'polygons':len(o.data.polygons) if o.type=='MESH' else 0,
        'materials':[m.name for m in o.data.materials if m] if o.type=='MESH' else []}
        for o in reference.objects],
    'materials':{m.name:{i.name:list(i.default_value) if hasattr(i.default_value,'__len__') else i.default_value
        for i in next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED').inputs
        if i.name in ['Base Color','Metallic','Roughness','Emission Color','Emission Strength']}
        for m in bpy.data.materials if m.node_tree and any(n.type=='BSDF_PRINCIPLED' for n in m.node_tree.nodes)}
}
(OUT/'source-inspection.json').write_text(json.dumps(source_report,indent=2),encoding='utf-8')

# Reopen the independent editable file, not the scene left in generator memory.
bpy.ops.wm.open_mainfile(filepath=str(BASE/'citadel_hangar_phase1.blend'))
scene=bpy.data.scenes['PRESIDIUM | Phase 1 maintenance concourse']
architecture=bpy.data.collections['ARCHITECTURE | export only']
assert len(architecture.objects)==16
assert all(o.type=='MESH' for o in architecture.objects)
assert bpy.data.collections.get('REVIEW ONLY | platform ship lights cameras') is not None
camera=next(o for o in scene.objects if o.type=='CAMERA' and o.name=='01 | Same game camera')
assert abs(math.degrees(camera.data.angle_y)-42)<.01

path=BASE/'exports/citadel_hangar_phase1.glb'
raw=path.read_bytes();magic,version,size=struct.unpack_from('<III',raw)
assert magic==0x46546c67 and version==2 and size==len(raw)
n=struct.unpack_from('<I',raw,12)[0];doc=json.loads(raw[20:20+n])
assert len(doc['meshes'])==16
assert not any('REVIEW ONLY' in o.get('name','') for o in doc['nodes'])
assert not doc.get('animations') and not doc.get('cameras') and not doc.get('images')
assert all(not b.get('uri') for b in doc['buffers'])
max_vertices=0;triangles=0;primitives=0
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        primitives+=1
        a=doc['accessors'][primitive['attributes']['POSITION']]
        assert a['count']<65536
        assert all(math.isfinite(v) for v in a['min']+a['max'])
        max_vertices=max(max_vertices,a['count'])
        triangles+=doc['accessors'][primitive['indices']]['count']//3
for mat in doc['materials']:
    if mat['name'] in source_report['materials']:
        original=source_report['materials'][mat['name']]
        pbr=mat['pbrMetallicRoughness']
        assert abs(pbr.get('metallicFactor',1)-original['Metallic'])<1e-5
        assert abs(pbr.get('roughnessFactor',1)-original['Roughness'])<1e-5

# Reimport and inspect the geometry which raylib will actually receive.
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(path))
meshes=[o for o in bpy.context.scene.objects if o.type=='MESH']
assert len(meshes)==16
degenerate=sum(1 for o in meshes for face in o.data.polygons if face.area<1e-12)
assert degenerate==0, f'Degenerate exported faces: {degenerate}'
assert all(face.normal.length>.99 for o in meshes for face in o.data.polygons)
points=[o.matrix_world@v.co for o in meshes for v in o.data.vertices]
assert all(math.isfinite(c) for v in points for c in v)
# Blender imported GLB is Z-up again. Keep the platform and whole ship clearance free.
interference=[v for v in points if v.x*v.x+v.y*v.y<4.75**2 and -.02<v.z<7.3]
assert not interference, f'Architecture interferes with frozen platform/ships: {len(interference)} vertices'
aperture=[v for v in points if abs(v.x)<9 and 1<v.z<8 and 27.2<v.y<30.9]
assert not aperture, f'Geometry fills the usable launch mouth: {len(aperture)} vertices'
bounds=[[min(v[i] for v in points),max(v[i] for v in points)] for i in range(3)]
assert bounds[0][0]<-18 and bounds[0][1]>18
assert 16<bounds[2][1]<17
report={'status':'PASS','original_source_unchanged':True,'source_sha256':manifest['source_sha256'],
    'editable_reopened':True,'glb_reimported':True,'architecture_meshes':len(meshes),
    'materials':len(doc['materials']),'primitives':primitives,'triangles':triangles,
    'max_vertices_per_primitive':max_vertices,'glb_bytes':len(raw),'bounds_blender':bounds,
    'degenerate_faces':degenerate,'unit_face_normals':True,
    'platform_and_ship_clearance':True,'aperture_open':True,
    'export_excludes':['platform','ships','camera','lights','animations'],
    'original_material_parameters_preserved':True,'game_camera_fov_y':42,
    'external_asset_dependencies':0}
(OUT/'validation.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print('PHASE1_VALIDATION_PASS '+json.dumps(report),flush=True)
