"""Estrella de la Muerte I: construcción procedural editable en Blender.

Ejecutar por fases desde el MCP: estructura(), detalles(), superlaser(), escena().
No requiere assets externos ni generadores de pago.
"""
import bpy
import math
import random
import os
from mathutils import Vector, Quaternion
from math import sin, cos, pi, sqrt, atan2, asin

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(os.path.dirname(SCRIPT_DIR)))
ROOT = os.path.join(REPO,'assets','models','planets','starwars')
REFS = os.path.join(REPO,'docs','references','starwars')
R = 10.0
TAU = 2*pi
AXIS = Vector((-0.24, -0.843, 0.48)).normalized()
U = (Vector((1, 0, 0))-AXIS*AXIS.x).normalized()
V = AXIS.cross(U).normalized()
APERTURE = 0.293
CUT = cos(APERTURE)

def enum(obj, prop, wanted):
    opts = [x.identifier for x in obj.bl_rna.properties[prop].enum_items]
    if wanted in opts:
        setattr(obj, prop, next(x for x in opts if x == wanted))

def collection(name, parent=None):
    c = bpy.data.collections.get(name)
    if c is None:
        c = bpy.data.collections.new(name)
        (parent or bpy.context.scene.collection).children.link(c)
    return c

def material(name, color, metallic=0.35, roughness=0.65, emission=0):
    m = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.use_nodes = True
    p = next(x for x in m.node_tree.nodes if x.type == 'BSDF_PRINCIPLED')
    p.inputs['Base Color'].default_value = (*color, 1)
    p.inputs['Metallic'].default_value = metallic
    p.inputs['Roughness'].default_value = roughness
    if emission:
        p.inputs['Emission Color'].default_value = (*color, 1)
        p.inputs['Emission Strength'].default_value = emission
    m.diffuse_color = (*color, 1)
    return m

def palette():
    mats = []
    for i in range(12):
        g = 0.19 + i*0.013
        mats.append(material('DS | Blindaje %02d'%i, (g*0.965, g*0.985, g), .42, .66))
    return mats

class Mesh:
    def __init__(self):
        self.verts=[]; self.faces=[]; self.ids=[]; self.smooth=[]
    def face(self, points, mat=0, smooth=False):
        k=len(self.verts)
        self.verts.extend(tuple(p) for p in points)
        self.faces.append(tuple(range(k,k+len(points))))
        self.ids.append(mat); self.smooth.append(smooth)
    def object(self, name, coll, mats, uv=False):
        m=bpy.data.meshes.new(name)
        m.from_pydata(self.verts, [], self.faces); m.update()
        o=bpy.data.objects.new(name,m); coll.objects.link(o)
        for mat in mats: m.materials.append(mat)
        for poly,mi,sm in zip(m.polygons,self.ids,self.smooth):
            poly.material_index=mi; poly.use_smooth=sm
        if any(self.smooth) and ('esferica' in name or 'blindaje' in name or 'Reflector concavo' in name or 'Sectores radiales' in name):
            normals=[]
            dish=('Reflector concavo' in name or 'Sectores radiales' in name)
            for poly in m.polygons:
                for vi in poly.vertices:
                    p=Vector(m.vertices[vi].co)
                    if not poly.use_smooth: normals.append((0,0,0))
                    elif dish:
                        radial=p-AXIS*p.dot(AXIS)
                        normals.append(tuple((AXIS-radial*(2*1.12/(sin(APERTURE)*R)**2)).normalized()))
                    else: normals.append(tuple(p.normalized()))
            m.normals_split_custom_set(normals)
        if uv:
            lay=m.uv_layers.new(name='Mapa esferico')
            for poly in m.polygons:
                us=[]; vs=[]
                for idx in poly.vertices:
                    p=Vector(m.vertices[idx].co).normalized()
                    us.append((atan2(p.y,p.x)/TAU)%1)
                    vs.append(asin(max(-1,min(1,p.z)))/pi+.5)
                if max(us)-min(us)>.5: us=[u+1 if u<.5 else u for u in us]
                for li,u,v in zip(poly.loop_indices,us,vs): lay.data[li].uv=(u,v)
        return o

