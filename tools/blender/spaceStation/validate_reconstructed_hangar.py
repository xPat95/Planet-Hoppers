"""Read-only validation of the saved reconstructed hangar; writes evidence only."""
from pathlib import Path
import bpy,json,hashlib,math,struct
from mathutils import Vector
from mathutils.bvhtree import BVHTree
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'docs/references/hangar/reconstruction'
TARGET=ROOT/'assets/models/environment/spaceStation/citadel_hangar_reconstructed.blend'
SOURCE=ROOT/'assets/models/environment/spaceStation/citadel_space_station.blend'
bpy.ops.wm.open_mainfile(filepath=str(TARGET))
s=bpy.context.scene;deps=bpy.context.evaluated_depsgraph_get()
manifest=json.loads((OUT/'build-manifest.json').read_text(encoding='utf-8'))
source_report=json.loads((OUT/'source-inspection.json').read_text(encoding='utf-8'))
provenance=json.loads((OUT/'reuse-provenance.json').read_text(encoding='utf-8'))
architecture=[o for o in s.objects if o.type=='MESH' and o.get('exportable')==True]
env=bpy.data.objects['NO EXPORTAR | Reserva radio 4.8 altura 6.9']
volume=BVHTree.FromObject(env,deps)
intersections=[];inside=[];verts=[];faces=[]
for o in architecture:
 if BVHTree.FromObject(o,deps).overlap(volume):intersections.append(o.name)
 if any(math.hypot(v.co.x,v.co.y)<4.8 and .001<v.co.z<6.9 for v in o.data.vertices):inside.append(o.name)
 ev=o.evaluated_get(deps);me=ev.to_mesh();offset=len(verts)
 verts.extend(o.matrix_world@v.co for v in me.vertices)
 faces.extend(tuple(offset+j for j in p.vertices) for p in me.polygons);ev.to_mesh_clear()
tree=BVHTree.FromPolygons(verts,faces)
cam=bpy.data.objects['01 | Camara real Rust N1 1280x800'];focus=Vector(manifest['camera']['target_blender'])
ray=focus-cam.location;hit=tree.ray_cast(cam.location,ray.normalized(),ray.length)
material_mismatches=[]
for name,expected in source_report['materials'].items():
 m=bpy.data.materials.get(name)
 if not m:continue
 n=next((n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED'),None)
 if n:
  for key,value in expected.items():
   actual=n.inputs[key].default_value
   if isinstance(value,list):
    if any(abs(a-b)>1e-6 for a,b in zip(actual,value)):material_mismatches.append([name,key])
   elif abs(actual-value)>1e-6:material_mismatches.append([name,key])
missing_images=[i.filepath for i in bpy.data.images if i.source=='FILE' and not i.packed_file and not Path(bpy.path.abspath(i.filepath)).exists()]
renders={}
for p in sorted(OUT.glob('*.png')):
 data=p.read_bytes();renders[p.name]={'bytes':len(data),'resolution':list(struct.unpack('>II',data[16:24])),'png_signature':data[:8]==b'\x89PNG\r\n\x1a\n'}
counts={o.name:len(o.data.vertices) for o in architecture}
provenance_errors=[]
for entry in provenance:
 o=bpy.data.objects.get(entry['object'])
 if not o or o.get('source_object')!=entry['source'] or len(o.data.vertices)!=entry['reused_vertices']:provenance_errors.append(entry['object'])
result={'reopened':bpy.data.filepath==str(TARGET),
 'source_unchanged':hashlib.sha256(SOURCE.read_bytes()).hexdigest()==manifest['source_sha256'],
 'external_libraries':[l.filepath for l in bpy.data.libraries],'missing_external_images':missing_images,
 'source_material_property_mismatches':material_mismatches,
 'platform_clearance_surface_intersections':intersections,'platform_clearance_interior_vertices':inside,
 'clearance_volume':{'radius':4.8,'height':6.9},
 'game_camera_center_ray_unobstructed':hit[0] is None,
 'game_camera_vertical_fov':math.degrees(2*math.atan(cam.data.sensor_height/(2*cam.data.lens))),
 'game_camera_eye':list(cam.location),'game_camera_target':list(focus),
 'reused_mesh_provenance_errors':provenance_errors,
 'original_mesh_object_names':sorted(set(e['source'] for e in provenance)),
 'architecture_objects':len(architecture),'architecture_faces':len(faces),'render_evidence':renders,
 'reference_only_objects':[o.name for o in s.objects if o.get('exportable')==False],
 'lighting_limit':'Area lights, procedural world, Cycles shadows and AgX are presentation only, not a verified raylib result.',
 'export_limit':'NO EXPORTAR/custom property is organizational metadata. A future GLB export must explicitly select architecture collections.',
 'visual_review_limit':'Artistic acceptance belongs to user. Visual inspection is not a watertight/collision proof of every architectural joint.'}
result['technical_pass']=all([result['reopened'],result['source_unchanged'],not result['external_libraries'],
 not missing_images,not material_mismatches,not intersections,not inside,not provenance_errors,
 result['game_camera_center_ray_unobstructed'],len(renders)>=9,all(i['png_signature'] for i in renders.values())])
(OUT/'validation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k not in ['reference_only_objects','render_evidence']},ensure_ascii=False,indent=2))
assert result['technical_pass'],'Inspect validation.json'
print('REOPEN_VALIDATION_PASS',flush=True)
