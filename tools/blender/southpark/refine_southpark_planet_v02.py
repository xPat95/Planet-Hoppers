"""Refina South Park v1 en la escena existente; nunca reconstruye la esfera.

Requiere southpark_planet.blend abierto y copia previa en docs/references/southpark/
backups/. No guarda el .blend ni exporta automáticamente. No ejecutar sobre v2.
"""
import ast
import bpy
import math
import json
import random
import hashlib
import struct
from pathlib import Path
from mathutils import Vector, Matrix

REPO = Path(__file__).resolve().parents[3]
DOCS = REPO / 'docs/references/southpark'
RADIUS = 22.0
root = bpy.data.objects.get('SouthPark_Planet_ROOT')
if not root or root.get('iteration') == 2:
    raise RuntimeError('Se requiere v1, con su raiz existente, sin refinar previamente.')
if not (DOCS/'backups/southpark_before_iteration02.blend').is_file():
    raise RuntimeError('Crear una copia previa antes de modificar la escena.')

# Reutilizar solo definiciones de herramientas; no ejecutar el generador de escenas.
source = Path(__file__).with_name('build_southpark_planet.py')
tree = ast.parse(source.read_text(encoding='utf-8'))
definition_names = {'Geometry','map_angles','radial_height','normal','surface','frame_at',
                    'place','ribbon','windows','house_mesh','text_mesh'}
selected = [node for node in tree.body if isinstance(node,(ast.ClassDef,ast.FunctionDef))
            and node.name in definition_names]
color_node = next(node for node in tree.body if isinstance(node,ast.Assign)
                  and any(isinstance(t,ast.Name) and t.id=='colors' for t in node.targets))
material_names = list(ast.literal_eval(color_node.value))
materials = {name:bpy.data.materials['SP_'+name] for name in material_names}
material_indices = {name:i for i,name in enumerate(material_names)}
collections = {name:bpy.data.collections[name] for name in
               ('Residential','Commercial','Landmarks','Public_Buildings','Roads',
                'Mountains','Trees','SouthPark_Sign')}
exec(compile(ast.Module(body=selected,type_ignores=[]),str(source),'exec'),globals())
house_cache = {}
palettes = ('Teal','Blue','Lilac','Salmon','Green','Brown','Cream','Red')
rng = random.Random(20702)
scene = bpy.context.scene
planet = bpy.data.objects['Planet_Surface_Snow']

def mesh_hash(mesh):
    h=hashlib.sha256()
    for v in mesh.vertices:h.update(struct.pack('<3f',*v.co))
    for p in mesh.polygons:h.update(struct.pack('<%di'%len(p.vertices),*p.vertices))
    return h.hexdigest()

surface_hash = mesh_hash(planet.data)
surface_pointer = planet.as_pointer()
original_buildings = [o for o in root.children_recursive if o.get('building_id')]
original_names = {o.name for o in original_buildings}
original_roads = {o.name:mesh_hash(o.data) for o in collections['Roads'].objects}
old_inventory = json.loads((DOCS/'southpark_inventory.json').read_text(encoding='utf-8'))
old_data = {entry['object']:entry for entry in old_inventory['buildings']}
records = []
source_cache = {}

def source_geometry(mesh,is_house=False,commercial_roof=False):
    key=(mesh.name,is_house,commercial_roof)
    if key in source_cache:return source_cache[key]
    verts=[]
    for v in mesh.vertices:
        x,y,z=v.co
        sag=math.sqrt(max(0,RADIUS**2-x*x-y*y))-RADIUS
        verts.append(Vector((x,y,z-sag)))
    faces=[tuple(p.vertices) for p in mesh.polygons]
    mats=[mesh.materials[p.material_index].name for p in mesh.polygons]
    if is_house:
        # Conservar el tejado existente y sustituir solo su capota blanca por una placa.
        adjacency={i:set() for i in range(len(verts))}
        for f in faces:
            for i in f:adjacency[i].update(f)
        seen=set();remove=set();patch=None
        for index in range(len(verts)):
            if index in seen:continue
            todo=[index];component=set()
            while todo:
                i=todo.pop()
                if i in component:continue
                component.add(i);todo.extend(adjacency[i]-component)
            seen.update(component)
            pids=[i for i,f in enumerate(faces) if f[0] in component]
            if len(component)==6 and pids and all(mats[i]=='SP_SnowWhite' for i in pids):
                remove.update(pids)
                c=[verts[i] for i in component]
                patch=(max(abs(v.x) for v in c),max(abs(v.y) for v in c),
                       min(v.z for v in c),max(v.z for v in c))
        faces2=[];mats2=[]
        for i,(f,m) in enumerate(zip(faces,mats)):
            if i not in remove:faces2.append(f);mats2.append(m)
        faces,mats=faces2,mats2
        if patch:
            halfw,halfd,low,high=patch
            start=len(verts)
            for x,y,dz in ((.08,-.72,0),(.84,-.72,0),(.84,.68,0),(.08,.68,0),
                           (.08,-.72,.055),(.84,-.72,.055),(.84,.68,.055),(.08,.68,.055)):
                xx=x*halfw;yy=y*halfd;zz=high-(high-low)*abs(xx)/halfw+.012+dz
                verts.append(Vector((xx,yy,zz)))
            for f in ((3,2,1,0),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)):
                faces.append(tuple(start+i for i in f));mats.append('SP_SnowWhite')
    elif commercial_roof:
        mats=['SP_RoofRed' if m=='SP_SnowWhite' else m for m in mats]
    used=sorted({i for f in faces for i in f})
    remap={old:new for new,old in enumerate(used)}
    result=([verts[i] for i in used],[tuple(remap[i] for i in f) for f in faces],mats)
    source_cache[key]=result
    return result

