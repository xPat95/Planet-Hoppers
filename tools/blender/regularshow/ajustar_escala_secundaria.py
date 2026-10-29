"""La casa principal domina el parque; las tres zonas secundarias se reducen al 65 %."""
import bpy
import os
import importlib.util
from mathutils import Vector, Matrix

ROOT=os.path.dirname(os.path.abspath(__file__))
spec=importlib.util.spec_from_file_location('domo_escala_helpers',os.path.join(ROOT,'crear_domo.py'))
D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D)

def aplicar(factor=.65):
    root=bpy.data.objects['RegularShow_Planet_ROOT']
    setups=[('10 | Snacks y terraza de mesas','ZONA | Claro de Snacks',(-3.65,-2.75),'DOMO | Camara snacks',(-3.92,-2.92,.35)),
            ('11 | Caravana y campamento','ZONA | Claro de Caravana',(4.05,3.50),'DOMO | Camara caravana',(4.05,3.48,.40)),
            ('12 | Taller y surtidor antiguo','ZONA | Claro de Taller',(3.35,-3.25),'DOMO | Camara taller',(3.35,-3.42,.42))]
    for collection,pad,xy,camera_name,target in setups:
        pivot=Vector((*xy,D.ground(*xy)+.03))
        objects=list(bpy.data.collections[collection].objects)+[bpy.data.objects[pad]]
        for ob in objects:
            previous=float(ob.get('Escala secundaria',1.0))
            ratio=factor/previous
            transform=root.matrix_world@Matrix.Translation(pivot)@Matrix.Scale(ratio,4)@Matrix.Translation(-pivot)@root.matrix_world.inverted()
            ob.matrix_world=transform@ob.matrix_world
            ob['Escala secundaria']=factor
        camera=bpy.data.objects.get(camera_name)
        if camera:
            previous=float(camera.get('Escala secundaria',1.0));ratio=factor/previous
            old=pivot+(Vector(target)-pivot)*previous
            new=pivot+(Vector(target)-pivot)*factor
            camera.location+=new-old
            camera.data.ortho_scale*=ratio
            camera['Escala secundaria']=factor
    bpy.context.view_layer.update()
    def bounds(objects):
        points=[ob.matrix_world@Vector(v) for ob in objects if ob.type=='MESH' for v in ob.bound_box]
        return tuple(max(p[i] for p in points)-min(p[i] for p in points) for i in range(3))
    house=bounds(bpy.data.collections['07 | Casa del parque segun referencia'].objects)
    for collection,_,_,_,_ in setups:
        size=bounds(bpy.data.collections[collection].objects)
        assert all(size[i]<house[i] for i in range(3)),(collection,size,house)
        print(collection,tuple(round(v,3) for v in size))
    bpy.context.scene['Jerarquia de escala']='Casa principal intacta; snacks, caravana y taller reducidos al 65 %'
    print('Casa principal:',tuple(round(v,3) for v in house),'mayor que cada zona secundaria en los tres ejes.')

if __name__=='__main__':aplicar()