def direction(lat, lon):
    return Vector((cos(lat)*cos(lon), cos(lat)*sin(lon), sin(lat)))

def clip(points):
    """Recorta una cara contra el círculo del superláser sobre la esfera."""
    out=[]
    for a,b in zip(points,points[1:]+points[:1]):
        da=a.normalized().dot(AXIS)-CUT
        db=b.normalized().dot(AXIS)-CUT
        if da<=0: out.append(a)
        if (da<0) != (db<0):
            # Bisección en arco normalizado; precisión estable en cualquier radio.
            lo=0.; hi=1.
            for _ in range(18):
                t=(lo+hi)/2
                q=a.lerp(b,t)
                if (q.normalized().dot(AXIS)-CUT>0)==(da>0): lo=t
                else: hi=t
            t=(lo+hi)/2
            out.append(a.lerp(b,t).normalized()*a.length)
    return out

def patch(mesh, la, lb, pa, pb, radius, mi, thickness=0, smooth=True):
    nx=max(1,math.ceil((pb-pa)*max(.15,cos((la+lb)/2))/.025))
    ny=max(1,math.ceil((lb-la)/.025))
    for j in range(ny):
        aa=la+(lb-la)*j/ny; bb=la+(lb-la)*(j+1)/ny
        for k in range(nx):
            cc=pa+(pb-pa)*k/nx; dd=pa+(pb-pa)*(k+1)/nx
            pts=clip([direction(aa,cc)*radius,direction(aa,dd)*radius,
                      direction(bb,dd)*radius,direction(bb,cc)*radius])
            if len(pts)>=3: mesh.face(pts,mi,smooth)
    if thickness:
        # Los cantos laterales dan relieve real al blindaje.
        for side in range(4):
            count=nx if side in (0,2) else ny
            for k in range(count):
                t0=k/count; t1=(k+1)/count
                if side==0: a=direction(la,pa+(pb-pa)*t0); b=direction(la,pa+(pb-pa)*t1)
                elif side==1: a=direction(la+(lb-la)*t0,pb); b=direction(la+(lb-la)*t1,pb)
                elif side==2: a=direction(lb,pb-(pb-pa)*t0); b=direction(lb,pb-(pb-pa)*t1)
                else: a=direction(lb-(lb-la)*t0,pa); b=direction(lb-(lb-la)*t1,pa)
                if a.dot(AXIS)<CUT and b.dot(AXIS)<CUT:
                    mesh.face([a*(radius-thickness),b*(radius-thickness),b*radius,a*radius],mi,False)

def box(mesh, center, ax, ay, az, sx, sy, sz, mi=0):
    pts=[center+ax*(x*sx/2)+ay*(y*sy/2)+az*(z*sz/2)
         for z in (-1,1) for y in (-1,1) for x in (-1,1)]
    for f in ((0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)):
        mesh.face([pts[i] for i in f],mi)

def spherebox(mesh, lat, lon, w, h, depth, radius, mi=0):
    n=direction(lat,lon)
    tx=Vector((-sin(lon),cos(lon),0))
    ty=n.cross(tx)
    corners=[(n*radius+tx*x*w/2+ty*y*h/2).normalized() for x in (-1,1) for y in (-1,1)]
    if any(c.dot(AXIS)>CUT-.002 for c in corners): return
    box(mesh,n*(radius+depth/2),tx,ty,n,w,h,depth,mi)

def ring(mesh, axis, u, v, axial, radius, width, height, mi=0, count=384):
    for i in range(count):
        a=TAU*i/count; b=TAU*(i+1)/count
        p=[]
        for z in (axial-height/2,axial+height/2):
            for r in (radius-width/2,radius+width/2):
                p.extend([axis*z+(u*cos(t)+v*sin(t))*r for t in (a,b)])
        for f in ((0,1,3,2),(4,6,7,5),(0,4,5,1),(2,3,7,6)):
            mesh.face([p[j] for j in f],mi,True)

