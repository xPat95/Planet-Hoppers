"""Prepare supplied MP4s without modifying them. FFmpeg is only a build-time tool.
Set FFMPEG_PATH, or put ffmpeg on PATH. The player streams indexed JPEGs + Ogg.
"""
from pathlib import Path
import hashlib, json, os, shutil, struct, subprocess
ROOT=Path(__file__).resolve().parents[2]/"assets/minigames/starwars"
W,H,FPS=1280,720,30

def prepare(ffmpeg, source, stem):
    dest=ROOT/'videos'
    temp=dest/(stem+'.sfv.tmp')
    proc=subprocess.Popen([ffmpeg,'-nostdin','-hide_banner','-loglevel','error','-i',str(source),'-map','0:v:0','-vf',f'fps={FPS},scale={W}:{H}:force_original_aspect_ratio=decrease:flags=lanczos,pad={W}:{H}:(ow-iw)/2:(oh-ih)/2,setsar=1','-c:v','mjpeg','-q:v','3','-f','image2pipe','pipe:1'],stdout=subprocess.PIPE)
    count=0; pending=b''
    with temp.open('wb') as f:
        f.write(b'SFV1'+struct.pack('<IIII',W,H,FPS,0))
        while True:
            chunk=proc.stdout.read(65536)
            pending+=chunk
            while True:
                end=pending.find(b'\xff\xd9')
                if end<0: break
                frame,pending=pending[:end+2],pending[end+2:]
                if not frame.startswith(b'\xff\xd8'): raise RuntimeError('Invalid JPEG stream')
                f.write(struct.pack('<I',len(frame))); f.write(frame); count+=1
            if not chunk:break
        if proc.wait()!=0 or pending or count==0:raise RuntimeError('Incomplete conversion')
        f.seek(16); f.write(struct.pack('<I',count))
    audio=dest/(stem+'.ogg.tmp')
    subprocess.run([ffmpeg,'-nostdin','-hide_banner','-loglevel','error','-y','-i',str(source),'-map','0:a:0','-vn','-c:a','libvorbis','-q:a','5','-f','ogg',str(audio)],check=True)
    temp.replace(dest/(stem+'.sfv'));audio.replace(dest/(stem+'.ogg'))
    meta=dict(source=source.name,sha256=hashlib.sha256(source.read_bytes()).hexdigest(),width=W,height=H,fps=FPS,frames=count,duration=count/FPS)
    (dest/(stem+'.json')).write_text(json.dumps(meta,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(meta),flush=True)
if __name__=='__main__':
    ffmpeg=os.environ.get('FFMPEG_PATH') or shutil.which('ffmpeg')
    if not ffmpeg:raise RuntimeError('Set FFMPEG_PATH to your ffmpeg executable')
    for source,stem in [('Inicio_video1_em.mp4','inicio'),('final_video1_em.mp4','final')]:prepare(ffmpeg,ROOT/'videos'/source,stem)
