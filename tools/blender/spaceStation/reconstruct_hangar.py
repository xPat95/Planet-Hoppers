"""Reconstruct an independent interior from actual original meshes; never integrate game."""
from pathlib import Path
import bpy, bmesh, math, json, hashlib
from mathutils import Vector
from math import sin, cos, pi, tan, atan
ROOT=Path(__file__).resolve().parents[3]
BASE=ROOT/'assets/models/environment/spaceStation'
OUT=ROOT/'docs/references/hangar/reconstruction'
OUT.mkdir(parents=True,exist_ok=True)
SOURCE=BASE/'citadel_space_station.blend'
TARGET=BASE/'citadel_hangar_reconstructed.blend'
source_hash=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
bpy.ops.wm.open_mainfile(filepath=str(SOURCE))
original=bpy.context.scene
source_world=original.world.copy()
original.render.resolution_x=1440;original.render.resolution_y=960;original.render.resolution_percentage=100
original.render.image_settings.file_format='PNG'
original.render.engine='CYCLES';original.cycles.samples=32;original.cycles.use_denoising=True
try:
 prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.compute_device_type='OPTIX';prefs.get_devices()
 for device in prefs.devices:device.use=device.type=='OPTIX'
 original.cycles.device='GPU'
except Exception:pass
def render(s,c,n):
 s.camera=c;s.render.filepath=str(OUT/n);bpy.ops.render.render(write_still=True);print('RENDERED',n,flush=True)
source={}
for o in original.objects:
 if o.type=='MESH':
  source[o.name]={'v':[tuple(v.co) for v in o.data.vertices],'f':[tuple(p.vertices) for p in o.data.polygons],
  'mi':[p.material_index for p in o.data.polygons],'norm':[tuple(p.normal) for p in o.data.polygons],
  'mat':list(o.data.materials)}
