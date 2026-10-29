"""Casa según referencia y tres motores horizontales en la parte trasera del domo."""
import bpy
import importlib.util
import os
import math
from mathutils import Vector

ROOT=os.path.dirname(os.path.abspath(__file__))
spec=importlib.util.spec_from_file_location('domo_helpers',os.path.join(ROOT,'crear_domo.py'))
D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D)
Mesh,box,tube,lathe,ring,mat=D.Mesh,D.box,D.tube,D.lathe,D.ring,D.mat

def quitar_casa_anterior():
    old=bpy.data.objects.get('DOMO | Casa principal, porche y caseta')
    if old:
        shed=Mesh()
        for face in old.data.polygons:
            if all(old.data.vertices[i].co.x<-2 for i in face.vertices):
                shed.face([old.data.vertices[i].co.copy() for i in face.vertices],face.material_index)
        if shed.f:
            ob=shed.object('DOMO | Caseta de mantenimiento',bpy.data.collections['04 | Casa del parque y mobiliario'],list(old.data.materials))
            ob.parent=bpy.data.objects['RegularShow_Planet_ROOT']
        bpy.data.objects.remove(old,do_unlink=True)
    old=bpy.data.objects.get('DOMO | Tejados y buhardillas')
    if old:bpy.data.objects.remove(old,do_unlink=True)

