"""South Park: primera interpretación del mapa sobre una esfera.

Ejecutar este archivo desde el editor de texto de Blender. Crea una escena nueva,
no altera escenas previas y no guarda ni exporta automáticamente el modelo.
Coordenadas del inventario aproximadas sobre la referencia de 1600 x 408.
"""
import math
import json
import random
from pathlib import Path
import bpy
from mathutils import Vector, Matrix

RADIUS = 22.0
SEED = 702
rng = random.Random(SEED)
REPO = Path(__file__).resolve().parents[3]
REFERENCES = REPO / 'docs' / 'references' / 'southpark'
REFERENCES.mkdir(parents=True, exist_ok=True)
scene = bpy.data.scenes.new('SouthPark_Planet')
bpy.context.window.scene = scene

def collection(name, parent=None):
    c = bpy.data.collections.new(name)
    (parent.children if parent else scene.collection.children).link(c)
    return c

collections = {}
for name in ('Planet_Surface', 'City', 'Roads', 'Mountains', 'Trees',
             'SouthPark_Sign', 'SouthPark_SnowFX', 'Presentation'):
    collections[name] = collection(name)
for name in ('Residential', 'Commercial', 'Public_Buildings', 'Landmarks'):
    collections[name] = collection(name, collections['City'])

root = bpy.data.objects.new('SouthPark_Planet_ROOT', None)
collections['Planet_Surface'].objects.link(root)
root['radius'] = RADIUS
root['design'] = 'Mapa panoramico adaptado a una esfera; materiales planos; sin particulas.'
root['export'] = 'Jerarquia de esta raiz; excluir Presentation. Sin exportacion definitiva.'

def mat(name, color):
    m = bpy.data.materials.new('SP_' + name)
    m.diffuse_color = (*color,1)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    bsdf.inputs['Base Color'].default_value = (*color,1)
    bsdf.inputs['Roughness'].default_value = 0.88
    return m

colors = {
    'Snow': (0.89,0.95,0.98), 'SnowBlue': (0.75,0.86,0.91),
    'SnowWhite': (0.97,0.98,0.96), 'Road': (0.075,0.084,0.105),
    'Marking': (0.93,0.77,0.36), 'Concrete': (0.49,0.56,0.59),
    'Roof': (0.16,0.20,0.22), 'RoofRed': (0.36,0.09,0.055),
    'Glass': (0.27,0.64,0.73), 'Frame': (0.85,0.88,0.84),
    'Door': (0.20,0.105,0.055), 'Black': (0.018,0.017,0.019),
    'Wood': (0.40,0.23,0.14), 'WoodLight': (0.51,0.31,0.20),
    'Pine': (0.035,0.14,0.075), 'PineLight': (0.055,0.22,0.12),
    'Mountain': (0.105,0.24,0.19), 'MountainBlue': (0.17,0.29,0.30),
    'Lake': (0.19,0.43,0.49), 'Red': (0.63,0.09,0.055),
    'Yellow': (0.86,0.56,0.12), 'Cream': (0.78,0.68,0.46),
    'Teal': (0.14,0.43,0.38), 'Blue': (0.22,0.36,0.58),
    'Lilac': (0.42,0.28,0.51), 'Salmon': (0.66,0.32,0.23),
    'Green': (0.27,0.49,0.20), 'Brown': (0.40,0.29,0.21),
}
materials = {name: mat(name,c) for name,c in colors.items()}
material_names = list(materials)
material_indices = {name:i for i,name in enumerate(material_names)}

def map_angles(x,y):
    return (x-800)/1600*math.tau, math.radians((280-y)*0.35)

def radial_height(lon,lat):
    return RADIUS + 0.10*math.sin(3*lon)*math.cos(lat)**2 + 0.08*math.sin(lat*3)*math.cos(lon*4)

def normal(lon,lat):
    return Vector((math.cos(lat)*math.sin(lon),-math.cos(lat)*math.cos(lon),math.sin(lat)))

def surface(x,y,offset=0):
    lon,lat=map_angles(x,y)
    return normal(lon,lat)*(radial_height(lon,lat)+offset)

