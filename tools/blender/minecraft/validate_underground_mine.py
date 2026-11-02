"""Reopen .blend, validate intact/open-gate topology, then reimport GLB. No model writes."""
from pathlib import Path
from collections import deque,Counter
import hashlib,json,math,struct
import bpy
from mathutils import Vector

REPO=Path(__file__).resolve().parents[3]
OUT=REPO/'assets/models/minigames/minecraft/mine'
REPORT=REPO/'docs/references/minecraft/phase3'


def check(ok,message):
    if not ok:raise AssertionError(message)


def bounds(objects):
    points=[o.matrix_world@Vector(p) for o in objects for p in o.bound_box]
    return [[min(p[i] for p in points) for i in range(3)],[max(p[i] for p in points) for i in range(3)]]


def accessibility(data,open_gates):
    boxes=[c for c in data['colliders'] if c['role']=='solid' and not(open_gates and c['category']=='breakable')]
    def blocked(x,y):
        return any(c['min'][2]<1.8 and c['max'][2]>0 and c['min'][0]-.35<x<c['max'][0]+.35 and c['min'][1]-.35<y<c['max'][1]+.35 for c in boxes) or any(
            r['min'][0]-.35<x<r['max'][0]+.35 and r['min'][1]-.35<y<r['max'][1]+.35 for r in data['lava_regions'])
    cells={(c['x'],c['y']) for c in data['floor_cells'] if not blocked(c['x']+.5,c['y']+.5)}
    start=(0,0);check(start in cells,'Spawn obstructed')
    reached={start};queue=deque([start])
    while queue:
        x,y=queue.popleft()
        for p in ((x-1,y),(x+1,y),(x,y-1),(x,y+1)):
            if p in cells and p not in reached:reached.add(p);queue.append(p)
    diamonds=[b['id'] for b in data['interactive_blocks'] if b['kind']=='diamond' and tuple(math.floor(v) for v in b['approach'][:2]) in reached]
    rooms=[n['id'] for n in data['graph']['nodes'] if tuple(math.floor(v) for v in n['center'][:2]) in reached]
    return dict(diamonds=diamonds,rooms=rooms,reached_cells=len(reached))


