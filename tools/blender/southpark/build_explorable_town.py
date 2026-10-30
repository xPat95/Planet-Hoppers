"""Build the phase-1 town from the approved planet. Run with Blender --background --python.

Only writes derived assets; never opens/saves the source as the working file.
No third-party modules, game engine, physics or player controller required.
"""
import ast
import hashlib
import json
import math
from pathlib import Path

import bpy
import bmesh
from mathutils import Matrix, Vector

REPO = Path(__file__).resolve().parents[3]
SOURCE = REPO / 'assets/models/planets/southpark/southpark_planet.blend'
OUT = REPO / 'assets/models/minigames/southpark'
REPORT = REPO / 'docs/references/southpark/phase1'
SCALE = 1.75  # Existing 0.66 * 1.90 residential door becomes about 2.19 m.
MAP_X, MAP_Y = .36, .55
ROAD_WIDTH = 6.0
EYE_HEIGHT = 1.7


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def map_point(x, y):
    return Vector(((x - 800) * MAP_X, (280 - y) * MAP_Y, 0))


def frame_at(x, y):
    lon, lat = (x - 800) / 1600 * math.tau, math.radians((280 - y) * .35)
    east = Vector((math.cos(lon), math.sin(lon), 0))
    north = Vector((-math.sin(lat)*math.sin(lon), math.sin(lat)*math.cos(lon), math.cos(lat)))
    normal = Vector((math.cos(lat)*math.sin(lon), -math.cos(lat)*math.cos(lon), math.sin(lat)))
    return Matrix((east, north, normal)).transposed().to_4x4()


def ground_z(frame, x, y):
    # Exact inverse of the v2/v3 foundation projection, including the radial relief.
    origin, axes = frame.translation, frame.to_3x3()
    point = origin + axes @ Vector((x, y, 0))
    for _ in range(2):
        n = point.normalized()
        lon, lat = math.atan2(n.x, -n.y), math.asin(n.z)
        radius = 22 + .10*math.sin(3*lon)*math.cos(lat)**2 + .08*math.sin(lat*3)*math.cos(lon*4)
        z = math.sqrt(max(0, radius*radius-x*x-y*y)) - origin.length
        point = origin + axes @ Vector((x, y, z))
    return z


def collection(name, parent):
    result = bpy.data.collections.new(name)
    parent.children.link(result)
    return result


def bounds(objects):
    points = [o.matrix_world @ Vector(v) for o in objects if o.type == 'MESH' for v in o.bound_box]
    return [min(p[i] for p in points) for i in range(3)], [max(p[i] for p in points) for i in range(3)]


def distance_segment(p, a, b):
    d = b-a
    return (p-(a+d*max(0, min(1, (p-a).dot(d)/d.length_squared)))).length


def box(name, low, high, group, material=None):
    vertices = [(x, y, z) for z in (low[2], high[2]) for y in (low[1], high[1]) for x in (low[0], high[0])]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], [(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)])
    if material:
        mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    group.objects.link(obj)
    return obj


def paths_from_history():
    # Read literal route data only. Never run the historical model generators.
    result = []
    for filename, variable in [('build_southpark_planet.py', 'paths'), ('refine_southpark_planet_v02.py', 'new_roads')]:
        tree = ast.parse(Path(__file__).with_name(filename).read_text(encoding='utf-8'))
        node = next(n for n in tree.body if isinstance(n, ast.Assign) and any(isinstance(t, ast.Name) and t.id == variable for t in n.targets))
        result.extend(ast.literal_eval(node.value))
    return [(name, [map_point(*p) for p in points]) for name, points in result]


def export_static_glb(path):
    # Same local workaround already used by the Butters exporter: this alpha
    # omits a shape-key module. No installation changes, animation stays disabled.
    import sys
    import types
    import io_scene_gltf2
    missing = Path(io_scene_gltf2.__file__).parent / 'blender/exp/animation/sampled/shapekeys/channel_target.py'
    if not missing.exists():
        stub = types.ModuleType('io_scene_gltf2.blender.exp.animation.animations')
        def no_animation(*args, **kwargs):
            raise RuntimeError('This Blender build only supports static export here.')
        stub.gather_animations = no_animation
        sys.modules[stub.__name__] = stub
    bpy.ops.export_scene.gltf(filepath=str(path),export_format='GLB',use_selection=True,export_extras=True,export_yup=True,export_animations=False)