def frame_at(x,y):
    lon,lat=map_angles(x,y)
    east=Vector((math.cos(lon),math.sin(lon),0))
    north=Vector((-math.sin(lat)*math.sin(lon),math.sin(lat)*math.cos(lon),math.cos(lat)))
    n=normal(lon,lat)
    return Matrix(((east.x,north.x,n.x),(east.y,north.y,n.y),(east.z,north.z,n.z))).to_4x4()

def place(obj,x,y,heading=0):
    obj.parent=root
    matrix=frame_at(x,y) @ Matrix.Rotation(heading,4,'Z')
    matrix.translation=surface(x,y)
    obj.matrix_world=matrix
    obj['reference_xy']=[x,y]
    return obj

class Geometry:
    def __init__(self):
        self.vertices=[]; self.faces=[]; self.indices=[]
    def face(self,points,material):
        start=len(self.vertices)
        self.vertices.extend(tuple(p) for p in points)
        self.faces.append(tuple(range(start,start+len(points))))
        self.indices.append(material_indices[material])
    def solid(self,points,faces,material):
        start=len(self.vertices)
        self.vertices.extend(tuple(p) for p in points)
        for f in faces:
            self.faces.append(tuple(start+i for i in f))
            self.indices.append(material_indices[material])
    def box(self,center,size,material):
        c=Vector(center); a,b,d=[v/2 for v in size]
        pts=[c+Vector((x*a,y*b,z*d)) for x,y,z in
             ((-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),
              (-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1))]
        self.solid(pts,((3,2,1,0),(0,1,5,4),(1,2,6,5),
                        (2,3,7,6),(3,0,4,7),(4,5,6,7)),material)
    def cone(self,z,radius,height,material,sides=9,rotation=0):
        pts=[(radius*math.cos(math.tau*i/sides+rotation),radius*math.sin(math.tau*i/sides+rotation),z)
             for i in range(sides)]+[(0,0,z+height)]
        faces=[tuple(reversed(range(sides)))]+[(i,(i+1)%sides,sides) for i in range(sides)]
        self.solid(pts,faces,material)
    def roof(self,w,d,z,rise,material):
        pts=[(-w/2,-d/2,z),(w/2,-d/2,z),(w/2,d/2,z),(-w/2,d/2,z),
             (0,-d/2,z+rise),(0,d/2,z+rise)]
        self.solid(pts,((0,1,4),(2,3,5),(0,4,5,3),(4,1,2,5),(3,2,1,0)),material)
    def mesh(self,name,curved=False):
        verts=self.vertices
        if curved:
            verts=[(x,y,z+math.sqrt(max(0,RADIUS**2-x*x-y*y))-RADIUS) for x,y,z in verts]
        mesh=bpy.data.meshes.new(name+'_Mesh')
        mesh.from_pydata(verts,[],self.faces);mesh.update()
        used=sorted(set(self.indices))
        remap={old:new for new,old in enumerate(used)}
        for i in used:mesh.materials.append(materials[material_names[i]])
        for p,i in zip(mesh.polygons,self.indices):p.material_index=remap[i]
        return mesh
    def object(self,name,group,curved=False):
        obj=bpy.data.objects.new(name,self.mesh(name,curved))
        collections[group].objects.link(obj)
        obj.parent=root
        return obj

# FASE A: esfera, elevaciones sutiles y base nevada real.
bpy.ops.mesh.primitive_uv_sphere_add(segments=96,ring_count=64,radius=RADIUS)
planet=bpy.context.object
planet.name='Planet_Surface_Snow'
for c in list(planet.users_collection):c.objects.unlink(planet)
collections['Planet_Surface'].objects.link(planet)
planet.parent=root
for m in ('Snow','SnowBlue','SnowWhite'):planet.data.materials.append(materials[m])
for v in planet.data.vertices:
    n=v.co.normalized()
    lat=math.asin(n.z);lon=math.atan2(n.x,-n.y)
    v.co=n*radial_height(lon,lat)
