"""Puesto de snacks, caravana y taller a partir de las tres referencias del usuario."""
import bpy
import os
import math
import random
import importlib.util
from mathutils import Vector
from math import sin, cos, pi

ROOT=os.path.dirname(os.path.abspath(__file__))
REPO=os.path.dirname(os.path.dirname(os.path.dirname(ROOT)))
REFS=os.path.join(REPO,'docs','references','regularshow')
spec=importlib.util.spec_from_file_location('domo_zonas_helpers',os.path.join(ROOT,'crear_domo.py'))
D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D)
Mesh,box,tube,lathe,ring,mat=D.Mesh,D.box,D.tube,D.lathe,D.ring,D.mat
S=.18
ZONES=[('Snacks',(-3.65,-2.75),(1.55,1.20)),
       ('Caravana',(4.05,3.50),(1.45,1.00)),
       ('Taller',(3.35,-3.25),(1.35,1.20))]

def inside_zone(x,y,margin=.22):
    return any(((x-c[0])/(r[0]+margin))**2+((y-c[1])/(r[1]+margin))**2<1 for _,c,r in ZONES)

def cylinder(mesh,center,r,h,mi=0,segments=24):
    lathe(mesh,[(0,-h/2),(r,-h/2),(r,h/2),(0,h/2)],center,mi,segments)

def wheel(mesh,center,r,width,mi=0,axis=(0,1,0)):
    tmp=Mesh();lathe(tmp,[(0,-width/2),(r*.80,-width/2),(r,-width*.32),(r,width*.32),(r*.80,width/2),(0,width/2)],mi=mi,segments=32)
    n=Vector(axis).normalized();u=n.cross(Vector((0,0,1))).normalized();v=n.cross(u)
    mesh.add([Vector(center)+u*p[0]+v*p[1]+n*p[2] for p in tmp.v],tmp.f,mi,True)

def local_object(mesh,name,c,mats,origin,angle=0):
    ob=mesh.object(name,c,mats)
    ca=cos(angle);sa=sin(angle)
    for p in ob.data.vertices:
        x,y,z=p.co
        p.co=(origin[0]+S*(x*ca-y*sa),origin[1]+S*(x*sa+y*ca),origin[2]+S*z)
    ob.parent=bpy.data.objects['RegularShow_Planet_ROOT']
    return ob

def label(text,name,c,origin,position,size,material,angle=0):
    typ=next(i.identifier for i in bpy.data.curves.bl_rna.functions['new'].parameters['type'].enum_items if i.identifier=='FONT')
    curve=bpy.data.curves.new(name,typ);curve.body=text;curve.size=size*S;curve.extrude=.008*S;curve.resolution_u=4
    D.enum(curve,'align_x','CENTER')
    ob=bpy.data.objects.new(name,curve);c.objects.link(ob)
    x,y,z=position
    ob.location=(origin[0]+S*(x*cos(angle)-y*sin(angle)),origin[1]+S*(x*sin(angle)+y*cos(angle)),origin[2]+S*z)
    ob.rotation_euler=(pi/2,0,angle)
    curve.materials.append(material)
    bpy.context.view_layer.update()
    mesh=bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    converted=bpy.data.objects.new(name+' malla',mesh);c.objects.link(converted)
    converted.matrix_world=ob.matrix_world.copy()
    bpy.data.objects.remove(ob,do_unlink=True);converted.name=name;ob=converted
    if curve.users==0:bpy.data.curves.remove(curve)
    ob.parent=bpy.data.objects['RegularShow_Planet_ROOT']
    return ob

