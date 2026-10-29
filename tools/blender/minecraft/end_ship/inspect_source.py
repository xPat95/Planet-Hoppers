from pathlib import Path
import gzip, struct, json, zipfile

ROOT = Path(__file__).resolve().parents[4]
SOURCE = Path.home() / 'AppData/Roaming/.minecraft/versions/1.21.11/1.21.11.jar'

def read_nbt(data):
    import io
    stream = io.BytesIO(gzip.decompress(data))
    def read(fmt):
        return struct.unpack('>' + fmt, stream.read(struct.calcsize('>' + fmt)))[0]
    def string():
        return stream.read(read('H')).decode('utf8')
    def payload(t):
        if t in (1,2,3,4,5,6): return read({1:'b',2:'h',3:'i',4:'q',5:'f',6:'d'}[t])
        if t == 7: return list(stream.read(read('i')))
        if t == 8: return string()
        if t == 9:
            sub, count = read('B'), read('i')
            return [payload(sub) for _ in range(count)]
        if t == 10:
            result = {}
            while True:
                sub = read('B')
                if not sub: return result
                name = string()
                result[name] = payload(sub)
        if t in (11,12): return [read('i' if t == 11 else 'q') for _ in range(read('i'))]
        raise ValueError(t)
    tag = read('B'); string()
    return payload(tag)

if __name__ == '__main__':
    from PIL import Image, ImageDraw
    outdir = ROOT / 'docs/references/minecraft/end_ship'
    outdir.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(SOURCE) as z:
        name = next(n for n in z.namelist() if n.endswith('end_city/ship.nbt'))
        raw = z.read(name)
    data = read_nbt(raw)
    (outdir/'ship_source.json').write_text(json.dumps(data, indent=2), encoding='utf8')
    import hashlib
    (outdir/'source_provenance.json').write_text(json.dumps({'game':'Minecraft Java 1.21.11', 'resource':name, 'sha256':hashlib.sha256(raw).hexdigest(), 'size':data['size']},indent=2),encoding='utf8')
    print('SIZE', data['size'])
    print('PALETTE',json.dumps(data['palette'],indent=2))
    files=list((ROOT/'assets/textures/ships').glob('*.png'))
    image=Image.new('RGB',(len(files)*180,170),'#333333'); draw=ImageDraw.Draw(image)
    for i,p in enumerate(files):
        im=Image.open(p); image.paste(im.resize((128,128),Image.Resampling.NEAREST),(i*180,0));draw.text((i*180,140),p.stem,fill='white')
    image.save(outdir/'texture_inspection.png')
