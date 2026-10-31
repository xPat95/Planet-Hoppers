"""Reopen the editable file and reimport GLBs in disposable scenes; never save.
Writes only target/phase4b-assets-validation.json.
"""
import bpy
import json
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'assets/models/minigames/southpark/threats/phase4b'
audit = json.loads((OUT/'asset-audit.json').read_text(encoding='utf-8'))
bpy.ops.wm.open_mainfile(filepath=str(OUT/'southpark_threats_4b.blend'))
scenes = {'piano': 'Grand_Piano', 'piano_support': 'Piano_Support',
          'piano_lock': 'Piano_Safety_Lock', 'snowman': 'Killer_Snowman', 'heater': 'Heater'}
assert len(bpy.data.scenes) == 5, 'Unexpected editable scenes'
results = {}
for name, suffix in scenes.items():
    scene = bpy.data.scenes['SouthPark_4B_'+suffix]
    objects = [o for o in scene.objects if o.type == 'MESH']
    assert len(objects) == audit[name]['objects'], 'Editable objects missing: '+name
    assert all(o.data.materials and all(m.use_nodes for m in o.data.materials) for o in objects)
    temp = bpy.data.scenes.new('Validation_'+name)
    bpy.context.window.scene = temp
    bpy.ops.import_scene.gltf(filepath=str(OUT/(name+'.glb')))
    imported = [o for o in temp.objects if o.type == 'MESH']
    points = [o.matrix_world@Vector(v) for o in imported for v in o.bound_box]
    assert len(imported) == len(objects), 'GLB mesh count mismatch: '+name
    error = max(abs(f(p[i] for p in points)-audit[name]['blender_bounds'][j][i])
                for j,f in enumerate((min,max)) for i in range(3))
    assert error < .002, 'GLB scale/orientation mismatch: '+name
    triangles = 0
    for o in imported:
        o.data.calc_loop_triangles()
        triangles += len(o.data.loop_triangles)
    assert triangles == audit[name]['triangles'], 'GLB geometry mismatch: '+name
    results[name] = {'editable_objects':len(objects),'triangles':triangles,'max_bounds_error_m':error}
report = {'status':'PASS','assets':results,'models_saved':False}
(ROOT/'target/phase4b-assets-validation.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print('PHASE4B_VALIDATION',json.dumps(report))
