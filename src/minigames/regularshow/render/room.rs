use super::*;

pub(super) struct RoomArt {
    wall: Texture2D,
    friends: [Texture2D; 6],
    hammer: Texture2D,
    punch: Texture2D,
}

fn cutout(
    d: &mut impl RaylibDraw,
    image: &Texture2D,
    feet: Vector2,
    h: f32,
    flip: bool,
    tint: Color,
) {
    let width = image.width as f32;
    let height = image.height as f32;
    let w = h * width / height;
    d.draw_texture_pro(
        image,
        Rectangle::new(0.0, 0.0, if flip { -width } else { width }, height),
        Rectangle::new(feet.x - w * 0.5, feet.y - h, w, h),
        Vector2::zero(),
        0.0,
        tint,
    );
}

impl RoomArt {
    pub(super) fn load(
        texture: &mut impl FnMut(&[u8]) -> Result<Texture2D, String>,
    ) -> Result<Self, String> {
        Ok(Self {
            wall: texture(include_bytes!(
                "../../../../assets/minigames/regularshow/sala/fondo.png"
            ))?,
            hammer: texture(include_bytes!(
                "../../../../assets/minigames/regularshow/sala/martillo.png"
            ))?,
            punch: texture(include_bytes!(
                "../../../../assets/minigames/regularshow/sala/golpe.png"
            ))?,
            friends: [
                texture(include_bytes!(
                    "../../../../assets/minigames/regularshow/sala/mordecai.png"
                ))?,
                texture(include_bytes!(
                    "../../../../assets/minigames/regularshow/sala/rigby.png"
                ))?,
                texture(include_bytes!(
                    "../../../../assets/minigames/regularshow/sala/pops.png"
                ))?,
                texture(include_bytes!(
                    "../../../../assets/minigames/regularshow/sala/musculoso.png"
                ))?,
                texture(include_bytes!(
                    "../../../../assets/minigames/regularshow/sala/skips.png"
                ))?,
                texture(include_bytes!(
                    "../../../../assets/minigames/regularshow/sala/fantasma.png"
                ))?,
            ],
        })
    }

    pub(super) fn title_cast(&self, d: &mut impl RaylibDraw) {
        cutout(
            d,
            &self.friends[0],
            Vector2::new(182.0, 359.0),
            147.0,
            false,
            Color::WHITE,
        );
        cutout(
            d,
            &self.friends[1],
            Vector2::new(291.0, 359.0),
            119.0,
            false,
            Color::WHITE,
        );
        cutout(
            d,
            &self.hammer,
            Vector2::new(511.0, 359.0),
            145.0,
            true,
            Color::WHITE,
        );
    }