for p in planet.data.polygons:
    p.use_smooth=True
    center=sum((planet.data.vertices[i].co for i in p.vertices),Vector())/len(p.vertices)
    p.material_index=1 if center.z < -10 else (2 if center.z>14 else 0)

# Inventario: cada entrada representa una construccion visible o un volumen principal,
# no una identificacion nominal cuando el mapa no permite leer un nombre.
inventory=[]
def entry(label,x,y,kind='house',sector='residential',palette=None):
    inventory.append(dict(id=f'SP_B{len(inventory)+1:03d}',label=label,x=x,y=y,
                          kind=kind,sector=sector,palette=palette))

for index,x in enumerate((528,557,586,616,647,676,708,738,775,805,839,869,897)):
    entry('Casa del barrio central %02d'%(index+1),x,338,sector='residential_central')
for index,x in enumerate((1034,1062,1146,1175,1208,1235,1273,1305,1354,1382)):
    entry('Casa del barrio oriental %02d'%(index+1),x,338,sector='residential_east')
for index,x in enumerate((584,664,745,786,844,887)):
    entry('Casa de segunda hilera central %02d'%(index+1),x,304,sector='residential_central_back')
for index,x in enumerate((1030,1100,1146,1202,1248)):
    entry('Casa de segunda hilera oriental %02d'%(index+1),x,304,sector='residential_east_back')
for index,x in enumerate((1138,1162,1188,1210)):
    entry('Casa del sector noreste %02d'%(index+1),x,168,sector='residential_northeast')
for index,x in enumerate((286,340,405,484)):
    entry('Comercio del sector oeste %02d'%(index+1),x,180,'shop','commercial_west')
for index,x in enumerate((835,867,900,927,958,989,1018,1058,1153,1199)):
    entry('Local del corredor comercial %02d'%(index+1),x,261,'shop','commercial_center')
for index,x in enumerate((1385,1450,1482,1513)):
    entry('Comercio al noreste %02d'%(index+1),x,208,'shop','commercial_northeast')
for index,x in enumerate((1290,1320,1390,1420,1450,1480)):
    entry('Comercio del corredor oriental %02d'%(index+1),x,260,'shop','commercial_east')

entry('Escuela amarilla y roja del suroeste',235,329,'school','landmarks','Yellow')
entry('Edificio rectangular de varias plantas del oeste',416,245,'office','public','Salmon')
entry('Construccion ancha junto a la escuela',390,321,'garage','public','Salmon')
entry('Edificio civico con cupula',596,246,'civic','landmarks','Cream')
entry('Gran edificio amarillo con estacionamiento',850,160,'mall','landmarks','Yellow')
entry('Torre pagoda roja del sector oeste',450,159,'pagoda','landmarks','Red')
entry('Granero rojo del oeste',242,242,'barn','landmarks','Red')
entry('Pabellon pequeno junto al edificio civico',655,250,'pavilion','public','Teal')
entry('Edificio naranja del corredor noreste',1324,201,'restaurant','commercial_northeast','Yellow')

# FASE B: calles continuas que siguen la esfera, con bordes blancos y marcas.
paths=[
    ('North_Avenue',[(235,202),(1266,202),(1266,228),(1540,228),(1540,281)]),
    ('Central_Avenue',[(145,284),(1420,284),(1420,360)]),
    ('Residential_Avenue',[(185,360),(1454,360)]),
    ('West_Link',[(185,284),(185,360)]),
    ('Civic_Link',[(475,202),(475,360)]),
    ('East_Link',[(930,284),(930,360)]),
    ('Eastern_Link',[(1266,228),(1266,284)]),
    ('Back_Road',[(1454,360),(1520,360),(1570,323),(1615,280),(1700,280),(1745,284)]),
    ('Town_Entrance',[(680,202),(680,165)]),
]
road_data=[]
def ribbon(points,width,offset,geom,material):
    for a,b in zip(points,points[1:]):
        va,vb=Vector(a),Vector(b)
        count=max(2,int((vb-va).length/3))
        previous=None
        for i in range(count+1):
            p=va.lerp(vb,i/count)
            c=surface(p.x,p.y,offset)
            tangent=(surface(p.x+(vb.x-va.x)*0.001,p.y+(vb.y-va.y)*0.001,offset)-c).normalized()
            across=c.normalized().cross(tangent).normalized()*width/2
            pair=[]
            for edge in (c-across,c+across):
                n=edge.normalized();lat=math.asin(n.z);lon=math.atan2(n.x,-n.y)
                pair.append(n*(radial_height(lon,lat)+offset))
            if previous:
                geom.face([previous[0],previous[1],pair[1],pair[0]],material)
            previous=pair
