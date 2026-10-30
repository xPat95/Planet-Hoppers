"""Render three review views from the saved source; never resaves source."""
from pathlib import Path
import bpy
from mathutils import Vector
REPO=Path(__file__).resolve().parents[5]
REF=REPO/'docs/references/southpark/ships/butters'
scene=bpy.data.scenes['SouthPark_Butters_Ship']; bpy.context.window.scene=scene
scene.cycles.samples=32
cam=scene.camera
views=[
 ('three_quarter',(7,-9.35,4.35),(0,-.05,0),7.6),
 ('side',(10,0,.12),(0,0,.12),6.7),
 ('front',(0,-12,.12),(0,-.35,.12),6.3),
 ('rear_three_quarter',(6,9,3.8),(0,.10,0),7.6),
]
for name,pos,target,scale in views:
    bpy.data.objects['BUTTERS | Studio floor'].hide_render=name in {'side','front'}
    cam.location=pos; cam.rotation_euler=(Vector(target)-cam.location).to_track_quat('-Z','Y').to_euler()
    cam.data.ortho_scale=scale
    scene.render.filepath=str(REF/('butters_ship_'+name+'.png'))
    bpy.ops.render.render(write_still=True,scene=scene.name)
    print('RENDERED',name,flush=True)
