"""Carrito de golf de Regular Show con Mordecai conductor y Rigby sobre el techo.
Modelo independiente; todos los resultados se guardan junto a este script.
"""
import bpy
import math
import os
import random
import json
from mathutils import Vector, Matrix
from math import sin, cos, pi, sqrt

SCRIPT_DIR=os.path.dirname(os.path.abspath(__file__))
REPO=os.path.abspath(os.path.join(SCRIPT_DIR,'..','..','..','..','..'))
ROOT=os.path.join(REPO,'assets/models/ships/regularshow/golfcart')
REVIEWS=os.path.join(REPO,'docs/references/regularshow/ships/golfcart')
os.makedirs(ROOT,exist_ok=True)
os.makedirs(REVIEWS,exist_ok=True)
TAU=2*pi
MODEL='NAVE REGULAR SHOW | Carrito de golf'

def enum(obj,name,value):
    options=[i.identifier for i in obj.bl_rna.properties[name].enum_items]
    if value in options:setattr(obj,name,next(i for i in options if i==value))
    elif not options:
        try:setattr(obj,name,value)
        except (TypeError,ValueError):pass

def coll(name,parent=None):
    c=bpy.data.collections.get(name)
    if c is None:
        c=bpy.data.collections.new(name);(parent or bpy.context.scene.collection).children.link(c)
    return c

def material(name,color,rough=.78,metal=0):
    m=bpy.data.materials.get(name) or bpy.data.materials.new(name);m.use_nodes=True
    p=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    p.inputs['Base Color'].default_value=(*color,1);p.inputs['Roughness'].default_value=rough;p.inputs['Metallic'].default_value=metal
    m.diffuse_color=(*color,1)
    return m

def palette():
    return [material('RS | Carroceria marfil',(.81,.81,.70)),
            material('RS | Asiento crema',(.73,.70,.54)),
            material('RS | Tinta negra',(.008,.012,.014),.9),
            material('RS | Estructura gris',(.085,.105,.105),.55,.20),
            material('RS | Neumaticos',(.006,.008,.010),.95),
            material('RS | Metal de los ejes',(.07,.085,.085),.78,.12),
            material('RS | Cubo verde',(.16,.23,.055)),
            material('RS | Madera',(.40,.255,.11)),
            material('RS | Mordecai azul',(.08,.27,.60)),
            material('RS | Mordecai blanco',(.84,.88,.84)),
            material('RS | Pupilas y pico',(.009,.015,.020)),
            material('RS | Boca interior',(.075,.018,.024)),
            material('RS | Lengua',(.48,.15,.18)),
            material('RS | Rigby pelaje',(.29,.165,.055)),
            material('RS | Rigby beige',(.40,.385,.27)),
            material('RS | Rigby bandas oscuras',(.10,.075,.035)),
            material('RS | Rigby pelaje claro',(.40,.25,.085)),
            material('RS | Gomas y suelo',(.11,.13,.12)),
            material('RS | Tornillos claros',(.46,.51,.49),.46,.30),
            material('RS | Carton kraft',(.43,.275,.115),.95),
            material('RS | Canto de carton',(.24,.125,.046),1),
            material('RS | Marcador gastado',(.055,.043,.026),1),
            material('RS | Tape plateado',(.38,.42,.40),.48,.05),
            material('RS | Carton parche',(.52,.35,.17),.96),
            material('RS | Ondas del corrugado',(.56,.385,.19),1)]

class Mesh:
    def __init__(self):self.v=[];self.f=[];self.mi=[];self.sm=[]
    def add(self,v,f,mi=0,smooth=False):
        k=len(self.v);self.v.extend(tuple(p) for p in v);self.f.extend(tuple(k+i for i in face) for face in f);self.mi.extend([mi]*len(f));self.sm.extend([smooth]*len(f))
    def face(self,p,mi=0,smooth=False):self.add(p,[tuple(range(len(p)))],mi,smooth)
    def object(self,name,c,mats,parent=None):
        me=bpy.data.meshes.new(name);me.from_pydata(self.v,[],self.f);me.update()
        ob=bpy.data.objects.new(name,me);c.objects.link(ob)
        for mat in mats:me.materials.append(mat)
        for p,i,s in zip(me.polygons,self.mi,self.sm):p.material_index=i;p.use_smooth=s
        if parent:ob.parent=parent
        return ob

def box(mesh,center,size,mi=0):
    c=Vector(center);v=[c+Vector((x*size[0]/2,y*size[1]/2,z*size[2]/2)) for z in (-1,1) for y in (-1,1) for x in (-1,1)]
    mesh.add(v,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],mi)

def rounded_box(name,c,mats,parent,center,size,mi=0,bevel=.035):
    m=Mesh();box(m,center,size,mi);ob=m.object(name,c,mats,parent)
    if bevel:
        mod=ob.modifiers.new('Bordes redondeados','BEVEL');mod.width=bevel;mod.segments=3
    return ob

def ellipsoid(mesh,center,radii,mi=0,segments=24,rings=16):
    v=[(center[0],center[1],center[2]-radii[2])]
    for j in range(1,rings):
        lat=-pi/2+pi*j/rings
        for i in range(segments):
            a=TAU*i/segments;v.append((center[0]+radii[0]*cos(lat)*cos(a),center[1]+radii[1]*cos(lat)*sin(a),center[2]+radii[2]*sin(lat)))
    top=len(v);v.append((center[0],center[1],center[2]+radii[2]));f=[]
    for i in range(segments):f.append((0,1+(i+1)%segments,1+i))
    for j in range(rings-2):
        for i in range(segments):f.append((1+j*segments+i,1+j*segments+(i+1)%segments,1+(j+1)*segments+(i+1)%segments,1+(j+1)*segments+i))
    for i in range(segments):f.append((1+(rings-2)*segments+i,1+(rings-2)*segments+(i+1)%segments,top))
    mesh.add(v,f,mi,True)

def catmull(points,steps):
    points=[Vector(p) for p in points];out=[]
    for i in range(len(points)-1):
        a=points[max(i-1,0)];b=points[i];c=points[i+1];d=points[min(i+2,len(points)-1)]
        for j in range(steps):
            t=j/steps;out.append((b*2+(c-a)*t+(a*2-b*5+c*4-d)*t*t+(-a+b*3-c*3+d)*t*t*t)*.5)
    return out+[points[-1]]

