use super::resources::StaticModel;
use crate::minigames::southpark::threats::{ThreatDirector, ThreatKind, ThreatResult, ThreatState};
use raylib::{ffi, prelude::*};
use std::path::Path;

pub struct ThreatModels {
    bus: StaticModel,
    ufo: StaticModel,
    sewer: StaticModel,
    lid: StaticModel,
    panel: StaticModel,
    piano: StaticModel,
    support: StaticModel,
    lock: StaticModel,
    snowman: StaticModel,
    heater: StaticModel,
}
impl ThreatModels {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        directory: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut load = |name: &str| {
            StaticModel::load(rl, thread, &directory.join(format!("{name}.glb")), None)
        };
        let result = Self {
            bus: load("bus")?,
            ufo: load("ufo")?,
            sewer: load("sewer")?,
            lid: load("lid")?,
            panel: load("brake_panel")?,
            piano: load("phase4b/piano")?,
            support: load("phase4b/piano_support")?,
            lock: load("phase4b/piano_lock")?,
            snowman: load("phase4b/snowman")?,
            heater: load("phase4b/heater")?,
        };
        let audit: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("asset-audit.json"))?)?;
        let audit4b: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("phase4b/asset-audit.json"))?)?;
        for (name, model, audit) in [
            ("bus", &result.bus, &audit),
            ("ufo", &result.ufo, &audit),
            ("sewer", &result.sewer, &audit),
            ("lid", &result.lid, &audit),
            ("brake_panel", &result.panel, &audit),
            ("piano", &result.piano, &audit4b),
            ("piano_support", &result.support, &audit4b),
            ("piano_lock", &result.lock, &audit4b),
            ("snowman", &result.snowman, &audit4b),
            ("heater", &result.heater, &audit4b),
        ] {
            let expected = &audit[name];
            if expected["triangles"].as_u64() != Some(model.audit.triangles as u64) {
                return Err(format!("{name} GLB includes missing/extra triangles").into());
            }
            let lo = &expected["blender_bounds"][0];
            let hi = &expected["blender_bounds"][1];
            let min = [
                lo[0].as_f64().unwrap() as f32,
                lo[2].as_f64().unwrap() as f32,
                -hi[1].as_f64().unwrap() as f32,
            ];
            let max = [
                hi[0].as_f64().unwrap() as f32,
                hi[2].as_f64().unwrap() as f32,
                -lo[1].as_f64().unwrap() as f32,
            ];
            if (0..3).any(|i| {
                (min[i] - model.audit.bounds_min[i]).abs() > 0.002
                    || (max[i] - model.audit.bounds_max[i]).abs() > 0.002
            }) {
                return Err(format!("{name} GLB does not match Blender dimensions").into());
            }
        }
        let b = &result.bus.audit;
        if b.bounds_max[1] < 2.5
            || b.bounds_max[1] > 2.9
            || b.bounds_min[0] < -3.11
            || b.bounds_max[0] > 3.11
            || b.bounds_max[2] > 1.23
        {
            return Err("Bus model dimensions disagree with its validated corridor".into());
        }
        println!(
            "THREAT_MODELS_LOADED bus={} ufo={} sewer={} lid={} panel={}",
            b.triangles,
            result.ufo.audit.triangles,
            result.sewer.audit.triangles,
            result.lid.audit.triangles,
            result.panel.audit.triangles
        );
        println!(
            "PHASE4B_MODELS_LOADED piano={} support={} lock={} snowman={} heater={}",
            result.piano.audit.triangles,
            result.support.audit.triangles,
            result.lock.audit.triangles,
            result.snowman.audit.triangles,
            result.heater.audit.triangles
        );
        Ok(result)
    }
    pub fn draw(
        &mut self,
        view: &mut RaylibMode3D<RaylibDrawHandle>,
        camera: Vector3,
        event: &ThreatDirector,
    ) {
        if event.state == ThreatState::Inactive || event.released {
            return;
        }
        match event.kind {
            Some(ThreatKind::Bus) => {
                self.bus
                    .draw_at(view, camera, event.bus_position(), std::f32::consts::PI);
                self.panel
                    .draw_at(view, camera, event.panel_position(), std::f32::consts::PI);
                view.draw_cube_wires(
                    event.panel_position() + Vector3::up() * 1.5,
                    0.72,
                    0.72,
                    0.5,
                    if event.result == Some(ThreatResult::Survived) {
                        Color::GREEN
                    } else {
                        Color::YELLOW
                    },
                );
            }
            Some(ThreatKind::Ufo) => {
                self.ufo.draw_at(view, camera, event.ufo_position(), 0.0);
                for i in 0..3 {
                    view.draw_sphere(
                        event.light_position(i),
                        0.25,
                        if event.lights[i] {
                            [Color::YELLOW, Color::SKYBLUE, Color::MAGENTA][i]
                        } else {
                            Color::new(30, 35, 40, 255)
                        },
                    );
                }
                if matches!(event.state, ThreatState::Active | ThreatState::Preparing) {
                    unsafe {
                        ffi::rlDrawRenderBatchActive();
                        ffi::rlDisableDepthMask();
                    }
                    view.draw_cylinder_ex(
                        event.stage(ThreatKind::Ufo) + Vector3::up() * 0.1,
                        event.ufo_position() - Vector3::up() * 0.5,
                        1.0,
                        0.45,
                        16,
                        Color::new(100, 255, 160, 55),
                    );
                    unsafe {
                        ffi::rlDrawRenderBatchActive();
                        ffi::rlEnableDepthMask();
                    }
                }
            }
            Some(ThreatKind::Sewer) => {
                self.sewer.draw_at(view, camera, event.hole_position(), 0.0);
                if let Some(p) = event.lid_position() {
                    self.lid.draw_at(view, camera, p, 0.0);
                }
                if event.state == ThreatState::Active {
                    let p = if event.carrying_lid {
                        event.hole_position()
                    } else {
                        event.cover_position()
                    };
                    view.draw_cube_wires(p + Vector3::up() * 0.16, 1.9, 0.25, 1.9, Color::YELLOW);
                }
            }
            Some(ThreatKind::Piano) => {
                self.support
                    .draw_at(view, camera, event.piano_impact(), 0.0);
                self.piano.draw_pose(
                    view,
                    camera,
                    event.piano_position(),
                    0.0,
                    event.piano_sway(),
                    Vector3::one(),
                );
                self.lock.draw_at(view, camera, event.lock_position(), 0.0);
                let color = if event.piano_locked {
                    Color::GREEN
                } else {
                    Color::YELLOW
                };
                view.draw_sphere(
                    event.lock_position() + Vector3::new(0.0, 1.35, -0.3),
                    0.17,
                    color,
                );
                if matches!(event.state, ThreatState::Preparing | ThreatState::Active) {
                    let warning = if event.progress > 0.75 {
                        Color::RED
                    } else {
                        Color::ORANGE
                    };
                    view.draw_cube_wires(
                        event.piano_impact() + Vector3::up() * 0.06,
                        4.6,
                        0.1,
                        3.0,
                        warning,
                    );
                    view.draw_cylinder_ex(
                        event.piano_impact() + Vector3::up() * 9.2,
                        event.piano_position() + Vector3::up() * 1.7,
                        0.04,
                        0.04,
                        6,
                        warning,
                    );
                }
                if event.piano_locked {
                    view.draw_cylinder_ex(
                        event.piano_impact() + Vector3::up() * 9.2,
                        event.piano_position() + Vector3::up() * 1.7,
                        0.04,
                        0.04,
                        6,
                        Color::GREEN,
                    );
                }
                if event.state == ThreatState::Failed && event.state_time < 0.7 {
                    view.draw_cube_wires(
                        event.piano_impact() + Vector3::up() * 0.15,
                        4.6 + event.state_time * 2.0,
                        0.2,
                        3.0 + event.state_time * 2.0,
                        Color::ORANGE,
                    );
                }
            }
            Some(ThreatKind::Snowman) => {
                for i in 0..3 {
                    let p = event.heater_position(i);
                    self.heater.draw_at(view, camera, p, 0.0);
                    let color = if event.heaters[i] {
                        Color::ORANGE
                    } else {
                        Color::DARKGRAY
                    };
                    view.draw_cube(p + Vector3::new(0.0, 0.9, -0.245), 0.5, 0.55, 0.015, color);
                    view.draw_sphere(
                        p + Vector3::new(0.0, 1.32, -0.3),
                        0.065,
                        if event.heaters[i] {
                            Color::GREEN
                        } else {
                            Color::RED
                        },
                    );
                }
                if event.state == ThreatState::Succeeded {
                    view.draw_cylinder(
                        event.snow_position() + Vector3::up() * 0.025,
                        1.0,
                        1.0,
                        0.045,
                        20,
                        Color::SKYBLUE,
                    );
                } else {
                    let sway = if event.state == ThreatState::Active {
                        (event.elapsed * 4.0).sin() * 0.045
                    } else {
                        0.0
                    };
                    self.snowman.draw_pose(
                        view,
                        camera,
                        event.snow_position(),
                        event.snow_yaw,
                        sway,
                        event.snow_scale(),
                    );
                }
            }
            None => {}
        }
    }
}

