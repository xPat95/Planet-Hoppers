"""Export existing character .blend files with their packed materials intact.

Run: blender -b --python tools/blender/export_abduction_characters.py -- <repo>
Authoring files are never saved or changed.
"""
import sys
from pathlib import Path
import bpy

root = Path(sys.argv[sys.argv.index('--') + 1]).resolve()
names = ['Steve', 'Stan', 'Kyle', 'Kenny', 'Cartman', 'Pim', 'Charlie',
         'Ghost_Face', 'Mordecai', 'Rigby', 'Grogu']
out = root / 'assets/models/characters/abductions'
out.mkdir(parents=True, exist_ok=True)
for name in names:
    bpy.ops.wm.open_mainfile(filepath=str(root / f'assets/models/characters/{name}/{name}.blend'))
    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH' and not o.hide_render]
    if not meshes:
        raise RuntimeError(f'No visible meshes: {name}')
    bpy.ops.object.select_all(action='DESELECT')
    for obj in meshes:
        obj.hide_set(False)
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.export_scene.gltf(filepath=str(out / f'{name}.glb'), export_format='GLB',
                              use_selection=True, export_animations=False,
                              export_cameras=False, export_lights=False)
    print('ABDUCTION_EXPORT', name, len(meshes), flush=True)