for index,(name,points) in enumerate(paths):
    g=Geometry()
    ribbon(points,1.12,0.035+index*0.001,g,'SnowWhite')
    ribbon(points,0.86,0.048+index*0.001,g,'Road')
    for a,b in zip(points,points[1:]):
        va,vb=Vector(a),Vector(b)
        count=max(1,int((vb-va).length/18))
        for j in range(count):
            p1=va.lerp(vb,(j+0.15)/count);p2=va.lerp(vb,(j+0.52)/count)
            ribbon([p1,p2],0.045,0.065+index*0.001,g,'Marking')
    g.object('Road_'+name,'Roads')
    road_data.append(dict(name=name,reference_points=points))

def patch(x,y,width,height,material,offset=0.06,name='Patch',group='Roads'):
    g=Geometry()
    nx=max(2,int(width/4));ny=max(2,int(height/4))
    for i in range(nx):
        for j in range(ny):
            x0=x-width/2+width*i/nx;x1=x-width/2+width*(i+1)/nx
            y0=y-height/2+height*j/ny;y1=y-height/2+height*(j+1)/ny
            g.face([surface(x0,y0,offset),surface(x1,y0,offset),
                    surface(x1,y1,offset),surface(x0,y1,offset)],material)
    return g.object(name,group)

patch(850,185,204,30,'Road',name='Mall_Curved_Parking')
for x in range(767,946,16):
    g=Geometry();ribbon([(x,177),(x,190)],0.035,0.082,g,'Marking')
    g.object('Parking_Bay_%d'%x,'Roads')

def windows(g,w,d,h,levels=1,columns=2):
    for level in range(levels):
        z=0.65+(h-0.8)*(level+0.4)/levels
        for column in range(columns):
            x=(-0.5+(column+0.5)/columns)*w*0.86
            if levels==1 and abs(x)<0.22:
                continue
            ww=min(0.32,w/columns*0.52)
            g.box((x,-d/2-0.018,z),(ww+0.07,0.055,0.44),'Frame')
            g.box((x,-d/2-0.052,z),(ww,0.022,0.36),'Glass')
    g.box((0,-d/2-0.032,0.39),(0.30,0.055,0.72),'Frame')
    g.box((0,-d/2-0.07,0.37),(0.23,0.025,0.66),'Door')
    # Ventanas posteriores y laterales: descubrir detalles al rotar el planeta.
    for side in (-1,1):
        g.box((side*w/2,0,h*0.60),(0.04,d*0.34,0.43),'Frame')
        g.box((side*(w/2+0.025),0,h*0.60),(0.02,d*0.29,0.35),'Glass')
        g.box((side*w*0.27,d/2+0.015,h*0.60),(0.32,0.04,0.40),'Glass')

house_cache={}
palettes=('Teal','Blue','Lilac','Salmon','Green','Brown','Cream','Red')
def house_mesh(style):
    if style in house_cache:return house_cache[style]
    g=Geometry()
    w=1.55+(style%3)*0.12;d=1.25+(style%2)*0.15;h=1.12+(style%3)*0.12
    g.box((0,0,0.02),(w+0.12,d+0.12,0.16),'Concrete')
    g.box((0,0,h/2+0.06),(w,d,h),palettes[style])
    windows(g,w,d,h+0.06,columns=3)
    g.roof(w+0.23,d+0.2,h+0.06,0.62,'RoofRed' if style%3==0 else 'Roof')
    g.roof(w+0.26,d+0.23,h+0.14,0.62,'SnowWhite')
    # Pequeno porche frontal y chimenea integrados en la misma malla.
    g.box((0,-d/2-0.20,0.04),(0.55,0.42,0.12),'SnowBlue')
    g.box((w*0.24,d*0.18,h+0.55),(0.22,0.23,0.73),'Brown')
    g.box((w*0.24,d*0.18,h+0.94),(0.28,0.29,0.08),'SnowWhite')
    mesh=g.mesh('House_Style_%02d'%style,curved=True)
    house_cache[style]=mesh
    return mesh

