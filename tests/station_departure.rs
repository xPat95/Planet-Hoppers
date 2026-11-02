use planet_hoppers::{
    gameplay::{
        selection::ShipCatalog,
        station::{chase_position, DepartureBounds, Station},
    },
    ships::{FlightInput, ShipModel, ShipsConfig},
    solar_system::{SolarSystem, SolarSystemConfig},
};
use raylib::prelude::{Matrix, Vector3};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Independent CPU bounds from the real glTF accessors and node hierarchy.
/// No GPU, fixed hand-authored departure coordinates or estimated station radius.
fn fitted_half(path: &Path, length: f32, rotation: [f32; 3]) -> Vector3 {
    let bytes = fs::read(path).unwrap();
    let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let doc: Value = serde_json::from_slice(&bytes[20..20 + len]).unwrap();
    let nodes = doc["nodes"].as_array().unwrap();
    let mut parents = vec![None; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        if let Some(children) = n["children"].as_array() {
            for c in children {
                parents[c.as_u64().unwrap() as usize] = Some(i);
            }
        }
    }
    let val = |v: &Value, default: f32| v.as_f64().map_or(default, |x| x as f32);
    let transform = |mut p: Vector3, mut at: Option<usize>| {
        while let Some(i) = at {
            let n = &nodes[i];
            if let Some(m) = n["matrix"].as_array() {
                let m: Vec<f32> = m.iter().map(|x| val(x, 0.0)).collect();
                p = Vector3::new(
                    m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
                    m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
                    m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
                );
            } else {
                p = Vector3::new(
                    p.x * val(&n["scale"][0], 1.0),
                    p.y * val(&n["scale"][1], 1.0),
                    p.z * val(&n["scale"][2], 1.0),
                );
                let q = Vector3::new(
                    val(&n["rotation"][0], 0.0),
                    val(&n["rotation"][1], 0.0),
                    val(&n["rotation"][2], 0.0),
                );
                let t = q.cross(p) * 2.0;
                p += t * val(&n["rotation"][3], 1.0) + q.cross(t);
                p += Vector3::new(
                    val(&n["translation"][0], 0.0),
                    val(&n["translation"][1], 0.0),
                    val(&n["translation"][2], 0.0),
                );
            }
            at = parents[i];
        }
        p
    };
    let mut lo = Vector3::one() * f32::INFINITY;
    let mut hi = Vector3::one() * f32::NEG_INFINITY;
    for (i, n) in nodes.iter().enumerate() {
        let Some(mesh) = n["mesh"].as_u64() else {
            continue;
        };
        for prim in doc["meshes"][mesh as usize]["primitives"]
            .as_array()
            .unwrap()
        {
            let a = &doc["accessors"][prim["attributes"]["POSITION"].as_u64().unwrap() as usize];
            for k in 0..8 {
                let point = transform(
                    Vector3::new(
                        val(&a[if k & 1 == 0 { "min" } else { "max" }][0], 0.0),
                        val(&a[if k & 2 == 0 { "min" } else { "max" }][1], 0.0),
                        val(&a[if k & 4 == 0 { "min" } else { "max" }][2], 0.0),
                    ),
                    Some(i),
                );
                lo = Vector3::new(lo.x.min(point.x), lo.y.min(point.y), lo.z.min(point.z));
                hi = Vector3::new(hi.x.max(point.x), hi.y.max(point.y), hi.z.max(point.z));
            }
        }
    }
    let half = (hi - lo) * 0.5;
    let rot = Matrix::rotate_x(rotation[0].to_radians())
        * Matrix::rotate_y(rotation[1].to_radians())
        * Matrix::rotate_z(rotation[2].to_radians());
    let mut result = Vector3::zero();
    for k in 0..8 {
        let p = Vector3::new(
            if k & 1 == 0 { -half.x } else { half.x },
            if k & 2 == 0 { -half.y } else { half.y },
            if k & 4 == 0 { -half.z } else { half.z },
        )
        .transform_with(rot);
        result = Vector3::new(
            result.x.max(p.x.abs()),
            result.y.max(p.y.abs()),
            result.z.max(p.z.abs()),
        );
    }
    result * (length / (2.0 * result.x.max(result.y).max(result.z)))
}

fn setup() -> (Station, ShipsConfig, Vec<ShipModel>, Vector3) {
    let root = root();
    let station = Station::load(&root).unwrap();
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
    let models = catalog
        .choices
        .iter()
        .map(|c| catalog.model(c, &ships))
        .collect();
    let half = fitted_half(&root.join(&station.model), station.length, [0.0; 3]);
    (station, ships, models, half)
}

fn bounds(station_half: Vector3, model: &ShipModel, collision: f32) -> DepartureBounds {
    DepartureBounds {
        station_half_extents: station_half,
        ship_half_extents: if model.model.is_empty() {
            Vector3::one() * model.length * 0.5
        } else {
            fitted_half(
                &root().join(&model.model),
                model.length,
                model.model_rotation_degrees,
            )
        },
        ship_collision_radius: collision,
        ship_length: model.length,
        camera_radius: 1.0,
    }
}

