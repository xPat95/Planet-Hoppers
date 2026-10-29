"""Prepara fuente portable, exportaciones near/far y auditoría de un planeta.

Ejecutar en una sesión independiente de Blender, desde el repositorio:
blender -b --python tools/blender/<franquicia>/prepare_<franquicia>_planet.py
La fuente aprobada se conserva geométricamente; las limpiezas son de exportación.
"""
from pathlib import Path
import bpy
import bmesh
import numpy as np
import math
import json
import hashlib
import struct
from mathutils import Vector, Matrix

PLANET = "regularshow"
REPO=Path(__file__).resolve().parents[3]
CONFIG={
 'starwars':dict(stem='starwars_death_star_planet',old='DS | CONTROL GENERAL',root='StarWars_Planet_ROOT',title='Planeta Estrella de la Muerte',scene='StarWars_Planet',diameter=20.0),
 'regularshow':dict(stem='regularshow_space_dome_planet',old='DOMO | CONTROL GENERAL',root='RegularShow_Planet_ROOT',title='Planeta Domo Espacial',scene='RegularShow_Planet',diameter=14.4),
}[PLANET]
MODEL=REPO/'assets/models/planets'/PLANET
TEXTURES=REPO/'assets/textures/planets'/PLANET
REFS=REPO/'docs/references'/PLANET
PRIMARY=MODEL/(CONFIG['stem']+'.blend')
EXPORT=MODEL/'exports'
for p in (MODEL,TEXTURES,REFS,EXPORT):p.mkdir(parents=True,exist_ok=True)
RUNTIME_SCALE=2.2  # South Park tiene diámetro base 44; se conserva la relación 20:14.4.

def enum(obj,name,value):
    values=[i.identifier for i in obj.bl_rna.properties[name].enum_items]
    if value in values:setattr(obj,name,next(i for i in values if i==value))
    elif not values:
        try:setattr(obj,name,value)
        except (TypeError,ValueError):pass

def mesh_stats(objects):
    count=verts=triangles=degenerate=0;points=[];mats=set()
    for ob in objects:
        if ob.type!='MESH':continue
        count+=1;me=ob.data;verts+=len(me.vertices);me.calc_loop_triangles();triangles+=len(me.loop_triangles)
        xyz=np.empty(len(me.vertices)*3,dtype=np.float64);me.vertices.foreach_get('co',xyz);xyz=xyz.reshape(-1,3)
        ids=np.empty(len(me.loop_triangles)*3,dtype=np.int32);me.loop_triangles.foreach_get('vertices',ids);ids=ids.reshape(-1,3)
        if len(ids):
            cross=np.cross(xyz[ids[:,1]]-xyz[ids[:,0]],xyz[ids[:,2]]-xyz[ids[:,0]])
            degenerate+=int(np.count_nonzero(np.einsum('ij,ij->i',cross,cross)<1e-20))
        mats.update(m.name for m in me.materials if m)
        points.extend(ob.matrix_world@Vector(p) for p in ob.bound_box)
    bounds=[[min(p[i] for p in points),max(p[i] for p in points)] for i in range(3)] if points else None
    return dict(mesh_objects=count,vertices=verts,triangles=triangles,zero_or_negligible_triangles=degenerate,materials=len(mats),bounds=bounds)

def geometry_signature(objects):
    h=hashlib.sha256()
    for ob in sorted(objects,key=lambda o:o.name):
        if ob.type!='MESH':continue
        me=ob.data;h.update(ob.name.encode());h.update(str([tuple(r) for r in ob.matrix_world]).encode())
        data=np.empty(len(me.vertices)*3,dtype=np.float32);me.vertices.foreach_get('co',data);h.update(data.tobytes())
        loops=np.empty(len(me.loops),dtype=np.int32);me.loops.foreach_get('vertex_index',loops);h.update(loops.tobytes())
        for layer in me.uv_layers:
            uv=np.empty(len(layer.data)*2,dtype=np.float32);layer.data.foreach_get('uv',uv);h.update(uv.tobytes())
        h.update(str([m.name if m else None for m in me.materials]).encode())
    return h.hexdigest()

