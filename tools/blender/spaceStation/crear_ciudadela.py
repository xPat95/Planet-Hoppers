"""Ciudadela de Mass Effect. Fuente procedural nativa; no usa assets descargados.

Ejecutar desde Blender. Crea la fuente y las vistas en las carpetas del repositorio.
X = eje longitudinal; cinco distritos habitables orientados hacia el interior.
"""
from pathlib import Path
from math import sin, cos, pi, sqrt
import bpy
import bmesh
import random
import json
from mathutils import Vector

REPO = Path(__file__).resolve().parents[3]
HERE = REPO/'assets/models/environment/spaceStation'
REFS = REPO/'docs/references/spaceStation'
HERE.mkdir(parents=True,exist_ok=True)
REFS.mkdir(parents=True,exist_ok=True)
TAU = 2*pi
SEED = 61083
rng = random.Random(SEED)


def set_enum(owner, key, choice):
    values = [i.identifier for i in owner.bl_rna.properties[key].enum_items]
    if choice in values:
        setattr(owner, key, next(v for v in values if v == choice))
    else:
        try:
            setattr(owner, key, choice)
        except (TypeError, ValueError):
            pass


class Mesh:
    def __init__(self):
        self.v, self.f, self.mi = [], [], []

    def add(self, verts, faces, material=0):
        n = len(self.v)
        self.v.extend(tuple(v) for v in verts)
        self.f.extend(tuple(n+i for i in f) for f in faces)
        self.mi.extend([material]*len(faces))

    def face(self, points, material=0):
        self.add(points, [tuple(range(len(points)))], material)

    def box(self, center, dims, material=0, transform=None):
        v = [(center[0]+a*dims[0]/2, center[1]+b*dims[1]/2,
              center[2]+c*dims[2]/2) for c in (-1,1) for b in (-1,1) for a in (-1,1)]
        if transform:
            v = [transform(p) for p in v]
        self.add(v, [(0,2,3,1),(4,5,7,6),(0,1,5,4),
                     (2,6,7,3),(0,4,6,2),(1,3,7,5)], material)

    def object(self, name, collection, parent=None, bevel=0):
        if not self.v:
            return None
        me = bpy.data.meshes.new(name)
        me.from_pydata(self.v, [], self.f)
        for m in MATS:
            me.materials.append(m)
        for p, index in zip(me.polygons, self.mi):
            p.material_index = index
        bm = bmesh.new()
        bm.from_mesh(me)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        bm.to_mesh(me)
        bm.free()
        me.update()
        ob = bpy.data.objects.new(name, me)
        collection.objects.link(ob)
        ob.parent = parent
        if bevel:
            mod = ob.modifiers.new('Cantos de blindaje', 'BEVEL')
            mod.width = bevel
            mod.segments = 2
        return ob


def collection(name, parent):
    c = bpy.data.collections.new(name)
    parent.children.link(c)
    return c


def empty(name, c, parent=None):
    ob = bpy.data.objects.new(name, None)
    c.objects.link(ob)
    ob.parent = parent
    return ob


