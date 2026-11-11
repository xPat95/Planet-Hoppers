//! Bloom del sistema solar: lo que brilla de verdad (el sol, la lava de Halloween, las ventanas, los motores de las
//! naves, las estrellas más fuertes) derrama una luz suave alrededor. La escena se dibuja en una textura; de ahí se
//! saca lo brillante a un cuarto de resolución, se desenfoca dos veces (horizontal y vertical) y se suma encima.
use raylib::prelude::*;

/// Desde qué brillo (0 a 1, el canal más fuerte) empieza a derramar luz, y cuánta luz se suma.
const THRESHOLD: f32 = 0.78;
const INTENSITY: f32 = 1.1;
/// Cuántas veces se desenfoca (más vueltas, halo más ancho).
const PASSES: usize = 2;

pub struct Bloom {
    pub scene: RenderTexture2D,
    a: RenderTexture2D,
    b: RenderTexture2D,
    bright: Shader,
    blur: Shader,
    dir: i32,
    composite: Shader,
    bloom_tex: i32,
    size: (i32, i32),
}

impl Bloom {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bright = rl.load_shader_from_memory(thread, None, Some(include_str!("shaders/bloom_bright.fs")));
        let blur = rl.load_shader_from_memory(thread, None, Some(include_str!("shaders/bloom_blur.fs")));
        if !bright.is_shader_valid() || !blur.is_shader_valid() {
            return Err("No se pudieron cargar los shaders del bloom".into());
        }
        let threshold = bright.get_shader_location("threshold");
        bright.set_shader_value(threshold, THRESHOLD);
        let dir = blur.get_shader_location("dir");
        let mut composite = rl.load_shader_from_memory(thread, None, Some(include_str!("shaders/bloom_composite.fs")));
        if !composite.is_shader_valid() {
            return Err("No se pudo cargar el shader del bloom".into());
        }
        let bloom_tex = composite.get_shader_location("bloom");
        let intensity = composite.get_shader_location("intensity");
        composite.set_shader_value(intensity, INTENSITY);
        let (scene, a, b) = Self::targets(rl, thread, width, height)?;
        Ok(Self { scene, a, b, bright, blur, dir, composite, bloom_tex, size: (width, height) })
    }

    fn targets(rl: &mut RaylibHandle, thread: &RaylibThread, w: i32, h: i32)
        -> Result<(RenderTexture2D, RenderTexture2D, RenderTexture2D), Box<dyn std::error::Error>> {
        let (qw, qh) = ((w / 4).max(1) as u32, (h / 4).max(1) as u32);
        let scene = rl.load_render_texture(thread, w.max(1) as u32, h.max(1) as u32)?;
        let a = rl.load_render_texture(thread, qw, qh)?;
        let b = rl.load_render_texture(thread, qw, qh)?;
        for t in [&a, &b] {
            t.texture().set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        }
        Ok((scene, a, b))
    }

    /// Antes de dibujar: las texturas del tamaño de la ventana (si cambió).
    pub fn prepare(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
        if self.size != (width, height) {
            (self.scene, self.a, self.b) = Self::targets(rl, thread, width, height)?;
            self.size = (width, height);
        }
        Ok(())
    }

    /// Dibuja la escena (ya dibujada en `scene`) en la pantalla, con el bloom encima.
    pub fn composite(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let (qw, qh) = (self.a.texture.width as f32, self.a.texture.height as f32);
        let flip = |t: &RenderTexture2D| Rectangle::new(0.0, 0.0, t.texture.width as f32, -(t.texture.height as f32));
        // Lo brillante, achicado
        {
            let mut t = d.begin_texture_mode(thread, &mut self.a);
            t.clear_background(Color::BLACK);
            let mut s = t.begin_shader_mode(&mut self.bright);
            s.draw_texture_pro(self.scene.texture(), flip(&self.scene), Rectangle::new(0.0, 0.0, qw, qh), Vector2::zero(), 0.0, Color::WHITE);
        }
        // Desenfocado, de ida y vuelta entre las dos texturas chicas
        for _ in 0..PASSES {
            for (horizontal, from_a) in [(true, true), (false, false)] {
                let dir = if horizontal { Vector2::new(1.0 / qw, 0.0) } else { Vector2::new(0.0, 1.0 / qh) };
                self.blur.set_shader_value(self.dir, dir);
                let (src, dst) = if from_a { (&self.a, &mut self.b) } else { (&self.b, &mut self.a) };
                let mut t = d.begin_texture_mode(thread, dst);
                t.clear_background(Color::BLACK);
                let mut s = t.begin_shader_mode(&mut self.blur);
                s.draw_texture_pro(src.texture(), flip(src), Rectangle::new(0.0, 0.0, qw, qh), Vector2::zero(), 0.0, Color::WHITE);
            }
        }
        // La escena con el brillo sumado (sobre todo alrededor de lo que brilla, ver bloom_composite.fs)
        // (raylib suelta la segunda textura al empezar el modo shader: se la da después de empezarlo)
        let (raw, glow, loc) = (*self.composite.as_ref(), self.a.texture, self.bloom_tex);
        let mut s = d.begin_shader_mode(&mut self.composite);
        unsafe { raylib::ffi::SetShaderValueTexture(raw, loc, glow) };
        s.draw_texture_pro(self.scene.texture(), flip(&self.scene), Rectangle::new(0.0, 0.0, w, h), Vector2::zero(), 0.0, Color::WHITE);
    }
}
