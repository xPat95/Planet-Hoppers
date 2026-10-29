"""N-1 de Naboo del Episodio I: piloto y astromecánico R2 a bordo.
Construcción nativa editable. La variante del Mandaloriano no se crea aquí.
"""
from pathlib import Path
import sys
SCRIPT_DIR=Path(__file__).resolve().parent
REPO=Path(__file__).resolve().parents[5]
ROOT=REPO/'assets/models/ships/starwars/mandalorian'
REVIEWS=REPO/'docs/references/starwars/ships/mandalorian'
ROOT.mkdir(parents=True,exist_ok=True)
REVIEWS.mkdir(parents=True,exist_ok=True)
if str(SCRIPT_DIR) not in sys.path:sys.path.insert(0,str(SCRIPT_DIR))
from geometria_n1 import *
import bmesh
import random
import json

MODEL='NAVE STAR WARS | Naboo N-1 Episodio I'
def palette():
    specs=[('Amarillo real',(.95,.60,.001),.34,.10),
           ('Cromo pulido',(.68,.74,.80),.135,1),
           ('Aluminio satinado',(.42,.49,.56),.25,.92),
           ('Juntas ocre',(.125,.067,.008),.48,.45),
           ('Carbono interior',(.012,.018,.026),.6,.15),
           ('Metal oscuro',(.065,.085,.12),.29,.8),
           ('Cuero del asiento',(.045,.026,.018),.78,0),
           ('Uniforme granate',(.23,.047,.023),.84,0),
           ('Piel piloto',(.54,.29,.15),.62,0),
           ('Casco cuero',(.095,.038,.013),.6,0),
           ('Arnes arena',(.43,.30,.16),.78,0),
           ('Blanco R2',(.78,.82,.82),.3,.15),
           ('Azul R2',(.012,.067,.34),.23,.65),
           ('Lente negro',(.002,.005,.010),.1,.15),
           ('Reactor azul',(.035,.19,.95),.24,.1),
           ('Testigo rojo',(.65,.008,.004),.3,0),
           ('Pantallas cian',(.015,.43,.52),.3,0),
           ('Dientes y ojos',(.78,.76,.67),.55,0),
           ('Vidrio cabina',(.18,.30,.35),.055,0),
           ('Visor ambar',(.37,.15,.025),.16,.05),
           ('Casco granate rigido',(.17,.027,.018),.42,.18),
           ('Visor panoramico dorado',(.55,.255,.025),.15,.50),
           ('Goma negra casco',(.007,.009,.012),.75,0),
           ('Costuras plateadas casco',(.38,.43,.46),.26,.90),
           ('Paneles granate casco',(.22,.038,.025),.46,.12)]
    mats=[material('N1 | '+name,color,rough,metal) for name,color,rough,metal in specs]
    for i in (0,1,12):
        p=next(n for n in mats[i].node_tree.nodes if n.type=='BSDF_PRINCIPLED')
        p.inputs['Coat Weight'].default_value=.38;p.inputs['Coat Roughness'].default_value=.12
    p=next(n for n in mats[0].node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Coat Weight'].default_value=.18
    for i,strength in ((14,2.8),(15,2.4),(16,1.8)):
        p=next(n for n in mats[i].node_tree.nodes if n.type=='BSDF_PRINCIPLED')
        p.inputs['Emission Color'].default_value=(*specs[i][1],1);p.inputs['Emission Strength'].default_value=strength
    for i,alpha,trans in ((18,.30,.90),(19,.60,.15)):
        p=next(n for n in mats[i].node_tree.nodes if n.type=='BSDF_PRINCIPLED')
        p.inputs['Alpha'].default_value=alpha;p.inputs['Transmission Weight'].default_value=trans;p.inputs['IOR'].default_value=1.46
        mats[i].diffuse_color=(*specs[i][1],alpha)
    p=next(n for n in mats[18].node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Roughness'].default_value=.025
    return mats

def obj(mesh,name,c,mats,parent):
    ob=mesh.object(name,c,mats,parent)
    bm=bmesh.new();bm.from_mesh(ob.data);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(ob.data);bm.free();ob.data.update()
    return ob

def root(name,c,parent=None,pivot=(0,0,0)):
    ob=bpy.data.objects.new(name,None);c.objects.link(ob);ob.parent=parent;ob.location=pivot;return ob

def init():
    scene=bpy.data.scenes.new('Naboo_N1_Episodio_I');bpy.context.window.scene=scene
    for old in list(bpy.data.scenes):
        if old!=scene:bpy.data.scenes.remove(old)
    for ob in list(bpy.data.objects):bpy.data.objects.remove(ob,do_unlink=True)
    for c in list(bpy.data.collections):bpy.data.collections.remove(c)
    bpy.data.orphans_purge(do_recursive=True)
    c=coll(MODEL);r=root('Naboo_N1_ROOT',c);r['asset_role']='ship';r['franchise']='starwars';r['variant']='Naboo Royal N-1 / Episode I';r['forward']='-Y';r['reference_length_m']=11.0
    for name,role in (('N1_Pilot_ROOT','pilot'),('N1_R2_ROOT','astromech'),('N1_Canopy_ROOT','canopy')):
        child=root(name,c,r);child['asset_role']=role
    return r

HULL=[(-4.4,.004,.012,.018,.02),(-4.10,.18,.13,.09,.02),
      (-3.55,.46,.31,.15,.035),(-2.75,.82,.41,.26,.04),
      (-1.8,.93,.47,.34,.05),(-.70,.74,.54,.37,.055),
      (.45,.56,.57,.33,.06),(1.40,.385,.51,.23,.075),
      (2.15,.29,.40,.17,.08),(3.1,.20,.28,.10,.10),
      (4.2,.109,.16,.05,.13),(5.2,.046,.065,.022,.16),
      (6,.002,.003,.003,.18)]

def interpolate(profile,value):
    for a,b in zip(profile,profile[1:]):
        if a[0]<=value<=b[0]:
            t=(value-a[0])/(b[0]-a[0]);return tuple(a[i]+(b[i]-a[i])*t for i in range(len(a)))
    return profile[0] if value<profile[0][0] else profile[-1]

def smooth_profile(profile,steps=5):
    # Catmull en datos escalares de dimensión arbitraria.
    result=[]
    for i in range(len(profile)-1):
        a=profile[max(0,i-1)];b=profile[i];c=profile[i+1];d=profile[min(len(profile)-1,i+2)]
        for j in range(steps):
            t=j/steps
            result.append(tuple(.5*(2*b[k]+(c[k]-a[k])*t+(2*a[k]-5*b[k]+4*c[k]-d[k])*t*t+(-a[k]+3*b[k]-3*c[k]+d[k])*t*t*t) for k in range(len(a))))
    return result+[profile[-1]]

def hull(mats):
    c=coll('01 | Casco y alas',coll(MODEL));r=bpy.data.objects['Naboo_N1_ROOT'];mesh=Mesh();n=80
    profile=smooth_profile(HULL,6);v=[]
    for y,rx,up,down,z in profile:
        rx=max(.002,rx)
        for k in range(n):
            a=TAU*k/n;v.append((rx*cos(a),y,z+(up if sin(a)>0 else down)*sin(a)))
    f=[];colors=[]
    for j in range(len(profile)-1):
        y=(profile[j][0]+profile[j+1][0])/2
        for k in range(n):
            f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));colors.append(1 if y<-2.95 else 0)
    f.extend([tuple(reversed(range(n))),tuple((len(profile)-1)*n+k for k in range(n))]);colors.extend([1,0]);mesh.add(v,f,0,True);mesh.mi[:]=colors
    ob=obj(mesh,'N1 | Fuselaje amarillo y proa cromada',c,mats,r)
    # Abertura real de la cabina: permite ver piloto, asiento y mandos.
    cutter=Mesh();v=[];n=80
    for z in (.075,2.5):v.extend((.458*cos(TAU*i/n),-.22+1.065*sin(TAU*i/n),z) for i in range(n))
    f=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]+[(i,(i+1)%n,n+(i+1)%n,n+i) for i in range(n)]
    cutter.add(v,f);cut=obj(cutter,'N1 | Cortador temporal de cabina',c,mats,r)
    mod=ob.modifiers.new('Cabina abierta en casco','BOOLEAN');enum(mod,'operation','DIFFERENCE');mod.object=cut
    bpy.context.view_layer.objects.active=ob;ob.select_set(True);bpy.ops.object.modifier_apply(modifier=mod.name);bpy.data.objects.remove(cut,do_unlink=True);ob.select_set(False)
    seams=Mesh();vents=Mesh();fasteners=Mesh()
    # Línea de separación del cromo y líneas delicadas en el casco posterior.
    y=-2.95;_,rx,up,down,zz=interpolate(HULL,y)
    tube(seams,[(rx*cos(TAU*k/100)*1.003,y,zz+(up if sin(TAU*k/100)>0 else down)*sin(TAU*k/100)*1.004) for k in range(100)],.004,3,6,True)
    for sign in (-1,1):
        pts=[]
        for j in range(80):
            y=.94+(4.8-.94)*j/79;_,rx,up,down,z=interpolate(HULL,y);a=.38
            pts.append((sign*rx*cos(a)*1.012,y,z+up*sin(a)*1.012))
        tube(seams,pts,.003,3,6)
        # Pequeñas rejillas junto al receptáculo del astromecánico.
        for j in range(5):
            y=1.70+j*.09;_,rx,up,down,z=interpolate(HULL,y)
            stroke(vents,[(sign*rx*.65,y,z+up*.77+.008),(sign*rx*.90,y+.01,z+up*.45+.008)],.010,5,2)
        # Dos cañones integrados en la proa.
        tube(vents,[(sign*.43,-2.70,.40),(sign*.43,-3.68,.28)],.029,5,12)
        tube(fasteners,[(sign*.43,-3.35,.32),(sign*.43,-3.72,.274)],.017,1,12)
        for j in range(4):
            y=-2.5+j*.11;ellipsoid(fasteners,(sign*.65,y,.365),(.013,.009,.010),2,10,8)
    obj(seams,'N1 | Juntas del casco',c,mats,r);obj(vents,'N1 | Canones y rejillas',c,mats,r);obj(fasteners,'N1 | Terminales y remaches',c,mats,r)
    # Semialas de perfil aerodinámico; cromo delantero y amarillo posterior.
    stations=[(0,-3.6,.34,.16),(.65,-3.23,.34,.13),(1.2,-2.77,.10,.105),(2.2,-2.34,-.35,.080),(3.0,-2.15,-.58,.065),(3.5,-2.10,-.60,.055)]
    profile=smooth_profile(stations,5)
    for sign,label in ((-1,'babor'),(1,'estribor')):
        wing=Mesh();v=[];f=[];mi=[];chords=40;n=chords*2
        for xx,lead,trail,thickness in profile:
            for k in range(n):
                theta=TAU*k/n;t=(1-cos(theta))/2;y=lead+(trail-lead)*t
                z=.075+thickness*sin(theta)*(1-.3*t)
                v.append((sign*xx,y,z))
        for j in range(len(profile)-1):
            for k in range(n):
                f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k))
                t=(1-cos(TAU*(k+.5)/n))/2;mi.append(1 if t<.77 else 0)
        wing.add(v,f,1,True);wing.mi[:]=mi;obj(wing,'N1 | Ala '+label,c,mats,r)
        detail=Mesh()
        for t in (.77,.94):
            pts=[]
            for xx,lead,trail,th in profile:
                if xx<.75:continue
                theta=math.acos(1-2*t);pts.append((sign*xx,lead+(trail-lead)*t,.075+th*sin(theta)*(1-.3*t)+.003))
            tube(detail,pts,.003,3,6)
        obj(detail,'N1 | Lineas del ala '+label,c,mats,r)
    print('CASCO Y ALAS LISTOS',flush=True)