def estructura():
    # Conserva los objetos originales fuera de la composición.
    original=collection('00 | Escena original (conservada)')
    for o in list(bpy.context.scene.objects):
        if not o.name.startswith('DS |'):
            if o.name not in original.objects:
                original.objects.link(o)
                for c in list(o.users_collection):
                    if c!=original: c.objects.unlink(o)
    original.hide_render=True; original.hide_viewport=True
    main=collection('ESTRELLA DE LA MUERTE | DS-1')
    hull=collection('01 | Casco y placas',main)
    palette_m=palette()
    dark=material('DS | Interior de las juntas',(.082,.084,.085),.38,.78)
    shell=Mesh()
    # Dos casquetes separados: la trinchera ecuatorial es una abertura geométrica.
    for sign in (-1,1):
        for j in range(96):
            a=.017+(pi/2-.017)*j/96; b=.017+(pi/2-.017)*(j+1)/96
            la,lb=(a,b) if sign>0 else (-b,-a)
            for i in range(384):
                pa=TAU*i/384; pb=TAU*(i+1)/384
                pts=clip([direction(la,pa)*9.966,direction(la,pb)*9.966,
                          direction(lb,pb)*9.966,direction(lb,pa)*9.966])
                if len(pts)>=3: shell.face(pts,0,True)
    shell.object('DS | Estructura esferica con abertura del superlaser',hull,[dark],uv=True)
    rng=random.Random(1977)
    plating=Mesh(); total=0
    # Filas escalonadas con anchos irregulares, canales anchos y paneles finos.
    for sign in (-1,1):
        lat=.018
        while lat<pi/2-.006:
            row=rng.uniform(.013,.030)
            if rng.random()<.12: row=rng.uniform(.006,.010)
            end=min(pi/2-.001,lat+row)
            mid=(lat+end)/2
            sectors=max(12,round(TAU*cos(mid)/rng.uniform(.055,.10)))
            offset=rng.random()*TAU/sectors
            for i in range(sectors):
                pa=TAU*i/sectors+offset+.00065
                pb=TAU*(i+1)/sectors+offset-.00065
                la,lb=(lat,end) if sign>0 else (-end,-lat)
                mi=rng.choices(range(12),weights=[1,2,4,7,10,12,12,10,7,4,2,1])[0]
                rr=10+rng.uniform(-.005,.013)
                patch(plating,la+.00055,lb-.00055,pa,pb,rr,mi,.042)
                total+=1
            lat=end+rng.uniform(.0010,.0024)
    plating.object('DS | Miles de placas de blindaje independientes',hull,palette_m,uv=True)
    bpy.context.scene['Modelo']='Estrella de la Muerte I / DS-1'
    bpy.context.scene['Construccion']='Geometría y texturas procedurales; sin assets externos'
    print('ESTRUCTURA LISTA. Placas:',total,'caras:',len(plating.faces),'vertices:',len(plating.verts))