def tube(mesh,points,radius,mi=0,sides=10,closed=False,radii=None,color_fn=None):
    pts=[Vector(p) for p in points];n=len(pts);v=[]
    for i,p in enumerate(pts):
        before=pts[(i-1)%n] if closed or i else pts[0]-(pts[1]-pts[0])
        after=pts[(i+1)%n] if closed or i<n-1 else pts[-1]+(pts[-1]-pts[-2])
        tangent=(after-before).normalized();ref=Vector((0,0,1)) if abs(tangent.z)<.94 else Vector((1,0,0))
        x=tangent.cross(ref).normalized();y=tangent.cross(x).normalized();r=radii[i] if radii else radius
        v.extend(p+r*(x*cos(TAU*j/sides)+y*sin(TAU*j/sides)) for j in range(sides))
    faces=[]
    for i in range(n if closed else n-1):
        faces.extend((i*sides+j,i*sides+(j+1)%sides,((i+1)%n)*sides+(j+1)%sides,((i+1)%n)*sides+j) for j in range(sides))
    offset=len(mesh.f);mesh.add(v,faces,mi,True)
    if color_fn:
        for i in range(n if closed else n-1):
            for j in range(sides):mesh.mi[offset+i*sides+j]=color_fn(i/(n-1))
    if not closed:
        mesh.face([v[i] for i in reversed(range(sides))],mi)
        mesh.face([v[(n-1)*sides+i] for i in range(sides)],mi)

def stroke(mesh,points,r=.008,mi=2,steps=6):tube(mesh,catmull(points,steps),r,mi,7)

def spike(mesh,base,tip,width,depth,mi=0):
    base=Vector(base);tip=Vector(tip);n=(tip-base).normalized();ref=Vector((0,1,0)) if abs(n.y)<.9 else Vector((1,0,0));x=n.cross(ref).normalized();y=n.cross(x).normalized()
    v=[base+x*cos(TAU*i/10)*width+y*sin(TAU*i/10)*depth for i in range(10)]+[tip]
    mesh.add(v,[tuple(reversed(range(10)))]+[(i,(i+1)%10,10) for i in range(10)],mi,True)

def lathe(mesh,profile,center,axis=(0,0,1),mi=0,segments=48):
    n=Vector(axis).normalized();ref=Vector((0,0,1)) if abs(n.z)<.9 else Vector((1,0,0));u=n.cross(ref).normalized();w=n.cross(u)
    v=[Vector(center)+n*z+(u*cos(TAU*i/segments)+w*sin(TAU*i/segments))*r for r,z in profile for i in range(segments)]
    f=[]
    for j in range(len(profile)-1):
        for i in range(segments):f.append((j*segments+i,j*segments+(i+1)%segments,(j+1)*segments+(i+1)%segments,(j+1)*segments+i))
    area=sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(profile,profile[1:]+profile[:1]))
    if area<0:f=[tuple(reversed(face)) for face in f]
    mesh.add(v,f,mi,True)

def init_scene():
    # Nunca sobrescribe los planetas: este archivo se crea en una sesión nueva.
    scene=bpy.data.scenes.new('RegularShow_GolfCart_Ship');previous=list(bpy.data.scenes)
    bpy.context.window.scene=scene
    for old in previous:
        if old!=scene:bpy.data.scenes.remove(old)
    for ob in list(bpy.data.objects):bpy.data.objects.remove(ob,do_unlink=True)
    for c in list(bpy.data.collections):bpy.data.collections.remove(c)
    bpy.data.orphans_purge(do_recursive=True)
    main=coll(MODEL);root=bpy.data.objects.new('RegularShow_GolfCart_ROOT',None);main.objects.link(root)
    root['asset_role']='ship';root['franchise']='regularshow';root['forward']='-Y';root['characters']='Mordecai y Rigby; controles independientes'
    for name in ('Mordecai_ROOT','Rigby_ROOT'):
        r=bpy.data.objects.new(name,None);main.objects.link(r);r.parent=root
    return root

