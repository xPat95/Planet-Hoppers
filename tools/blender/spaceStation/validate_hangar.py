"""Validate the actual exported interior, with Blender reimport when available.
python tools/blender/spaceStation/validate_hangar.py
blender -b --python tools/blender/spaceStation/validate_hangar.py
"""
from pathlib import Path
import json, struct, hashlib, math
ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'assets/models/environment/spaceStation'
OUT=ROOT/'docs/references/hangar'
raw=(BASE/'exports/citadel_hangar_interior.glb').read_bytes()
magic,version,length=struct.unpack_from('<III',raw)
assert magic==0x46546c67 and version==2 and length==len(raw)
n=struct.unpack_from('<I',raw,12)[0]
doc=json.loads(raw[20:20+n])
assert not doc.get('animations') and not doc.get('cameras')
assert all(not b.get('uri') for b in doc['buffers'])
assert not doc.get('images'), 'Uses the source constant materials, no missing external images'
assert not any('PREVIEW' in node.get('name','') for node in doc['nodes'])
assert len(doc['meshes'])==11
triangles=0;max_vertices=0
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        a=doc['accessors'][primitive['attributes']['POSITION']]
        max_vertices=max(max_vertices,a['count'])
        assert a['count']<65536, 'Raylib 16-bit index limit'
        assert all(math.isfinite(v) for v in a['min']+a['max'])
        triangles+=doc['accessors'][primitive['indices']]['count']//3
source_hash=hashlib.sha256((BASE/'citadel_space_station.blend').read_bytes()).hexdigest()
before=json.loads((OUT/'source-inspection.json').read_text(encoding='utf-8'))
assert before['sha256']==source_hash, 'Original source must stay untouched'
dock=json.loads((ROOT/'config/station.json').read_text(encoding='utf-8'))['dock']
assert [dock[k] for k in ['room_width','room_height','room_depth','width','height']]==[36,16,56,20,12]
report={'status':'PASS','triangles':triangles,'meshes':len(doc['meshes']),
        'max_vertices_per_primitive':max_vertices,'materials':len(doc['materials']),
        'external_dependencies':0,'textures':'Not required: original constant PBR/emissive materials',
        'source_unchanged':True,'source_sha256':source_hash,'blender_reimport':False}
try: import bpy
except ImportError: pass
else:
    from mathutils import Vector
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(BASE/'exports/citadel_hangar_interior.glb'))
    meshes=[o for o in bpy.context.scene.objects if o.type=='MESH']
    assert len(meshes)==11
    points=[o.matrix_world@Vector(v) for o in meshes for v in o.bound_box]
    bounds=[[min(v[i] for v in points),max(v[i] for v in points)] for i in range(3)]
    assert bounds[2][0]<0 and 15.9<bounds[2][1]<17
    assert bounds[0][0]<-18 and bounds[0][1]>18
    report.update(blender_reimport=True,blender_bounds=bounds)
    report['reimport_materials']=[m.name for m in bpy.data.materials if m.name.startswith('CI | ')]
(OUT/('export-validation-blender.json' if report['blender_reimport'] else 'export-validation.json')).write_text(
    json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report,indent=2))