fn verify(station: &Station, b: DepartureBounds) {
    let p = station.departure(b).unwrap();
    let f = p.ship.forward();
    assert!(f.dot(station.exit_forward()) > 0.98);
    assert!((station.center() - p.ship.position).dot(f) < 0.0);
    assert!(
        (p.ship.position - station.center()).length()
            > station.collision_radius + p.ship_radius + p.safety_margin
    );
    assert!(station
        .push_out(p.ship.position, p.ship.position, p.ship_radius)
        .is_none());
    assert!(
        (p.camera_position - station.center()).length()
            > station.collision_radius + b.camera_radius + p.safety_margin - 0.02
    );
    assert!((p.camera_position - p.ship.position).dot(f) < 0.0);
    assert!(station
        .push_out(p.camera_position, p.camera_position, b.camera_radius)
        .is_none());
    assert!(
        (chase_position(p.ship.position, f, b.ship_length) - p.camera_position).length() < 0.001
    );
    let model_rotation = Matrix::rotate_x(p.ship.pitch) * Matrix::rotate_y(p.ship.yaw);
    let rendered_nose = Vector3::new(0.0, 0.0, -1.0)
        .transform_with(model_rotation)
        .normalized();
    assert!(rendered_nose.dot(f) > 0.9999);
    let optical = (p.ship.position + f * b.ship_length * 2.0 - p.camera_position).normalized();
    let right = optical.cross(Vector3::up()).normalized();
    let up = right.cross(optical).normalized();
    let near = 0.5;
    let height = near * (30.0_f32.to_radians()).tan();
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            let lens = p.camera_position
                + optical * near
                + right * (x * height * 16.0 / 9.0)
                + up * (y * height);
            assert!((lens - station.center()).length() > station.collision_radius);
            assert!((lens - station.center()).dot(f) > p.hull_support);
        }
    }
    for k in 0..8 {
        let h = b.station_half_extents;
        let corner = station.center()
            + Vector3::new(
                if k & 1 == 0 { -h.x } else { h.x },
                if k & 2 == 0 { -h.y } else { h.y },
                if k & 4 == 0 { -h.z } else { h.z },
            )
            .transform_with(station.rotation());
        assert!((corner - p.ship.position).dot(f) < -p.ship_radius - p.safety_margin + 0.02);
        assert!((corner - p.camera_position).dot(f) < -b.camera_radius - p.safety_margin + 0.02);
    }
    assert_eq!(p.ship.speed, 0.0);
    assert_eq!(p.ship.bank, 0.0);
    assert_eq!(p.ship.carried_by, None);
    assert!(!p.ship.bumped);
    let again = station.departure(b).unwrap();
    assert_eq!(p.ship.position, again.ship.position);
    assert_eq!(p.ship.yaw, again.ship.yaw);
    assert_eq!(p.ship.pitch, again.ship.pitch);
}

#[test]
fn all_six_real_ship_bounds_launch_beyond_visual_and_collision_hulls() {
    let (station, ships, models, half) = setup();
    assert_eq!(models.len(), 6);
    println!(
        "station fitted half={half:?}; collider radius={}",
        station.collision_radius
    );
    for model in models {
        let b = bounds(half, &model, ships.flight.radius);
        println!(
            "{} half={:?} radius={}",
            model.id,
            b.ship_half_extents,
            b.ship_half_extents.length()
        );
        verify(&station, b);
    }
}

#[test]
fn departure_respects_rotations_translation_scale_and_mismatched_colliders() {
    let (mut station, ships, models, half) = setup();
    let base_length = station.length;
    let base_radius = station.collision_radius;
    for angles in [
        [0.0, -90.0, 0.0],
        [22.0, 37.0, 13.0],
        [0.0, 180.0, 0.0],
        [0.0, 0.0, 90.0],
    ] {
        station.rotation_degrees = angles;
        station.position = [430.0, 8200.0, -710.0];
        for scale in [0.5, 1.0, 1.7] {
            station.length = base_length * scale;
            for collider_scale in [0.45, 1.0, 1.8] {
                station.collision_radius = base_radius * collider_scale;
                for model in &models {
                    verify(&station, bounds(half * scale, model, ships.flight.radius));
                }
            }
        }
    }
}

#[test]
fn first_updates_are_stable_and_forward_input_moves_away_without_collision() {
    let (station, ships, models, half) = setup();
    let mut system = SolarSystem::new(
        &SolarSystemConfig::load(&root().join("config/solar-system.json")).unwrap(),
    );
    for model in models {
        let b = bounds(half, &model, ships.flight.radius);
        let plan = station.departure(b).unwrap();
        let mut ship = plan.ship.clone();
        for _ in 0..3 {
            system.update(1.0 / 60.0);
            ship.update(1.0 / 60.0, &FlightInput::default(), &ships.flight, &system);
            assert_eq!(ship.position, plan.ship.position);
            assert!(!ship.bumped);
        }
        for _ in 0..120 {
            let before = ship.position;
            system.update(1.0 / 60.0);
            ship.update(
                1.0 / 60.0,
                &FlightInput {
                    throttle: 1.0,
                    ..Default::default()
                },
                &ships.flight,
                &system,
            );
            assert!(!ship.bumped);
            assert!(station
                .push_out(before, ship.position, plan.ship_radius)
                .is_none());
            assert!(
                (ship.position - station.center()).length() > (before - station.center()).length()
            );
        }
        assert!((ship.position - plan.ship.position).dot(plan.ship.forward()) > 0.0);
    }
}

#[test]
fn legacy_dock_lane_does_not_disguise_an_unsafe_launch_and_return_still_works() {
    let (station, ships, models, half) = setup();
    let old = station.dock_target();
    assert!((old - station.center()).length() < station.collision_radius);
    assert!(station.push_out(old, old, ships.flight.radius).is_none());
    assert!(station.in_reach(old));
    assert!(!station.in_reach(station.approach()));
    for model in models {
        let plan = station
            .departure(bounds(half, &model, ships.flight.radius))
            .unwrap();
        assert!(!station.in_reach(plan.ship.position));
        assert!(station
            .push_out(station.approach(), old, ships.flight.radius)
            .is_none());
        // Outside the lane, the station's collision remains fully active.
        assert!(station
            .push_out(
                station.center() + station.exit_right() * station.collision_radius * 2.0,
                station.center(),
                ships.flight.radius
            )
            .is_some());
    }
}