pub fn draw_status(d: &mut RaylibDrawHandle, event: &ThreatDirector, width: i32) {
    let Some(kind) = event.kind else {
        return;
    };
    d.draw_rectangle(8, 100, width - 16, 90, Color::new(15, 25, 35, 220));
    let state = match event.state {
        ThreatState::Inactive => "Inactiva",
        ThreatState::Preparing => "Preparando",
        ThreatState::Active => "Activa",
        ThreatState::Resolving => "Resolviendo",
        ThreatState::Succeeded => "Superada",
        ThreatState::Failed => "Fallida",
    };
    d.draw_text(
        &format!("{} | {} | {:.1} s", kind.label(), state, event.remaining()),
        18,
        110,
        19,
        Color::WHITE,
    );
    if event.kind == Some(ThreatKind::Ufo) {
        d.draw_text(
            &format!(
                "Luces desactivadas: {}/3",
                event.lights.iter().filter(|on| !**on).count()
            ),
            18,
            140,
            20,
            Color::YELLOW,
        );
    }
    if event.carrying_lid {
        d.draw_text(
            "Llevas una tapa: apunta al agujero y pulsa E",
            18,
            140,
            20,
            Color::YELLOW,
        );
    }
    if event.kind == Some(ThreatKind::Snowman) {
        d.draw_text(
            &format!(
                "Calefactores: {}/3",
                event.heaters.iter().filter(|on| **on).count()
            ),
            18,
            140,
            20,
            Color::YELLOW,
        );
    }
    if event.kind == Some(ThreatKind::Piano) && event.state == ThreatState::Active {
        d.draw_text(
            if event.progress > 0.75 {
                "¡El cable está por romperse! Busca el seguro amarillo a nivel del suelo."
            } else {
                "Peligro sobre la calle. Seguro amarillo junto a la acera."
            },
            18,
            140,
            18,
            Color::ORANGE,
        );
    }
    if let Some(result) = event.result {
        let (text, color) = if result == ThreatResult::Survived {
            ("Kenny sobrevivió. Amenaza superada.", Color::GREEN)
        } else {
            (
                "Kenny murió. F8 para reiniciar esta amenaza.",
                Color::ORANGE,
            )
        };
        if result == ThreatResult::Survived || event.state_time < 6.0 {
            d.draw_text(text, 18, 164, 18, color);
        }
    }
}
