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
        };
        let audit: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("asset-audit.json"))?)?;
        for (name, model) in [
            ("bus", &result.bus),
            ("ufo", &result.ufo),
            ("sewer", &result.sewer),
            ("lid", &result.lid),
            ("brake_panel", &result.panel),
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
        Ok(result)
    }
    pub fn draw(
        &mut self,
        view: &mut RaylibMode3D<RaylibDrawHandle>,
        camera: Vector3,
        event: &ThreatDirector,
    ) {
        if event.state == ThreatState::Inactive {
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
                        ThreatKind::Ufo.stage() + Vector3::up() * 0.1,
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