def ground_z(frame,x,y):
    # Proyeccion de la cimentacion sobre la esfera original, sin alterar la esfera.
    origin=frame.translation
    axes=frame.to_3x3()
    point=origin+axes@Vector((x,y,0))
    n=point.normalized();lon=math.atan2(n.x,-n.y);lat=math.asin(n.z)
    radius=radial_height(lon,lat)
    z=math.sqrt(max(0,radius*radius-x*x-y*y))-origin.length
    point=origin+axes@Vector((x,y,z))
    n=point.normalized();lon=math.atan2(n.x,-n.y);lat=math.asin(n.z)
    radius=radial_height(lon,lat)
    return math.sqrt(max(0,radius*radius-x*x-y*y))-origin.length

# Completar el gran bloque entre avenidas con una calle secundaria y dos conexiones.
new_roads=[('Infill_Main',[(700,238),(1220,238)]),
           ('Infill_West_Link',[(700,202),(700,284)]),
           ('Infill_East_Link',[(1220,202),(1220,284)]),
           ('South_Neighborhood',[(490,401),(1390,401)]),
           ('South_West_Link',[(490,360),(490,401)]),
           ('South_East_Link',[(1390,360),(1390,401)])]
all_paths=[(r['name'],r['reference_points']) for r in old_inventory['roads']]+new_roads
for index,(name,points) in enumerate(new_roads):
    g=Geometry()
    ribbon(points,1.14,.078+index*.001,g,'SnowWhite')
    ribbon(points,.88,.086+index*.001,g,'Road')
    for a,b in zip(points,points[1:]):
        a,b=Vector(a),Vector(b)
        count=max(1,int((b-a).length/20))
        for j in range(count):
            ribbon([a.lerp(b,(j+.18)/count),a.lerp(b,(j+.5)/count)],.045,.097+index*.001,g,'Marking')
    g.object('Road_'+name,'Roads')

road_samples=[]
for _,points in all_paths:
    for a,b in zip(points,points[1:]):
        a,b=Vector(a),Vector(b)
        count=max(2,int((b-a).length/3))
        for i in range(count+1):
            p=a.lerp(b,i/count);road_samples.append(surface(p.x,p.y))

def footprint(frame,halfx,halfy):
    return [frame@Vector((x,y,0)) for x,y in
            ((-halfx,-halfy),(halfx,-halfy),(halfx,halfy),(-halfx,halfy))]

def polygons_overlap(a,b,basis,margin=.10):
    inv=basis.inverted()
    aa=[(inv@p).xy for p in a];bb=[(inv@p).xy for p in b]
    for polygon in (aa,bb):
        for p,q in zip(polygon,polygon[1:]+polygon[:1]):
            edge=q-p
            if edge.length<1e-6:continue
            axis=Vector((-edge.y,edge.x)).normalized()
            pa=[v.dot(axis) for v in aa];pb=[v.dot(axis) for v in bb]
            if max(pa)+margin<min(pb) or max(pb)+margin<min(pa):return False
    return True

