"""N-1 reconstruido de Din Djarin con Grogu en la subcabina posterior.
Parte de una copia local de la construcción N-1 aprobada; no modifica el original.
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
import base_n1_aprobado as base
from geometria_n1 import *
import bmesh
import random

MODEL='NAVE STAR WARS | N1 Mandaloriano y Grogu'
base.MODEL=MODEL
obj=base.obj

def palette():
    mats=base.palette()
    # IDs de la base conservados para casco/alas, ahora metal desnudo.
    for index,color,rough,metal in ((0,(.32,.37,.42),.32,.93),(1,(.50,.57,.64),.22,1),(2,(.22,.28,.34),.37,.86),(3,(.025,.033,.042),.52,.55),(7,(.045,.031,.022),.90,0),(9,(.025,.019,.014),.85,0)):
        p=next(n for n in mats[index].node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Base Color'].default_value=(*color,1);p.inputs['Roughness'].default_value=rough;p.inputs['Metallic'].default_value=metal;mats[index].diffuse_color=(*color,1)
    extra=[('Beskar claro',(.48,.54,.61),.24,.98), #25
           ('Beskar sombra',(.20,.255,.32),.29,.95), #26
           ('Visor T negro',(.002,.004,.007),.16,.25), #27
           ('Cuero oscuro',(.055,.031,.016),.85,0), #28
           ('Pintura amarilla restante',(.72,.39,.005),.63,.24), #29
           ('Cobre envejecido',(.24,.095,.038),.48,.73), #30
           ('Cable rojo',(.18,.017,.011),.74,.05), #31
           ('Grogu piel verde',(.255,.34,.16),.74,0), #32
           ('Grogu piel clara',(.285,.365,.19),.77,0), #33
           ('Grogu orejas interior',(.30,.18,.135),.83,0), #34
           ('Grogu ojos negros',(.004,.006,.004),.075,.05), #35
           ('Grogu tunica',(.26,.205,.13),.97,0), #36
           ('Grogu cuello lana',(.46,.385,.27),.97,0), #37
           ('Grogu pliegues',(.16,.13,.075),.99,0), #38
           ('Grogu arrugas',(.15,.21,.085),.91,0), #39
           ('Metal quemado',(.045,.052,.063),.69,.73), #40
           ('Capa gris parda',(.034,.031,.025),.98,0)] #41
    mats.extend(material('MANDO | '+name,col,rough,metal) for name,col,rough,metal in extra)
    for i in (25,26):
        p=next(n for n in mats[i].node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Coat Weight'].default_value=.15
    glass=material('MANDO | Cristal claro subcabina Grogu',(.48,.68,.78),.025)
    p=next(n for n in glass.node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Alpha'].default_value=.10;p.inputs['Transmission Weight'].default_value=.95;p.inputs['IOR'].default_value=1.45
    glass.diffuse_color=(.48,.68,.78,.10);mats.append(glass)
    mats[0].name='MANDO | Metal desnudo del casco'
    return mats

def helmet_mando(mats,c,r):
    dome=Mesh();lower=Mesh();visor=Mesh();cheeks=Mesh();trim=Mesh();ears=Mesh()
    n=72;rows=24;v=[];f=[]
    for j in range(rows+1):
        lat=.245+(pi/2-.246)*j/rows
        v.extend((.19*cos(lat)*cos(TAU*k/n),.025+.178*cos(lat)*sin(TAU*k/n),1.02+.22*sin(lat)) for k in range(n))
    for j in range(rows):
        for k in range(n):f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k))
    dome.add(v,f,25,True)
    profile=[(.79,.13,.13),(.85,.172,.153),(.97,.19,.176),(1.075,.185,.173)]
    v=[];f=[];colors=[]
    for z,rx,ry in base.smooth_profile(profile,5):
        v.extend((rx*sin(TAU*k/n),.025-ry*cos(TAU*k/n),z) for k in range(n))
    rings=len(v)//n
    for j in range(rings-1):
        for k in range(n):
            a=TAU*(k+.5)/n;front=min(a,TAU-a)<1.25
            f.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));colors.append(27 if front else 26)
    lower.add(v,f,26,True);lower.mi[:]=colors
    # Banda horizontal negra y ranura vertical forman una T continua.
    vv=[];ff=[]
    for z in (1.025,1.077):
        for k in range(65):
            a=-1.23+2.46*k/64;vv.append((.19*sin(a),.025-.181*cos(a)-.004,z))
    ff=[(k,k+1,65+k+1,65+k) for k in range(64)];visor.add(vv,ff,27,True)
    visor.face([(-.026,-.212,.797),(.026,-.212,.797),(.026,-.212,1.058),(-.026,-.212,1.058)],27)
    # Dos carrilleras angulares, con crestas y planos de distinto brillo.
    for sign in (-1,1):
        p=[(sign*.18,-.105,1.025),(sign*.055,-.195,1.017),(sign*.045,-.205,.956),
           (sign*.033,-.194,.803),(sign*.132,-.119,.792),(sign*.185,-.070,.86),(sign*.12,-.200,.943)]
        cheeks.add(p,[(0,1,2,6),(2,3,4,6),(0,6,4,5)],25)
        cheeks.mi[-3:]=[25,26,25]
        rounded_box('CASCO PILOTO | Auricular beskar '+str(sign),c,mats,r,(sign*.189,.028,.987),(.045,.105,.172),26,.009)
        for j in range(3):tube(ears,[(sign*.215,-.003,.94+j*.023),(sign*.215,.056,.94+j*.023)],.0035,27,6)
        tube(trim,[(sign*.185*sin(-1.22+2.44*k/80),.025-.181*cos(-1.22+2.44*k/80)-.005,1.087) for k in range(81)],.0045,25,8)
    # Cresta central longitudinal de la coronilla.
    vv=[];ff=[]
    for j in range(60):
        a=.26+(pi-.52)*j/59
        for sign in (-1,1):vv.append((sign*.016,.025-.181*cos(a),1.02+.224*sin(a)))
    ff=[(2*j,2*j+1,2*j+3,2*j+2) for j in range(59)];trim.add(vv,ff,25,True)
    for sign in (-1,1):tube(trim,[(sign*.017,.025-.181*cos(.26+(pi-.52)*j/59),1.02+.224*sin(.26+(pi-.52)*j/59)) for j in range(60)],.0028,26,6)
    for mesh,name in ((dome,'Cupula beskar'),(lower,'Casco envolvente'),(visor,'Visor negro en T'),(cheeks,'Carrilleras angulares'),(trim,'Borde y cresta central'),(ears,'Rejillas laterales')):
        obj(mesh,'CASCO PILOTO | '+name,c,mats,r)

def mando(mats):
    c=coll('04 | Piloto de la guardia de Naboo',coll(MODEL));r=bpy.data.objects['N1_Pilot_ROOT']
    suit=Mesh();armor=Mesh();gloves=Mesh();strap=Mesh();cape=Mesh();detail=Mesh()
    ellipsoid(suit,(0,.14,.49),(.23,.17,.285),7,32,24)
    ellipsoid(suit,(0,.07,.79),(.075,.068,.09),7,24,16)
    helmet_mando(mats,c,r)
    # Coraza bipartita y pequeño diamante central de beskar.
    for sign in (-1,1):
        p=[(sign*.020,-.012,.69),(sign*.16,.013,.68),(sign*.20,-.023,.56),(sign*.13,-.043,.50),(sign*.025,-.055,.54)]
        armor.add(p,[(0,1,2,3,4)],25)
        ellipsoid(armor,(sign*.192,.09,.67),(.086,.105,.080),25,28,18)
        pts=catmull([(sign*.18,.10,.66),(sign*.28,-.07,.49),(sign*.235,-.36,.45),(sign*.23,-.54,.46)],6);tube(suit,pts,.063,7,16)
        tube(armor,catmull([(sign*.25,-.25,.47),(sign*.235,-.39,.45)],4),.069,26,16)
        ellipsoid(gloves,(sign*.23,-.535,.47),(.046,.06,.044),28,24,16)
        tube(suit,catmull([(sign*.105,.17,.27),(sign*.15,-.22,.19),(sign*.15,-.53,.13)],6),.075,7,18)
        ellipsoid(gloves,(sign*.15,-.65,.14),(.085,.16,.060),28,24,16)
        for j in range(3):stroke(detail,[(sign*.23+(j-1)*.02,-.58,.483),(sign*.23+(j-1)*.02,-.59,.454)],.003,5,2)
    armor.face([(0,-.063,.625),(.023,-.065,.59),(0,-.067,.555),(-.023,-.065,.59)],26)
    vv=[]
    path=catmull([(-.15,.01,.70),(-.08,-.046,.60),(.015,-.055,.46),(.11,.008,.30)],6)
    for p in path:vv.extend([p+Vector((-.019,-.006,0)),p+Vector((.019,-.006,0))])
    strap.add(vv,[(2*j,2*j+1,2*j+3,2*j+2) for j in range(len(path)-1)],28)
    for j in range(5):
        p=path[2+j*3];box(detail,p+Vector((0,-.014,0)),(.053,.019,.019),30)
    box(detail,(0,-.032,.31),(.075,.025,.044),26)
    # Capa doblada detrás del asiento, sin invadir los mandos.
    vv=[];ff=[]
    for j in range(12):
        z=.76-j*.048
        for k in range(13):
            x=-.21+.42*k/12;y=.35+.023*cos(k*pi/3)+.08*j/11;vv.append((x,y,z))
    for j in range(11):
        for k in range(12):ff.append((j*13+k,j*13+k+1,(j+1)*13+k+1,(j+1)*13+k))
    cape.add(vv,ff,41,True)
    for mesh,name in ((suit,'Traje de Din Djarin'),(armor,'Coraza hombreras y brazales beskar'),(gloves,'Guantes y botas'),(strap,'Bandolera de cuero'),(cape,'Capa recogida'),(detail,'Cartucheras y herrajes')):
        obj(mesh,'PILOTO | '+name,c,mats,r)
    r['description']='Din Djarin sentado con armadura beskar y casco de visor T'

def engines(mats):
    c=coll('02 | Motores J y agujas',coll(MODEL));r=bpy.data.objects['Naboo_N1_ROOT']
    profile=[(-5,.31),(-4.94,.415),(-4.74,.48),(-4.35,.50),(-3.7,.50),(-2.8,.485),(-2.0,.45),(-1.35,.37),(-.85,.25),(-.55,.16),(.4,.085),(1.5,.045),(2.5,.018),(3.20,.002)]
    prof=base.smooth_profile(profile,5);n=72
    for sign,label in ((-1,'babor'),(1,'estribor')):
        x=sign*3.5;shell=Mesh();core=Mesh();ribs=Mesh();tubes=Mesh();fans=Mesh();nozzle=Mesh();glow=Mesh();vv=[];ff=[]
        for y,rad in prof:vv.extend((x+max(.002,rad)*cos(TAU*k/n),y,.075+max(.002,rad)*sin(TAU*k/n)) for k in range(n))
        for j in range(len(prof)-1):
            y=(prof[j][0]+prof[j+1][0])/2
            for k in range(n):
                a=TAU*(k+.5)/n
                # Paneles retirados en la mayor parte superior del motor.
                if -4.50<y<-1.45 and sin(a)>-.30:continue
                ff.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k))
        shell.add(vv,ff,2,True)
        lathe(core,[(.04,-4.99),(.27,-4.88),(.31,-4.55),(.31,-1.55),(.24,-1.1),(.12,-.55)],(x,0,.075),(0,1,0),40,72)
        for y in (-4.85,-4.43,-3.85,-3.30,-2.75,-2.20,-1.60,-1.10):
            _,rad=base.interpolate(profile,y)
            for yy in (y-.025,y+.025):tube(ribs,[(x+rad*cos(TAU*k/100),yy,.075+rad*sin(TAU*k/100)) for k in range(100)],.022,2,9,True)
            for j in range(20):
                a=TAU*j/20;ellipsoid(ribs,(x+(rad+.01)*cos(a),y,.075+(rad+.01)*sin(a)),(.012,.016,.012),1,10,8)
        for k in range(8):
            a=TAU*k/8
            pts=[]
            for j in range(40):
                y=-4.6+3.3*j/39;_,rad=base.interpolate(profile,y);pts.append((x+rad*cos(a),y,.075+rad*sin(a)))
            tube(ribs,pts,.020,1,8)
        for k in range(4):
            a=.25*pi+k*pi*.17
            tube(tubes,catmull([(x+.35*cos(a),-4.4,.075+.35*sin(a)),(x+.39*cos(a),-3.1,.075+.39*sin(a)),(x+.32*cos(a),-1.6,.075+.32*sin(a))],10),.018,30 if k%2 else 31,8)
        for y in (-4.9,-4.96):tube(fans,[(x+.30*cos(TAU*k/96),y,.075+.30*sin(TAU*k/96)) for k in range(96)],.020,1,9,True)
        ellipsoid(fans,(x,-4.95,.075),(.10,.065,.10),26,28,20)
        for k in range(12):
            a=TAU*k/12;b=a+.24
            fans.face([(x+.10*cos(a),-4.965,.075+.10*sin(a)),(x+.28*cos(a+.05),-4.97,.075+.28*sin(a+.05)),(x+.28*cos(b),-4.92,.075+.28*sin(b)),(x+.11*cos(b),-4.92,.075+.11*sin(b))],5)
        for y,rad,mi in ((-.87,.276,5),(-.815,.250,14),(-.77,.23,1),(-.71,.195,5)):
            tube(glow if mi==14 else nozzle,[(x+rad*cos(TAU*k/96),y,.075+rad*sin(TAU*k/96)) for k in range(96)],.020,mi,9,True)
        for mesh,name in ((shell,'Paneles abiertos y aguja'),(core,'Nucleo mecanico'),(ribs,'Anillos y largueros'),(tubes,'Tuberias y cableado'),(fans,'Turbina frontal'),(nozzle,'Tobera posterior'),(glow,'Aro ionico azul')):
            obj(mesh,'MOD MOTOR '+label+' | '+name,c,mats,r)

def grogu(mats):
    c=coll('05 | Unidad R2 embarcada',coll(MODEL));r=bpy.data.objects['N1_R2_ROOT'];y=1.42
    skin=Mesh();ears=Mesh();eyes=Mesh();robe=Mesh();collar=Mesh();details=Mesh();hair=Mesh()
    # Cabeza ancha, mejillas bajas y ojos negros grandes del personaje.
    ellipsoid(skin,(0,y-.02,.965),(.20,.135,.16),32,48,32)
    ellipsoid(skin,(0,y-.048,.874),(.144,.075,.055),33,36,24)
    ellipsoid(skin,(0,y-.153,.948),(.028,.025,.020),33,24,16)
    for sign in (-1,1):
        ellipsoid(skin,(sign*.079,y-.119,.982),(.079,.044,.083),33,36,24)
        ellipsoid(eyes,(sign*.079,y-.143,.984),(.064,.034,.069),35,40,28)
        # Párpados y cejas modelados con trazos de piel muy finos.
        stroke(details,[(sign*.025,y-.145,.995),(sign*.074,y-.146,1.065),(sign*.13,y-.115,1.023)],.006,32,9)
        stroke(details,[(sign*.027,y-.143,.957),(sign*.077,y-.146,.920),(sign*.131,y-.116,.951)],.0045,33,9)
        stroke(details,[(sign*.022,y-.118,1.053),(sign*.066,y-.116,1.089),(sign*.132,y-.080,1.052)],.005,39,9)
        ellipsoid(eyes,(sign*.079-.016,y-.174,1.010),(.008,.003,.010),17,16,10)
        ellipsoid(details,(sign*.010,y-.177,.942),(.0034,.002,.003),39,12,8)
        # Orejas largas con interior cóncavo rosado y borde verde.
        outline=[(.148,1.031),(.224,1.044),(.37,1.023),(.445,1.010),(.395,.975),(.31,.934),(.23,.922),(.166,.948),(.148,1.031)]
        points=catmull([(sign*x,y-.018,z) for x,z in outline],5)[:-1];n=len(points);center=Vector((sign*.268,y+.005,.981));vv=[];ff=[];colors=[]
        for j in range(9):
            t=.001+.999*j/8
            for p in points:
                q=center.lerp(p,t);q.y=y+.010-.045*t*t;vv.append(q)
        for j in range(8):
            for k in range(n):ff.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k));colors.append(34 if j<6 else 32)
        ears.add(vv,ff,32,True);ears.mi[-len(colors):]=colors
        tube(ears,[Vector(p)+Vector((0,-.025,0)) for p in points],.010,32,9,True)
        # Manitas de tres dedos asomadas sobre el cuello de lana.
        ellipsoid(skin,(sign*.15,y-.10,.735),(.036,.028,.037),32,24,16)
        for j in range(3):stroke(skin,[(sign*.15+(j-1)*.018,y-.12,.741),(sign*.15+(j-1)*.019,y-.16,.716)],.010,32,4)
        ellipsoid(skin,(sign*.065,y-.015,.47),(.045,.064,.025),32,24,14)
        for j in range(3):ellipsoid(skin,(sign*.065+(j-1)*.020,y-.067,.468),(.012,.026,.012),33,16,10)
    ellipsoid(eyes,(0,y-.137,.882),(.031,.007,.010),35,24,14)
    stroke(details,[(-.031,y-.142,.869),(0,y-.146,.863),(.032,y-.142,.870)],.0035,33,8)
    for j in range(3):
        z=1.069+j*.014
        stroke(details,[(-.06,y-.092+j*.018,z),(0,y-.11+j*.014,z+.014),(.06,y-.092+j*.018,z)],.0025,39,8)
    # Túnica de tela con cuello grueso doblado en dos solapas.
    ellipsoid(robe,(0,y+.015,.625),(.177,.14,.19),36,36,28)
    for sign in (-1,1):
        tube(collar,catmull([(sign*.13,y+.06,.815),(sign*.17,y-.045,.802),(sign*.115,y-.153,.777),(sign*.012,y-.17,.734)],8),.041,37,16)
        ellipsoid(robe,(sign*.16,y-.015,.685),(.064,.095,.09),36,28,20)
        for j in range(3):stroke(details,[(sign*.085,y-.129,.57+j*.035),(sign*.13,y-.08,.59+j*.035)],.003,38,5)
    rng=random.Random(50)
    for j in range(12):
        xx=rng.uniform(-.12,.12);zz=.965+.16*math.sqrt(max(0,1-(xx/.2)**2))
        stroke(hair,[(xx,y-.02,zz),(xx+.003,y-.012,zz+.014),(xx+.008,y+.003,zz+.018)],.0011,33,4)
    for mesh,name in ((skin,'Piel cabeza nariz y manos'),(ears,'Orejas largas e interior rosado'),(eyes,'Ojos negros y boca'),(robe,'Tunica marron'),(collar,'Cuello de lana doblado'),(details,'Parpados arrugas y pliegues'),(hair,'Pelo fino de la coronilla')):
        mesh.v=[(p[0]*.83,y+(p[1]-y)*.83,.65+(p[2]-.65)*.83) for p in mesh.v]
        obj(mesh,'GROGU | '+name,c,mats,r)
    # Subcabina que sustituye por completo la unidad astromecánica.
    pod=Mesh();rim=Mesh();glass=Mesh();interior=Mesh()
    lathe(pod,[(.32,.48),(.46,.56),(.48,.625),(.46,.675),(.425,.675),(.32,.48)],(0,y,0),(0,0,1),2,96)
    tube(rim,[(.455*cos(TAU*k/144),y+.345*sin(TAU*k/144),.675) for k in range(144)],.015,1,10,True)
    ellipsoid(interior,(0,y,.58),(.245,.24,.055),6,36,24)
    n=80;rows=30;vv=[];ff=[]
    for j in range(rows+1):
        a=(pi/2-.001)*j/rows
        for k in range(n):vv.append((.47*cos(a)*cos(TAU*k/n),y+.355*cos(a)*sin(TAU*k/n),.675+.55*sin(a)))
    for j in range(rows):
        for k in range(n):ff.append((j*n+k,j*n+(k+1)%n,(j+1)*n+(k+1)%n,(j+1)*n+k))
    glass.add(vv,ff,42,True)
    for mesh,name in ((pod,'Base metalica de subcabina'),(rim,'Aro de cierre'),(glass,'Cristal de subcabina'),(interior,'Asiento interior')):obj(mesh,'GROGU POD | '+name,c,mats,r)
    r['description']='Grogu en subcabina transparente que reemplaza el alojamiento de R2'

def mechanics(mats):
    c=coll('06 | Mecanica expuesta y reconstruccion',coll(MODEL));r=bpy.data.objects['Naboo_N1_ROOT']
    hull=bpy.data.objects['N1 | Fuselaje amarillo y proa cromada']
    # Retirar la cubierta posterior para mostrar el larguero y sus acoples.
    for ob in (hull,bpy.data.objects['N1 | Juntas del casco']):
        bm=bmesh.new();bm.from_mesh(ob.data)
        bmesh.ops.delete(bm,geom=[v for v in bm.verts if v.co.y>4.75],context='VERTS');bm.to_mesh(ob.data);bm.free();ob.data.update()
    spine=Mesh();cables=Mesh();plates=Mesh();wear=Mesh();booster=Mesh()
    lathe(spine,[(.12,4.55),(.17,4.76),(.14,5.2),(.105,6.2),(.075,7.5),(.045,8.6),(.003,9.7)],(0,0,.12),(0,1,0),2,56)
    for j,y in enumerate((4.68,4.95,5.30,5.62,6.2,6.55,7.3,7.68,8.1)):
        rad=.17 if y<5.5 else .125 if y<7 else .085
        tube(spine,[(rad*cos(TAU*k/80),y,.12+rad*sin(TAU*k/80)) for k in range(80)],.018,1,9,True)
    def surface_z(x,y):
        original=y if y<=-3 else -3+(y+3)/1.3 if y<=2.2 else 1+(y-2.2)/1.5
        _,rx,up,down,z=base.interpolate(base.HULL,original)
        return z+up*math.sqrt(max(0,1-(x/rx)**2))
    for sign in (-1,1):
        tube(cables,catmull([(sign*.12,4.60,.18),(sign*.16,5.6,.22),(sign*.07,7.7,.15),(sign*.02,9.2,.13)],10),.012,30 if sign>0 else 5,8)
    # Huecos de servicio en los dos hombros del casco: fondo oscuro y piezas.
    for sign in (-1,1):
        cx=sign*.47
        for j in range(9):
            yy=-.90+j*.14;zz=surface_z(cx,yy)+.022
            rounded_box('MOD | Rejilla lateral '+str(sign)+' '+str(j),c,mats,r,(cx,yy,zz),(.125,.085,.042),5,.006)
            tube(cables,[(cx,yy,zz+.02),(cx+sign*.05,yy+.05,surface_z(cx+sign*.05,yy+.05)+.03)],.012,30,7)
        # Restos amarillos irregulares en las alas metálicas.
        for j in range(3):
            xx=sign*(1.33+j*.24);yy=-1.7+j*.12
            plates.face([(xx,yy,.17),(xx+sign*.28,yy+.045,.166),(xx+sign*.25,yy+.14,.155),(xx+sign*.03,yy+.09,.17)],29)
        # Parche soldado en el hombro posterior.
        p=[(sign*.34,1.65,.58),(sign*.46,1.78,.54),(sign*.40,2.15,.51),(sign*.31,2.00,.58)]
        plates.face([(xx,yy,surface_z(xx,yy)+.008) for xx,yy,zz in p],2)
        for j in range(5):
            yy=1.80+j*.045;ellipsoid(wear,(sign*.355,yy,surface_z(sign*.355,yy)+.015),(.009,.011,.006),40,10,8)
    # Turbocompresor central inferior y su boca de empuje.
    lathe(booster,[(.04,1.9),(.19,2.08),(.24,2.65),(.24,3.4),(.20,3.70),(.14,3.76),(.14,3.60),(.04,3.60)],(0,0,-.28),(0,1,0),5,64)
    for y in (2.13,2.55,3.15,3.58):tube(booster,[(.245*cos(TAU*k/80),y,-.28+.245*sin(TAU*k/80)) for k in range(80)],.014,2,9,True)
    tube(booster,[(.15*cos(TAU*k/80),3.73,-.28+.15*sin(TAU*k/80)) for k in range(80)],.018,14,9,True)
    # Ralladuras discretas y planas, sin texturas externas necesarias.
    rng=random.Random(501)
    for sign in (-1,1):
        for j in range(20):
            xx=sign*rng.uniform(1.1,2.8);yy=rng.uniform(-1.8,-1.12);zz=.16
            stroke(wear,[(xx,yy,zz),(xx+sign*.08,yy+.025,zz)],.0025,40,1)
    for mesh,name in ((spine,'Espina posterior sin carenado'),(cables,'Conductos y cableado'),(plates,'Parches y pintura restante'),(wear,'Desgaste y soldaduras'),(booster,'Acelerador central inferior')):obj(mesh,'MOD | '+name,c,mats,r)

def rename_and_stage():
    s=bpy.context.scene;s.name='N1_Mandaloriano_Grogu'
    r=bpy.data.objects['Naboo_N1_ROOT'];r.name='Mandalorian_N1_ROOT';r['variant']='Din Djarin custom silver N-1';r['characters']='Din Djarin y Grogu';r['revision']='Proporciones largas aprobadas, casco beskar, motores abiertos y subcabina de Grogu'
    bpy.data.objects['N1_Pilot_ROOT'].name='Din_Djarin_ROOT';bpy.data.objects['N1_R2_ROOT'].name='Grogu_ROOT';bpy.data.objects['N1_Canopy_ROOT'].name='Mando_Canopy_ROOT'
    bpy.data.objects['Din_Djarin_ROOT']['asset_role']='character';bpy.data.objects['Grogu_ROOT']['asset_role']='character'
    pod=base.root('Grogu_Pod_ROOT',bpy.data.collections[MODEL],r,(0,2.83,.675));pod['asset_role']='canopy';pod['occupant']='Grogu'
    bpy.context.view_layer.update()
    for child in list(bpy.data.objects['Grogu_ROOT'].children):
        if child.name.startswith('GROGU POD |'):
            matrix=child.matrix_world.copy();child.parent=pod;child.matrix_world=matrix
    bpy.data.collections['04 | Piloto de la guardia de Naboo'].name='04 | Din Djarin al mando'
    bpy.data.collections['05 | Unidad R2 embarcada'].name='05 | Grogu y su subcabina'
    for ob in list(bpy.data.objects):
        if ob.name.startswith('CASCO PILOTO |'):ob.name=ob.name.replace('CASCO PILOTO |','CASCO MANDO |',1)
        elif ob.name.startswith('PILOTO |'):ob.name=ob.name.replace('PILOTO |','MANDO |',1)
    # Cámara principal y entorno heredados de la nave aprobada.
    base.presentation(schedule_preview=False,filename='mandalorian_n1_ship.blend')
    s.camera=bpy.data.objects['N1 CAM | Principal'];s.render.filepath=str(REVIEWS/'Mandaloriano_N1_Preview.png')
    camera=bpy.data.objects['N1 CAM | Cabina y R2'];camera.name='N1 CAM | Din y Grogu';camera.location=(3.7,-3.2,3.2)
    camera.rotation_euler=(Vector((0,1.55,.73))-camera.location).to_track_quat('-Z','Y').to_euler();camera.data.ortho_scale=4.7
    c=bpy.data.collections['90 | Presentacion y camaras']
    data=bpy.data.cameras.new('N1 CAM | Grogu');ob=bpy.data.objects.new('N1 CAM | Grogu',data);c.objects.link(ob);enum(data,'type','ORTHO');data.ortho_scale=1.38;ob.location=(1.15,.50,1.80);ob.rotation_euler=(Vector((0,2.83,.91))-ob.location).to_track_quat('-Z','Y').to_euler()
    camera=bpy.data.objects['N1 CAM | Casco'];camera.data.ortho_scale=1.03
    camera.rotation_euler=(Vector((0,.97,.69))-camera.location).to_track_quat('-Z','Y').to_euler()
    text=bpy.data.texts.get('README | Naboo N-1')
    if text:text.name='README | N1 Mandaloriano';text.clear();text.write('N-1 DEL MANDALORIANO\nDin Djarin al mando y Grogu en subcabina transparente.\nModelo independiente guardado en ProyectoFinal/Nave_Mandaloriano_N1.\nMandalorian_N1_ROOT mueve la nave; Din_Djarin_ROOT y Grogu_ROOT son personajes separados.\nEl N-1 amarillo aprobado se conserva en su carpeta original.\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'mandalorian_n1_ship.blend'))
    print('MANDALORIANO GUARDADO; vista previa en curso',flush=True)

def build():
    base.init();mats=palette();base.hull(mats);engines(mats);base.canopy(mats);mando(mats);grogu(mats)
    base.revised_proportions();base.pivots();mechanics(mats)
    rename_and_stage()
    def preview():bpy.ops.render.render(write_still=True);return None
    bpy.app.timers.register(preview,first_interval=1)

if __name__=='__main__':build()