def building(item,index):
    kind=item['kind'];g=Geometry()
    color=item['palette'] or palettes[index%len(palettes)]
    if kind=='house':
        mesh=house_mesh(index%8)
        obj=bpy.data.objects.new(item['id']+'_House',mesh)
        collections['Residential'].objects.link(obj)
        place(obj,item['x'],item['y'])
        if item['sector']=='residential_northeast':obj.scale=(0.80,0.80,0.80)
        item['dimensions']='casa simplificada, techo inclinado, nieve, ventanas y porche'
    else:
        dimensions={'shop':(1.7,1.3,1.25),'school':(5.2,1.7,2.0),
                    'office':(3.7,1.65,2.8),'garage':(3.0,1.5,1.15),
                    'civic':(2.3,1.8,1.55),'mall':(8.4,2.4,1.85),
                    'pagoda':(1.1,1.1,3.7),'barn':(1.7,1.7,1.8),
                    'pavilion':(1.3,1.0,0.95),'restaurant':(3.1,1.6,1.55)}
        w,d,h=dimensions[kind]
        g.box((0,0,0.03),(w+0.22,d+0.22,0.15),'Concrete')
        g.box((0,0,h/2+0.08),(w,d,h),color)
        if kind=='pagoda':
            # Torre roja escalonada: silueta visible directamente en la referencia.
            for level in range(4):
                z=0.85+level*0.85
                g.box((0,0,z),(1.45,1.45,0.13),'Red')
                g.roof(1.7,1.7,z+0.08,0.25,'RoofRed')
                for a in (-0.38,0.38):g.box((a,-0.57,z+0.36),(0.16,0.08,0.40),'Yellow')
            g.roof(1.9,1.9,h+0.15,0.43,'RoofRed')
            g.roof(1.95,1.95,h+0.20,0.43,'SnowWhite')
        elif kind=='civic':
            g.roof(w+0.35,d+0.3,h+0.06,0.36,'Roof')
            for x in (-0.7,0.7):g.box((x,-d/2-0.18,0.79),(0.16,0.22,1.55),'Frame')
            # Dome low-poly reel, visible depuis tous les angles.
            for ring in range(5):
                z=h+0.3+ring*0.15
                radius=0.60*math.sqrt(max(0,1-(ring/5)**2))
                g.cone(z,radius,0.17,'Concrete',sides=12)
            g.box((0,0,h+1.22),(0.07,0.07,0.65),'Roof')
        elif kind=='barn':
            g.roof(w+0.16,d+0.2,h+0.08,1.1,'RoofRed')
            g.roof(w+0.19,d+0.23,h+0.16,1.1,'SnowWhite')
            g.box((0,-d/2-0.03,0.80),(0.92,0.05,1.4),'Yellow')
            for x in (-0.4,0,0.4):g.box((x,-d/2-0.07,0.8),(0.045,0.02,1.40),'Brown')
        elif kind=='pavilion':
            g.roof(w+0.3,d+0.3,h+0.08,0.5,'Roof')
            g.roof(w+0.33,d+0.33,h+0.16,0.5,'SnowWhite')
            windows(g,w,d,h,columns=2)
        else:
            g.box((0,0,h+0.12),(w+0.20,d+0.20,0.18),'Roof')
            g.box((0,0,h+0.24),(w+0.15,d+0.15,0.09),'SnowWhite')
            windows(g,w,d,h,levels=2 if kind in ('school','office') else 1,
                    columns=8 if kind=='mall' else (7 if kind=='school' else 4))
            if kind=='school':
                for x in (-w/2+0.2,w/2-0.2):g.box((x,-d/2-0.04,h*0.55),(0.3,0.16,h),'Red')
                g.box((0,-d/2-0.10,h*0.86),(w-0.65,0.16,0.28),'Red')
            if kind=='garage':
                for x in (-0.85,0,0.85):g.box((x,-d/2-0.03,0.57),(0.65,0.03,0.90),'Roof')
            if kind in ('shop','restaurant'):
                g.box((0,-d/2-0.25,h*0.64),(w+0.08,0.56,0.10),'RoofRed' if index%2 else 'Teal')
                for x in (-w*0.3,w*0.3):g.box((x,-d/2-0.06,h*0.81),(w*0.30,0.07,0.18),'Cream')
        group='Landmarks' if item['sector']=='landmarks' else ('Public_Buildings' if item['sector']=='public' else 'Commercial')
        obj=g.object(item['id']+'_'+kind.title(),group,curved=True)
        place(obj,item['x'],item['y'])
        item['dimensions']=[w,d,h]
    obj['reference_description']=item['label']
    obj['reference_sector']=item['sector']
    obj['building_id']=item['id']
    item['object']=obj.name
    return obj

