"""Identidad visual v3 sobre South Park v2 existente. No genera otra esfera.

Requiere la copia previa docs/references/southpark/backups/
southpark_before_iteration03.blend. No guarda el .blend automaticamente.
"""
import ast
import bpy
import math
import json
import hashlib
import struct
from pathlib import Path
from mathutils import Vector, Matrix
from mathutils.kdtree import KDTree

REPO=Path(__file__).resolve().parents[3]
DOCS=REPO/'docs/references/southpark'
root=bpy.data.objects.get('SouthPark_Planet_ROOT')
if not root or root.get('iteration')!=2:
    raise RuntimeError('Abrir v2 existente; este refinamiento no debe repetirse sobre v3.')
if not (DOCS/'backups/southpark_before_iteration03.blend').is_file():
    raise RuntimeError('Crear primero la copia de seguridad de v2.')

# Solo importar definiciones; nunca ejecutar la construccion de la escena v1.
source=Path(__file__).with_name('build_southpark_planet.py')
parsed=ast.parse(source.read_text(encoding='utf-8'))
names={'Geometry','map_angles','radial_height','normal','surface','frame_at','place'}
definitions=[n for n in parsed.body if isinstance(n,(ast.ClassDef,ast.FunctionDef)) and n.name in names]
colors_node=next(n for n in parsed.body if isinstance(n,ast.Assign)
                 and any(isinstance(t,ast.Name) and t.id=='colors' for t in n.targets))
material_names=list(ast.literal_eval(colors_node.value))
materials={name:bpy.data.materials['SP_'+name] for name in material_names}
material_indices={name:i for i,name in enumerate(material_names)}
collections={name:bpy.data.collections[name] for name in
             ('Residential','Commercial','Landmarks','Public_Buildings','Roads',
              'Mountains','Trees','SouthPark_Sign')}
RADIUS=22.0
exec(compile(ast.Module(body=definitions,type_ignores=[]),str(source),'exec'),globals())

def mesh_hash(mesh):
    h=hashlib.sha256()
    for v in mesh.vertices:h.update(struct.pack('<3f',*v.co))
    for p in mesh.polygons:h.update(struct.pack('<%di'%len(p.vertices),*p.vertices))
    return h.hexdigest()

buildings=[o for o in root.children_recursive if o.get('building_id')]
assert len(buildings)==160
inventory=json.loads((DOCS/'southpark_iteration02_inventory.json').read_text(encoding='utf-8'))
items={i['object']:i for i in inventory['buildings']}
sphere=bpy.data.objects['Planet_Surface_Snow']
sphere_hash=mesh_hash(sphere.data)
roads_hash={o.name:mesh_hash(o.data) for o in collections['Roads'].objects}
sign_hash={o.name:mesh_hash(o.data) for o in collections['SouthPark_Sign'].objects if o.type=='MESH'}

def components(mesh):
    adjacency={i:set() for i in range(len(mesh.vertices))}
    for p in mesh.polygons:
        for i in p.vertices:adjacency[i].update(p.vertices)
    seen=set();result=[]
    for i in range(len(mesh.vertices)):
        if i in seen:continue
        todo=[i];component=set()
        while todo:
            k=todo.pop()
            if k in component:continue
            component.add(k);todo.extend(adjacency[k]-component)
        seen.update(component);result.append(component)
    return result

wall_names={'SP_Teal','SP_Blue','SP_Lilac','SP_Salmon','SP_Green','SP_Brown',
            'SP_Cream','SP_Red','SP_Yellow'}
def body_dimensions(obj):
    mesh=obj.data;options=[]
    for c in components(mesh):
        if len(c)!=8:continue
        ps=[p for p in mesh.polygons if p.vertices[0] in c]
        if not ps or mesh.materials[ps[0].material_index].name not in wall_names:continue
        points=[mesh.vertices[i].co for i in c]
        w=max(v.x for v in points)-min(v.x for v in points)
        d=max(v.y for v in points)-min(v.y for v in points)
        h=max(v.z for v in points)-min(v.z for v in points)
        options.append((w*d*h,w,d,h,min(v.z for v in points),c))
    option=max(options,key=lambda item:item[0])
    return option[1:]

