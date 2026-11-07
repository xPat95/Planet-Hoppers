//! Arcade combat simulation. Coordinates describe the floor, independent of rendering.
use std::hash::{BuildHasher, RandomState};

pub const ROOM_INTRO: f32 = 3.4;
pub const ESCAPE_TIME: f32 = 10.0;
pub const TV_X: f32 = 622.0;
pub const WIDTH: i32 = 960;
pub const HEIGHT: i32 = 540;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Tutorial,
    Hammer,
    LivingRoom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Title,
    Playing,
    Paused,
    StageClear,
    Defeat,
    ArcadeClear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Player,
    Rookie,
    Hammer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    Punch,
    Kick,
    Slam,
    Furniture(Furniture),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Idle,
    Strike {
        attack: Attack,
        time: f32,
        fired: bool,
    },
    Guard,
    Entering,
    Dodge(f32),
    Hurt(f32),
    Down,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub x: f32,
    pub y: f32,
    pub punch: bool,
    pub kick: bool,
    pub dodge: bool,
    pub jump: bool,
    pub guard: bool,
    pub grab: bool,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub facing: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub action: Action,
    pub cooldown: f32,
    pub invulnerable: f32,
    pub jump: f32,
    pub jump_velocity: f32,
    pub walk: f32,
    queued_strike: Option<(Attack, f32)>,
}