def palette():
    return [mat('ZONAS | Arena clara',(.61,.54,.31)),
            mat('ZONAS | Amarillo del quiosco',(.63,.48,.22)),
            mat('ZONAS | Marfil',(.78,.79,.69)),
            mat('ZONAS | Verde oscuro',(.055,.16,.135)),
            mat('ZONAS | Madera de mesas',(.40,.26,.09)),
            mat('ZONAS | Metal de sillas',(.09,.115,.095),.54,.25),
            mat('ZONAS | Rojo oxido',(.36,.075,.045)),
            mat('ZONAS | Ventanas',(.055,.14,.15),.26,.12),
            mat('ZONAS | Aluminio claro',(.59,.65,.63),.50,.30),
            mat('ZONAS | Neumaticos',(.022,.029,.026)),
            mat('ZONAS | Papel blanco',(.88,.865,.73)),
            mat('ZONAS | Piedra gris',(.42,.45,.40)),
            mat('ZONAS | Verde del surtidor',(.30,.46,.38)),
            mat('ZONAS | Paredes del taller',(.47,.45,.34)),
            mat('ZONAS | Sillas de tela azul',(.22,.43,.46)),
            mat('ZONAS | Juntas oscuras',(.13,.15,.12))]

def patches(mats):
    c=D.coll('09 | Nuevos claros y accesos',D.coll(D.MODEL))
    for ob in list(c.objects):bpy.data.objects.remove(ob,do_unlink=True)
    for name,center,radii in ZONES:
        m=Mesh();n=120;rows=12;verts=[]
        for j in range(rows+1):
            for i in range(n):
                a=D.TAU*i/n;rr=j/rows*(1+.018*sin(5*a)+.014*cos(9*a))
                x=center[0]+radii[0]*cos(a)*rr;y=center[1]+radii[1]*sin(a)*rr
                verts.append((x,y,D.ground(x,y)+.032))
        faces=[]
        for j in range(rows):
            for i in range(n):faces.append((j*n+i,(j+1)*n+i,(j+1)*n+(i+1)%n,j*n+(i+1)%n))
        m.add(verts,faces,0,True)
        ob=m.object('ZONA | Claro de '+name,c,[mats[0]]);ob.parent=bpy.data.objects['RegularShow_Planet_ROOT']
        if hasattr(ob,'visible_shadow'):ob.visible_shadow=False

def relocate_trees():
    """Mueve árboles completos manteniendo sus tres copas y ramas correspondientes."""
    wood=bpy.data.objects['DOMO | Troncos y ramas']
    canopy=next(o for o in bpy.data.objects if o.name.startswith('DOMO | Copas de '))
    assert len(wood.data.vertices)==340*38,'La vegetacion fue editada; no se aplican offsets sin verificar.'
    expected=sum(50 if i%13==0 else 288 for i in range(340))
    assert len(canopy.data.vertices)==expected,'Orden de copas inesperado.'
    centers=[]
    for i in range(340):
        p=sum((wood.data.vertices[i*38+j].co for j in range(7)),Vector())/7
        centers.append(p.copy())
    rng=random.Random(5026);changed=0;offset=0
    for i,p in enumerate(centers):
        count=50 if i%13==0 else 288
        if inside_zone(p.x,p.y,.30):
            for _ in range(20000):
                a=rng.uniform(0,D.TAU);r=rng.uniform(3.0,6.55);x=r*cos(a);y=r*sin(a)
                if inside_zone(x,y,.38):continue
                if (x+.9)**2+(y-1.65)**2<2.0**2:continue
                if (x-2.75)**2+(y-1.75)**2<1.05**2:continue
                if (x-1.4)**2+(y+.05)**2<.55**2:continue
                if (x+3.6)**2+(y-2.9)**2<.65**2:continue
                if abs((x/4.65)**2+((y+.25)/3.6)**2-1)<.07:continue
                if min((x-q.x)**2+(y-q.y)**2 for j,q in enumerate(centers) if j!=i)<.20**2:continue
                delta=Vector((x,y,D.ground(x,y)))-p
                for j in range(i*38,(i+1)*38):wood.data.vertices[j].co+=delta
                for j in range(offset,offset+count):canopy.data.vertices[j].co+=delta
                centers[i]+=delta;changed+=1;break
            else:raise RuntimeError('No hay un lugar libre para reubicar un arbol.')
        offset+=count
    wood.data.update();canopy.data.update()
    shrubs=bpy.data.objects['DOMO | Setos y arbustos bajos']
    if len(shrubs.data.vertices)==95*60:
        for i in range(95):
            points=[shrubs.data.vertices[j].co for j in range(i*60,(i+1)*60)]
            p=sum(points,Vector())/60
            if inside_zone(p.x,p.y,.10):
                for _ in range(1000):
                    a=rng.uniform(0,D.TAU);r=rng.uniform(6.25,6.7);x=r*cos(a);y=r*sin(a)
                    if not inside_zone(x,y,.2):break
                delta=Vector((x,y,D.ground(x,y)+.08))-p
                for j in range(i*60,(i+1)*60):shrubs.data.vertices[j].co+=delta
        shrubs.data.update()
    print('Arboles reubicados:',changed,'; se conservan los 340 arboles.')