for index,item in enumerate(inventory):building(item,index)
print('Ciudad: construcciones',len(inventory),'y',len(house_cache),'mallas residenciales compartidas.')

# FASE C: lago occidental, montanas, bosque y cartel tridimensional.
g=Geometry()
lake_x,lake_y=78,248
segments=40
center=surface(lake_x,lake_y,0.035)
for i in range(segments):
    a=math.tau*i/segments;b=math.tau*(i+1)/segments
    r1=1+0.1*math.sin(a*5);r2=1+0.1*math.sin(b*5)
    g.face([center,surface(lake_x+58*math.cos(a)*r1,lake_y+14*math.sin(a)*r1,0.035),
            surface(lake_x+58*math.cos(b)*r2,lake_y+14*math.sin(b)*r2,0.035)],'Lake')
g.object('West_Lake','Planet_Surface')

mountain_positions=[]
for index,x in enumerate(range(60,1570,110)):
    y=110+rng.uniform(-10,7)
    h=rng.uniform(3.7,6.7);base=rng.uniform(2.0,2.8)
    g=Geometry();sides=7
    # Frontera de nieve irregular entre ladera y pico, sin un fondo plano.
    bottom=[];snowline=[]
    for side in range(sides):
        a=math.tau*side/sides
        bottom.append((base*math.cos(a),base*math.sin(a),-0.12))
        snowline.append((base*0.30*math.cos(a),base*0.30*math.sin(a),h*(0.61+rng.uniform(-0.08,0.08))))
    peak=(base*0.08,-base*0.12,h)
    for side in range(sides):
        k=(side+1)%sides
        g.face([bottom[side],bottom[k],snowline[k],snowline[side]],'Mountain' if index%3 else 'MountainBlue')
        g.face([snowline[side],snowline[k],peak],'SnowWhite')
    obj=g.object('Mountain_%02d'%(index+1),'Mountains',curved=True)
    place(obj,x,y)
    mountain_positions.append(dict(object=obj.name,x=x,y=y,height=h))

# Malla de pino compartida: copas conicas, tronco y cubierta de nieve.
pine=Geometry()
pine.box((0,0,0.36),(0.13,0.13,0.75),'Wood')
pine.cone(0.35,0.57,1.12,'Pine',sides=8)
pine.cone(0.79,0.43,0.96,'PineLight',sides=8)
pine.cone(1.16,0.31,0.82,'Pine',sides=8)
pine.cone(1.28,0.28,0.73,'SnowWhite',sides=8)
pine_mesh=pine.mesh('Pine_Shared')

def distance_to_segment(px,py,a,b):
    p=Vector((px,py));a=Vector(a);b=Vector(b)
    t=max(0,min(1,(p-a).dot(b-a)/(b-a).length_squared))
    return (p-(a+(b-a)*t)).length

