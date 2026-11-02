//! Small presentation calculations shared by rendering and geometry tests.
use crate::ships::ShipModel;
use raylib::prelude::Vector3;

/// The prepared N-1 keeps its Blender -Y nose as glTF +Z; other selectable
/// ships face -Z after their configured model corrections (including Calaca).
pub fn hangar_nose(ship: &ShipModel) -> Vector3 {
    Vector3::new(
        0.0,
        0.0,
        if ship.model.contains("/starwars/") {
            1.0
        } else {
            -1.0
        },
    )
}

/// Start 30 degrees before facing the viewer, so positive exhibition yaw
/// initially approaches the front. Independent from platform rotation.
pub fn hangar_start_yaw(ship: &ShipModel, camera: Vector3) -> f32 {
    (camera.x.atan2(camera.z) - hangar_nose(ship).x.atan2(hangar_nose(ship).z)).to_degrees() - 30.0
}

/// A compact plume behind the full model bound, on the propulsion axis.
/// offset is measured from the model centre backwards; no sphere enters the hull.
pub fn exhaust_spheres(length: f32, rear_extent: f32, throttle: f32) -> [(f32, f32, u8); 3] {
    let k = throttle.clamp(0.0, 1.0);
    let first_radius = length * 0.028 * (0.6 + k);
    let base = rear_extent + first_radius + length * 0.015;
    std::array::from_fn(|index| {
        let i = index as f32 * 0.35;
        let radius = length * (0.028 - i * 0.014) * (0.6 + k);
        let offset = base + length * i * (0.3 + k) * 0.22;
        (offset, radius, [90, 55, 25][index])
    })
}
