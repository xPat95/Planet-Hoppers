"""Actual raylib execution, debug captures and launch/return regression checks."""
from pathlib import Path
import json
import shutil
import subprocess

root=Path(__file__).resolve().parents[1]
out=root/'docs/references/station-diagnosis'
raw=root/'target/station-diagnosis-runtime'
out.mkdir(parents=True,exist_ok=True)
raw.mkdir(parents=True,exist_ok=True)
records=[]
runs=[('debug','solarsystem',['--smoke-station','--debug-collisions'],'STATION_SMOKE_PASS'),
      ('starwars','planet-hoppers',['--smoke','--ship','starwars'],'LAUNCH_SMOKE_PASS'),
      ('smilingfriends','planet-hoppers',['--smoke','--ship','smilingfriends'],'LAUNCH_SMOKE_PASS'),
      ('return-starwars','planet-hoppers',['--smoke-docking','--ship','starwars'],'DOCKING_SMOKE_PASS cycles=3'),
      ('return-smilingfriends','planet-hoppers',['--smoke-docking','--ship','smilingfriends'],'DOCKING_SMOKE_PASS cycles=3')]
for label,exe,args,marker in runs:
    with (raw/f'{label}.log').open('wb') as log:
        result=subprocess.run([str(root/f'target/debug/{exe}.exe'),*args],cwd=root,stdout=log,stderr=subprocess.STDOUT,timeout=240)
    text=(raw/f'{label}.log').read_text(encoding='utf-8',errors='replace')
    selected=[line for line in text.splitlines() if any(k in line for k in ['_PASS','COLLISION_HULL','DEPARTURE_SAFE','ENTRY_OUTSIDE','Error','ERROR','WARNING'])]
    (out/f'{label}.log').write_text('\n'.join(selected)+'\n',encoding='utf-8')
    record=dict(run=label,exit_code=result.returncode,passed=result.returncode==0 and marker in text)
    records.append(record)
    (out/'runtime-summary.json').write_text(json.dumps(records,indent=2),encoding='utf-8')
    print(json.dumps(record),flush=True)
    if label=='debug': patterns=['station-*.png']
    elif label.startswith('return'): patterns=['dock-*.png']
    else: patterns=[f'launch-{label}.png']
    for pattern in patterns:
        for path in (root/'target/solarsystem').glob(pattern):
            shutil.copy2(path,out/f'{label}-{path.name}')
    if label in ['starwars','smilingfriends']:
        shutil.copy2(root/f'target/front-end/hangar-{label}.png',out/f'hangar-{label}.png')
    if not record['passed']:
        raise SystemExit(1)
