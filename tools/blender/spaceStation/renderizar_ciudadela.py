"""Renderiza vistas de revision desde el .blend, sin alterar el archivo fuente.
Uso: blender -b assets/models/environment/spaceStation/citadel_space_station.blend -P tools/blender/spaceStation/renderizar_ciudadela.py -- --draft
"""
from pathlib import Path
import sys
import bpy

HERE=Path(__file__).resolve().parents[3]/'docs/references/spaceStation'
s=bpy.data.scenes['Ciudadela_Mass_Effect']
bpy.context.window.scene=s
draft='--draft' in sys.argv
views=[('CI CAM | Principal','Ciudadela_Principal.png')]
if not draft:
    views += [('CI CAM | Anillo frontal','Ciudadela_Anillo.png'),
              ('CI CAM | Perfil','Ciudadela_Perfil.png'),
              ('CI CAM | Distritos interiores','Ciudadela_Distritos.png')]
s.render.resolution_percentage=60 if draft else 100
s.render.threads_mode=next(i.identifier for i in s.render.bl_rna.properties['threads_mode'].enum_items if i.identifier=='FIXED')
s.render.threads=8
for camera,filename in views:
    s.camera=bpy.data.objects[camera]
    s.render.filepath=str(HERE/('Borrador_'+filename if draft else filename))
    print('RENDER_START '+filename,flush=True)
    bpy.ops.render.render(write_still=True)
    print('RENDER_DONE '+filename,flush=True)
print('CIUDADELA_RENDER_COMPLETE',flush=True)
