"""Expanded scene resource audit. Reopens blend, validates every removable volume and reimports GLB."""
from pathlib import Path
import bpy,json,hashlib,struct
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'assets/models/minigames/minecraft/rush'
REPORT=ROOT/'docs/references/minecraft/phase5'
def check(ok,message):
    if not ok:raise AssertionError(message)
def bounds(objects):
    p=[o.matrix_world@Vector(v) for o in objects for v in o.bound_box]
    return [[min(v[i] for v in p) for i in range(3)],[max(v[i] for v in p) for i in range(3)]]
def main():
    report=dict(status='running',checks=[])
    try:
        protected=json.loads((REPORT/'protected-sources.json').read_text(encoding='utf8'))
        for path,digest in protected.items():check(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,'Original changed: '+path)
        data=json.loads((OUT/'minecraft_mine.scene.json').read_text());c=data['comparison']
        check(c['ratio']>=2 and c['rooms']==14 and c['tunnels']==18,'Expansion too small')
        bpy.ops.wm.open_mainfile(filepath=str(OUT/'minecraft_mine.blend'))
        check(bpy.context.scene.name=='Diamond_Rush_Expanded','Wrong scene')
        check(bpy.context.scene.unit_settings.scale_length==1,'Wrong scale')
        blocks=data['interactive_blocks'];check(len(blocks)==88,'Extractable count')
        check(sum(b['kind']=='diamond' for b in blocks)==16,'Diamond count')
        boxes=[c for c in data['colliders'] if c['shape']=='aabb']
        for b in blocks:
            obj=bpy.data.objects.get(b['id']);check(obj is not None and len(obj.data.polygons)==6,'Merged ore/block')
            own=next(c for c in boxes if c['name']==b['id']);check(own['min']==b['min'] and own['max']==b['max'],'Collider differs')
            check(all(not all(c['min'][i]<b['max'][i] and c['max'][i]>b['min'][i] for i in range(3)) for c in boxes if c['name']!=b['id']),'Static overlap on '+b['id'])
        for r in data['lava_regions']:
            p=r['safe_return'];check(r['damage_enabled'] and p[2]==0,'Hazard data incomplete')
            check(not any(c['min'][2]<1.8 and c['max'][2]>0 and c['min'][0]-.35<p[0]<c['max'][0]+.35 and c['min'][1]-.35<p[1]<c['max'][1]+.35 for c in boxes),'Lava return blocked')
            check(not any(o['min'][0]-.35<p[0]<o['max'][0]+.35 and o['min'][1]-.35<p[1]<o['max'][1]+.35 for o in data['lava_regions']),'Lava return inside lava')
        visuals=[o for name in ('DRM_Static','DRM_Interactive','DRM_Lava') for o in bpy.data.collections[name].objects]
        for o in visuals:
            check(o.type=='MESH' and o.data.uv_layers.active is not None,'Missing mesh/UV')
            check(not any(f.use_smooth for f in o.data.polygons),'Non-flat surface')
        images=[n for mat in bpy.data.materials if mat.use_nodes for n in mat.node_tree.nodes if n.type=='TEX_IMAGE']
        check(all(n.interpolation=='Closest' and n.image.packed_file and tuple(n.image.size)==(16,16) for n in images),'Pixel texture contract')
        report['checks'].extend(['Expanded editable blend opens with cubic geometry, packed 16x16 images and Closest UVs',
            '88 individually removable blocks retain exclusive collider volumes and original scale',
            'Four active lava regions have safe ground outside every lava volume'])
        # Floor coverage, excluding occupied gate/formation cells; every free cell has visual support.
        bpy.context.view_layer.update();deps=bpy.context.evaluated_depsgraph_get();floor_checked=0
        for c in data['floor_cells']:
            x,y=c['x']+.5,c['y']+.5
            if any(b['min'][2]<1.8 and b['max'][2]>0 and b['min'][0]-.35<x<b['max'][0]+.35 and b['min'][1]-.35<y<b['max'][1]+.35 for b in boxes):continue
            hit,p,*_=bpy.context.scene.ray_cast(deps,Vector((x,y,1.79)),Vector((0,0,-1)),distance=2)
            check(hit and -.001<=p.z<=.11,'Missing floor at %s,%s'%(x,y));floor_checked+=1
        names={o.name for o in visuals};expected_bounds=bounds(visuals);triangles=sum(len(f.vertices)-2 for o in visuals for f in o.data.polygons)
        blob=(OUT/'minecraft_mine.glb').read_bytes();size=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+size])
        check(all('uri' not in i for i in doc['images']+doc['buffers']),'External dependency')
        check(all(s['magFilter']==9728 and s['minFilter']==9728 for s in doc['samplers']),'Blurred sampling')
        check(all(any(n.get('name')==b['id'] for n in doc['nodes']) for b in blocks),'Merged GLB group')
        palette={m.name for o in visuals for m in o.data.materials};check(palette=={m['name'] for m in doc['materials']},'Material palette changed')
        bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(OUT/'minecraft_mine.glb'))
        imported=[o for o in bpy.context.scene.objects if o.type=='MESH']
        check({o.name for o in imported}==names,'Roundtrip object loss')
        check(sum(len(f.vertices)-2 for o in imported for f in o.data.polygons)==triangles,'Triangle loss')
        check(max(abs(a-b) for aa,bb in zip(bounds(imported),expected_bounds) for a,b in zip(aa,bb))<.001,'Roundtrip scale/orientation')
        check(all(o.data.uv_layers.active for o in imported),'Roundtrip UV loss')
        report['checks'].append('GLB reimport preserves all meshes, UVs, materials, individual nodes, scale and axes')
        report.update(status='passed',comparison=c,floor_rays=floor_checked,meshes=len(imported),triangles=triangles,materials=len(palette),images=len(doc['images']),lights=len(data['lights']),colliders=len(boxes),bounds_blender=expected_bounds,protected_files=len(protected))
    except Exception as e:report.update(status='failed',error=str(e));raise
    finally:(REPORT/'asset-validation.json').write_text(json.dumps(report,indent=2),encoding='utf8')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
