//! Conservative 4-unit surface cells generated from the original GLB triangles.
//! Empty space between arms and inside the ring is not a solid world boundary.
use raylib::prelude::Vector3;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct CollisionHull {
    pub source: String,
    pub reference_length: f32,
    pub raw_center: [f32; 3],
    pub raw_scale: f32,
    pub cell: f32,
    /// Inclusive x range, y, z. Adjacent occupied cells are merged along x.
    pub rows: Vec<[i32; 4]>,
    #[serde(skip)]
    pub offset_fraction: Vector3,
    #[serde(skip)]
    pub extra: Vec<(Vector3, Vector3)>,
    #[serde(skip)]
    pub bounds: [Vector3; 2],
}

impl CollisionHull {
    pub fn boxes(&self) -> impl Iterator<Item = (Vector3, Vector3)> + '_ {
        self.rows
            .iter()
            .map(|r| {
                (
                    Vector3::new(r[0] as f32, r[2] as f32, r[3] as f32) * self.cell,
                    Vector3::new((r[1] + 1) as f32, (r[2] + 1) as f32, (r[3] + 1) as f32)
                        * self.cell,
                )
            })
            .chain(self.extra.iter().copied())
    }
    fn embedded(&self, p: Vector3, radius: f32) -> bool {
        self.boxes().any(|(lo, hi)| {
            contains(
                p,
                lo - Vector3::one() * radius,
                hi + Vector3::one() * radius,
            )
        })
    }
    /// Escape a connected union of expanded cells, not just one overlapping box.
    /// This prevents alternating corrections at adjacent surface cells.
    fn recover(&self, p: Vector3, radius: f32) -> Option<Vector3> {
        if !self.embedded(p, radius) {
            return None;
        }
        let mut best = None;
        let mut distance = f32::INFINITY;
        for axis in 0..3 {
            let a = (axis + 1) % 3;
            let b = (axis + 2) % 3;
            let mut intervals: Vec<_> = self
                .boxes()
                .filter_map(|(lo, hi)| {
                    let lo = lo - Vector3::one() * radius;
                    let hi = hi + Vector3::one() * radius;
                    (component(p, a) > component(lo, a)
                        && component(p, a) < component(hi, a)
                        && component(p, b) > component(lo, b)
                        && component(p, b) < component(hi, b))
                    .then_some((component(lo, axis), component(hi, axis)))
                })
                .collect();
            intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut merged: Vec<(f32, f32)> = Vec::new();
            for (lo, hi) in intervals {
                if let Some(last) = merged.last_mut().filter(|last| lo <= last.1) {
                    last.1 = last.1.max(hi);
                } else {
                    merged.push((lo, hi));
                }
            }
            for (lo, hi) in merged {
                if component(p, axis) > lo && component(p, axis) < hi {
                    for edge in [lo - 0.02, hi + 0.02] {
                        let mut candidate = p;
                        set_component(&mut candidate, axis, edge);
                        let d = (candidate - p).length();
                        if d < distance {
                            best = Some(candidate);
                            distance = d;
                        }
                    }
                }
            }
        }
        best
    }
    pub fn push_out(&self, start: Vector3, end: Vector3, radius: f32) -> Option<Vector3> {
        if let Some(out) = self.recover(start, radius) {
            return Some(out);
        }
        let movement = end - start;
        let mut first: Option<(f32, Vector3)> = None;
        for (lo, hi) in self.boxes() {
            if let Some((t, normal)) = sweep_box(start, end, lo, hi, radius) {
                if first.as_ref().is_none_or(|(old, _)| t < *old) {
                    first = Some((t, normal));
                }
            }
        }
        first.map(|(t, normal)| {
            let p = start + movement * t + normal * 0.02;
            self.recover(p, radius).unwrap_or(p)
        })
    }
}

pub fn component(p: Vector3, axis: usize) -> f32 {
    [p.x, p.y, p.z][axis]
}
fn set_component(p: &mut Vector3, axis: usize, value: f32) {
    match axis {
        0 => p.x = value,
        1 => p.y = value,
        _ => p.z = value,
    }
}
fn contains(p: Vector3, lo: Vector3, hi: Vector3) -> bool {
    (0..3).all(|axis| {
        component(p, axis) > component(lo, axis) && component(p, axis) < component(hi, axis)
    })
}
/// Swept sphere against a conservative expanded cell; earliest contact, no tunnelling.
pub fn sweep_box(
    start: Vector3,
    end: Vector3,
    lo: Vector3,
    hi: Vector3,
    radius: f32,
) -> Option<(f32, Vector3)> {
    let lo = lo - Vector3::one() * radius;
    let hi = hi + Vector3::one() * radius;
    let mut near = 0.0_f32;
    let mut far = 1.0_f32;
    let mut normal = Vector3::zero();
    for axis in 0..3 {
        let origin = component(start, axis);
        let delta = component(end - start, axis);
        if delta.abs() < 1e-8 {
            if origin < component(lo, axis) || origin > component(hi, axis) {
                return None;
            }
        } else {
            let a = (component(lo, axis) - origin) / delta;
            let b = (component(hi, axis) - origin) / delta;
            if a.min(b) >= near {
                near = a.min(b);
                normal = Vector3::zero();
                set_component(&mut normal, axis, -delta.signum());
            }
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    (near >= 0.0 && near <= 1.0 && normal.length() > 0.0).then_some((near, normal))
}
