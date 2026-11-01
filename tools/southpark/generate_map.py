"""Derived map from the phase-1 geometry manifest; never edits models/images.
python tools/southpark/generate_map.py [--check]
Uses the existing Pillow tooling, not a game/runtime dependency.
"""
import hashlib
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[2]
SOURCE=ROOT/'assets/models/minigames/southpark/southpark_town.scene.json'
OUT=ROOT/'assets/textures/minigames/South Park'
metadata_path=OUT/'southpark_map.json'
digest=hashlib.sha256(SOURCE.read_bytes()).hexdigest()
if '--check' in sys.argv:
    metadata=json.loads(metadata_path.read_text(encoding='utf-8'))
    assert metadata['source_sha256']==digest, 'Manifest changed; regenerate the map'
    assert metadata['image_sha256']==hashlib.sha256((OUT/'southpark_map.png').read_bytes()).hexdigest(), 'Map image changed'
    print('MAP_SOURCE_CHECK PASS')
    raise SystemExit(0)
data=json.loads(SOURCE.read_text(encoding='utf-8'))
ground=next(c for c in data['colliders'] if c['role']=='walkable')
low=[ground['min'][0],-ground['max'][1]]
high=[ground['max'][0],-ground['min'][1]]
size=(1024,512)
def point(x,z):
    return ((x-low[0])/(high[0]-low[0])*(size[0]-1),
            (z-low[1])/(high[1]-low[1])*(size[1]-1))
image=Image.new('RGB',size,(224,235,234))
d=ImageDraw.Draw(image)
for asset in data['assets']:
    a,b=asset['bounds']
    p,q=point(a[0],-b[1]),point(b[0],-a[1])
    category=asset['category']
    if category=='Mountains':
        mx,my=(p[0]+q[0])/2,(p[1]+q[1])/2
        d.polygon([(mx,p[1]),(q[0],my),(mx,q[1]),(p[0],my)],fill=(159,179,181),outline=(129,154,159))
    elif category=='Buildings':
        palette=[(184,150,123),(170,164,126),(139,171,174),(169,145,156)]
        color=palette[sum(asset['source'].encode())%len(palette)]
        d.rectangle([p,q],fill=color,outline=(92,112,121),width=1)
    elif category=='Trees':
        d.ellipse([p,q],fill=(112,152,130),outline=(83,128,110))
    else:
        d.rectangle([p,q],fill=(192,155,96))
# Road surfaces are the manifest's actual polyline coordinates, six metres wide.
road_width=round(6/(high[0]-low[0])*size[0])
for road in data['roads']:
    points=[point(p[0],-p[1]) for p in road['points']]
    d.line(points,fill=(98,118,132),width=road_width+2,joint='curve')
    d.line(points,fill=(249,247,227),width=road_width,joint='curve')
d.rectangle((0,0,size[0]-1,size[1]-1),outline=(70,99,114),width=3)
image.save(OUT/'southpark_map.png',optimize=True)
metadata={'schema_version':1,'world_min':low,'world_max':high,'image_size':list(size),
          'source':str(SOURCE.relative_to(ROOT)),'source_sha256':digest,
          'image_sha256':hashlib.sha256((OUT/'southpark_map.png').read_bytes()).hexdigest(),
          'roads':len(data['roads']),'footprints':len(data['assets']),
          'axes':'world X right, world Z down; north = -Z; authoring (x,y,z) -> (x,z,-y)'}
metadata_path.write_text(json.dumps(metadata,indent=2),encoding='utf-8')
print('MAP_GENERATED',json.dumps(metadata))
