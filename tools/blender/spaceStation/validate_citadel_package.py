"""Validacion del paquete sin Blender: python tools/blender/spaceStation/validate_citadel_package.py."""
from pathlib import Path
import ast
import hashlib
import json
import re
import struct

REPO=Path(__file__).resolve().parents[3]
MODEL=REPO/'assets/models/environment/spaceStation'
REFS=REPO/'docs/references/spaceStation'
TOOLS=Path(__file__).resolve().parent
VALIDATION=REFS/'package_validation.json'
VALIDATION.write_text(json.dumps({'valid':False,'status':'checking'}),encoding='utf-8')
asset=json.loads((MODEL/'asset.json').read_text(encoding='utf-8'))
audit=json.loads((REPO/asset['audit']).read_text(encoding='utf-8'))
assert audit['source_geometry_preserved']
assert audit['source_unchanged_during_export']
assert hashlib.sha256((REPO/asset['source']).read_bytes()).hexdigest()==audit['integrated_source_sha256']
for key in ('source','near','far','obj','audit','generator','exporter'):
    assert (REPO/asset[key]).is_file(),key
for p in TOOLS.glob('*.py'):ast.parse(p.read_text(encoding='utf-8'),filename=str(p))
for doc in (MODEL/'README.md',REFS/'README.md',TOOLS/'README.md'):
    for href in re.findall(r'\]\(([^)]+)\)',doc.read_text(encoding='utf-8')):
        if href.startswith(('http:','https:','#')):continue
        assert (doc.parent/href.split('#')[0]).exists(),(doc,href)
for level in ('near','far'):
    path=REPO/asset[level];data=path.read_bytes()
    assert data[:4]==b'glTF' and struct.unpack_from('<I',data,4)[0]==2
    assert struct.unpack_from('<I',data,8)[0]==len(data)
    size=struct.unpack_from('<I',data,12)[0]
    gltf=json.loads(data[20:20+size])
    assert not gltf.get('cameras') and not gltf.get('animations')
    assert not any(b.get('uri') for b in gltf.get('buffers',[]))
    assert not any(i.get('uri') for i in gltf.get('images',[]))
    roots=[n for n in gltf['nodes'] if n.get('name')==asset['root']]
    assert len(roots)==1 and roots[0]['extras']['asset_role']=='space_station'
    assert sum(n.get('name','').startswith('CI | BRAZO ') for n in gltf['nodes'])==5
    triangles=sum(gltf['accessors'][p['indices']]['count']//3
                  for m in gltf['meshes'] for p in m['primitives'])
    assert triangles==audit['exports'][level]['triangles']
    assert audit['exports'][level]['reimport']=='OK'
    assert audit['exports'][level]['degenerate_triangles']==0
    assert hashlib.sha256(data).hexdigest()==audit['exports'][level]['sha256']
    assert any(any(x>0 for x in m.get('emissiveFactor',[])) for m in gltf['materials'])
obj=REPO/asset['obj'];mtl=obj.with_suffix('.mtl')
names=set()
for line in mtl.read_text(encoding='utf-8').splitlines():
    if line.startswith('newmtl '):names.add(line[7:])
    if line.startswith(('map_Kd ','map_Ke ','map_Bump ','norm ')):
        assert (mtl.parent/line.split()[-1]).is_file()
triangles=0
for line in obj.open(encoding='utf-8'):
    if line.startswith('mtllib '):assert (obj.parent/line[7:].strip()).is_file()
    if line.startswith('usemtl '):assert line[7:].strip() in names
    if line.startswith('f '):
        assert len(line.split())==4
        triangles+=1
assert triangles==audit['exports']['near']['triangles']
report={'valid':True,'checks':['python_syntax','documentation_links','source_hash','self_contained_glb',
    'five_wards','station_role','emissive_materials','glb_reimport_audit','obj_mtl','triangle_counts'],
    'near_triangles':audit['exports']['near']['triangles'],'far_triangles':audit['exports']['far']['triangles']}
VALIDATION.write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report))