def textura():
    import numpy as np
    rng=np.random.default_rng(1977)
    W,H=4096,2048
    al=np.full((H,W),.63,dtype=np.float32)
    ht=np.full((H,W),.52,dtype=np.float32)
    y=0
    while y<H:
        row=int(rng.integers(7,22)); y1=min(H,y+row)
        x=-int(rng.integers(0,55))
        while x<W:
            wid=int(rng.integers(12,66)); x1=min(W,x+wid)
            xx=max(0,x)
            tone=float(rng.uniform(.49,.76))
            al[y:y1,xx:x1]=tone
            ht[y:y1,xx:x1]=float(rng.uniform(.46,.56))
            al[y:y+1,xx:x1]=.22; ht[y:y+1,xx:x1]=.08
            if x>=0: al[y:y1,x:x+1]=.23; ht[y:y1,x:x+1]=.10
            if row>8 and x1-xx>12:
                for _ in range(int(rng.integers(1,6))):
                    px=int(rng.integers(xx+2,max(xx+3,x1-2)))
                    py=int(rng.integers(y+2,max(y+3,y1-2)))
                    pw=min(int(rng.integers(1,8)),x1-px-1)
                    ph=min(int(rng.integers(1,4)),y1-py-1)
                    al[py:py+ph,px:px+pw]=rng.uniform(.23,.62)
                    ht[py:py+ph,px:px+pw]=rng.uniform(.2,.85)
            x+=wid
        y+=row
    # Bandas discontinuas de infraestructura, no ruido aleatorio uniforme.
    for _ in range(95):
        py=int(rng.integers(25,H-25)); px=int(rng.integers(0,W-150))
        pw=int(rng.integers(55,340)); ph=int(rng.integers(2,6))
        al[py:py+ph,px:min(W,px+pw)]*=.62
        ht[py:py+ph,px:min(W,px+pw)]*=.42
    al+=rng.normal(0,.012,(H,W)).astype(np.float32)
    def image(name,arr):
        old=bpy.data.images.get(name)
        if old: bpy.data.images.remove(old)
        im=bpy.data.images.new(name,width=W,height=H,alpha=False)
        # Los mapas son datos lineales; se empaquetan dentro del .blend.
        opts=[i.identifier for i in im.colorspace_settings.bl_rna.properties['name'].enum_items]
        try: im.colorspace_settings.name='Non-Color'
        except (TypeError,ValueError): pass
        pix=np.ones((H,W,4),dtype=np.float32)
        pix[:,:,:3]=arr[:,:,None]
        im.pixels.foreach_set(pix.ravel()); im.update()
        im.pack()
        return im
    color=image('DS | Atlas de paneles 4096',al)
    height=image('DS | Microrelieve 4096',ht)
    for m in palette():
        nt=m.node_tree; p=next(n for n in nt.nodes if n.type=='BSDF_PRINCIPLED')
        for node in list(nt.nodes):
            if node.type not in ('BSDF_PRINCIPLED','OUTPUT_MATERIAL'): nt.nodes.remove(node)
        t=nt.nodes.new('ShaderNodeTexImage'); t.image=color; t.label='Paneles y circuitos'
        mult=nt.nodes.new('ShaderNodeMixRGB')
        enum(mult,'blend_type','MULTIPLY')
        mult.inputs[0].default_value=1
        mult.inputs[2].default_value=p.inputs['Base Color'].default_value
        nt.links.new(t.outputs['Color'],mult.inputs[1]); nt.links.new(mult.outputs[0],p.inputs['Base Color'])
        bumptex=nt.nodes.new('ShaderNodeTexImage'); bumptex.image=height
        bump=nt.nodes.new('ShaderNodeBump'); bump.inputs['Strength'].default_value=.42; bump.inputs['Distance'].default_value=.038
        nt.links.new(bumptex.outputs['Color'],bump.inputs['Height']); nt.links.new(bump.outputs['Normal'],p.inputs['Normal'])
        t.location=(-600,140); mult.location=(-300,180); bumptex.location=(-600,-150); bump.location=(-300,-100); p.location=(0,100)
    print('TEXTURAS LISTAS: dos atlas 4096x2048 empaquetados en el archivo.')

