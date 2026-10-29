"""Domo espacial de Regular Show: modelo procedural basado en las referencias del usuario."""
import bpy
import math
import random
import os
from math import sin, cos, pi, sqrt
from mathutils import Vector

ROOT=os.path.dirname(os.path.abspath(__file__))
REPO=os.path.dirname(os.path.dirname(os.path.dirname(ROOT)))
MODEL_DIR=os.path.join(REPO,'assets','models','planets','regularshow')
REFS=os.path.join(REPO,'docs','references','regularshow')
TAU=2*pi
R=7.2
MODEL='DOMO ESPACIAL | Regular Show'

def enum(obj, prop, wanted):
    values=[i.identifier for i in obj.bl_rna.properties[prop].enum_items]
    if wanted in values: setattr(obj,prop,next(v for v in values if v==wanted))

def coll(name,parent=None):
    c=bpy.data.collections.get(name)
    if c is None:
        c=bpy.data.collections.new(name)
        (parent or bpy.context.scene.collection).children.link(c)
    return c

def mat(name,color,rough=.72,metal=.0,emission=0):
    m=bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.use_nodes=True
    p=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    p.inputs['Base Color'].default_value=(*color,1)
    p.inputs['Roughness'].default_value=rough
    p.inputs['Metallic'].default_value=metal
    if emission:
        p.inputs['Emission Color'].default_value=(*color,1)
        p.inputs['Emission Strength'].default_value=emission
    m.diffuse_color=(*color,1)
    return m

class Mesh:
    def __init__(self): self.v=[]; self.f=[]; self.mi=[]; self.sm=[]
    def add(self,v,f,mi=0,smooth=False):
        off=len(self.v); self.v.extend(tuple(p) for p in v)
        self.f.extend(tuple(off+i for i in q) for q in f)
        self.mi.extend([mi]*len(f)); self.sm.extend([smooth]*len(f))
    def face(self,p,mi=0,smooth=False): self.add(p,[tuple(range(len(p)))],mi,smooth)
    def object(self,name,collection,mats):
        mesh=bpy.data.meshes.new(name); mesh.from_pydata(self.v,[],self.f); mesh.update()
        ob=bpy.data.objects.new(name,mesh); collection.objects.link(ob)
        for m in mats: mesh.materials.append(m)
        for p,i,s in zip(mesh.polygons,self.mi,self.sm): p.material_index=i; p.use_smooth=s
        return ob

def box(mesh,center,size,mi=0,angle=0):
    c=Vector(center); x=Vector((cos(angle),sin(angle),0)); y=Vector((-sin(angle),cos(angle),0)); z=Vector((0,0,1))
    points=[c+x*a*size[0]/2+y*b*size[1]/2+z*d*size[2]/2 for d in (-1,1) for b in (-1,1) for a in (-1,1)]
    mesh.add(points,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],mi)

def tube(mesh,points,radius,mi=0,sides=6,closed=False):
    pts=[Vector(p) for p in points]
    n=len(pts); verts=[]
    for i,p in enumerate(pts):
        before=pts[(i-1)%n] if closed or i>0 else pts[0]-(pts[1]-pts[0])
        after=pts[(i+1)%n] if closed or i<n-1 else pts[-1]+(pts[-1]-pts[-2])
        tangent=(after-before).normalized()
        ref=Vector((0,0,1)) if abs(tangent.z)<.92 else Vector((1,0,0))
        a=tangent.cross(ref).normalized(); b=tangent.cross(a).normalized()
        verts.extend(p+radius*(a*cos(TAU*j/sides)+b*sin(TAU*j/sides)) for j in range(sides))
    faces=[]
    for i in range(n if closed else n-1):
        for j in range(sides): faces.append((i*sides+j,i*sides+(j+1)%sides,((i+1)%n)*sides+(j+1)%sides,((i+1)%n)*sides+j))
    if not closed: faces.extend([tuple(reversed(range(sides))),tuple((n-1)*sides+j for j in range(sides))])
    mesh.add(verts,faces,mi,True)

def ring(mesh,r,z,tube_radius,mi=0,center=(0,0),steps=160):
    tube(mesh,[(center[0]+r*cos(TAU*i/steps),center[1]+r*sin(TAU*i/steps),z) for i in range(steps)],tube_radius,mi,8,True)

