"""Criaturas soldadas: piezas que se funden unas con otras (como una soldadura) y se doblan con huesos.

En vez de tubos y elipsoides que solo se tocan, cada pieza es un campo de distancia y todas se juntan con una unión
suave (`k` es el ancho de la soldadura). La superficie sale de una rejilla (surface nets, como la calavera del
planeta) en una pose neutra y se simplifica a un tamaño de juego. Cada pieza pertenece a un hueso; cada vértice
recibe pesos de los huesos cercanos y las poses se arman doblando esa misma malla (skinning), así que todas las
poses tienen los mismos vértices y el juego las puede mezclar.

Las puntas finas (dientes, garras, ojos) no entran en la rejilla: son piezas del Builder pegadas a un hueso.
"""

import bpy
import numpy as np
from mathutils import Vector


def _np(v):
    return np.array(tuple(v), dtype=np.float64)


def smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0, 1)
    return b + (a - b) * h - k * h * (1 - h)


class Weld:
    def __init__(self):
        self.parts = []                                   # (hueso, función de distancia, caja, k, resta, material)

    def _add(self, bone, f, lo, hi, k, sub=False, m=None):
        self.parts.append((bone, f, np.asarray(lo, float), np.asarray(hi, float), k, sub, m))

    def capsule(self, bone, a, b, r0, r1=None, k=0.012, sub=False, m=None):
        """Cápsula de a a b con radio que pasa de r0 a r1."""
        a, b = _np(a), _np(b)
        r1 = r0 if r1 is None else r1
        ba = b - a
        bb = max(ba @ ba, 1e-12)

        def f(P):
            h = np.clip((P - a) @ ba / bb, 0, 1)
            return np.linalg.norm(P - a - h[:, None] * ba, axis=1) - (r0 + (r1 - r0) * h)
        r = max(r0, r1)
        self._add(bone, f, np.minimum(a, b) - r, np.maximum(a, b) + r, k, sub, m)

    def chain(self, bone, pts, radii, k=0.012, sub=False, m=None):
        for i in range(len(pts) - 1):
            self.capsule(bone, pts[i], pts[i + 1], radii[i], radii[i + 1], k, sub, m)

    def ellipsoid(self, bone, c, R, r, k=0.012, sub=False, m=None):
        c, Ri, r = _np(c), np.array(R.transposed()), np.asarray(r, float)

        def f(P):
            q = (P - c) @ Ri.T / r
            k0 = np.linalg.norm(q, axis=1)
            k1 = np.linalg.norm(q / r, axis=1)
            return k0 * (k0 - 1.0) / np.maximum(k1, 1e-9)
        self._add(bone, f, c - r.max(), c + r.max(), k, sub, m)

    # --- Superficie ---------------------------------------------------------------------------------------------
    def surface(self, h):
        """Rejilla con la unión suave de todas las piezas (menos las restas) y su superficie."""
        adds = [p for p in self.parts if not p[5]]
        lo = np.min([p[2] - p[4] for p in adds], axis=0) - 3 * h
        hi = np.max([p[3] + p[4] for p in adds], axis=0) + 3 * h
        shape = tuple(np.ceil((hi - lo) / h).astype(int) + 1)
        print("WELD rejilla", shape, "%.0f M celdas" % (np.prod(shape) / 1e6))
        F = np.full(shape, 1.0, np.float32)
        for _, f, plo, phi, k, sub, _m in sorted(self.parts, key=lambda p: p[5]):   # Primero suma, después resta
            i0 = np.maximum(np.floor((plo - k - 2 * h - lo) / h).astype(int), 0)
            i1 = np.minimum(np.ceil((phi + k + 2 * h - lo) / h).astype(int) + 1, shape)
            ax = [lo[d] + h * np.arange(i0[d], i1[d]) for d in range(3)]
            g = np.stack(np.meshgrid(*ax, indexing="ij"), axis=-1).reshape(-1, 3)
            d = f(g).reshape(tuple(i1 - i0)).astype(np.float32)
            sl = tuple(slice(i0[i], i1[i]) for i in range(3))
            F[sl] = -smin(-F[sl], d, k) if sub else smin(F[sl], d, k)
        return surface_nets(F, lo, h)

    def field(self, P):
        """El mismo campo, en puntos sueltos (cada pieza solo se evalúa cerca de su caja)."""
        F = np.full(len(P), 1.0)
        for _, f, plo, phi, k, sub, _m in sorted(self.parts, key=lambda p: p[5]):
            sel = np.all((P >= plo - k - 1e-3) & (P <= phi + k + 1e-3), axis=1)
            if sel.any():
                d = f(P[sel])
                F[sel] = -smin(-F[sel], d, k) if sub else smin(F[sel], d, k)
        return F

    def face_materials(self, V, F, default):
        """Material de cada cara: el de la pieza más cercana (o default si esa pieza no tiene)."""
        C = np.array([V[list(f)].mean(axis=0) for f in F])
        best = np.full(len(C), np.inf)
        out = [default] * len(C)
        for _, f, plo, phi, k, sub, m in self.parts:
            if sub:
                continue
            d = f(C)
            for i in np.nonzero(d < best)[0]:
                out[i] = m or default
            best = np.minimum(best, d)
        return out

    def weights(self, V, sigma):
        """Pesos por hueso para cada vértice: los huesos más cerca pesan más; se mezclan en un ancho de ~sigma."""
        bones = sorted({p[0] for p in self.parts})
        D = np.full((len(V), len(bones)), 1e9)
        for bone, f, plo, phi, k, sub, _m in self.parts:
            if sub:
                continue
            j = bones.index(bone)
            D[:, j] = np.minimum(D[:, j], f(V))
        D -= D.min(axis=1, keepdims=True)
        W = np.where(D < 4 * sigma, np.exp(-D / sigma), 0.0)
        return bones, W / W.sum(axis=1, keepdims=True)


