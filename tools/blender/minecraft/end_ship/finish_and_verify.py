"""Reopen the asset, correct rod atlas UVs, render review views, validate and save."""
from pathlib import Path
import bpy, json
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[4]
OUT=ROOT/'assets/models/ships/minecraft/end_ship.blend'
REF=ROOT/'docs/references/minecraft/end_ship'
bpy.ops.wm.open_mainfile(filepath=str(OUT))
scene=bpy.context.scene
for vert in bpy.data.objects['EndShip_Dragon_Head'].data.vertices:
    if abs(vert.co.y-13.85)<.0001:vert.co.y=13.5
mesh=bpy.data.objects['EndShip_End_Rods'].data
for poly in mesh.polygons:
    part=(poly.index//6)%2
    side=poly.index%6
    rect=([2,2,6,6] if side in (2,3) else [2,6,6,7]) if part==0 else ([2,0,4,2] if side in (2,3) else [0,0,2,15])
    u0,v0,u1,v1=rect
    uv=[(u0/16,1-v1/16),(u1/16,1-v1/16),(u1/16,1-v0/16),(u0/16,1-v0/16)]
    for loop,co in zip(poly.loop_indices,uv):mesh.uv_layers.active.data[loop].uv=co
next(n for n in scene.world.node_tree.nodes if n.type=='BACKGROUND').inputs[1].default_value=.8
cam=scene.camera
def aim(target):cam.rotation_euler=(Vector(target)-cam.location).to_track_quat('-Z','Y').to_euler()
views=[('three_quarter',(-36,43,29),(0,0,3),38),('side',(-40,0,3.5),(0,0,3.5),35),('top',(0,0,60),(0,0,0),39)]
for name,loc,target,scale in views:
    cam.location=loc;aim(target);cam.data.ortho_scale=scale
    scene.render.filepath=str(REF/('end_ship_'+name+'.png'))
    bpy.ops.render.render(write_still=True)
cam.location=views[0][1];aim(views[0][2]);cam.data.ortho_scale=views[0][3]
scene.render.filepath='//../../../../docs/references/minecraft/end_ship/end_ship_three_quarter.png'
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type=='VIEW_3D':area.spaces.active.region_3d.view_rotation=cam.rotation_euler.to_quaternion()
checks={'reopened_successfully':True,'packed_images':all(im.packed_file for im in bpy.data.images if im.type=='IMAGE' and im.source=='FILE'),'all_meshes_have_uv':all(o.data.uv_layers for o in scene.objects if o.type=='MESH'),'all_meshes_parented':all(o.parent and o.parent.name=='EndShip_ROOT' for o in scene.objects if o.type=='MESH'),'all_meshes_flat':all(not p.use_smooth for o in scene.objects if o.type=='MESH' for p in o.data.polygons),'no_modifiers':all(not o.modifiers for o in scene.objects if o.type=='MESH'),'no_animation':all(not o.animation_data for o in scene.objects),'no_missing_materials':all(o.data.materials and all(m for m in o.data.materials) for o in scene.objects if o.type=='MESH')}
assert all(checks.values()),checks
report=json.loads((REF/'validation.json').read_text())
report['checks']=checks
(REF/'validation.json').write_text(json.dumps(report,indent=2),encoding='utf8')
bpy.context.preferences.filepaths.save_version=0
bpy.ops.wm.save_as_mainfile(filepath=str(OUT))
print(json.dumps(checks,indent=2))
