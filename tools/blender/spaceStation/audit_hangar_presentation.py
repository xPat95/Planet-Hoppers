"""Read-only geometric and HUD visibility audit of the approved general camera."""
from pathlib import Path
import bpy,json,math
from mathutils import Vector
from mathutils.bvhtree import BVHTree

ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'docs/references/hangar/presentation'
bpy.ops.wm.open_mainfile(filepath=str(ROOT/'assets/models/environment/spaceStation/citadel_hangar_reconstructed.blend'))
verts=[];faces=[];deps=bpy.context.evaluated_depsgraph_get()
for o in bpy.context.scene.objects:
    if o.type!='MESH' or o.get('exportable')!=True:continue
    ev=o.evaluated_get(deps);me=ev.to_mesh();offset=len(verts)
    verts.extend(o.matrix_world@v.co for v in me.vertices)
    faces.extend(tuple(offset+j for j in p.vertices) for p in me.polygons);ev.to_mesh_clear()
tree=BVHTree.FromPolygons(verts,faces)
cfg=json.loads((ROOT/'config/ships.json').read_text(encoding='utf-8'))
choices=json.loads((ROOT/'config/ship-selection.json').read_text(encoding='utf-8'))['choices']
results=[]
for choice in choices:
    ship=next((s for s in cfg['ships'] if s['id']==choice['ship_id']),None)
    if ship:
        temp=bpy.data.scenes.new('TEMP | ship bounds');bpy.context.window.scene=temp
        bpy.ops.import_scene.gltf(filepath=str(ROOT/ship['model']));bpy.context.view_layer.update()
        points=[o.matrix_world@Vector(v) for o in temp.objects if o.type=='MESH' for v in o.bound_box]
        lo=Vector(tuple(min(v[i] for v in points) for i in range(3)))
        hi=Vector(tuple(max(v[i] for v in points) for i in range(3)))
        half=(hi-lo)/2;half*=3/max(half);center=Vector((0,0,.8+half.z))
    else:
        half=Vector((2.3,3,.8));center=Vector((0,0,.8+.225*6/4))
    eye=Vector((12,-14,8));target=Vector((0,7,6))
    f=(target-eye).normalized();right=f.cross(Vector((0,0,1))).normalized();up=right.cross(f).normalized()
    for w,h in [(960,640),(1280,800),(1920,1080)]:
        top=max(.1,1-2*151/h);bottom=max(.1,1-2*156/h);side=max(.1,1-180/w)
        tangent=math.tan(math.radians(74/2));r=max(4.7,math.hypot(half.x,half.y))*1.08
        for step in range(16):
            a=step*math.tau/16
            for z in [-half.z,half.z]:
                d=center+Vector((r*math.cos(a),r*math.sin(a),z*1.08))-eye
                depth=max(.1,d.dot(f));v=d.dot(up)/depth;x=d.dot(right)/depth
                tangent=max(tangent,v/top if v>=0 else -v/bottom,abs(x)/(side*w/h))
        corner_hits=0;hud_errors=0
        for yaw in [step*math.tau/8 for step in range(8)]:
            for sx in [-1,1]:
                for sy in [-1,1]:
                    for sz in [-1,1]:
                        x,y=sx*half.x,sy*half.y
                        point=center+Vector((x*math.cos(yaw)-y*math.sin(yaw),x*math.sin(yaw)+y*math.cos(yaw),sz*half.z))
                        d=point-eye
                        if tree.ray_cast(eye,d.normalized(),d.length)[0] is not None:corner_hits+=1
                        depth=d.dot(f)
                        px=(1+d.dot(right)/(depth*tangent*w/h))*w/2
                        py=(1-d.dot(up)/(depth*tangent))*h/2
                        if not (90<=px<=w-90 and 151<=py<=h-156):hud_errors+=1
        d=center-eye;hit=tree.ray_cast(eye,d.normalized(),d.length)
        result={'ship':choice['id'],'viewport':[w,h],'eye_blender':list(eye),'target_blender':list(target),
            'fov_y':math.degrees(2*math.atan(tangent)),'center_obstructed':hit[0] is not None,
            'rotating_corner_obstructions':corner_hits,'hud_errors':hud_errors}
        results.append(result);print(result,flush=True)
(OUT/'camera-audit.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
assert len(results)==18
assert all(not r['center_obstructed'] and r['rotating_corner_obstructions']==0 and r['hud_errors']==0 for r in results)
print('PRESENTATION_CAMERA_PASS cases=18',flush=True)