    pub(super) fn background(&self, d: &mut impl RaylibDraw, g: &Game) {
        d.draw_texture_pro(
            &self.wall,
            Rectangle::new(0.0, 0.0, self.wall.width as f32, self.wall.height as f32),
            Rectangle::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32),
            Vector2::zero(),
            0.0,
            if g.intro_left > 0.0 {
                Color::WHITE
            } else {
                c(197, 195, 224)
            },
        );
        // A CRT sitting directly on the carpet, with its console and controller cables.
        let x = TV_X as i32;
        d.draw_ellipse(x + 5, 319, 76.0, 7.0, Color::new(34, 28, 49, 85));
        for (dx, dy) in [(-12.0, -8.0), (27.0, -20.0)] {
            line(
                d,
                (TV_X, 223.0),
                (TV_X + dx, 207.0 + dy),
                2.5,
                c(63, 60, 64),
            );
        }
        let case = Rectangle::new(TV_X - 54.0, 221.0, 117.0, 95.0);
        d.draw_rectangle_rounded(case, 0.14, 8, c(43, 37, 46));
        d.draw_rectangle_rounded(
            Rectangle::new(case.x + 2.0, 223.0, 112.0, 82.0),
            0.13,
            8,
            c(110, 102, 99),
        );
        d.draw_rectangle_rounded(
            Rectangle::new(case.x + 6.0, 227.0, 90.0, 72.0),
            0.23,
            8,
            c(31, 36, 47),
        );
        for y in [239, 253] {
            d.draw_circle(x + 51, y, 5.0, c(48, 45, 55));
            line(
                d,
                (TV_X + 49.0, y as f32 + 2.0),
                (TV_X + 53.0, y as f32 - 2.0),
                1.4,
                c(175, 171, 163),
            );
        }
        for y in (268..292).step_by(3) {
            d.draw_line(x + 45, y, x + 56, y, c(48, 45, 55));
        }
        let lit = g.intro_left == 0.0 || ROOM_INTRO - g.intro_left < 0.75;
        if lit && g.phase != Phase::ArcadeClear {
            screen(d, g.elapsed, g.intro_left == 0.0);
        }
        line(d, (TV_X - 56.0, 315.0), (TV_X - 66.0, 321.0), 2.0, INK);
        line(d, (TV_X - 66.0, 321.0), (TV_X - 95.0, 317.0), 2.0, INK);
        block(d, TV_X - 114.0, 311.0, 48.0, 9.0, c(63, 58, 64));
        rect(d, TV_X - 105.0, 307.0, 28.0, 5.0, c(56, 50, 61));
        line(d, (TV_X - 92.0, 321.0), (TV_X - 108.0, 330.0), 2.0, INK);
        line(d, (TV_X - 108.0, 330.0), (TV_X - 127.0, 329.0), 2.0, INK);
        block(d, TV_X - 141.0, 326.0, 16.0, 6.0, c(67, 62, 70));
    }

    pub(super) fn intro(&self, d: &mut impl RaylibDraw, g: &Game) {
        let t = ROOM_INTRO - g.intro_left;
        if t < 0.75 {
            return;
        }
        let reveal = ((t - 1.5) / 1.9).clamp(0.0, 1.0);
        rect(
            d,
            0.0,
            0.0,
            960.0,
            540.0,
            Color::new(12, 13, 28, (255.0 - 190.0 * reveal) as u8),
        );
        if t >= 1.5 {
            screen(d, g.elapsed, true);
            let p = ((t - 1.85) / (ROOM_INTRO - 1.85)).clamp(0.0, 1.0);
            if p > 0.0 {
                cutout(
                    d,
                    &self.hammer,
                    Vector2::new(TV_X - 3.0 - 27.0 * p, 265.0 + 60.0 * p),
                    201.0 * p,
                    true,
                    Color::WHITE,
                );
            }
        }
    }

    pub(super) fn visitor(&self, d: &mut impl RaylibDraw, delivery: &Delivery, time: f32) {
        let left = delivery.until - time;
        let opacity = (left / 0.22).clamp(0.0, 1.0);
        let offset = if delivery.x < 480.0 { -48.0 } else { 48.0 };
        let feet = Vector2::new(delivery.x + offset, delivery.y - 8.0);
        d.draw_ellipse(
            feet.x as i32,
            feet.y as i32,
            22.0,
            4.0,
            Color::new(38, 30, 39, (55.0 * opacity) as u8),
        );
        cutout(
            d,
            &self.friends[delivery.helper],
            feet,
            if delivery.helper == 1 { 85.0 } else { 111.0 },
            offset > 0.0,
            Color::new(255, 255, 255, (opacity * 255.0) as u8),
        );
    }

    pub(super) fn fighter(&self, d: &mut impl RaylibDraw, f: &Fighter, row: usize, tint: Color) {
        let boss = f.kind == Kind::Hammer;
        let striking = matches!(f.action, Action::Strike { fired: true, .. });
        let image = if boss {
            if striking {
                &self.punch
            } else {
                &self.hammer
            }
        } else {
            &self.friends[row]
        };
        let mut height = if boss {
            201.0
        } else if row == 0 {
            144.0
        } else {
            103.0
        };
        let mut shift = 0.0;
        let mut lift = 0.0;
        match f.action {
            Action::Strike {
                fired: false,
                time,
                attack,
            } => {
                let wind = (time / windup(f.kind, attack, f.enraged())).min(1.0);
                shift = -7.0 * wind;
                height *= 1.0 - 0.065 * wind;
            }
            Action::Strike { fired: true, .. } => shift = if boss { 23.0 } else { 12.0 },
            Action::Guard | Action::Dodge(_) => height *= 0.82,
            Action::Idle if f.walk > 0.0 => lift = 4.0 * f.walk.sin().abs(),
            _ => {}
        }
        // In the supplied pose Mordecai faces left. Rigby and the Hammer face right.
        let flip = if !boss && row == 0 {
            f.facing > 0.0
        } else {
            f.facing < 0.0
        };
        cutout(
            d,
            image,
            Vector2::new(f.x + shift * f.facing, f.y - f.jump - lift),
            height,
            flip,
            tint,
        );
    }

    pub(super) fn portrait(&self, d: &mut impl RaylibDraw, idx: usize, x: i32, y: i32) {
        let (image, cx, cy) = match idx {
            0 => (&self.friends[0], 0.54, 0.36),
            1 => (&self.friends[1], 0.60, 0.30),
            _ => (&self.hammer, 0.58, 0.17),
        };
        let w = image.width as f32;
        let h = image.height as f32;
        let side = w * 0.47;
        block(d, x as f32, y as f32, 38.0, 38.0, c(50, 61, 69));
        d.draw_texture_pro(
            image,
            Rectangle::new(w * cx - side * 0.5, h * cy - side * 0.5, side, side),
            Rectangle::new(x as f32, y as f32, 38.0, 38.0),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}

fn screen(d: &mut impl RaylibDraw, time: f32, haunted: bool) {
    let x = TV_X - 44.0;
    d.draw_rectangle_rounded(
        Rectangle::new(x, 231.0, 82.0, 63.0),
        0.20,
        12,
        if haunted {
            c(118, 222, 216)
        } else {
            c(204, 137, 87)
        },
    );
    if !haunted {
        rect(d, x + 7.0, 239.0, 26.0, 3.0, c(156, 210, 98));
        rect(d, x + 49.0, 239.0, 26.0, 3.0, c(156, 210, 98));
        rect(d, x + 26.0, 262.0, 7.0, 13.0, c(65, 54, 85));
        rect(d, x + 48.0, 254.0, 13.0, 22.0, c(63, 145, 155));
    }
    for i in 0..12 {
        let y = 234.0 + i as f32 * 4.7;
        line(
            d,
            (x + 3.0, y),
            (x + 79.0, y),
            1.0,
            Color::new(43, 68, 76, 48),
        );
    }
    let scan = 234.0 + (time * 33.0) % 56.0;
    line(
        d,
        (x + 3.0, scan),
        (x + 79.0, scan),
        2.0,
        Color::new(239, 255, 245, 98),
    );
}