ENGINE=[(-5,.003),(-4.77,.135),(-4.22,.33),(-3.50,.465),(-2.8,.50),(-2.30,.492),(-1.8,.475),(-1.30,.42),(-1.0,.325),(-.78,.245),(-.62,.175),(-.40,.13),(.4,.085),(1.5,.045),(2.5,.018),(3.20,.002)]
def engines(mats):
    c=coll('02 | Motores J y agujas',coll(MODEL));r=bpy.data.objects['Naboo_N1_ROOT'];profile=smooth_profile(ENGINE,5)
    for sign,label in ((-1,'babor'),(1,'estribor')):
        x=sign*3.5;mesh=Mesh();v=[];f=[];mi=[];n=72
        for y,radius in profile:
            radius=max(.002,radius)
            v.extend((x+radius*cos(TAU*k/n),y,.075+radius*sin(TAU*k/n)) for k in range(n))
        for j in range(len(profile)-1):
            y=(profile[j][0]+profile[j+1][0])/2
            color=1 if y<-2.3 else 0 if y<-.86 else 2
            for k in range(n):f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));mi.append(color)
        f.extend([tuple(reversed(range(n))),tuple((len(profile)-1)*n+i for i in range(n))]);mi.extend([1,2])
        mesh.add(v,f,0,True);mesh.mi[:]=mi;obj(mesh,'N1 | Motor '+label+' con aguja larga',c,mats,r)
        seams=Mesh();nozzle=Mesh();glow=Mesh();details=Mesh()
        for y in (-4.66,-2.3,-1.04):
            _,rad=interpolate(ENGINE,y);tube(seams,[(x+(rad+.002)*cos(TAU*k/96),y,.075+(rad+.002)*sin(TAU*k/96)) for k in range(96)],.006,3 if y!=-4.66 else 0,7,True)
        for y,rad,color in ((-.87,.276,5),(-.815,.250,14),(-.77,.230,1),(-.71,.195,5)):
            tube(glow if color==14 else nozzle,[(x+rad*cos(TAU*k/96),y,.075+rad*sin(TAU*k/96)) for k in range(96)],.017 if color!=14 else .025,color,10,True)
        for k in range(12):
            a=TAU*k/12
            tube(nozzle,[(x+.27*cos(a),-.91,.075+.27*sin(a)),(x+.19*cos(a),-.63,.075+.19*sin(a))],.010,5,7)
        for k in range(5):
            a=pi*.25+pi*.12*k
            pts=[]
            for j in range(12):
                y=-1.77+j*.045;_,rad=interpolate(ENGINE,y);pts.append((x+(rad+.005)*cos(a),y,.075+(rad+.005)*sin(a)))
            tube(details,pts,.007,3,6)
        for mesh,name in ((seams,'Juntas'),(nozzle,'Tobera y aletas'),(glow,'Anillo ionico azul'),(details,'Lamas de refrigeracion')):
            obj(mesh,'N1 | '+label+' '+name,c,mats,r)
    print('MOTORES LISTOS',flush=True)

