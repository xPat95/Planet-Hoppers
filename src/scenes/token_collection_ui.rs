//! PNG medals rendered with the solar HUD's fonts, in the shared holographic style (`holo`): glass panels, glowing
//! titles, and medals that glow in their franchise's color, with a glint around the rim and a slow coin wobble.
use super::token_collection::{slot_index, CollectionLayout, TokenPresentation, PANEL_H, PANEL_W, TOKENS};
use crate::gameplay::progress::{Franchise, ProgressState};
use raylib::prelude::*;
use std::path::Path;

const INK: Color = Color::new(223, 238, 255, 255);
const MUTED: Color = Color::new(121, 146, 170, 255);
const CYAN: Color = Color::new(137, 228, 255, 255);
/// El acento (antes dorado): el blanco hielo del estilo holográfico.
const ACCENT: Color = holo::ACCENT;

use super::holo::{self, fade};

/// El color de cada franquicia: el de su logo (también la distancia de su planeta en el sistema solar).
pub fn franchise_color(franchise: Franchise) -> Color {
    match franchise {
        Franchise::Halloween => Color::new(255, 40, 150, 255),
        Franchise::StarWars => Color::new(255, 196, 40, 255),
        Franchise::SmilingFriends => Color::new(247, 150, 205, 255),
        Franchise::Minecraft => Color::new(215, 215, 222, 255),
        Franchise::RegularShow => Color::new(240, 240, 240, 255),
        Franchise::SouthPark => Color::new(205, 150, 105, 255),
    }
}
fn smooth(t: f32) -> f32 { let t = t.clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
fn centered(d: &mut impl RaylibDraw, font: &Font, label: &str, center: f32, y: f32, size: f32, color: Color) {
    let size = size.max(12.0);
    d.draw_text_ex(font, label, Vector2::new(center - font.measure_text(label, size, 1.0).x * 0.5, y), size, 1.0, color);
}

pub struct TokenTextures { medals: Vec<Texture2D> }

impl TokenTextures {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut medals = Vec::new();
        for art in &TOKENS {
            let texture = rl.load_texture(thread, &root.join(art.png).to_string_lossy())?;
            texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            medals.push(texture);
        }
        Ok(Self { medals })
    }

    fn medal(&self, d: &mut impl RaylibDraw, index: usize, center: Vector2, size: f32, rotation: f32, tint: Color) {
        self.medal_turned(d, index, center, size, rotation, tint, 1.0);
    }

    /// Como `medal`, angosta de costado (`turn`, de 0 a 1): la moneda girando apenas.
    #[allow(clippy::too_many_arguments)]
    fn medal_turned(&self, d: &mut impl RaylibDraw, index: usize, center: Vector2, size: f32, rotation: f32, tint: Color, turn: f32) {
        let texture = &self.medals[index];
        let scale = size / texture.width.max(texture.height) as f32;
        let (w, h) = (texture.width as f32 * scale * turn, texture.height as f32 * scale);
        d.draw_texture_pro(texture,
            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
            Rectangle::new(center.x, center.y, w, h), Vector2::new(w * 0.5, h * 0.5), rotation, tint);
    }

    pub fn hud_button(width: i32, height: i32) -> Rectangle {
        Rectangle::new(18.0, (height - 106) as f32, (width as f32 * 0.36).min(290.0), 86.0)
    }

    pub fn draw_hud(&self, d: &mut impl RaylibDraw, font: &Font, progress: &ProgressState, width: i32, height: i32) {
        let rect = Self::hud_button(width, height);
        holo::panel(d, rect, CYAN, 1.0);
        // Seis medallitas: las ganadas, la moneda chiquita con el brillo de su franquicia; las otras, huecos
        let step = (rect.width - 92.0) / 5.0;
        for (i, art) in TOKENS.iter().enumerate() {
            let c = Vector2::new(rect.x + 24.0 + i as f32 * step, rect.y + 29.0);
            let color = franchise_color(art.franchise);
            if progress.is_completed(art.franchise) {
                d.draw_circle_gradient(c.x as i32, c.y as i32, 20.0, fade(color, 0.4), fade(color, 0.0));
                self.medal(d, i, c, 26.0, 0.0, Color::WHITE);
            } else {
                d.draw_circle_v(c, 11.0, Color::new(2, 8, 18, 255));
                d.draw_ring(c, 10.5, 12.0, 0.0, 360.0, 32, Color::new(52, 86, 112, 255));
            }
        }
        holo::glow_text(d, font, &format!("{}/6", progress.token_count()), Vector2::new(rect.x + rect.width - 50.0, rect.y + 16.0),
            24.0, Color::WHITE, CYAN);
        let chip = holo::key_chip(d, font, "T", Vector2::new(rect.x + 14.0, rect.y + 56.0), 16.0, 1.0);
        d.draw_text_ex(font, "MEDALLERO", Vector2::new(rect.x + 22.0 + chip, rect.y + 56.0), 17.0, 1.0, CYAN);
    }

    pub fn draw_collection(&self, d: &mut impl RaylibDraw, regular: &Font, bold: &Font, progress: &ProgressState,
        width: i32, height: i32, opacity: f32, concealed: Option<Franchise>) {
        let l = CollectionLayout::new(width as f32, height as f32);
        let s = l.scale;
        let panel = Rectangle::new(l.x, l.y, PANEL_W * s, PANEL_H * s);
        d.draw_rectangle(0, 0, width, height, fade(Color::new(1, 6, 20, 210), opacity));
        holo::panel(d, panel, CYAN, opacity);
        d.draw_rectangle_rec(Rectangle::new(l.x + 26.0 * s, l.y + 22.0 * s, 72.0 * s, 3.0 * s), fade(ACCENT, opacity));
        let title = "MEDALLERO GALÁCTICO";
        let size = 36.0 * s;
        let tw = bold.measure_text(title, size, 1.0).x;
        holo::glow_text(d, bold, title, Vector2::new(width as f32 * 0.5 - tw * 0.5, l.y + 33.0 * s), size,
            fade(Color::WHITE, opacity), CYAN);
        // El progreso: seis segmentos, encendidos del color de cada franquicia ganada
        let seg = 30.0 * s;
        for (i, art) in TOKENS.iter().enumerate() {
            let won = progress.is_completed(art.franchise) && concealed != Some(art.franchise);
            let r = Rectangle::new(panel.x + panel.width - 28.0 * s - (6 - i) as f32 * (seg + 6.0 * s), l.y + 24.0 * s, seg, 5.0 * s);
            d.draw_rectangle_rounded(r, 1.0, 4, if won { fade(franchise_color(art.franchise), opacity) }
                else { fade(Color::new(40, 66, 90, 255), opacity) });
        }
        let heading = if progress.is_complete() { "6/6 TOKENS · COLECCIÓN COMPLETA".to_owned() }
            else { format!("{}/6 TOKENS · CADA VICTORIA DEJA SU HUELLA", progress.token_count()) };
        centered(d, bold, &heading, width as f32 * 0.5, l.y + 82.0 * s, 20.0 * s, fade(ACCENT, opacity));
        holo::divider(d, panel.x + 80.0 * s, panel.x + panel.width - 80.0 * s, l.y + 114.0 * s, CYAN, opacity * 0.7);
        let t = holo::now();
        for (i, art) in TOKENS.iter().enumerate() {
            let [_, y, _, _] = l.slot(i);
            let acquired = progress.is_completed(art.franchise) && concealed != Some(art.franchise);
            let [cx, cy] = l.medal_center(i);
            let c = Vector2::new(cx, cy);
            if acquired {
                self.shining_medal(d, i, c, 112.0 * s, franchise_color(art.franchise), opacity, t + i as f32 * 1.3);
                centered(d, bold, art.name, cx, y + 137.0 * s, 23.0 * s, fade(INK, opacity));
            } else {
                // Un hueco tallado, con la sombra de una moneda apenas visible: no muestra el dibujo ni de quién es
                d.draw_circle_v(c, 58.0 * s, fade(Color::new(2, 8, 18, 255), opacity));
                d.draw_circle_gradient(cx as i32, cy as i32, 58.0 * s, fade(Color::new(0, 0, 0, 0), opacity),
                    fade(Color::new(0, 0, 0, 200), opacity));
                self.medal(d, i, c, 104.0 * s, 0.0, fade(Color::new(6, 13, 22, 255), opacity * 0.55));
                d.draw_ring(c, 57.0 * s, 59.5 * s, 0.0, 360.0, 64, fade(Color::new(52, 86, 112, 255), opacity));
                d.draw_ring(c, 61.0 * s, 62.0 * s, 0.0, 360.0, 64, fade(CYAN, opacity * 0.18));
                centered(d, bold, "???", cx, y + 137.0 * s, 21.0 * s, fade(MUTED, opacity));
            }
        }
        centered(d, regular, "T / Esc: cerrar · Gana minijuegos para llenar tu colección.",
            width as f32 * 0.5, l.y + (PANEL_H - 30.0) * s, 17.0 * s, fade(MUTED, opacity));
    }

    /// Una medalla ganada: brilla del color de su franquicia (latiendo apenas), se mece como una moneda que gira un
    /// poco, un destello recorre el borde cada tanto y titila una estrellita arriba a la izquierda.
    #[allow(clippy::too_many_arguments)]
    fn shining_medal(&self, d: &mut impl RaylibDraw, index: usize, c: Vector2, size: f32, color: Color, opacity: f32, t: f32) {
        let r = size * 0.5;
        let pulse = 0.85 + 0.15 * (t * 2.2).sin();
        d.draw_circle_gradient(c.x as i32, c.y as i32, r * 1.45, fade(color, opacity * 0.42 * pulse), fade(color, 0.0));
        d.draw_ring(c, r * 1.02, r * 1.08, 0.0, 360.0, 64, fade(color, opacity * 0.55));
        let turn = 0.9 + 0.1 * (t * 1.1).cos();
        self.medal_turned(d, index, c, size, 2.0 * (t * 0.8).sin(), fade(Color::WHITE, opacity), turn);
        // El destello: un arco claro que da la vuelta por el borde y se apaga (una vez cada pocos segundos)
        let cycle = (t / 3.2).fract();
        if cycle < 0.35 {
            let k = cycle / 0.35;
            let a = -120.0 + k * 300.0;
            let glint = (std::f32::consts::PI * k).sin();
            d.draw_ring(c, r * 0.9, r * 1.0, a, a + 55.0, 24, fade(Color::WHITE, opacity * 0.55 * glint));
        }
        // La estrellita: cuatro puntas que crecen y se apagan
        let tw = ((t * 0.9 + 0.4).fract() * 2.0 - 1.0).abs();
        let star = (1.0 - tw).powi(3);
        if star > 0.02 {
            let p = c + Vector2::new(-r * 0.5, -r * 0.55);
            let len = r * 0.32 * star;
            for (dx, dy) in [(1.0, 0.0), (0.0, 1.0)] {
                let v = Vector2::new(dx, dy) * len;
                d.draw_line_ex(p - v, p + v, 2.0, fade(Color::WHITE, opacity * star));
            }
            d.draw_circle_v(p, 2.5 * star + 0.5, fade(Color::WHITE, opacity * star));
        }
    }

    pub fn draw_ceremony(&self, d: &mut impl RaylibDraw, regular: &Font, bold: &Font,
        progress: &ProgressState, presentation: &TokenPresentation, width: i32, height: i32) {
        let Some(notice) = presentation.active() else { return; };
        let t = presentation.elapsed();
        let index = slot_index(notice.franchise);
        let art = &TOKENS[index];
        let l = CollectionLayout::new(width as f32, height as f32);
        let board = smooth((t - 1.8) / 0.5);
        let end = 1.0 - smooth((t - 3.35) / 0.25);
        let flight = smooth((t - 2.1) / 0.9);
        let reveal = smooth(t / 0.45);
        if board > 0.0 {
            self.draw_collection(d, regular, bold, progress, width, height, board * end,
                (flight < 1.0).then_some(notice.franchise));
        } else {
            d.draw_rectangle(0, 0, width, height, fade(Color::new(1, 6, 20, 210), reveal));
        }
        let center = Vector2::new(width as f32 * 0.5, height as f32 * 0.5);
        let [tx, ty] = l.medal_center(index);
        let position = center + (Vector2::new(tx, ty) - center) * flight;
        let large = (height as f32 * 0.34).min(270.0);
        let size = (large + (112.0 * l.scale - large) * flight) * (0.35 + 0.65 * reveal);
        let rotation = -34.0 * (1.0 - reveal) + 7.0 * (t * 2.2).sin() * (1.0 - flight) * reveal;
        d.draw_circle_gradient(position.x as i32, position.y as i32, size * 0.83,
            fade(ACCENT, reveal * end * (0.22 + 0.06 * (t * 5.0).sin())), fade(ACCENT, 0.0));
        self.medal(d, index, position, size, rotation, fade(Color::WHITE, reveal * end));
        let text_opacity = reveal * (1.0 - board);
        // El título va arriba del contador, separado según el tamaño de la letra (no se encima)
        let count_y = center.y - large * 0.5 - 12.0 - 23.0 * l.scale;
        centered(d, bold, "¡TOKEN OBTENIDO!", center.x, count_y - 50.0 * l.scale, 38.0 * l.scale, fade(ACCENT, text_opacity));
        centered(d, regular, &format!("Has conseguido el token de {}", art.name), center.x,
            center.y + large * 0.5 + 19.0, 25.0 * l.scale, fade(INK, text_opacity));
        centered(d, bold, &format!("{}/6 TOKENS", progress.token_count()), center.x,
            count_y, 23.0 * l.scale, fade(INK, text_opacity));
    }

    pub fn draw_ship_notice(&self, d: &mut impl RaylibDraw, font: &Font,
        presentation: &TokenPresentation, width: i32, height: i32) {
        let Some(notice) = presentation.toast() else { return; };
        let t = presentation.elapsed();
        let alpha = smooth(t / 0.3) * (1.0 - smooth((t - 3.5) / 0.5));
        let w = (width as f32 - 24.0).min(360.0);
        let rect = Rectangle::new(width as f32 - w - 12.0 + (1.0-alpha)*w, height as f32 - 158.0, w, 76.0);
        holo::panel(d, rect, CYAN, alpha);
        d.draw_rectangle_rec(Rectangle::new(rect.x,rect.y,4.0,rect.height), fade(ACCENT,alpha));
        d.draw_text_ex(font, "NUEVA NAVE DISPONIBLE", Vector2::new(rect.x+18.0,rect.y+12.0), 20.0,1.0,fade(ACCENT,alpha));
        let name = notice.ship_name.as_deref().unwrap_or("");
        let size = 19.0f32.min(19.0 * (w-36.0) / font.measure_text(name,19.0,1.0).x.max(1.0));
        d.draw_text_ex(font, name, Vector2::new(rect.x+18.0,rect.y+42.0),size,1.0,fade(INK,alpha));
    }
}
