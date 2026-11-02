"""Export approved architecture with its current presentation lighting.

--geometry-only writes a diagnostic asset under target, never the runtime GLB.
The source Blender file is preserved byte for byte in both paths.
"""
from pathlib import Path
import bpy, json, hashlib, math, types, sys
from mathutils import Vector, Matrix

ROOT = Path(__file__).resolve().parents[3]
if '--geometry-only' not in sys.argv:
    import runpy
    runpy.run_path(str(Path(__file__).with_name('prepare_hangar_presentation.py')), run_name='__main__')
    raise SystemExit(0)
BASE = ROOT / 'assets/models/environment/spaceStation'
OUT = ROOT / 'target/hangar-geometry-diagnostic'
OUT.mkdir(parents=True, exist_ok=True)
SOURCE = BASE / 'citadel_hangar_reconstructed.blend'
TARGET = OUT / 'geometry-only.glb'
before = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
scene = bpy.context.scene
collections = [c for c in scene.collection.children if c.name[:2] in ['01', '02', '03', '04']]
objects = sorted({o for c in collections for o in c.all_objects if o.type == 'MESH'}, key=lambda o:o.name)
assert len(collections) == 4 and len(objects) == 241
assert all(o.get('exportable') == True and 'NO EXPORTAR' not in o.name for o in objects)
assert all(o.matrix_world == Matrix.Identity(4) for o in objects), 'Geometry must be authored in world coordinates'
bpy.ops.object.select_all(action='DESELECT')
for o in objects:
    o.hide_set(False)
    o.select_set(True)
bpy.context.view_layer.objects.active = objects[0]
# Some alpha Blender distributions omit this optional animation module.
try:
    from io_scene_gltf2.blender.exp.animation.animations import gather_animations
except ModuleNotFoundError:
    stub = types.ModuleType('io_scene_gltf2.blender.exp.animation.animations')
    stub.gather_animations = lambda *a, **k: []
    sys.modules[stub.__name__] = stub
bpy.ops.export_scene.gltf(filepath=str(TARGET), export_format='GLB', use_selection=True,
    export_apply=True, export_animations=False, export_cameras=False, export_lights=False,
    export_extras=False)
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == before
# Re-import in a fresh scene to check the actual exported asset, not just selection flags.
check = bpy.data.scenes.new('GLB export verification')
bpy.context.window.scene = check
bpy.ops.import_scene.gltf(filepath=str(TARGET))
bpy.context.view_layer.update()
meshes = [o for o in check.objects if o.type == 'MESH']
assert meshes and all('NO EXPORTAR' not in o.name for o in check.objects)
assert not any(o.type in ['LIGHT', 'CAMERA'] for o in check.objects)
points = [o.matrix_world @ Vector(v) for o in meshes for v in o.bound_box]
lo = [min(v[i] for v in points) for i in range(3)]
hi = [max(v[i] for v in points) for i in range(3)]
from mathutils.bvhtree import BVHTree
verts=[]; faces=[]
for o in meshes:
    offset=len(verts)
    verts.extend(o.matrix_world @ v.co for v in o.data.vertices)
    faces.extend(tuple(offset+j for j in p.vertices) for p in o.data.polygons)
tree=BVHTree.FromPolygons(verts,faces)
# Clearance cylinder includes the platform base and a conservative six-unit fitted ship.
bpy.ops.mesh.primitive_cylinder_add(vertices=192,radius=4.8,depth=6.9,location=(0,0,3.45))
env=bpy.context.object
envelope=BVHTree.FromObject(env,bpy.context.evaluated_depsgraph_get())
overlaps=tree.overlap(envelope)
inside=[tuple(v) for v in verts if math.hypot(v.x,v.y)<4.8 and .001<v.z<6.9]
assert not overlaps and not inside, 'Export intersects platform/rotating ship reserve'
report={'source':str(SOURCE.relative_to(ROOT)), 'source_sha256':before, 'source_unchanged':True,
    'export':str(TARGET.relative_to(ROOT)), 'export_sha256':hashlib.sha256(TARGET.read_bytes()).hexdigest(),
    'export_bytes':TARGET.stat().st_size, 'architecture_objects':len(objects), 'reimported_meshes':len(meshes),
    'collections':[c.name for c in collections], 'excluded':['platform proxy','reference ship','clearance envelope','lights','cameras','procedural world'],
    'bounds_blender':{'min':lo,'max':hi},
    'game_half_extents_from_origin':[max(abs(lo[0]),abs(hi[0])),max(abs(lo[2]),abs(hi[2])),max(abs(lo[1]),abs(hi[1]))],
    'units':'unchanged game units; Blender (x,y,z) -> glTF/raylib (x,z,-y)',
    'transform':'identity; no recentering or automatic fit',
    'clearance':{'radius':4.8,'height':6.9,'surface_intersections':len(overlaps),'interior_vertices':len(inside)},
    'presentation':'World shader, Cycles area lights and AgX are not exported. Runtime uses its existing sky and architecture shader.'}
(OUT/'export-manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('APPROVED_HANGAR_EXPORT_PASS',json.dumps(report,ensure_ascii=False),flush=True)