def clean_mesh(me):
    bm=bmesh.new();bm.from_mesh(me)
    bmesh.ops.triangulate(bm,faces=list(bm.faces))
    before=len(bm.faces)
    # Suelda solo coincidencias a escala microscópica; conserva los UV por esquina.
    bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=1e-5)
    for face in list(bm.faces):
        if face.calc_area()<1e-10:bm.faces.remove(face)
    for edge in list(bm.edges):
        if not edge.link_faces:bm.edges.remove(edge)
    for vert in list(bm.verts):
        if not vert.link_faces:bm.verts.remove(vert)
    bm.to_mesh(me);bm.free();me.update()
    # La decimación puede dejar caras coincidentes después de soldar vértices.
    # Se valida la copia antes de medirla para que el exportador no la corrija
    # implícitamente y cambie los conteos al reimportar.
    me.validate(verbose=False,clean_customdata=True);me.update()
    removed=before-len(me.polygons)
    return removed

def configure_gpu(scene):
    try:
        scene.render.engine='CYCLES'
        prefs=bpy.context.preferences.addons['cycles'].preferences;prefs.get_devices()
        typ=next((d.type for d in prefs.devices if d.type=='OPTIX'),None)
        if typ:
            prefs.compute_device_type=typ;prefs.get_devices()
            for device in prefs.devices:device.use=device.type==typ
            enum(scene.cycles,'device','GPU')
        scene.cycles.samples=32;scene.cycles.use_denoising=True;scene.cycles.transparent_max_bounces=24
    except Exception as exc:print('GPU no disponible:',exc,flush=True)
    scene.render.resolution_x=1000;scene.render.resolution_y=1000;scene.render.resolution_percentage=100
    enum(scene.render.image_settings,'file_format','PNG')

def portable_materials(objects):
    materials={m for ob in objects for m in ob.data.materials if m}
    if PLANET=='starwars':
        # Los doce mapas de color ya incorporan el tinte del blindaje (sRGB).
        height=bpy.data.images.load(str(TEXTURES/'microrelieve.png'),check_existing=True)
        enum(height.colorspace_settings,'name','Non-Color')
        w,h=height.size;pixels=np.empty(w*h*4,dtype=np.float32);height.pixels.foreach_get(pixels)
        data=pixels.reshape(h,w,4)[:,:,0]
        dx=(np.roll(data,-1,axis=1)-np.roll(data,1,axis=1))*.5*.016/(2*math.pi*10/w)
        dy=(np.roll(data,-1,axis=0)-np.roll(data,1,axis=0))*.5*.016/(math.pi*10/h)
        normal=np.stack((-dx,-dy,np.ones_like(dx)),axis=2);normal/=np.linalg.norm(normal,axis=2)[:,:,None]
        rgba=np.ones((h,w,4),dtype=np.float32);rgba[:,:,:3]=normal*.5+.5
        normal_image=bpy.data.images.new('SW_Runtime_Normal',width=w,height=h,alpha=False)
        enum(normal_image.colorspace_settings,'name','Non-Color')
        normal_image.pixels.foreach_set(rgba.ravel());normal_image.filepath_raw=str(TEXTURES/'blindaje_normal.png')
        enum(normal_image,'file_format','PNG');normal_image.scale(2048,1024);normal_image.save()
        for m in materials:
            if 'Blindaje ' not in m.name:continue
            index=int(m.name.rsplit(' ',1)[1]);nt=m.node_tree;nt.nodes.clear()
            p=nt.nodes.new('ShaderNodeBsdfPrincipled');out=nt.nodes.new('ShaderNodeOutputMaterial');nt.links.new(p.outputs[0],out.inputs['Surface'])
            p.inputs['Metallic'].default_value=.42;p.inputs['Roughness'].default_value=.66
            image=bpy.data.images.load(str(TEXTURES/f'blindaje_{index:02d}.png'),check_existing=True)
            if max(image.size)>2048:
                image.scale(2048,1024);image.save()
            tex=nt.nodes.new('ShaderNodeTexImage');tex.image=image;nt.links.new(tex.outputs['Color'],p.inputs['Base Color'])
            tex=nt.nodes.new('ShaderNodeTexImage');tex.image=normal_image
            n=nt.nodes.new('ShaderNodeNormalMap');nt.links.new(tex.outputs['Color'],n.inputs['Color']);nt.links.new(n.outputs['Normal'],p.inputs['Normal'])
    else:
        for m in materials:
            if 'Vidrio transparente' not in m.name and 'Reticula fina' not in m.name:continue
            glass='Vidrio transparente' in m.name;nt=m.node_tree;nt.nodes.clear()
            p=nt.nodes.new('ShaderNodeBsdfPrincipled');out=nt.nodes.new('ShaderNodeOutputMaterial');nt.links.new(p.outputs[0],out.inputs['Surface'])
            p.inputs['Base Color'].default_value=(.53,.77,.78,1) if glass else (.15,.28,.28,1)
            p.inputs['Alpha'].default_value=.09 if glass else .5
            p.inputs['Roughness'].default_value=.12 if glass else .65
            p.inputs['IOR'].default_value=1.1
            enum(m,'surface_render_method','DITHERED');m.use_backface_culling=False
    return materials