def detalles():
    main=collection('ESTRELLA DE LA MUERTE | DS-1')
    coll=collection('02 | Infraestructura y trincheras',main)
    mats=[material('DS | Mecanismos oscuros',(.075,.083,.096),.55,.64),
          material('DS | Maquinaria gris',(.20,.215,.235),.55,.62),
          material('DS | Maquinaria clara',(.31,.325,.34),.5,.61),
          material('DS | Canales negros',(.018,.023,.031),.3,.8)]
    rng=random.Random(1138)
    mech=Mesh()
    # Relieve a varias escalas siguiendo corredores urbanos en los paralelos.
    for j in range(48):
        lat=-1.32+j*2.64/47+rng.uniform(-.014,.014)
        if abs(lat)<.034: continue
        count=int(320*cos(lat))
        for k in range(count):
            lon=TAU*k/count+rng.uniform(-.005,.005)
            active=(sin(lon*7+j*1.31)+sin(lon*19-j*.6))>-.25
            if not active or rng.random()<.19: continue
            w=rng.uniform(.035,.16); h=rng.uniform(.018,.068)
            spherebox(mech,lat+rng.uniform(-.008,.008),lon,w,h,rng.uniform(.008,.035),10.018,rng.choices([0,1,2],[5,5,1])[0])
            if rng.random()<.24:
                spherebox(mech,lat+.008,lon+.007,w*.7,.008,.014,10.022,2)
    mech.object('DS | Bloques tecnicos y circuitos en relieve',coll,mats)
    trench=Mesh()
    # Fondo del canal y labios en los dos bordes del ecuador.
    patch(trench,-.018,.018,0,TAU,9.79,3)
    Z=Vector((0,0,1)); X=Vector((1,0,0)); Y=Vector((0,1,0))
    for z in (-.169,.169):
        ring(trench,Z,X,Y,z,9.98,.07,.030,2,768)
        ring(trench,Z,X,Y,z*.87,9.89,.07,.024,0,768)
    # Murallas del canal; el interior sigue siendo visible y tiene profundidad.
    for i in range(768):
        a=TAU*i/768; b=TAU*(i+1)/768
        for z in (-.168,.168):
            trench.face([Vector((cos(t)*r,sin(t)*r,z)) for r,t in ((9.79,a),(9.99,a),(9.99,b),(9.79,b))],1)
    for k in range(1250):
        a=TAU*k/1250+rng.uniform(-.001,.001)
        n=Vector((cos(a),sin(a),0)); tx=Vector((-sin(a),cos(a),0))
        box(trench,n*rng.uniform(9.79,9.88)+Z*rng.uniform(-.11,.11),tx,Z,n,
            rng.uniform(.022,.085),rng.uniform(.015,.075),rng.uniform(.025,.10),rng.choice([0,0,1,1,2]))
    for k in range(96):
        a=TAU*k/96
        n=Vector((cos(a),sin(a),0)); tx=Vector((-sin(a),cos(a),0))
        box(trench,n*9.88,tx,Z,n,.035,.28,.035,0)
    trench.object('DS | Trinchera ecuatorial profunda y maquinaria',coll,mats)
    # Hangares empotrados con dinteles y luces diminutas, sin exagerar el brillo.
    bays=Mesh(); glow=Mesh()
    lightmat=material('DS | Luces de hangares',(.59,.69,.82),.0,.4,1.8)
    for a in (-1.58,-1.0,-2.25,.3,1.8,2.7):
        n=Vector((cos(a),sin(a),0)); tx=Vector((-sin(a),cos(a),0))
        box(bays,n*9.91,tx,Z,n,.38,.105,.018,3)
        for z in (-.062,.062): box(bays,n*9.925+Z*z,tx,Z,n,.43,.025,.045,1)
        for x in (-.214,.214): box(bays,n*9.925+tx*x,tx,Z,n,.025,.15,.045,1)
        for x in (-.13,-.08,-.03,.02,.07,.12): box(glow,n*9.93+tx*x-Z*.042,tx,Z,n,.008,.009,.008)
    bays.object('DS | Hangares del ecuador',coll,mats)
    glow.object('DS | Luces interiores de hangares',coll,[lightmat])
    print('DETALLES LISTOS. Caras de maquinaria:',len(mech.faces),'caras de trinchera:',len(trench.faces))

