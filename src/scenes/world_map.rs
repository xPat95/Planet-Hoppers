//! One cached geometry-derived texture; per-frame work is projection/markers.
use crate::minigames::{
    common::world_map::{circle_marker, MapBounds, MapView},
    southpark::{
        session::{SaveKennySession, SessionState},
        SouthParkScene,
    },
};
use raylib::{ffi, prelude::*};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Metadata {
    schema_version: u32,
    world_min: [f32; 2],
    world_max: [f32; 2],
    image_size: [i32; 2],
}
pub struct WorldMap {
    texture: Texture2D,
    mask: Shader,
    pub view: MapView,
    pub zoomed: bool,
    pub dragged: bool,
}
impl WorldMap {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
        scene: &SouthParkScene,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let folder = root.join("assets/textures/minigames/South Park");
        let meta: Metadata =
            serde_json::from_slice(&std::fs::read(folder.join("southpark_map.json"))?)?;
        let texture =
            rl.load_texture(thread, &folder.join("southpark_map.png").to_string_lossy())?;
        let b = scene.world.terrain;
        if meta.schema_version != 1
            || meta.image_size != [texture.width, texture.height]
            || meta.world_min != [b.min.x, b.min.z]
            || meta.world_max != [b.max.x, b.max.z]
        {
            return Err("Map/scene bounds mismatch; regenerate map".into());
        }
        let bounds = MapBounds {
            min: Vector2::new(b.min.x, b.min.z),
            max: Vector2::new(b.max.x, b.max.z),
        };
        let mask = rl.load_shader_from_memory(
            thread,
            None,
            Some(include_str!("../graphics/shaders/minimap.fs")),
        );
        println!(
            "WORLD_MAP_LOADED {}x{} {:?}",
            texture.width, texture.height, bounds
        );
        Ok(Self {
            texture,
            mask,
            view: MapView::new(bounds),
            zoomed: false,
            dragged: false,
        })
    }
    fn viewport(w: i32, h: i32) -> (Vector2, Vector2) {
        (
            Vector2::new(24.0, 58.0),
            Vector2::new((w - 48) as f32, (h - 112) as f32),
        )
    }
    pub fn input(&mut self, rl: &RaylibHandle, player: Vector3) {
        let (_, size) = Self::viewport(rl.get_screen_width(), rl.get_screen_height());
        let wheel = rl.get_mouse_wheel_move();
        if wheel.abs() > 0.0 {
            self.view.zoom_by(wheel, size);
            self.zoomed = true;
        }
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let delta = rl.get_mouse_delta();
            if delta.length() > 0.0 {
                self.view.pan(delta, size);
                self.dragged = true;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            self.view.recenter(player, size);
        }
    }
    pub fn draw(
        &mut self,
        d: &mut RaylibDrawHandle,
        scene: &SouthParkScene,
        s: &SaveKennySession,
        w: i32,
        h: i32,
    ) {
        let marked = s.location_visible() || (!s.in_match() && scene.threats.running());
        if s.state == SessionState::MapOpen {
            self.full(d, scene, s, marked, w, h);
            return;
        }
        if s.blocks_controls() {
            return;
        }
        let radius = (h as f32 * 0.14).clamp(60.0, 96.0);
        let centre = Vector2::new(radius + 18.0, h as f32 - radius - 18.0);
        let metres = 65.0;
        let scale = radius / metres;
        d.draw_circle_v(centre, radius + 5.0, Color::new(21, 43, 58, 255));
        d.draw_circle_v(centre, radius, Color::new(224, 235, 234, 255));
        self.mask
            .set_shader_value(self.mask.get_shader_location("circleCenter"), centre);
        self.mask
            .set_shader_value(self.mask.get_shader_location("circleRadius"), radius);
        self.mask
            .set_shader_value(self.mask.get_shader_location("screenHeight"), h as f32);
        let min = self.view.bounds.min;
        let size = self.view.bounds.size();
        let full = Rectangle::new(
            centre.x + (min.x - scene.player.feet.x) * scale,
            centre.y + (min.y - scene.player.feet.z) * scale,
            size.x * scale,
            size.y * scale,
        );
        let left = full.x.max(centre.x - radius);
        let top = full.y.max(centre.y - radius);
        let right = (full.x + full.width).min(centre.x + radius);
        let bottom = (full.y + full.height).min(centre.y + radius);
        if right > left && bottom > top {
            let source = Rectangle::new(
                (left - full.x) / full.width * self.texture.width as f32,
                (top - full.y) / full.height * self.texture.height as f32,
                (right - left) / full.width * self.texture.width as f32,
                (bottom - top) / full.height * self.texture.height as f32,
            );
            let mut shader = d.begin_shader_mode(&mut self.mask);
            shader.draw_texture_pro(
                &self.texture,
                source,
                Rectangle::new(left, top, right - left, bottom - top),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        d.draw_text(
            "N",
            centre.x as i32 - 5,
            (centre.y - radius + 4.0) as i32,
            16,
            Color::DARKBLUE,
        );
        arrow(d, centre, scene.player.yaw, Color::BLUE);
        if let Some(a) = &s.adventure {
            for (i,g) in a.groups.iter().enumerate() {
                let (p, _) = circle_marker(scene.player.feet, g.positions[0], centre, radius - 12.0, metres);
                encounter_marker(d,p,i,a.completed[i]);
            }
        }
        if marked {
            let (danger, edge) = circle_marker(
                scene.player.feet,
                scene.threats.hazard_position(),
                centre,
                radius - 16.0,
                metres,
            );
            warning(d, danger, edge);
            let (kenny, edge) = circle_marker(
                scene.player.feet,
                scene.kenny.position(),
                centre,
                radius - 12.0,
                metres,
            );
            kenny_marker(d, kenny, edge);
        }
        d.draw_text(if s.adventure.as_ref().is_some_and(|a| !a.spanish) { "M: map" } else { "M: mapa" }, 20, h - 16, 12, Color::DARKBLUE);
    }
    fn full(&self, d: &mut RaylibDrawHandle, scene: &SouthParkScene, s: &SaveKennySession, marked: bool, w: i32, h: i32) {
        let tr = |en, es| if s.adventure.as_ref().is_some_and(|a| !a.spanish) { en } else { es };
        d.draw_rectangle(0, 0, w, h, Color::new(13, 28, 42, 250));
        d.draw_text(tr("SOUTH PARK - MAP","SOUTH PARK - MAPA"), 24, 20, 24, Color::WHITE);
        d.draw_text("N", w - 42, 18, 24, Color::WHITE);
        let (origin, size) = Self::viewport(w, h);
        let scale = self.view.scale(size);
        unsafe {
            ffi::BeginScissorMode(
                origin.x as i32,
                origin.y as i32,
                size.x as i32,
                size.y as i32,
            );
        }
        d.draw_rectangle_rec(
            Rectangle::new(origin.x, origin.y, size.x, size.y),
            Color::new(195, 214, 216, 255),
        );
        let min = self.view.bounds.min;
        let extent = self.view.bounds.size();
        let p = self
            .view
            .screen(Vector3::new(min.x, 0.0, min.y), origin, size);
        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(
                0.0,
                0.0,
                self.texture.width as f32,
                self.texture.height as f32,
            ),
            Rectangle::new(p.x, p.y, extent.x * scale, extent.y * scale),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        arrow(
            d,
            self.view.screen(scene.player.feet, origin, size),
            scene.player.yaw,
            Color::BLUE,
        );
        if let Some(a) = &s.adventure {
            for (i,g) in a.groups.iter().enumerate() {
                encounter_marker(d,self.view.screen(g.positions[0],origin,size),i,a.completed[i]);
            }
        }
        if marked {
            warning(
                d,
                self.view
                    .screen(scene.threats.hazard_position(), origin, size),
                false,
            );
            kenny_marker(
                d,
                self.view.screen(scene.kenny.position(), origin, size),
                false,
            );
        }
        unsafe {
            ffi::EndScissorMode();
        }
        d.draw_rectangle_lines_ex(
            Rectangle::new(origin.x, origin.y, size.x, size.y),
            2.0,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            tr("Drag: move | Wheel: zoom | R: player | M / Esc: close", "Arrastrar: mover | Rueda: zoom | R: jugador | M / Esc: cerrar"),
            24,
            h - 40,
            18,
            Color::WHITE,
        );
        d.draw_text(
            tr("Blue: player | Orange K: Kenny | Red !: danger | Colored !: encounters", "Azul: jugador | Naranja K: Kenny | Rojo !: peligro | ! de colores: encuentros"),
            24,
            h - 20,
            16,
            Color::LIGHTGRAY,
        );
    }
}
fn arrow(d: &mut RaylibDrawHandle, p: Vector2, yaw: f32, color: Color) {
    let forward = Vector2::new(yaw.cos(), yaw.sin());
    let right = Vector2::new(-forward.y, forward.x);
    d.draw_circle_v(p, 9.0, Color::new(240, 248, 255, 235));
    d.draw_triangle(
        p + forward * 10.0,
        p - forward * 6.0 - right * 6.0,
        p - forward * 6.0 + right * 6.0,
        color,
    );
}
fn encounter_marker(d: &mut RaylibDrawHandle, p: Vector2, i: usize, done: bool) {
    d.draw_circle_v(p,8.0,if done { Color::GREEN } else { crate::minigames::southpark::encounters::COLORS[i] });
    if done {
        d.draw_line_ex(p+Vector2::new(-4.0,0.0),p+Vector2::new(-1.0,3.0),2.0,Color::WHITE);
        d.draw_line_ex(p+Vector2::new(-1.0,3.0),p+Vector2::new(5.0,-4.0),2.0,Color::WHITE);
    } else { d.draw_text("!",p.x as i32-2,p.y as i32-6,14,Color::BLACK); }
}
fn kenny_marker(d: &mut RaylibDrawHandle, p: Vector2, edge: bool) {
    d.draw_circle_v(p, 9.0, Color::new(245, 134, 25, 255));
    d.draw_text("K", p.x as i32 - 5, p.y as i32 - 7, 16, Color::BLACK);
    if edge {
        d.draw_circle_lines(p.x as i32, p.y as i32, 11.0, Color::ORANGE);
    }
}
fn warning(d: &mut RaylibDrawHandle, p: Vector2, _edge: bool) {
    d.draw_circle_lines(p.x as i32, p.y as i32, 13.0, Color::RED);
    d.draw_text("!", p.x as i32 + 9, p.y as i32 - 16, 20, Color::RED);
}
