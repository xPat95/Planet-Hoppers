"""Read and render the approved source without saving it."""
from pathlib import Path
import bpy, json, hashlib
ROOT = Path(__file__).resolve().parents[3]
source = ROOT/'assets/models/environment/spaceStation/citadel_space_station.blend'
out = ROOT/'docs/references/hangar'
out.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.open_mainfile(filepath=str(source))
s = bpy.data.scenes['Ciudadela_Mass_Effect']
bpy.context.window.scene = s
report = {'source': str(source.relative_to(ROOT)), 'sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
 'objects': [{'name': o.name, 'type': o.type, 'dimensions': list(o.dimensions),
              'materials': [m.name for m in o.data.materials if m] if o.type == 'MESH' else []}
             for o in s.objects],
 'materials': {m.name: {i.name: list(i.default_value) if hasattr(i.default_value, '__len__') else i.default_value
    for i in next(n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED').inputs
    if i.name in ['Base Color', 'Metallic', 'Roughness', 'Emission Color', 'Emission Strength']}
    for m in bpy.data.materials if m.use_nodes and any(n.type == 'BSDF_PRINCIPLED' for n in m.node_tree.nodes)}}
(out/'source-inspection.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
s.camera = bpy.data.objects['CI CAM | Anillo frontal']
s.render.resolution_percentage = 50
s.render.filepath = str(out/'reference-ring.png')
bpy.ops.render.render(write_still=True)
s.camera = bpy.data.objects['CI CAM | Distritos interiores']
s.render.filepath = str(out/'reference-districts.png')
bpy.ops.render.render(write_still=True)
print('REFERENCE_INSPECTED', flush=True)