def snacks(mats):
    c=D.coll('10 | Snacks y terraza de mesas',D.coll(D.MODEL))
    for ob in list(c.objects):bpy.data.objects.remove(ob,do_unlink=True)
    origin=(-4.13,-1.96,D.ground(-4.13,-1.96)+.028)
    building=Mesh();trim=Mesh();furniture=Mesh()
    box(building,(0,0,1.60),(4.80,2.45,3.20),1)
    box(building,(0,0,3.25),(5.15,2.75,.18),3)
    box(trim,(0,-1.27,2.02),(3.87,.09,1.31),11)
    box(trim,(0,-1.335,2.02),(3.58,.045,1.04),7)
    box(trim,(0,-1.66,1.40),(4.12,.75,.14),4)
    for x in (-1.97,1.97):box(trim,(x,-1.35,2.03),(.12,.12,1.49),2)
    for z in (1.35,2.71):box(trim,(0,-1.35,z),(4.05,.12,.12),2)
    for x in (-1.1,0,1.1):
        box(trim,(x,-1.42,1.64),(.50,.30,.29),1)
        cylinder(trim,(x+.15,-1.50,1.89),.11,.17,6,16)
    # Cartel volumétrico, incluido como malla en el OBJ.
    box(building,(0,-1.18,3.83),(5.32,.25,1.14),2)
    for x in (-2.70,2.70):box(trim,(x,-1.19,3.82),(.10,.29,1.23),4)
    for z in (3.23,4.43):box(trim,(0,-1.19,z),(5.48,.29,.09),4)
    local_object(building,'SNACKS | Quiosco y cartel',c,mats,origin)
    local_object(trim,'SNACKS | Mostrador y marcos de atencion',c,mats,origin)
    label('SNACKS','SNACKS | Letras del cartel',c,origin,(0,-1.322,3.47),.77,mats[6])
    label('BURGERS','SNACKS | Letras bajo el mostrador',c,origin,(0,-1.28,.49),.30,mats[3])
    # Siete mesas y veinticuatro sillas de hierro con respaldo ovalado.
    table_centers=[(-3.25,-4.2),(0,-4.0),(3.15,-3.8),(-3.9,-7.0),(-.3,-7.2),(3.4,-6.6),(2.4,-9.1)]
    for j,(x,y) in enumerate(table_centers):
        cylinder(furniture,(x,y,1.12),1.05,.13,4,40)
        cylinder(furniture,(x,y,.58),.10,1.04,5,14)
        for k in range(4):
            a=k*pi/2+.2;tube(furniture,[(x,y,.18),(x+.65*cos(a),y+.65*sin(a),.08)],.055,5,6)
        for k in range(3 if j<4 else 4):
            a=D.TAU*k/(3 if j<4 else 4)+j*.25
            cx=x+1.58*cos(a);cy=y+1.58*sin(a)
            cylinder(furniture,(cx,cy,.70),.40,.08,4,24)
            for dx,dy in ((-.25,-.25),(.25,-.25),(-.25,.25),(.25,.25)):
                tube(furniture,[(cx+dx,cy+dy,.05),(cx+dx*.85,cy+dy*.85,.72)],.038,5,6)
            back=Vector((cos(a),sin(a),0));side=Vector((-sin(a),cos(a),0))
            p=Vector((cx,cy,.72))+back*.27
            arc=[]
            for q in range(24):
                t=-pi/2+D.TAU*q/24
                arc.append(p+side*(.30*cos(t))+Vector((0,0,.53+.61*sin(t))))
            tube(furniture,arc,.032,5,6,True)
            tube(furniture,[p+Vector((0,0,.05)),p+Vector((0,0,1.10))],.028,5,6)
    # Papeleras verdes, una junto al kiosco y otra cerca de la terraza.
    for x,y in ((-4.6,-2.2),(4.7,-5.4)):
        box(furniture,(x,y,.65),(.71,.70,1.25),12)
        box(furniture,(x,y,1.36),(.78,.77,.17),11)
        box(furniture,(x,y-.39,1.29),(.48,.025,.22),7)
    local_object(furniture,'SNACKS | Siete mesas, sillas y papeleras',c,mats,origin)
    print('Quiosco SNACKS y terraza terminados.')