def carrito(mats):
    main=coll(MODEL);c=coll('01 | Carrito de golf',main);root=bpy.data.objects['RegularShow_GolfCart_ROOT']
    frame=Mesh();trim=Mesh();wheels=Mesh();hood=Mesh();roof=Mesh();basket=Mesh();tools=Mesh()
    rounded_box('CART | Plataforma',c,mats,root,(0,0,.53),(2.06,3.12,.15),0,.045)
    rounded_box('CART | Suelo de goma',c,mats,root,(0,-.05,.621),(1.78,2.60,.045),17,.025)
    # Panel frontal inclinado, con grosor y bordes redondeados.
    v=[(-1.01,-1.73,.55),(1.01,-1.73,.55),(1.00,-1.38,1.29),(-1.00,-1.38,1.29),
       (-.96,-1.65,.58),(.96,-1.65,.58),(.96,-1.30,1.25),(-.96,-1.30,1.25)]
    hood.add(v,[(0,1,2,3),(4,7,6,5),(0,4,5,1),(3,2,6,7),(0,3,7,4),(1,5,6,2)],0)
    # Laterales abiertos, paneles cortos posteriores y guardabarros.
    for sign in (-1,1):
        rounded_box('CART | Panel posterior '+str(sign),c,mats,root,(sign*.96,.98,.95),(.15,.93,.68),0,.06)
        for cy in (-1.10,1.02):
            pts=[(sign*1.01,cy+.44*cos(pi*i/36),.38+.44*sin(pi*i/36)) for i in range(37)]
            tube(trim,pts,.057,0,10)
            tube(trim,[(sign*1.07,p[1],p[2]) for p in pts],.012,2,7)
        # Postes del techo y apoyo lateral curvo de la banca.
        for y in (-1.18,1.14):tube(frame,[(sign*.89,y,.66),(sign*.89,y,2.79)],.039,3,10)
        stroke(trim,[(sign*.92,.15,1.05),(sign*.94,.42,1.09),(sign*.94,.94,1.48)],.049,0)
        stroke(trim,[(sign*.94,-.70,.59),(sign*.94,.45,.59),(sign*.94,1.35,.57)],.013,2)
    rounded_box('CART | Bumper frontal',c,mats,root,(0,-1.80,.51),(2.18,.14,.13),0,.045)
    rounded_box('CART | Bumper trasero',c,mats,root,(0,1.58,.53),(2.10,.13,.12),0,.035)
    # Emblema oval blanco con doble borde negro, sobre el panel delantero.
    for rx,rz,r,mi in ((.145,.087,.014,2),(.112,.060,.009,18)):
        pts=[]
        for i in range(64):
            a=TAU*i/64;z=.99+rz*sin(a);y=-1.73+(z-.55)/.74*.35-.012
            pts.append((rx*cos(a),y,z))
        tube(trim,pts,r,mi,7,True)
    # Neumáticos y tapacubos, con ejes separados para futura animación.
    for x in (-1.01,1.01):
        for y in (-1.10,1.02):
            lathe(wheels,[(0,-.14),(.23,-.14),(.32,-.105),(.366,-.055),(.372,.055),(.32,.105),(.23,.14),(0,.14)],(x,y,.38),(1,0,0),4,56)
            side=x+(.151 if x>0 else -.151)
            lathe(wheels,[(0,-.008),(.155,-.008),(.173,0),(.155,.015),(0,.015)],(side,y,.38),(1,0,0),5,40)
            for k in range(5):
                a=TAU*k/5;ellipsoid(wheels,(side+(.021 if x>0 else -.021),y+.103*cos(a),.38+.103*sin(a)),(.013,.014,.014),18,10,7)
    for y in (-1.1,1.02):tube(frame,[(-.91,y,.38),(.91,y,.38)],.07,5,12)
    # Asiento corrido marfil, costuras oscuras y respaldo ligeramente inclinado.
    rounded_box('CART | Cojin del asiento',c,mats,root,(0,.63,1.04),(1.75,.88,.20),1,.08)
    rounded_box('CART | Respaldo crema',c,mats,root,(0,1.13,1.51),(1.78,.20,.84),1,.085)
    for x in (-.64,-.32,0,.32,.64):
        stroke(trim,[(x,.26,1.145),(x,.64,1.147),(x,1.00,1.135)],.007,2)
        stroke(trim,[(x,1.019,1.18),(x,1.013,1.51),(x,1.03,1.87)],.007,2)
    # Volante inclinado y tres radios.
    center=Vector((.36,-.86,1.50));n=Vector((0,-.65,.76)).normalized();u=Vector((1,0,0));w=n.cross(u)
    tube(frame,[center+u*.24*cos(TAU*i/64)+w*.24*sin(TAU*i/64) for i in range(64)],.026,2,9,True)
    for a in (pi/2,pi*7/6,pi*11/6):tube(frame,[center,center+u*.22*cos(a)+w*.22*sin(a)],.020,3,8)
    ellipsoid(frame,center,(.075,.056,.065),3,20,12)
    tube(frame,[(.36,-1.23,1.02),center],.040,3,10)
    rounded_box('CART | Tablero',c,mats,root,(.1,-1.23,1.19),(1.70,.24,.10),17,.04)
    # Techo con contorno redondeado y curvatura suave en la parte superior.
    boundary=[];r=.22
    for cx,cy,start in ((.98,-1.43,-pi/2),(.98,1.43,0),(-.98,1.43,pi/2),(-.98,-1.43,pi)):
        for j in range(12):
            a=start+pi/2*j/12;boundary.append(Vector((cx+r*cos(a),cy+r*sin(a),2.80)))
    count=len(boundary);v=[]
    for factor,z in ((1,2.78),(1,2.84),(.88,2.91),(.30,2.935)):
        v.extend((p.x*factor,p.y*factor,z) for p in boundary)
    f=[]
    for j in range(3):
        for i in range(count):f.append((j*count+i,j*count+(i+1)%count,(j+1)*count+(i+1)%count,(j+1)*count+i))
    f.extend([tuple(reversed(range(count))),tuple(3*count+i for i in range(count))]);roof.add(v,f,0,True)
    tube(trim,[tuple(p) for p in boundary],.018,2,8,True)
    # Cubeta verde trasera, abierta por arriba, y herramientas del parque.
    v=[]
    for sx,sy,z in ((.40,.25,.80),(.54,.36,1.54),(.47,.29,1.54),(.34,.20,.92)):
        v.extend((.52+x*sx,1.78+y*sy,z) for x,y in ((-1,-1),(1,-1),(1,1),(-1,1)))
    f=[]
    for j in range(3):
        for i in range(4):f.append((j*4+i,j*4+(i+1)%4,(j+1)*4+(i+1)%4,(j+1)*4+i))
    f.append((12,13,14,15));basket.add(v,f,6)
    for z in (1.42,1.51):tube(trim,[(-.04,1.39,z),(1.08,1.39,z),(1.08,2.17,z),(-.04,2.17,z)],.023,5,7,True)
    for a,b in ((((.26,1.80,.98),(.07,2.00,2.36))),(((.64,1.85,.98),(.72,2.11,2.24)))):tube(tools,[a,b],.026,7,10)
    box(tools,(.06,2.00,2.40),(.39,.14,.11),3)
    for x in (-.09,-.02,.05,.12,.19):tube(tools,[(x,1.94,2.36),(x,1.94,2.44)],.011,18,5)
    ellipsoid(tools,(.73,2.10,2.24),(.07,.04,.15),7,16,10)
    hood.object('CART | Frente inclinado',c,mats,root)
    roof.object('CART | Techo blanco redondeado',c,mats,root)
    frame.object('CART | Postes, volante y ejes',c,mats,root)
    trim.object('CART | Costuras, bordes y emblema',c,mats,root)
    wheels.object('CART | Cuatro ruedas y tapacubos',c,mats,root)
    basket.object('CART | Cubeta verde de mantenimiento',c,mats,root)
    tools.object('CART | Escobas y herramientas',c,mats,root)
    print('CARRITO LISTO',flush=True)

