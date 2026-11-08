//! Minijuego de Halloween: la bruja de guardia baja el volumen de la fiesta dibujando hechizos.
//!
//! Todo pasa en la oscuridad, desde los ojos de la bruja. Criaturas (diablos, fantasmas y demonios de parálisis del
//! sueño) vienen corriendo desde lejos, sin frenar, sin que se las vea: recién cuando entran a `reveal_radius` las
//! agarra la luz fría del borde, y ahí hay que maldecirlas antes de que lleguen a su cara. Cada
//! una lleva su glifo; dibujar un glifo maldice a la más cercana que lo lleva: se derrite en un charco negro que se
//! hunde en el piso. Los glifos se aprenden antes, en la intro (ver `app`): acá hay dos oleadas, tres puntos de salud y una protección corta después de cada golpe. Este archivo es la
//! lógica, sin ventana (diseño en `docs/minigames/halloween-plan.md`): la pantalla dibuja lo que hay en `Session` y
//! le pasa el tiempo (`update`) y los glifos reconocidos (`cast`). La escena está en `render` y la ventana en `app`.
pub mod app;
pub mod glyph;
mod render;
mod sound;
mod story;

pub use app::run_visit;
pub use glyph::{recognize, Glyph};

impl crate::minigames::common::host::AttemptOutcome for Session {
    fn attempt_outcome(&self) -> Option<crate::minigames::common::host::MinigameResult> {
        use crate::minigames::common::host::MinigameResult;
        match self.phase {
            Phase::Victory => Some(MinigameResult::Victory),
            Phase::Defeat => Some(MinigameResult::Defeat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Devil,
    Ghost,
    /// Demonio de parálisis del sueño: silueta negra con ojos neón. Solo en la segunda oleada; lleva dos glifos.
    Demon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Wave(u8),
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    /// Avanza hacia la bruja.
    Approaching,
    /// Maldecida: se derrite en `spot` (donde estaba); cuando pasan `wait` segundos, el charco ya se hundió y se va
    /// de la escena.
    Calmed { spot: (f32, f32), wait: f32 },
    /// Llegó a la bruja, o terminó de alejarse calmada: se va de la escena.
    Gone,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub id: u32,
    pub kind: Kind,
    /// Glifos que le faltan, en orden (el primero es el que se ve primero).
    pub glyphs: Vec<Glyph>,
    /// Posición en el piso (la bruja está en el origen; +y es hacia el fondo, lejos de la cámara).
    pub pos: (f32, f32),
    pub speed: f32,
    /// Es parte del trío final de la oleada: ojos encendidos, a saltos y en unísono, y si llega quita un corazón sin
    /// importar la protección.
    pub boss: bool,
    pub state: State,
}

impl Creature {
    pub fn distance(&self) -> f32 {
        (self.pos.0 * self.pos.0 + self.pos.1 * self.pos.1).sqrt()
    }
}

/// Valores de balance (objetivos de diseño; se ajustan después de probar).
#[derive(Debug, Clone)]
pub struct Balance {
    pub wave_time: [f32; 2],
    pub spawn_every: [f32; 2],
    pub spawn_radius: f32,
    /// Mitad del arco por donde aparecen, alrededor de adelante (en vueltas de media circunferencia): siempre a la vista.
    pub spawn_arc: f32,
    pub hit_radius: f32,
    /// Más lejos que esto no se las ve (ni se las puede maldecir): vienen en lo oscuro.
    pub reveal_radius: f32,
    /// Velocidad con que vienen corriendo, sin frenar, hasta su cara (m/s), en cada oleada.
    pub speed: [f32; 2],
    /// El demonio lleva dos glifos: viene a esta fracción de la velocidad de los demás.
    pub demon_pace: f32,
    /// El trío final de cada oleada: a qué velocidad viene (m/s, despacio porque son seis glifos), cuántos glifos lleva
    /// cada una, a qué distancia aparece y qué tan separadas (como `spawn_arc`).
    pub boss_speed: [f32; 2],
    pub boss_glyphs: usize,
    pub boss_radius: f32,
    pub boss_arc: f32,
    /// Cada cuántas criaturas maldecidas se recupera un corazón.
    pub life_every: u32,
    /// Cuánto tarda una maldecida en derretirse y hundirse en el piso.
    pub melt: f32,
    pub health: u8,
    pub protection: f32,
    /// La partida completa no pasa de este tiempo (sin contar la intro).
    pub max_time: f32,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            wave_time: [60.0, 65.0],
            spawn_every: [2.15, 1.7],
            spawn_radius: 18.0,
            spawn_arc: 0.15,
            hit_radius: 0.75,
            reveal_radius: 8.5,
            speed: [3.4, 3.8],
            demon_pace: 0.9,
            boss_speed: [1.15, 1.25],
            boss_glyphs: 2,
            boss_radius: 11.0,
            boss_arc: 0.1,
            life_every: 15,
            melt: 1.4,
            health: 3,
            protection: 1.2,
            max_time: 170.0,
        }
    }
}

pub struct Session {
    pub balance: Balance,
    pub phase: Phase,
    pub creatures: Vec<Creature>,
    pub health: u8,
    /// Tiempo que queda de protección después de un golpe.
    pub protected: f32,
    /// Tiempo en la fase actual y en toda la partida.
    pub phase_time: f32,
    pub total_time: f32,
    next_id: u32,
    spawn_clock: f32,
    rng: u64,
    calmed: u32,
    /// Ya salió el trío de esta oleada.
    boss_out: bool,
    /// Maldecidas desde el último corazón recuperado (con la salud llena se queda en `life_every`, y el próximo golpe
    /// se recupera con la próxima que se maldiga).
    pub life_meter: u32,
}

/// Resultado de un hechizo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cast {
    /// Le pegó a esta criatura; `calmed` si era su último glifo.
    Hit { id: u32, calmed: bool },
    /// Nadie lleva ese glifo ahora (sin castigo).
    Miss,
}