def casa_fiel():
    name='07 | Casa del parque segun referencia'
    c=D.coll(name,D.coll(D.MODEL))
    for ob in list(c.objects):bpy.data.objects.remove(ob,do_unlink=True)
    materials=[
        mat('CASA | Revestimiento verde',(.145,.39,.255)),
        mat('CASA | Verde claro',(.20,.445,.30)),
        mat('CASA | Molduras marfil',(.83,.835,.73)),
        mat('CASA | Piedra del basamento',(.40,.44,.33)),
        mat('CASA | Piedra clara',(.53,.56,.44)),
        mat('CASA | Ventanas verde oscuro',(.105,.205,.175),.32,.10),
        mat('CASA | Escalones de cemento',(.63,.65,.59)),
        mat('CASA | Interior y juntas',(.075,.11,.095)),
        mat('CASA | Puerta blanca',(.72,.745,.66)),
        mat('CASA | Piso del porche',(.215,.265,.245)),
        mat('CASA | Lineas del revestimiento',(.13,.32,.22)),
        mat('CASA | Manijas de bronce',(.34,.23,.085),.35,.60),
    ]
    # Unidades de arquitectura locales. La casa sigue siendo pequeña en el parque.
    scale=.18
    origin=Vector((-.90,1.65,D.ground(-.90,1.65)))
    walls=Mesh();wood=Mesh();glass=Mesh();trim=Mesh();stairs=Mesh();siding=Mesh()
    roof=Mesh();rooflines=Mesh()
    rm=[mat('CASA | Teja pizarra %02d'%i,(.13+i*.007,.185+i*.007,.20+i*.007)) for i in range(7)]
    rm.append(mat('CASA | Juntas finas del tejado',(.245,.30,.30)))
    def b(mesh,x,y,z,w,d,h,mi):box(mesh,(x,y,z),(w,d,h),mi)
    def line(mesh,points,r=.018,mi=2):tube(mesh,points,r,mi,5)
    def frame(mesh,cx,cy,cz,w,h,mi=2,normal='front',cross=True):
        # Ventana sobre pared frontal o lateral, con marcos geométricos.
        if normal=='front':
            b(glass,cx,cy+.015,cz,w-.07,.026,h-.07,5)
            for x in (cx-w/2,cx+w/2):b(mesh,x,cy,cz,.048,.055,h+.045,mi)
            for z in (cz-h/2,cz+h/2):b(mesh,cx,cy,z,w+.045,.055,.048,mi)
            if cross:b(mesh,cx,cy-.005,cz,w,.038,.038,mi)
        else:
            b(glass,cx,cy,cz,.026,w-.07,h-.07,5)
            for y in (cy-w/2,cy+w/2):b(mesh,cx, y,cz,.055,.048,h+.045,mi)
            for z in (cz-h/2,cz+h/2):b(mesh,cx,cy,z,.055,w+.045,.048,mi)
            if cross:b(mesh,cx,cy,cz,.038,w,.038,mi)
    # Sótano de piedra, terraza elevada y cuerpo de dos pisos.
    b(walls,0,-.32,.88,7.30,6.45,1.76,3)
    b(wood,0,-.32,1.83,7.52,6.65,.16,9)
    b(walls,0,0,3.04,5.65,4.36,2.34,0)
    b(walls,0,0,5.20,5.65,4.36,2.10,0)
    # Revestimiento horizontal en las cuatro caras.
    for j in range(27):
        z=1.98+j*.158
        if z>6.21:break
        b(siding,0,-2.188,z,5.62,.012,.014,10)
        b(siding,0,2.188,z,5.62,.012,.014,10)
        for x in (-2.834,2.834):b(siding,x,0,z,.012,4.35,.014,10)
    for x in (-2.84,2.84):
        for y in (-2.19,2.19):b(trim,x,y,4.08,.075,.075,4.3,2)
    # Porche envolvente con columnas cuadradas y capiteles.
    for x,y in ((-3.55,-3.36),(-1.12,-3.36),(1.12,-3.36),(3.55,-3.36),(-3.55,.05),(3.55,.05),(-3.55,2.56),(3.55,2.56)):
        b(trim,x,y,2.91,.20,.20,2.15,2)
        b(trim,x,y,2.02,.28,.28,.18,2)
        b(trim,x,y,3.92,.28,.28,.16,2)
        b(trim,x,y,2.43,.30,.30,.11,2)
    # Barandillas frontales, laterales y traseras con balaustres individuales.
    def railing(a,bp):
        a=Vector(a);end=Vector(bp)
        for z,r in ((2.38,.037),(1.99,.030)):
            line(trim,[(a.x,a.y,z),(end.x,end.y,z)],r)
        count=max(2,round((end-a).length/.18))
        for j in range(count+1):
            p=a.lerp(end,j/count);b(trim,p.x,p.y,2.19,.040,.040,.39,2)
    railing((-3.55,-3.36,0),(-1.05,-3.36,0));railing((1.05,-3.36,0),(3.55,-3.36,0))
    railing((-3.55,-3.36,0),(-3.55,2.56,0));railing((3.55,-3.36,0),(3.55,.5,0))
    railing((3.55,1.8,0),(3.55,2.56,0));railing((-3.55,2.56,0),(3.55,2.56,0))
    # Dieciséis escalones y muros inclinados al frente.
    steps=16;start=-7.15;end=-3.47
    for i in range(steps):
        y=start+(end-start)*(i+.5)/steps;h=1.83*(i+1)/steps
        b(stairs,0,y,h/2,1.65,(end-start)/steps+.012,h,6)
        b(trim,0,y-(end-start)/steps/2,h+.005,1.66,.018,.020,2)
    for sign in (-1,1):
        x=sign*.95
        pts=[(x-.09,start-.04,0),(x+.09,start-.04,0),(x+.09,end,0),(x-.09,end,0),
             (x-.09,start-.04,.34),(x+.09,start-.04,.34),(x+.09,end,2.42),(x-.09,end,2.42)]
        stairs.add(pts,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7)],6)
        line(trim,[(x,start-.04,.37),(x,end,2.45)],.065)
    # Acceso exterior lateral, visible desde la derecha de la casa.
    for i in range(13):
        h=1.83*(i+1)/13;y=-1.5+3.0*(i+.5)/13
        b(stairs,4.06,y,h/2,.80,3/13+.01,h,6)
    line(trim,[(4.51,-1.5,.36),(4.51,1.55,2.38)],.05)
    line(trim,[(3.60,-1.5,.36),(3.60,1.55,2.38)],.05)
    # Garaje en el basamento y molduras de la puerta.
    frame(trim,2.23,-3.56,.87,1.72,1.52,2,cross=False)
    b(walls,2.23,-3.563,.87,1.62,.035,1.43,8)
    for j in range(5):b(trim,2.23,-3.59,.27+j*.27,1.62,.02,.022,6)
    # Puerta doble con travesaños y ventanas.
    frame(trim,0,-2.225,2.86,1.42,1.93,2,cross=False)
    for x in (-.35,.35):
        b(wood,x,-2.243,2.82,.66,.050,1.74,8)
        frame(trim,x,-2.278,3.20,.57,.81,2,cross=False)
        frame(trim,x,-2.278,2.40,.57,.49,2,cross=False)
    b(trim,0,-2.285,2.85,.045,.045,1.9,2)
    D.ellipsoid(trim,(.11,-2.33,2.82),(.048,.035,.048),11,10,6)
    frame(trim,0,-2.24,3.99,1.44,.25,2,cross=False)
    # Miradores poligonales, con tres caras acristaladas.
    def bay(cx,bottom,top,w,depth,upper=False):
        y=-2.19
        polygon=[(cx-w/2,y),(cx-w/2,y-depth*.48),(cx-w*.32,y-depth),
                 (cx+w*.32,y-depth),(cx+w/2,y-depth*.48),(cx+w/2,y)]
        v=[(x,yy,z) for z in (bottom,top) for x,yy in polygon]
        faces=[tuple(reversed(range(6))),tuple(range(6,12))]+[(i,(i+1)%6,(i+1)%6+6,i+6) for i in range(6)]
        walls.add(v,faces,1 if upper else 0)
        for i in (1,2,3):
            a=Vector((*polygon[i],0));bb=Vector((*polygon[(i+1)%6],0))
            edge=bb-a;side=edge.normalized();normal=Vector((edge.y,-edge.x,0)).normalized()
            p=a+side*.06+normal*.008;q=bb-side*.06+normal*.008
            z0=bottom+.23;z1=top-.09
            glass.face([(p.x,p.y,z0),(q.x,q.y,z0),(q.x,q.y,z1),(p.x,p.y,z1)],5)
            for z in (z0,z1,z0+(z1-z0)*.73):line(trim,[(p.x,p.y,z),(q.x,q.y,z)],.027,1 if upper else 2)
            for v0 in (p,q):line(trim,[(v0.x,v0.y,z0),(v0.x,v0.y,z1)],.029,1 if upper else 2)
        return polygon
    bay(-1.86,2.00,3.95,1.35,.43)
    bay(1.86,2.00,3.95,1.35,.43)
    bay(1.70,4.38,6.29,1.48,.56,True)
    frame(trim,-1.63,-2.225,5.29,.78,1.24,2)
    b(trim,-1.63,-2.26,5.59,.035,.04,.59,2)
    frame(trim,-.22,-2.225,5.52,.43,.47,2, cross=False)
    for x in (-2.86,2.86):
        for y in (-1.24,.25,1.55):
            for z in (2.95,5.25):frame(trim,x,y,z,.69,1.19,2,normal='side')
    for x in (-1.7,0,1.7):
        for z in (2.95,5.25):frame(trim,x,2.21,z,.74,1.15,2)
    # Piedra irregular discreta en el sótano.
    for j in range(19):
        x=-3.30+(j%7)*1.01;z=.25+(j//7)*.48
        if abs(x)<1.10 or x>1.3:continue
        b(walls,x,-3.554,z,.23+(j%3)*.08,.016,.13+(j%2)*.06,4)
    # Cubiertas inclinadas y tejas reales a escala de arquitectura.
    def roofquad(points,rows=9,cols=20):
        p=[Vector(v) for v in points]
        def at(u,v):return p[0].lerp(p[1],u).lerp(p[3].lerp(p[2],u),v)
        # Soporte continuo detrás de las tejas y sus juntas.
        roof.face(p,2)
        normal=(p[1]-p[0]).cross(p[3]-p[0]).normalized()
        if normal.z<0:normal=-normal
        for j in range(rows):
            va=j/rows;vb=(j+1)/rows
            line(rooflines,[at(i/cols,va)+normal*.008 for i in range(cols+1)],.006,7)
            offset=.5 if j%2 else 0
            for k in range(-1,cols):
                ua=max(0,(k+offset)/cols);ub=min(1,(k+1+offset)/cols)
                if ub<=ua:continue
                roof.face([at(ua,va)+normal*.003,at(ub,va)+normal*.003,at(ub,vb)+normal*.003,at(ua,vb)+normal*.003],(j*3+k*5)%7)
                line(rooflines,[at(ub,va)+normal*.008,at(ub,vb)+normal*.008],.005,7)
    # Techo del porche, ancho y envolvente.
    out=[(-3.94,-3.82,3.96),(3.94,-3.82,3.96),(3.94,3.02,3.96),(-3.94,3.02,3.96)]
    inn=[(-2.81,-2.16,4.78),(2.81,-2.16,4.78),(2.81,2.16,4.78),(-2.81,2.16,4.78)]
    for i in range(4):
        k=(i+1)%4;roofquad([out[i],out[k],inn[k],inn[i]],7,24)
        line(trim,[out[i],out[k]],.061)
    # Tejado principal a cuatro aguas.
    e=[(-3.26,-2.61,6.34),(3.26,-2.61,6.34),(3.26,2.61,6.34),(-3.26,2.61,6.34)]
    ridge=[(-1.78,0,7.94),(1.78,0,7.94)]
    roofquad([e[0],e[1],ridge[1],ridge[0]],12,26)
    roofquad([e[2],e[3],ridge[0],ridge[1]],12,26)
    roofquad([e[1],e[2],ridge[1],ridge[1]],10,16)
    roofquad([e[3],e[0],ridge[0],ridge[0]],10,16)
    for i in range(4):line(trim,[e[i],e[(i+1)%4]],.065)
    # Buhardilla frontal con tres ventanas y tímpano blanco rayado.
    b(walls,0,-.76,7.88,1.88,1.48,1.28,0)
    for x in (-.58,0,.58):frame(trim,x,-1.515,7.91,.43,.91,2)
    a=(-1.06,-1.63,8.48);bb=(1.06,-1.63,8.48);peak=(0,-1.63,9.14)
    walls.face([a,bb,peak],2)
    walls.face([(-1.06,.14,8.48),(0,.14,9.14),(1.06,.14,8.48)],2)
    roofquad([(-1.12,-1.69,8.49),(0,-1.69,9.18),(0,.23,9.18),(-1.12,.23,8.49)],9,10)
    roofquad([(0,-1.69,9.18),(1.12,-1.69,8.49),(1.12,.23,8.49),(0,.23,9.18)],9,10)
    line(trim,[a,peak,bb],.05)
    for j in range(-8,9):
        x=j*.105;top=9.12-abs(x)/1.06*.66
        line(siding,[(x,-1.638,8.50),(x,-1.638,top)],.010,7)
    # Pequeña cubierta sobre el mirador derecho.
    roofquad([(.85,-2.88,6.36),(2.57,-2.88,6.36),(2.43,-2.14,6.68),(.99,-2.14,6.68)],5,10)
    line(trim,[(.85,-2.88,6.36),(2.57,-2.88,6.36)],.045)
    # Chimenea alta en el lado izquierdo, con remate abierto.
    b(walls,-1.92,-.15,7.96,.58,.64,2.22,4)
    for x in (-2.245,-1.595):b(trim,x,-.15,9.11,.12,.82,.19,4)
    for y in (-.51,.21):b(trim,-1.92,y,9.11,.70,.10,.19,4)
    b(walls,-1.92,-.15,9.04,.51,.53,.018,7)
    for j in range(8):
        z=7.00+j*.245
        b(walls,-2.219,-.27+(j%2)*.16,z,.025,.22,.13,3)
        b(walls,-1.99+(j%2)*.21,-.483,z+.13,.20,.025,.13,3)
    # Ménsulas bajo los aleros.
    for x in (-2.75,-1.5,0,1.5,2.75):b(trim,x,-2.30,6.26,.11,.26,.12,2)
    groups=[(walls,'CASA | Volumen verde y basamento',materials),
            (wood,'CASA | Suelos y puertas',materials),(glass,'CASA | Ventanas y miradores',materials),
            (trim,'CASA | Columnas, marcos y barandillas',materials),
            (stairs,'CASA | Escalinatas y muros de acceso',materials),
            (siding,'CASA | Revestimiento horizontal',materials),
            (roof,'CASA | Techos inclinados y tejas',rm),(rooflines,'CASA | Juntas de las tejas',rm)]
    for mesh,name,mats in groups:
        ob=mesh.object(name,c,mats)
        for vertex in ob.data.vertices:vertex.co=origin+vertex.co*scale
        ob.parent=bpy.data.objects['RegularShow_Planet_ROOT']
    print('Casa reconstruida: revestimiento verde, basamento, dos niveles, porche, miradores, buhardilla, chimenea y escalinatas.')

def motores_traseros():
    old=bpy.data.collections.get('08 | Motores laterales horizontales')
    if old:
        for ob in list(old.objects):bpy.data.objects.remove(ob,do_unlink=True)
        bpy.data.collections.remove(old)
    c=D.coll('08 | Tres motores traseros horizontales',D.coll(D.MODEL))
    for ob in list(c.objects):bpy.data.objects.remove(ob,do_unlink=True)
    materials=[bpy.data.materials['DOMO | Metal de los motores'],bpy.data.materials['DOMO | Borde metalico claro'],bpy.data.materials['DOMO | Interior de las toberas'],bpy.data.materials['DOMO | Uniones de los motores']]
    for x,label in ((-2.65,'izquierdo'),(0.0,'central'),(2.65,'derecho')):
        body=Mesh();edges=Mesh();detail=Mesh()
        lathe(body,[(.45,-.12),(.50,.04),(.50,.17),(.41,.30),(.36,.49),(.40,.65),(.49,.88),(.61,1.16),(.68,1.38),(.68,1.45),(.60,1.45),(.56,1.19),(.43,.91),(.29,.70),(.20,.61)],mi=0,segments=80)
        lathe(body,[(.20,.61),(0,.61)],mi=2,segments=64)
        for r,z,w in ((.50,.10,.035),(.40,.35,.028),(.36,.50,.026),(.68,1.41,.034)):
            ring(edges,r,z,w,1,steps=112)
        for k in range(12):
            a=D.TAU*k/12
            tube(detail,[(r*math.cos(a),r*math.sin(a),z) for r,z in ((.405,.32),(.37,.49),(.405,.65),(.495,.89),(.615,1.17),(.687,1.40))],.010,3,5)
        # La fachada de la casa mira a -Y. Su espalda y la popa del domo miran a +Y.
        # Las tres toberas son paralelas y quedan montadas en el arco posterior.
        n=Vector((0,1,0));u=Vector((1,0,0));v=n.cross(u)
        origin=Vector((x,math.sqrt(7.16**2-x**2),-.64))
        for mesh,kind in ((body,'Campana hueca'),(edges,'Aros'),(detail,'Paneles')):
            ob=mesh.object('DOMO | Motor trasero '+label+' | '+kind,c,materials)
            for vertex in ob.data.vertices:
                p=vertex.co.copy();vertex.co=origin+u*p.x+v*p.y+n*p.z
            ob.parent=bpy.data.objects['RegularShow_Planet_ROOT']
    print('Tres motores traseros horizontales: izquierdo, central y derecho, todos hacia +Y.')
    stage=D.coll('90 | Presentacion espacial (no exportar)')
    name='DOMO | Camara trasera';camera=bpy.data.objects.get(name)
    if camera is None:
        data=bpy.data.cameras.new(name);camera=bpy.data.objects.new(name,data);stage.objects.link(camera)
    camera.location=(0,32,10)
    camera.rotation_euler=(Vector((0,0,.1))-camera.location).to_track_quat('-Z','Y').to_euler()
    D.enum(camera.data,'type','ORTHO');camera.data.ortho_scale=18.8

def aplicar():
    quitar_casa_anterior();casa_fiel();motores_traseros()
    stage=D.coll('90 | Presentacion espacial (no exportar)')
    name='DOMO | Camara detalle de la casa'
    camera=bpy.data.objects.get(name)
    if camera is None:
        data=bpy.data.cameras.new(name);camera=bpy.data.objects.new(name,data);stage.objects.link(camera)
    camera.location=(1.9,-3.95,3.30)
    camera.rotation_euler=(Vector((-.9,1.50,.87))-camera.location).to_track_quat('-Z','Y').to_euler()
    D.enum(camera.data,'type','ORTHO');camera.data.ortho_scale=2.90
    print('Mejoras aplicadas; parque, cupula y cuatro motores inferiores conservados.')

if __name__=='__main__':aplicar()