def lathe(mesh,profile,center=(0,0,0),mi=0,segments=64,smooth=True):
    verts=[(center[0]+r*cos(TAU*i/segments),center[1]+r*sin(TAU*i/segments),center[2]+z) for r,z in profile for i in range(segments)]
    faces=[]
    for j in range(len(profile)-1):
        for i in range(segments): faces.append((j*segments+i,j*segments+(i+1)%segments,(j+1)*segments+(i+1)%segments,(j+1)*segments+i))
    area=sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(profile,profile[1:]+profile[:1]))
    if area<0: faces=[tuple(reversed(f)) for f in faces]
    mesh.add(verts,faces,mi,smooth)

def ellipsoid(mesh,center,scale,mi=0,segments=12,rings=8,seed=0):
    rng=random.Random(seed); pts=[]
    for j in range(rings+1):
        lat=-pi/2+pi*j/rings
        for i in range(segments):
            a=TAU*i/segments
            noise=1+rng.uniform(-.06,.06)
            pts.append((center[0]+scale[0]*cos(lat)*cos(a)*noise,center[1]+scale[1]*cos(lat)*sin(a)*noise,center[2]+scale[2]*sin(lat)*noise))
    faces=[]
    for j in range(rings):
        for i in range(segments): faces.append((j*segments+i,j*segments+(i+1)%segments,(j+1)*segments+(i+1)%segments,(j+1)*segments+i))
    mesh.add(pts,faces,mi,True)

def ground(x,y): return .065+.025*sin(x*.8)*cos(y*.9)+.018*cos(x*.45+y*.7)

def iniciar():
    # El archivo de la Estrella de la Muerte ya está guardado. Se crea otra escena.
    previous=list(bpy.data.scenes)
    scene=bpy.data.scenes.new('Domo espacial - Regular Show')
    bpy.context.window.scene=scene
    for s in previous: bpy.data.scenes.remove(s)
    for ob in list(bpy.data.objects): bpy.data.objects.remove(ob,do_unlink=True)
    for c in list(bpy.data.collections): bpy.data.collections.remove(c)
    bpy.data.orphans_purge(do_recursive=True)
    main=coll(MODEL)
    root=bpy.data.objects.new('RegularShow_Planet_ROOT',None); main.objects.link(root)
    root.empty_display_size=1
    root['Referencia']='Domo espacial de Regular Show; dos imágenes suministradas por el usuario'
    root['Diametro']=14.4
    root['Escala relativa']='72 % del diámetro del modelo anterior de la Estrella de la Muerte'
    print('Nueva escena preparada; el proyecto anterior se conserva en su archivo.')

def base():
    main=coll(MODEL); c=coll('01 | Isla rocosa y borde',main)
    rocks=[mat('DOMO | Roca ocre %02d'%i,(.29+i*.019,.16+i*.013,.075+i*.008)) for i in range(10)]
    seams=mat('DOMO | Juntas de roca',(.105,.060,.028))
    rim=mat('DOMO | Borde de tierra',(.40,.27,.135))
    rng=random.Random(2016); count=32
    profile=[(7.24,.015),(7.32,-.29),(7.22,-.73),(6.85,-1.30),(6.26,-1.95),(5.48,-2.65),(4.47,-3.25),(3.15,-3.76),(1.60,-4.02),(0,-4.14)]
    offsets=[rng.uniform(-.22,.22) for i in range(count)]
    rings=[]
    for j,(radius,z) in enumerate(profile):
        row=[]
        for i in range(count):
            a=TAU*i/count
            wave=offsets[i]*(min(1,j*.55))
            rr=max(0,radius+wave)
            zz=z+(rng.uniform(-.13,.13) if j>1 and radius>0 else 0)
            row.append(Vector((rr*cos(a),rr*sin(a),zz)))
        rings.append(row)
    mesh=Mesh(); lines=Mesh()
    for j in range(len(rings)-1):
        for i in range(count):
            k=(i+1)%count
            mesh.face([rings[j][i],rings[j+1][i],rings[j+1][k],rings[j][k]],rng.choices(range(10),[1,2,3,5,7,7,5,3,2,1])[0])
    for j in range(1,len(rings)-1): tube(lines,[p+Vector((p.x,p.y,0)).normalized()*.006 for p in rings[j]],.012,0,5,True)
    for i in range(count): tube(lines,[row[i] for row in rings[:-1]],.012,0,5)
    mesh.object('DOMO | Casco rocoso escalonado',c,rocks)
    lines.object('DOMO | Grietas y divisiones de los bloques',c,[seams])
    lip=Mesh()
    lathe(lip,[(7.19,-.19),(7.31,-.19),(7.34,-.06),(7.34,.075),(7.18,.075),(7.19,-.19)],mi=0,segments=192,smooth=False)
    lip.object('DOMO | Cinturon superior de tierra',c,[rim])
    print('Base rocosa lista: 32 sectores y nueve niveles irregulares.')