def main():
    report=dict(status='running',checks=[])
    try:
        protected=json.loads((REPORT/'protected-sources.json').read_text(encoding='utf8'))
        for path,digest in protected.items():check(hashlib.sha256((REPO/path).read_bytes()).hexdigest()==digest,'Protected source changed: '+path)
        report['checks'].append('All %s protected sources unchanged'%len(protected))
        data=json.loads((OUT/'minecraft_mine.scene.json').read_text())
        bpy.ops.wm.open_mainfile(filepath=str(OUT/data['blend']))
        scene=bpy.context.scene;check(scene.name=='Minecraft_Underground_Mine' and len(bpy.data.scenes)==1,'Wrong scene')
        check(scene.unit_settings.scale_length==1,'Wrong scale')
        visuals=[o for name in ('DRM_Static','DRM_Interactive','DRM_Lava') for o in bpy.data.collections[name].objects]
        check(all(o.type=='MESH' and o.data.uv_layers.active for o in visuals),'Missing mesh/UV')
        check(all(not p.use_smooth for o in visuals for p in o.data.polygons),'Smoothed cube faces')
        check(all(i.packed_file for i in bpy.data.images if i.users),'External image')
        textures=[n for m in bpy.data.materials if m.use_nodes for n in m.node_tree.nodes if n.type=='TEX_IMAGE']
        check(len(textures)==16 and all(n.interpolation=='Closest' and tuple(n.image.size)==(16,16) for n in textures),'Pixel textures')
        for obj in visuals:
            if obj.name.startswith('Shell_'):
                check(all(len(p.vertices)==4 for p in obj.data.polygons),'Static shell not cubic quads')
                check(all(max(abs(v) for v in p.normal)>.999 for p in obj.data.polygons),'Non-axis-aligned shell face')
                uv=obj.data.uv_layers.active.data
                for face in obj.data.polygons:
                    vals=[uv[i].uv for i in face.loop_indices]
                    check(all(abs(max(v[a] for v in vals)-min(v[a] for v in vals)-1)<.001 for a in (0,1)),'Stretched shell UV')
        report['checks'].append('Blend reopened; cubic shell, metre scale, packed square images and block-scale Closest UVs')
        check(len(data['graph']['nodes'])==9 and len(data['graph']['edges'])==10,'Graph dimensions')
        blocks=data['interactive_blocks'];check(len(blocks)==26 and len({b['id'] for b in blocks})==26,'Individual IDs')
        for block in blocks:
            obj=bpy.data.objects.get(block['object']);check(obj is not None and len(obj.data.polygons)==6,'Independent full block missing')
            check(obj.get('interactive_id')==block['id'],'Object ID extras')
            c=next(c for c in data['colliders'] if c['name']==block['collider_id'])
            check(c['min']==block['min'] and c['max']==block['max'],'Block collider mismatch')
            for other in data['colliders']:
                if other['shape']!='aabb' or other['name']==c['name']:continue
                overlap=all(other['min'][i]<c['max'][i]-1e-6 and other['max'][i]>c['min'][i]+1e-6 for i in range(3))
                check(not overlap,'Other collider prevents future removal of '+block['id']+' / '+other['name'])
        intact=accessibility(data,False);opened=accessibility(data,True)
        check(len(intact['diamonds'])>=8,'Fewer than 8 safe accessible diamonds')
        check(len(opened['diamonds'])==14 and len(opened['rooms'])==9,'Future-gate traversal incomplete')
        check(set(r[0] for r in intact['rooms'])=={'A','B','C'},'Sector inaccessible')
        check(len(data['lava_regions'])==2 and all(not r['damage_enabled'] for r in data['lava_regions']),'Lava sensors')
        report['checks'].append('10 tunnels / 9 chambers; three sectors, exit return paths and safe diamond access checked while treating lava as forbidden')
        report['checks'].append('14 diamonds and 12 breakable blocks remain separate meshes; colliders have no overlapping static wall volumes')
        deps=bpy.context.evaluated_depsgraph_get()
        for cell in data['floor_cells']:
            x,y=cell['x']+.5,cell['y']+.5
            if any(b['kind']=='breakable' and b['min'][0]<=x<b['max'][0] and b['min'][1]<=y<b['max'][1] for b in blocks):continue
            hit,loc,*_=scene.ray_cast(deps,Vector((x,y,1.79)),Vector((0,0,-1)),distance=2)
            check(hit and -.001<=loc.z<=.11,'Floating navigation floor at %s,%s'%(x,y))
        report['checks'].append('Every floor cell raycast has solid visual ground; decorative rails/lava limited to 0.11 m')
        expected_names={o.name for o in visuals};expected_bounds=bounds(visuals)
        triangles=sum(len(f.vertices)-2 for o in visuals for f in o.data.polygons)
        materials={m.name for o in visuals for m in o.data.materials}
        blob=(OUT/data['visual_asset']).read_bytes();size=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+size])
        check(all('uri' not in i for i in doc['images']+doc['buffers']),'External GLB dependency')
        check(all(s['magFilter']==9728 and s['minFilter']==9728 for s in doc['samplers']),'Non-nearest GLB sampler')
        check({m['name'] for m in doc['materials']}==materials,'Exported material loss')
        check(all(any(n.get('name')==b['id'] for n in doc['nodes']) for b in blocks),'Interactive node merged')
        bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(OUT/data['visual_asset']))
        imported=[o for o in bpy.context.scene.objects if o.type=='MESH']
        check({o.name for o in imported}==expected_names,'Mesh names lost')
        check(sum(len(f.vertices)-2 for o in imported for f in o.data.polygons)==triangles,'Triangle loss')
        check(max(abs(a-b) for aa,bb in zip(bounds(imported),expected_bounds) for a,b in zip(aa,bb))<.001,'Scale/orientation changed')
        check(all(o.data.uv_layers.active for o in imported),'Reimport UV loss')
        report['checks'].append('GLB reimport preserves meshes, triangles, material palette, UVs, independent nodes, axes and bounds')
        report.update(status='passed',intact=intact,with_future_gates_open=opened,counts=data['counts'],
                      visual_meshes=len(imported),triangles=triangles,materials=len(materials),images=len(doc['images']),
                      solid_colliders=sum(c['role']=='solid' for c in data['colliders']),lights=len(data['lights']),bounds_blender=expected_bounds)
    except Exception as e:report.update(status='failed',error=str(e));raise
    finally:(REPORT/'asset-validation.json').write_text(json.dumps(report,indent=2),encoding='utf8')
    print(json.dumps(report,indent=2))


if __name__=='__main__':main()