def canopy(mats):
    c=coll('03 | Cabina transparente e interior',coll(MODEL));r=bpy.data.objects['N1_Canopy_ROOT'];ship=bpy.data.objects['Naboo_N1_ROOT']
    glass=Mesh();v=[];n=32;rows=52
    for j in range(rows+1):
        u=-pi/2+pi*(.001+.998*j/rows)
        for k in range(n+1):
            a=-pi/2+pi*k/n;v.append((.455*cos(u)*sin(a),-.22+1.10*sin(u),.62+.82*cos(u)*cos(a)))
    f=[]
    for j in range(rows):
        for k in range(n):f.append((j*(n+1)+k,j*(n+1)+k+1,(j+1)*(n+1)+k+1,(j+1)*(n+1)+k))
    glass.add(v,f,18,True);ob=obj(glass,'N1 | Cristal transparente de cabina',c,mats,r);ob['canopy']='Ocultar este objeto permite inspeccionar el piloto; no es una cabina vacia'
    frame=Mesh();rim=Mesh();floor=Mesh();dash=Mesh();controls=Mesh()
    tube(rim,[(.462*cos(TAU*k/144),-.22+1.1*sin(TAU*k/144),.619) for k in range(144)],.028,0,12,True)
    tube(frame,[(.460*cos(TAU*k/144),-.22+1.1*sin(TAU*k/144),.642) for k in range(144)],.010,1,9,True)
    for y in (.65,):
        u=math.asin((y+.22)/1.1)
        tube(frame,[(.455*cos(u)*sin(-pi/2+pi*k/64),y,.62+.82*cos(u)*cos(-pi/2+pi*k/64)) for k in range(65)],.008,2,9)
    # Sillín y paredes oscuras bajos; el casco exterior tiene un hueco real.
    for k in range(100):
        a=TAU*k/100;b=TAU*(k+1)/100
        floor.face([(.448*cos(a),-.22+1.055*sin(a),.61),(.448*cos(b),-.22+1.055*sin(b),.61),(.448*cos(b),-.22+1.055*sin(b),.09),(.448*cos(a),-.22+1.055*sin(a),.09)],4)
    floor.face([(.444*cos(TAU*k/100),-.22+1.052*sin(TAU*k/100),.09) for k in reversed(range(100))],4)
    obj(floor,'N1 | Banera interior de cabina',c,mats,ship)
    rounded_box('N1 | Asiento del piloto',c,mats,ship,(0,.14,.17),(.52,.57,.15),6,.055)
    rounded_box('N1 | Respaldo y reposacabezas',c,mats,ship,(0,.46,.57),(.53,.16,.80),6,.07)
    rounded_box('N1 | Consola de instrumentos',c,mats,ship,(0,-.81,.43),(.64,.20,.24),5,.035)
    # Instrumentos hacia el piloto, legibles a través del cristal.
    box(dash,(0,-.699,.46),(.24,.015,.13),13)
    box(dash,(0,-.687,.46),(.20,.008,.090),16)
    for sign in (-1,1):
        for j in range(3):ellipsoid(controls,(sign*(.17+j*.045),-.69,.51),(.013,.011,.013),15 if j==0 else 16,12,8)
        rounded_box('N1 | Consola lateral '+str(sign),c,mats,ship,(sign*.34,-.15,.29),(.13,.68,.12),5,.02)
        for j in range(5):box(controls,(sign*.34,-.41+j*.11,.359),(.045,.027,.01),16 if j%2 else 2)
        tube(controls,[(sign*.23,-.50,.31),(sign*.23,-.54,.47)],.018,5,10)
        ellipsoid(controls,(sign*.23,-.54,.47),(.035,.033,.045),4,16,12)
    for mesh,name,parent in ((rim,'Marco amarillo de cabina',ship),(frame,'Arcos metalicos de cabina',r),(dash,'Pantalla principal',ship),(controls,'Botones y mandos',ship)):
        obj(mesh,'N1 | '+name,c,mats,parent)
    print('CABINA LISTA',flush=True)

