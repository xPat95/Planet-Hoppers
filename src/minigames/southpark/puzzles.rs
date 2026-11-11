//! Pure 2D puzzle rules. No renderer, world mutation, audio handles or wall clocks.
use super::threats::ThreatKind;

pub const FADE_SECONDS: f32 = 0.32;
pub const HEATER_SOLUTION: [u8; 16] = [10, 14, 10, 10, 3, 5, 9, 6, 12, 3, 14, 10, 6, 9, 3, 10];
pub const BUS_SOLUTION: [usize; 4] = [2, 0, 3, 1];
pub const SEWER_ROUTE: [(f32, f32); 6] = [
    (0.08, 0.19),
    (0.36, 0.19),
    (0.36, 0.48),
    (0.74, 0.48),
    (0.74, 0.80),
    (0.93, 0.80),
];
pub const HOLES: [f32; 7] = [0.14, 0.28, 0.41, 0.55, 0.69, 0.81, 0.92];

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
    random: u64,
}
impl Ufo {
    fn next(&mut self) {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        self.target = (self.target + 1 + (self.random as usize % 5)) % 6;
        self.reaction = 0.0;
    }
    pub fn window(&self, elapsed: f32) -> f32 {
        1.55 - 0.70 * (elapsed / 20.0).clamp(0.0, 1.0)
    }
}
#[derive(Clone, Debug)]
pub struct Bus {
    pub entered: Vec<usize>,
    pub attempts: u8,
    pub rejected: bool,
}
#[derive(Clone, Debug)]
pub struct Piano {
    pub notes: [usize; 6],
    pub round: usize,
    pub input: usize,
    pub errors: u8,
    pub demo_time: f32,
    pub demonstrating: bool,
}
impl Piano {
    pub fn length(&self) -> usize {
        self.round + 2
    }
    pub fn lit(&self) -> Option<usize> {
        if !self.demonstrating || self.demo_time < 0.45 {
            return None;
        }
        let time = self.demo_time - 0.45;
        let n = (time / 0.75) as usize;
        (n < self.length() && time % 0.75 < 0.50).then(|| self.notes[n])
    }
}
#[derive(Clone, Debug)]
pub struct Sewers {
    pub open: [bool; 7],
    pub closed: [bool; 7],
    pub position: f32,
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
}
impl PuzzleRun {
    pub fn new(kind: ThreatKind, group: usize) -> Self {
        let board = match kind {
            ThreatKind::Snowman => Board::Heaters(Heaters::default()),
            ThreatKind::Ufo => Board::Ufo(Ufo {
                target: 2,
                reaction: 0.0,
                charge: 0.15,
                hits: 0,
                misses: 0,
                random: 0x5137a9,
            }),
            ThreatKind::Bus => Board::Bus(Bus {
                entered: vec![],
                attempts: 3,
                rejected: false,
            }),
            ThreatKind::Piano => Board::Piano(Piano {
                notes: [1, 3, 0, 4, 2, 0],
                round: 0,
                input: 0,
                errors: 0,
                demo_time: 0.0,
                demonstrating: true,
            }),
            ThreatKind::Sewer => Board::Sewers(Sewers {
                open: [true, false, true, false, true, false, true],
                closed: [false; 7],
                position: 0.0,
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
                    if u.reaction >= u.window(self.elapsed) {
                        u.charge = (u.charge + 0.22).min(1.0);
                        u.misses += 1;
                        u.next();
                        self.feedback = 0.35;
                        self.mistake = true;
                    }
                    if u.charge >= 1.0 {
                        self.result = Some(false);
                    } else if self.elapsed >= 20.0 {
                        self.result = Some(true);
                    }
                }
                Board::Piano(p) => {
                    if p.demonstrating {
                        p.demo_time += step;
                        if p.demo_time >= 0.45 + p.length() as f32 * 0.75 {
                            p.demonstrating = false;
                            p.input = 0;
                        }
                    }
                }
                Board::Sewers(s) => {
                    s.position = (self.elapsed / 28.0).min(1.0);
                    for (i, at) in HOLES.iter().enumerate() {
                        if !s.closed[i] && self.elapsed >= at * 28.0 - 4.0 {
                            s.open[i] = true;
                        }
                        if s.position >= *at && s.open[i] {
                            self.result = Some(false);
                            s.position = *at;
                            break;
                        }
                    }
                    if self.result.is_none() && s.position >= 1.0 {
                        self.result = Some(true);
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
            Board::Ufo(u) if index < 6 => {
                if index == u.target {
                    u.hits += 1;
                    u.charge = (u.charge - 0.065).max(0.0);
                    u.next();
                } else {
                    u.charge = (u.charge + 0.10).min(1.0);
                    self.mistake = true;
                    if u.charge >= 1.0 {
                        self.result = Some(false);
                    }
                }
            }
            Board::Bus(b) if index < 4 => {
                if b.entered.contains(&index) {
                    return;
                }
                b.rejected = false;
                b.entered.push(index);
                if b.entered.len() == 4 {
                    if b.entered.as_slice() == BUS_SOLUTION {
                        self.result = Some(true);
                    } else {
                        b.attempts -= 1;
                        b.entered.clear();
                        b.rejected = true;
                        self.mistake = true;
                        if b.attempts == 0 {
                            self.result = Some(false);
                        }
                    }
                }
            }
            Board::Piano(p) if index < 5 => {
                if index == p.notes[p.input] {
                    p.input += 1;
                    if p.input == p.length() {
                        if p.round == 4 {
                            self.result = Some(true);
                        } else {
                            p.round += 1;
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
                    if p.errors == 3 {
                        self.result = Some(false);
                    }
                }
            }
            Board::Sewers(s) if index < 7 && s.open[index] => {
                s.open[index] = false;
                s.closed[index] = true;
            }
            _ => {}
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
