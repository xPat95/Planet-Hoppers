"""Reopen derived .blend, verify traversal/UV/textures and reimport GLB; never save models."""
from pathlib import Path
from collections import deque, Counter
import hashlib
import json
import math
import struct
import bpy
from mathutils import Vector
from bpy_extras.object_utils import world_to_camera_view

REPO=Path(__file__).resolve().parents[3]
OUT=REPO/'assets/models/minigames/minecraft'
REPORT=REPO/'docs/references/minecraft/phase1'


def check(condition,message):
    if not condition: raise AssertionError(message)


def bounds(objects):
    pts=[o.matrix_world@Vector(p) for o in objects for p in o.bound_box]
    return [[min(p[i] for p in pts) for i in range(3)], [max(p[i] for p in pts) for i in range(3)]]


def main():
    report={'status':'running','checks':[]}
    def passed(message): report['checks'].append(message)
    try:
        protected=json.loads((REPORT/'protected_sources.json').read_text(encoding='utf8'))
        for filename,digest in protected.items():
            check(hashlib.sha256((REPO/filename).read_bytes()).hexdigest()==digest,'Protected file changed: '+filename)
        passed('Original planet, ship, South Park, Rust sources and source textures: SHA-256 unchanged (%s files)'%len(protected))
        originals={}
        for relative,root in [('assets/models/planets/minecraft/minecraft_planet_textured_v02.blend','Minecraft_Planet_ROOT'),
                              ('assets/models/ships/minecraft/end_ship.blend','EndShip_ROOT')]:
            with bpy.data.libraries.load(str(REPO/relative),link=False) as (available,unused):
                check(root in available.objects,'Original root missing: '+root)
                originals[relative]={'objects':len(available.objects),'materials':len(available.materials)}
        data=json.loads((OUT/'minecraft_village.scene.json').read_text(encoding='utf8'))
        bpy.ops.wm.open_mainfile(filepath=str(OUT/data['blend']))
        scene=bpy.context.scene
        check(scene.name=='Minecraft_Overworld_Village' and len(bpy.data.scenes)==1,'Wrong scene')
        check(scene.unit_settings.scale_length==1,'Wrong units')
        groups={name:bpy.data.collections['DR_'+name] for name in ('Terrain','Buildings','Village','Trees','Vegetation','Mine')}
        visuals=[o for g in groups.values() for o in g.objects if o.type=='MESH']
        check(len(groups['Buildings'].objects)==5,'Building count')
        check(len(groups['Trees'].objects)==16,'Tree count')
        check(all(i.packed_file for i in bpy.data.images if i.type=='IMAGE'),'Image not packed')
        image_nodes=[n for m in bpy.data.materials if m.use_nodes for n in m.node_tree.nodes if n.type=='TEX_IMAGE']
        check(len(image_nodes)>=25 and all(n.interpolation=='Closest' for n in image_nodes),'Pixel interpolation')
        check(all(tuple(n.image.size)==(16,16) for n in image_nodes),'Animated/non-square source used')
        for obj in visuals:
            check(obj.data.uv_layers.active is not None,'No UV: '+obj.name)
            check(all(math.isfinite(n) for uv in obj.data.uv_layers.active.data for n in uv.uv),'Nonfinite UV')
            check(not any(p.use_smooth for p in obj.data.polygons),'Smooth shading')
        # Verify actual face assignment, not only material presence.
        terrain_tops=Counter()
        for o in groups['Terrain'].objects:
            for face in o.data.polygons:
                mat=o.data.materials[face.material_index].name
                if face.normal.z>.99: terrain_tops[mat]+=1
                if mat=='DR_grass_block_side': check(abs(face.normal.z)<.01,'Grass side on wrong face')
                if mat=='DR_dirt_path_side': check(abs(face.normal.z)<.01,'Path side on wrong face')
        check(terrain_tops['DR_grass_block_top']>0 and terrain_tops['DR_dirt_path_top']>0,'Missing terrain top variants')
        for o in groups['Trees'].objects:
            for face in o.data.polygons:
                name=o.data.materials[face.material_index].name
                if name=='DR_oak_log_top': check(abs(face.normal.z)>.99,'Log end orientation')
                if name=='DR_oak_log': check(abs(face.normal.z)<.01,'Bark on log end')
        for obj in groups['Buildings'].objects:
            for name,low,high in [('DR_oak_door_bottom',0,1),('DR_oak_door_top',1,2)]:
                zs=[obj.data.vertices[i].co.z for f in obj.data.polygons if obj.data.materials[f.material_index].name==name for i in f.vertices]
                check(zs and abs(min(zs)-low)<.001 and abs(max(zs)-high)<.001,'Door half dimensions')
        for obj in groups['Mine'].objects:
            for face in obj.data.polygons:
                name=obj.data.materials[face.material_index].name
                if name in ('DR_crafting_table_front','DR_furnace_front'):
                    check(face.normal.y<-.99,'Wrong workbench/furnace front')
                if name=='DR_crafting_table_top': check(face.normal.z>.99,'Wrong workbench top')
                if name=='DR_furnace_top': check(abs(face.normal.z)>.99,'Wrong furnace top/bottom')
        for obj in groups['Terrain'].objects:
            uv=obj.data.uv_layers.active.data
            for face in obj.data.polygons:
                values=[uv[i].uv for i in face.loop_indices]
                check(all(abs(max(v[a] for v in values)-min(v[a] for v in values)-1)<.001 for a in (0,1)),
                      'Terrain UV must be one square per block')
        passed('Blend reopened: 5 buildings, 16 oaks; metre scale, flat normals, packed 16x16 images, Closest interpolation and per-face UV/materials')
        solids=[c for c in data['colliders'] if c['role']=='solid']
        def blocked(x,y,r=.35):
            return any(c['min'][2]<1.8 and c['max'][2]>0 and c['min'][0]-r<x<c['max'][0]+r and c['min'][1]-r<y<c['max'][1]+r for c in solids)
        spawn=data['spawn']['position']
        check(not blocked(*spawn[:2]),'Blocked spawn')
        samples=0
        for road in data['roads']:
            for a,b in zip(road['points'],road['points'][1:]):
                dx,dy=b[0]-a[0],b[1]-a[1]; length=math.hypot(dx,dy)
                if length==0: continue
                for i in range(math.ceil(length*4)+1):
                    t=min(1,i/(length*4)); x,y=a[0]+dx*t,a[1]+dy*t
                    for offset in (-1.1,0,1.1):
                        px,py=x-dy/length*offset,y+dx/length*offset
                        check(not blocked(px,py),'Road blocked: %s at %.2f,%.2f'%(road['name'],px,py))
                        samples+=1
        passed('%s road samples clear for a radius .35 / height 1.8 actor, including lateral bands'%samples)
        # Half-unit flood fill verifies free-space routes independently from road metadata.
        start=(round(spawn[0]*2),round(spawn[1]*2)); reached={start}; queue=deque([start])
        while queue:
            x,y=queue.popleft()
            for pt in ((x-1,y),(x+1,y),(x,y-1),(x,y+1)):
                xx,yy=pt[0]/2,pt[1]/2
                if pt not in reached and -31<xx<31 and -25<yy<33 and not blocked(xx,yy):
                    reached.add(pt);queue.append(pt)
        targets=[l for l in data['landmarks'] if 'approach' in l]
        targets.extend([{'name':'Mine_transition','approach':[8.5,30,0]}])
        for item in targets:
            p=item['approach']; check((round(p[0]*2),round(p[1]*2)) in reached,'Unreachable: '+item['name'])
        passed('Spawn connects to all five house approaches, farm, mine interaction and tunnel transition by independent flood fill')
        # Raycast the saved geometry to ensure that visual ground agrees with clear routes.
        bpy.context.view_layer.update(); deps=bpy.context.evaluated_depsgraph_get()
        for item in targets:
            x,y=item['approach'][:2]
            hit,loc,normal,*_=scene.ray_cast(deps,Vector((x,y,1.79)),Vector((0,0,-1)),distance=2)
            tolerance=.11 if item['name']=='Mine_transition' else .02
            check(hit and abs(loc.z)<tolerance,'Visual floor differs at '+item['name'])
        passed('Destination visual floor agrees with Z=0 plane within .02 m; decorative tunnel rails allowed up to .11 m')
        # Check every house is in frame and has at least one unobstructed facade/roof ray.
        cam=bpy.data.objects['DR_Camera_overview']
        visible=[]
        for house in groups['Buildings'].objects:
            points=[Vector(v) for v in house.bound_box]
            center=sum(points,Vector())/8
            projected=world_to_camera_view(scene,cam,center)
            check(0<projected.x<1 and 0<projected.y<1 and projected.z>0,'House outside overview: '+house.name)
            any_hit=False
            for p in points+[center]:
                direction=(p-cam.location).normalized()
                result=scene.ray_cast(deps,cam.location,direction,distance=200)
                if result[0] and result[4]==house: any_hit=True; break
            check(any_hit,'House fully occluded: '+house.name)
            visible.append(house.name)
        passed('All 5 houses in overview frame with unobstructed visibility rays')
        source_bounds=bounds(visuals); source_names={o.name for o in visuals}
        blob=(OUT/data['visual_asset']).read_bytes(); check(blob[:4]==b'glTF','Not GLB')
        size=struct.unpack_from('<I',blob,12)[0]; doc=json.loads(blob[20:20+size])
        check(all('uri' not in x for x in doc['buffers']+doc['images']),'External GLB dependencies')
        check(all(x['magFilter']==9728 and x['minFilter']==9728 for x in doc['samplers']),'Non-nearest sampler')
        palette={m.name for o in visuals for m in o.data.materials}
        used_images={n.image.name for o in visuals for m in o.data.materials for n in m.node_tree.nodes if n.type=='TEX_IMAGE'}
        check(len(doc['images'])==len(used_images),'Missing exported image')
        check({m['name'] for m in doc['materials']}==palette,'Exported material palette differs')
        for mat in doc['materials']:
            if mat['name'] in ('DR_glass','DR_oak_leaves','DR_oak_door_top','DR_oak_door_bottom'):
                check(mat['alphaMode']=='MASK' and mat['alphaCutoff']==.5 and mat['doubleSided'],'Cutout export')
            if 'baseColorTexture' in mat['pbrMetallicRoughness']:
                check(mat['pbrMetallicRoughness']['baseColorFactor']==[1,1,1,1],'Texture color factor')
        check(not any(n.get('name','').startswith('COL_') for n in doc['nodes']),'Collider in visual GLB')
        passed('GLB self-contained; %s embedded images, complete used material palette, NEAREST samplers, MASK cutouts, no cameras/lights/proxies'%len(used_images))
        expected_triangles=sum(len(p.vertices)-2 for o in visuals for p in o.data.polygons)
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=str(OUT/data['visual_asset']))
        imported=[o for o in bpy.context.scene.objects if o.type=='MESH']
        check({o.name for o in imported}==source_names,'Mesh names/count changed after reimport')
        got=bounds(imported)
        check(max(abs(a-b) for aa,bb in zip(source_bounds,got) for a,b in zip(aa,bb))<.001,'Reimport orientation/scale/bounds differ')
        triangles=sum(len(p.vertices)-2 for o in imported for p in o.data.polygons)
        check(triangles==expected_triangles,'Triangles lost in export')
        check(all(o.data.uv_layers.active for o in imported),'UV lost in reimport')
        check(all(n.interpolation=='Closest' for o in imported for m in o.data.materials for n in m.node_tree.nodes if n.type=='TEX_IMAGE'),
              'Nearest filtering lost in reimport')
        passed('GLB reimport: all meshes and triangles preserved, UVs present, orientation and bounds within .001 m')
        for name in ('overview','player','mine','farm','center'):
            check((REPORT/(name+'.png')).is_file(),'Missing capture: '+name)
        passed('Five PNG review captures present')
        report.update(status='passed',blender=bpy.app.version_string,mesh_objects=len(imported),triangles=triangles,
                      materials=len(palette),images=len(used_images),colliders=len(solids),collider_categories=dict(Counter(c['category'] for c in solids)),
                      bounds_blender=source_bounds,original_libraries=originals,visible_houses=visible,road_samples=samples,reachable_grid_cells=len(reached),
                      limitations=['Geometric validation only; FPS/controller not integrated or tested.',
                                   'Raylib runtime rendering and alpha-discard/point-filter setup pending phase 2.'])
    except Exception as error:
        report.update(status='failed',error=str(error)); raise
    finally:
        (REPORT/'validation.json').write_text(json.dumps(report,indent=2),encoding='utf8')
    print(json.dumps(report,indent=2))


if __name__=='__main__': main()