def superlaser():
    main=collection('ESTRELLA DE LA MUERTE | DS-1')
    coll=collection('03 | Plato del superlaser',main)
    mats=[material('DS | Reflector gris',(.198,.202,.205),.32,.58),
          material('DS | Reflector panel oscuro',(.171,.174,.177),.32,.66),
          material('DS | Reflector panel claro',(.234,.238,.242),.32,.60),
          material('DS | Fisuras del reflector',(.025,.031,.043),.4,.77),
          material('DS | Aro mecanizado',(.183,.187,.19),.32,.53)]
    rng=random.Random(1983)
    radius=sin(APERTURE)*R
    lip=cos(APERTURE)*R
    depth=1.12
    def axial(r): return lip-depth+depth*(r/radius)**2
    bowl=Mesh()
    # Paraboloide cóncavo real, no una esfera pintada ni un disco plano.
    for j in range(56):
        ra=radius*j/56; rb=radius*(j+1)/56
        for i in range(256):
            a=TAU*i/256; b=TAU*(i+1)/256
            bowl.face([AXIS*axial(r)+(U*cos(t)+V*sin(t))*r for r,t in ((ra,a),(rb,a),(rb,b),(ra,b))],3,True)
    bowl.object('DS | Reflector concavo parabolico',coll,mats)
    panels=Mesh()
    # Sectores radiales y anillos con juntas discretas.
    radial=[.13,.25,.41,.60,.81,1.05,1.32,1.61,1.94,2.25,2.58,radius-.035]
    for j,(ra,rb) in enumerate(zip(radial[:-1],radial[1:])):
        count=[20,24,32,40,48,56,64,72,80,88,96][j]
        offset=(j%2)*pi/count
        for i in range(count):
            a=TAU*i/count+offset+.0016
            b=TAU*(i+1)/count+offset-.0016
            r0=ra+.007; r1=rb-.007
            mi=rng.choices([0,1,2],[16,2,2])[0]
            for k in range(4):
                s=r0+(r1-r0)*k/4; t=r0+(r1-r0)*(k+1)/4
                panels.face([AXIS*(axial(r)+.008)+(U*cos(q)+V*sin(q))*r for r,q in ((s,a),(t,a),(t,b),(s,b))],mi,True)
    panels.object('DS | Sectores radiales del reflector',coll,mats)
    rings=Mesh()
    # Un borde fino al ras; ocho puntos de enfoque reconocibles.
    for rr,w,h,z,mi in ((radius+.008,.065,.045,lip+.008,4),
                         (radius-.085,.028,.024,axial(radius-.085)+.015,3),
                         (.20,.035,.022,axial(.20)+.023,4),
                         (.12,.025,.030,axial(.12)+.027,3)):
        ring(rings,AXIS,U,V,z,rr,w,h,mi,384)
    # Centro negro y colimador.
    for i in range(96):
        a=TAU*i/96; b=TAU*(i+1)/96
        rings.face([AXIS*(axial(0)+.016),AXIS*(axial(.105)+.016)+(U*cos(a)+V*sin(a))*.105,
                    AXIS*(axial(.105)+.016)+(U*cos(b)+V*sin(b))*.105],3)
    for i in range(8):
        a=TAU*i/8+pi/8; radialdir=U*cos(a)+V*sin(a); tang=-U*sin(a)+V*cos(a)
        rr=radius*.69
        center=AXIS*(axial(rr)+.027)+radialdir*rr
        box(rings,center,tang,radialdir,AXIS,.092,.18,.035,3)
        box(rings,center+AXIS*.021,tang,radialdir,AXIS,.044,.115,.012,1)
        # Canal fino que conecta el emisor con el borde del plato.
        for k in range(12):
            r0=rr+.12+(radius-.16-rr-.12)*k/12
            r1=rr+.12+(radius-.16-rr-.12)*(k+1)/12
            w=.009
            rings.face([AXIS*(axial(r)+.017)+radialdir*r+tang*s for r,s in ((r0,-w),(r1,-w),(r1,w),(r0,w))],3)
    rings.object('DS | Anillos, ocho emisores y colimador central',coll,mats)
    print('SUPERLASER LISTO. Apertura:',round(radius*2,3),'profundidad:',depth)