glyph_cache={}
def glyph(body):
    if body in glyph_cache:return glyph_cache[body]
    curve=bpy.data.curves.new('SP_Temporary_Glyph','FONT')
    curve.body=body;curve.align_x='CENTER';curve.align_y='CENTER'
    curve.size=1;curve.extrude=.002;curve.resolution_u=2
    obj=bpy.data.objects.new('SP_Temporary_Glyph',curve)
    collections['Landmarks'].objects.link(obj)
    bpy.context.view_layer.update()
    mesh=bpy.data.meshes.new_from_object(obj.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    data=([v.co.copy() for v in mesh.vertices],[tuple(p.vertices) for p in mesh.polygons])
    bpy.data.objects.remove(obj,do_unlink=True);bpy.data.curves.remove(curve);bpy.data.meshes.remove(mesh)
    glyph_cache[body]=data
    return data

def lettering(g,body,center,width,height,material='Frame'):
    verts,faces=glyph(body)
    xmin,xmax=min(v.x for v in verts),max(v.x for v in verts)
    ymin,ymax=min(v.y for v in verts),max(v.y for v in verts)
    scale=min(width/(xmax-xmin),height/(ymax-ymin))
    points=[(center[0]+(v.x-(xmin+xmax)/2)*scale,
             center[1]-v.z*scale,center[2]+(v.y-(ymin+ymax)/2)*scale) for v in verts]
    g.solid(points,faces,material)

def attached(g,name,parent,group,curved=False):
    obj=g.object(name,group,curved=curved)
    obj.parent=parent;obj.matrix_parent_inverse=Matrix.Identity(4);obj.matrix_basis=Matrix.Identity(4)
    obj['iteration']=3
    return obj

def set_color(name,color):
    m=bpy.data.materials[name];m.diffuse_color=(*color,1)
    node=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    node.inputs['Base Color'].default_value=(*color,1)

# Paleta limpia: ventanas definidas, nieve clara y laderas oscuras de caricatura.
set_color('SP_Glass',(.105,.34,.48))
set_color('SP_Mountain',(.035,.145,.075))
set_color('SP_MountainBlue',(.12,.19,.20))
set_color('SP_Snow',(.92,.96,.98))
set_color('SP_SnowBlue',(.84,.90,.94))

# Variacion tonal amplia y sutil, solo materiales: geometria esferica inalterada.
for p in sphere.data.polygons:
    center=sum((sphere.data.vertices[i].co for i in p.vertices),Vector())/len(p.vertices)
    n=center.normalized();lo=math.atan2(n.x,-n.y);la=math.asin(n.z)
    value=math.sin(lo*3+la)*math.cos(la*4)+.3*math.cos(lo*5-la)
    p.material_index=1 if value<-.72 else (2 if value>.70 else 0)

# Variaciones pequeñas de implantacion, con prueba de huellas y de carreteras.
def ground_z(frame,x,y):
    origin=frame.translation;axes=frame.to_3x3()
    point=origin+axes@Vector((x,y,0))
    for _ in range(2):
        n=point.normalized();lo=math.atan2(n.x,-n.y);la=math.asin(n.z)
        r=radial_height(lo,la)
        z=math.sqrt(max(0,r*r-x*x-y*y))-origin.length
        point=origin+axes@Vector((x,y,z))
    return z

def footprint(obj,frame=None):
    frame=frame if frame is not None else obj.matrix_world
    xs=[v.co.x for v in obj.data.vertices];ys=[v.co.y for v in obj.data.vertices]
    return [frame@Vector((x,y,0)) for x,y in
            ((min(xs),min(ys)),(max(xs),min(ys)),(max(xs),max(ys)),(min(xs),max(ys)))]

def overlap(a,b,frame):
    inv=frame.inverted();aa=[(inv@v).xy for v in a];bb=[(inv@v).xy for v in b]
    for poly in (aa,bb):
        for p,q in zip(poly,poly[1:]+poly[:1]):
            e=q-p
            if e.length<1e-7:continue
            axis=Vector((-e.y,e.x)).normalized()
            pa=[v.dot(axis) for v in aa];pb=[v.dot(axis) for v in bb]
            if max(pa)+.04<min(pb) or max(pb)+.04<min(pa):return False
    return True

road_points=[o.matrix_world@v.co for o in collections['Roads'].objects
             if o.name.startswith('Road_') for v in o.data.vertices]
polygons={o.name:footprint(o) for o in buildings}
adjusted=[]
houses=[o for o in buildings if items[o.name]['kind']=='house']
for obj in houses:
    number=int(obj['building_id'][-3:])
    if number%4:continue
    x,y=obj['reference_xy']
    nx=x+(-1 if number%3 else 1)*1.1;ny=y+(.45 if number%5 else -.45)
    old=obj.matrix_world.copy()
    # Conservar la orientacion hacia la calle; sumar un giro local muy moderado.
    old_basis=frame_at(x,y).to_3x3().inverted()@old.to_3x3()
    frame=frame_at(nx,ny);frame.translation=surface(nx,ny)
    frame=frame@old_basis.to_4x4()@Matrix.Rotation(.018 if number%3 else -.018,4,'Z')
    candidate=footprint(obj,frame)
    if any((frame.translation-other.location).length<8 and
           overlap(candidate,polygons[other.name],frame) for other in buildings if other!=obj):continue
    inv=frame.inverted();xs=[v.co.x for v in obj.data.vertices];ys=[v.co.y for v in obj.data.vertices]
    blocked=False
    for p in road_points:
        if (p-frame.translation).length>4:continue
        v=inv@p
        if min(xs)-.05<v.x<max(xs)+.05 and min(ys)-.05<v.y<max(ys)+.05:
            blocked=True;break
    if blocked:continue
    for v in obj.data.vertices:
        v.co.z+=ground_z(frame,v.co.x,v.co.y)-ground_z(old,v.co.x,v.co.y)
    obj.matrix_world=frame;obj['reference_xy']=[nx,ny]
    obj.data.update();polygons[obj.name]=candidate
    adjusted.append(obj.name)

# Ocho familias de tejados; sin cambiar cimentaciones ni densidad de los barrios.
roof_variants={}
for obj in houses:
    style=int(obj['building_id'][-3:])%8
    roof_variants[obj.name]=style;obj['residential_family']='SP_Residential_%02d'%style
    mesh=obj.data
    primary=[c for c in components(mesh) if len(c)==6 and
             any(mesh.materials[p.material_index].name in ('SP_Roof','SP_RoofRed')
                 for p in mesh.polygons if p.vertices[0] in c)]
    if not primary:continue
    roof=primary[0];points=[mesh.vertices[i].co for i in roof]
    halfw=max(abs(v.x) for v in points)
    halfd=max(abs(v.y) for v in points)
    low=min(v.z-ground_z(obj.matrix_world,v.x,v.y) for v in points)
    high=max(v.z-ground_z(obj.matrix_world,v.x,v.y) for v in points)
    delta=(-.09,.08,.13,-.05,.11,.02,-.08,.06)[style]
    for i in roof:
        v=mesh.vertices[i]
        g=ground_z(obj.matrix_world,v.co.x,v.co.y)
        if v.co.z-g>low+(high-low)*.7 and style in (2,5):
            v.co.y*=.66
        v.co.z+=delta*max(0,1-abs(v.co.x)/halfw)
    # Las pequeñas placas de nieve siguen la pendiente modificada.
    snowy={i for p in mesh.polygons if mesh.materials[p.material_index].name=='SP_SnowWhite'
           for i in p.vertices if mesh.vertices[i].co.z-ground_z(obj.matrix_world,
               mesh.vertices[i].co.x,mesh.vertices[i].co.y)>low+.2}
    for i in snowy:
        v=mesh.vertices[i]
        v.co.z+=delta*max(0,1-abs(v.co.x)/halfw)
        if style in (2,5):v.co.y*=.60
    # Una familia pequeña de combinaciones, no un material nuevo por vivienda.
    target=('SP_RoofRed','SP_Brown','SP_Roof','SP_RoofRed',
            'SP_Brown','SP_Roof','SP_RoofRed','SP_Brown')[style]
    if target not in [m.name for m in mesh.materials]:mesh.materials.append(bpy.data.materials[target])
    slot=next(i for i,m in enumerate(mesh.materials) if m.name==target)
    for p in mesh.polygons:
        if p.vertices[0] in roof:p.material_index=slot
    mesh.update()

# Cuatro kits compartidos para casas de esquina: marquesina y ventana lateral.
corner_meshes=[]
for style in range(4):
    g=Geometry();side=1 if style%2==0 else -1
    g.box((0,-.49,.43),(.42,.26,.055),'RoofRed' if style<2 else 'Brown')
    for x in (-.17,.17):g.box((x,-.54,.28),(.025,.025,.28),'Frame')
    g.box((side*.48,.02,.59),(.13,.42,.32),'Frame')
    g.box((side*.552,.02,.59),(.025,.34,.25),'Glass')
    g.box((side*.568,.02,.59),(.018,.025,.25),'Frame')
    corner_meshes.append(g.mesh('SP_Residential_Corner_Kit_%02d'%style))

intersections=[(475,202),(475,284),(475,360),(700,202),(700,238),(700,284),
               (930,284),(930,360),(1220,202),(1220,238),(1220,284),(1390,401),(490,401)]
ranking=sorted(houses,key=lambda o:min((surface(*xy)-o.location).length for xy in intersections))
corner_objects=[]
for index,obj in enumerate(ranking[:24]):
    w,d,h,bottom,_=body_dimensions(obj)
    child=bpy.data.objects.new(obj.name+'_Corner_Details',corner_meshes[index%4])
    collections['Residential'].objects.link(child);child.parent=obj
    child.location=(0,0,bottom+.03);child.scale=(w,d,h)
    corner_objects.append(child.name)

# Escaparates comerciales: tres kits completos compartidos, con letreros de malla.
store_meshes=[]
for style,word in enumerate(('MARKET','CAFE','STORE')):
    g=Geometry()
    for x in (-.275,.275):
        g.box((x,-.548,.37),(.37,.035,.56),'Frame')
        g.box((x,-.575,.37),(.31,.025,.49),'Glass')
        g.box((x,-.592,.37),(.016,.015,.49),'Frame')
        g.box((x,-.594,.38),(.31,.015,.022),'Frame')
    g.box((0,-.555,.32),(.19,.04,.64),'Frame')
    g.box((0,-.586,.33),(.145,.025,.56),'Glass')
    g.box((.045,-.61,.30),(.015,.015,.10),'Yellow')
    g.box((0,-.65,.75),(1.02,.31,.065),('Red','Teal','Brown')[style])
    for x in (-.40,-.20,0,.20,.40):g.box((x,-.66,.788),(.09,.30,.012),'Frame')
    g.box((0,-.56,.89),(.96,.065,.16),'Roof')
    lettering(g,word,(0,-.599,.89),.84,.105,'Frame')
    store_meshes.append(g.mesh('SP_Storefront_%s_Shared'%word))

store_objects=[]
for index,obj in enumerate([o for o in buildings if items[o.name]['kind'] in ('shop','restaurant')]):
    w,d,h,bottom,_=body_dimensions(obj)
    # Elevar el toldo preexistente, para despejar el escaparate inferior nuevo.
    for c in components(obj.data):
        if len(c)!=8:continue
        points=[obj.data.vertices[i].co for i in c]
        cx=sum(v.x for v in points)/8;cy=sum(v.y for v in points)/8
        width=max(v.x for v in points)-min(v.x for v in points)
        if cy<-d*.58 and width>w*.8:
            center=sum(v.z for v in points)/8
            dz=bottom+h*.75-center
            for i in c:obj.data.vertices[i].co.z+=dz
    for child in list(obj.children):
        if child.name.startswith('Shop_Label_'):bpy.data.objects.remove(child,do_unlink=True)
    child=bpy.data.objects.new(obj.name+'_Storefront',store_meshes[index%3])
    collections['Commercial'].objects.link(child);child.parent=obj
    child.location=(0,0,bottom+.02);child.scale=(w,d,h)
    child['label_identity']='Rotulo generico; no identifica un comercio canonico.'
    store_objects.append(child.name);obj.data.update()

def by_kind(kind):return next(o for o in buildings if items[o.name]['kind']==kind)

# South Park Elementary: fascia azul, bandas rojas y entrada doble propia de escuela.
school=by_kind('school');g=Geometry()
for y in (-1.28,1.28):g.box((0,y,4.00),(7.25,.13,.20),'Blue')
for x in (-3.57,3.57):g.box((x,0,4.00),(.13,2.56,.20),'Blue')
g.box((0,-1.30,2.32),(6.90,.12,.14),'Red')
for x in (-.70,.70):g.box((x,-1.36,.87),(.16,.20,1.68),'Red')
g.box((0,-1.36,1.73),(1.55,.20,.19),'Red')
for x in (-.28,.28):
    g.box((x,-1.405,.78),(.52,.07,1.40),'Frame')
    g.box((x,-1.455,.81),(.43,.035,1.28),'Glass')
    g.box((x+(-.13 if x>0 else .13),-1.48,.74),(.035,.025,.15),'Yellow')
school_feature=attached(g,'SouthPark_Elementary_Facade',school,'Landmarks',curved=True)

# Edificio publico con cupula: reforzar la lectura civica sin afirmar identidad no comprobada.
civic=by_kind('civic');g=Geometry()
# Sustituir solo los cinco conos provisionales de la cupula, no el edificio.
old_civic=civic.data
dome_vertices=set()
for component in components(old_civic):
    polys=[p for p in old_civic.polygons if p.vertices[0] in component]
    if len(component)==13 and polys and all(old_civic.materials[p.material_index].name=='SP_Concrete' for p in polys):
        dome_vertices.update(component)
if dome_vertices:
    faces=[tuple(p.vertices) for p in old_civic.polygons if p.vertices[0] not in dome_vertices]
    slots=[p.material_index for p in old_civic.polygons if p.vertices[0] not in dome_vertices]
    used=sorted({i for f in faces for i in f});remap={old:new for new,old in enumerate(used)}
    mesh=bpy.data.meshes.new('SP_Civic_Body_v03')
    mesh.from_pydata([old_civic.vertices[i].co.copy() for i in used],[],
                     [tuple(remap[i] for i in f) for f in faces])
    for mat in old_civic.materials:mesh.materials.append(mat)
    for p,index in zip(mesh.polygons,slots):p.material_index=index
    mesh.update();civic.data=mesh
for ring in range(5):
    a=ring/5*math.pi/2;b=(ring+1)/5*math.pi/2
    for side in range(12):
        u=side/12*math.tau;v=(side+1)/12*math.tau
        def dome_point(theta,phi):
            return (.84*math.cos(theta)*math.cos(phi),.84*math.cos(theta)*math.sin(phi),
                    3.13+1.20*math.sin(theta))
        points=[dome_point(a,u),dome_point(a,v),dome_point(b,v)]
        if ring<4:points.append(dome_point(b,u))
        g.face(points,'Concrete')
for index in range(3):g.box((0,-1.37,.03+index*.085),(3.05-index*.17,.48-index*.07,.13),'Concrete')
g.box((0,-1.31,1.10),(.56,.065,1.68),'Frame')
g.box((0,-1.356,1.10),(.44,.035,1.54),'Glass')
g.box((.12,-1.38,1.04),(.035,.02,.15),'Yellow')
for x in (-.75,.75):
    g.box((x,-1.31,1.58),(.35,.06,.88),'Frame')
    g.box((x,-1.355,1.58),(.27,.025,.72),'Glass')
for side in (-1,1):
    for y in (-.42,.48):
        g.box((side*1.64,y,1.58),(.07,.55,.88),'Frame')
        g.box((side*1.69,y,1.58),(.025,.43,.72),'Glass')
for x in (-1.12,-.38,.38,1.12):
    g.box((x,-1.38,1.36),(.18,.26,2.23),'Frame')
    g.box((x,-1.38,.29),(.27,.34,.12),'Frame')
    g.box((x,-1.38,2.48),(.27,.34,.12),'Frame')
pediment_start=len(g.vertices)
g.roof(3.22,.48,2.61,.59,'Frame')
for index in range(pediment_start,len(g.vertices)):
    x,y,z=g.vertices[index];g.vertices[index]=(x,y-1.38,z)
g.box((0,-1.655,2.43),(2.72,.04,.28),'Roof')
lettering(g,'CIVIC HALL',(0,-1.68,2.43),2.48,.18,'Frame')
# Reloj tridimensional sencillo en el fronton.
for i in range(16):
    a=math.tau*i/16;b=math.tau*(i+1)/16
    g.face([(0,-1.665,2.94),(.19*math.cos(a),-1.665,2.94+.19*math.sin(a)),
            (.19*math.cos(b),-1.665,2.94+.19*math.sin(b))],'SnowWhite')
g.box((0,-1.69,3.005),(.018,.018,.14),'Black')
g.box((.055,-1.69,2.94),(.13,.018,.018),'Black')
civic_feature=attached(g,'SP_Civic_Portico_and_Clock',civic,'Landmarks',curved=True)
civic_feature['identity_note']='Interpretacion del edificio con cupula; identidad de ayuntamiento sin confirmar.'

# Gran edificio con estacionamiento: galeria y cartel descriptivo, sin nombre canonico supuesto.
mall=by_kind('mall');g=Geometry()
for x in (-3.7,-1.85,1.85,3.7):
    g.box((x,-1.47,1.15),(1.42,.075,1.45),'Frame')
    g.box((x,-1.525,1.15),(1.25,.035,1.28),'Glass')
for x in (-4.60,-2.78,-.92,.92,2.78,4.60):g.box((x,-1.48,1.24),(.16,.15,2.27),'Cream')
g.box((0,-1.47,2.73),(5.70,.15,.46),'Yellow')
lettering(g,'MALL',(0,-1.56,2.73),3.0,.34,'Black')
g.box((0,-1.45,.83),(1.33,.10,1.55),'Frame')
g.box((0,-1.52,.83),(1.14,.05,1.38),'Glass')
mall_feature=attached(g,'SP_Mall_Arcade',mall,'Landmarks',curved=True)

# Pagoda y granero: reforzar los elementos que se distinguen en el mapa.
pagoda=by_kind('pagoda');g=Geometry()
for z in (1.55,2.88,4.19,5.48):
    g.box((0,-.80,z),(1.75,.08,.11),'Yellow')
    for x in (-.70,.70):g.box((x,-.76,z-.30),(.10,.10,.54),'Yellow')
pagoda_feature=attached(g,'SP_Red_Pagoda_Trim',pagoda,'Landmarks')

barn=by_kind('barn');g=Geometry()
for x in (-.55,.55):g.box((x,-1.29,1.28),(.10,.08,2.18),'Frame')
g.box((0,-1.29,2.34),(1.20,.08,.11),'Frame')
def beam(g,a,b,width,material):
    a,b=Vector(a),Vector(b);v=b-a
    perpendicular=Vector((-v.z,0,v.x)).normalized()*width/2
    g.face([a-perpendicular,b-perpendicular,b+perpendicular,a+perpendicular],material)
beam(g,(-.47,-1.34,.30),(.47,-1.34,2.22),.08,'Frame')
beam(g,(.47,-1.345,.30),(-.47,-1.345,2.22),.08,'Frame')
barn_feature=attached(g,'SP_Barn_Door_Braces',barn,'Landmarks',curved=True)

# Tres variantes enlazadas de pino: misma silueta, diferente presencia de nieve.
pine=bpy.data.meshes.get('Pine_Shared_Mesh')
pine_variants=[pine]
for style in (1,2):
    mesh=pine.copy();mesh.name='SP_Pine_Snow_Variant_%d'%style
    desired='SP_PineLight' if style==1 else 'SP_Pine'
    if desired not in [m.name for m in mesh.materials]:mesh.materials.append(bpy.data.materials[desired])
    target=next(i for i,m in enumerate(mesh.materials) if m.name==desired)
    for p in mesh.polygons:
        if mesh.materials[p.material_index].name=='SP_SnowWhite' and (style==1 or p.index%3):
            p.material_index=target
    pine_variants.append(mesh)
for index,obj in enumerate(sorted(collections['Trees'].objects,key=lambda o:o.name)):
    x,y=obj['reference_xy']
    obj.data=pine_variants[0 if y<190 else 1+index%2]

# Revisar tambien el ancho real de los bordes de calzada, no solo sus ejes.
kd=KDTree(len(road_points))
for index,p in enumerate(road_points):kd.insert(p,index)
kd.balance()
tree_road_adjustments=[]
for tree in collections['Trees'].objects:
    canopy=.57*max(tree.scale)
    nearest,_,distance=kd.find(tree.location)
    if distance>=canopy+.04:continue
    origin=tree.location.copy();n=origin.normalized()
    away=origin-nearest;away-=n*away.dot(n)
    away.normalize();side=n.cross(away).normalized()
    chosen=None
    for step in (.7,1.0,1.4,1.8):
        for angle in (0,.5,-.5,1.0,-1.0,1.6,-1.6,math.pi):
            direction=(origin+(away*math.cos(angle)+side*math.sin(angle))*step).normalized()
            lo=math.atan2(direction.x,-direction.y);la=math.asin(direction.z)
            x=lo/math.tau*1600+800;y=280-math.degrees(la)/.35
            point=surface(x,y)
            if kd.find(point)[2]<canopy+.10:continue
            if any((point-other.location).length<.84*(canopy+.57*max(other.scale))
                   for other in collections['Trees'].objects if other!=tree):continue
            blocked=False
            for building in buildings:
                if (point-building.location).length>6:continue
                p=building.matrix_world.inverted()@point
                xs=[v.co.x for v in building.data.vertices];ys=[v.co.y for v in building.data.vertices]
                dx=max(min(xs)-p.x,0,p.x-max(xs));dy=max(min(ys)-p.y,0,p.y-max(ys))
                if dx*dx+dy*dy<(canopy+.08)**2:blocked=True;break
            if blocked:continue
            chosen=(x,y);break
        if chosen:break
    if not chosen:raise RuntimeError('Revisar pino junto a calzada: '+tree.name)
    scale=tree.scale.copy();x,y=tree['reference_xy']
    basis=frame_at(x,y).to_3x3().inverted()@tree.matrix_world.to_3x3()
    yaw=math.atan2(basis.col[0].y,basis.col[0].x)
    place(tree,*chosen,heading=yaw);tree.scale=scale
    tree_road_adjustments.append(tree.name)

# Picos algo menos simetricos; bases y distribucion de la cordillera sin cambios.
for index,obj in enumerate(sorted(collections['Mountains'].objects,key=lambda o:o.name)):
    high=max(v.co.z for v in obj.data.vertices)
    for v in obj.data.vertices:
        if v.co.z>high-.02:
            v.co.x+=(-.12,.16,.04)[index%3]
            v.co.y+=(.08,-.05,.10)[index%3]
    obj.data.update()

assert mesh_hash(sphere.data)==sphere_hash
assert len([o for o in root.children_recursive if o.get('building_id')])==160
for name,digest in roads_hash.items():assert mesh_hash(bpy.data.objects[name].data)==digest
for name,digest in sign_hash.items():assert mesh_hash(bpy.data.objects[name].data)==digest
root['iteration']=3
root['design']='Identidad South Park: hitos, escaparates, ocho familias residenciales y nieve estilizada.'
report={'iteration':3,'buildings':160,'new_buildings':0,'sphere_geometry_unchanged':True,
        'sphere_geometry_sha256':sphere_hash,'roads_unchanged':True,'sign_geometry_unchanged':True,
        'residential_roof_families':8,'residential_corner_kits':4,'storefront_shared_kits':3,
        'pine_shared_variants':3,'adjusted_residential_positions':adjusted,
        'trees_shifted_from_road':tree_road_adjustments,
        'corner_objects':corner_objects,'storefront_objects':store_objects,
        'landmark_features':[school_feature.name,civic_feature.name,mall_feature.name,
                             pagoda_feature.name,barn_feature.name],
        'identity_note':'Escuela identificable; edificio civico y otros hitos conservan descripciones. '
                        'No se afirma identidad de ayuntamiento, comisaria ni comercios ilegibles.',
        'roof_families':roof_variants}
(DOCS/'southpark_iteration03_inventory.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('Identidad v3 aplicada: 160 edificios conservados;',len(adjusted),'ajustes leves de implantacion;',
      len(corner_objects),'casas de esquina;',len(store_objects),'escaparates.')
