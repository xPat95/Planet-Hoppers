//! Pure 2D puzzle rules. No renderer, world mutation, audio handles or wall clocks.
use super::threats::ThreatKind;

pub const FADE_SECONDS: f32 = 0.32;
pub const HEATER_SOLUTION: [u8; 16] = [10, 14, 10, 10, 3, 5, 9, 6, 12, 3, 14, 10, 6, 9, 3, 10];
pub use super::murdoku::Case as Bus;
pub const SEWER_ROUTE: [(f32, f32); 6] = [
    (0.08, 0.19),
    (0.36, 0.19),
    (0.36, 0.48),
    (0.74, 0.48),
    (0.74, 0.80),
    (0.93, 0.80),
];
pub const HOLES: [f32; 12] = [
    0.07, 0.14, 0.22, 0.30, 0.38, 0.46, 0.54, 0.62, 0.70, 0.78, 0.86, 0.94,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rating {
    Perfect,
    Good,
    Miss,
}
impl Rating {
    pub fn judge(error: f32, perfect: f32, good: f32) -> Self {
        if error.abs() <= perfect {
            Self::Perfect
        } else if error.abs() <= good {
            Self::Good
        } else {
            Self::Miss
        }
    }
    pub fn label(self, es: bool) -> &'static str {
        match (self, es) {
            (Self::Perfect, false) => "PERFECT",
            (Self::Perfect, true) => "PERFECTO",
            (Self::Good, false) => "GOOD",
            (Self::Good, true) => "BIEN",
            (Self::Miss, false) => "MISS",
            (Self::Miss, true) => "FALLO",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Heaters {
    pub tiles: [u8; 16],
    pub powered: [bool; 16],
    pub destinations: [bool; 3],
}
impl Default for Heaters {
    fn default() -> Self {
        let mut s = Self {
            tiles: HEATER_SOLUTION,
            powered: [false; 16],
            destinations: [false; 3],
        };
        for (i, turns) in [1, 2, 1, 1, 3, 1, 2, 0, 1, 2, 1, 1, 2, 3, 2, 1]
            .into_iter()
            .enumerate()
        {
            for _ in 0..turns {
                s.tiles[i] = rotate(s.tiles[i]);
            }
        }
        s.recompute();
        s
    }
}
pub fn rotate(mask: u8) -> u8 {
    ((mask << 1) | (mask >> 3)) & 15
}
impl Heaters {
    pub fn recompute(&mut self) {
        self.powered = [false; 16];
        let mut stack = vec![];
        if self.tiles[0] & 8 != 0 {
            self.powered[0] = true;
            stack.push(0);
        }
        while let Some(i) = stack.pop() {
            for (dx, dy, out, back) in [(0, -1, 1, 4), (1, 0, 2, 8), (0, 1, 4, 1), (-1, 0, 8, 2)] {
                let (x, y) = ((i % 4) as i32 + dx, (i / 4) as i32 + dy);
                if !(0..4).contains(&x) || !(0..4).contains(&y) {
                    continue;
                }
                let j = (y * 4 + x) as usize;
                if !self.powered[j] && self.tiles[i] & out != 0 && self.tiles[j] & back != 0 {
                    self.powered[j] = true;
                    stack.push(j);
                }
            }
        }
        self.destinations = [3, 11, 15].map(|i| self.powered[i] && self.tiles[i] & 2 != 0);
    }
}
#[derive(Clone, Debug)]
pub struct Ufo {
    pub target: usize,
    pub reaction: f32,
    pub charge: f32,
    pub hits: u32,
    pub misses: u32,
    pub completed: usize,
    pub score: u32,
    random: u64,
}
impl Ufo {
    fn next(&mut self) {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        self.target = self.random as usize % 3;
        self.reaction = -0.35;
    }
    pub fn window(&self, _elapsed: f32) -> f32 {
        match self.completed {
            0..=5 => 1.2,
            6..=12 => 0.95,
            _ => 0.75,
        }
    }
    fn resolve(&mut self, rating: Rating) -> Option<bool> {
        match rating {
            Rating::Perfect => {
                self.score += 100;
                self.hits += 1;
                self.charge -= 0.08;
            }
            Rating::Good => {
                self.score += 60;
                self.hits += 1;
                self.charge -= 0.04;
            }
            Rating::Miss => {
                self.misses += 1;
                self.charge += 0.20;
            }
        }
        self.charge = self.charge.clamp(0., 1.);
        self.completed += 1;
        let result = if self.charge >= 1. {
            Some(false)
        } else if self.completed == 20 {
            Some(true)
        } else {
            None
        };
        self.next();
        result
    }
}
#[derive(Clone, Debug)]
pub struct Piano {
    pub notes: [usize; 7],
    pub round: usize,
    pub input: usize,
    pub errors: u8,
    pub demo_time: f32,
    pub demonstrating: bool,
}
impl Piano {
    pub fn length(&self) -> usize {
        [3, 4, 5, 5, 6, 6, 7, 7][self.round.min(7)]
    }
    pub fn illumination(&self) -> f32 {
        0.45
    }
    pub fn beat(&self) -> f32 {
        self.illumination() + 0.18
    }
    fn pattern(round: usize) -> [usize; 7] {
        let mut seed = 0x91a3_u64.wrapping_mul(round as u64 + 1);
        let mut notes = [0; 7];
        for i in 0..7 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            notes[i] = seed as usize % 5;
            if i > 0 && notes[i] == notes[i - 1] {
                notes[i] = (notes[i] + 1) % 5;
            }
        }
        notes
    }
    pub fn lit(&self) -> Option<usize> {
        if !self.demonstrating || self.demo_time < 0.45 {
            return None;
        }
        let time = self.demo_time - 0.45;
        let n = (time / self.beat()) as usize;
        (n < self.length() && time % self.beat() < self.illumination()).then(|| self.notes[n])
    }
}
#[derive(Clone, Debug)]
pub struct Sewers {
    pub open: [bool; 12],
    pub closed: [bool; 12],
    pub position: f32,
    pub target: usize,
    pub approach: f32,
    pub cooldown: f32,
}
impl Sewers {
    pub fn window(&self) -> f32 {
        match self.target {
            0..=2 => 1.4,
            3..=6 => 1.1,
            _ => 0.85,
        }
    }
    pub fn active(&self) -> bool {
        self.target < 12 && self.approach >= 0.
    }
}
#[derive(Clone, Debug)]
pub enum Board {
    Heaters(Heaters),
    Ufo(Ufo),
    Bus(Bus),
    Piano(Piano),
    Sewers(Sewers),
}
#[derive(Clone, Debug)]
pub struct PuzzleRun {
    pub kind: ThreatKind,
    pub group: usize,
    pub board: Board,
    pub transition: f32,
    pub started: bool,
    pub elapsed: f32,
    pub animation: f32,
    pub result: Option<bool>,
    pub result_time: f32,
    pub committed: bool,
    pub paused: bool,
    pub leaving: bool,
    pub feedback: f32,
    pub last_hit: Option<usize>,
    pub mistake: bool,
    pub rating: Option<Rating>,
    /// The native host samples Instant before input, so timing judgments and
    /// rendered rings use the same monotonic active-game clock.
    pub externally_clocked: bool,
}
impl PuzzleRun {
    pub fn new(kind: ThreatKind, group: usize) -> Self {
        let board = match kind {
            ThreatKind::Snowman => Board::Heaters(Heaters::default()),
            ThreatKind::Ufo => Board::Ufo(Ufo {
                target: 2,
                reaction: 0.0,
                charge: 0.25,
                hits: 0,
                misses: 0,
                completed: 0,
                score: 0,
                random: 0x5137a9,
            }),
            ThreatKind::Bus => Board::Bus(Bus::default()),
            ThreatKind::Piano => Board::Piano(Piano {
                notes: Piano::pattern(0),
                round: 0,
                input: 0,
                errors: 0,
                demo_time: 0.0,
                demonstrating: true,
            }),
            ThreatKind::Sewer => Board::Sewers(Sewers {
                open: [true; 12],
                closed: [false; 12],
                position: 0.0,
                target: 0,
                approach: 0.,
                cooldown: 0.,
            }),
        };
        Self {
            kind,
            group,
            board,
            transition: 0.0,
            started: false,
            elapsed: 0.0,
            animation: 0.0,
            result: None,
            result_time: 0.0,
            committed: false,
            paused: false,
            leaving: false,
            feedback: 0.0,
            last_hit: None,
            mistake: false,
            rating: None,
            externally_clocked: false,
        }
    }
    pub fn ready(&self) -> bool {
        self.transition >= FADE_SECONDS && !self.leaving
    }
    pub fn start(&mut self) {
        if self.ready() && self.result.is_none() {
            self.started = true;
        }
    }
    pub fn interactive(&self) -> bool {
        self.started && self.ready() && !self.paused && self.result.is_none()
    }
    pub fn restart(&mut self) {
        *self = Self::new(self.kind, self.group);
    }
    pub fn leave(&mut self) {
        self.leaving = true;
        self.transition = FADE_SECONDS;
        self.paused = false;
    }
    pub fn tick(&mut self, dt: f32) {
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        if self.leaving {
            self.transition = (self.transition - dt).max(0.0);
            return;
        }
        if self.paused {
            return;
        }
        self.animation += dt;
        if self.result.is_some() {
            self.result_time += dt;
        }
        self.feedback = (self.feedback - dt).max(0.0);
        if !self.ready() {
            self.transition = (self.transition + dt).min(FADE_SECONDS);
            return;
        }
        if !self.interactive() {
            return;
        }
        // Substeps make reaction deadlines independent of rendering frame rate.
        let mut left = dt;
        while left > 0.00001 && self.result.is_none() {
            let step = left.min(0.025);
            left -= step;
            self.elapsed += step;
            match &mut self.board {
                Board::Heaters(_) => {
                    if self.elapsed >= 30.0 {
                        self.result = Some(false);
                    }
                }
                Board::Ufo(u) => {
                    u.reaction += step;
                    if u.reaction > u.window(self.elapsed) + 0.140 {
                        let excess = u.reaction - u.window(self.elapsed) - 0.140;
                        self.result = u.resolve(Rating::Miss);
                        u.reaction += excess;
                        self.rating = Some(Rating::Miss);
                        self.feedback = 0.6;
                        self.mistake = true;
                    }
                }
                Board::Piano(p) => {
                    if p.demonstrating {
                        p.demo_time += step;
                        if p.demo_time >= 0.45 + p.length() as f32 * p.beat() {
                            p.demonstrating = false;
                            p.input = 0;
                        }
                    }
                }
                Board::Sewers(s) => {
                    s.cooldown = (s.cooldown - step).max(0.);
                    if s.target == 12 {
                        s.position = (s.position + step * 0.08).min(1.);
                        if s.position >= 1. {
                            self.result = Some(true);
                        }
                        continue;
                    }
                    s.approach += step;
                    let start = if s.target == 0 {
                        0.
                    } else {
                        HOLES[s.target - 1]
                    };
                    // Kenny reaches the lid 260 ms after coincidence: the full
                    // GOOD window remains available, including at low FPS.
                    let crossing = s.window() + 0.26;
                    s.position =
                        start + (HOLES[s.target] - start) * (s.approach / crossing).clamp(0., 1.);
                    if s.approach >= crossing {
                        if s.open[s.target] {
                            self.result = Some(false);
                        } else {
                            s.target += 1;
                            s.approach -= crossing;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    pub fn click(&mut self, index: usize) {
        if !self.interactive() {
            return;
        }
        if matches!(&self.board,Board::Piano(p) if p.demonstrating) {
            return;
        }
        if matches!(&self.board,Board::Ufo(u) if u.reaction<0.) {
            return;
        }
        self.last_hit = Some(index);
        self.feedback = 0.35;
        self.mistake = false;
        match &mut self.board {
            Board::Heaters(h) if index < 16 => {
                h.tiles[index] = rotate(h.tiles[index]);
                h.recompute();
                if h.destinations.iter().all(|x| *x) {
                    self.result = Some(true);
                }
            }
            Board::Ufo(u) if index < 3 && u.reaction >= 0. => {
                let rating = if index == u.target {
                    Rating::judge(u.reaction - u.window(self.elapsed), 0.065, 0.140)
                } else {
                    Rating::Miss
                };
                self.result = u.resolve(rating);
                self.rating = Some(rating);
                self.mistake = rating == Rating::Miss;
                self.feedback = 0.6;
            }
            Board::Piano(p) if index < 5 => {
                if index == p.notes[p.input] {
                    p.input += 1;
                    if p.input == p.length() {
                        if p.round == 7 {
                            self.result = Some(true);
                        } else {
                            p.round += 1;
                            p.errors = 0;
                            p.notes = Piano::pattern(p.round);
                            p.input = 0;
                            p.demo_time = 0.0;
                            p.demonstrating = true;
                        }
                    }
                } else {
                    p.errors += 1;
                    self.mistake = true;
                    p.input = 0;
                    p.demo_time = -0.40;
                    p.demonstrating = true;
                    if p.errors == 2 {
                        self.result = Some(false);
                    }
                }
            }
            Board::Sewers(s) if index < 12 && s.open[index] && s.cooldown <= 0. => {
                let rating = if index == s.target && s.active() {
                    Rating::judge(s.approach - s.window(), 0.09, 0.17)
                } else {
                    Rating::Miss
                };
                self.rating = Some(rating);
                self.feedback = 0.5;
                self.mistake = rating == Rating::Miss;
                if rating != Rating::Miss {
                    s.open[index] = false;
                    s.closed[index] = true;
                } else {
                    s.cooldown = 0.22;
                }
            }
            _ => {}
        }
    }
    pub fn confirm_bus(&mut self) {
        if !self.interactive() {
            return;
        }
        if let Board::Bus(b) = &self.board {
            if b.ready() {
                self.result = Some(b.correct());
            }
        }
    }
    pub fn select_bus_piece(&mut self, piece: usize) {
        if self.interactive() {
            if let Board::Bus(b) = &mut self.board {
                b.select(piece);
            }
        }
    }
    pub fn annotate_bus_cell(&mut self,cell:usize) {if self.interactive() {if let Board::Bus(b)=&mut self.board {b.annotate(cell);}}}
    pub fn toggle_bus_mode(&mut self) {if self.interactive() {if let Board::Bus(b)=&mut self.board {b.toggle_mode();}}}
    pub fn place_bus_piece(&mut self, cell: usize) {
        if self.interactive() {
            if let Board::Bus(b) = &mut self.board {
                b.place(cell);
            }
        }
    }
    pub fn remove_bus_piece(&mut self) {
        if self.interactive() {
            if let Board::Bus(b) = &mut self.board {
                b.remove();
            }
        }
    }
    pub fn accuse_bus_driver(&mut self, piece: usize) {
        if self.interactive() {
            if let Board::Bus(b) = &mut self.board {
                b.accuse(piece);
            }
        }
    }
    pub fn clear_bus(&mut self) {
        if self.interactive() {
            if let Board::Bus(b) = &mut self.board {
                b.clear();
            }
        }
    }
}
/// Constant-speed position along the illustrated street route (not a second world minimap).
pub fn route_point(progress: f32) -> (f32, f32) {
    let lengths: Vec<_> = SEWER_ROUTE
        .windows(2)
        .map(|p| (p[1].0 - p[0].0).abs() + (p[1].1 - p[0].1).abs())
        .collect();
    let mut distance = progress.clamp(0.0, 1.0) * lengths.iter().sum::<f32>();
    for (i, length) in lengths.into_iter().enumerate() {
        if distance <= length {
            let t = distance / length;
            let (a, b) = (SEWER_ROUTE[i], SEWER_ROUTE[i + 1]);
            return (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        }
        distance -= length;
    }
    SEWER_ROUTE[5]
}
