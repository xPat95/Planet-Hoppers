"""Read-only proof of camera obstruction; compares current and constrained framing."""
from pathlib import Path
import bpy,json,math
from mathutils import Vector
from mathutils.bvhtree import BVHTree

ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'docs/references/hangar/integration'
bpy.ops.wm.open_mainfile(filepath=str(ROOT/'assets/models/environment/spaceStation/citadel_hangar_reconstructed.blend'))
verts=[];faces=[]
deps=bpy.context.evaluated_depsgraph_get()
for o in bpy.context.scene.objects:
    if o.type!='MESH' or o.get('exportable')!=True:continue
    ev=o.evaluated_get(deps);me=ev.to_mesh();offset=len(verts)
    verts.extend(o.matrix_world@v.co for v in me.vertices)
    faces.extend(tuple(offset+j for j in p.vertices) for p in me.polygons)
    ev.to_mesh_clear()
tree=BVHTree.FromPolygons(verts,faces)
cfg=json.loads((ROOT/'config/ships.json').read_text(encoding='utf-8'))
choices=json.loads((ROOT/'config/ship-selection.json').read_text(encoding='utf-8'))['choices']
results=[]
for choice in choices:
    ship=next((s for s in cfg['ships'] if s['id']==choice['ship_id']),None)
    if ship:
        temp=bpy.data.scenes.new('TEMP | ship bounds');bpy.context.window.scene=temp
        bpy.ops.import_scene.gltf(filepath=str(ROOT/ship['model']))
        bpy.context.view_layer.update()
        points=[o.matrix_world@Vector(v) for o in temp.objects if o.type=='MESH' for v in o.bound_box]
        lo=Vector(tuple(min(v[i] for v in points) for i in range(3)))
        hi=Vector(tuple(max(v[i] for v in points) for i in range(3)))
        half=(hi-lo)/2;half*=3/max(half)
        center=Vector((0,0,.8+half.z))
    else:
        half=Vector((2.3,3,.8));center=Vector((0,0,.8+.225*6/4))
    for w,h in [(960,640),(1280,800),(1920,1080)]:
        usable_h=max(.2,min(1,(h-307)/h));usable_w=max(.2,min(1,(w-180)/w))
        usable=min(usable_h,w/h*usable_w)
        radius=half.length*1.08
        distance=max(11,radius/math.sin(math.atan(math.tan(math.radians(21))*usable)))
        views=[]
        for limit in [None,17.0,16.0,15.0,14.0,13.0,12.0,11.0]:
            d=min(distance,limit) if limit else distance
            eye=center+Vector((.9,-1.4,.6)).normalized()*d
            ray=center-eye;hit=tree.ray_cast(eye,ray.normalized(),ray.length)
            corner_hits=0
            for yaw in [0,math.pi/4,math.pi/2,math.pi*3/4]:
                for sx in [-1,1]:
                    for sy in [-1,1]:
                        for sz in [-1,1]:
                            x,y=sx*half.x,sy*half.y
                            point=center+Vector((x*math.cos(yaw)-y*math.sin(yaw),x*math.sin(yaw)+y*math.cos(yaw),sz*half.z))
                            r=point-eye
                            if tree.ray_cast(eye,r.normalized(),r.length)[0] is not None:corner_hits+=1
            fov=math.degrees(2*math.atan(math.tan(math.asin(min(.999,radius/d)))/usable)) if d<distance else 42
            views.append({'maximum_distance':limit,'distance':d,'eye_blender':list(eye),'vertical_fov':fov,
                'center_obstructed':hit[0] is not None,'rotated_bounds_corner_obstructions':corner_hits,
                'hit_blender':list(hit[0]) if hit[0] is not None else None})
        results.append({'choice':choice['id'],'viewport':[w,h],'fitted_half_blender':list(half),'views':views})
(OUT/'camera-obstruction-audit.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
for r in results:
    print(r['choice'],r['viewport'],[(v['distance'],round(v['vertical_fov'],2),v['center_obstructed']) for v in r['views']])
print('CURRENT_OBSTRUCTIONS',sum(r['views'][0]['center_obstructed'] for r in results))
for i,limit in enumerate([None,17.0,16.0,15.0,14.0,13.0,12.0,11.0]):
    print('LIMIT',limit,'CENTER_OBSTRUCTIONS',sum(r['views'][i]['center_obstructed'] for r in results),
        'BOUND_CORNERS',sum(r['views'][i]['rotated_bounds_corner_obstructions'] for r in results))