def motores():
    c=coll('02 | Cuatro propulsores',coll(MODEL))
    mats=[mat('DOMO | Metal de los motores',(.225,.218,.19),.62,.28),
          mat('DOMO | Borde metalico claro',(.31,.305,.27),.58,.32),
          mat('DOMO | Interior de las toberas',(.030,.034,.035),.75,.2),
          mat('DOMO | Uniones de los motores',(.10,.105,.095),.72,.25)]
    bodies=Mesh(); rims=Mesh(); details=Mesh()
    for i in range(4):
        a=TAU*i/4; x=3.95*cos(a); y=3.95*sin(a)
        # Soporte insertado en la roca, cuello estrecho y campana de escape abierta.
        lathe(bodies,[(.47,-2.82),(.56,-3.16),(.51,-3.61),(.46,-4.05),(.44,-4.32)],(x,y,0),0,64)
        lathe(bodies,[(.44,-4.12),(.46,-4.37),(.56,-4.63),(.73,-4.94),(.95,-5.33),(1.04,-5.69),(1.035,-5.86),(.93,-5.89),(.89,-5.66),(.77,-5.32),(.61,-4.96),(.43,-4.64),(.30,-4.39)],(x,y,0),0,80)
        lathe(bodies,[(.30,-4.39),(0,-4.39)],(x,y,0),2,64)
        for rr,z,th in ((.55,-3.24,.055),(.505,-3.73,.043),(.46,-4.17,.045),(1.035,-5.78,.045)):
            ring(rims,rr,z,th,1,(x,y),96)
        for k in range(16):
            t=TAU*k/16
            points=[(x+r*cos(t),y+r*sin(t),z) for r,z in ((.537,-3.25),(.51,-3.49),(.48,-3.73),(.46,-4.03))]
            tube(details,points,.018,3,5)
        for k in range(8):
            t=TAU*k/8
            tube(details,[(x+r*cos(t),y+r*sin(t),z) for r,z in ((.47,-4.4),(.57,-4.65),(.74,-4.96),(.96,-5.35),(1.05,-5.72))],.010,3,5)
        # Abrazaderas rectangulares que anclan los motores bajo el casco.
        for sign in (-1,1): box(details,(x+sign*.48,y,-3.12),(.13,.40,.58),1)
    bodies.object('DOMO | Cuatro campanas y cuellos de propulsion',c,mats)
    rims.object('DOMO | Aros de las toberas',c,mats)
    details.object('DOMO | Estrias y soportes mecanicos',c,mats)
    print('Cuatro propulsores construidos, con campanas huecas y soportes.')

def parque():
    c=coll('03 | Parque, praderas y senderos',coll(MODEL))
    greens=[mat('DOMO | Pradera %02d'%i,(.23+i*.012,.39+i*.011,.055+i*.006)) for i in range(9)]
    soil=mat('DOMO | Tierra bajo el cesped',(.27,.22,.095))
    pathmat=mat('DOMO | Arena de los senderos',(.64,.565,.32))
    pathborder=mat('DOMO | Bordes del camino',(.39,.38,.19))
    water=mat('DOMO | Agua del estanque',(.095,.34,.39),.19,.20)
    rng=random.Random(154)
    grass=Mesh()
    n=160; layers=18
    verts=[]
    for j in range(layers+1):
        r=7.16*j/layers
        for i in range(n):
            a=TAU*i/n; x=r*cos(a);y=r*sin(a)
            verts.append((x,y,ground(x,y)))
    for j in range(layers):
        for i in range(n):
            ids=(j*n+i,(j+1)*n+i,(j+1)*n+(i+1)%n,j*n+(i+1)%n)
            # La variación es suave y se organiza en manchas de pradera.
            p=Vector(verts[ids[0]])
            color=max(0,min(8,round(4+1.3*sin(p.x*.6)+1.1*cos(p.y*.5)+rng.uniform(-.5,.5))))
            grass.face([verts[k] for k in ids],color,True)
    grass.object('DOMO | Suelo verde del parque',c,greens)
    edge=Mesh(); lathe(edge,[(0,-.025),(7.18,-.025),(7.18,.055),(0,.055)],mi=0,segments=192)
    edge.object('DOMO | Espesor del terreno',c,[soil])
    walks=Mesh(); borders=Mesh()
    def strip(points,width):
        for i in range(len(points)-1):
            p=Vector((*points[i],0)); q=Vector((*points[i+1],0));d=q-p
            side=Vector((-d.y,d.x,0)).normalized()*width/2
            vv=[p-side,q-side,q+side,p+side]
            for v in vv:v.z=ground(v.x,v.y)+.012
            walks.face(vv)
        for sign in (-1,1):
            pp=[]
            for i,p in enumerate(points):
                a=Vector((*points[max(0,i-1)],0));b=Vector((*points[min(len(points)-1,i+1)],0));d=b-a
                side=Vector((-d.y,d.x,0)).normalized()*width/2*sign
                x=p[0]+side.x;y=p[1]+side.y;pp.append((x,y,ground(x,y)+.018))
            tube(borders,pp,.009,0,5)
    # Paseo oval alrededor de un amplio césped central.
    loop=[(4.65*cos(TAU*i/180),3.6*sin(TAU*i/180)-.25) for i in range(181)]
    strip(loop,.16)
    points=[(-5.55+11.1*i/100,-1.8+.55*sin(i/100*pi*1.5)) for i in range(101)]
    strip(points,.18)
    points=[(-.85+.34*sin(i/40*pi),-1.29+2.6*i/40) for i in range(41)]
    strip(points,.17)
    points=[(2.0+2.8*i/40,2.1-2.7*i/40) for i in range(41)]
    strip(points,.13)
    walks.object('DOMO | Red de senderos curvos',c,[pathmat])
    borders.object('DOMO | Bordes discretos de los senderos',c,[pathborder])
    pond=Mesh(); bank=Mesh(); center=(2.75,1.75)
    pts=[]
    for i in range(96):
        a=TAU*i/96;r=.72*(1+.14*sin(a*3)+.07*cos(a*5))
        pts.append((center[0]+r*cos(a),center[1]+r*.64*sin(a),.116))
    for i in range(96):pond.face([(center[0],center[1],.116),pts[i],pts[(i+1)%96]])
    tube(bank,pts,.035,0,7,True)
    pond.object('DOMO | Estanque pequeno',c,[water]);bank.object('DOMO | Orilla del estanque',c,[pathmat])
    print('Parque listo: praderas, senderos curvos y estanque.')

