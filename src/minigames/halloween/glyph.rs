//! Reconocedor de glifos dibujados con el mouse: línea horizontal, V, círculo y rayo, de un solo trazo.
//!
//! Se evalúa al soltar el botón. El trazo se remuestrea a una cantidad fija de puntos y se mide: qué tan cerrado
//! queda, cuánto gira en total y cuántas esquinas tiene. Se acepta cualquier tamaño, posición y dirección de trazo,
//! y un poco de temblor; lo que no se parece claramente a ninguno se rechaza (sin castigo: el jugador reintenta).
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Glyph {
    /// Línea horizontal.
    Line,
    /// V (con la punta para abajo).
    Vee,
    /// Círculo (luna).
    Circle,
    /// Rayo: zigzag de dos quiebres.
    Bolt,
}

impl Glyph {
    pub const ALL: [Glyph; 4] = [Glyph::Line, Glyph::Vee, Glyph::Circle, Glyph::Bolt];

    /// Color de cada glifo (ayuda a leerlos): naranja, verde veneno, morado y amarillo rayo.
    pub fn color(self) -> (u8, u8, u8) {
        match self {
            Glyph::Line => (255, 138, 31),
            Glyph::Vee => (63, 214, 40),
            Glyph::Circle => (176, 92, 255),
            Glyph::Bolt => (255, 224, 74),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Glyph::Line => "line",
            Glyph::Vee => "V",
            Glyph::Circle => "circle",
            Glyph::Bolt => "lightning",
        }
    }
}

/// Puntos en que se remuestrea el trazo.
const SAMPLES: usize = 48;
/// Largo mínimo del trazo (en píxeles): lo más corto es un clic o un temblor, no un glifo.
pub const MIN_LENGTH: f32 = 40.0;

/// Reconoce un trazo (puntos de pantalla, con y hacia abajo). `None` si no se parece claramente a ningún glifo.
pub fn recognize(points: &[(f32, f32)]) -> Option<Glyph> {
    let length = path_length(points);
    if points.len() < 3 || length < MIN_LENGTH {
        return None;
    }
    let p = resample(points, SAMPLES, length);
    let (min_x, max_x, min_y, max_y) = bounds(&p);
    let (w, h) = (max_x - min_x, max_y - min_y);
    let diag = (w * w + h * h).sqrt().max(1e-3);
    let chord = dist(p[0], p[SAMPLES - 1]);

    // Círculo: termina cerca de donde empezó y gira casi una vuelta entera, parejo
    let turn = total_turning(&p);
    if chord < 0.35 * diag && turn.abs() > 1.45 * PI && w > 0.35 * h && h > 0.35 * w {
        return Some(Glyph::Circle);
    }

    // Línea: casi recta y más bien horizontal (hasta ~30°)
    if chord > 0.9 * length {
        let (dx, dy) = (p[SAMPLES - 1].0 - p[0].0, p[SAMPLES - 1].1 - p[0].1);
        return (dy.abs() < dx.abs() * 0.58).then_some(Glyph::Line);
    }

    let corners = corners(&p);
    match corners.len() {
        1 => {
            // V: dos brazos de largo parecido y la punta por debajo de los dos extremos
            let c = p[corners[0]];
            let (a, b) = (dist(p[0], c), dist(c, p[SAMPLES - 1]));
            let tip_low = c.1 > p[0].1 + 0.25 * h && c.1 > p[SAMPLES - 1].1 + 0.25 * h;
            (tip_low && a.min(b) > 0.35 * a.max(b)).then_some(Glyph::Vee)
        }
        2 | 3 => {
            // Rayo: zigzag que avanza en una dirección (más alto que ancho, o al menos no una V con temblor)
            let open = chord > 0.4 * diag;
            let zig = alternates(&p, &corners);
            (open && zig).then_some(Glyph::Bolt)
        }
        _ => None,
    }
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn path_length(p: &[(f32, f32)]) -> f32 {
    p.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn bounds(p: &[(f32, f32)]) -> (f32, f32, f32, f32) {
    p.iter().fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |(a, b, c, d), &(x, y)| {
        (a.min(x), b.max(x), c.min(y), d.max(y))
    })
}

/// `n` puntos a la misma distancia uno del otro a lo largo del trazo.
fn resample(points: &[(f32, f32)], n: usize, length: f32) -> Vec<(f32, f32)> {
    let step = length / (n - 1) as f32;
    let mut out = vec![points[0]];
    let mut acc = 0.0;
    let mut prev = points[0];
    let mut i = 1;
    while i < points.len() && out.len() < n {
        let cur = points[i];
        let d = dist(prev, cur);
        if d > 0.0 && acc + d >= step {
            let t = (step - acc) / d;
            let q = (prev.0 + t * (cur.0 - prev.0), prev.1 + t * (cur.1 - prev.1));
            out.push(q);
            prev = q;
            acc = 0.0;
        } else {
            acc += d;
            prev = cur;
            i += 1;
        }
    }
    while out.len() < n {
        out.push(*points.last().unwrap());
    }
    out
}

fn heading(a: (f32, f32), b: (f32, f32)) -> f32 {
    (b.1 - a.1).atan2(b.0 - a.0)
}

fn wrap(mut a: f32) -> f32 {
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

/// Cuánto gira el trazo en total (con signo): una vuelta entera son ±2π.
fn total_turning(p: &[(f32, f32)]) -> f32 {
    let h: Vec<f32> = p.windows(2).map(|w| heading(w[0], w[1])).collect();
    h.windows(2).map(|w| wrap(w[1] - w[0])).sum()
}

/// Índices de las esquinas: donde la dirección de antes y la de después cambian más de ~60°.
fn corners(p: &[(f32, f32)]) -> Vec<usize> {
    const W: usize = 4;
    let mut cand: Vec<(usize, f32)> = Vec::new();
    for i in W..p.len() - W {
        let before = heading(p[i - W], p[i]);
        let after = heading(p[i], p[i + W]);
        let turn = wrap(after - before).abs();
        if turn > 1.05 {
            cand.push((i, turn));
        }
    }
    // Una esquina por grupo (la que más gira)
    let mut out: Vec<usize> = Vec::new();
    let mut best: Option<(usize, f32)> = None;
    for (i, t) in cand {
        match best {
            Some((bi, bt)) if i - bi <= W => {
                if t > bt {
                    best = Some((i, t));
                }
            }
            Some((bi, _)) => {
                out.push(bi);
                best = Some((i, t));
            }
            None => best = Some((i, t)),
        }
    }
    if let Some((bi, _)) = best {
        out.push(bi);
    }
    out
}

/// En un zigzag las esquinas giran para lados alternados (izquierda, derecha, ...).
fn alternates(p: &[(f32, f32)], corners: &[usize]) -> bool {
    const W: usize = 4;
    let signs: Vec<f32> = corners
        .iter()
        .map(|&i| {
            let lo = i.saturating_sub(W);
            let hi = (i + W).min(p.len() - 1);
            wrap(heading(p[i], p[hi]) - heading(p[lo], p[i])).signum()
        })
        .collect();
    signs.windows(2).all(|w| w[0] != w[1])
}
