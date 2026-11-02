"""Phase 5 source of truth. Expanded finite mine, using existing geometry/ore preparation.
Only new rush derivatives are written; earlier Blender files are read-only.
"""
from pathlib import Path
import sys,importlib.util,tempfile,json,math,hashlib
sys.dont_write_bytecode=True
import bpy
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'assets/models/minigames/minecraft/rush'
REPORT=ROOT/'docs/references/minecraft/phase5'
def module(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def main():
    m=module('rush_geometry','build_underground_mine.py');p=module('rush_ores','prepare_mining_blocks.py')
    baseline=json.loads((ROOT/'assets/models/minigames/minecraft/mining/minecraft_mine.scene.json').read_text())
    m.ROOMS.extend([('D_cavern','D',[10,63,30,83],9),('D_foundry','D',[37,47,51,61],6),
                   ('D_quartz','D',[36,70,46,81],5),('D_gallery','D',[-4,68,7,80],5),('E_grotto','E',[-44,43,-31,55],5)])
    m.TUNNELS.extend([
        ('C_foundry','C_vault','D_foundry',[[24,51,37,55]],[[20,53],[39,53],[39,49]],4,None),
        ('C_cavern','C_vault','D_cavern',[[18,57,23,63]],[[20,55],[20,67]],5,None),
        ('C_gallery','C_lava','D_gallery',[[0,58,4,68]],[[2,56],[2,73]],4,None),
        ('D_gallery_cavern','D_gallery','D_cavern',[[7,72,10,76]],[[3,75],[13,75]],4,None),
        ('D_quartz_link','D_cavern','D_quartz',[[30,73,36,77]],[[28,75],[40,75]],4,None),
        ('D_east_loop','D_foundry','D_quartz',[[41,61,45,70]],[[43,59],[43,73]],4,None),
        ('D_foundry_loop','D_cavern','D_foundry',[[28,63,41,67],[37,61,41,65]],[[27,65],[39,65],[39,59]],4,None),
        ('E_hidden_reward','C_secret','E_grotto',[[-31,49,-22,52]],[[-18,50.5],[-38,50.5]],3,None)])
    original_rect=m.rect_cells
    def cells(rect):
        result=original_rect(rect)
        if rect==[10,63,30,83]:result|=original_rect([7,68,10,78])|original_rect([30,68,34,78])
        return result
    m.rect_cells=cells
    m.DIAMONDS=[('A_junction',(-4,21,1)),('A_supply',(-20,16,1)),('B_hub',(-6,30,1)),('B_east',(22,28,1)),
        ('B_secret',(-22,31,1)),('C_lava',(-7,47,1)),('C_lava',(-3,58,1)),('C_vault',(24,48,1)),('C_secret',(-18,55,1)),
        ('D_cavern',(9,66,1)),('D_cavern',(16,83,1)),('D_cavern',(30,80,1)),('D_foundry',(51,50,1)),
        ('D_foundry',(48,61,1)),('D_quartz',(46,75,1)),('E_grotto',(-45,47,1))]
    m.ORE_SPEC=[('coal_ore',14,-3,83),('iron_ore',14,6,83),('gold_ore',10,28,83),('redstone_ore',12,28,83),('lapis_ore',10,26,83)]
    original_decor=m.decorations
    def decor(height):
        regions=original_decor(height)
        b=m.v.Batch('Rush_Cavern_Formations','structural_formation')
        for index,(x,y,w,h) in enumerate([(14,70,2,5),(26,77,2,9),(11,77,2,3)]):
            for xx in range(x,x+w):
                for yy in range(y,y+w):
                    for z in range(h):
                        faces=[i for i,(dx,dy,dz) in enumerate(m.OFFSETS) if not(x<=xx+dx<x+w and y<=yy+dy<y+w and 0<=z+dz<h)]
                        b.box((xx,yy,z),(xx+1,yy+1,z+1),'andesite',visible=faces)
            m.collider('Rush_Pillar_%s'%index,(x,y,0),(x+w,y+w,h),'formation')
        for x,y in [(18,67),(23,79)]:b.box((x,y,6),(x+2,y+2,9),'stone');m.collider('Rush_Stalactite_%s'%x,(x,y,6),(x+2,y+2,9),'formation')
        # Timber gallery and a diorite landmark distinguish the new routes.
        for y in (69,74,78):
            for x in (-2.5,5.5):b.box((x,y,0),(x+.4,y+.4,4),'oak_log',top='oak_log_top');m.collider('Rush_Timber_%s_%s'%(x,y),(x,y,0),(x+.4,y+.4,4),'support')
            b.box((-2.5,y,4),(5.9,y+.4,4.5),'oak_planks')
        b.box((37,78,0),(39,80,2),'diorite');m.collider('Rush_Quartz_Landmark',(37,78,0),(39,80,2),'formation')
        b.finish(m.GROUPS['Static'])
        for ident,rect,safe in [('Lava_Cavern',[20,72,25,76],[17,68,0]),('Lava_Foundry',[42,52,46,55],[39,49,0])]:
            x,y,xx,yy=rect;b=m.v.Batch(ident,'lava');b.box((x,y,-.08),(xx,yy,.025),'lava_still');b.finish(m.GROUPS['Lava'])
            regions.append(dict(id=ident,min=[x,y,-.1],max=[xx,yy,.5],damage_enabled=True,safe_return=safe))
            m.LIGHTS.append(dict(id=ident,position=[(x+xx)/2,(y+yy)/2,.7],color=[1,.22,.035],radius=11))
        for r in regions:
            r['damage_enabled']=True
            if r['id']=='Lava_West':r['safe_return']=[0,48,0]
            if r['id']=='Lava_Vault':r['safe_return']=[17,47,0]
        lamps=[(12,65,3),(29,65,3),(10,77,3),(28,81,3),(19,81,3),(33,75,3),
               (38,48,3),(49,58,3),(43,65,3),(39,72,3),(44,79,3),(1,70,3),(3,78,3),(-28,50,2.5),(-40,46,3)]
        b=m.v.Batch('Rush_Extra_Lanterns','light_fixture')
        for i,(x,y,z) in enumerate(lamps):
            b.box((x-.12,y-.12,z),(x+.12,y+.12,z+.25),'torch_flame');b.box((x-.035,y-.035,z+.25),(x+.035,y+.035,height[math.floor(x),math.floor(y)]),'metal')
            m.LIGHTS.append(dict(id='Rush_Torch_%s'%i,position=[x,y,z],color=[.65,.84,1] if x<7 or x>35 and y>65 else [1,.66,.32],radius=10))
        b.finish(m.GROUPS['Static'])
        for title,pos in [('GRAN CAVERNA',(20,82.95,5)),('FUNDICION',(47,60.95,3.5)),('CANTERA CLARA',(41,80.95,3)),('GALERIA DE VETAS',(1,79.95,3)),('GRUTA OCULTA',(-37,54.95,3))]:
            curve=bpy.data.curves.new(title,'FONT');curve.body=title;curve.align_x='CENTER';curve.size=.4;curve.extrude=.004
            obj=bpy.data.objects.new('Sign_'+title,curve);m.GROUPS['Static'].objects.link(obj);obj.location=pos;obj.rotation_euler=(math.pi/2,0,0);curve.materials.append(m.v.MATS['sign_green'])
            bpy.context.view_layer.objects.active=obj;obj.select_set(True);bpy.ops.object.convert(target='MESH');obj.select_set(False)
            for f in obj.data.polygons:f.use_smooth=False
        return regions
    m.decorations=decor
    REPORT.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='diamond_rush_build_') as temp:
        m.OUT=Path(temp)/'base';m.REPORT=Path(temp)/'review'
        sys.argv.append('--skip-renders');m.main()
        p.SOURCE=m.OUT;p.OUT=OUT;p.REPORT=REPORT;p.SOURCE_LABEL='generated by tools/blender/minecraft/build_rush_mine.py';p.main()
    data=json.loads((OUT/'minecraft_mine.scene.json').read_text());data['phase']=5
    for node in data['graph']['nodes']:
        if node['id']=='D_cavern':node['center']=[17,68,0]
        if node['id']=='D_foundry':node['center']=[39,49,0]
    data['notes']=['Expanded phase-5 mine generated from layout definitions. Previous assets retained.', 'Flat FPS floor; raised cubic formations and ceilings up to 9 metres.']
    data['comparison']=dict(previous_floor_cells=len(baseline['floor_cells']),floor_cells=len(data['floor_cells']),ratio=len(data['floor_cells'])/len(baseline['floor_cells']),previous_rooms=9,rooms=14,previous_tunnels=10,tunnels=18)
    data['counts'].update(chambers=14,tunnels=18,diamonds=16,lava_regions=4,extractable_blocks=len(data['interactive_blocks']))
    (OUT/'minecraft_mine.scene.json').write_text(json.dumps(data,indent=2),encoding='utf8')
    # Saved camera is FPS; technical overview hides ceilings only during rendering.
    scene=bpy.context.scene;scene.name='Diamond_Rush_Expanded';scene.cycles.samples=16
    bpy.data.objects['Sign_SALIDA / OVERWORLD'].rotation_euler=(math.pi/2,0,math.pi)
    p.m.export_glb()
    cam=bpy.data.cameras.new('Rush_Cavern');obj=bpy.data.objects.new(cam.name,cam);scene.collection.objects.link(obj);obj.location=(11,65,1.7);cam.lens=20
    obj.rotation_euler=(Vector((24,78,5))-obj.location).to_track_quat('-Z','Y').to_euler()
    scene.camera=obj;bpy.context.preferences.filepaths.save_version=0;bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'minecraft_mine.blend'))
    scene.render.filepath=str(REPORT/'blender_cavern.png');bpy.ops.render.render(write_still=True)
    def overview(filename):
        s=bpy.context.scene;cam=bpy.data.cameras.new('Comparison_Top');cam.type='ORTHO';cam.ortho_scale=160
        o=bpy.data.objects.new(cam.name,cam);s.collection.objects.link(o);o.location=(5,40,140);o.rotation_euler=(0,0,0);s.camera=o
        for o in bpy.data.objects:
            if o.name.startswith('Shell_') and o.name.endswith('Ceiling'):o.hide_render=True
        light=bpy.data.lights.new('Technical_Sun','SUN');light.energy=1.4;o=bpy.data.objects.new(light.name,light);s.collection.objects.link(o)
        s.cycles.samples=12;s.render.filepath=str(REPORT/filename);bpy.ops.render.render(write_still=True)
    overview('expanded_overview.png')
    bpy.ops.wm.open_mainfile(filepath=str(ROOT/'assets/models/minigames/minecraft/mining/minecraft_mine.blend'))
    overview('previous_overview.png')
    (REPORT/'expansion.json').write_text(json.dumps(dict(status='generated',**data['comparison'],diamonds=16,lights=len(data['lights'])),indent=2),encoding='utf8')
    print('EXPANSION',data['comparison'])

if __name__=='__main__':main()