def alas_carton(mats):
    c=coll('04 | Alas caseras de carton y tape',coll(MODEL));root=bpy.data.objects['RegularShow_GolfCart_ROOT']
    # Paneles planos recortados a mano, con un ligero pliegue hacia arriba.
    contour=[(.91,-.47),(1.55,-.51),(2.84,-.02),(3.17,.12),(3.11,.38),(3.20,.59),(3.02,.86),(2.87,.82),(1.54,.58),(.91,.55)]
    def height(x,y):return .86+.10*(x-.91)+.045*max(0,x-1.9)
    def top(s,x,y,offset=0):return (s*x,y,height(x,y)+offset)
    for sign,label in ((-1,'izquierda'),(1,'derecha')):
        board=Mesh();edge=Mesh();marks=Mesh();tape=Mesh();supports=Mesh();patch=Mesh()
        n=len(contour);vertices=[top(sign,x,y,offset) for offset in (0,-.055) for x,y in contour]
        faces=[tuple(range(n)),tuple(reversed(range(n,2*n)))]
        faces.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n))
        if sign<0:faces=[tuple(reversed(f)) for f in faces]
        board.add(vertices,faces,19);board.mi[2:]=[20]*n
        # Canto con ondas reales: el espesor no parece una chapa metálica.
        for i,(a,b) in enumerate(zip(contour,contour[1:]+contour[:1])):
            length=math.dist(a,b);count=max(3,int(length/.045));pts=[]
            for j in range(count*6+1):
                t=j/(count*6);xx=a[0]+(b[0]-a[0])*t;yy=a[1]+(b[1]-a[1])*t
                zz=-.028+.018*sin(t*count*TAU);pts.append(top(sign,xx,yy,zz))
            tube(edge,pts,.0038,24,5)
        # Pliegues finos, rayas de fibra y uniones de cajas reutilizadas.
        for u in (1.78,2.28):
            y0=-.51+(u-1.55)*.38;y1=.58+(u-1.54)*.18
            stroke(marks,[top(sign,u,y0+.015,.003),top(sign,u+.025,(y0+y1)/2,.004),top(sign,u-.018,y1-.02,.003)],.005,20,3)
            stroke(marks,[top(sign,u+.025,y0+.02,.004),top(sign,u+.04,y1-.03,.004)],.0025,23,2)
        rng=random.Random(110+sign)
        for i in range(24):
            xx=rng.uniform(1.15,2.8);yy=rng.uniform(-.04,.37)
            stroke(marks,[top(sign,xx,yy,.003),top(sign,xx+.09,yy+.003,.003)],.0017,23,1)
        # Flecha dibujada con marcador negro, con trazo algo irregular.
        for coords in ([(2.38,.53),(2.38,.11),(2.35,-.025)],[(2.18,.14),(2.35,-.025),(2.55,.20)]):
            stroke(marks,[top(sign,x,y,.007) for x,y in coords],.013,21,5)
        # Parche de otra caja y cinta sobre los cuatro extremos del parche.
        panel=[(1.86,.13),(2.17,.17),(2.14,.47),(1.82,.43)]
        pv=[top(sign,x,y,.008) for x,y in panel]
        patch.face(pv if sign>0 else list(reversed(pv)),23)
        def ribbon(points,width,seed):
            points=[Vector(p) for p in points];rng=random.Random(seed);vv=[]
            for j,p in enumerate(points):
                # Anchos y bordes irregulares imitan cinta arrancada a mano.
                w=width*(.95+rng.random()*.12)
                vv.extend([p+Vector((0,-w/2,0)),p+Vector((0,w/2,0))])
            ff=[(2*i,2*i+2,2*i+3,2*i+1) for i in range(len(points)-1)]
            if sign<0:ff=[tuple(reversed(face)) for face in ff]
            count=len(vv);vv.extend(p-Vector((0,0,.003)) for p in list(vv))
            ff+=list(tuple(index+count for index in reversed(face)) for face in list(ff))
            for i in range(len(points)-1):
                for side in (0,1):
                    a=2*i+side;b=2*(i+1)+side;ff.append((a,b,b+count,a+count))
            ff.extend([(0,1,count+1,count),(count-2,count-1,2*count-1,2*count-2)])
            tape.add(vv,ff,22)
        # Dos cinchas que cruzan la raíz, envuelven la carrocería y se arrugan.
        for j,y in enumerate((-.25,.36)):
            pts=[(sign*.92,y,.64),(sign*.969,y,.72),(sign*.973,y,.87)]
            pts.extend(top(sign,xx,y+.01*sin(k*1.4),.010+.003*(k%2)) for k,xx in enumerate((1.02,1.14,1.27,1.40,1.53,1.62)))
            ribbon(pts,.145,55+j+sign)
            for xx in (1.14,1.40):
                stroke(marks,[top(sign,xx,y-.053,.016),top(sign,xx+.018,y,.018),top(sign,xx-.01,y+.049,.016)],.0025,18,2)
        for xx,yy in ((1.84,.22),(2.13,.38)):
            ribbon([top(sign,xx-.075,yy-.025,.015),top(sign,xx,yy,.018),top(sign,xx+.075,yy+.018,.015)],.075,72+sign)
        # Palitos de madera sujetan las alas por debajo, sin motores añadidos.
        for yy in (-.24,.36):
            tube(supports,[(sign*.88,yy,.63),top(sign,1.91,yy,-.068)],.022,7,6)
        for mesh,name in ((board,'Panel kraft'),(edge,'Corrugado expuesto'),(marks,'Pliegues y marcador'),(tape,'Cinta adhesiva arrancada'),(supports,'Palitos de refuerzo'),(patch,'Parche de caja')):
            ob=mesh.object('ALA '+label+' | '+name,c,mats,root)
            ob['construction']='Carton corrugado sujeto con cinta adhesiva y palitos'
    root['homemade_wings']=True
    print('ALAS DE CARTON LISTAS',flush=True)

