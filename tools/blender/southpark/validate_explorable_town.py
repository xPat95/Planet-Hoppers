"""Asset/integration checks. Run with Blender --background --python-exit-code 1 --python.

Reopens the delivered blend and reimports the delivered GLB in a disposable scene.
Writes the validation report only; does not save either model.
"""
import hashlib
import json
import math
import statistics
import struct
from collections import Counter, deque
from pathlib import Path

import bpy
from mathutils import Vector

REPO = Path(__file__).resolve().parents[3]
ASSETS = REPO/'assets/models/minigames/southpark'
REPORT = REPO/'docs/references/southpark/phase1/validation.json'


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def main():
    data = json.loads((ASSETS/'southpark_town.scene.json').read_text(encoding='utf-8'))
    source = REPO/data['source']
    check(hashlib.sha256(source.read_bytes()).hexdigest() == data['source_sha256'], 'Source planet changed')
    bpy.ops.wm.open_mainfile(filepath=str(ASSETS/data['blend']))
    scene = bpy.context.scene
    check(scene.name == 'SouthPark_Explorable', 'Wrong active scene')
    check(len(bpy.data.scenes) == 1, 'Source/default scene leaked into asset')
    check(not any(o.name == 'SouthPark_Planet_ROOT' for o in scene.objects), 'Spherical planet leaked into asset')
    counts = Counter(a['category'] for a in data['assets'])
    check(counts == {'Buildings':160,'Trees':220,'Mountains':14,'Decoration':1}, 'Missing reused assets')
    check(len(data['roads']) == 15, 'Missing historical road route')
    check(scene.unit_settings.scale_length == 1, 'World is not in metres')
    ground = bpy.data.objects['SPT_Snow_Ground']
    check(abs(max(v.co.z for v in ground.data.vertices)) < 1e-6, 'Ground must be flat at Z=0')
    meshes = [o for o in bpy.data.collections['SPT_Visuals'].all_objects if o.type == 'MESH']
    expected_names = {o.name for o in meshes}
    materials = {m for o in meshes for m in o.data.materials}
    check(len(materials) == 27, 'Unexpected material palette (only the lake material is omitted)')
    wanted=sorted(m.name for m in materials)
    with bpy.data.libraries.load(str(source),link=False) as (available,loaded):
        check(set(wanted)=={m for m in available.materials if m.startswith('SP_')}-{'SP_Lake'}, 'Source material names changed')
        loaded.materials=wanted[:]
    for name,original in zip(wanted,loaded.materials):
        old=next(n for n in original.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
        new=next(n for n in bpy.data.materials[name].node_tree.nodes if n.type=='BSDF_PRINCIPLED')
        for field in ('Base Color','Roughness','Metallic','Alpha'):
            a,b=old.inputs[field].default_value,new.inputs[field].default_value
            check(max(abs(x-y) for x,y in zip(a,b))<1e-6 if field=='Base Color' else abs(a-b)<1e-6, 'Material changed: '+name)
    for material in materials:
        node = next(n for n in material.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
        check(node.inputs['Alpha'].default_value == 1, 'Unexpected transparency')
        check(not any(n.type == 'TEX_IMAGE' for n in material.node_tree.nodes), 'Unexpected texture dependency')
    doors = []
    for asset in data['assets']:
        low,high = asset['bounds']
        check(all(math.isfinite(v) for v in low+high), 'Non-finite object bounds')
        check(low[0]>=-350 and high[0]<=370 and low[1]>=-160 and high[1]<=175, 'Asset outside snowy ground: '+asset['source'])
        check(abs(low[2]+.02)<.002, 'Floating or sunk asset: '+asset['source'])
        for name in asset['objects']:
            obj = bpy.data.objects[name]
            check(obj.matrix_world.to_3x3().col[2].normalized().z > .999, 'Tilted asset: '+name)
        if asset['category'] == 'Buildings' and asset['source'].endswith('_House'):
            obj=bpy.data.objects[asset['objects'][0]]
            indices={i for p in obj.data.polygons if obj.data.materials[p.material_index].name=='SP_Door' for i in p.vertices}
            if indices:
                heights=[(obj.matrix_world @ obj.data.vertices[i].co).z for i in indices]
                doors.append(max(heights)-min(heights))
    check(len(doors)==118, 'Missing residential doors')
    check(all(1.8<h<2.6 for h in doors), 'Residential door is not at human scale')

    buildings = [a for a in data['assets'] if a['category']=='Buildings']
    for index,a in enumerate(buildings):
        for b in buildings[index+1:]:
            check(any(a['bounds'][1][i] < b['bounds'][0][i] or b['bounds'][1][i] < a['bounds'][0][i] for i in range(2)), 'Overlapping buildings')
    solids=[c for c in data['colliders'] if c['role']=='solid']
    check(len(solids)==395, 'Missing building/tree/mountain/sign collider')

    def blocked(point, radius=.35):
        return any(all(c['min'][i]-radius <= point[i] <= c['max'][i]+radius for i in range(2)) for c in solids)

    spawn=data['spawn']['position']
    check(not blocked(spawn), 'Spawn inside a collider')
    check(abs(spawn[2]-1.7)<1e-5, 'Wrong player eye height')
    # Test a 70 cm player along the centre and two walking lanes of every street.
    samples=0
    segments=[]
    for road in data['roads']:
        for a,b in zip(road['points'],road['points'][1:]):
            a,b=Vector(a).xy,Vector(b).xy
            segments.append((a,b))
            d=b-a
            side=Vector((-d.y,d.x)).normalized()
            for j in range(math.ceil(d.length)+1):
                center=a.lerp(b,min(1,j/d.length))
                for lane in (-2,0,2):
                    p=center+side*lane
                    check(not blocked(p), 'Blocked street lane: '+road['name'])
                    samples+=1
    # Navigation connectivity over the actual road widths, including intersections
    # in the middle of segments (not just matching path endpoints).
    def distance(p,a,b):
        d=b-a
        return (p-(a+d*max(0,min(1,(p-a).dot(d)/d.length_squared)))).length
    grid=set()
    for a,b in segments:
        for x in range(math.floor(min(a.x,b.x)/2)-2,math.ceil(max(a.x,b.x)/2)+3):
            for y in range(math.floor(min(a.y,b.y)/2)-2,math.ceil(max(a.y,b.y)/2)+3):
                p=Vector((x*2,y*2))
                if distance(p,a,b)<=2.65 and not blocked(p):
                    grid.add((x,y))
    start=min(grid,key=lambda p:(p[0]*2-spawn[0])**2+(p[1]*2-spawn[1])**2)
    reached={start}
    queue=deque([start])
    while queue:
        x,y=queue.popleft()
        for p in ((x-1,y),(x+1,y),(x,y-1),(x,y+1)):
            if p in grid and p not in reached:
                reached.add(p)
                queue.append(p)
    check(reached==grid, 'Road network has disconnected walkable sections')

    # Validate that the artifact is self-contained, then test Blender's glTF loader.
    blob=(ASSETS/data['visual_asset']).read_bytes()
    magic,version,total=struct.unpack_from('<4sII',blob)
    check(magic==b'glTF' and version==2 and total==len(blob), 'Invalid GLB container')
    length,chunk_type=struct.unpack_from('<II',blob,12)
    check(chunk_type==0x4E4F534A, 'Missing GLB JSON')
    gltf=json.loads(blob[20:20+length])
    check(not any('uri' in b for b in gltf['buffers']), 'External binary dependency')
    check(not gltf.get('images'), 'Unexpected image dependencies')
    check({m['name'] for m in gltf['materials']}=={m.name for m in materials}, 'GLB material palette changed')
    check(not any(n.get('name','').startswith('COL_') for n in gltf['nodes']), 'Collision proxy leaked into visual export')
    expected_bounds={o.name:[[min((o.matrix_world@Vector(v))[i] for v in o.bound_box) for i in range(3)], [max((o.matrix_world@Vector(v))[i] for v in o.bound_box) for i in range(3)]] for o in meshes}
    for obj in list(bpy.data.objects):
        obj.name='Validation_Original_'+obj.name
    imported=bpy.data.scenes.new('Validation_Reimport')
    bpy.context.window.scene=imported
    bpy.ops.import_scene.gltf(filepath=str(ASSETS/data['visual_asset']))
    actual=[o for o in imported.objects if o.type=='MESH']
    check(len(actual)==len(expected_names), 'GLB visual mesh count changed')
    maximum_error=0
    for obj in actual:
        name=obj.name
        check(name in expected_names, 'Unexpected reimported mesh: '+obj.name)
        for side,func in enumerate((min,max)):
            for axis in range(3):
                value=func((obj.matrix_world@Vector(v))[axis] for v in obj.bound_box)
                maximum_error=max(maximum_error,abs(value-expected_bounds[name][side][axis]))
    check(maximum_error<.002, 'GLB position/axis conversion changed bounds')
    report={'status':'PASS','source_unchanged':True,'source_sha256':data['source_sha256'],
            'assets':dict(counts),'visual_meshes':len(meshes),'materials':len(materials),
            'external_textures':0,'solid_proxies':len(solids),'roads':len(data['roads']),
            'street_lane_samples':samples,'connected_navigation_cells':len(reached),
            'residential_doors_m':{'min':min(doors),'median':statistics.median(doors),'max':max(doors)},
            'glb_reimport':'PASS','glb_bounds_max_error_m':maximum_error,
            'limits':['No game runtime exists yet; engine/solar regression tests unavailable.',
                      'Navigation checks validate prepared geometry, not a physics controller.']}
    REPORT.write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding='utf-8')
    print('TOWN_VALIDATION',json.dumps(report))


if __name__=='__main__':
    main()