def select_planet(root):
    for ob in list(bpy.context.selected_objects):ob.select_set(False)
    root.select_set(True)
    for ob in root.children_recursive:
        if ob.type=='MESH':ob.select_set(True)
    bpy.context.view_layer.objects.active=root

def glb_export(root,path):
    select_planet(root)
    props=bpy.ops.export_scene.gltf.get_rna_type().properties
    values=dict(filepath=str(path),use_selection=True,export_cameras=False,export_lights=False,export_animations=False,export_extras=True,export_apply=True,export_yup=True)
    # El exportador glTF usa items dinámicos y puede devolver una lista RNA vacía
    # en background. El propio operador valida el identificador en ese caso.
    values['export_format']=next((i.identifier for i in props['export_format'].enum_items if i.identifier=='GLB'),'GLB')
    bpy.ops.export_scene.gltf(**{k:v for k,v in values.items() if k in props})

def obj_export(root,path):
    select_planet(root);props=bpy.ops.wm.obj_export.get_rna_type().properties
    def value(name,wanted):return next(i.identifier for i in props[name].enum_items if i.identifier==wanted)
    bpy.ops.wm.obj_export(filepath=str(path),export_selected_objects=True,export_uv=True,export_normals=True,export_materials=True,export_object_groups=True,export_triangulated_mesh=True,forward_axis=value('forward_axis','NEGATIVE_Z'),up_axis=value('up_axis','Y'),path_mode=value('path_mode','RELATIVE'))
    if PLANET=='starwars':
        # MTL utiliza altura; GLB utiliza la normal tangente calculada más arriba.
        import re
        file=path.with_suffix('.mtl');text=file.read_text(encoding='utf-8')
        rel=Path(__import__('os').path.relpath(TEXTURES/'microrelieve.png',path.parent)).as_posix()
        chunks=re.split(r'(?=^newmtl )',text,flags=re.MULTILINE)
        for i,chunk in enumerate(chunks):
            if chunk.startswith('newmtl ') and 'Blindaje_' in chunk.splitlines()[0]:
                chunk=re.sub(r'^(map_Bump|bump|norm) .*\n?','',chunk,flags=re.MULTILINE)
                chunks[i]=chunk.rstrip()+f'\nmap_Bump -bm 0.016 {rel}\n\n'
        file.write_text(''.join(chunks),encoding='utf-8',newline='\n')

def render_preview(scene,name):
    configure_gpu(scene);scene.render.filepath=str(REFS/name)
    bpy.ops.render.render(write_still=True)

def low_dome_meshes(objects):
    # Sustituye vidrio y retícula por equivalentes de baja resolución, conservando
    # el hemisferio, el color y los materiales de transparencia.
    for ob in objects:
        if 'Hemisferio de vidrio' in ob.name:
            segments=64;rows=20;v=[];f=[]
            for j in range(rows):
                lat=math.pi/2*j/rows
                for i in range(segments):
                    a=2*math.pi*i/segments;v.append((7.2*math.cos(lat)*math.cos(a)*RUNTIME_SCALE,7.2*math.cos(lat)*math.sin(a)*RUNTIME_SCALE,(.12+6.55*math.sin(lat))*RUNTIME_SCALE))
            for j in range(rows-1):
                for i in range(segments):f.append((j*segments+i,j*segments+(i+1)%segments,(j+1)*segments+(i+1)%segments,(j+1)*segments+i))
            top=len(v);v.append((0,0,6.67*RUNTIME_SCALE))
            for i in range(segments):f.append(((rows-1)*segments+i,(rows-1)*segments+(i+1)%segments,top))
            old=ob.data;me=bpy.data.meshes.new(ob.name+' LOD');me.from_pydata(v,[],f);me.update()
            for m in old.materials:me.materials.append(m)
            for p in me.polygons:p.use_smooth=True
            ob.data=me
        elif 'Meridianos y paralelos' in ob.name:
            import importlib.util
            spec=importlib.util.spec_from_file_location('domo_lod_helpers',Path(__file__).parent/'crear_domo.py')
            D=importlib.util.module_from_spec(spec);spec.loader.exec_module(D);mesh=D.Mesh()
            for i in range(16):
                a=2*math.pi*i/16
                D.tube(mesh,[((7.207)*math.cos(math.pi/2*j/40)*math.cos(a),(7.207)*math.cos(math.pi/2*j/40)*math.sin(a),.12+6.557*math.sin(math.pi/2*j/40)) for j in range(41)],.007,0,5)
            for j in range(1,9):
                lat=math.pi/2*j/9;D.ring(mesh,7.207*math.cos(lat),.12+6.557*math.sin(lat),.007,0,steps=96)
            old=ob.data;me=bpy.data.meshes.new(ob.name+' LOD');me.from_pydata([tuple(Vector(v)*RUNTIME_SCALE) for v in mesh.v],[],mesh.f);me.update()
            for m in old.materials:me.materials.append(m)
            for p in me.polygons:p.use_smooth=True
            ob.data=me