impl Fighter {
    pub fn new(kind: Kind, x: f32, y: f32) -> Self {
        let hp = match kind {
            Kind::Player => 100,
            Kind::Rookie => 30,
            Kind::Hammer => 650,
        };
        Self {
            kind,
            x,
            y,
            facing: if kind == Kind::Player { 1.0 } else { -1.0 },
            hp,
            max_hp: hp,
            action: Action::Idle,
            cooldown: 0.4,
            invulnerable: 0.0,
            jump: 0.0,
            jump_velocity: 0.0,
            walk: 0.0,
            queued_strike: None,
        }
    }
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
    pub fn enraged(&self) -> bool {
        self.kind == Kind::Hammer && self.hp * 2 <= self.max_hp
    }
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub gold: bool,
}
#[derive(Clone, Debug)]
pub struct Shockwave {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub hit: [bool; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PropState {
    Ready,
    Held(usize),
    Broken(f32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Furniture {
    Chair,
    Stool,
    Table,
}

#[derive(Clone, Debug)]
pub struct Prop {
    pub x: f32,
    pub y: f32,
    pub kind: Furniture,
    pub state: PropState,
}

impl Prop {
    pub fn reachable(&self, p: &Fighter) -> bool {
        self.state == PropState::Ready && (self.x - p.x).hypot((self.y - p.y) * 1.4) < 60.0
    }
}

/// Coordinates stay at the delivery site even if a player immediately picks up the prop.
pub struct Delivery {
    pub helper: usize,
    pub x: f32,
    pub y: f32,
    pub until: f32,
}

pub struct Game {
    pub intro_left: f32,
    pub delivery: Option<Delivery>,
    next_delivery: f32,
    random: RandomState,
    draw_number: u64,
    pub phase: Phase,
    pub stage: Stage,
    pub players: Vec<Fighter>,
    pub enemies: Vec<Fighter>,
    pub particles: Vec<Particle>,
    pub waves: Vec<Shockwave>,
    pub props: Vec<Prop>,
    pub score: u32,
    pub combo: u32,
    pub combo_time: f32,
    pub elapsed: f32,
    pub stage_time: f32,
    pub shake: f32,
    pub flash: f32,
    pub blocked: [f32; 2],
    pub immune_flash: f32,
    pub coop: bool,
    pub impacts: u32,
    pub slams: u32,
    pub stage_start_score: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            intro_left: 0.0,
            delivery: None,
            next_delivery: ESCAPE_TIME,
            random: RandomState::new(),
            draw_number: 0,
            phase: Phase::Title,
            stage: Stage::Tutorial,
            players: vec![],
            enemies: vec![],
            particles: vec![],
            waves: vec![],
            props: vec![],
            score: 0,
            combo: 0,
            combo_time: 0.0,
            elapsed: 0.0,
            stage_time: 0.0,
            shake: 0.0,
            flash: 0.0,
            blocked: [0.0; 2],
            immune_flash: 0.0,
            coop: false,
            impacts: 0,
            slams: 0,
            stage_start_score: 0,
        };
        g.load_stage(Stage::Tutorial);
        g.phase = Phase::Title;
        g
    }
    pub fn start(&mut self) {
        self.score = 0;
        self.load_stage(Stage::Tutorial);
    }
    pub fn retry(&mut self) {
        self.score = self.stage_start_score;
        self.load_stage(self.stage);
    }
    pub fn load_stage(&mut self, stage: Stage) {
        self.stage = stage;
        self.stage_start_score = self.score;
        self.phase = Phase::Playing;
        self.players = vec![Fighter::new(Kind::Player, 235.0, 380.0)];
        if self.coop {
            self.players.push(Fighter::new(Kind::Player, 310.0, 433.0));
        }
        self.enemies = match stage {
            Stage::Tutorial => vec![
                Fighter::new(Kind::Rookie, 690.0, 375.0),
                Fighter::new(Kind::Rookie, 775.0, 435.0),
                Fighter::new(Kind::Rookie, 830.0, 330.0),
            ],
            Stage::LivingRoom => vec![Fighter::new(Kind::Hammer, TV_X - 30.0, 325.0)],
            Stage::Hammer => {
                let mut boss = Fighter::new(Kind::Hammer, 369.0, 235.0);
                boss.action = Action::Entering;
                vec![boss]
            }
        };
        if self.coop {
            for e in &mut self.enemies {
                e.hp = e.hp * 3 / 2;
                e.max_hp = e.hp;
            }
        }
        self.props = if stage == Stage::Hammer {
            [
                (170.0, 364.0, Furniture::Chair),
                (445.0, 450.0, Furniture::Stool),
                (784.0, 328.0, Furniture::Chair),
                (840.0, 456.0, Furniture::Stool),
            ]
            .into_iter()
            .map(|(x, y, kind)| Prop {
                x,
                y,
                kind,
                state: PropState::Ready,
            })
            .collect()
        } else {
            Vec::new()
        };
        self.particles.clear();
        self.waves.clear();
        self.combo = 0;
        self.combo_time = 0.0;
        self.stage_time = 0.0;
        self.intro_left = if stage == Stage::LivingRoom {
            ROOM_INTRO
        } else {
            0.0
        };
        self.delivery = None;
        self.next_delivery = ESCAPE_TIME;
        self.blocked = [0.0; 2];
        self.immune_flash = 0.0;
        self.shake = 0.0;
        self.flash = 0.0;
    }
    pub fn toggle_pause(&mut self) {
        self.phase = match self.phase {
            Phase::Playing => Phase::Paused,
            Phase::Paused => Phase::Playing,
            p => p,
        };
    }
    pub fn advance(&mut self) {
        if self.phase == Phase::StageClear {
            match self.stage {
                Stage::Tutorial => self.load_stage(Stage::Hammer),
                Stage::Hammer => self.load_stage(Stage::LivingRoom),
                Stage::LivingRoom => {}
            }
        }
    }
    pub fn update(&mut self, dt: f32, inputs: [Input; 2]) {
        if self.phase != Phase::Playing || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        // Fixed-size substeps make hit windows independent of frame rate.
        let dt = dt.min(0.1);
        let n = (dt / (1.0 / 120.0)).ceil() as u32;
        for step in 0..n {
            let mut sub = inputs;
            if step > 0 {
                for i in &mut sub {
                    i.punch = false;
                    i.kick = false;
                    i.dodge = false;
                    i.jump = false;
                    i.grab = false;
                }
            }
            self.tick(dt / n as f32, sub);
            if self.phase != Phase::Playing {
                break;
            }
        }
    }
    fn tick(&mut self, dt: f32, inputs: [Input; 2]) {
        self.elapsed += dt;
        if self.intro_left > 0.0 {
            self.intro_left = (self.intro_left - dt).max(0.0);
            return;
        }
        self.stage_time += dt;
        if self.stage == Stage::LivingRoom {
            self.update_deliveries();
        }
        self.immune_flash = (self.immune_flash - dt).max(0.0);
        for t in &mut self.blocked {
            *t = (*t - dt).max(0.0);
        }
        self.shake = (self.shake - dt * 25.0).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
        self.combo_time = (self.combo_time - dt).max(0.0);
        if self.combo_time == 0.0 {
            self.combo = 0;
        }
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 280.0 * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        for prop in &mut self.props {
            match prop.state {
                PropState::Held(owner) if !self.players[owner].alive() => {
                    prop.state = PropState::Ready
                }
                PropState::Broken(left) => {
                    prop.state = if left <= dt {
                        PropState::Ready
                    } else {
                        PropState::Broken(left - dt)
                    }
                }
                _ => {}
            }
        }
        for (idx, (p, input)) in self.players.iter_mut().zip(inputs).enumerate() {
            if !p.alive() {
                continue;
            }
            timers(p, dt);
            let holding = self
                .props
                .iter()
                .find(|prop| prop.state == PropState::Held(idx))
                .map(|p| p.kind);
            // Keep a tap near the recovery frame so a combo does not eat inputs.
            p.queued_strike = if input.guard {
                None
            } else if input.kick || input.punch {
                Some((
                    if input.kick {
                        Attack::Kick
                    } else {
                        Attack::Punch
                    },
                    0.13,
                ))
            } else {
                p.queued_strike
                    .and_then(|(a, left)| (left > dt).then_some((a, left - dt)))
            };
            if p.action == Action::Guard && !input.guard {
                p.action = Action::Idle;
            }
            if matches!(p.action, Action::Idle | Action::Guard) {
                if input.x != 0.0 {
                    p.facing = input.x.signum();
                }
                if input.guard && p.jump == 0.0 {
                    p.action = Action::Guard;
                    p.walk = 0.0;
                    continue;
                }
                if input.grab && holding.is_none() && p.jump == 0.0 {
                    if let Some(prop) =
                        self.props
                            .iter_mut()
                            .filter(|v| v.reachable(p))
                            .min_by(|a, b| {
                                (a.x - p.x)
                                    .hypot(a.y - p.y)
                                    .total_cmp(&(b.x - p.x).hypot(b.y - p.y))
                            })
                    {
                        prop.state = PropState::Held(idx);
                        continue;
                    }
                }
                if input.jump && p.jump == 0.0 {
                    p.jump_velocity = 300.0;
                }
                if input.dodge && p.cooldown <= 0.0 {
                    if input.x != 0.0 {
                        p.facing = input.x.signum();
                    }
                    p.action = Action::Dodge(0.32);
                    p.invulnerable = 0.36;
                    p.cooldown = 0.75;
                } else if let Some((strike, _)) = p.queued_strike.take() {
                    p.action = Action::Strike {
                        attack: holding.map_or(strike, Attack::Furniture),
                        time: 0.0,
                        fired: false,
                    };
                } else {
                    let len = input.x.hypot(input.y).max(1.0);
                    p.x += input.x / len * 190.0 * dt;
                    p.y += input.y / len * 125.0 * dt;
                    if input.x != 0.0 {
                        p.facing = input.x.signum();
                    }
                    if input.x != 0.0 || input.y != 0.0 {
                        p.walk += dt * 12.0;
                    } else {
                        p.walk = 0.0;
                    }
                }
            }
            if matches!(p.action, Action::Dodge(_)) {
                p.x += p.facing * 390.0 * dt;
            }
            clamp_floor(p);
        }
        for (index, enemy) in self.enemies.iter_mut().enumerate() {
            if !enemy.alive() {
                continue;
            }
            if enemy.action == Action::Entering {
                // Step out of the breach in the rear wall before joining the playable floor.
                let t = (self.stage_time / 1.05).min(1.0);
                enemy.x = 369.0 + 48.0 * t;
                enemy.y = 235.0 + 82.0 * t;
                enemy.walk += dt * 7.0;
                if t >= 1.0 {
                    enemy.action = Action::Idle;
                }
                continue;
            }
            timers(enemy, dt);
            let Some(target) = self
                .players
                .iter()
                .filter(|p| p.alive())
                .min_by(|a, b| distance(enemy, a).total_cmp(&distance(enemy, b)))
            else {
                continue;
            };
            if enemy.action == Action::Idle {
                let dx = target.x - enemy.x;
                let lane = if enemy.kind == Kind::Rookie {
                    [0.0, 18.0, -18.0][index % 3]
                } else {
                    0.0
                };
                let dy = (target.y + lane).clamp(310.0, 468.0) - enemy.y;
                enemy.facing = if dx < 0.0 { -1.0 } else { 1.0 };
                let boss = enemy.kind == Kind::Hammer;
                let range = if boss { 115.0 } else { 54.0 };
                if boss && enemy.cooldown <= 0.0 && (self.stage_time as u32 % 6 >= 4) {
                    enemy.action = Action::Strike {
                        attack: Attack::Slam,
                        time: 0.0,
                        fired: false,
                    };
                } else if dx.abs() < range
                    && (target.y - enemy.y).abs() < 25.0
                    && enemy.cooldown <= 0.0
                {
                    enemy.action = Action::Strike {
                        attack: Attack::Punch,
                        time: 0.0,
                        fired: false,
                    };
                } else if self.stage_time > 0.65 + index as f32 * 0.45 {
                    let speed = if boss {
                        if enemy.enraged() {
                            110.0
                        } else {
                            78.0
                        }
                    } else {
                        71.0 + index as f32 * 6.0
                    };
                    if dx.abs() > range * 0.75 {
                        enemy.x += dx.signum() * speed * dt;
                        enemy.walk += dt * 8.0;
                    }
                    if dy.abs() > 10.0 {
                        enemy.y += dy.signum() * speed * 0.65 * dt;
                    }
                }
            }
            clamp_floor(enemy);
        }
        // Resolve each attack once, at the end of its announced wind-up.
        let mut strikes = Vec::new();
        for (is_player, fighters) in [(true, &mut self.players), (false, &mut self.enemies)] {
            for (idx, f) in fighters.iter_mut().enumerate() {
                if let Action::Strike {
                    attack,
                    time,
                    fired,
                } = f.action
                {
                    let windup = windup(f.kind, attack, f.enraged());
                    let next = time + dt;
                    if !fired && next >= windup {
                        strikes.push((is_player, idx, attack));
                    }
                    f.action = if next > windup + if attack == Attack::Slam { 0.9 } else { 0.24 } {
                        f.cooldown = if is_player {
                            0.0
                        } else if f.kind == Kind::Hammer {
                            0.65
                        } else {
                            1.5
                        };
                        Action::Idle
                    } else {
                        Action::Strike {
                            attack,
                            time: next,
                            fired: fired || next >= windup,
                        }
                    };
                }
            }
        }
        for (player, idx, attack) in strikes {
            if player && self.players[idx].alive() {
                let f = self.players[idx].clone();
                let (range, damage) = match attack {
                    Attack::Furniture(_) => (145.0, 135),
                    Attack::Kick => (103.0, 23),
                    _ => (78.0, 14),
                };
                if matches!(attack, Attack::Furniture(_)) {
                    let Some(prop) = self
                        .props
                        .iter_mut()
                        .find(|p| p.state == PropState::Held(idx))
                    else {
                        continue;
                    };
                    prop.state = PropState::Broken(3.0);
                }
                let mut impacts = Vec::new();
                for enemy in &mut self.enemies {
                    if enemy.action == Action::Entering || !in_reach(&f, enemy, range) {
                        continue;
                    }
                    if enemy.kind == Kind::Hammer && !matches!(attack, Attack::Furniture(_)) {
                        self.immune_flash = 0.6;
                        continue;
                    }
                    if hit(enemy, damage, f.facing * 12.0) {
                        self.combo += 1;
                        self.combo_time = 2.0;
                        self.score += damage as u32 * 10;
                        if !enemy.alive() {
                            self.score += 500;
                        }
                        impacts.push((
                            enemy.x,
                            enemy.y
                                - if enemy.kind == Kind::Hammer {
                                    100.0
                                } else {
                                    55.0
                                },
                        ));
                    }
                }
                for (x, y) in impacts {
                    self.burst(x, y, true);
                }
            } else if !player && matches!(self.enemies[idx].action, Action::Strike { .. }) {
                let f = self.enemies[idx].clone();
                if attack == Attack::Slam {
                    self.waves.push(Shockwave {
                        x: f.x,
                        y: f.y,
                        radius: 0.0,
                        hit: [false; 2],
                    });
                    self.shake = 10.0;
                    self.flash = 0.12;
                    self.slams += 1;
                } else {
                    let mut impacts = Vec::new();
                    for (pi, p) in self.players.iter_mut().enumerate() {
                        if !in_reach(&f, p, if f.kind == Kind::Hammer { 140.0 } else { 70.0 })
                            || p.jump >= 22.0
                        {
                            continue;
                        }
                        if p.action == Action::Guard && (f.x - p.x) * p.facing >= -4.0 {
                            self.blocked[pi] = 0.35;
                            continue;
                        }
                        if hit(
                            p,
                            if f.kind == Kind::Hammer { 18 } else { 5 },
                            f.facing * 22.0,
                        ) {
                            impacts.push((p.x, p.y - 55.0));
                        }
                    }
                    for (x, y) in impacts {
                        self.burst(x, y, false);
                    }
                }
            }
        }
        for wave in &mut self.waves {
            wave.radius += dt * 255.0;
            for (idx, p) in self.players.iter_mut().enumerate() {
                let dist = (p.x - wave.x).hypot((p.y - wave.y) * 1.8);
                if !wave.hit[idx] && (dist - wave.radius).abs() < 18.0 && p.alive() {
                    wave.hit[idx] = true;
                    if p.jump < 25.0 {
                        hit(p, 22, (p.x - wave.x).signum() * 20.0);
                    }
                }
            }
        }
        self.waves.retain(|w| w.radius < 1100.0);
        if self.players.iter().all(|p| !p.alive()) {
            self.phase = Phase::Defeat;
        } else if self.enemies.iter().all(|e| !e.alive()) {
            self.phase = match self.stage {
                Stage::Tutorial | Stage::Hammer => Phase::StageClear,
                Stage::LivingRoom => Phase::ArcadeClear,
            };
            self.waves.clear();
        }
    }
    fn draw_random(&mut self, count: usize) -> usize {
        self.draw_number += 1;
        (self.random.hash_one(self.draw_number) % count as u64) as usize
    }

    fn update_deliveries(&mut self) {
        // Broken room furniture is replaced only by another delivery.
        self.props
            .retain(|p| !matches!(p.state, PropState::Broken(_)));
        if self
            .delivery
            .as_ref()
            .is_some_and(|d| self.stage_time >= d.until)
        {
            self.delivery = None;
        }
        if self.stage_time < self.next_delivery || self.props.len() >= 3 {
            return;
        }
        let mut sites = Vec::new();
        for x in [152.0, 368.0, 584.0, 800.0] {
            for y in [342.0, 441.0] {
                if !self.props.iter().any(|p| p.x == x && p.y == y) {
                    sites.push((x, y));
                }
            }
        }
        let (x, y) = sites[self.draw_random(sites.len())];
        let kind = [Furniture::Chair, Furniture::Table, Furniture::Stool][self.draw_random(3)];
        self.props.push(Prop {
            x,
            y,
            kind,
            state: PropState::Ready,
        });
        // Mordecai is P1 and Rigby is P2. Only off-duty characters deliver.
        let first = self.players.len();
        let helper = first + self.draw_random(6 - first);
        self.delivery = Some(Delivery {
            helper,
            x,
            y,
            until: self.stage_time + 1.7,
        });
        self.next_delivery = self.stage_time + 2.5 + self.draw_random(201) as f32 / 100.0;
    }

    fn burst(&mut self, x: f32, y: f32, gold: bool) {
        self.impacts += 1;
        self.shake = 4.0;
        for i in 0..12 {
            let a = i as f32 * 2.399;
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * 130.0,
                vy: a.sin() * 120.0 - 70.0,
                life: 0.35 + i as f32 * 0.012,
                gold,
            });
        }
    }
}