def palette():
    spec = [
        ('Titanio azulado', (.14,.175,.235), .46,.43),
        ('Blindaje grafito', (.080,.105,.15), .40,.48),
        ('Placas plata', (.25,.30,.38), .58,.37),
        ('Placas acero', (.118,.15,.208), .48,.46),
        ('Juntas oscuras', (.008,.013,.024), .4,.65),
        ('Edificios pizarra', (.065,.09,.13), .48,.6),
        ('Edificios claros', (.20,.23,.28), .55,.48),
        ('Edificios oscuros', (.025,.041,.068), .6,.5),
        ('Parques', (.017,.060,.048), .1,.94),
        ('Canales', (.011,.16,.20), .6,.18),
        ('Luz ambar', (1.0,.39,.095), .1,.4),
        ('Luz marfil', (1.0,.79,.46), .1,.4),
        ('Luz cian', (.12,.65,1.0), .1,.4),
        ('Luz hielo', (.64,.85,1.0), .1,.4),
        ('Luz roja baliza', (1.0,.04,.015), .1,.4),
        ('Marcas de panel', (.38,.46,.55), .55,.4),
    ]
    result = []
    for i,(name,color,metal,rough) in enumerate(spec):
        m = bpy.data.materials.new('CI | '+name)
        m.use_nodes = True
        p = next(n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
        p.inputs['Base Color'].default_value = (*color,1)
        p.inputs['Metallic'].default_value = metal
        p.inputs['Roughness'].default_value = rough
        m.diffuse_color = (*color,1)
        if i in range(10,15):
            p.inputs['Emission Color'].default_value = (*color,1)
            p.inputs['Emission Strength'].default_value = {10:3.2,11:3.8,12:3.5,13:4.0,14:4.0}[i]
        result.append(m)
    return result


def tube(mesh, pts, radius, material, sides=6):
    pts = [Vector(p) for p in pts]
    verts=[]
    previous_tangent=None;previous_a=None
    for i,p in enumerate(pts):
        tangent = (pts[min(i+1,len(pts)-1)]-pts[max(i-1,0)]).normalized()
        if previous_a is None:
            ref=min([Vector((1,0,0)),Vector((0,1,0)),Vector((0,0,1))],key=lambda v:abs(tangent.dot(v)))
            a=tangent.cross(ref).normalized()
        else:
            a=previous_tangent.rotation_difference(tangent)@previous_a
            a=(a-tangent*a.dot(tangent)).normalized()
        b = tangent.cross(a).normalized()
        verts.extend(p+radius*(cos(TAU*j/sides)*a+sin(TAU*j/sides)*b) for j in range(sides))
        previous_tangent=tangent;previous_a=a
    faces=[(i*sides+j,i*sides+(j+1)%sides,(i+1)*sides+(j+1)%sides,(i+1)*sides+j)
           for i in range(len(pts)-1) for j in range(sides)]
    faces += [tuple(reversed(range(sides))),tuple((len(pts)-1)*sides+j for j in range(sides))]
    mesh.add(verts, faces, material)


def annulus(mesh, x0, x1, r0, r1, a0=0, a1=TAU, material=0, count=160):
    v=[]
    for i in range(count+1):
        a=a0+(a1-a0)*i/count
        v.extend((x,r*cos(a),r*sin(a)) for x,r in [(x0,r0),(x0,r1),(x1,r1),(x1,r0)])
    f=[]
    for i in range(count):
        for j in range(4):
            f.append((i*4+j,i*4+(j+1)%4,(i+1)*4+(j+1)%4,(i+1)*4+j))
    f.extend([(3,2,1,0),tuple(count*4+j for j in range(4))])
    mesh.add(v,f,material)


def ring_line(mesh,x,r,radius,material,a0=0,a1=TAU,count=220):
    tube(mesh,[(x,r*cos(a0+(a1-a0)*i/count),r*sin(a0+(a1-a0)*i/count))
               for i in range(count+1)],radius,material)


PROFILE=[(-3,8.7,36),(-13,13.7,36.5),(-28,14.1,37),(-65,13.7,39.0),
         (-110,12.7,42.6),(-160,11.7,47),(-203,10.4,51.1),
         (-231,8.2,54),(-248,5.5,56),(-258,2.9,58.6),(-263,.12,60.3)]


def profile(x):
    for a,b in zip(PROFILE,PROFILE[1:]):
        if b[0] <= x <= a[0]:
            t=(x-a[0])/(b[0]-a[0])
            return tuple(a[k]+(b[k]-a[k])*t for k in (1,2))
    p=PROFILE[0] if x>PROFILE[0][0] else PROFILE[-1]
    return p[1],p[2]


CROSS=[(0,0),(-.73,0),(-1,-2.8),(-1.07,-2.2),(-1.07,2.2),
       (-.79,6.5),(0,8.4),(.79,6.5),(1.07,2.2),(1.07,-2.2),(1,-2.8),(.73,0)]


def arm_point(x, u, h):
    w,r=profile(x)
    taper=min(1.,max(.09,(-x-1)/12),max(.09,(264+x)/17))
    return (x,w*u,r-36+h*taper)


def build_arm(index, theta):
    title=['Zakera','Kithoi','Tayseri','Bachjret','Shalta'][index]
    c=collection(f'{index+2:02d} | Brazo {index+1} - {title}', MODEL)
    pivot=empty(f'CI | BRAZO {index+1} | {title}',c,ROOT)
    pivot.location=(0,36*cos(theta),36*sin(theta))
    pivot.rotation_euler.x=theta-pi/2
    pivot['district']=title
    pivot['axis']='X longitudinal, Z exterior, -Z ciudad interior'
    shell=Mesh()
    xs=[-3-260*i/100 for i in range(101)]
    vs=[arm_point(x,u,h) for x in xs for u,h in CROSS]
    n=len(CROSS)
    fs=[(i*n+j,i*n+(j+1)%n,(i+1)*n+(j+1)%n,(i+1)*n+j)
        for i in range(len(xs)-1) for j in range(n)]
    fs.extend([tuple(reversed(range(n))),tuple((len(xs)-1)*n+j for j in range(n))])
    shell.add(vs,fs,1)
    shell.object(f'CI {index+1} | Casco facetado continuo',c,pivot,.065)
    panels,ribs,rails,lights,city,parks=Mesh(),Mesh(),Mesh(),Mesh(),Mesh(),Mesh()
    # Las juntas se abren entre placas, sin una textura plana sobre el casco.
    for j in range(43):
        xa=-10-j*5.75
        xb=max(-260,xa-5.48)
        for k in range(3,9):
            u0,h0=CROSS[k];u1,h1=CROSS[k+1]
            gap=.025
            ua=u0+(u1-u0)*gap;ub=u0+(u1-u0)*(1-gap)
            ha=h0+(h1-h0)*gap+.12;hb=h0+(h1-h0)*(1-gap)+.12
            panels.face([arm_point(xa,ua,ha),arm_point(xb,ua,ha),
                         arm_point(xb,ub,hb),arm_point(xa,ub,hb)],rng.choices([0,1,3],[6,1,7])[0])
            # Microplacas escalonadas sobre las grandes facetas del casco.
            for q in range(3):
                mx=xa-.28-q*1.72
                for lane in range(2):
                    t0=.09+lane*.46;t1=t0+.35
                    a=u0+(u1-u0)*t0;b=u0+(u1-u0)*t1
                    ah=h0+(h1-h0)*t0+.16;bh=h0+(h1-h0)*t1+.16
                    panels.face([arm_point(mx,a,ah),arm_point(mx-1.45,a,ah),
                                 arm_point(mx-1.45,b,bh),arm_point(mx,b,bh)],rng.choice([0,0,3,3,1]))
        if j%3==0:
            tube(ribs,[arm_point(xa,u,h+.24) for u,h in CROSS[3:10]],.15,3)
        for side in (-1,1):
            # Doble reborde negro, nervio plateado y luces de navegación discontinuas.
            tube(rails,[arm_point(x,side*.97,-2.63) for x in (xa,xb)],.22,2)
            if j%2==0:
                tube(lights,[arm_point(x,side*.91,-2.63) for x in (xa-.7,xb+.7)],.06,13)
            # Dos bandas de ranuras hundidas en los flancos del blindaje.
            for t in range(4):
                xx=xa-.8-t*.8
                panels.face([arm_point(xx,side*1.074,.1),arm_point(xx-.28,side*1.074,.1),
                             arm_point(xx-.28,side*1.074,1.8),arm_point(xx,side*1.074,1.8)],4)
    # Quilla angular longitudinal en el dorso.
    tube(rails,[arm_point(x,0,8.7) for x in xs[3:-2]],.24,0)
    for side in (-1,1):
        tube(rails,[arm_point(x,side*.77,6.64) for x in xs[4:-3]],.16,3)
        # Grieta tecnologica longitudinal: una franja urbana entre las placas.
        def flank(x,u,raise_by=0):
            h=6.5-(u-.79)*4.3/.28
            return arm_point(x,side*u,h+raise_by)
        for j in range(90):
            xa=-28-j*2.45;xb=xa-2.40
            if xb<-248:
                break
            panels.face([flank(xa,.865,.20),flank(xb,.865,.20),
                         flank(xb,.932,.20),flank(xa,.932,.20)],4)
            for lane in range(3):
                for cell in range(4):
                    if rng.random()<.25:
                        continue
                    xx=xa-.17-cell*.54
                    ua=.873+lane*.016;ub=ua+.006
                    lights.face([flank(xx,ua,.225),flank(xx-.25,ua,.225),
                                 flank(xx-.25,ub,.225),flank(xx,ub,.225)],rng.choice([10,10,11,12]))
            if j%8==0:
                tube(rails,[flank(xa,.84,.27),flank(xa,.96,.27)],.085,2)
    # Ciudad tridimensional a ambos lados del parque y del canal longitudinal.
    rows=104
    for row in range(rows):
        x=-20-row*2.17
        w,r=profile(x)
        z=r-36
        width=w*.70
        parks.box((x,0,z-.04),(2.13,2.55,.11),8)
        parks.box((x,.18*sin(row*.08),z-.13),(2.18,.46,.08),9)
        # Viales blancos sobre el suelo oscuro, reducidos para no saturar la ciudad.
        for side in (-1,1):
            lights.box((x,side*1.43,z-.04),(2.10,.045,.025),11 if row%7 else 12)
        if row%5==0:
            lights.box((x,0,z-.09),(.045,width*1.91,.03),10)
        for side in (-1,1):
            for col in range(6):
                y=side*(1.96+col*(width-2.2)/6)
                if abs(y)>width-.2 or rng.random()<.17:
                    continue
                dx=rng.uniform(.67,1.60)
                dy=rng.uniform(.46,.88)*max(.6,(width-2.1)/6)
                h=rng.uniform(.18,.8)*(1.7 if row%11 in (1,2) else 1)
                xx=x+rng.uniform(-.20,.20)
                zz=profile(xx)[1]-36
                city.box((xx,y,zz-h/2-.05),(dx,dy,h),rng.choice([5,5,6,7]))
                floors=max(1,int(h/.20))
                color=rng.choices([10,11,12],[6,4,1])[0]
                for floor in range(floors):
                    zh=zz-.12-floor*.20
                    # Ventanas en cubierta interior y en las dos fachadas largas.
                    lights.box((xx,y-dy/2-.008,zh),(dx*.74,.014,.045),color)
                    lights.box((xx,y+dy/2+.008,zh),(dx*.74,.014,.045),color)
                if rng.random()<.7:
                    lights.box((xx,y,zz-h-.06),(dx*.50,.05,.014),color)
                if rng.random()<.12:
                    city.box((xx,y,zz-h-.12),(dx*.65,dy*.66,.18),7)
        if row%9==0:
            # Puentes que cruzan el parque lineal.
            city.box((x,0,z-.48),(.30,3.2,.13),2)
            lights.box((x-.18,0,z-.55),(.025,3.1,.025),13)
        if row%6==0:
            for side in (-1,1):
                yy=side*.85
                parks.box((x,yy,z-.17),(.65,.42,.22),8)
    # Rascacielos y núcleos cívicos a escala de la megaestructura.
    for j in range(18):
        x=-28-j*11.9+rng.uniform(-2,2)
        w,r=profile(x)
        y=rng.choice([-1,1])*w*rng.uniform(.32,.58)
        h=rng.uniform(1.15,2.7)
        city.box((x,y,r-36-h/2),(.95,.86,h),6)
        city.box((x,y,r-36-h-.30),(.57,.51,.6),3)
        for k in range(int(h/.19)):
            z=r-36-.14-k*.19
            lights.box((x,y-.438,z),(.72,.018,.045),11)
            lights.box((x,y+.438,z),(.72,.018,.045),10)
        tube(rails,[(x,y,r-36-h-.4),(x,y,r-36-h-1.15)],.025,2)
        lights.box((x,y,r-36-h-1.15),(.085,.085,.07),14)
    # Raiz: grandes escudos segmentados, bahias de atraque y contrafuertes.
    for side in (-1,1):
        shield=Mesh()
        shield.add([arm_point(x,side*u,h) for x,u,h in
                    [(-4,.24,4),(-9,1.16,4),(-27,1.11,3.2),(-40,.83,5),
                     (-21,.62,11.0),(-6,.60,10.0)]],
                   [(0,1,4),(1,2,3,4),(0,4,5),(4,3,5),(0,5,3,2,1)],0)
        shield.object(f'CI {index+1} | Escudo de raiz {side:+}',c,pivot,.18)
        for k in range(3):
            xx=-8.5-k*2.4
            rails.box((xx,side*9.7,profile(xx)[1]-36-1.9),(1.5,1.8,1.1),4)
            lights.box((xx,side*9.7,profile(xx)[1]-36-2.46),(1.05,1.25,.028),12)
    panels.object(f'CI {index+1} | Placas y rejillas exteriores',c,pivot)
    ribs.object(f'CI {index+1} | Costillas estructurales',c,pivot)
    rails.object(f'CI {index+1} | Quillas rebordes y puentes',c,pivot)
    city.object(f'CI {index+1} | Bloques urbanos y torres',c,pivot)
    parks.object(f'CI {index+1} | Parque y canal interior',c,pivot)
    lights.object(f'CI {index+1} | Ventanas avenidas y balizas',c,pivot)


def build_ring():
    c=collection('01 | Presidium - anillo central',MODEL)
    ring=empty('CI | PRESIDIUM',c,ROOT)
    body,plates,structure,city,lights=Mesh(),Mesh(),Mesh(),Mesh(),Mesh()
    section=[(-7,32.0),(-7,37.6),(-3.5,40.0),(7.6,40.0),(12.4,37.7),
             (13.4,33.0),(10.4,29.5),(-3.4,29.5)]
    n=len(section);count=240
    v=[(x,r*cos(TAU*i/count),r*sin(TAU*i/count)) for i in range(count)
       for x,r in section]
    f=[(i*n+j,i*n+(j+1)%n,((i+1)%count)*n+(j+1)%n,((i+1)%count)*n+j)
       for i in range(count) for j in range(n)]
    body.add(v,f,1)
    body.object('CI | Anillo estructural octogonal',c,ring,.08)
    for j in range(120):
        a=TAU*j/120+.004;b=TAU*(j+1)/120-.004
        annulus(plates,-2.8,6.9,39.91,40.18,a,b,rng.choice([0,2,3]),3)
        annulus(plates,12.65,13.13,33.1,37.0,a,b,0 if j%4 else 2,3)
        # Dientes de acoplamiento alrededor de los dos bordes.
        if j%3==0:
            annulus(structure,-7.3,-5.7,32.2,37.8,a+.015,b-.015,3,2)
            annulus(structure,12.3,13.75,32.4,33.1,a,b,2,2)
    for x,r,radius,mat in [(-6.8,37.5,.46,2),(-4.3,39.6,.28,0),
                          (8.4,39.5,.40,2),(12.7,37.3,.29,2),
                          (13.55,33.1,.13,13),(-5.8,31.4,.12,12),
                          (9.2,29.65,.11,11),(-2.7,29.65,.09,11)]:
        ring_line(lights if mat>=10 else structure,x,r,radius,mat)
    # Dos autopistas iluminadas, discontinuas y concéntricas en la cara frontal.
    for r in (34.0,36.3):
        for j in range(60):
            a=TAU*j/60+.009;b=TAU*(j+1)/60-.018
            ring_line(lights,13.38,r,.055,11 if j%5 else 12,a,b,4)
    # Ciudad del Presidium en la superficie interior del anillo.
    for j in range(256):
        a=TAU*j/256
        for k in range(9):
            x=-2.2+k*1.3+rng.uniform(-.13,.13)
            h=rng.uniform(.13,.77)*(1.9 if j%17 in (0,1) else 1)
            r=29.48-h/2
            transform=lambda p,a=a:(p[0],p[2]*cos(a)-p[1]*sin(a),p[2]*sin(a)+p[1]*cos(a))
            city.box((x,0,r),(.63+rng.random()*.34,.34+rng.random()*.17,h),rng.choice([5,6,7]),transform)
            color=rng.choice([10,10,11,12])
            lights.box((x,0,29.46-h-.015),(.5,.055,.022),color,transform)
            if h>.4:
                lights.box((x,.265,29.25),(.55,.015,.045),color,transform)
    # Cinco macizos de amarre alineados con los distritos.
    for j in range(5):
        a=pi/2+TAU*j/5
        annulus(structure,-13,-1,35.6,42.0,a-.15,a+.15,0,6)
        annulus(plates,-8,-2,41.9,42.4,a-.12,a+.12,3,4)
        # Pilares dobles de unión entre aro y brazo.
        for offset in (-.18,.18):
            aa=a+offset
            tube(structure,[(-17,37*cos(aa),37*sin(aa)),(-6,39.2*cos(aa),39.2*sin(aa)),
                            (6,39.5*cos(aa),39.5*sin(aa))],.63,2,8)
    # Aguja de la Torre de la Ciudadela: posición lateral, conserva la abertura central.
    a=pi*.82
    def tower(p):
        return(p[0],p[2]*cos(a)-p[1]*sin(a),p[2]*sin(a)+p[1]*cos(a))
    for x,r,dx,dy,h in [(1,27.3,4.8,3.9,3.9),(1.4,24.6,3.1,2.8,2.1),
                       (1.7,22.4,2.1,1.7,2.4),(1.9,20.5,1.2,.9,1.6)]:
        city.box((x,0,r),(dx,dy,h),2,tower)
        lights.box((x+.1,-dy/2-.018,r),(dx*.65,.03,h*.57),12,tower)
    tube(structure,[tower((1.9,0,20.0)),tower((1.9,0,17.0))],.09,2,8)
    lights.box((1.9,0,17),(.16,.16,.16),13,tower)
    plates.object('CI | Segmentos de blindaje del Presidium',c,ring)
    structure.object('CI | Aros nervios y cinco acoplamientos',c,ring)
    city.object('CI | Ciudad interior y Torre del Consejo',c,ring)
    lights.object('CI | Luces del Presidium',c,ring)


def world():
    w=bpy.data.worlds.new('CI | Espacio y nebulosa procedural')
    w.use_nodes=True
    nodes=w.node_tree.nodes;nodes.clear();links=w.node_tree.links
    out=nodes.new('ShaderNodeOutputWorld')
    bg=nodes.new('ShaderNodeBackground');bg.inputs['Strength'].default_value=.75
    coord=nodes.new('ShaderNodeTexCoord')
    noise=nodes.new('ShaderNodeTexNoise');noise.inputs['Scale'].default_value=3.4
    noise.inputs['Detail'].default_value=5.;noise.inputs['Roughness'].default_value=.68
    links.new(coord.outputs['Normal'],noise.inputs['Vector'])
    ramp=nodes.new('ShaderNodeValToRGB')
    cr=ramp.color_ramp
    cr.elements.remove(cr.elements[1])
    for i,(pos,color) in enumerate([(.20,(.008,.010,.028,1)),(.42,(.045,.027,.105,1)),
                                  (.56,(.19,.105,.34,1)),(.70,(.44,.31,.57,1))]):
        e=cr.elements[0] if i==0 else cr.elements.new(pos)
        e.position=pos;e.color=color
    links.new(noise.outputs['Fac'],ramp.inputs['Fac'])
    stars=nodes.new('ShaderNodeTexVoronoi');stars.inputs['Scale'].default_value=175
    links.new(coord.outputs['Normal'],stars.inputs['Vector'])
    star_ramp=nodes.new('ShaderNodeValToRGB')
    star_ramp.color_ramp.elements[0].position=.013
    star_ramp.color_ramp.elements[0].color=(2.3,2.5,3.0,1)
    star_ramp.color_ramp.elements[1].position=.041
    star_ramp.color_ramp.elements[1].color=(0,0,0,1)
    links.new(stars.outputs['Distance'],star_ramp.inputs['Fac'])
    add=nodes.new('ShaderNodeMixRGB');set_enum(add,'blend_type','ADD');add.inputs[0].default_value=1
    links.new(ramp.outputs['Color'],add.inputs[1]);links.new(star_ramp.outputs['Color'],add.inputs[2])
    links.new(add.outputs[0],bg.inputs['Color']);links.new(bg.outputs[0],out.inputs[0])
    SCENE.world=w


def camera(name,location,target,scale,c):
    d=bpy.data.cameras.new(name);set_enum(d,'type','ORTHO');d.ortho_scale=scale
    d.clip_start=.1;d.clip_end=3000
    o=bpy.data.objects.new(name,d);c.objects.link(o);o.location=location
    o.rotation_euler=(Vector(target)-o.location).to_track_quat('-Z','Y').to_euler()
    return o


def presentation():
    c=collection('90 | Presentacion - excluida del modelo',SCENE.collection)
    cams=[('CI CAM | Principal',(260,-290,165),(-110,0,0),322),
          ('CI CAM | Anillo frontal',(330,0,0),(-11,0,0),218),
          ('CI CAM | Perfil',(-115,-350,75),(-121,0,0),312),
          ('CI CAM | Distritos interiores',(-185,-68,58),(-114,9,25),151)]
    for args in cams:
        camera(*args,c)
    SCENE.camera=bpy.data.objects['CI CAM | Principal']
    set_enum(SCENE.camera.data,'type','PERSP')
    SCENE.camera.data.lens=55
    for name,loc,power,color,angle in [
        ('CI LUZ | Sol frio',(20,-180,280),3.2,(.78,.85,1),.12),
        ('CI LUZ | Rebote violeta',(-100,160,40),2.0,(.68,.59,1),.20),
        ('CI LUZ | Luz de anillo',(200,15,-80),2.2,(1,.86,.70),.16)]:
        d=bpy.data.lights.new(name,next(i.identifier for i in bpy.types.Light.bl_rna.properties['type'].enum_items if i.identifier=='SUN'))
        d.energy=power;d.color=color;d.angle=angle
        o=bpy.data.objects.new(name,d);c.objects.link(o);o.location=loc
        o.rotation_euler=(Vector((-110,0,0))-o.location).to_track_quat('-Z','Y').to_euler()
    # Reflector grande que dibuja el metal en la camara principal.
    d=bpy.data.lights.new('CI LUZ | Reflejo amplio',next(i.identifier for i in bpy.types.Light.bl_rna.properties['type'].enum_items if i.identifier=='AREA'))
    d.energy=700000;d.size=240;d.color=(.68,.76,1)
    o=bpy.data.objects.new(d.name,d);c.objects.link(o);o.location=(-70,-160,190)
    o.rotation_euler=(Vector((-120,0,0))-o.location).to_track_quat('-Z','Y').to_euler()
    world()
    # Compositor opcional: brillo sutil, sin ocultar la arquitectura.
    try:
        if hasattr(SCENE,'compositing_node_group'):
            nt=bpy.data.node_groups.new('CI | Brillo de las ciudades','CompositorNodeTree')
            SCENE.compositing_node_group=nt
            nt.interface.new_socket(name='Image',in_out='OUTPUT',socket_type='NodeSocketColor')
            output=nt.nodes.new('NodeGroupOutput')
        else:
            SCENE.use_nodes=True;nt=SCENE.node_tree;nt.nodes.clear()
            output=nt.nodes.new('CompositorNodeComposite')
        layer=nt.nodes.new('CompositorNodeRLayers');layer.scene=SCENE
        glow=nt.nodes.new('CompositorNodeGlare')
        if 'Type' in glow.inputs:
            set_enum(glow.inputs['Type'],'default_value','Fog Glow')
            glow.inputs['Threshold'].default_value=1.6
            glow.inputs['Strength'].default_value=.24
            glow.inputs['Size'].default_value=.34
        else:
            set_enum(glow,'glare_type','FOG_GLOW')
            glow.threshold=1.6
        nt.links.new(layer.outputs['Image'],glow.inputs['Image'])
        nt.links.new(glow.outputs['Image'],output.inputs['Image'])
    except (AttributeError,TypeError):
        pass
    SCENE.render.resolution_x=1920;SCENE.render.resolution_y=1280
    SCENE.render.resolution_percentage=100
    set_enum(SCENE.render.image_settings,'file_format','PNG')
    set_enum(SCENE.render.image_settings,'color_mode','RGBA')
    SCENE.render.film_transparent=False
    try:
        set_enum(SCENE.view_settings,'view_transform','AgX')
    except Exception:
        pass
    if hasattr(SCENE,'cycles'):
        SCENE.cycles.samples=48
        SCENE.cycles.use_denoising=True
    SCENE.render.filepath=str(REFS/'Ciudadela_Principal.png')


def stats():
    obs=[o for o in MODEL.all_objects if o.type=='MESH']
    coords=[o.matrix_world@Vector(c) for o in obs for c in o.bound_box]
    lo=[min(v[k] for v in coords) for k in range(3)]
    hi=[max(v[k] for v in coords) for k in range(3)]
    return {'asset':'Ciudadela / Mass Effect','status':'aprobado e integrado como recurso',
            'seed':SEED,'ward_count':5,'longitudinal_axis':'X','units':'unidades de modelado; escala de juego pendiente',
            'mesh_objects':len(obs),'vertices':sum(len(o.data.vertices) for o in obs),
            'polygons':sum(len(o.data.polygons) for o in obs),
            'triangles':sum(len(p.vertices)-2 for o in obs for p in o.data.polygons),
            'dimensions':[round(hi[k]-lo[k],3) for k in range(3)],
            'bounds_min':lo,'bounds_max':hi,'external_assets':False,
            'primary_project_modified':True,'references':['docs/references/spaceStation/source/01_ciudadela_abierta.png','docs/references/spaceStation/source/02_ciudadela_exterior.png']}


def main():
    global SCENE,MODEL,ROOT,MATS
    # Mantiene la escena original en el archivo; no borra trabajo ajeno.
    if bpy.data.scenes.get('Ciudadela_Mass_Effect'):
        raise RuntimeError('Ya existe la escena Ciudadela_Mass_Effect; abre un archivo limpio para reconstruir.')
    SCENE=bpy.data.scenes.new('Ciudadela_Mass_Effect')
    bpy.context.window.scene=SCENE
    MODEL=collection('CIUDADELA | Modelo editable',SCENE.collection)
    ROOT=empty('CIUDADELA_ROOT | Transformacion general',MODEL)
    ROOT['asset_role']='space_station';ROOT['franchise']='Mass Effect'
    ROOT['approval_status']='Aprobado por el usuario; recurso integrado en Planet-Hoppers'
    ROOT['ward_count']=5;ROOT['source']='Geometria procedural original basada en las dos imagenes del usuario'
    MATS=palette()
    build_ring()
    for i in range(5):
        build_arm(i,pi/2+TAU*i/5)
    presentation()
    bpy.context.view_layer.update()
    for ob in bpy.context.selected_objects:
        ob.select_set(False)
    ROOT.select_set(True);bpy.context.view_layer.objects.active=ROOT
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type=='VIEW_3D':
                space=area.spaces.active
                space.clip_end=3000
                set_enum(space.region_3d,'view_perspective','CAMERA')
                space.overlay.show_overlays=False
                set_enum(space.shading,'color_type','MATERIAL')
                set_enum(space.shading,'type','MATERIAL')
    report=stats()
    (REFS/'modelo_diagnostico.json').write_text(json.dumps(report,indent=2,ensure_ascii=False),encoding='utf-8')
    txt=bpy.data.texts.new('LEEME | Ciudadela de Mass Effect')
    readme=HERE/'README.md'
    if readme.exists():
        txt.write(readme.read_text(encoding='utf-8'))
    bpy.ops.wm.save_as_mainfile(filepath=str(HERE/'citadel_space_station.blend'))
    print(json.dumps(report,ensure_ascii=False))


if __name__=='__main__':
    main()