occupied=[]
def valid_position(x,y,halfx,halfy,heading=0,ignore_roads=False):
    frame=frame_at(x,y)@Matrix.Rotation(heading,4,'Z')
    frame.translation=surface(x,y)
    radius=math.hypot(halfx,halfy)
    poly=footprint(frame,halfx,halfy)
    for record in occupied:
        if (frame.translation-record['frame'].translation).length>radius+record['radius']+.3:continue
        if polygons_overlap(poly,record['polygon'],frame):return None
    if not ignore_roads:
        inv=frame.inverted()
        for p in road_samples:
            if (p-frame.translation).length>radius+1:continue
            local=inv@p
            dx=max(0,abs(local.x)-halfx);dy=max(0,abs(local.y)-halfy)
            if dx*dx+dy*dy < .52**2:return None
    # Proteger el cartel y el estacionamiento existente.
    sign=surface(650,164)
    if (frame.translation-sign).length<radius+1.7:return None
    lon,lat=map_angles(x,y)
    if 738<x<963 and 167<y<205:return None
    return frame

def bake_building(obj,source_mesh,x,y,factors,heading=0,is_house=False,roof_color=False):
    verts,faces,mats=source_geometry(source_mesh,is_house,roof_color)
    sx,sy,sz=factors
    hx=max(abs(v.x*sx) for v in verts);hy=max(abs(v.y*sy) for v in verts)
    offsets=[(0,0),(0,4),(0,-4),(5,0),(-5,0),(0,8),(0,-8),(10,0),(-10,0),
             (10,8),(-10,8),(10,-8),(-10,-8),(0,14),(0,-14),(16,0),(-16,0)]
    chosen=None
    for dx,dy in offsets:
        frame=valid_position(x+dx,y+dy,hx,hy,heading)
        if frame is not None:chosen=(frame,x+dx,y+dy);break
    if chosen is None:return None
    frame,x,y=chosen
    low=min(v.z for v in verts)
    transformed=[]
    for v in verts:
        xx,yy=v.x*sx,v.y*sy
        # Una cimentacion ligera (0.01) evita huecos por precision numerica.
        zz=(v.z-low)*sz+ground_z(frame,xx,yy)-.01
        transformed.append((xx,yy,zz))
    mesh=bpy.data.meshes.new(obj.name+'_RefinedMesh')
    mesh.from_pydata(transformed,[],faces);mesh.update()
    names=sorted(set(mats));mapping={name:i for i,name in enumerate(names)}
    for name in names:mesh.materials.append(bpy.data.materials[name])
    for p,name in zip(mesh.polygons,mats):p.material_index=mapping[name]
    obj.data=mesh;obj.parent=root;obj.matrix_world=frame
    obj['reference_xy']=[x,y];obj['iteration']=2;obj['scale_xyz_v2']=list(factors)
    occupied.append(dict(object=obj,frame=frame,polygon=footprint(frame,hx,hy),
                         radius=math.hypot(hx,hy),halfx=hx,halfy=hy))
    return x,y

def desired_scale(item):
    kind=item['kind']
    if kind=='house':
        if item['sector']=='residential_northeast':return (1.24,1.38,1.90)
        return (1.27,1.40,1.90)
    return {'school':(1.38,1.45,1.88),'office':(1.28,1.32,1.75),
            'civic':(1.40,1.40,1.60),'mall':(1.15,1.20,1.72),
            'pagoda':(1.30,1.30,1.55),'barn':(1.38,1.38,1.68),
            'garage':(1.27,1.35,1.65),'pavilion':(1.36,1.36,1.70),
            'restaurant':(1.18,1.28,1.75),'shop':(1.18,1.30,1.88)}[kind]

# Preservar los 71 objetos existentes. Priorizar los hitos al reservar posiciones.
order=sorted(original_buildings,key=lambda o:0 if old_data[o.name]['kind'] not in ('house','shop')
             else (1 if old_data[o.name]['kind']=='house' else 2))
central=sorted([o for o in original_buildings if o.get('reference_sector')=='residential_central'],
               key=lambda o:o['reference_xy'][0])
northeast=sorted([o for o in original_buildings if o.get('reference_sector')=='residential_northeast'],
                 key=lambda o:o['reference_xy'][0])
for obj in order:
    item=dict(old_data[obj.name]);x,y=item['x'],item['y'];heading=0
    if obj in central:x=505+central.index(obj)*32.5
    if obj in northeast:x=1110+northeast.index(obj)*42
    if item['sector'] in ('residential_central_back','residential_east_back'):
        y=305;heading=math.pi
    factors=desired_scale(item)
    original_mesh=obj.data
    result=bake_building(obj,original_mesh,x,y,factors,heading,item['kind']=='house',
                         item['kind']=='shop' and int(item['id'][-3:])%3!=0)
    if result is None:
        # En parcelas estrechas priorizar la altura y limitar solo la huella.
        factors=(factors[0]*.88,factors[1]*.88,factors[2])
        result=bake_building(obj,original_mesh,x,y,factors,heading,item['kind']=='house')
    if result is None:raise RuntimeError('Revisar parcela del edificio existente '+obj.name)
    item.update(x=result[0],y=result[1],scale_xyz=list(factors),source='v1_preserved')
    records.append(item)