pub fn windup(kind: Kind, attack: Attack, enraged: bool) -> f32 {
    match (kind, attack) {
        (Kind::Player, Attack::Punch) => 0.08,
        (Kind::Player, _) => 0.21,
        (Kind::Hammer, Attack::Slam) => {
            if enraged {
                0.8
            } else {
                1.1
            }
        }
        (Kind::Hammer, _) => {
            if enraged {
                0.42
            } else {
                0.65
            }
        }
        _ => 0.95,
    }
}
fn distance(a: &Fighter, b: &Fighter) -> f32 {
    (a.x - b.x).hypot((a.y - b.y) * 2.0)
}
fn in_reach(a: &Fighter, b: &Fighter, range: f32) -> bool {
    b.alive()
        && (b.x - a.x) * a.facing >= -12.0
        && (a.x - b.x).abs() < range
        && (a.y - b.y).abs() < 29.0
}
fn hit(f: &mut Fighter, damage: i32, knockback: f32) -> bool {
    if !f.alive() || f.invulnerable > 0.0 {
        return false;
    }
    f.hp = (f.hp - damage).max(0);
    f.x += knockback;
    clamp_floor(f);
    // The boss resists stun; his announced attacks remain readable and dangerous.
    if !f.alive() {
        f.action = Action::Down;
    } else if f.kind != Kind::Hammer {
        f.action = Action::Hurt(0.22);
    }
    f.invulnerable = if f.kind == Kind::Player { 0.85 } else { 0.12 };
    true
}
fn clamp_floor(f: &mut Fighter) {
    f.x = f.x.clamp(55.0, 905.0);
    f.y = f.y.clamp(310.0, 468.0);
}
fn timers(f: &mut Fighter, dt: f32) {
    f.cooldown = (f.cooldown - dt).max(0.0);
    f.invulnerable = (f.invulnerable - dt).max(0.0);
    if f.jump_velocity != 0.0 || f.jump > 0.0 {
        f.jump += f.jump_velocity * dt;
        f.jump_velocity -= 780.0 * dt;
        if f.jump < 0.0 {
            f.jump = 0.0;
            f.jump_velocity = 0.0;
        }
    }
    match f.action {
        Action::Dodge(t) | Action::Hurt(t) => {
            f.action = if t <= dt {
                Action::Idle
            } else if matches!(f.action, Action::Dodge(_)) {
                Action::Dodge(t - dt)
            } else {
                Action::Hurt(t - dt)
            };
        }
        _ => {}
    }
}
