"""Capturas de revision: no guarda ni exporta el modelo ni cambia materiales."""
from pathlib import Path
import bpy
from mathutils import Vector, Matrix

repo=Path(__file__).resolve().parents[3]
folder=repo/'docs/references/southpark'
scene=bpy.context.scene
root=bpy.data.objects['SouthPark_Planet_ROOT']
assert root.get('iteration')==3
camera=scene.camera
original=camera.matrix_world.copy();ortho=camera.data.ortho_scale
light=bpy.data.objects['SouthPark_Front']
light_matrix=light.matrix_world.copy();power=light.data.energy;size=light.data.size
resolution=(scene.render.resolution_x,scene.render.resolution_y)
filepath=scene.render.filepath
try:
    scene.render.resolution_x=1400;scene.render.resolution_y=1400
    scene.render.filepath=str(folder/'southpark_v03_three_quarter.png')
    bpy.ops.render.render(write_still=True)
    scene.render.resolution_x=1200;scene.render.resolution_y=1200
    for name,direction in [('front',(0,-1,.03)),('back',(0,1,.03)),
                           ('east',(1,0,.03)),('west',(-1,0,.03)),('top',(0,0,1))]:
        camera.location=Vector(direction).normalized()*95
        camera.rotation_euler=(-camera.location).to_track_quat('-Z','Y').to_euler()
        camera.data.ortho_scale=65
        scene.render.filepath=str(folder/('southpark_v03_'+name+'.png'))
        bpy.ops.render.render(write_still=True)
    details=[('school','SP_B063_School',(0,0,1.9),(5,-13,7.5),12),
             ('civic','SP_B066_Civic',(0,-.2,2.2),(4,-11,7),8),
             ('commercial','SP_B046_Shop',(0,0,1.35),(3,-11,5),9),
             ('pagoda','SP_B068_Pagoda',(0,0,3.2),(4,-10,7),9),
             ('sign','SouthPark_Sign',(0,0,2.5),(0,-8,5.6),7)]
    for name,object_name,target_local,eye_local,scale in details:
        obj=bpy.data.objects[object_name]
        target=obj.matrix_world@Vector(target_local)
        eye=obj.matrix_world@Vector(eye_local)
        direction=(target-eye).normalized()
        up_ref=obj.matrix_world.to_3x3().col[2].normalized()
        right=direction.cross(up_ref).normalized();up=right.cross(direction)
        matrix=Matrix(((right.x,up.x,-direction.x),(right.y,up.y,-direction.y),
                       (right.z,up.z,-direction.z))).to_4x4()
        matrix.translation=eye
        camera.matrix_world=matrix;camera.data.ortho_scale=scale
        # Luz tecnica temporal para leer fachadas; no se hornea ni queda aplicada.
        light.location=obj.matrix_world@Vector((0,-10,13))
        light.rotation_euler=(target-light.location).to_track_quat('-Z','Y').to_euler()
        light.data.energy=3800;light.data.size=10
        scene.render.filepath=str(folder/('southpark_v03_'+name+'_detail.png'))
        bpy.ops.render.render(write_still=True)
finally:
    camera.matrix_world=original;camera.data.ortho_scale=ortho
    light.matrix_world=light_matrix;light.data.energy=power;light.data.size=size
    scene.render.resolution_x,scene.render.resolution_y=resolution
    scene.render.filepath=filepath
print('Capturas v3 guardadas; camara y luces de presentacion restauradas.')