def casa():
    c=coll('04 | Casa del parque y mobiliario',coll(MODEL))
    mats=[mat('DOMO | Casa paredes marfil',(.78,.775,.67)),
          mat('DOMO | Casa columnas blancas',(.88,.87,.77)),
          mat('DOMO | Casa tejado azul gris',(.105,.165,.205)),
          mat('DOMO | Casa ventanas',(.07,.19,.235),.29,.15),
          mat('DOMO | Casa marcos oscuros',(.135,.20,.22)),
          mat('DOMO | Madera de bancos',(.36,.20,.095)),
          mat('DOMO | Tejado de caseta',(.34,.12,.065)),
          mat('DOMO | Casa puerta',(.34,.28,.16))]
    m=Mesh(); detail=Mesh(); roof=Mesh()
    ox,oy=-.90,1.65; basez=ground(ox,oy)
    def b(x,y,z,sx,sy,sz,mi=0):box(m,(ox+x,oy+y,basez+z),(sx,sy,sz),mi)
    # Casa de dos niveles, porche, columnas y buhardilla frontal.
    b(0,0,.16,1.50,1.06,.22,1)
    b(0,.05,.75,1.28,.83,1.10,0)
    b(-.65,.05,.51,.45,.68,.66,0)
    b(.65,.05,.51,.45,.68,.66,0)
    b(0,-.57,.25,1.55,.46,.12,1)
    b(0,-.87,.12,.68,.24,.09,1)
    b(0,-.78,.17,.86,.18,.09,1)
    for x in (-.68,-.25,.25,.68): b(x,-.68,.56,.047,.047,.56,1)
    b(0,-.56,.88,1.60,.52,.075,2)
    for x in (-.67,.67):
        for z in (.37,.52):b(x,-.68,z,.035,.035,.05,1)
    for x in (-.46,.46):
        b(x,-.68,.42,.40,.023,.034,1)
    # Cubierta a dos aguas, con cumbrera paralela al eje del fondo.
    def gable(cx,cy,z,w,d,h):
        vv=[(ox+cx-w/2,oy+cy-d/2,basez+z),(ox+cx+w/2,oy+cy-d/2,basez+z),
            (ox+cx,oy+cy-d/2,basez+z+h),(ox+cx-w/2,oy+cy+d/2,basez+z),
            (ox+cx+w/2,oy+cy+d/2,basez+z),(ox+cx,oy+cy+d/2,basez+z+h)]
        roof.add(vv,[(0,3,5,2),(2,5,4,1),(0,2,1),(3,4,5)],2)
    gable(0,.05,1.30,1.50,1.08,.43)
    gable(-.65,.06,.86,.66,.86,.24);gable(.65,.06,.86,.66,.86,.24)
    b(0,-.39,1.37,.34,.30,.30,0);gable(0,-.41,1.52,.43,.40,.19)
    b(.43,.18,1.60,.13,.15,.38,0)
    for x,z in ((-.40,.58),(.40,.58),(-.40,1.05),(.40,1.05),(0,1.41)):
        y=-.375 if z<1.3 else -.555
        b(x,y-.009,z,.21,.026,.25 if z<1.3 else .16,4)
        b(x,y-.025,z,.162,.014,.205 if z<1.3 else .12,3)
        b(x,y-.036,z,.014,.012,.21 if z<1.3 else .13,1)
        b(x,y-.036,z,.175,.012,.014,1)
    b(0,-.385,.50,.21,.04,.50,7)
    for x in (-.655,.655):
        for z in (.54,1.02):
            b(x,0,z,.023,.21,.24,4);b(x*1.015,0,z,.012,.165,.20,3)
    # Líneas de revestimiento en la fachada, sin tapar las ventanas.
    for z in (.29,.36,.70,.78,.94,1.22):b(0,-.373,z,1.25,.008,.007,1)
    # Pequeña caseta de mantenimiento lejos del edificio principal.
    box(m,(-3.6,2.9,.34),(.50,.42,.48),0)
    box(m,(-3.6,2.9,.61),(.59,.51,.10),6)
    box(m,(-3.6,2.68,.29),(.14,.02,.30),7)
    # Bancos de parque y mesas de picnic.
    for x,y,a in ((-3.7,-1.55,.35),(3.6,-2.0,-.5),(1.45,3.15,0),(-4.5,1.4,1.2),(4.6,.1,1.4)):
        z=ground(x,y)
        for h,sz in ((.14,(.34,.10,.035)),(.28,(.34,.035,.095))):box(detail,(x,y+(0 if h<.2 else .065),z+h),sz,5,a)
        for dx in (-.12,.12):box(detail,(x+dx*cos(a),y+dx*sin(a),z+.065),(.025,.07,.12),4,a)
    for x,y in ((-2.1,-3.6),(2.3,3.45)):
        z=ground(x,y)
        box(detail,(x,y,z+.21),(.45,.27,.045),5)
        for sy in (-.22,.22):box(detail,(x,y+sy,z+.12),(.45,.08,.035),5)
        for dx in (-.14,.14):box(detail,(x+dx,y,z+.105),(.035,.22,.20),4)
    # Fuente baja circular en un claro junto al camino.
    stone=mat('DOMO | Piedra de la fuente',(.56,.57,.44))
    fountain=Mesh()
    lathe(fountain,[(.33,.10),(.34,.13),(.32,.22),(.26,.22),(.25,.13)],(1.4,-.05,0),0,64)
    lathe(fountain,[(0,.16),(.25,.16)],(1.4,-.05,0),1,64)
    lathe(fountain,[(.055,.16),(.055,.40),(.12,.41),(.12,.44),(0,.44)],(1.4,-.05,0),0,48)
    fountain.object('DOMO | Fuente circular del parque',c,[stone,bpy.data.materials['DOMO | Agua del estanque']])
    house=m.object('DOMO | Casa principal, porche y caseta',c,mats)
    rooftops=roof.object('DOMO | Tejados y buhardillas',c,mats)
    pivot=Vector((ox,oy,basez))
    for ob in (house,rooftops):
        for vertex in ob.data.vertices:
            if vertex.co.x>-2.0:vertex.co=pivot+(vertex.co-pivot)*.68
    detail.object('DOMO | Bancos y mesas del parque',c,mats)
    print('Casa con porche, buhardillas, fuente y mobiliario lista.')