def create_lod(objects):
    if PLANET=='starwars':
        # A distancia las placas se leen mejor como textura sobre una superficie
        # suave que como miles de pequeños sólidos decimados irregularmente.
        hull=next(o for o in objects if 'Estructura esferica' in o.name)
        surface=next(m for o in objects for m in o.data.materials if m and 'Blindaje 06' in m.name)
        hull.data.materials.clear();hull.data.materials.append(surface)
        for face in hull.data.polygons:face.material_index=0
        for ob in list(objects):
            if 'Miles de placas' in ob.name or 'Bloques tecnicos y circuitos' in ob.name:
                objects.remove(ob);bpy.data.objects.remove(ob,do_unlink=True)
    if PLANET=='regularshow':low_dome_meshes(objects)
    for ob in objects:
        if PLANET=='regularshow' and ('Hemisferio de vidrio' in ob.name or 'Meridianos y paralelos' in ob.name):
            clean_mesh(ob.data);continue
        if PLANET=='regularshow' and (ob.name.startswith('CASA |') or any(x in ob.name for x in ('Casco rocoso','Grietas y divisiones','Cinturon superior'))):
            # La casa es el hito principal; la roca ya es de baja resolución.
            # Conservarlas evita alterar su forma por ahorrar pocos triángulos.
            continue
        n=len(ob.data.polygons)
        if n<160:continue
        ratio=.16 if PLANET=='starwars' else .20
        if 'Estructura esferica' in ob.name:ratio=.30
        if 'Reflector' in ob.name or 'Sectores radiales' in ob.name:ratio=.35
        if 'Trinchera' in ob.name:ratio=.28
        if 'Copas' in ob.name:ratio=.14
        if 'Claro de' in ob.name:ratio=.05
        modifier=ob.modifiers.new('LOD_Far','DECIMATE');enum(modifier,'decimate_type','COLLAPSE');modifier.ratio=ratio;modifier.use_collapse_triangulate=True
        bpy.context.view_layer.objects.active=ob
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        clean_mesh(ob.data)
    # Texturas lejanas de menor resolución, empaquetadas dentro del GLB.
    originals={}
    for ob in objects:
        for m in ob.data.materials:
            if not m or not m.use_nodes:continue
            for node in m.node_tree.nodes:
                if node.type=='TEX_IMAGE' and node.image and max(node.image.size)>1024:
                    original=node.image
                    if original.name not in originals:
                        small=original.copy();small.name=original.name+' LOD_Far';small.scale(1024,512);small.pack();originals[original.name]=small
                    node.image=originals[original.name]

def read_glb(path):
    data=path.read_bytes();magic,version,total=struct.unpack_from('<4sII',data,0)
    assert magic==b'glTF' and version==2 and total==len(data)
    size,kind=struct.unpack_from('<II',data,12);assert kind==0x4e4f534a
    doc=json.loads(data[20:20+size].decode('utf-8'))
    assert not any('uri' in b for b in doc.get('buffers',[]))
    assert not any('uri' in i and not i['uri'].startswith('data:') for i in doc.get('images',[]))
    if PLANET=='regularshow':
        glass=[m for m in doc['materials'] if 'Vidrio transparente' in m.get('name','')]
        assert glass and glass[0].get('alphaMode')=='BLEND',glass
    return dict(bytes=len(data),embedded_images=len(doc.get('images',[])),materials=len(doc.get('materials',[])),mesh_primitives=sum(len(m['primitives']) for m in doc.get('meshes',[])))

def validate_import(path,expected):
    for ob in list(bpy.data.objects):bpy.data.objects.remove(ob,do_unlink=True)
    bpy.ops.import_scene.gltf(filepath=str(path))
    imported=[ob for ob in bpy.context.scene.objects if ob.type=='MESH'];actual=mesh_stats(imported)
    error=max(abs(actual['bounds'][i][j]-expected['bounds'][i][j]) for i in range(3) for j in range(2))
    assert error<.002,(path,error)
    assert actual['triangles']==expected['triangles'],(actual,expected)
    assert actual['zero_or_negligible_triangles']==0,actual
    return dict(bounds_max_error=error,triangles=actual['triangles'],zero_triangles=actual['zero_or_negligible_triangles'])