print('Edificios originales conservados y apoyados:',len(records))

# Relleno de barrios en hileras: no es una dispersion aleatoria sobre la esfera.
candidates=[]
def row(xs,y,kind,sector,heading=0):
    candidates.extend((x,y,kind,sector,heading) for x in xs)
row(range(730,1201,39),216,'house','infill_north_block')
row((525,563,600,638,677,715,753,791,829,867,905),305,'house','infill_central_back',math.pi)
row((992,1032,1072,1112,1152,1192,1232,1272,1312,1352),305,'house','infill_east_back',math.pi)
row(range(520,1370,40),379,'house','infill_south_front',math.pi)
row(range(520,1370,44),422,'house','infill_south_back',math.pi)
row((206,275,313,350,530,568,720,760,795),258,'shop','infill_main_street')
row((225,275,325,365,530,575),219,'house','infill_west_block')
row((520,565,735,980,1020,1060),177,'house','infill_north_avenue')
row((1120,1165,1208,1260,1310,1360,1410),215,'shop','infill_east_north',math.pi)
row((1460,1505,1550,1595),316,'house','infill_back_neighborhood')
row((230,275,320,365,410),379,'house','infill_west_residential',math.pi)

new_count=0;rejected=[]
for index,(x,y,kind,sector,heading) in enumerate(candidates):
    # Los nuevos comercios parten del datablock v1 conservado, no de otro ampliado.
    mesh=house_mesh(index%8) if kind=='house' else bpy.data.meshes['SP_B039_Shop_Mesh']
    number=72+new_count
    obj=bpy.data.objects.new('SP_B%03d_%s'%(number,kind.title()),mesh)
    collections['Residential' if kind=='house' else 'Commercial'].objects.link(obj)
    factors=(1.22,1.34,1.78) if kind=='house' else (1.15,1.24,1.82)
    result=bake_building(obj,mesh,x,y,factors,heading,kind=='house',kind=='shop' and index%2==0)
    if result is None:
        bpy.data.objects.remove(obj,do_unlink=True);rejected.append([x,y,sector]);continue
    new_count+=1
    obj['building_id']='SP_B%03d'%number
    obj['reference_description']='Relleno reinterpretado del barrio: '+sector
    obj['reference_sector']=sector
    records.append(dict(id=obj['building_id'],object=obj.name,label=obj['reference_description'],
                        x=result[0],y=result[1],kind=kind,sector=sector,
                        scale_xyz=list(factors),source='v2_neighborhood_infill'))
print('Nuevas construcciones aceptadas:',new_count,'; total:',len(records))

# Hitos: escuela mas reconocible y rotulos descriptivos, sin asignar identidades inciertas.
school=next(o for o in original_buildings if old_data[o.name]['kind']=='school')
for child in list(school.children):
    if child.name.startswith('School_Label'):bpy.data.objects.remove(child,do_unlink=True)
for body,z in (('SOUTH PARK',3.49),('ELEMENTARY',3.11)):
    label=text_mesh('School_v2_'+body.replace(' ','_'),body,.34,(0,-1.55,z),
                    (math.pi/2,0,0),school,'Landmarks')
    label.data.materials.clear();label.data.materials.append(materials['Frame'])

for index,item in enumerate([i for i in records if i['kind']=='shop'][::6]):
    obj=bpy.data.objects[item['object']]
    extent=max(abs(v.co.y) for v in obj.data.vertices)
    height=max(v.co.z for v in obj.data.vertices)
    text_mesh('Shop_Label_%02d'%index,('SHOP','CAFE','MARKET')[index%3],.24,
              (0,-extent-.04,height*.72),(math.pi/2,0,0),obj,'Commercial')

# Distribuir las 14 montanas existentes en una cordillera abierta y ramificada.
mountains=sorted(collections['Mountains'].objects,key=lambda o:o.name)
mountain_coords=[(200,156),(300,126),(380,100),(480,115),(560,142),(660,117),
                 (750,90),(820,60),(940,83),(1030,110),(1140,138),(1260,120),
                 (1360,144),(1080,65)]