def arboles():
    c=coll('05 | Arboles y vegetacion',coll(MODEL))
    leaves=[mat('DOMO | Follaje %02d'%i,(.10+i*.019,.22+i*.024,.035+i*.007)) for i in range(9)]
    trunks=[mat('DOMO | Tronco oscuro',(.20,.135,.065)),mat('DOMO | Tronco claro',(.30,.22,.10))]
    rng=random.Random(2929); locations=[]
    # Una arboleda continua sigue el perímetro y deja ver los claros interiores.
    for i in range(172):
        a=TAU*i/172+rng.uniform(-.008,.008);r=rng.uniform(6.22,6.85)
        locations.append((r*cos(a),r*sin(a),rng.uniform(.65,1.0)))
    tries=0
    while len(locations)<340 and tries<10000:
        tries+=1
        x=rng.uniform(-6,6);y=rng.uniform(-5.9,5.9)
        if x*x+y*y>36:continue
        if (x+.90)**2/1.7**2+(y-1.65)**2/1.45**2<1:continue
        if (x-2.75)**2+(y-1.75)**2<1.0:continue
        if (x-1.4)**2+(y+.05)**2<.40:continue
        if (x/3.2)**2+((y+.4)/1.45)**2<1 and rng.random()<.91:continue
        # Se respetan los senderos principales.
        if abs((x/4.65)**2+((y+.25)/3.6)**2-1)<.10:continue
        if abs(y-(-1.8+.55*sin((x+5.55)/11.1*pi*1.5)))<.23:continue
        if min((x-p[0])**2+(y-p[1])**2 for p in locations)<.18**2:continue
        locations.append((x,y,rng.uniform(.70,1.20)))
    canopy=Mesh(); wood=Mesh(); shrubs=Mesh()
    for i,(x,y,s) in enumerate(locations):
        s*=.82
        z=ground(x,y);h=.32*s;radius=.15*s
        tube(wood,[(x,y,z),(x+.012*s,y,z+h+.09*s)],.024*s,i%2,7)
        for sign in (-1,1):tube(wood,[(x,y,z+h*.65),(x+sign*.10*s,y+.02,z+h+.04*s)],.011*s,i%2,6)
        color=rng.randrange(9)
        if i%13==0:
            # Árboles estilizados altos entre las copas redondeadas.
            lathe(canopy,[(0,.76*s),(.11*s,.65*s),(.14*s,.46*s),(.12*s,.28*s),(0,.26*s)],(x,y,z),color,10)
        else:
            for j,(dx,dy,dz,sc) in enumerate(((0,0,.43,1),(-.10,.025,.36,.76),(.095,.025,.37,.80))):
                ellipsoid(canopy,(x+dx*s,y+dy*s,z+dz*s),(.18*s*sc,.16*s*sc,.205*s*sc),color if j==0 else min(8,color+1),12,7,i*13+j)
    for i in range(95):
        a=rng.uniform(0,TAU);r=rng.uniform(5.2,6.7);x=r*cos(a);y=r*sin(a);z=ground(x,y)
        ellipsoid(shrubs,(x,y,z+.08),(.11,.09,.10),rng.randrange(9),10,5,i)
    canopy.object('DOMO | Copas de 340 arboles del parque',c,leaves)
    wood.object('DOMO | Troncos y ramas',c,trunks)
    shrubs.object('DOMO | Setos y arbustos bajos',c,leaves)
    bpy.context.scene['Cantidad de arboles']=len(locations)
    print('Vegetacion lista:',len(locations),'arboles y 95 arbustos.')