tree_records=[]
attempt=0
while len(tree_records)<220 and attempt<10000:
    attempt+=1
    x=rng.uniform(10,1590)
    # Bosque montanoso, bandas urbanas y algunas zonas del hemisferio sur.
    band=rng.choices(['north','city','south'],[0.42,0.43,0.15])[0]
    y=rng.uniform(132,191) if band=='north' else (rng.uniform(210,385) if band=='city' else rng.uniform(392,465))
    if any(abs(x-item['x'])<20 and abs(y-item['y'])<12 for item in inventory):continue
    if abs(x-850)<115 and 166<y<205:continue
    if abs(x-lake_x)<70 and abs(y-lake_y)<22:continue
    if any(distance_to_segment(x,y,a,b)<9 for _,points in paths for a,b in zip(points,points[1:])):continue
    pos=surface(x,y)
    if any((pos-Vector(t['position'])).length<0.94 for t in tree_records):continue
    obj=bpy.data.objects.new('Pine_%03d'%(len(tree_records)+1),pine_mesh)
    collections['Trees'].objects.link(obj)
    place(obj,x,y,rng.uniform(-math.pi,math.pi))
    size=rng.uniform(0.62,1.04)
    obj.scale=(size,size,size)
    tree_records.append(dict(object=obj.name,x=x,y=y,position=list(pos)))

sign_anchor=bpy.data.objects.new('SouthPark_Sign',None)
collections['SouthPark_Sign'].objects.link(sign_anchor)
place(sign_anchor,650,164)
sign_anchor.scale=(1.35,1.35,1.35)
sign_anchor['reference']='Cartel de madera con dos lineas SOUTH / PARK, geometria real.'
sign=Geometry()
sign.box((0,0,1.08),(0.23,0.27,2.30),'Wood')
# Tablas ligeramente oblicuas; cartel independiente de la superficie.
for row in range(3):
    z=2.08+row*0.46
    sign.solid([(-1.47,-0.11,z-0.24),(1.47,-0.11,z-0.08),
                (1.47,-0.11,z+0.37),(-1.47,-0.11,z+0.21),
                (-1.47,0.11,z-0.24),(1.47,0.11,z-0.08),
                (1.47,0.11,z+0.37),(-1.47,0.11,z+0.21)],
               ((3,2,1,0),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,6,7)),
               'WoodLight' if row%2 else 'Wood')
obj=sign.object('SouthPark_Sign_Board_and_Post','SouthPark_Sign')
obj.parent=sign_anchor;obj.matrix_parent_inverse=Matrix.Identity(4);obj.matrix_basis=Matrix.Identity(4)

