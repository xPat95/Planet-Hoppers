"""Optional read-only analysis using Blender's aud/numpy, not a runtime dependency.
blender --background --python-exit-code 1 --python tools/audio/inspect_southpark_voice.py
No speech recognition: silence boundaries cannot establish who is speaking.
"""
from pathlib import Path
import json
import aud
import numpy as np

ROOT = Path(__file__).resolve().parents[2]
path = ROOT/'assets/audio/sfx/South Park/they_killed_kenny.mp3'
sound = aud.Sound.file(str(path))
samples = sound.data()
rate, channels = sound.specs
step = int(rate*.02)
rms = [float(np.sqrt(np.mean(samples[i:i+step]**2)))
       for i in range(0, len(samples), step)]
pauses = []
start = None
for i, value in enumerate(rms+[1]):
    if value < .01 and start is None:
        start = i
    elif value >= .01 and start is not None:
        if (i-start)*.02 >= .06:
            pauses.append([round(start*.02, 2), round(i*.02, 2)])
        start = None
report = {'source':str(path.relative_to(ROOT)), 'frames':len(samples),
          'sample_rate':rate, 'channels':channels,
          'duration_seconds':len(samples)/rate, 'window_seconds':.02,
          'low_energy_intervals_seconds':pauses,
          'configured_kyle_switch_seconds':2.18,
          'manual_listening_verified':False,
          'note':'Initial switch inferred near the late pause; verify by listening.'}
out = ROOT/'target/phase5-voice-analysis.json'
out.write_text(json.dumps(report,indent=2),encoding='utf-8')
print('VOICE_ANALYSIS',json.dumps(report))