def cupula():
    c=coll('06 | Cupula transparente y reticula',coll(MODEL))
    glass=bpy.data.materials.new('DOMO | Vidrio transparente de la cupula');glass.use_nodes=True
    nt=glass.node_tree;nt.nodes.clear()
    out=nt.nodes.new('ShaderNodeOutputMaterial')
    trans=nt.nodes.new('ShaderNodeBsdfTransparent');trans.inputs[0].default_value=(.96,.995,1,1)
    p=nt.nodes.new('ShaderNodeBsdfPrincipled');p.inputs['Base Color'].default_value=(.46,.70,.73,1)
    p.inputs['Roughness'].default_value=.055;p.inputs['Metallic'].default_value=.0
    p.inputs['Transmission Weight'].default_value=.85;p.inputs['IOR'].default_value=1.10
    mix=nt.nodes.new('ShaderNodeMixShader');mix.inputs[0].default_value=.065
    nt.links.new(trans.outputs[0],mix.inputs[1]);nt.links.new(p.outputs[0],mix.inputs[2]);nt.links.new(mix.outputs[0],out.inputs['Surface'])
    glass.diffuse_color=(.53,.77,.78,.11)
    enum(glass,'surface_render_method','DITHERED')
    glass.use_transparent_shadow=True
    gridmat=mat('DOMO | Reticula fina del vidrio',(.15,.28,.28),.65,.0)
    gnt=gridmat.node_tree;gp=next(n for n in gnt.nodes if n.type=='BSDF_PRINCIPLED');go=next(n for n in gnt.nodes if n.type=='OUTPUT_MATERIAL')
    gt=gnt.nodes.new('ShaderNodeBsdfTransparent');gm=gnt.nodes.new('ShaderNodeMixShader');gm.inputs[0].default_value=.5
    gnt.links.new(gt.outputs[0],gm.inputs[1]);gnt.links.new(gp.outputs[0],gm.inputs[2]);gnt.links.new(gm.outputs[0],go.inputs['Surface'])
    rim=mat('DOMO | Junta verde de la cupula',(.36,.50,.31),.60,.12)
    shell=Mesh(); seg=192; rows=80;verts=[]
    for j in range(rows+1):
        lat=pi/2*j/rows
        for i in range(seg):
            a=TAU*i/seg;verts.append((R*cos(lat)*cos(a),R*cos(lat)*sin(a),.12+6.55*sin(lat)))
    faces=[]
    for j in range(rows):
        for i in range(seg):faces.append((j*seg+i,j*seg+(i+1)%seg,(j+1)*seg+(i+1)%seg,(j+1)*seg+i))
    shell.add(verts,faces,0,True)
    ob=shell.object('DOMO | Hemisferio de vidrio transparente',c,[glass])
    ob['Exportacion OBJ']='Transparencia en el material MTL; el shader completo se conserva en Blender'
    grid=Mesh()
    for i in range(32):
        a=TAU*i/32
        pts=[]
        for j in range(81):
            lat=pi/2*j/80;pts.append(((R+.007)*cos(lat)*cos(a),(R+.007)*cos(lat)*sin(a),.12+6.557*sin(lat)))
        tube(grid,pts,.007,0,5)
    for j in range(1,15):
        lat=pi/2*j/15
        ring(grid,(R+.007)*cos(lat),.12+6.557*sin(lat),.007,0,steps=192)
    grid.object('DOMO | Meridianos y paralelos de la cupula',c,[gridmat])
    border=Mesh();ring(border,R,.12,.038,0,steps=224)
    ring(border,R-.08,.10,.018,0,steps=224)
    border.object('DOMO | Anillo sellado del hemisferio',c,[rim])
    print('Cupula transparente lista: 32 meridianos y 14 paralelos finos.')