def caravana(mats):
    c=D.coll('11 | Caravana y campamento',D.coll(D.MODEL))
    for ob in list(c.objects):bpy.data.objects.remove(ob,do_unlink=True)
    origin=(4.05,3.50,D.ground(4.05,3.50)+.035)
    body=Mesh();trim=Mesh();details=Mesh();truck=Mesh()
    profile=[(-3.83,.66),(3.30,.66),(3.83,.98),(4.05,1.65),(4.03,2.74),(3.67,3.47),(3.02,3.79),(-2.92,3.79),(-3.60,3.52),(-3.98,2.80),(-4.03,1.49)]
    n=len(profile)
    vertices=[(x,y,z) for y in (-1.47,1.47) for x,z in profile]
    faces=[tuple(reversed(range(n))),tuple(range(n,n*2))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
    body.add(vertices,faces,2)
    for side in (-1,1):
        y=side*1.485
        box(trim,(0,y,1.64),(7.98,.035,.26),6)
        for z in (1.05,1.32,1.98,2.23,2.49,2.75,3.01,3.26,3.52):
            intersections=[]
            for a,b in zip(profile,profile[1:]+profile[:1]):
                if (a[1]<=z<b[1]) or (b[1]<=z<a[1]):intersections.append(a[0]+(b[0]-a[0])*(z-a[1])/(b[1]-a[1]))
            if len(intersections)>=2:
                lo=min(intersections);hi=max(intersections)
                box(trim,((lo+hi)/2,y*1.006,z),(hi-lo,.017,.017),8)
        tube(trim,[(x,y,z) for x,z in profile],.040,8,7,True)
    # Ventana de persianas y puerta del remolque.
    box(trim,(-2.12,-1.53,2.58),(2.28,.12,.83),8)
    box(trim,(-2.12,-1.61,2.58),(2.04,.05,.59),7)
    for z in (2.40,2.58,2.76):box(details,(-2.12,-1.66,z),(2.02,.04,.06),3)
    box(trim,(.35,-1.56,2.07),(1.42,.14,2.53),8)
    box(trim,(.35,-1.65,2.07),(1.20,.05,2.32),2)
    box(trim,(.35,-1.69,2.64),(.67,.04,.89),7)
    box(details,(-.14,-1.73,1.86),(.075,.055,.16),5)
    box(details,(.35,-1.98,.43),(1.60,.80,.25),11)
    # Ventana delantera con toldo verde y tirantes.
    box(trim,(2.68,-1.53,2.48),(2.05,.10,1.15),8)
    box(trim,(2.68,-1.60,2.48),(1.83,.045,.93),7)
    awning=[(1.43,-1.64,3.18),(3.93,-1.64,3.18),(3.93,-3.30,3.45),(1.43,-3.30,3.45)]
    body.face(awning,3)
    for i in range(8):
        x=1.43+2.50*i/7;tube(details,[(x,-1.64,3.20),(x,-3.30,3.47)],.018,8,5)
    for x in (1.47,3.89):tube(details,[(x,-1.67,1.93),(x,-3.28,3.44)],.032,8,6)
    # Chasis, apoyos de bloques y ruedas visibles.
    box(details,(0,0,.59),(6.8,2.55,.22),5)
    for x in (-2.55,2.65):
        for y in (-1.05,1.05):
            box(details,(x,y,.14),(.83,.55,.28),11)
            box(details,(x,y,.39),(.72,.49,.22),11)
    for x in (-.80,.24):
        for y in (-1.40,1.40):
            wheel(details,(x,y,.64),.58,.29,9)
            wheel(details,(x,y*1.11,.64),.25,.04,8)
    # Lanza de enganche y dos bombonas con cinturón rojo.
    for y in (-1,1):tube(details,[(3.55,y,.55),(6.12,0,.55)],.075,8,8)
    cylinder(details,(5.66,0,.33),.08,.64,8)
    for x in (4.20,5.13):
        lathe(details,[(0,.69),(.34,.69),(.47,.84),(.48,1.77),(.34,1.99),(.16,2.04),(.15,2.18),(0,2.18)],(x,-.34,0),8,32)
        cylinder(details,(x,-.34,1.20),.485,.31,6)
        ring(details,.18,2.15,.045,8,(x,-.34),32)
    # Antena de televisión, tubería y luces de posición.
    tube(details,[(-2.94,.80,3.58),(-2.94,.80,5.55)],.025,5,6)
    for z,w in ((4.87,1.10),(5.10,.92),(5.34,.72)):
        tube(details,[(-2.94-w/2,.80,z),(-2.94+w/2,.80,z)],.022,5,6)
    for x in (2.5,3.0,3.5):D.ellipsoid(details,(x,-1.38,3.48),(.095,.065,.09),6,10,6)
    # Barriles, neumáticos, caja de herramientas y silla plegable.
    for x,y in ((-5.00,-1.8),(-5.12,-.6)):
        cylinder(details,(x,y,.65),.45,1.30,5,32)
        for z in (.15,.67,1.19):ring(details,.45,z,.035,9,(x,y),32)
    for x,y,z in ((-4.80,1.0,.22),(-4.80,1.0,.64),(-4.10,1.25,.22)):
        lathe(details,[(.25,-.18),(.47,-.18),(.50,-.10),(.50,.10),(.47,.18),(.25,.18),(.25,-.18)],(x,y,z),9,32)
    box(details,(4.7,-2.2,.24),(1.05,.74,.47),4)
    tube(details,[(4.49,-2.2,.49),(4.49,-2.2,.64),(4.90,-2.2,.64),(4.90,-2.2,.49)],.045,5,6)
    cx,cy=-2.7,-4.0
    box(details,(cx,cy,.72),(1.12,1.02,.11),14)
    box(details,(cx,cy+.48,1.41),(1.12,.11,1.15),14)
    for x in (cx-.55,cx+.55):
        tube(details,[(x,cy-.55,.06),(x,cy+.50,1.94)],.038,8,7)
        tube(details,[(x,cy+.60,.06),(x,cy-.52,1.09)],.038,8,7)
        tube(details,[(x,cy-.35,1.15),(x,cy+.50,1.15)],.041,8,7)
    # Camioneta vieja, de color rojizo, al fondo del campamento.
    ox,oy=1.15,4.0
    box(truck,(ox,oy,.82),(5.4,2.18,.55),6)
    box(truck,(ox+1.12,oy,1.36),(1.8,2.13,.62),6)
    box(truck,(ox+.40,oy,1.97),(1.92,2.0,.13),6)
    for sy in (-1,1):
        box(truck,(ox+.40,oy+sy*1.00,1.66),(1.68,.045,.55),7)
        box(truck,(ox-1.75,oy+sy*1.06,1.24),(2.60,.17,.56),6)
    box(truck,(ox-1.75,oy,1.04),(2.48,1.92,.10),15)
    box(truck,(ox+1.38,oy,1.67),(.045,1.78,.45),7)
    box(truck,(ox+2.76,oy,.81),(.12,2.15,.23),8)
    for y in (oy-.70,oy+.70):box(truck,(ox+2.70,y,1.13),(.06,.42,.29),2)
    for x in (ox-1.76,ox+1.64):
        for y in (oy-1.14,oy+1.14):
            wheel(truck,(x,y,.64),.57,.32,9)
            wheel(truck,(x,y+(.19 if y>oy else -.19),.64),.23,.035,8)
    for mesh,name in ((body,'CARAVANA | Casco blanco y toldo verde'),(trim,'CARAVANA | Franja roja, puerta y ventanas'),(details,'CARAVANA | Enganche, bombonas, apoyos y mobiliario'),(truck,'CARAVANA | Camioneta antigua rojiza')):
        local_object(mesh,name,c,mats,origin)
    print('Caravana con toldo, bombonas, antena, barriles, silla y camioneta lista.')

def taller(mats):
    c=D.coll('12 | Taller y surtidor antiguo',D.coll(D.MODEL))
    for ob in list(c.objects):bpy.data.objects.remove(ob,do_unlink=True)
    origin=(3.35,-3.25,D.ground(3.35,-3.25)+.035)
    walls=Mesh();trim=Mesh();roof=Mesh();equipment=Mesh();shingles=Mesh()
    # Habitación de la derecha y garaje abierto a la izquierda.
    box(walls,(2.22,0,1.71),(4.44,4.25,3.42),13)
    box(walls,(-2.25,2.02,1.58),(4.50,.16,3.16),13)
    box(walls,(-4.44,0,1.58),(.17,4.14,3.16),13)
    box(walls,(-2.23,0,.075),(4.54,4.26,.15),11)
    for z in (.32,.71,1.10,1.49,1.88,2.27,2.66,3.05):
        box(trim,(2.22,-2.14,z),(4.45,.025,.027),15)
        box(trim,(4.46,0,z),(.025,4.25,.027),15)
        box(trim,(-4.54,0,z),(.025,4.25,.027),15)
    for x,y in ((-4.49,-2.20),(-.03,-2.20),(-4.49,2.14)):
        box(trim,(x,y,1.60),(.14,.17,3.20),2)
    box(trim,(-2.27,-2.25,3.10),(4.69,.17,.16),2)
    # Hoja de la puerta del garaje levantada y vigas interiores.
    roof.face([(-4.45,-2.26,2.99),(-.09,-2.26,2.99),(-.09,-4.02,3.17),(-4.45,-4.02,3.17)],2)
    for j in range(5):
        y=-2.27-1.75*j/4;z=3.00+.18*j/4
        tube(trim,[(-4.43,y,z),(-.10,y,z)],.023,8,6)
    for x in (-4.13,-3.1,-2.07,-1.04):box(trim,(x,0,3.15),(.10,4.13,.11),4)
    # Fachada con puerta menta y ventanas de marcos blancos.
    box(trim,(2.22,-2.19,1.43),(1.22,.10,2.43),2)
    box(trim,(2.22,-2.26,1.43),(1.02,.045,2.23),12)
    for z in (.70,1.74):box(trim,(2.22,-2.29,z),(.78,.025,.69),3)
    D.ellipsoid(trim,(1.84,-2.34,1.39),(.045,.025,.045),5,10,6)
    box(trim,(2.22,-2.56,.16),(1.50,.78,.22),11)
    for x in (.72,3.71):
        box(trim,(x,-2.20,1.82),(.92,.08,1.22),2)
        box(trim,(x,-2.26,1.82),(.70,.035,1.01),7)
        for dx in (-.19,.19):box(trim,(x+dx,-2.29,1.82),(.025,.035,1.00),2)
        box(trim,(x,-2.29,1.82),(.72,.035,.029),2)
    # Cubiertas verdes: tejado a dos aguas sobre la sala y plano sobre el garaje.
    walls.face([(-.10,-2.13,3.43),(4.53,-2.13,3.43),(2.22,-2.13,5.14)],13)
    walls.face([(-.10,2.13,3.43),(2.22,2.13,5.14),(4.53,2.13,3.43)],13)
    def panel(points,rows,cols):
        p=[Vector(v) for v in points];roof.face(p,3)
        normal=(p[1]-p[0]).cross(p[3]-p[0]).normalized()
        if normal.z<0:normal=-normal
        def at(u,v):return p[0].lerp(p[1],u).lerp(p[3].lerp(p[2],u),v)+normal*.012
        for j in range(rows+1):tube(shingles,[at(i/cols,j/rows) for i in range(cols+1)],.014,15,5)
        for j in range(rows):
            for k in range(cols):
                u=(k+(.5 if j%2 else 0))/cols
                if u<=1:tube(shingles,[at(u,j/rows),at(u,(j+1)/rows)],.010,15,5)
    panel([(-.30,-2.43,3.47),(2.22,-2.43,5.24),(2.22,2.43,5.24),(-.30,2.43,3.47)],12,14)
    panel([(2.22,-2.43,5.24),(4.73,-2.43,3.47),(4.73,2.43,3.47),(2.22,2.43,5.24)],12,14)
    panel([(-4.75,-2.45,3.22),(.06,-2.45,3.22),(.06,2.43,3.59),(-4.75,2.43,3.59)],11,16)
    tube(trim,[(-.30,-2.44,3.47),(2.22,-2.44,5.24),(4.73,-2.44,3.47)],.065,2,6)
    for x in (-4.76,.07):tube(trim,[(x,-2.46,3.24),(x,2.44,3.61)],.058,2,6)
    tube(trim,[(-4.77,-2.46,3.24),(.08,-2.46,3.24)],.065,2,6)
    # Tímpano con listones verticales y chimenea pequeña.
    for j in range(-9,10):
        x=2.22+j*.13;top=5.1-abs(x-2.22)*.74
        if top>4.24:tube(trim,[(x,-2.15,4.22),(x,-2.15,top)],.022,2,5)
    box(trim,(2.22,-2.20,2.96),(.28,.07,.35),8)
    box(trim,(.49,.90,4.74),(.50,.55,2.08),11)
    box(trim,(.49,.90,5.78),(.66,.69,.13),8)
    for j in range(6):box(trim,(.22,.80,3.95+j*.28),(.03,.22,.10),15)
    # Banco de trabajo, herramientas y mancha de aceite bajo el garaje.
    box(equipment,(-2.28,1.40,1.30),(3.60,.86,.14),4)
    for x in (-3.7,-.9):box(equipment,(x,1.42,.65),(.12,.68,1.30),5)
    box(equipment,(-1.2,1.4,1.61),(.90,.50,.48),6)
    box(equipment,(-3.35,1.42,1.50),(.49,.38,.30),8)
    for x in (-3.3,-2.9,-2.5):tube(equipment,[(x,1.89,1.7),(x,1.89,2.3)],.045,5,6)
    D.ellipsoid(equipment,(-2.10,-.35,.10),(.81,.49,.024),15,24,5)
    # Surtidor antiguo con reloj circular, manguera y boquilla.
    px,py=-3.78,-5.30
    cylinder(equipment,(px,py,.13),.47,.26,11)
    box(equipment,(px,py,1.00),(.64,.56,1.66),12)
    box(equipment,(px,py,1.89),(.80,.70,.17),8)
    cylinder(equipment,(px,py,2.17),.13,.49,8)
    wheel(equipment,(px,py,2.48),.40,.13,8,axis=(0,1,0))
    wheel(equipment,(px,py-.08,2.48),.30,.025,10,axis=(0,1,0))
    tube(equipment,[(px,py-.104,2.48),(px+.14,py-.104,2.65)],.024,5,6)
    box(equipment,(px,py-.30,1.36),(.41,.04,.43),10)
    hose=[(px+.35,py,1.75),(px+.62,py,1.61),(px+.70,py,.60),(px+.57,py,.34),(px+.37,py,.49),(px+.42,py,1.17)]
    tube(equipment,hose,.046,9,7)
    box(equipment,(px+.44,py,1.26),(.12,.12,.27),5)
    for mesh,name in ((walls,'TALLER | Edificio y garaje abierto'),(trim,'TALLER | Puertas, ventanas, aleros y chimenea'),(roof,'TALLER | Cubiertas verdes y puerta levantada'),(shingles,'TALLER | Tejas y listones'),(equipment,'TALLER | Surtidor antiguo, manguera y herramientas')):
        local_object(mesh,name,c,mats,origin)
    label('GAS','TALLER | Letras del surtidor',c,origin,(px,py-.33,1.29),.20,mats[3])
    print('Taller con garaje abierto y surtidor antiguo terminado.')

def cameras():
    stage=D.coll('90 | Presentacion espacial (no exportar)')
    setups=[('DOMO | Camara snacks',(-3.92,-2.92,.35),3.85,'Snacks_Detalle.png'),
            ('DOMO | Camara caravana',(4.05,3.48,.40),3.85,'Caravana_Detalle.png'),
            ('DOMO | Camara taller',(3.35,-3.42,.42),3.40,'Taller_Detalle.png')]
    for name,target,scale,file in setups:
        camera=bpy.data.objects.get(name)
        if camera is None:
            data=bpy.data.cameras.new(name);camera=bpy.data.objects.new(name,data);stage.objects.link(camera)
        target=Vector(target);camera.location=target+Vector((3.10,-7.20,5.10))
        camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler()
        D.enum(camera.data,'type','ORTHO');camera.data.ortho_scale=scale
        camera['Archivo de render']=file

def apply():
    materials=palette();relocate_trees();patches(materials)
    snacks(materials);caravana(materials);taller(materials);cameras()
    collections=['09 | Nuevos claros y accesos','10 | Snacks y terraza de mesas','11 | Caravana y campamento','12 | Taller y surtidor antiguo']
    root=bpy.data.objects['RegularShow_Planet_ROOT']
    for name in collections:
        for ob in bpy.data.collections[name].objects:
            if ob.type=='MESH':
                for v in ob.data.vertices:
                    assert v.co.x**2+v.co.y**2<7.17**2,(ob.name,'fuera del terreno')
                    if v.co.z>.12:assert (v.co.x**2+v.co.y**2)/7.2**2+((v.co.z-.12)/6.55)**2<1,(ob.name,'fuera de la cupula')
                ob.parent=root
    bpy.context.scene['Nuevas zonas']='Snacks al suroeste; caravana al noreste; taller al sureste'
    print('Tres zonas comprobadas dentro del domo y separadas de la casa.')

def render_final(render_general=True):
    """Vistas cercanas sin la retícula delante; restaura la cúpula para la vista general."""
    s=bpy.context.scene
    glass=list(bpy.data.collections['06 | Cupula transparente y reticula'].objects)
    flags={ob.name:ob.hide_render for ob in glass}
    try:
        for ob in glass:ob.hide_render=True
        s.render.resolution_percentage=80;s.cycles.samples=96
        for camera,file in [('DOMO | Camara snacks','Snacks_Detalle.png'),('DOMO | Camara caravana','Caravana_Detalle.png'),('DOMO | Camara taller','Taller_Detalle.png')]:
            s.camera=bpy.data.objects[camera];s.render.filepath=os.path.join(REFS,file)
            bpy.ops.render.render(write_still=True)
    finally:
        for ob in glass:ob.hide_render=flags[ob.name]
        s.camera=bpy.data.objects['DOMO | Camara principal'];s.render.filepath=os.path.join(REFS,'regularshow_preview.png');s.render.resolution_percentage=100
    if render_general:bpy.ops.render.render(write_still=True)
    return None

if __name__=='__main__':apply()