def matinfo(m):
 n=next((n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED'),None)
 return {i.name:list(i.default_value) if hasattr(i.default_value,'__len__') else i.default_value for i in n.inputs
 if i.name in ['Base Color','Metallic','Roughness','Emission Color','Emission Strength']} if n else {}
report={'source':str(SOURCE.relative_to(ROOT)),'sha256':source_hash,
 'objects':[{'name':o.name,'type':o.type,'dimensions':list(o.dimensions),
 'vertices':len(o.data.vertices) if o.type=='MESH' else None,
 'polygons':len(o.data.polygons) if o.type=='MESH' else None,
 'materials':[m.name for m in o.data.materials if m] if o.type=='MESH' else []} for o in original.objects],
 'materials':{m.name:matinfo(m) for m in bpy.data.materials if m.use_nodes}}
(OUT/'source-inspection.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
for cam,name in [('CI CAM | Principal','01-original-general.png'),('CI CAM | Anillo frontal','02-original-presidium.png'),
 ('CI CAM | Distritos interiores','03-original-districts.png')]:render(original,bpy.data.objects[cam],name)
scene=bpy.data.scenes.new('HANGAR | Presidium reconstructed');bpy.context.window.scene=scene
def collection(name):
 c=bpy.data.collections.new(name);scene.collection.children.link(c);return c
direct=collection('01 | Componentes originales reutilizados')
adapted=collection('02 | Componentes adaptados - arquitectura principal')
connect=collection('03 | Conexiones y suelo nuevo')
mechanical=collection('04 | Detalles mecanicos originales')
references=collection('90 | NO EXPORTAR - plataforma y nave')
lighting=collection('91 | NO EXPORTAR - iluminacion de presentacion')
cameras=collection('92 | NO EXPORTAR - camaras')
for c in [references,lighting,cameras]:c['exportable']=False
mats={m.name.removeprefix('CI | '):m for m in bpy.data.materials if m.name.startswith('CI | ')}
provenance=[]
def make(name,verts,faces,materials,indices=None,col=connect):
 me=bpy.data.meshes.new(name);me.from_pydata(verts,[],faces);me.update()
 for m in materials:me.materials.append(m)
 if indices:
  for p,i in zip(me.polygons,indices):p.material_index=i
 o=bpy.data.objects.new(name,me);col.objects.link(o);return o
def reuse(name,src,transform,predicate=None,col=adapted,description='',cap=False):
 d=source[src];chosen=[]
 for i,f in enumerate(d['f']):
  v=sum((Vector(d['v'][j]) for j in f),Vector())/len(f)
  if predicate is None or predicate(v,i,d):chosen.append(i)
 ids=sorted(set(j for i in chosen for j in d['f'][i]));mapping={j:k for k,j in enumerate(ids)}
 o=make(name,[transform(Vector(d['v'][j])) for j in ids],
 [tuple(mapping[j] for j in d['f'][i]) for i in chosen],d['mat'],[d['mi'][i] for i in chosen],col)
 if cap:
  bm=bmesh.new();bm.from_mesh(o.data);edges=[e for e in bm.edges if e.is_boundary]
  if edges:bmesh.ops.holes_fill(bm,edges=edges,sides=0)
  bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(o.data);bm.free()
 o['source_object']=src;o['source_file']=str(SOURCE.relative_to(ROOT));o['adaptation']=description;o['exportable']=True
 provenance.append({'object':name,'source':src,'mode':'directa' if predicate is None and col==direct else 'adaptada / subconjunto real',
 'description':description,'source_vertex_indices':ids,'source_polygon_indices':chosen,
 'source_vertex_count':len(d['v']),'reused_vertices':len(ids),'reused_polygons':len(chosen)})
 return o
def closed(v,i,d):return not 55<math.degrees(math.atan2(v.z,v.y))%360<125
def horizontal(v):return (v.y*.64,v.z*.64,v.x*.64+4.58)
for src,label in [('CI | Anillo estructural octogonal','Perfil portante'),
 ('CI | Segmentos de blindaje del Presidium','Blindaje radial'),('CI | Aros nervios y cinco acoplamientos','Nervios y acoplamientos')]:
 reuse('PRESIDIUM | '+label,src,horizontal,lambda v,i,d:closed(v,i,d) and (src!='CI | Aros nervios y cinco acoplamientos' or all(j<6644 for j in d['f'][i])),adapted,
 'Giro X->Z Y->X Z->Y, escala uniforme 0.64; Z +4.58; retirar sector 55..125 grados para salida. En Aros: excluir antena central de vertices 6644..6659 asociada a la torre omitida.',cap='Anillo estructural' in src)
reuse('GALERIA | Bloques del distrito','CI | Ciudad interior y Torre del Consejo',horizontal,
 lambda v,i,d:closed(v,i,d) and math.hypot(v.y,v.z)>26,mechanical,
 'Mismo giro y escala 0.64 del tambor; excluir torre central (radio<=26). Bloques urbanos readaptados como volumen de servicio.')
reuse('GALERIA | Luminarias originales','CI | Luces del Presidium',horizontal,
 lambda v,i,d:closed(v,i,d) and math.hypot(v.y,v.z)>30.8,mechanical,
 'Transformacion del tambor; conservar luminarias de radio local>30.8.')
def portal(v):return (v.y*.30,19-v.x*.30,9+v.z*.30)
for src,label in [('CI | Anillo estructural octogonal','Perfil escalonado'),('CI | Segmentos de blindaje del Presidium','Escamas'),
 ('CI | Aros nervios y cinco acoplamientos','Aros y cinco acoplamientos')]:
 reuse('SALIDA | '+label,src,portal,(lambda v,i,d:all(j<6644 for j in d['f'][i])) if src=='CI | Aros nervios y cinco acoplamientos' else None,direct,'Giro X->-Y Y->X Z->Z; escala uniforme 0.30; centro (0,19,9). Anillo y blindaje completos; Aros excluye antena central 6644..6659, fuera del marco portante.')
reuse('SALIDA | Luminarias originales','CI | Luces del Presidium',portal,
 lambda v,i,d:math.hypot(v.y,v.z)>31.5,mechanical,'Transformacion de salida, conservar luminarias de radio local>31.5.')
# Octagonal transition surround closes the gaps outside the real circular portal.
# This is authored connection geometry, not claimed as a reused original mesh.
boundary=[(-20,0),(20,0),(23,4),(23,14),(12,22),(-12,22),(-23,14),(-23,4)]
def outerradius(a):
 dx,dz=cos(a),sin(a);distances=[]
 for p,q in zip(boundary,boundary[1:]+boundary[:1]):
  px,pz=p[0],p[1]-9;ex,ez=q[0]-p[0],q[1]-p[1]
  det=dx*(-ez)-(-ex)*dz
  if abs(det)<1e-9:continue
  t=(px*(-ez)+ex*pz)/det;u=(dx*pz-px*dz)/det
  if t>0 and -.0001<=u<=1.0001:distances.append(t)
 return min(distances)
verts=[];faces=[]
for i in range(121):
 a=2*pi*i/120;outer=outerradius(a)
 for r in [8.9,max(8.9,outer)]:verts.append((r*cos(a),21,9+r*sin(a)))
for i in range(120):faces.append((2*i,2*i+2,2*i+3,2*i+1))
o=make('SALIDA NUEVA | Transicion octogonal perforada',verts,faces,[mats['Titanio azulado']])
o.modifiers.new('Espesor de transicion','SOLIDIFY').thickness=.24
for src,label in [('CI | Anillo estructural octogonal','Segundo perfil'),('CI | Segmentos de blindaje del Presidium','Blindaje exterior'),('CI | Aros nervios y cinco acoplamientos','Soportes del blindaje exterior')]:
 reuse('SALIDA EXTERIOR | '+label,src,lambda v:(v.y*.395,23-v.x*.395,9+v.z*.395),
 (lambda v,i,d:all(j<6644 for j in d['f'][i])) if src=='CI | Aros nervios y cinco acoplamientos' else None,direct,'Escala uniforme 0.395; giro X->-Y Y->X Z->Z; centro (0,23,9). Segundo estrato con soportes originales de las tapas de blindaje. Aros excluye antena central 6644..6659.')

def roof(v):return (v.y*1.15,-(v.x+27)*1.15,v.z*1.15+11)
o=reuse('CUBIERTA | Casco facetado Zakera','CI 1 | Casco facetado continuo',roof,
 lambda v,i,d:-44<=v.x<=-10 and d['norm'][i][2]>-.05,adapted,
 'Seccion local X -44..-10, escala uniforme 1.15, giro al eje Y, Z+11. Retirar caras inferiores para ahuecar el casco conservando su perfil superior.')
o.modifiers.new('Espesor interior adaptado','SOLIDIFY').thickness=.12
for src,label in [('CI 1 | Placas y rejillas exteriores','Placas y rejillas')]:
 reuse('CUBIERTA | '+label,src,roof,lambda v,i,d:-44<=v.x<=-10,adapted,
 'Recorte longitudinal X -44..-10; escala uniforme 1.15; conservar las placas reales del casco.')
 if label=='Placas y rejillas':
  for v in bpy.data.objects['CUBIERTA | '+label].data.vertices:v.co.z-=.65
  provenance[-1]['description']+=' Placas desplazadas Z -0.65 para revestir cara interior conservando sus formas.'
for yy in [-11,-3,5,13]:
 reuse('CUBIERTA | Costilla original Y '+str(yy),'CI 1 | Costillas estructurales',
 lambda v,yy=yy:(v.y*1.25,-(v.x+27.25)*1.25+yy,v.z*1.25+7.2),
 lambda v,i,d:all(42<=j<84 for j in d['f'][i]),adapted,
 'Copiar componente conectado 42..83 de Zakera (costilla X=-27.25); escala uniforme 1.25, centrada en Y='+str(yy)+', Z+7.2. Extremos anclados al tambor.')
for side in [-1,1]:
 src='CI 1 | Escudo de raiz '+('-1' if side<0 else '+1')
 o=reuse('HOMBRO | Escudo '+str(side),src,
 lambda v,side=side:(v.y*.8+side*6,-(v.x+22)*.8,v.z*.8+7),None,direct,
 'Escala uniforme 0.8; giro al eje Y; X desplazado '+str(side*6)+', Z+7. Escudo completo entre cubierta y galeria.')
 o.modifiers.new('Espesor de union adaptado','SOLIDIFY').thickness=.18
for s,label in [(.22,'Interior'),(.39,'Exterior')]:
 reuse('SUELO | Perfil '+label,'CI | Anillo estructural octogonal',
 lambda v,s=s:(v.y*s,v.z*s,v.x*s+.55-13.4*s),None,direct,
 'Giro horizontal, escala uniforme '+str(s)+'; Z ajustado para labio superior a 0.55. Perfil original enterrado.')
def sector(name,r0,r1,a,b,z,depth,mat,steps=1):
 vs=[];fs=[]
 for i in range(steps+1):
  t=a+(b-a)*i/steps
  for zz in [z-depth,z]:
   for r in [r0,r1]:vs.append((r*cos(t),r*sin(t),zz))
 for i in range(steps):
  j=4*i;k=j+4;fs.extend([(j,j+1,k+1,k),(j+2,k+2,k+3,j+3),(j,k,k+2,j+2),(j+1,j+3,k+3,k+1)])
 fs.extend([(0,2,3,1),(4*steps,4*steps+1,4*steps+3,4*steps+2)])
 return make(name,vs,fs,[mats[mat]])
for i in range(48):
 a=i*2*pi/48+.002;b=(i+1)*2*pi/48-.002
 sector('SUELO NUEVO | Dovela %02d'%i,4.85,25.5,a,b,.12,.4,'Placas acero' if i%3 else 'Titanio azulado',2)
 for r0,r1 in [(5.25,6.35),(9.05,10.7),(15.8,18.25)]:
  sector('SUELO NUEVO | Placa %02d %.1f'%(i,r0),r0,r1,a+.009,b-.009,.17,.045,
  'Blindaje grafito' if i%4 else 'Placas plata',2)
reuse('SUELO | Collar tubular original','CI | Aros nervios y cinco acoplamientos',
 lambda v:(v.y*.133,v.z*.133,.38+(v.x+6.8)*.133),
 lambda v,i,d:all(960<=j<2286 for j in d['f'][i]),mechanical,
 'Extraer exclusivamente componente conectado de vertices 960..2285 (aro X=-6.8); escala uniforme 0.133; radio aproximado 5.05, Z=0.38.')
for i in range(12):
 a=i*2*pi/12;b=(i+1)*2*pi/12
 if 55<math.degrees((a+b)/2)<125:continue
 sector('CUBIERTA NUEVA | Conexion %02d'%i,16.8,23.3,a,b,11.5,.22,'Blindaje grafito',8)
for i in range(12):
 a=i*2*pi/12;sector('GUIA NUEVA | Radial %02d'%i,5.45,5.85,a-.008,a+.008,.22,.025,'Luz marfil')
def box(name,c,d,mat,col=connect):
 x,y,z=c;u,v,w=[q/2 for q in d]
 vs=[(x+a*u,y+b*v,z+e*w) for e in [-1,1] for b in [-1,1] for a in [-1,1]]
 return make(name,vs,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],[mats[mat]],col=col)
box('SALIDA NUEVA | Umbral',(0,18.8,.04),(9,10,.25),'Placas acero')
for x in [-4.2,4.2]:box('SALIDA NUEVA | Guia',(x,18.8,.2),(.055,9,.04),'Luz hielo')
def cylinder(name,r,depth,z,mat):
 bpy.ops.mesh.primitive_cylinder_add(vertices=96,radius=r,depth=depth,location=(0,0,z))
 o=bpy.context.object;o.name=name
 for c in list(o.users_collection):c.objects.unlink(o)
 references.objects.link(o);o.data.materials.append(mats[mat]);o['exportable']=False;return o
cylinder('NO EXPORTAR | Plataforma base Rust',4.7,.72,.36,'Titanio azulado')
cylinder('NO EXPORTAR | Plataforma borde Rust',4.54,.04,.74,'Luz cian')
cylinder('NO EXPORTAR | Plataforma tapa Rust',4.35,.04,.78,'Juntas oscuras')
cfg=json.loads((ROOT/'config/ships.json').read_text(encoding='utf-8'))
ship=next(s for s in cfg['ships'] if s['id']=='starwars_mandalorian')
ship_path=ROOT/ship['model'];ship_hash=hashlib.sha256(ship_path.read_bytes()).hexdigest()
old=set(bpy.data.objects);bpy.ops.import_scene.gltf(filepath=str(ship_path))
shipobjects=[o for o in bpy.data.objects if o not in old];bpy.context.view_layer.update()
shipmeshes=[o for o in shipobjects if o.type=='MESH']
pts=[o.matrix_world@Vector(v) for o in shipmeshes for v in o.bound_box]
lo=Vector(tuple(min(v[i] for v in pts) for i in range(3)));hi=Vector(tuple(max(v[i] for v in pts) for i in range(3)))
center=(lo+hi)/2;half=(hi-lo)/2;scale=6/(2*max(half));half*=scale
for o in shipmeshes:
 world=o.matrix_world.copy();o.parent=None;o.matrix_world=world
 o.location=(o.location-center)*scale+Vector((0,0,.8+half.z));o.scale*=scale
for o in shipobjects:
 for c in list(o.users_collection):c.objects.unlink(o)
 references.objects.link(o);o.name='NO EXPORTAR | Nave '+o.name;o['exportable']=False
env=cylinder('NO EXPORTAR | Reserva radio 4.8 altura 6.9',4.8,6.9,3.45,'Marcas de panel')
env.hide_render=True;env.hide_set(True);env.display_type='WIRE'
world=source_world;world.name='Espacio original | copia de presentacion';scene.world=world
def lamp(name,loc,target,power,size,color):
 d=bpy.data.lights.new(name,'AREA');d.energy=power;d.shape='DISK';d.size=size;d.color=color
 o=bpy.data.objects.new(name,d);lighting.objects.link(o);o.location=loc
 o.rotation_euler=(Vector(target)-o.location).to_track_quat('-Z','Y').to_euler()
for y in [-11,0,11]:lamp('Cubierta '+str(y),(0,y,13),(0,y,0),2300,10,(.78,.87,1))
lamp('Clave nave Rust',(-4,-5,9),(0,0,1.8),1900,7,(.86,.92,1))
for x in [-13,13]:
 lamp('Rebote galeria '+str(x),(x,-1,8),(0,2,4),2400,9,(.78,.86,1))
 lamp('Hombros '+str(x),(x,11,9),(x,3,10),1500,7,(1,.80,.56))
lamp('Entrada espacial',(0,23,10),(0,0,4),3000,11,(.65,.78,1))
def camera(name,eye,target,fov):
 d=bpy.data.cameras.new(name);d.sensor_fit='VERTICAL';d.sensor_height=24;d.lens=24/(2*tan(fov*pi/360));d.clip_end=2000
 o=bpy.data.objects.new(name,d);cameras.objects.link(o);o.location=eye
 o.rotation_euler=(Vector(target)-o.location).to_track_quat('-Z','Y').to_euler();return o
angle=min(atan(tan(21*pi/180)*493/800),atan(tan(21*pi/180)*1.6*1100/1280))
distance=max(11,half.length/sin(angle)*1.08)
target=Vector((0,0,.8+half.z));eye=target+Vector((.9,-1.4,.6)).normalized()*distance
cams=[camera('01 | Camara real Rust N1 1280x800',eye,target,42),
 camera('02 | Interior general',(12,-14,8),(0,7,6),68),
 camera('03 | Galeria lateral',(-5,-5,5),(-17,5,6),61),
 camera('04 | Cubierta Zakera',(0,-8,6),(0,4,13),76),
 camera('05 | Salida espacial',(0,-2,5),(0,20,9),67),
 camera('06 | Planta arquitectura',(27,-30,34),(0,2,3),65)]
scene.render.engine='CYCLES';scene.cycles.samples=64;scene.cycles.use_denoising=True;scene.cycles.device=original.cycles.device
scene.render.resolution_x=1280;scene.render.resolution_y=800;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX';scene.camera=cams[0]
for s in list(bpy.data.scenes):
 if s!=scene:bpy.data.scenes.remove(s)
for o in list(bpy.data.objects):
 if o.name not in scene.objects:bpy.data.objects.remove(o,do_unlink=True)
bpy.ops.file.pack_all();bpy.context.view_layer.update()
(OUT/'reuse-provenance.json').write_text(json.dumps(provenance,ensure_ascii=False,indent=2),encoding='utf-8')
arch=[o for c in [direct,adapted,connect,mechanical] for o in c.objects if o.type=='MESH']
for o in arch:o['exportable']=True
from mathutils.bvhtree import BVHTree
deps=bpy.context.evaluated_depsgraph_get();envelope=BVHTree.FromObject(env,deps)
overlaps=[];inside=[]
for o in arch:
 if BVHTree.FromObject(o,deps).overlap(envelope):overlaps.append(o.name)
 if any(math.hypot(v.co.x,v.co.y)<4.8 and .001<v.co.z<6.9 for v in o.data.vertices):inside.append(o.name)
manifest={'source_sha256':source_hash,'source_unchanged':hashlib.sha256(SOURCE.read_bytes()).hexdigest()==source_hash,
 'ship_unchanged':hashlib.sha256(ship_path.read_bytes()).hexdigest()==ship_hash,
 'architecture_objects':len(arch),'architecture_polygons':sum(len(o.data.polygons) for o in arch),
 'reused_objects':len(provenance),'reused_source_polygons':sum(p['reused_polygons'] for p in provenance),
 'platform':{'center_game':[0,0,0],'radius':4.5,'base_max_radius':4.7,'top':.8,'no_export':True},
 'clearance':{'conservative_radius':4.8,'height':6.9,'surface_intersections':overlaps,'interior_vertices':inside,
 'ship_bound':'Fit::Length(6): half extents <=3, yaw radius<=sqrt(18), top<=6.8'},
 'camera':{'viewport':[1280,800],'hud':[151,156],'eye_blender':list(eye),'target_blender':list(target),
 'vertical_fov':42,'distance':distance,'ship':ship['id'],'half_extents_blender':list(half)},
 'external_libraries':[l.filepath for l in bpy.data.libraries],
 'images':[{'name':i.name,'path':i.filepath,'packed':i.packed_file is not None} for i in bpy.data.images if i.source=='FILE'],
 'export_collections':[c.name for c in [direct,adapted,connect,mechanical]],
 'non_export_collections':[c.name for c in [references,lighting,cameras]],
 'notes':['Area lights and AgX are presentation only, not verified in raylib.',
 'No GLB exported; future export must select architecture only.',
 'Changed architectural dimensions need future dock review; Rust and config untouched.']}
(OUT/'build-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
scene['review_status']='Art review only; not integrated';scene['coordinate_conversion']='Game(x,y,z)->Blender(x,-z,y)'
scene['source_sha256']=source_hash;bpy.context.preferences.filepaths.save_version=0
bpy.ops.wm.save_as_mainfile(filepath=str(TARGET));print('MODEL_SAVED',str(TARGET),flush=True)
for cam,name in zip(cams,['04-game-camera.png','05-interior-general.png','06-lateral.png','07-roof.png','08-exit.png','09-architecture.png']):
 render(scene,cam,name)
scene.camera=cams[0];bpy.ops.wm.save_as_mainfile(filepath=str(TARGET))
print('RECONSTRUCTION_RENDER_COMPLETE',flush=True)