def presentacion():
    scene=bpy.context.scene
    stage=coll('90 | Presentacion espacial (no exportar)')
    world=bpy.data.worlds.new('DOMO | Nebulosa morada suave');world.use_nodes=True
    nt=world.node_tree;bg=next(n for n in nt.nodes if n.type=='BACKGROUND')
    tex=nt.nodes.new('ShaderNodeTexCoord');noise=nt.nodes.new('ShaderNodeTexNoise')
    noise.inputs['Scale'].default_value=3.5;noise.inputs['Detail'].default_value=4;noise.inputs['Roughness'].default_value=.72
    nt.links.new(tex.outputs['Normal'],noise.inputs['Vector'])
    ramp=nt.nodes.new('ShaderNodeValToRGB')
    ramp.color_ramp.elements[0].position=.25;ramp.color_ramp.elements[0].color=(.001,.001,.007,1)
    ramp.color_ramp.elements[1].position=.78;ramp.color_ramp.elements[1].color=(.045,.006,.078,1)
    mid=ramp.color_ramp.elements.new(.52);mid.color=(.006,.004,.025,1)
    nt.links.new(noise.outputs['Fac'],ramp.inputs[0]);nt.links.new(ramp.outputs[0],bg.inputs['Color'])
    bg.inputs['Strength'].default_value=.45;scene.world=world
    d=bpy.data.cameras.new('DOMO | Camara principal');camera=bpy.data.objects.new('DOMO | Camara principal',d);stage.objects.link(camera)
    camera.location=(10,-31,9.5);target=Vector((0,0,.10));camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler()
    enum(d,'type','ORTHO');d.ortho_scale=18.8;d.clip_end=500;scene.camera=camera
    def area(name,pos,power,size,color):
        typ=next(i.identifier for i in bpy.types.Light.bl_rna.properties['type'].enum_items if i.identifier=='AREA')
        d=bpy.data.lights.new(name,typ);d.energy=power;d.size=size;d.color=color
        ob=bpy.data.objects.new(name,d);stage.objects.link(ob);ob.location=pos;ob.rotation_euler=(-ob.location).to_track_quat('-Z','Y').to_euler()
    area('DOMO | Luz principal suave',(-12,-15,24),7200,12,(1,.95,.84))
    area('DOMO | Relleno del parque',(12,-8,10),2800,16,(.84,.94,1))
    area('DOMO | Luz de contorno',(3,13,15),5000,12,(.69,.85,1))
    area('DOMO | Rebote bajo la isla',(0,-18,-7),1800,12,(1,.88,.73))
    star_mats=[mat('DOMO | Estrella blanca',(.81,.88,1),.5,0,2.2),mat('DOMO | Estrella calida',(1,.85,.67),.5,0,1.6),mat('DOMO | Estrella azul',(.50,.65,1),.5,0,2)]
    star=Mesh();rng=random.Random(716)
    rotation=camera.rotation_euler.to_quaternion();right=rotation@Vector((1,0,0));up=rotation@Vector((0,1,0));normal=rotation@Vector((0,0,1))
    center=-normal*25
    for i in range(210):
        p=center+right*rng.uniform(-13,13)+up*rng.uniform(-13,13)
        radius=rng.uniform(.008,.025)*(1.7 if i%19==0 else 1)
        v=[p]+[p+right*cos(TAU*k/8)*radius+up*sin(TAU*k/8)*radius for k in range(8)]
        star.add(v,[(0,k+1,(k+1)%8+1) for k in range(8)],rng.choices([0,1,2],[7,1,2])[0])
    star.object('DOMO | Fondo de estrellas (solo presentacion)',stage,star_mats)
    try:scene.render.engine='CYCLES'
    except TypeError as e:print(e)
    if scene.render.engine=='CYCLES':
        scene.cycles.samples=64;scene.cycles.use_denoising=True
        scene.cycles.max_bounces=10;scene.cycles.transparent_max_bounces=20
        try:
            prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.get_devices()
            found=next((d.type for d in prefs.devices if d.type=='OPTIX'),None)
            if found:
                prefs.compute_device_type=found;prefs.get_devices()
                for device in prefs.devices:device.use=device.type==found
                enum(scene.cycles,'device','GPU')
        except Exception as e:print('GPU:',e)
    scene.render.resolution_x=1600;scene.render.resolution_y=1600;scene.render.resolution_percentage=65
    enum(scene.render.image_settings,'file_format','PNG');enum(scene.render.image_settings,'color_mode','RGB')
    scene.render.filepath=os.path.join(REFS,'regularshow_preview.png')
    scene.view_settings.exposure=.0
    root=bpy.data.objects['RegularShow_Planet_ROOT']
    for ob in bpy.data.collections[MODEL].all_objects:
        if ob!=root:ob.parent=root
    for ob in bpy.context.selected_objects:ob.select_set(False)
    root.select_set(True);bpy.context.view_layer.objects.active=root
    for screen in bpy.data.screens:
        for ar in screen.areas:
            if ar.type=='VIEW_3D':
                ar.spaces.active.region_3d.view_rotation=camera.rotation_euler.to_quaternion()
                ar.spaces.active.region_3d.view_location=(0,0,.10);ar.spaces.active.region_3d.view_distance=27
                enum(ar.spaces.active.region_3d,'view_perspective','CAMERA')
                enum(ar.spaces.active.shading,'type','MATERIAL')
                ar.spaces.active.shading.use_scene_world=True;ar.spaces.active.shading.use_scene_lights=True
                ar.spaces.active.overlay.show_overlays=False
    text=bpy.data.texts.new('LEEME | Domo espacial')
    text.write('DOMO ESPACIAL DE REGULAR SHOW\n\nBasado en las dos imagenes de referencia proporcionadas por el usuario.\nDiametro: 14.4 unidades, equivalente al 72 % del modelo de la Estrella de la Muerte.\nCupula transparente con reticula fina. Parque con 340 arboles, caminos, casa de dos pisos, fuente, estanque y bancos.\nBase rocosa escalonada y cuatro propulsores abiertos.\nEl control general transforma el modelo entero. La coleccion 90 es solo presentacion y no pertenece al OBJ.\nEl shader completo del vidrio se conserva en Blender; OBJ representa la transparencia mediante MTL.\n')
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(MODEL_DIR,'regularshow_space_dome_planet.blend'))
    print('Proyecto guardado. Mallas:',sum(ob.type=='MESH' for ob in bpy.data.collections[MODEL].all_objects))
    print('Poligonos:',sum(len(ob.data.polygons) for ob in bpy.data.collections[MODEL].all_objects if ob.type=='MESH'))

if __name__=='__main__':
    iniciar();base();motores();parque();casa();arboles();cupula()
    import importlib.util
    spec=importlib.util.spec_from_file_location('domo_mejoras',os.path.join(ROOT,'mejorar_casa_y_motores.py'))
    mejoras=importlib.util.module_from_spec(spec);spec.loader.exec_module(mejoras)
    mejoras.aplicar()
    spec=importlib.util.spec_from_file_location('domo_zonas',os.path.join(ROOT,'agregar_zonas_del_parque.py'))
    zonas=importlib.util.module_from_spec(spec);spec.loader.exec_module(zonas)
    zonas.apply()
    spec=importlib.util.spec_from_file_location('domo_escala',os.path.join(ROOT,'ajustar_escala_secundaria.py'))
    escala=importlib.util.module_from_spec(spec);spec.loader.exec_module(escala)
    escala.aplicar()
    presentacion()