def main():
    bpy.ops.wm.open_mainfile(filepath=str(PRIMARY));scene=bpy.context.scene
    root=bpy.data.objects.get(CONFIG['root']) or bpy.data.objects[CONFIG['old']]
    objects=[o for o in root.children_recursive if o.type=='MESH']
    before=geometry_signature(objects);source=mesh_stats(objects)
    root.name=CONFIG['root'];scene.name=CONFIG['scene']
    root['asset_role']='planet';root['franchise']=PLANET;root['title']=CONFIG['title'];root['runtime_scale']=RUNTIME_SCALE;root['source_base_diameter']=CONFIG['diameter']
    root['integration_note']='Planeta del sistema solar; no nave, minijuego ni estacion espacial compartida.'
    scene.render.filepath=bpy.path.relpath(str(REFS/(PLANET+'_authoring_preview.png')))
    for text in bpy.data.texts:
        if text.filepath:
            candidate=Path(__file__).parent/Path(text.filepath).name
            text.filepath=bpy.path.relpath(str(candidate)) if candidate.is_file() else ''
    for im in bpy.data.images:
        if im.packed_file and im.filepath and Path(bpy.path.abspath(im.filepath)).is_absolute():im.filepath='//packed/'+Path(im.filepath).name
    select_planet(root)
    bpy.ops.wm.save_as_mainfile(filepath=str(PRIMARY))
    assert before==geometry_signature(objects),'La fuente editable cambió geométricamente'
    print('FUENTE VERIFICADA',PLANET,source,flush=True)
    # Copias de datos para exportar: no se vuelve a guardar PRIMARY después.
    matrices={o:o.matrix_world.copy() for o in objects}
    root.matrix_world=Matrix.Identity(4)
    removed=0
    for ob in objects:
        ob.data=ob.data.copy();ob.data.transform(Matrix.Scale(RUNTIME_SCALE,4)@matrices[ob]);ob.matrix_parent_inverse=Matrix.Identity(4);ob.matrix_basis=Matrix.Identity(4)
        removed+=clean_mesh(ob.data)
    portable_materials(objects)
    # Mantiene el encuadre y la iluminación para las vistas de revisión.
    for ob in list(scene.objects):
        if ob not in objects and ob!=root:
            ob.matrix_world=Matrix.Scale(RUNTIME_SCALE,4)@ob.matrix_world
            if ob.type=='CAMERA' and ob.data.type=='ORTHO':ob.data.ortho_scale*=RUNTIME_SCALE
            if ob.type=='LIGHT':
                ob.data.energy*=RUNTIME_SCALE**2
                if hasattr(ob.data,'size'):ob.data.size*=RUNTIME_SCALE
    bpy.context.view_layer.update()
    near=mesh_stats(objects);assert near['zero_or_negligible_triangles']==0,near
    near_path=EXPORT/(CONFIG['stem']+'_near.glb')
    glb_export(root,near_path);obj_export(root,EXPORT/(CONFIG['stem']+'.obj'))
    render_preview(scene,PLANET+'_runtime_near.png')
    print('NEAR LISTO',near,flush=True)
    create_lod(objects);bpy.context.view_layer.update()
    far=mesh_stats(objects);assert far['triangles']<near['triangles'];assert far['zero_or_negligible_triangles']==0,far
    far_path=EXPORT/(CONFIG['stem']+'_far.glb');glb_export(root,far_path)
    render_preview(scene,PLANET+'_runtime_far.png')
    print('FAR LISTO',far,flush=True)
    report=dict(planet=PLANET,role='planet',source=source,source_geometry_sha256=before,source_geometry_preserved=True,runtime_scale=RUNTIME_SCALE,near=near,far=far,removed_triangles=removed,glb_near=read_glb(near_path),glb_far=read_glb(far_path))
    report['import_near']=validate_import(near_path,near);report['import_far']=validate_import(far_path,far)
    report['triangle_reduction_far']=round(1-far['triangles']/near['triangles'],4)
    report['validation_limit']='Reimportado en Blender; motor, FPS, colisiones y seleccion de LOD en el juego pendientes.'
    (REFS/(PLANET+'_integration_audit.json')).write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print('INTEGRACION COMPLETA',PLANET,json.dumps(report,ensure_ascii=False),flush=True)

if __name__=='__main__':main()