mountain_coords[10]=(1150,103)  # Separar la ladera del barrio noreste ampliado.
mountain_records=[]
for index,(obj,(x,y)) in enumerate(zip(mountains,mountain_coords)):
    place(obj,x,y,(-.18 if index%2 else .22))
    obj.scale=(1.10+(index%3)*.11,.82+(index%4)*.12,.72+(index%5)*.10)
    mountain_records.append(dict(object=obj.name,x=x,y=y,scale=list(obj.scale)))

# Reagrupar los pinos existentes. Una malla compartida, copas mas visibles y claros urbanos.
trees=sorted(collections['Trees'].objects,key=lambda o:o.name)
clusters=[(150,209,42,20),(320,170,70,22),(470,167,50,17),
          (880,151,65,20),(1040,154,70,22),(1430,175,75,30),
          (110,380,55,32),(420,420,60,30),(790,463,95,25),
          (1180,462,110,30),(1515,385,85,34),(65,290,45,24),
          (520,238,52,15),(1420,244,45,12),(220,447,95,25),
          (490,465,110,26),(1430,449,100,24),(40,415,65,26)]
tree_records=[]
def tree_valid(position,canopy):
    for record in occupied:
        if (position-record['frame'].translation).length>record['radius']+canopy+.2:continue
        p=record['frame'].inverted()@position
        dx=max(0,abs(p.x)-record['halfx']);dy=max(0,abs(p.y)-record['halfy'])
        if dx*dx+dy*dy < (canopy+.20)**2:return False
    for p in road_samples:
        if (p-position).length<canopy+.48:return False
    if any((position-Vector(t['position'])).length<.84*(canopy+t['canopy']) for t in tree_records):return False
    if (position-surface(650,164)).length<3.1:return False
    for obj in mountains:
        if (position-obj.location).length<2.9*max(obj.scale.x,obj.scale.y):return False
    n=position.normalized();lo=math.atan2(n.x,-n.y);la=math.asin(n.z)
    x=lo/math.tau*1600+800;y=280-math.degrees(la)/.35
    if 730<x<970 and 168<y<205:return False
    if abs(x-78)<65 and abs(y-248)<18:return False
    return True

for index,obj in enumerate(trees):
    size=1.10+(index%5)*.075
    canopy=.57*size
    candidate=None
    for attempt in range(3500):
        cx,cy,rx,ry=clusters[(index+attempt)%len(clusters)]
        a=rng.random()*math.tau;rr=math.sqrt(rng.random())
        x=cx+math.cos(a)*rx*rr;y=cy+math.sin(a)*ry*rr
        position=surface(x,y)
        if tree_valid(position,canopy):candidate=(x,y,position);break
    if candidate is None:raise RuntimeError('No hay espacio seguro para el pino '+obj.name)
    x,y,position=candidate
    place(obj,x,y,rng.uniform(-math.pi,math.pi));obj.scale=(size,size,size)
    tree_records.append(dict(object=obj.name,x=x,y=y,position=list(position),canopy=canopy,scale=size))

# Colores mas presentes, respetando materiales compartidos y acabado mate.
palette_adjustments={'SP_Teal':(.10,.43,.33),'SP_Blue':(.16,.32,.62),
                     'SP_Lilac':(.44,.21,.56),'SP_Salmon':(.69,.26,.15),
                     'SP_Green':(.22,.46,.12),'SP_Red':(.64,.055,.028)}
for name,color in palette_adjustments.items():
    mat=bpy.data.materials[name];mat.diffuse_color=(*color,1)
    node=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    node.inputs['Base Color'].default_value=(*color,1)

assert planet.as_pointer()==surface_pointer and mesh_hash(planet.data)==surface_hash
assert original_names.issubset({o.name for o in root.children_recursive})
for name,digest in original_roads.items():assert mesh_hash(bpy.data.objects[name].data)==digest
root['iteration']=2
root['design']='Ciudad ampliada por categorias y barrios; cordillera abierta y bosques agrupados.'
report={'iteration':2,'buildings_before':71,'buildings_after':len(records),
        'new_buildings':new_count,'original_buildings_preserved':len(original_names),
        'sphere_geometry_sha256':surface_hash,'sphere_geometry_unchanged':True,
        'original_road_geometry_unchanged':True,'added_roads':[name for name,_ in new_roads],
        'buildings':records,'mountains':mountain_records,'trees':tree_records,
        'rejected_infill_slots':rejected,
        'reference_note':'Las adiciones son relleno interpretado de los barrios del mapa, '
                         'no nuevos lugares famosos identificados ni una certificacion de cobertura del 80%.'}
(DOCS/'southpark_iteration02_inventory.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('Iteracion 2 aplicada sin reconstruir esfera ni red original:',len(records),'edificios.')
