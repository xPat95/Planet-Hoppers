import bpy
import math
from mathutils import Vector, Matrix
from math import sin, cos, pi, sqrt
TAU=2*pi
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
