//! PNG medals rendered with the solar HUD's fonts, dark panels and luminous accents.
use super::token_collection::{slot_index, CollectionLayout, TokenPresentation, TOKENS};
use crate::gameplay::progress::{Franchise, ProgressState};
use raylib::prelude::*;
use std::path::Path;

const INK: Color = Color::new(223, 238, 255, 255);
const MUTED: Color = Color::new(121, 146, 170, 255);
const CYAN: Color = Color::new(137, 228, 255, 255);
const GOLD: Color = Color::new(255, 205, 91, 255);

fn fade(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a as f32 * opacity.clamp(0.0, 1.0)) as u8;
    color
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
        let texture = &self.medals[index];
        let scale = size / texture.width.max(texture.height) as f32;
        let (w, h) = (texture.width as f32 * scale, texture.height as f32 * scale);
        d.draw_texture_pro(texture,
            Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
            Rectangle::new(center.x, center.y, w, h), Vector2::new(w * 0.5, h * 0.5), rotation, tint);
    }

    pub fn hud_button(width: i32, height: i32) -> Rectangle {
        Rectangle::new(18.0, (height - 78) as f32, (width as f32 * 0.28).min(198.0), 58.0)
    }

    pub fn draw_hud(&self, d: &mut impl RaylibDraw, font: &Font, progress: &ProgressState, width: i32, height: i32) {
        let rect = Self::hud_button(width, height);
        d.draw_rectangle_rounded(rect, 0.16, 8, Color::new(5, 15, 33, 232));
        d.draw_rectangle_lines_ex(rect, 1.0, Color::new(55, 112, 142, 220));
        for (i, art) in TOKENS.iter().enumerate() {
            let x = rect.x + 14.0 + i as f32 * (rect.width - 62.0).min(128.0) / 5.0;
            let y = rect.y + 17.0;
            let acquired = progress.is_completed(art.franchise);
            if acquired {
                d.draw_circle_gradient(x as i32, y as i32, 10.0, fade(GOLD, 0.18), fade(GOLD, 0.0));
            }
            d.draw_circle_v(Vector2::new(x, y), 5.5, if acquired { GOLD } else { Color::new(2, 7, 14, 255) });
            d.draw_circle_lines(x as i32, y as i32, 5.5, if acquired { GOLD } else { Color::new(61, 86, 105, 255) });
        }
        d.draw_text_ex(font, &format!("{}/6", progress.token_count()), Vector2::new(rect.x + rect.width - 34.0, rect.y + 8.0), 18.0, 1.0, INK);
        d.draw_text_ex(font, "[T] MEDALLERO", Vector2::new(rect.x + 14.0, rect.y + 34.0), 15.0, 1.0, CYAN);
    }

    pub fn draw_collection(&self, d: &mut impl RaylibDraw, regular: &Font, bold: &Font, progress: &ProgressState,
        width: i32, height: i32, opacity: f32, concealed: Option<Franchise>) {
        let l = CollectionLayout::new(width as f32, height as f32);
        let s = l.scale;
        let panel = Rectangle::new(l.x, l.y, 900.0 * s, 620.0 * s);
        d.draw_rectangle(0, 0, width, height, fade(Color::new(1, 6, 20, 210), opacity));
        d.draw_rectangle_rounded(panel, 0.035, 8, fade(Color::new(8, 23, 43, 252), opacity));
        d.draw_rectangle_lines_ex(panel, 1.5, fade(CYAN, opacity * 0.6));
        d.draw_rectangle_rec(Rectangle::new(l.x + 26.0 * s, l.y + 22.0 * s, 72.0 * s, 3.0 * s), fade(GOLD, opacity));
        centered(d, bold, "MEDALLERO GALÁCTICO", width as f32 * 0.5, l.y + 35.0 * s, 34.0 * s, fade(INK, opacity));
        let heading = if progress.is_complete() { "6/6 TOKENS · COLECCIÓN COMPLETA".to_owned() }
            else { format!("{}/6 TOKENS · CADA VICTORIA DEJA SU HUELLA", progress.token_count()) };
        centered(d, bold, &heading, width as f32 * 0.5, l.y + 82.0 * s, 20.0 * s, fade(GOLD, opacity));
        for (i, art) in TOKENS.iter().enumerate() {
            let [_, y, _, _] = l.slot(i);
            let acquired = progress.is_completed(art.franchise) && concealed != Some(art.franchise);
            let [cx, cy] = l.medal_center(i);
            if acquired {
                d.draw_circle_gradient(cx as i32, cy as i32, 70.0 * s, fade(GOLD, opacity * 0.16), fade(GOLD, 0.0));
                self.medal(d, i, Vector2::new(cx, cy), 112.0 * s, 0.0, fade(Color::WHITE, opacity));
                centered(d, bold, art.name, cx, y + 137.0 * s, 23.0 * s, fade(INK, opacity));
            } else {
                // Unrevealed medals expose neither their artwork nor their identity.
                d.draw_circle_v(Vector2::new(cx, cy), 56.0 * s, fade(Color::new(3, 11, 23, 255), opacity));
                d.draw_circle_lines(cx as i32, cy as i32, 56.0 * s, fade(Color::new(46, 73, 94, 255), opacity));
            }
        }
        centered(d, regular, "T / Esc: cerrar · Gana minijuegos para llenar tu colección.",
            width as f32 * 0.5, l.y + 585.0 * s, 17.0 * s, fade(MUTED, opacity));
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
            fade(GOLD, reveal * end * (0.22 + 0.06 * (t * 5.0).sin())), fade(GOLD, 0.0));
        self.medal(d, index, position, size, rotation, fade(Color::WHITE, reveal * end));
        let text_opacity = reveal * (1.0 - board);
        centered(d, bold, "¡TOKEN OBTENIDO!", center.x, center.y - large * 0.5 - 84.0, 38.0 * l.scale, fade(GOLD, text_opacity));
        centered(d, regular, &format!("Has conseguido el token de {}", art.name), center.x,
            center.y + large * 0.5 + 19.0, 25.0 * l.scale, fade(INK, text_opacity));
        centered(d, bold, &format!("{}/6 TOKENS", progress.token_count()), center.x,
            center.y - large * 0.5 - 38.0, 23.0 * l.scale, fade(INK, text_opacity));
    }

    pub fn draw_ship_notice(&self, d: &mut impl RaylibDraw, font: &Font,
        presentation: &TokenPresentation, width: i32, height: i32) {
        let Some(notice) = presentation.toast() else { return; };
        let t = presentation.elapsed();
        let alpha = smooth(t / 0.3) * (1.0 - smooth((t - 3.5) / 0.5));
        let w = (width as f32 - 24.0).min(360.0);
        let rect = Rectangle::new(width as f32 - w - 12.0 + (1.0-alpha)*w, height as f32 - 158.0, w, 76.0);
        d.draw_rectangle_rounded(rect, 0.12, 8, fade(Color::new(8,23,43,245), alpha));
        d.draw_rectangle_lines_ex(rect, 1.0, fade(CYAN, alpha));
        d.draw_rectangle_rec(Rectangle::new(rect.x,rect.y,4.0,rect.height), fade(GOLD,alpha));
        d.draw_text_ex(font, "NUEVA NAVE DISPONIBLE", Vector2::new(rect.x+18.0,rect.y+12.0), 20.0,1.0,fade(GOLD,alpha));
        let name = notice.ship_name.as_deref().unwrap_or("");
        let size = 19.0f32.min(19.0 * (w-36.0) / font.measure_text(name,19.0,1.0).x.max(1.0));
        d.draw_text_ex(font, name, Vector2::new(rect.x+18.0,rect.y+42.0),size,1.0,fade(INK,alpha));
    }
}