def helmet_naboo(mats,c,parent):
    """Casco integral granate: visor único ámbar, auriculares y mentonera."""
    shell=Mesh();visor=Mesh();trim=Mesh();ears=Mesh();grille=Mesh()
    profile=[(.795,.075,.090),(.83,.118,.133),(.88,.166,.176),
             (.95,.195,.198),(1.06,.205,.199),(1.145,.193,.185),
             (1.21,.164,.157),(1.255,.107,.105),(1.278,.006,.009)]
    rings=smooth_profile(profile,5);n=96;v=[];f=[];colors=[]
    # El visor reemplaza el frente de la carcasa, no son dos gafas redondas.
    for z,rx,ry in rings:
        v.extend((rx*sin(TAU*k/n),.023-ry*cos(TAU*k/n),z) for k in range(n))
    for j in range(len(rings)-1):
        z=(rings[j][0]+rings[j+1][0])/2
        for k in range(n):
            a=TAU*(k+.5)/n;front_angle=min(a,TAU-a)
            if .969<z<1.116 and front_angle<1.07:continue
            f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));colors.append(24 if z>1.145 and front_angle<.40 else 20)
    f.extend([tuple(reversed(range(n))),tuple((len(rings)-1)*n+k for k in range(n))]);colors.extend([20,20])
    shell.add(v,f,20,True);shell.mi[:]=colors
    # Lente panorámica curvada con bordes suavemente redondeados.
    cols=64;rows=14;vv=[]
    def lens(a,t):
        edge=abs(a)/1.085;bottom=.964+.010*cos(a*2);top=1.119-.008*edge**4
        z=bottom+(top-bottom)*t
        return (.209*sin(a),.023-.207*cos(a)-.004,z)
    for j in range(rows+1):
        for k in range(cols+1):vv.append(lens(-1.085+2.17*k/cols,j/rows))
    ff=[(j*(cols+1)+k,j*(cols+1)+k+1,(j+1)*(cols+1)+k+1,(j+1)*(cols+1)+k) for j in range(rows) for k in range(cols)]
    visor.add(vv,ff,21,True)
    for t in (0,1):
        tube(trim,[lens(-1.085+2.17*k/96,t) for k in range(97)],.0060,22,8)
        tube(trim,[(x,y-.002,z+(.008 if t else -.008)) for x,y,z in [lens(-1.085+2.17*k/96,t) for k in range(97)]],.0040,20,7)
    for a in (-1.085,1.085):tube(trim,[lens(a,j/20) for j in range(21)],.007,22,9)
    # Ribetes plateados sobre los paneles largos de la coronilla.
    for a in (-.98,-.42,.42,.98):
        points=[]
        for z,rx,ry in rings:
            if z<1.134:continue
            points.append(Vector((rx*sin(a)*1.015,.023-ry*cos(a)*1.015,z+.002)))
        tube(trim,points,.0036,23,7)
        # Pequeños pespuntes metálicos que cruzan el cordón principal.
        for j in range(1,len(points)-1,2):
            p=points[j];tube(trim,[p+Vector((-.003,-.001,-.001)),p+Vector((.003,.001,.001))],.0015,23,5)
    # Costura de la frente y banda alta del visor.
    z=1.137;_,rx,ry=interpolate(profile,z)
    tube(trim,[(rx*sin(-1.30+2.6*k/80),.023-ry*cos(-1.30+2.6*k/80)-.004,z) for k in range(81)],.0038,23,7)
    for sign in (-1,1):
        ellipsoid(ears,(sign*.202,.033,1.035),(.045,.091,.127),22,32,24)
        ellipsoid(ears,(sign*.239,.033,1.030),(.011,.066,.091),23,28,20)
        ellipsoid(ears,(sign*.249,.036,1.030),(.006,.048,.067),22,24,18)
        stroke(ears,[(sign*.217,.098,1.14),(sign*.240,.14,1.05),(sign*.214,.12,.91)],.009,22,7)
        for j in range(4):
            z=.982+j*.027;tube(grille,[(sign*.254,-.006,z),(sign*.254,.059,z)],.003,5,6)
    # Mentonera rígida y rejilla negra curvada, como en la cuarta referencia.
    for j in range(4):
        z=.825+j*.014;_,rx,ry=interpolate(profile,z)
        pts=[]
        for k in range(42):
            a=-.79+1.58*k/41;pts.append((rx*sin(a),.023-ry*cos(a)-.006,z))
        tube(grille,pts,.0060,22,7)
    for mesh,name in ((shell,'Carcasa granate y mentonera'),(visor,'Visor panoramico ambar'),(trim,'Juntas y costuras plateadas'),(ears,'Auriculares y placas laterales'),(grille,'Rejillas de respiracion')):
        obj(mesh,'CASCO PILOTO | '+name,c,mats,parent)