def surface_nets(F, lo, h):
    """Un vértice por celda que cruza la superficie y un cuadrilátero por arista de la rejilla que la cruza."""
    ins = F < 0
    nx, ny, nz = F.shape
    corners = [(a, b, c) for a in (0, 1) for b in (0, 1) for c in (0, 1)]
    cnt = sum(ins[a:nx - 1 + a, b:ny - 1 + b, c:nz - 1 + c].astype(np.int8) for a, b, c in corners)
    act = (cnt > 0) & (cnt < 8)
    del cnt
    ci = np.argwhere(act)
    vid = np.full(act.shape, -1, np.int32)
    vid[act] = np.arange(len(ci))
    acc = np.zeros((len(ci), 3))
    n = np.zeros(len(ci))
    for a in corners:                                    # Las 12 aristas de cada celda
        for ax in range(3):
            if a[ax]:
                continue
            b = list(a)
            b[ax] = 1
            ia, ib = ci + a, ci + b
            fa, fb = F[ia[:, 0], ia[:, 1], ia[:, 2]], F[ib[:, 0], ib[:, 1], ib[:, 2]]
            m = (fa < 0) != (fb < 0)
            t = fa[m] / (fa[m] - fb[m])
            acc[m] += ia[m] + t[:, None] * (np.array(b) - np.array(a))
            n[m] += 1
    verts = lo + h * acc / n[:, None]
    quads = []
    for ax in range(3):
        bx, cx = (ax + 1) % 3, (ax + 2) % 3
        s0 = [slice(None)] * 3
        s1 = [slice(None)] * 3
        s0[ax], s1[ax] = slice(0, -1), slice(1, None)
        low = ins[tuple(s0)]
        P = np.argwhere(low != ins[tuple(s1)])
        P = P[(P[:, bx] >= 1) & (P[:, cx] >= 1) & (P[:, bx] <= F.shape[bx] - 2) & (P[:, cx] <= F.shape[cx] - 2)]

        def cell(db, dc):
            Q = P.copy()
            Q[:, bx] += db
            Q[:, cx] += dc
            return vid[Q[:, 0], Q[:, 1], Q[:, 2]]
        q = np.stack([cell(-1, -1), cell(0, -1), cell(0, 0), cell(-1, 0)], axis=1)
        flip = ~low[P[:, 0], P[:, 1], P[:, 2]]           # Normal hacia afuera del volumen
        q[flip] = q[flip][:, ::-1]
        quads.append(q)
    return verts, np.concatenate(quads)