def mordecai(mats):
    c=coll('02 | Mordecai al volante',coll(MODEL));root=bpy.data.objects['Mordecai_ROOT']
    body=Mesh();head=Mesh();eyes=Mesh();beak=Mesh();arms=Mesh();legs=Mesh();ink=Mesh()
    x=.36
    def sector_angle(k,n,boundary):
        t=k/n
        if t<.25:return -pi+(pi-boundary)*t/.25
        if t<.75:return -boundary+2*boundary*(t-.25)/.5
        return boundary+(pi-boundary)*(t-.75)/.25
    # Una única superficie de cadera a cuello. El blanco es parte de la piel,
    # evitando las esferas superpuestas que abultaban el tronco anterior.
    profile=[(1.145,.16,.17,.40),(1.23,.228,.215,.36),
             (1.40,.248,.215,.28),(1.57,.224,.183,.18),
             (1.73,.192,.151,.075),(1.88,.151,.129,-.035),
             (2.035,.149,.132,-.082)]
    rings=[]
    for i in range(len(profile)-1):
        a=Vector(profile[max(0,i-1)]);b=Vector(profile[i]);cc=Vector(profile[i+1]);d=Vector(profile[min(len(profile)-1,i+2)])
        for j in range(5):
            t=j/5;rings.append((b*2+(cc-a)*t+(a*2-b*5+cc*4-d)*t*t+(-a+b*3-cc*3+d)*t*t*t)*.5)
    rings.append(Vector(profile[-1]));n=64;v=[]
    for z,rx,ry,cy in rings:
        bib=.77+.22*min(1,max(0,(z-1.22)/.53))
        v.extend((x+rx*sin(sector_angle(j,n,bib)),cy-ry*cos(sector_angle(j,n,bib)),z) for j in range(n))
    faces=[];colors=[]
    for j in range(len(rings)-1):
        z=(rings[j][0]+rings[j+1][0])/2
        bib=.77+.22*min(1,max(0,(z-1.22)/.53))
        for k in range(n):
            faces.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));colors.append(9 if n//4<=k<3*n//4 else 8)
    faces.extend([tuple(reversed(range(n))),tuple((len(rings)-1)*n+k for k in range(n))]);colors.extend([8,8])
    body.add(v,faces,8,True);body.mi[:]=colors
    # La retícula sigue el borde curvo del plumaje blanco, evitando escalones.
    n=64;nr=40;v=[(x,-.10,2.17-.305)];facecolors=[]
    for j in range(1,nr):
        latitude=-pi/2+pi*j/nr;z=2.17+.305*sin(latitude)
        boundary=1.4 if z<=2.12 else max(.018,1.4*sqrt(max(0,1-((z-2.12)/.15)**2)))
        for k in range(n):
            a=sector_angle(k,n,boundary)
            v.append((x+.245*cos(latitude)*sin(a),-.10-.215*cos(latitude)*cos(a),z))
    top=len(v);v.append((x,-.10,2.475));f=[]
    for k in range(n):f.append((0,1+(k+1)%n,1+k));facecolors.append(9 if n//4<=k<3*n//4 else 8)
    for j in range(nr-2):
        z=(v[1+j*n][2]+v[1+(j+1)*n][2])/2
        for k in range(n):
            f.append((1+j*n+k,1+j*n+(k+1)%n,1+(j+1)*n+(k+1)%n,1+(j+1)*n+k));facecolors.append(9 if z<2.27 and n//4<=k<3*n//4 else 8)
    for k in range(n):f.append((1+(nr-2)*n+k,1+(nr-2)*n+(k+1)%n,top));facecolors.append(8)
    head.add(v,f,8,True);head.mi[:]=facecolors
    # Cresta inclinada hacia atrás y plumas triangulares laterales.
    spike(head,(x,.015,2.36),(x,.33,2.66),.205,.105,8)
    spike(head,(x,.115,2.28),(x,.43,2.38),.17,.085,8)
    spike(head,(x-.17,.015,2.25),(x-.25,.20,2.31),.09,.05,8)
    # Globos blancos con borde negro y pupilas muy visibles.
    for dx in (-.128,.128):
        ellipsoid(eyes,(x+dx,-.305,2.274),(.129,.083,.140),10,28,20)
        ellipsoid(eyes,(x+dx,-.335,2.278),(.116,.077,.125),9,28,20)
        ellipsoid(eyes,(x+dx+.018,-.408,2.278),(.037,.017,.047),10,20,14)
        ellipsoid(eyes,(x+dx+.030,-.423,2.298),(.009,.006,.012),9,12,8)
    # Pico superior largo, negro y afilado; la base conserva volumen.
    points=catmull([(x,-.31,2.175),(x,-.51,2.178),(x,-.79,2.148),(x,-1.01,2.125)],8)
    verts=[];segments=18
    for i,p in enumerate(points):
        t=i/(len(points)-1);rx=.147*(1-t)**.65+.002;rz=.050*(1-t)**.55+.001
        verts.extend(p+Vector((rx*cos(TAU*j/segments),0,rz*sin(TAU*j/segments))) for j in range(segments))
    faces=[]
    for j in range(len(points)-1):
        for k in range(segments):faces.append((j*segments+k,j*segments+(k+1)%segments,(j+1)*segments+(k+1)%segments,(j+1)*segments+k))
    beak.add(verts,faces,10,True)
    # Sonrisa abierta debajo del pico, mandíbula y lengua.
    ellipsoid(beak,(x,-.342,2.047),(.172,.061,.113),10,28,18)
    ellipsoid(beak,(x,-.382,2.045),(.145,.027,.082),11,28,18)
    stroke(beak,[(x-.155,-.365,2.075),(x-.08,-.57,1.986),(x,-.77,2.020),(x+.11,-.52,2.015),(x+.156,-.36,2.075)],.016,10,8)
    ellipsoid(beak,(x,-.441,2.001),(.070,.065,.019),12,20,12)
    for dx in (-.079,-.018,.047):
        spike(beak,(x+dx,-.402,2.094),(x+dx,-.416,2.068),.018,.014,9)
    # Brazos doblados, dos bandas blancas por antebrazo y manos en el volante.
    for sign in (-1,1):
        start=(x+sign*.177,.067,1.744);elbow=(x+sign*.304,-.25,1.421);wrist=(x+sign*.227,-.845,1.52)
        pts=catmull([start,(x+sign*.265,-.04,1.61),elbow,(x+sign*.270,-.53,1.455),wrist],7)
        radii=[.064-.011*i/(len(pts)-1) for i in range(len(pts))]
        def bands(t):return 9 if .58<t<.65 or .77<t<.84 else 8
        tube(arms,pts,.08,8,16,radii=radii,color_fn=bands)
        ellipsoid(arms,wrist,(.073,.081,.053),8,20,14)
        for j in range(3):
            xx=x+sign*.20+(j-1)*.040
            stroke(arms,[(xx,-.83,1.55),(xx,-.92,1.52),(xx,-.925,1.47)],.023,8,5)
            stroke(ink,[(xx-.013,-.934,1.514),(xx-.013,-.927,1.477)],.004,10,3)
    # Muslos sentados, patas oscuras y dedos sobre el piso del carro.
    for sign in (-1,1):
        pts=catmull([(x+sign*.125,.38,1.21),(x+sign*.15,.05,1.18),(x+sign*.16,-.24,1.05)],6)
        tube(legs,pts,.065,9,14)
        stroke(legs,[(x+sign*.16,-.24,1.05),(x+sign*.17,-.46,.83),(x+sign*.17,-.75,.73)],.031,10)
        for dx in (-.05,0,.05):stroke(legs,[(x+sign*.17,-.74,.73),(x+sign*.17+dx,-.92,.68)],.020,10)
    # Abanico de plumas de la cola, detrás del conductor.
    for j,(tipx,tipy,tipz) in enumerate(((-.05,.92,1.69),(-.18,1.15,1.55),(-.12,1.35,1.36),(.03,1.41,1.18))):
        pts=catmull([(x,.65,1.20),((x+tipx)/2,.91,(1.20+tipz)/2),(tipx,tipy,tipz)],7)
        r=[.049*sin(pi*(i+1)/(len(pts)+1))+.009 for i in range(len(pts))]
        tube(legs,pts,.06,8,10,radii=r)
        stroke(ink,[(tipx+.008,tipy-.035,tipz-.035),((x+tipx)/2,.9,(1.2+tipz)/2)],.005,10)
    for mesh,name in ((body,'MORDECAI | Torso azul y pecho blanco'),(head,'MORDECAI | Cabeza, mejillas y cresta'),(eyes,'MORDECAI | Ojos y pupilas'),(beak,'MORDECAI | Pico largo y sonrisa'),(arms,'MORDECAI | Brazos, bandas blancas y manos'),(legs,'MORDECAI | Piernas y cola de plumas'),(ink,'MORDECAI | Trazos negros del plumaje')):
        mesh.object(name,c,mats,root)
    root['pose']='Conductor sentado, con las dos manos sobre el volante'
    print('MORDECAI LISTO',flush=True)

def rigby(mats):
    c=coll('03 | Rigby sobre el techo',coll(MODEL));root=bpy.data.objects['Rigby_ROOT']
    body=Mesh();face=Mesh();eyes=Mesh();mouth=Mesh();limbs=Mesh();tail=Mesh();ink=Mesh()
    x=-.10;y=.08
    # Torso continuo de mapache, más ancho en la cabeza y con vientre beige.
    profile=[(3.25,.18,.14),(3.34,.22,.17),(3.55,.245,.19),(3.80,.26,.205),(4.02,.30,.215),(4.13,.265,.19),(4.20,.16,.13),(4.215,.04,.04)]
    profile=[tuple(p) for p in catmull(profile,4)]
    v=[];n=32
    for z,rx,ry in profile:
        v.extend((x+rx*cos(TAU*i/n),y+ry*sin(TAU*i/n),z) for i in range(n))
    f=[]
    for j in range(len(profile)-1):
        for i in range(n):f.append((j*n+i,j*n+(i+1)%n,(j+1)*n+(i+1)%n,(j+1)*n+i))
    f.extend([tuple(reversed(range(n))),tuple((len(profile)-1)*n+i for i in range(n))]);body.add(v,f,13,True)
    ellipsoid(body,(x,-.089,3.55),(.202,.075,.325),14,28,20)
    # Orejas con interior oscuro y mechones de la cabeza.
    for sign in (-1,1):
        ellipsoid(body,(x+sign*.25,.09,4.115),(.082,.048,.105),13,24,16)
        ellipsoid(face,(x+sign*.25,.045,4.126),(.045,.019,.064),15,20,14)
    for dx,z,dy in ((-.19,4.26,.10),(-.075,4.32,.12),(.035,4.30,.14),(.145,4.27,.12)):
        spike(body,(x+dx,.075,4.105),(x+dx-.035,dy,z),.075,.05,16)
    # Gafas naturales oscuras y ojos grandes juntos, fieles a la expresión.
    for dx in (-.132,.132):
        ellipsoid(eyes,(x+dx,-.134,4.075),(.136,.083,.140),15,28,20)
        ellipsoid(eyes,(x+dx,-.181,4.083),(.128,.067,.131),10,28,20)
        ellipsoid(eyes,(x+dx,-.202,4.087),(.116,.065,.119),9,28,20)
        ellipsoid(eyes,(x+dx+.012,-.264,4.087),(.034,.017,.043),10,20,14)
        ellipsoid(eyes,(x+dx+.022,-.279,4.106),(.008,.005,.009),9,12,8)
    # Hocico puntiagudo color marrón claro, nariz negra y boca abierta.
    pts=catmull([(x,-.135,3.983),(x,-.30,3.98),(x,-.45,3.984),(x,-.59,4.012)],7)
    v=[];segments=18
    for i,p in enumerate(pts):
        t=i/(len(pts)-1);rx=.16*(1-t)**.65+.012;rz=.059*(1-t)**.65+.01
        v.extend(p+Vector((rx*cos(TAU*j/segments),0,rz*sin(TAU*j/segments))) for j in range(segments))
    f=[]
    for j in range(len(pts)-1):
        for k in range(segments):f.append((j*segments+k,j*segments+(k+1)%segments,(j+1)*segments+(k+1)%segments,(j+1)*segments+k))
    face.add(v,f,16,True)
    ellipsoid(face,(x,-.60,4.016),(.040,.036,.031),10,20,14)
    ellipsoid(mouth,(x,-.144,3.871),(.192,.073,.142),10,32,20)
    ellipsoid(mouth,(x,-.199,3.865),(.163,.031,.114),11,28,18)
    ellipsoid(mouth,(x+.012,-.229,3.807),(.070,.015,.040),12,24,14)
    # Dientes separados: tres arriba y dos abajo.
    for dx in (-.105,-.031,.05):
        spike(mouth,(x+dx,-.230,3.954),(x+dx+.008,-.238,3.909),.023,.017,9)
    for dx in (-.073,.026):
        spike(mouth,(x+dx,-.220,3.780),(x+dx,-.232,3.815),.022,.016,9)
    stroke(ink,[(x-.17,-.179,3.917),(x-.19,-.19,3.85),(x-.09,-.211,3.758),(x+.10,-.211,3.772),(x+.185,-.172,3.875)],.008,10,7)
    # Brazos finos y largos, pose asimétrica juguetona como en la referencia.
    left=[(x-.22,.06,3.79),(x-.46,-.03,3.60),(x-.77,-.05,3.48),(x-.97,-.11,3.44)]
    right=[(x+.22,.06,3.79),(x+.56,.07,3.73),(x+.74,-.015,3.54),(x+.65,-.16,3.40)]
    for points,sign in ((left,-1),(right,1)):
        pts=catmull(points,7);r=[.037-.009*i/(len(pts)-1) for i in range(len(pts))]
        tube(limbs,pts,.035,13,12,radii=r)
        p=Vector(points[-1]);ellipsoid(limbs,p,(.053,.038,.038),16,18,12)
        for j in range(3):
            start=p+Vector((sign*.025,-.008*(j-1),-.008))
            end=p+Vector((sign*(.09-.018*j),-.045-j*.014,-.023-j*.017))
            stroke(limbs,[start,start.lerp(end,.6)+Vector((0,-.015,.012)),end],.012,13,4)
    # Patas delgadas y tres dedos largos apoyados sobre la cubierta.
    for sign in (-1,1):
        hip=(x+sign*.15,.07,3.30);knee=(x+sign*.29,-.015,3.13);ankle=(x+sign*.25,-.105,2.971)
        stroke(limbs,[hip,knee,ankle],.032,13,8)
        for j in (-1,0,1):
            end=(x+sign*.25+j*.057,-.268-abs(j)*.01,2.962)
            stroke(limbs,[ankle,(ankle[0]+j*.035,-.21,2.966),end],.017,16,5)
            stroke(ink,[(end[0],end[1],end[2]+.004),(end[0],end[1]-.025,end[2]-.002)],.005,15,3)
    # Cola gruesa con cuatro bandas oscuras, curvada hacia atrás y a la izquierda.
    pts=catmull([(x-.06,.25,3.34),(x-.44,.08,3.32),(x-.85,-.18,3.46),(x-1.20,-.38,3.68),(x-1.38,-.48,3.75)],12)
    radii=[]
    for i in range(len(pts)):
        t=i/(len(pts)-1);radii.append(.10+.13*sin(pi*t)**.65-.025*t)
    tube(tail,pts,.15,13,20,radii=radii,color_fn=lambda t:15 if int(t*8)%2 else 16)
    ellipsoid(tail,pts[-1],(.075,.075,.075),16,20,14)
    # Trazos de pelaje a ambos lados del vientre y junto a las mejillas.
    def fur_point(z,side,fraction):
        index=next((i for i in range(len(profile)-1) if profile[i][0]<=z<=profile[i+1][0]),len(profile)-2)
        a=profile[index];b=profile[index+1];t=(z-a[0])/(b[0]-a[0]);rx=a[1]+(b[1]-a[1])*t;ry=a[2]+(b[2]-a[2])*t
        return (x+side*rx*fraction,y-ry*sqrt(1-fraction*fraction)-.004,z)
    for sign in (-1,1):
        for j in range(7):
            z=3.32+j*.085
            stroke(ink,[fur_point(z,sign,.78),fur_point(z+.028,sign,.82)],.0035,15,3)
        for j in range(3):
            z=3.98+j*.055
            stroke(ink,[fur_point(z,sign,.88),fur_point(z+.022,sign,.91)],.0035,15,3)
    for mesh,name in ((body,'RIGBY | Cuerpo, vientre y orejas'),(face,'RIGBY | Hocico, nariz y orejas'),(eyes,'RIGBY | Ojos y pupilas'),(mouth,'RIGBY | Sonrisa, dientes y lengua'),(limbs,'RIGBY | Brazos, piernas y dedos'),(tail,'RIGBY | Cola con bandas oscuras'),(ink,'RIGBY | Trazos de pelaje')):
        mesh.object(name,c,mats,root)
    root['pose']='Sobre el techo, con brazos extendidos y cola anillada'
    print('RIGBY LISTO',flush=True)

def character_pivots():
    for name,pivot in (('Mordecai_ROOT',(.36,.10,1.14)),('Rigby_ROOT',(-.10,.08,2.94))):
        root=bpy.data.objects[name];matrices={ob:ob.matrix_world.copy() for ob in root.children}
        root.location=pivot;bpy.context.view_layer.update()
        for ob,matrix in matrices.items():ob.matrix_world=matrix
        root['asset_role']='character';root['franchise']='regularshow'

def outlines():
    c=coll('90 | Contornos de presentacion');m=bpy.data.materials.new('RS | Contorno de caricatura');m.use_nodes=True
    nt=m.node_tree;nt.nodes.clear()
    output=nt.nodes.new('ShaderNodeOutputMaterial');ink=nt.nodes.new('ShaderNodeEmission');ink.inputs['Color'].default_value=(.003,.005,.006,1);ink.inputs['Strength'].default_value=1
    trans=nt.nodes.new('ShaderNodeBsdfTransparent');geo=nt.nodes.new('ShaderNodeNewGeometry');mix=nt.nodes.new('ShaderNodeMixShader')
    nt.links.new(ink.outputs[0],mix.inputs[1]);nt.links.new(trans.outputs[0],mix.inputs[2]);nt.links.new(geo.outputs['Backfacing'],mix.inputs[0]);nt.links.new(mix.outputs[0],output.inputs['Surface'])
    for ob in list(bpy.data.collections[MODEL].all_objects):
        if ob.type!='MESH':continue
        target=any(token in ob.name for token in ('Torso azul','Cabeza, mejillas','Cuerpo, vientre','Cola con bandas','Frente inclinado','Techo blanco','Co jin','Cojin del asiento','Respaldo crema'))
        if not target:continue
        me=bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
        for vertex in me.vertices:vertex.co+=vertex.normal*.008
        me.flip_normals();me.update()
        outline=bpy.data.objects.new('INK | '+ob.name,me);c.objects.link(outline)
        outline.parent=ob.parent;outline.matrix_world=ob.matrix_world.copy()
        outline.data.materials.clear();outline.data.materials.append(m)
        for face in outline.data.polygons:face.material_index=0
        outline['render_only']=True
        if hasattr(outline,'visible_shadow'):outline.visible_shadow=False

def presentation():
    scene=bpy.context.scene;c=coll('91 | Camaras, luces y espacio')
    world=bpy.data.worlds.new('RS | Espacio violeta');world.use_nodes=True
    nt=world.node_tree;bg=next(n for n in nt.nodes if n.type=='BACKGROUND')
    coord=nt.nodes.new('ShaderNodeTexCoord');noise=nt.nodes.new('ShaderNodeTexNoise');noise.inputs['Scale'].default_value=3.1;noise.inputs['Detail'].default_value=4
    ramp=nt.nodes.new('ShaderNodeValToRGB');ramp.color_ramp.elements[0].position=.28;ramp.color_ramp.elements[0].color=(.001,.002,.008,1)
    ramp.color_ramp.elements[1].position=.78;ramp.color_ramp.elements[1].color=(.055,.008,.075,1)
    middle=ramp.color_ramp.elements.new(.52);middle.color=(.008,.003,.024,1)
    nt.links.new(coord.outputs['Normal'],noise.inputs['Vector']);nt.links.new(noise.outputs['Fac'],ramp.inputs[0]);nt.links.new(ramp.outputs[0],bg.inputs['Color']);bg.inputs['Strength'].default_value=.45
    scene.world=world
    def camera(name,location,target,scale):
        data=bpy.data.cameras.new(name);ob=bpy.data.objects.new(name,data);c.objects.link(ob);ob.location=location
        ob.rotation_euler=(Vector(target)-ob.location).to_track_quat('-Z','Y').to_euler();enum(data,'type','ORTHO');data.ortho_scale=scale;data.clip_end=500
        return ob
    hero=camera('RS | Camara principal',(9,-8,5.5),(0,.05,2.08),7.0)
    camera('RS | Camara frontal',(0,-11,4.5),(0,0,2.15),7.0)
    camera('RS | Camara lateral',(11,-3.2,4.9),(0,0,2.10),7.0)
    camera('RS | Camara caras',(4.4,-9,4.5),(.06,-.03,2.88),3.5)
    scene.camera=hero
    def area(name,position,power,size,color):
        typ=next(i.identifier for i in bpy.types.Light.bl_rna.properties['type'].enum_items if i.identifier=='AREA')
        data=bpy.data.lights.new(name,typ);data.energy=power;data.size=size;data.color=color
        ob=bpy.data.objects.new(name,data);c.objects.link(ob);ob.location=position;ob.rotation_euler=(Vector((0,0,2))-ob.location).to_track_quat('-Z','Y').to_euler()
    area('RS | Luz principal',(-5,-7,9),1500,6,(1,.95,.87))
    area('RS | Relleno de caras',(5,-5,5),950,5,(.84,.92,1))
    area('RS | Contraluz',(2,5,7),1800,5,(.77,.85,1))
    stars=Mesh();rng=random.Random(1029);rotation=hero.rotation_euler.to_quaternion();right=rotation@Vector((1,0,0));up=rotation@Vector((0,1,0));normal=rotation@Vector((0,0,1))
    for i in range(135):
        center=-normal*15+right*rng.uniform(-6,6)+up*rng.uniform(-6,6)+Vector((0,0,2))
        r=rng.uniform(.004,.014);v=[center]+[center+right*r*cos(TAU*j/8)+up*r*sin(TAU*j/8) for j in range(8)]
        stars.add(v,[(0,j+1,(j+1)%8+1) for j in range(8)],0)
    star=material('RS | Estrellas',(.71,.80,1));p=next(n for n in star.node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Emission Color'].default_value=(.71,.80,1,1);p.inputs['Emission Strength'].default_value=2
    stars.object('RS | Estrellas de presentacion',c,[star])
    try:scene.render.engine='CYCLES'
    except TypeError as exc:print(exc)
    if scene.render.engine=='CYCLES':
        scene.cycles.samples=64;scene.cycles.use_denoising=True;scene.cycles.transparent_max_bounces=20
        try:
            prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.get_devices();typ=next((d.type for d in prefs.devices if d.type=='OPTIX'),None)
            if typ:
                prefs.compute_device_type=typ;prefs.get_devices()
                for d in prefs.devices:d.use=d.type==typ
                enum(scene.cycles,'device','GPU')
        except Exception as exc:print('GPU',exc)
    scene.render.resolution_x=1800;scene.render.resolution_y=1500;scene.render.resolution_percentage=65
    scene.view_settings.exposure=.15;scene.render.filepath=os.path.join(REVIEWS,'Carrito_Mordecai_Rigby.png')
    enum(scene.render.image_settings,'file_format','PNG');enum(scene.render.image_settings,'color_mode','RGB')
    for screen in bpy.data.screens:
        for ar in screen.areas:
            if ar.type=='VIEW_3D':
                ar.spaces.active.region_3d.view_rotation=hero.rotation_euler.to_quaternion();ar.spaces.active.region_3d.view_location=(0,.05,2.08);ar.spaces.active.region_3d.view_distance=9
                enum(ar.spaces.active.region_3d,'view_perspective','CAMERA');enum(ar.spaces.active.shading,'type','MATERIAL')
                ar.spaces.active.shading.use_scene_world=True;ar.spaces.active.shading.use_scene_lights=True;ar.spaces.active.overlay.show_overlays=False
    for ob in list(bpy.context.selected_objects):ob.select_set(False)
    root=bpy.data.objects['RegularShow_GolfCart_ROOT'];root.select_set(True);bpy.context.view_layer.objects.active=root
    text=bpy.data.texts.new('README | Carrito de Regular Show')
    text.write('NAVE REGULAR SHOW: CARRITO DE GOLF\nMordecai conduce y Rigby esta de pie sobre el techo.\nDos alas de carton corrugado sujetas con tape y palitos.\nModelo separado de los planetas y guardado solo en ProyectoFinal/Nave_RegularShow.\nRegularShow_GolfCart_ROOT mueve todo el conjunto. Mordecai_ROOT y Rigby_ROOT permiten separar o mover a cada personaje.\nLos contornos de presentacion y las camaras/luces se excluyen de las exportaciones.\nNo se ha creado logica de vuelo ni un rig de animacion.\n')
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(ROOT,'regularshow_golfcart_ship.blend'))
    print('GUARDADO',bpy.data.filepath,flush=True)

def build():
    init_scene();mats=palette();carrito(mats);alas_carton(mats);mordecai(mats);rigby(mats);character_pivots();outlines();presentation()

if __name__=='__main__':build()