def main():
    source_hash = digest(SOURCE)
    OUT.mkdir(parents=True, exist_ok=True)
    REPORT.mkdir(parents=True, exist_ok=True)
    scene = bpy.data.scenes.new('SouthPark_Explorable')
    bpy.context.window.scene = scene
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1
    visuals = collection('SPT_Visuals', scene.collection)
    groups = {name: collection('SPT_'+name, visuals) for name in ['Buildings','Trees','Mountains','Decoration','Ground','Roads']}
    colliders = collection('SPT_Colliders', scene.collection)
    colliders.hide_render = True
    dev = collection('SPT_Development', scene.collection)
    with bpy.data.libraries.load(str(SOURCE), link=False) as (available, loaded):
        loaded.objects = available.objects
    originals = {o.name:o for o in loaded.objects}
    imported = collection('SPT_Temporary_Source', scene.collection)
    for obj in originals.values():
        imported.objects.link(obj)
    # Parent matrices are only reliable after the imported hierarchy is evaluated.
    bpy.context.view_layer.update()
    planet = originals['SouthPark_Planet_ROOT']
    buildings = sorted([o for o in planet.children_recursive if o.get('building_id')], key=lambda o:o.name)
    anchors = buildings + sorted([o for o in planet.children if 'reference_xy' in o and not o.get('building_id')], key=lambda o:o.name)
    assert len(buildings) == 160
    paths = paths_from_history()
    segments = [(a.xy,b.xy) for _, ps in paths for a,b in zip(ps, ps[1:])]
    occupied = []
    provenance, relocations, collision_data = [], [], []
    exported = []

    def road_clear(low, high, margin=.6):
        # Exact segment vs expanded axis-aligned footprint (conservative at corners).
        for a,b in segments:
            t0,t1 = 0.,1.
            for axis in range(2):
                lo,hi = low[axis]-ROAD_WIDTH/2-margin,high[axis]+ROAD_WIDTH/2+margin
                delta = b[axis]-a[axis]
                if abs(delta)<1e-9:
                    if not lo <= a[axis] <= hi:
                        break
                else:
                    u,v = sorted(((lo-a[axis])/delta,(hi-a[axis])/delta))
                    t0,t1 = max(t0,u),min(t1,v)
                    if t0>t1:
                        break
            else:
                if t0<=t1:
                    return False
        return True

    def free(low, high, margin=1.5):
        return road_clear(low, high) and all(any(high[i]+margin < a[i] or low[i]-margin > b[i] for i in range(2)) for a,b in occupied)

    for source in anchors:
        x,y = source['reference_xy']
        radial_frame = frame_at(x,y)
        basis = radial_frame.to_3x3().inverted() @ source.matrix_world.to_3x3()
        matrix = basis.to_4x4()
        matrix.translation = map_point(x,y)
        kind = 'Buildings' if source.get('building_id') else ('Trees' if source.name.startswith('Pine_') else 'Mountains' if source.name.startswith('Mountain_') else 'Decoration')
        asset_scale = SCALE * (2 if kind == 'Mountains' else .6 if kind == 'Decoration' else 1)
        matrix = matrix @ Matrix.Scale(asset_scale,4)
        members = [source] + list(source.children_recursive)
        copies = []
        for old in members:
            if old.type != 'MESH':
                continue
            mesh = old.data.copy()
            relative = source.matrix_world.inverted() @ old.matrix_world
            # Bake the complete assembly into the anchor's local frame so lettering,
            # storefronts and corner ornaments follow the same foundation correction.
            for vertex in mesh.vertices:
                p = relative @ vertex.co
                if kind == 'Buildings':
                    p.z -= ground_z(source.matrix_world,p.x,p.y)
                elif kind == 'Mountains':
                    p.z -= math.sqrt(max(0,22**2-p.x*p.x-p.y*p.y))-22
                vertex.co = p
            mesh.update()
            bm = bmesh.new()
            bm.from_mesh(mesh)
            bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
            bm.to_mesh(mesh)
            bm.free()
            obj = bpy.data.objects.new('SPT_'+old.name,mesh)
            groups[kind].objects.link(obj)
            obj.matrix_world = matrix.copy()
            obj['source_object'] = old.name
            obj['collision_role'] = 'solid' if kind == 'Buildings' else 'obstacle' if kind in ('Trees','Mountains') else 'decoration'
            copies.append(obj)
        bpy.context.view_layer.update()
        low,high = bounds(copies)
        # Sink only the tiny original foundation overlap, never the roof or facade.
        dz = -low[2] - .02
        for obj in copies:
            obj.location.z += dz
        bpy.context.view_layer.update()
        low,high = bounds(copies)
        if kind in ('Buildings','Trees','Decoration'):
            found = None
            # Deterministic local packing, retaining the map position whenever clear.
            offsets = [(0,0)]
            for radius in range(2,65,2):
                offsets.extend((radius*math.cos(a*math.tau/24),radius*math.sin(a*math.tau/24)) for a in range(24))
            for dx,dy in offsets:
                a,b = [*low],[*high]
                a[0]+=dx; b[0]+=dx; a[1]+=dy; b[1]+=dy
                if free(a,b,1.5 if kind=='Buildings' else .7):
                    found = (dx,dy,a,b)
                    break
            if found is None:
                raise RuntimeError('Cannot place existing asset safely: '+source.name)
            dx,dy,low,high = found
            for obj in copies:
                obj.location.x+=dx; obj.location.y+=dy
            if dx or dy:
                relocations.append({'source':source.name,'offset_m':[dx,dy]})
            occupied.append((low,high))
        if kind in ('Buildings','Trees','Mountains','Decoration'):
            if kind=='Trees':
                center = [(low[i]+high[i])/2 for i in range(3)]
                # Only the trunk blocks the future player; foliage remains decoration.
                a,b = [center[0]-.20,center[1]-.20,0],[center[0]+.20,center[1]+.20,high[2]]
            else:
                a,b = low,high
            proxy = box('COL_'+source.name,a,b,colliders)
            proxy.display_type = 'WIRE'
            proxy.hide_set(True)
            proxy['collision_role'] = 'solid'
            proxy['source_object'] = source.name
            collision_data.append({'name':proxy.name,'source':source.name,'shape':'aabb','min':a,'max':b,'role':'solid'})
        provenance.append({'source':source.name,'category':kind,'objects':[o.name for o in copies],'bounds':[low,high]})
        exported.extend(copies)

    # Only curved surfaces need new geometry. Architecture and its materials are reused.
    snow = bpy.data.materials['SP_Snow']
    terrain = box('SPT_Snow_Ground',[-350,-160,-1],[370,175,0],groups['Ground'],snow)
    terrain['collision_role'] = 'walkable'
    exported.append(terrain)
    collision_data.append({'name':terrain.name,'shape':'plane','z':0,'min':[-350,-160],'max':[370,175],'role':'walkable'})

    def ribbon(name, ps, width, z, material):
        verts,faces = [],[]
        for a,b in zip(ps,ps[1:]):
            side = Vector((-(b-a).y,(b-a).x,0)).normalized()*width/2
            n = len(verts)
            verts.extend([(p.x,p.y,z) for p in [a-side,b-side,b+side,a+side]])
            faces.append((n,n+1,n+2,n+3))
        mesh=bpy.data.meshes.new(name)
        mesh.from_pydata(verts,[],faces)
        mesh.materials.append(material)
        obj=bpy.data.objects.new(name,mesh)
        groups['Roads'].objects.link(obj)
        obj['collision_role']='walkable_overlay'
        exported.append(obj)

    for route_index,(name,ps) in enumerate(paths):
        # Tiny height offsets prevent coplanar z-fighting at road intersections.
        ribbon('SPT_Road_'+name,ps,ROAD_WIDTH,.012+route_index*.001,bpy.data.materials['SP_Road'])
        for a,b in zip(ps,ps[1:]):
            count=max(1,int((b-a).length/8))
            for j in range(count):
                ribbon('SPT_Marking_'+name,[a.lerp(b,(j+.18)/count),a.lerp(b,(j+.55)/count)],.12,.04+route_index*.0001,bpy.data.materials['SP_Marking'])

    # Restore the mall's flat parking lot using its original reference coordinates.
    parking=box('SPT_Mall_Parking',list(map_point(748,200)+Vector((0,0,.002))),list(map_point(952,170)+Vector((0,0,.009))),groups['Roads'],bpy.data.materials['SP_Road'])
    exported.append(parking)

    spawn=map_point(680,202)+Vector((0,0,EYE_HEIGHT))
    spawn_obj=bpy.data.objects.new('SPT_Player_Spawn',None)
    dev.objects.link(spawn_obj)
    spawn_obj.location=spawn
    spawn_obj['eye_height_m']=EYE_HEIGHT
    camera_data=bpy.data.cameras.new('SPT_Street_Camera')
    camera=bpy.data.objects.new('SPT_Street_Camera',camera_data)
    dev.objects.link(camera)
    camera.location=spawn
    camera.rotation_euler=Vector((1,0,0)).to_track_quat('-Z','Y').to_euler()
    camera_data.lens=24
    camera_data.clip_end=1400
    scene.camera=camera
    sun_data=bpy.data.lights.new('SPT_Winter_Sun','SUN')
    sun_data.energy=2.2
    sun_data.angle=math.radians(15)
    sun=bpy.data.objects.new('SPT_Winter_Sun',sun_data)
    dev.objects.link(sun)
    sun.rotation_euler=(math.radians(28),math.radians(-25),math.radians(-35))
    world=bpy.data.worlds.new('SPT_Winter_Sky')
    world.use_nodes=True
    background=next(n for n in world.node_tree.nodes if n.type=='BACKGROUND')
    background.inputs['Color'].default_value=(.62,.78,.94,1)
    background.inputs['Strength'].default_value=.8
    scene.world=world
    scene.render.engine='CYCLES'
    scene.cycles.samples=24
    scene.render.resolution_x=1280
    scene.render.resolution_y=800
    scene.render.resolution_percentage=100
    scene.view_settings.view_transform='Standard'
    bpy.context.view_layer.update()
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type=='VIEW_3D':
                space=area.spaces.active
                space.clip_end=1500
                space.shading.type='MATERIAL'
                space.region_3d.view_perspective='CAMERA'
    scene['phase']=1
    scene['source_sha256']=source_hash
    scene['units']='meters; Z up (glTF converts to Y up)'
    scene['development_navigation']='Camera view: Numpad 0, then Shift+grave (Walk); WASD, mouse, Q/E, Esc to cancel.'

    # Remove only temporary imported datablocks from this background working session.
    # The source .blend and the user's live Blender session are never written.
    for obj in originals.values():
        bpy.data.objects.remove(obj,do_unlink=True)
    bpy.data.collections.remove(imported)
    for old_scene in list(bpy.data.scenes):
        if old_scene != scene:
            bpy.data.scenes.remove(old_scene)
    assert digest(SOURCE)==source_hash
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'southpark_town.blend'))
    bpy.ops.object.select_all(action='DESELECT')
    for obj in exported:
        obj.select_set(True)
    bpy.context.view_layer.objects.active=terrain
    export_static_glb(OUT/'southpark_town.glb')
    metadata={
        'schema_version':1,'id':'southpark.save-kenny.environment','phase':1,
        'source':SOURCE.relative_to(REPO).as_posix(),'source_sha256':source_hash,
        'blend':'southpark_town.blend','visual_asset':'southpark_town.glb',
        'coordinates':{'units':'meters','up':'Z','gltf_up':'Y','to_gltf':'(x,y,z) -> (x,z,-y)'},
        'spawn':{'position':list(spawn),'forward':[1,0,0],'eye_height':EYE_HEIGHT},
        'bounds':{'min':[-350,-160,0],'max':[370,175,0]},
        'road_width':ROAD_WIDTH,
        'roads':[{'name':name,'points':[list(p) for p in ps]} for name,ps in paths],
        'colliders':collision_data,'assets':provenance,'relocations':relocations,
        'exit_contract':{'event':'requestReturn','planet':'southpark','result':None,'preserve_entry_context':True},
    }
    (OUT/'southpark_town.scene.json').write_text(json.dumps(metadata,indent=2,ensure_ascii=False),encoding='utf-8')
    # Render independent development views, then restore the saved street camera.
    street_location=camera.location.copy()
    street_rotation=camera.rotation_euler.copy()
    scene.render.filepath=str(REPORT/'street.png')
    bpy.ops.render.render(write_still=True)
    camera.location=(310,-360,340)
    camera.rotation_euler=(Vector((0,0,0))-camera.location).to_track_quat('-Z','Y').to_euler()
    camera_data.type='ORTHO'
    camera_data.ortho_scale=760
    scene.render.filepath=str(REPORT/'overview.png')
    bpy.ops.render.render(write_still=True)
    camera.location=street_location
    camera.rotation_euler=street_rotation
    camera_data.type='PERSP'
    assert digest(SOURCE)==source_hash
    print('TOWN_BUILD_OK',json.dumps({'assets':len(provenance),'buildings':len(buildings),'relocated':len(relocations),'source_sha256':source_hash}))


if __name__=='__main__':
    main()
