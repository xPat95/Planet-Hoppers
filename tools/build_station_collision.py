"""Read the original GLB (including u32 indices); conservatively voxelize surfaces.
No Blender/model changes. Offline dependency: numpy. Runtime needs only the JSON.
"""
from pathlib import Path
import hashlib
import json
import struct
import numpy as np

root = Path(__file__).resolve().parents[1]
config = json.loads((root / 'config/station.json').read_text(encoding='utf-8'))
source = root / config['model']
blob = source.read_bytes()
json_size = struct.unpack_from('<I', blob, 12)[0]
doc = json.loads(blob[20:20 + json_size])
binary = memoryview(blob)[28 + json_size:]

def accessor(index):
    a = doc['accessors'][index]
    view = doc['bufferViews'][a['bufferView']]
    dtype = {5123: '<u2', 5125: '<u4', 5126: '<f4'}[a['componentType']]
    width = {'SCALAR': 1, 'VEC3': 3}[a['type']]
    start = view.get('byteOffset', 0) + a.get('byteOffset', 0)
    return np.ndarray((a['count'], width), dtype=dtype, buffer=binary, offset=start,
                      strides=(view.get('byteStride', np.dtype(dtype).itemsize * width), np.dtype(dtype).itemsize))

nodes = doc['nodes']
parents = {child: i for i, node in enumerate(nodes) for child in node.get('children', [])}
def matrix(i):
    n = nodes[i]
    if 'matrix' in n:
        m = np.array(n['matrix']).reshape(4, 4).T
    else:
        x, y, z, w = n.get('rotation', [0, 0, 0, 1])
        m = np.eye(4)
        m[:3, :3] = [[1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w)],
                      [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w)],
                      [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y)]]
        m[:3, :3] *= n.get('scale', [1, 1, 1])
        m[:3, 3] = n.get('translation', [0, 0, 0])
    return matrix(parents[i]) @ m if i in parents else m

parts = []
for i, node in enumerate(nodes):
    if 'mesh' not in node:
        continue
    m = matrix(i)
    for prim in doc['meshes'][node['mesh']]['primitives']:
        assert prim.get('mode', 4) == 4
        vertices = accessor(prim['attributes']['POSITION']) @ m[:3, :3].T + m[:3, 3]
        indices = accessor(prim['indices']).reshape(-1, 3)
        parts.append((vertices, indices))
lo = np.min([v.min(axis=0) for v, _ in parts], axis=0)
hi = np.max([v.max(axis=0) for v, _ in parts], axis=0)
scale = config['length'] / (hi - lo).max()
triangles = np.concatenate([((v - (lo + hi) / 2) * scale)[ix] for v, ix in parts]).astype(np.float32)
cell = 4.0
keys = []
for start in range(0, len(triangles), 512):
    tri = triangles[start:start + 512]
    low = np.floor(tri.min(axis=1) / cell).astype(np.int32)
    high = np.floor(tri.max(axis=1) / cell).astype(np.int32)
    dims = high - low + 1
    counts = dims.prod(axis=1)
    ids = np.repeat(np.arange(len(tri)), counts)
    flat = np.arange(counts.sum()) - np.repeat(np.cumsum(counts) - counts, counts)
    d = dims[ids]
    coords = low[ids] + np.stack([flat % d[:, 0], flat // d[:, 0] % d[:, 1], flat // (d[:, 0] * d[:, 1])], axis=1)
    centres = (coords + 0.5) * cell
    edges = np.roll(tri, -1, axis=1) - tri
    axes = np.concatenate([np.cross(edges[:, 0], edges[:, 1])[:, None, :],
                           np.cross(edges[:, :, None, :], np.eye(3)[None, None, :, :]).reshape(-1, 9, 3)], axis=1)
    projections = np.einsum('tvi,tai->tva', tri, axes)
    radii = np.abs(axes).sum(axis=2) * cell * 0.5
    centre_proj = np.einsum('ni,nai->na', centres, axes[ids])
    keep = ((projections.min(axis=1)[ids] <= centre_proj + radii[ids] + 1e-4)
            & (projections.max(axis=1)[ids] >= centre_proj - radii[ids] - 1e-4)).all(axis=1)
    c = coords[keep] + 512
    assert ((c >= 0) & (c < 1024)).all()
    keys.append(c[:, 0] + 1024 * (c[:, 1] + 1024 * c[:, 2]))
occupied = np.unique(np.concatenate(keys))
rows = []
for key in occupied.tolist():
    x, y, z = key % 1024 - 512, key // 1024 % 1024 - 512, key // 1048576 - 512
    if rows and rows[-1][2:] == [y, z] and rows[-1][1] + 1 == x:
        rows[-1][1] = x
    else:
        rows.append([x, x, y, z])
out = source.with_suffix('.collision.json')
data = {'source': config['model'], 'source_sha256': hashlib.sha256(blob).hexdigest(),
        'reference_length': config['length'], 'half_extents': ((hi - lo) * scale / 2).tolist(),
        'raw_center': ((lo + hi) / 2).tolist(), 'raw_scale': float(scale),
        'raw_center': ((lo + hi) / 2).tolist(), 'raw_scale': float(scale),
        'cell': cell, 'triangles': len(triangles), 'occupied_cells': len(occupied), 'rows': rows}
out.write_text(json.dumps(data, separators=(',', ':')), encoding='utf-8')
print(json.dumps({k: v for k, v in data.items() if k != 'rows'}), 'boxes=', len(rows), flush=True)