def escena():
    scene=bpy.context.scene
    scene.name='StarWars_Planet'
    main=collection('ESTRELLA DE LA MUERTE | DS-1')
    stage=collection('04 | Camaras e iluminacion',main)
    world=bpy.data.worlds.new('DS | Espacio profundo')
    world.use_nodes=True
    bg=next(n for n in world.node_tree.nodes if n.type=='BACKGROUND')
    bg.inputs['Color'].default_value=(.003,.005,.009,1)
    bg.inputs['Strength'].default_value=.20
    scene.world=world
    def camera(name, pos, focal):
        d=bpy.data.cameras.new(name); o=bpy.data.objects.new(name,d); stage.objects.link(o)
        o.location=pos; o.rotation_euler=(-o.location).to_track_quat('-Z','Y').to_euler()
        d.lens=focal; d.clip_end=1000
        return o
    hero=camera('DS | Camara principal',(0,-39,4),58)
    camera('DS | Camara detalle superlaser',tuple(AXIS*17+U*1.0),62)
    scene.camera=hero
    def area(name, pos, power, size, color):
        typ=next(i.identifier for i in bpy.types.Light.bl_rna.properties['type'].enum_items if i.identifier=='AREA')
        d=bpy.data.lights.new(name,typ); d.energy=power; d.color=color; d.shape=next(i.identifier for i in d.bl_rna.properties['shape'].enum_items if i.identifier=='DISK'); d.size=size
        o=bpy.data.objects.new(name,d); stage.objects.link(o); o.location=pos
        o.rotation_euler=(-o.location).to_track_quat('-Z','Y').to_euler()
        return o
    area('DS | Luz principal superior izquierda',(-17,-20,23),6800,9,(1,.96,.90))
    area('DS | Relleno frontal suave',(10,-25,3),1150,14,(.75,.84,1))
    area('DS | Contraluz del hemisferio',(10,8,15),3700,7,(.68,.79,1))
    scene.render.resolution_x=1600; scene.render.resolution_y=1600; scene.render.resolution_percentage=100
    scene.render.film_transparent=False
    fmt=[i.identifier for i in scene.render.image_settings.bl_rna.properties['file_format'].enum_items]
    scene.render.image_settings.file_format=next(x for x in fmt if x=='PNG')
    scene.render.filepath=os.path.join(REFS,'starwars_preview.png')
    # El motor actual ya es EEVEE en la sesión del usuario.
    try: scene.render.engine='CYCLES'
    except TypeError: pass
    if scene.render.engine=='CYCLES':
        scene.cycles.samples=64; scene.cycles.use_denoising=True
        enum(scene.cycles,'device','CPU')
        scene.cycles.max_bounces=6
    scene.view_settings.exposure=.8
    root=bpy.data.objects.get('StarWars_Planet_ROOT')
    if not root:
        root=bpy.data.objects.new('StarWars_Planet_ROOT',None); main.objects.link(root)
    root.empty_display_size=1
    root['Descripcion']='Estrella de la Muerte I; blindaje, trincheras y reflector editables por coleccion'
    for coll_name in ('01 | Casco y placas','02 | Infraestructura y trincheras','03 | Plato del superlaser'):
        for o in bpy.data.collections[coll_name].objects: o.parent=root
    for o in bpy.context.selected_objects: o.select_set(False)
    root.select_set(True); bpy.context.view_layer.objects.active=root
    for screen in bpy.data.screens:
        for ar in screen.areas:
            if ar.type=='VIEW_3D':
                ar.spaces.active.region_3d.view_rotation=hero.rotation_euler.to_quaternion()
                ar.spaces.active.region_3d.view_distance=30
                ar.spaces.active.region_3d.view_location=(0,0,0)
                enum(ar.spaces.active.shading,'type','MATERIAL')
                ar.spaces.active.overlay.show_overlays=False
    text=bpy.data.texts.get('LEEME | Estrella de la Muerte') or bpy.data.texts.new('LEEME | Estrella de la Muerte')
    text.clear(); text.write('ESTRELLA DE LA MUERTE I / DS-1\n\nModelo procedural creado mediante MCP.\nColecciones: casco y blindaje, infraestructura y trincheras, superlaser, camaras e iluminacion.\nEl plato es un paraboloide concavo abierto en el casco. La trinchera tiene profundidad y maquinaria real.\nDos mapas de 4096x2048 estan empaquetados dentro del .blend.\nUsa StarWars_Planet_ROOT para transformar el modelo completo.\nCamara principal para la vista de presentacion; segunda camara para inspeccionar el plato.\nLa escena original se conserva oculta en la coleccion 00.\n')
    path=os.path.join(ROOT,'starwars_death_star_planet.blend')
    bpy.ops.wm.save_as_mainfile(filepath=path)
    print('GUARDADO:',path)
    print('OBJETOS:',len(scene.objects),'CARAS:',sum(len(o.data.polygons) for o in main.all_objects if o.type=='MESH'))

if __name__=='__main__':
    estructura(); textura(); detalles(); superlaser(); escena()