def decimate(verts, faces, tris, mats=None):
    """Simplifica la malla hasta ~tris triángulos (con el Decimate de Blender) y la devuelve como arreglos. Con
    mats (un material por cara) devuelve también el material de cada cara nueva."""
    me = bpy.data.meshes.new("_weld")
    me.from_pydata([tuple(v) for v in verts], [], [tuple(int(i) for i in f) for f in faces])
    if mats is not None:
        slots = list(dict.fromkeys(mats))
        for m in slots:
            me.materials.append(m)
        me.polygons.foreach_set("material_index", [slots.index(m) for m in mats])
    me.validate()
    ob = bpy.data.objects.new("_weld", me)
    bpy.context.scene.collection.objects.link(ob)
    me.calc_loop_triangles()
    mod = ob.modifiers.new("dec", "DECIMATE")
    mod.ratio = min(1.0, tris / max(len(me.loop_triangles), 1))
    mod.use_collapse_triangulate = True
    dg = bpy.context.evaluated_depsgraph_get()
    out = bpy.data.meshes.new_from_object(ob.evaluated_get(dg))
    V = np.array([tuple(v.co) for v in out.vertices])
    Fc = [tuple(p.vertices) for p in out.polygons]
    Mc = [out.materials[p.material_index] for p in out.polygons] if mats is not None else None
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(me)
    bpy.data.meshes.remove(out)
    return (V, Fc) if mats is None else (V, Fc, Mc)


def skin(V, bones, W, rest, pose):
    """Dobla los vértices de la pose neutra: rest/pose son {hueso: (origen, rotación)}."""
    out = np.zeros_like(V)
    for j, b in enumerate(bones):
        w = W[:, j]
        if not w.any():
            continue
        (o0, R0), (o1, R1) = rest[b], pose[b]
        M = np.array(R1 @ R0.transposed())
        out += w[:, None] * ((V - _np(o0)) @ M.T + _np(o1))
    return out


def rigid(v, bone, rest, pose):
    """Un punto pegado a un hueso, de la pose neutra a la pose."""
    (o0, R0), (o1, R1) = rest[bone], pose[bone]
    return o1 + (R1 @ R0.transposed()) @ (Vector(v) - o0)


def carve(V, F, M, cutters, cut_mat, keep_cut=True):
    """Resta sólidos a la malla con un booleano exacto (cutters: lista de (vértices, caras)). Las paredes del corte
    quedan con cut_mat (tallado) o se quitan con keep_cut=False (agujeros en una tela abierta)."""
    import bmesh
    slots = list(dict.fromkeys(M))
    me = bpy.data.meshes.new("_carve")
    me.from_pydata([tuple(v) for v in V], [], [tuple(int(i) for i in f) for f in F])
    for m in slots:
        me.materials.append(m)
    me.polygons.foreach_set("material_index", [slots.index(m) for m in M])
    me.validate()
    cv, cf = [], []
    for v_, f_ in cutters:
        cf += [tuple(i + len(cv) for i in f) for f in f_]
        cv += [tuple(v) for v in v_]
    cm = bpy.data.meshes.new("_cut")
    cm.from_pydata(cv, [], cf)
    cm.materials.append(cut_mat)
    bm = bmesh.new()
    bm.from_mesh(cm)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.to_mesh(cm)
    bm.free()
    ob, co = bpy.data.objects.new("_carve", me), bpy.data.objects.new("_cut", cm)
    for o in (ob, co):
        bpy.context.scene.collection.objects.link(o)
    mod = ob.modifiers.new("cut", "BOOLEAN")
    mod.object, mod.operation, mod.solver, mod.material_mode = co, "DIFFERENCE", "EXACT", "TRANSFER"
    mod.use_self = True                                  # Los cortes pueden pisarse entre sí
    out = bpy.data.meshes.new_from_object(ob.evaluated_get(bpy.context.evaluated_depsgraph_get()))
    keep = [p for p in out.polygons if keep_cut or out.materials[p.material_index].name != cut_mat.name]
    used = sorted({i for p in keep for i in p.vertices})
    remap = {v: j for j, v in enumerate(used)}
    V2 = np.array([tuple(out.vertices[v].co) for v in used])
    F2 = [tuple(remap[i] for i in p.vertices) for p in keep]
    M2 = [out.materials[p.material_index] for p in keep]
    print("CARVE: %d caras del corte" % sum(1 for m in M2 if m.name == cut_mat.name))
    for o in (ob, co):
        bpy.data.objects.remove(o)
    for m_ in (me, cm, out):
        bpy.data.meshes.remove(m_)
    return V2, F2, M2