impl Session {
    pub fn new(balance: Balance, seed: u64) -> Self {
        let health = balance.health;
        Self {
            balance,
            phase: Phase::Wave(1),
            creatures: Vec::new(),
            health,
            protected: 0.0,
            phase_time: 0.0,
            total_time: 0.0,
            next_id: 0,
            spawn_clock: 0.0,
            rng: seed.max(1),
            calmed: 0,
            boss_out: false,
            life_meter: 0,
        }
    }

    /// Si ya salió de lo oscuro (y se la puede maldecir).
    pub fn seen(&self, c: &Creature) -> bool {
        c.distance() <= self.balance.reveal_radius
    }

    pub fn finished(&self) -> bool {
        matches!(self.phase, Phase::Victory | Phase::Defeat)
    }

    pub fn calmed_count(&self) -> u32 {
        self.calmed
    }

    fn rand(&mut self) -> f32 {
        // xorshift: misma semilla, misma partida (las pruebas lo necesitan)
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn update(&mut self, dt: f32) {
        if self.finished() {
            return;
        }
        self.phase_time += dt;
        self.total_time += dt;
        self.protected = (self.protected - dt).max(0.0);
        self.spawn(dt);
        self.advance(dt);
        self.next_phase();
    }

    fn wave(&self) -> Option<usize> {
        match self.phase {
            Phase::Wave(n) => Some(n as usize - 1),
            _ => None,
        }
    }

    fn spawn(&mut self, dt: f32) {
        let (every, phase_len) = match self.phase {
            Phase::Wave(n) => (self.balance.spawn_every[n as usize - 1], self.balance.wave_time[n as usize - 1] - 6.0),
            _ => return,
        };
        if self.phase_time > phase_len {
            // Al final de cada oleada no salen más: se terminan de resolver las que están, y viene el trío
            if !self.boss_out && !self.creatures.iter().any(|c| c.state == State::Approaching) {
                self.spawn_boss();
            }
            return;
        }
        self.spawn_clock -= dt;
        if self.spawn_clock > 0.0 {
            return;
        }
        self.spawn_clock = every * (0.8 + 0.4 * self.rand());
        let kind = match self.wave() {
            None | Some(0) => {
                if self.rand() < 0.55 { Kind::Devil } else { Kind::Ghost }
            }
            _ => match self.rand() {
                r if r < 0.35 => Kind::Demon,
                r if r < 0.7 => Kind::Ghost,
                _ => Kind::Devil,
            },
        };
        let mut glyphs = vec![Glyph::ALL[(self.rand() * 4.0) as usize % 4]];
        if kind == Kind::Demon {
            glyphs.push(Glyph::ALL[(self.rand() * 4.0) as usize % 4]);
        }
        // Aparecen adelante de la bruja (+y), dentro de lo que ve, lejos de las que ya vienen (no una dentro de otra)
        let taken: Vec<f32> = self.creatures.iter().filter(|c| c.state == State::Approaching)
            .map(|c| c.pos.1.atan2(c.pos.0)).collect();
        let mut a = 0.0;
        let mut best = -1.0;
        for _ in 0..6 {
            let cand = std::f32::consts::PI * (0.5 + self.balance.spawn_arc * (2.0 * self.rand() - 1.0));
            let gap = taken.iter().map(|t| (cand - t).abs()).fold(f32::MAX, f32::min);
            if gap > best {
                best = gap;
                a = cand;
            }
        }
        let r = self.balance.spawn_radius;
        // El demonio, con dos glifos, viene apenas más despacio
        let speed = self.balance.speed[self.wave().unwrap_or(0)] * (0.85 + 0.3 * self.rand())
            * if kind == Kind::Demon { self.balance.demon_pace } else { 1.0 };
        self.creatures.push(Creature {
            id: self.next_id,
            kind,
            glyphs,
            pos: (r * a.cos(), r * a.sin()),
            speed,
            boss: false,
            state: State::Approaching,
        });
        self.next_id += 1;
    }

    /// El trío final: tres fantasmas (primera oleada) o tres demonios (segunda), de frente y a la par.
    fn spawn_boss(&mut self) {
        self.boss_out = true;
        let wave = self.wave().unwrap_or(0);
        let kind = if wave == 0 { Kind::Ghost } else { Kind::Demon };
        let r = self.balance.boss_radius;
        for side in [-1.0f32, 0.0, 1.0] {
            let a = std::f32::consts::PI * (0.5 + self.balance.boss_arc * side);
            let glyphs = (0..self.balance.boss_glyphs).map(|_| Glyph::ALL[(self.rand() * 4.0) as usize % 4]).collect();
            self.creatures.push(Creature {
                id: self.next_id,
                kind,
                glyphs,
                pos: (r * a.cos(), r * a.sin()),
                speed: self.balance.boss_speed[wave],
                boss: true,
                state: State::Approaching,
            });
            self.next_id += 1;
        }
    }

    fn advance(&mut self, dt: f32) {
        let hit_r = self.balance.hit_radius;
        let mut hits = 0;
        let mut boss_hits = 0;
        for c in &mut self.creatures {
            match c.state {
                State::Approaching => {
                    let d = c.distance();
                    if d <= hit_r {
                        // Llegó a su cara
                        c.state = State::Gone;
                        if c.boss {
                            boss_hits += 1;
                        } else {
                            hits += 1;
                        }
                    } else {
                        // Se le viene encima corriendo, sin frenar
                        let step = (c.speed * dt).min(d);
                        c.pos.0 -= c.pos.0 / d * step;
                        c.pos.1 -= c.pos.1 / d * step;
                    }
                }
                State::Calmed { spot, wait } if wait > 0.0 => {
                    // Se derrite en el lugar
                    c.state = State::Calmed { spot, wait: wait - dt };
                }
                State::Calmed { .. } => {
                    // El charco ya se hundió: se va
                    c.state = State::Gone;
                }
                State::Gone => {}
            }
        }
        {
            for _ in 0..hits {
                if self.protected <= 0.0 && self.health > 0 {
                    self.health -= 1;
                    self.protected = self.balance.protection;
                }
            }
        }
        if boss_hits > 0 {
            self.health = self.health.saturating_sub(boss_hits);
            self.protected = self.balance.protection;
        }
        self.creatures.retain(|c| c.state != State::Gone);
        if self.health == 0 {
            self.phase = Phase::Defeat;
        }
    }

    fn next_phase(&mut self) {
        if self.finished() {
            return;
        }
        if !matches!(self.phase, Phase::Wave(_)) {
            return;
        }
        // La oleada termina cuando el trío ya salió y no queda nadie (ni un charco derritiéndose)
        let out_of_time = self.total_time >= self.balance.max_time;
        if (self.boss_out && self.creatures.is_empty()) || out_of_time {
            self.boss_out = false;
            self.phase = match self.phase {
                Phase::Wave(1) if !out_of_time => Phase::Wave(2),
                _ => Phase::Victory,
            };
            self.phase_time = 0.0;
            self.spawn_clock = 0.5;
            if self.phase == Phase::Victory {
                // La fiesta se queda: las que faltaban también se calman
                let ids: Vec<u32> = self.creatures.iter().filter(|c| c.state == State::Approaching).map(|c| c.id).collect();
                for id in ids {
                    self.calm(id);
                }
            }
        }
    }

    /// Lanza un hechizo con el glifo dibujado: le pega a la más cercana que lo lleva primero.
    pub fn cast(&mut self, glyph: Glyph) -> Cast {
        if self.finished() {
            return Cast::Miss;
        }
        let target = self
            .creatures
            .iter()
            .filter(|c| c.state == State::Approaching && self.seen(c) && c.glyphs.first() == Some(&glyph))
            .min_by(|a, b| a.distance().total_cmp(&b.distance()))
            .map(|c| c.id);
        let Some(id) = target else { return Cast::Miss };
        let c = self.creatures.iter_mut().find(|c| c.id == id).unwrap();
        c.glyphs.remove(0);
        let calmed = c.glyphs.is_empty();
        if calmed {
            self.calm(id);
            // Cada tantas maldecidas se recupera un corazón
            self.life_meter = (self.life_meter + 1).min(self.balance.life_every);
            if self.life_meter >= self.balance.life_every && self.health < self.balance.health {
                self.health += 1;
                self.life_meter = 0;
            }
        }
        Cast::Hit { id, calmed }
    }

    fn calm(&mut self, id: u32) {
        // Se derrite donde está
        if let Some(c) = self.creatures.iter_mut().find(|c| c.id == id) {
            c.glyphs.clear();
            c.state = State::Calmed { spot: c.pos, wait: self.balance.melt };
            self.calmed += 1;
        }
    }
}