def pilot(mats):
    c=coll('04 | Piloto de la guardia de Naboo',coll(MODEL));r=bpy.data.objects['N1_Pilot_ROOT']
    suit=Mesh();face=Mesh();helmet=Mesh();goggles=Mesh();harness=Mesh();details=Mesh()
    ellipsoid(suit,(0,.14,.49),(.228,.17,.285),7,32,24)
    ellipsoid(face,(0,.075,.79),(.075,.068,.10),8,24,16)
    ellipsoid(face,(0,.018,.994),(.140,.125,.175),8,32,24)
    helmet_naboo(mats,c,r)
    for sign in (-1,1):
        pts=catmull([(sign*.18,.10,.66),(sign*.27,-.07,.49),(sign*.235,-.36,.45),(sign*.23,-.54,.46)],6)
        tube(suit,pts,.065,7,16)
        ellipsoid(helmet,(sign*.23,-.535,.47),(.046,.06,.044),9,24,16)
        for j in range(3):stroke(details,[(sign*.23+(j-1)*.020,-.58,.483),(sign*.23+(j-1)*.020,-.59,.453)],.003,10,2)
        tube(suit,catmull([(sign*.105,.17,.27),(sign*.15,-.22,.19),(sign*.15,-.53,.13)],6),.075,7,18)
        ellipsoid(helmet,(sign*.15,-.65,.14),(.085,.16,.060),9,24,16)
        # Arneses planos sobre el pecho, sin esferas añadidas.
        pts=[(sign*.15,.014,.72),(sign*.14,-.033,.65),(sign*.12,-.048,.52),(sign*.10,-.008,.33)]
        vv=[]
        for xx,yy,zz in catmull(pts,5):vv.extend([(xx-.023,yy-.007,zz),(xx+.023,yy-.007,zz)])
        harness.add(vv,[(2*i,2*i+1,2*i+3,2*i+2) for i in range(len(vv)//2-1)],10)
    box(details,(0,-.043,.45),(.105,.035,.115),5)
    for z in (.421,.456,.49):box(details,(0,-.063,z),(.069,.008,.009),2)
    box(details,(0,-.036,.315),(.075,.024,.044),2)
    for mesh,name in ((suit,'Uniforme y extremidades sentadas'),(face,'Cabeza protegida y cuello'),(helmet,'Guantes y botas'),(harness,'Arneses y correas'),(details,'Equipo de vuelo')):
        if mesh.f:obj(mesh,'PILOTO | '+name,c,mats,r)
    r['description']='Piloto de la guardia de Naboo, sentado con ambas manos en los mandos; no es Din Djarin'
    print('PILOTO LISTO',flush=True)

def r2(mats):
    c=coll('05 | Unidad R2 embarcada',coll(MODEL));r=bpy.data.objects['N1_R2_ROOT'];ship=bpy.data.objects['Naboo_N1_ROOT'];y=1.42
    body=Mesh();dome=Mesh();panels=Mesh();eyes=Mesh();socket=Mesh();details=Mesh()
    lathe(socket,[(.31,.48),(.36,.54),(.36,.62),(.315,.68),(.285,.68),(.285,.48)],(0,y,0),(0,0,1),0,72)
    lathe(socket,[(.281,.60),(.299,.60),(.299,.68),(.281,.68),(.281,.60)],(0,y,0),(0,0,1),5,72)
    lathe(body,[(0,.23),(.265,.23),(.265,.79),(0,.79)],(0,y,0),(0,0,1),11,64)
    # Domo hemisférico cromado, con paneles azules curvos integrados.
    n=96;rows=28;v=[];f=[];colors=[]
    for j in range(rows+1):
        lat=(pi/2-.001)*j/rows
        for k in range(n):
            a=TAU*k/n;v.append((.285*cos(lat)*cos(a),y+.285*cos(lat)*sin(a),.80+.285*sin(lat)))
    for j in range(rows):
        for k in range(n):
            a=TAU*(k+.5)/n;sector=int((a+.12)/(TAU/6))%6
            # Seis paneles reales separados por juntas plateadas.
            in_panel=(k%16 not in (0,1,14,15)) and (7<j<23)
            f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));colors.append(12 if in_panel else 1)
    dome.add(v,f,1,True);dome.mi[:]=colors
    tube(details,[(.287*cos(TAU*k/120),y+.287*sin(TAU*k/120),.804) for k in range(120)],.017,1,10,True)
    tube(details,[(.274*cos(TAU*k/120),y+.274*sin(TAU*k/120),.759) for k in range(120)],.015,12,10,True)
    # Bloque del fotoreceptor, lente negra, proyector y testigos.
    rounded_box('R2 | Carcasa azul del ojo',c,mats,r,(0,y-.247,.925),(.155,.075,.132),12,.014)
    ellipsoid(eyes,(0,y-.295,.934),(.057,.025,.055),5,32,20)
    ellipsoid(eyes,(0,y-.315,.934),(.043,.018,.042),13,32,20)
    ellipsoid(eyes,(.012,y-.33,.95),(.010,.004,.011),1,16,12)
    for xx,z,color,rad in ((-.15,.842,15,.025),(.16,.84,16,.019),(.12,.98,1,.022)):
        yy=y-math.sqrt(max(.001,.285**2-xx**2-(z-.8)**2))-.009
        ellipsoid(eyes,(xx,yy,z),(rad,.012,rad),color,24,16)
    for xx in (-.11,0,.11):
        box(panels,(xx,y-.263,.711),(.082,.018,.040),12)
    for sign in (-1,1):
        box(panels,(sign*.236,y-.10,.52),(.038,.16,.24),12)
        for j in range(4):box(details,(sign*.258,y-.11,.46+j*.035),(.009,.11,.012),2)
    for mesh,name,parent in ((socket,'Receptaculo del astromecanico',ship),(body,'Cuerpo blanco encastrado',r),(dome,'Domo cromado con paneles azules',r),(panels,'Paneles del cuerpo',r),(eyes,'Fotoreceptor y luces',r),(details,'Anillos y rejillas',r)):
        obj(mesh,'R2 | '+name,c,mats,parent)
    r['description']='Astromecanico R2 azul y blanco; domo expuesto detras de la cabina y cuerpo encastrado'
    print('R2 LISTO',flush=True)

def central_y(y):
    if y<=-3:return y
    if y<=1:return -3+(y+3)*1.30
    return 2.2+(y-1)*1.50

def revised_proportions():
    """Proporciones de las nuevas imágenes: centro largo y cabina enrasada."""
    c=bpy.data.collections[MODEL]
    cabin=set(bpy.data.collections['03 | Cabina transparente e interior'].all_objects)
    droid=set(bpy.data.collections['05 | Unidad R2 embarcada'].all_objects)
    r2_shift=central_y(1.42)-1.42
    for ob in list(c.all_objects):
        if ob.type!='MESH':continue
        if ob in droid:
            for v in ob.data.vertices:v.co.y+=r2_shift
        elif ob.name.startswith(('PILOTO |','CASCO PILOTO |')):
            for v in ob.data.vertices:v.co.y=central_y(v.co.y);v.co.z-=.33
        elif ob in cabin:
            surface=ob.name in ('N1 | Cristal transparente de cabina','N1 | Arcos metalicos de cabina','N1 | Marco amarillo de cabina')
            for v in ob.data.vertices:
                v.co.y=central_y(v.co.y)
                if surface:v.co.z=.62+(v.co.z-.62)*.44
                elif ob.name=='N1 | Banera interior de cabina':v.co.z=.61+(v.co.z-.61)*1.70
                else:v.co.z-=.33
        elif ob.name.startswith(('N1 | Fuselaje','N1 | Juntas del casco','N1 | Canones y rejillas','N1 | Terminales y remaches')):
            for v in ob.data.vertices:v.co.y=central_y(v.co.y)
        ob.data.update()
    ship=bpy.data.objects['Naboo_N1_ROOT'];ship['model_length_m']=14.7
    ship['revision']='Fuselaje central alargado, cupula baja y casco integral granate segun nuevas referencias'
    print('PROPORCIONES REVISADAS: cuerpo central +3.7 m, cupula de 0.36 m',flush=True)

def pivots():
    for name,pivot in (('N1_Pilot_ROOT',(0,central_y(.14),-.15)),('N1_R2_ROOT',(0,central_y(1.42),.62)),('N1_Canopy_ROOT',(0,central_y(.84),.62))):
        ob=bpy.data.objects[name];matrices={child:child.matrix_world.copy() for child in ob.children}
        ob.location=pivot;bpy.context.view_layer.update()
        for child,matrix in matrices.items():child.matrix_world=matrix

def presentation(schedule_preview=True,filename='naboo_n1_royal_ship.blend'):
    s=bpy.context.scene;c=coll('90 | Presentacion y camaras')
    def camera(name,pos,target,scale):
        data=bpy.data.cameras.new(name);ob=bpy.data.objects.new(name,data);c.objects.link(ob);ob.location=pos;ob.rotation_euler=(Vector(target)-ob.location).to_track_quat('-Z','Y').to_euler();enum(data,'type','ORTHO');data.ortho_scale=scale;data.clip_end=500;return ob
    hero=camera('N1 CAM | Principal',(13,-18,12),(0,1.15,.12),15.8)
    top=camera('N1 CAM | Superior',(0,2.35,22),(0,2.35,0),16.0)
    top.rotation_euler=(top.rotation_euler.to_matrix().to_4x4()@Matrix.Rotation(pi/2,4,'Z')).to_euler()
    camera('N1 CAM | Frontal',(0,-22,5.3),(0,-.7,.25),9.6)
    camera('N1 CAM | Lateral',(18,2.35,3.5),(0,2.35,.25),16.0)
    camera('N1 CAM | Cabina y R2',(4,-3.5,3.0),(0,1.60,.70),4.7)
    camera('N1 CAM | Piloto',(1.65,-1.5,1.9),(0,1.0,.60),1.65)
    camera('N1 CAM | Casco',(.65,-1.4,1.0),(0,.97,.72),.95)
    s.camera=hero
    world=bpy.data.worlds.new('N1 | Entorno de reflejos y espacio');world.use_nodes=True;s.world=world
    nt=world.node_tree;bg=next(n for n in nt.nodes if n.type=='BACKGROUND');lp=nt.nodes.new('ShaderNodeLightPath');mix=nt.nodes.new('ShaderNodeMixRGB')
    mix.inputs[1].default_value=(.19,.23,.29,1);mix.inputs[2].default_value=(.0025,.006,.017,1)
    nt.links.new(lp.outputs['Is Camera Ray'],mix.inputs[0]);nt.links.new(mix.outputs[0],bg.inputs['Color']);bg.inputs['Strength'].default_value=.55
    def area(name,pos,power,size,color,size_y=None):
        data=bpy.data.lights.new(name,next(i.identifier for i in bpy.types.Light.bl_rna.properties['type'].enum_items if i.identifier=='AREA'))
        data.energy=power;data.size=size;data.color=color
        if size_y is not None:enum(data,'shape','RECTANGLE');data.size_y=size_y
        ob=bpy.data.objects.new(name,data);c.objects.link(ob);ob.location=pos;ob.rotation_euler=(Vector((0,-.5,0))-ob.location).to_track_quat('-Z','Y').to_euler()
    area('N1 LUZ | Caja superior',(-3,-4,9),1400,9,(1,.91,.75),5)
    area('N1 LUZ | Reflejo frontal',(0,-9,6),900,7,(.84,.93,1),2)
    area('N1 LUZ | Banda lateral',(7,2,5),1400,8,(.71,.84,1),2.5)
    area('N1 LUZ | Contraluz',(-6,5,4),1700,7,(1,.79,.42),2)
    area('N1 LUZ | Cabina',(0,-1,5),140,2,(1,.88,.73))
    # Estrellas lejanas de presentación, no forman parte de la nave exportada.
    stars=Mesh();rng=random.Random(1977);rot=hero.rotation_euler.to_quaternion();right=rot@Vector((1,0,0));up=rot@Vector((0,1,0));normal=rot@Vector((0,0,1))
    for i in range(260):
        pos=-normal*35+right*rng.uniform(-12,12)+up*rng.uniform(-10,10);radius=rng.uniform(.006,.018)
        stars.add([pos]+[pos+right*radius*cos(TAU*k/8)+up*radius*sin(TAU*k/8) for k in range(8)],[(0,k+1,(k+1)%8+1) for k in range(8)],0)
    mat=material('N1 | Estrellas',(.57,.69,.9),.6);p=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Emission Color'].default_value=(.57,.69,.9,1);p.inputs['Emission Strength'].default_value=2
    obj(stars,'N1 STAGE | Estrellas',c,[mat],None)
    try:s.render.engine='CYCLES'
    except TypeError as exc:print(exc)
    s.cycles.samples=64;s.cycles.use_denoising=True;s.cycles.transparent_max_bounces=16
    try:
        prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.get_devices();typ=next((d.type for d in prefs.devices if d.type=='OPTIX'),None)
        if typ:
            prefs.compute_device_type=typ;prefs.get_devices()
            for d in prefs.devices:d.use=d.type==typ
            enum(s.cycles,'device','GPU')
    except Exception as exc:print('GPU',exc)
    s.render.resolution_x=2200;s.render.resolution_y=1400;s.render.resolution_percentage=55;s.view_settings.exposure=0
    try:s.view_settings.look='AgX - Medium High Contrast'
    except (TypeError,ValueError):pass
    enum(s.render.image_settings,'file_format','PNG');enum(s.render.image_settings,'color_mode','RGB')
    s.render.filepath=str(REVIEWS/'Naboo_N1_Preview.png')
    for screen in bpy.data.screens:
        for ar in screen.areas:
            if ar.type=='VIEW_3D':
                space=ar.spaces.active;space.region_3d.view_location=(0,0,0);space.region_3d.view_distance=16;space.region_3d.view_rotation=hero.rotation_euler.to_quaternion();enum(space.region_3d,'view_perspective','CAMERA');enum(space.shading,'type','MATERIAL');space.shading.use_scene_world=True;space.shading.use_scene_lights=True;space.overlay.show_overlays=False
    text=bpy.data.texts.new('README | Naboo N-1')
    text.write('NABOO N-1 DEL EPISODIO I\nFuente editable en ProyectoFinal/Nave_Naboo_N1.\nPiloto de la Guardia de Naboo a bordo y R2 encastrado detras de la cabina.\nNaboo_N1_ROOT mueve la nave. N1_Pilot_ROOT, N1_R2_ROOT y N1_Canopy_ROOT son controles independientes.\nLa variante del Mandaloriano no esta creada; pendiente del banderazo del usuario.\nNo se integra todavia en Planet-Hoppers.\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/filename))
    def preview():bpy.ops.render.render(write_still=True);return None
    if schedule_preview:bpy.app.timers.register(preview,first_interval=1)
    print('GUARDADO; PREVIEW EN CURSO',flush=True)

def build():
    init();mats=palette();hull(mats);engines(mats);canopy(mats);pilot(mats);r2(mats);revised_proportions();pivots();presentation()

if __name__=='__main__':build()