font_path=Path('C:/Windows/Fonts/ROCKEB.TTF')
font=bpy.data.fonts.load(str(font_path)) if font_path.exists() else None
def text_mesh(name,body,size,location,orientation,parent,group,font_override=None):
    data=bpy.data.curves.new(name+'_Text','FONT')
    data.body=body;data.align_x='CENTER';data.align_y='CENTER';data.size=size
    data.extrude=0.006;data.resolution_u=3
    if font_override:data.font=font_override
    obj=bpy.data.objects.new(name,data);collections[group].objects.link(obj)
    data.materials.append(materials['Black'])
    obj.parent=parent;obj.location=location;obj.rotation_euler=orientation
    bpy.context.view_layer.update()
    mesh=bpy.data.meshes.new_from_object(obj.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    mesh_obj=bpy.data.objects.new(name+'_Mesh',mesh);collections[group].objects.link(mesh_obj)
    mesh_obj.parent=parent;mesh_obj.matrix_basis=obj.matrix_basis.copy()
    bpy.data.objects.remove(obj,do_unlink=True)
    bpy.data.curves.remove(data)
    return mesh_obj

for side in (-1,1):
    for body,z in (('SOUTH',2.88),('PARK',2.22)):
        # +Y mirando al norte y -Y mirando al sur; texto legible desde ambos lados.
        orientation=(math.pi/2,0,0 if side==-1 else math.pi)
        obj=text_mesh('Sign_%s_%s'%(body,'Front' if side==1 else 'Back'),body,0.60,
                      (0,side*0.128,z),orientation,sign_anchor,'SouthPark_Sign',font)
        obj.scale.x=0.96

# Rotulos discretos solo para hitos reconocibles por su forma en la referencia.
school=next(o for o in root.children_recursive if o.get('reference_description')=='Escuela amarilla y roja del suroeste')
text_mesh('School_Label','SCHOOL',0.28,(0,-0.966,1.79),(math.pi/2,0,0),school,'Landmarks')
if font:
    # Todo el texto ya es malla; no conservar una dependencia de la fuente local.
    bpy.data.fonts.remove(font)

# FASE D: vista de inspeccion independiente, sin efectos cinematograficos.
world=bpy.data.worlds.new('SouthPark_World')
world.use_nodes=True
bg=next(n for n in world.node_tree.nodes if n.type=='BACKGROUND')
bg.inputs['Color'].default_value=(0.018,0.036,0.055,1)
bg.inputs['Strength'].default_value=0.65
scene.world=world
camera_data=bpy.data.cameras.new('SouthPark_Overview')
camera=bpy.data.objects.new('SouthPark_Overview',camera_data)
collections['Presentation'].objects.link(camera)
camera.location=(-37,-77,28)
camera.rotation_euler=(-camera.location).to_track_quat('-Z','Y').to_euler()
camera_data.type='ORTHO';camera_data.ortho_scale=65
scene.camera=camera
for name,position,power,size in (
    ('Key',(-30,-42,60),10500,32),
    ('Fill',(35,-28,12),8500,30),
    ('Rim',(10,30,45),11000,28),
    ('Front',(-40,-42,-10),6000,28),
):
    data=bpy.data.lights.new('SouthPark_'+name,'AREA')
    data.energy=power;data.size=size
    obj=bpy.data.objects.new('SouthPark_'+name,data)
    collections['Presentation'].objects.link(obj);obj.location=position
    obj.rotation_euler=(-obj.location).to_track_quat('-Z','Y').to_euler()
scene.render.resolution_x=1400;scene.render.resolution_y=1400
scene.view_settings.exposure=1.2
scene.render.resolution_percentage=100
scene.render.filepath='//../../../../docs/references/southpark/southpark_three_quarter.png'
for area in bpy.context.screen.areas:
    if area.type=='VIEW_3D':
        space=area.spaces.active;space.shading.type='MATERIAL'
        space.region_3d.view_rotation=camera.rotation_euler.to_quaternion()
        space.region_3d.view_distance=83
        space.region_3d.view_location=(0,0,0)
        space.overlay.show_floor=False;space.overlay.show_axis_x=False;space.overlay.show_axis_y=False
for obj in bpy.context.selected_objects:obj.select_set(False)
root.select_set(True);bpy.context.view_layer.objects.active=root

report={
    'radius':RADIUS,'seed':SEED,'buildings':inventory,'building_count':len(inventory),
    'houses':sum(i['kind']=='house' for i in inventory),
    'trees':len(tree_records),'mountains':mountain_positions,'roads':road_data,
    'coverage_note':'Objetivo de cobertura aproximada del mapa: se priorizan los volumenes principales. '
                    'La imagen no permite confirmar un denominador preciso ni un porcentaje exacto; '
                    'el inventario enumera construcciones representadas, no anexos ni decoracion.',
    'generic_labels':True,
    'map_transform':'longitud=(x-800)/1600*360; latitud=(280-y)*0.35 grados; normal radial local.',
    'omitted':'Edificios minimos de los extremos, anexos ambiguos, detalles interiores, vehiculos, '
              'rotulos ilegibles, limites exactos de parcelas y el simbolo de Canada al extremo superior derecho.',
}
(REFERENCES/'southpark_inventory.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print('South Park listo:',len(inventory),'edificios;',len(tree_records),'pinos;',len(mountain_positions),'montanas.')
