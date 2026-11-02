"""Phase 4 derivative: separate only the 60 decorative ore cubes from the approved mine.

Preserves phase-3 source files. Adds niche backing and exact collider subtractions.
"""
from pathlib import Path
import sys,importlib.util,json,hashlib,math
sys.dont_write_bytecode=True
import bpy,bmesh
from mathutils import Vector

ROOT=Path(__file__).resolve().parents[3]
SOURCE=ROOT/'assets/models/minigames/minecraft/mine'
OUT=ROOT/'assets/models/minigames/minecraft/mining'
REPORT=ROOT/'docs/references/minecraft/phase4'
spec=importlib.util.spec_from_file_location('mine_tools',Path(__file__).with_name('build_underground_mine.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


def subtract(lo,hi,cutlo,cuthi):
    a=[max(lo[i],cutlo[i]) for i in range(3)];b=[min(hi[i],cuthi[i]) for i in range(3)]
    if any(a[i]>=b[i] for i in range(3)):return [(lo,hi)]
    pieces=[];left=list(lo);right=list(hi)
    for axis in range(3):
        if left[axis]<a[axis]:
            end=right[:];end[axis]=a[axis];pieces.append((left[:],end));left[axis]=a[axis]
        if right[axis]>b[axis]:
            start=left[:];start[axis]=b[axis];pieces.append((start,right[:]));right[axis]=b[axis]
    return pieces


def main():
    OUT.mkdir(parents=True,exist_ok=True);REPORT.mkdir(parents=True,exist_ok=True)
    data=json.loads((SOURCE/'minecraft_mine.scene.json').read_text())
    bpy.ops.wm.open_mainfile(filepath=str(SOURCE/'minecraft_mine.blend'))
    bpy.context.preferences.filepaths.save_version=0
    m.OUT=OUT
    m.GROUPS.update({name:bpy.data.collections['DRM_'+name] for name in ('Static','Interactive','Lava','Colliders','Markers','Presentation')})
    m.v.MATS.update({mat.name.removeprefix('DR_'):mat for mat in bpy.data.materials if mat.name.startswith('DR_')})
    kinds={'coal_ore':'coal','iron_ore':'iron','gold_ore':'gold','redstone_ore':'redstone','lapis_ore':'lapis'}
    found={};removed=0
    for obj in list(m.GROUPS['Static'].objects):
        if not obj.name.startswith('Shell_'):continue
        faces=[]
        for face in obj.data.polygons:
            material=obj.data.materials[face.material_index].name.removeprefix('DR_')
            if material not in kinds:continue
            center=sum((obj.data.vertices[i].co for i in face.vertices),Vector())/len(face.vertices)-face.normal*.5
            cell=tuple(math.floor(x+1e-5) for x in center)
            entry=found.setdefault(cell,dict(material=material,approaches=[]))
            assert entry['material']==material
            if abs(face.normal.z)<.01:
                air=Vector(cell)+face.normal
                entry['approaches'].append([air.x+.5,air.y+.5,0])
            faces.append(face.index)
        if faces:
            bm=bmesh.new();bm.from_mesh(obj.data);bm.faces.ensure_lookup_table()
            bmesh.ops.delete(bm,geom=[bm.faces[i] for i in faces],context='FACES_ONLY')
            loose=[v for v in bm.verts if not v.link_faces]
            if loose:bmesh.ops.delete(bm,geom=loose,context='VERTS')
            bm.to_mesh(obj.data);bm.free();obj.data.update();removed+=len(faces)
    assert len(found)==60,len(found)
    all_blocks={tuple(b['min']) for b in data['interactive_blocks']}|set(found)
    floor={(c['x'],c['y']) for c in data['floor_cells']}
    def can_stand(x,y):
        return not any(c['shape']=='aabb' and c['category'] not in ('diamond','breakable') and
            c['min'][2]<1.8 and c['max'][2]>0 and c['min'][0]-.35<x<c['max'][0]+.35 and
            c['min'][1]-.35<y<c['max'][1]+.35 for c in data['colliders'])
    air={(c['x'],c['y'],z) for c in data['floor_cells'] for z in range(c['ceiling'])}
    backing=m.v.Batch('Mining_Niche_Backing','static_backing')
    for index,(p,info) in enumerate(sorted(found.items()),1):
        x,y,z=p;material=info['material'];kind=kinds[material]
        ident='ORE_%s_%02d'%(kind.upper(),index)
        batch=m.v.Batch(ident,kind);batch.box(p,(x+1,y+1,z+1),material)
        obj=batch.finish(m.GROUPS['Interactive']);obj['interactive_id']=ident;obj['kind']=kind
        approach=None;occluded=[]
        for raw in info['approaches']:
            dx=math.floor(raw[0])-x;dy=math.floor(raw[1])-y
            for step in range(1,5):
                cell=(x+dx*step,y+dy*step)
                if cell in floor and can_stand(cell[0]+.5,cell[1]+.5):
                    approach=[cell[0]+.5,cell[1]+.5,0]
                    occluded=[(x+dx*s,y+dy*s) for s in range(1,step)];break
            if approach is not None:break
        assert approach is not None,('No walkable ore approach',p)
        rooms=data['graph']['nodes']
        room=min(rooms,key=lambda n:(n['center'][0]-approach[0])**2+(n['center'][1]-approach[1])**2)
        data['interactive_blocks'].append(dict(id=ident,object=ident,collider_id=ident,kind=kind,room=room['id'],sector=room['sector'],
            min=list(p),max=[x+1,y+1,z+1],approach=approach,occluded_cells=occluded,initial_visible=True,initial_solid=True,reset=dict(visible=True,solid=True),gate=None))
        for i,(dx,dy,dz) in enumerate(m.OFFSETS):
            n=(x+dx,y+dy,z+dz)
            if n in air or n in all_blocks:continue
            backing.box(n,(n[0]+1,n[1]+1,n[2]+1),'stone',visible=[i^1])
    backing.finish(m.GROUPS['Static'])
    for b in data['interactive_blocks']:
        cells={tuple(p) for p in b.pop('occluded_cells',[])}
        b['occluded_by']=[other['id'] for other in data['interactive_blocks'] if tuple(other['min'][:2]) in cells]
    # Remove precisely the new ore volumes from structural boxes, preserving all other collision space.
    colliders=[]
    for c in data['colliders']:
        if c['shape']!='aabb':colliders.append(c);continue
        parts=[(c['min'],c['max'])]
        for x,y,z in found:
            parts=[part for lo,hi in parts for part in subtract(lo,hi,[x,y,z],[x+1,y+1,z+1])]
        for i,(lo,hi) in enumerate(parts):
            item=c.copy();item.update(min=lo,max=hi)
            if len(parts)!=1 or lo!=c['min'] or hi!=c['max']:item['name']=c['name']+'_piece_%03d'%i
            colliders.append(item)
    for b in data['interactive_blocks'][26:]:
        colliders.append(dict(name=b['collider_id'],shape='aabb',role='solid',category=b['kind'],min=b['min'],max=b['max']))
    data['colliders']=colliders
    for obj in list(m.GROUPS['Colliders'].objects):bpy.data.objects.remove(obj,do_unlink=True)
    for c in colliders:
        if c['shape']!='aabb':continue
        b=m.v.Batch('COL_'+c['name'],c['category']);b.box(c['min'],c['max'],'stone')
        obj=b.finish(m.GROUPS['Colliders']);obj.display_type='WIRE';obj.hide_render=True;obj.hide_set(True)
    data['phase']=4;data['counts']['extractable_blocks']=len(data['interactive_blocks']);data['counts']['common_ores']=60
    data['source']=str((SOURCE/'minecraft_mine.blend').relative_to(ROOT)).replace('\\','/')
    data['source_sha256']=hashlib.sha256((SOURCE/'minecraft_mine.blend').read_bytes()).hexdigest()
    data['extractable_common_ores']=data.pop('decorative_ores')
    data['notes']=['Phase-4 derivative; original phase-3 resources untouched.',
        'Only designated ore and gate blocks are removable. Structural floors, ceilings, supports and boundaries remain solid.',
        'Common ore faces separated in place; stone backing added behind the new niches.']
    bpy.context.scene.name='Minecraft_Mining_Ready'
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'minecraft_mine.blend'))
    m.export_glb()
    (OUT/'minecraft_mine.scene.json').write_text(json.dumps(data,indent=2),encoding='utf8')
    report=dict(status='passed',common_ores=len(found),removed_static_faces=removed,extractable_blocks=len(data['interactive_blocks']),
                solid_colliders=sum(c['role']=='solid' for c in colliders),source_sha256=data['source_sha256'])
    (REPORT/'preparation.json').write_text(json.dumps(report,indent=2),encoding='utf8');print(report)


if __name__=='__main__':main()
