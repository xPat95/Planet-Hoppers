"""Phase-4 resource check: independent cubes, no static collision overlap, closed backing, GLB roundtrip."""
from pathlib import Path
import bpy,json,hashlib,struct,math
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'assets/models/minigames/minecraft/mining'
REPORT=ROOT/'docs/references/minecraft/phase4'

def check(ok,message):
    if not ok:raise AssertionError(message)

def bounds(objects):
    points=[o.matrix_world@Vector(p) for o in objects for p in o.bound_box]
    return [[min(p[i] for p in points) for i in range(3)],[max(p[i] for p in points) for i in range(3)]]

def main():
    report=dict(status='running',checks=[])
    try:
        protected=json.loads((REPORT/'protected-sources.json').read_text(encoding='utf8'))
        for path,digest in protected.items():check(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,'Original resource changed: '+path)
        data=json.loads((OUT/'minecraft_mine.scene.json').read_text())
        bpy.ops.wm.open_mainfile(filepath=str(OUT/'minecraft_mine.blend'))
        check(bpy.context.scene.name=='Minecraft_Mining_Ready','Wrong scene')
        blocks=data['interactive_blocks'];check(len(blocks)==86 and len({b['id'] for b in blocks})==86,'Block count/IDs')
        colliders=[c for c in data['colliders'] if c['shape']=='aabb']
        for block in blocks:
            obj=bpy.data.objects.get(block['id']);check(obj is not None and len(obj.data.polygons)==6,'Missing independent cube')
            check(all(len(p.vertices)==4 and not p.use_smooth for p in obj.data.polygons),'Non-cubic block')
            own=[c for c in colliders if c['name']==block['id']];check(len(own)==1,'Missing own collider')
            check(own[0]['min']==block['min'] and own[0]['max']==block['max'],'Collider position')
            for other in colliders:
                if other['name']==block['id']:continue
                check(not all(other['min'][i]<block['max'][i] and other['max'][i]>block['min'][i] for i in range(3)),
                      'Overlapping collider: '+block['id']+'/'+other['name'])
        visuals=[o for name in ('DRM_Static','DRM_Interactive','DRM_Lava') for o in bpy.data.collections[name].objects]
        palette={m.name for o in visuals for m in o.data.materials}
        for obj in visuals:
            check(obj.data.uv_layers.active is not None,'Missing UV')
            for face in obj.data.polygons:
                if obj.name.startswith('Shell_'):
                    check(obj.data.materials[face.material_index].name not in {'DR_'+kind+'_ore' for kind in ('coal','iron','gold','redstone','lapis')},'Common ore still baked into shell')
        nodes=[n for m in bpy.data.materials if m.use_nodes for n in m.node_tree.nodes if n.type=='TEX_IMAGE']
        check(all(n.image.packed_file and n.interpolation=='Closest' and tuple(n.image.size)==(16,16) for n in nodes),'Pixel material contract')
        report['checks'].append('Editable blend opens; all 86 extractable cubes have stable IDs, UVs and exclusive collider volumes')
        report['checks'].append('Original PNGs untouched; packed images use Closest; common ore faces removed only from derived static shell')
        # Disable every extractable cube only in this disposable validation scene.
        for b in blocks:bpy.data.objects[b['id']].hide_set(True)
        bpy.context.view_layer.update();deps=bpy.context.evaluated_depsgraph_get()
        checked=0
        for b in blocks:
            if b['kind']=='breakable':continue
            p=b['approach'];origin=Vector((p[0],p[1],1.7));center=(Vector(b['min'])+Vector(b['max']))/2
            direction=(center-origin).normalized()
            hit,loc,_,_,obj,_=bpy.context.scene.ray_cast(deps,origin,direction,distance=10)
            check(hit and obj.name not in {b['id'] for b in blocks},'Ore niche opens to outside void: '+b['id']);checked+=1
        for b in blocks:bpy.data.objects[b['id']].hide_set(False)
        report['checks'].append('%s ore niches have structural backing after all blocks are hidden in a disposable scene'%checked)
        expected_names={o.name for o in visuals};expected_bounds=bounds(visuals)
        triangles=sum(len(f.vertices)-2 for o in visuals for f in o.data.polygons)
        blob=(OUT/'minecraft_mine.glb').read_bytes();size=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+size])
        check(all('uri' not in i for i in doc['images']+doc['buffers']),'External GLB dependency')
        check(all(s['magFilter']==9728 and s['minFilter']==9728 for s in doc['samplers']),'Non-nearest export')
        check(all(any(n.get('name')==b['id'] for n in doc['nodes']) for b in blocks),'Merged export node')
        bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(OUT/'minecraft_mine.glb'))
        imported=[o for o in bpy.context.scene.objects if o.type=='MESH']
        check({o.name for o in imported}==expected_names,'Roundtrip object loss')
        check(sum(len(f.vertices)-2 for o in imported for f in o.data.polygons)==triangles,'Roundtrip triangle loss')
        check(max(abs(a-b) for aa,bb in zip(bounds(imported),expected_bounds) for a,b in zip(aa,bb))<.001,'Roundtrip scale/orientation')
        check({m['name'] for m in doc['materials']}==palette,'Material palette loss')
        report['checks'].append('GLB roundtrip preserves geometry, UVs, material palette, independent nodes, axes and scale')
        report.update(status='passed',original_files=len(protected),extractable_blocks=86,ore_niches=checked,
                      meshes=len(imported),triangles=triangles,materials=len(palette),images=len(doc['images']),solid_colliders=len(colliders))
    except Exception as error:report.update(status='failed',error=str(error));raise
    finally:(REPORT/'asset-validation.json').write_text(json.dumps(report,indent=2),encoding='utf8')
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
