//! Render-independent free flight. World +Z is forward; +X is screen-left.
use std::f32::consts::TAU;
pub const TUNNEL_RADIUS: f32 = 10.5;
pub const WALL_APOTHEM: f32 = TUNNEL_RADIUS * 0.923_879_5;
pub const SHIP_HALF: [f32; 3] = [1.0, 0.48, 1.5];
pub const REACTOR_Z: f32 = 2500.0;
pub const ATTACK_Z: f32 = REACTOR_Z - 66.0;
pub const CORE_RADIUS: f32 = 2.15;
// Full steel bulkheads, each with independently breakable passage panels.
// Irregular but repeatable placement: two after halfway, four in the final approach.
pub const BULKHEADS: [(f32, f32, f32); 6] = [
    (1350.0, -3.0, 2.0),
    (1800.0, 3.0, -2.0),
    (2130.0, 3.0, 2.0),
    (2210.0, -3.0, 2.0),
    (2275.0, -3.0, -2.0),
    (2340.0, 3.0, -2.0),
];
pub const REAR_ENCOUNTERS: [f32; 4] = [420.0, 930.0, 1440.0, 1940.0];
pub const REAR_WARNING: f32 = 1.65;
pub const REAR_FLIGHT: f32 = 0.55;
pub const REAR_RADIUS: f32 = 0.38;
#[derive(Clone, Copy, Debug)]
pub struct RearAttack {
    pub target: [f32; 2],
    pub age: f32,
    pub resolved: bool,
}
impl RearAttack {
    pub fn warning(&self) -> bool {
        self.age < REAR_WARNING
    }
}
pub const STEP: f32 = 1.0 / 120.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Briefing,
    Intro,
    Running,
    Paused,
    Lost,
    Explosion,
    Outro,
    Won,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    Armor,
    Cargo,
}
#[derive(Clone, Copy, Default)]
pub struct Controls {
    pub x: f32,
    pub y: f32,
    pub boost: bool,
    pub fire: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Girder,
    Pipe,
    Duct,
    Panel,
}
#[derive(Clone, Debug)]
pub struct Obstacle {
    pub pos: [f32; 3],
    pub half: [f32; 3],
    pub material: Material,
    pub hp: u8,
    pub flash: f32,
    pub contacted: bool,
    pub seed: usize,
    pub shape: Shape,
    pub angle: f32, // rotation around world Z; local X follows the member
    pub group: usize,
}
impl Obstacle {
    pub fn alive(&self) -> bool {
        self.material == Material::Armor || self.hp > 0
    }
    pub fn hit(&self, from: [f32; 3], to: [f32; 3], margin: [f32; 3]) -> Option<f32> {
        if !self.alive() {
            return None;
        }
        let (sn, cs) = self.angle.sin_cos();
        let local = |p: [f32; 3]| {
            let x = p[0] - self.pos[0];
            let y = p[1] - self.pos[1];
            [cs * x + sn * y, -sn * x + cs * y, p[2] - self.pos[2]]
        };
        let expanded = [
            self.half[0] + cs.abs() * margin[0] + sn.abs() * margin[1],
            self.half[1] + sn.abs() * margin[0] + cs.abs() * margin[1],
            self.half[2] + margin[2],
        ];
        segment_box(local(from), local(to), [0.0; 3], expanded)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub pos: [f32; 3],
    pub origin_z: f32,
    pub torpedo: bool,
}
#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: [f32; 3],
    pub velocity: [f32; 3],
    pub age: f32,
    pub life: f32,
    pub size: f32,
    pub seed: f32,
    pub debris: bool,
}
pub struct Game {
    pub ship_half: [f32; 3],
    pub phase: Phase,
    pub pos: [f32; 2],
    pub velocity: [f32; 2],
    pub camera: [f32; 2],
    pub bank: f32,
    pub pitch: f32,
    pub distance: f32,
    pub travel: f32,
    pub speed: f32,
    pub time: f32,
    pub hull: u32,
    pub score: u32,
    pub destroyed: u32,
    pub reactor_hp: u32,
    pub cooldown: f32,
    pub invulnerability: f32,
    pub flash: f32,
    pub end_time: f32,
    pub muzzle: f32,
    pub obstacles: Vec<Obstacle>,
    pub shots: Vec<Shot>,
    pub particles: Vec<Particle>,
    pub shots_fired: u32,
    pub rear_attack: Option<RearAttack>,
    pub rear_encounters: usize,
}
pub fn segment_box(from: [f32; 3], to: [f32; 3], center: [f32; 3], half: [f32; 3]) -> Option<f32> {
    let (mut near, mut far) = (0.0_f32, 1.0_f32);
    for i in 0..3 {
        let delta = to[i] - from[i];
        if delta.abs() < 0.000_01 {
            if (from[i] - center[i]).abs() > half[i] {
                return None;
            }
        } else {
            let a = (center[i] - half[i] - from[i]) / delta;
            let b = (center[i] + half[i] - from[i]) / delta;
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}
// Every visible structural member is its own collider; empty space has no parent box.
struct Builder {
    items: Vec<Obstacle>,
    group: usize,
}
impl Builder {
    fn part(&mut self, pos: [f32; 3], half: [f32; 3], angle: f32, shape: Shape) {
        self.items.push(Obstacle {
            pos,
            half,
            angle,
            shape,
            group: self.group,
            material: if shape == Shape::Panel {
                Material::Cargo
            } else {
                Material::Armor
            },
            hp: 3,
            flash: 0.0,
            contacted: false,
            seed: self.items.len(),
        });
    }
    fn bar(&mut self, a: [f32; 2], b: [f32; 2], z: f32, radius: f32, depth: f32, shape: Shape) {
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        self.part(
            [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5, z],
            [dx.hypot(dy) * 0.5, radius, depth],
            dy.atan2(dx),
            shape,
        );
    }
    fn truss(&mut self, y: f32, z: f32) {
        for depth in [-0.8, 0.8] {
            for dy in [-0.75, 0.75] {
                self.bar(
                    [-9.0, y + dy],
                    [9.0, y + dy],
                    z + depth,
                    0.16,
                    0.15,
                    Shape::Girder,
                );
            }
            for i in 0..6 {
                let x = -9.0 + i as f32 * 3.0;
                let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                self.bar(
                    [x, y - sign * 0.75],
                    [x + 3.0, y + sign * 0.75],
                    z + depth,
                    0.11,
                    0.13,
                    Shape::Girder,
                );
            }
        }
        for x in [-8.8, -3.0, 3.0, 8.8] {
            self.part([x, y, z], [0.18, 0.9, 0.95], 0.0, Shape::Girder);
        }
    }
    fn portal(&mut self, x: f32, y: f32, z: f32) {
        for dx in [-4.4, 4.4] {
            self.bar([x + dx, -8.0], [x + dx, 6.0], z, 0.24, 1.2, Shape::Girder);
            self.bar(
                [x + dx, y + 3.7],
                [x + dx * 0.62, y + 2.4],
                z,
                0.19,
                1.2,
                Shape::Girder,
            );
        }
        for dy in [-3.7, 3.7] {
            self.bar(
                [x - 4.4, y + dy],
                [x + 4.4, y + dy],
                z,
                0.25,
                1.2,
                Shape::Girder,
            );
        }
    }
}
fn gate_openings(x: f32, y: f32) -> [[f32; 4]; 3] {
    [
        [x, y, 2.8, 2.7],
        [-x.signum() * 3.4, y, 2.5, 2.3],
        [0.0, -y.signum() * 4.0, 2.8, 1.9],
    ]
}
fn route() -> Vec<Obstacle> {
    let mut b = Builder {
        items: vec![],
        group: 0,
    };
    for wave in 0..26 {
        b.group = wave;
        let z = 155.0 + wave as f32 * 76.0;
        let side = if (wave / 6) % 2 == 0 { 1.0 } else { -1.0 };
        match wave % 6 {
            0 => {
                // Double-depth open lattice, wall to wall.
                b.truss(-2.8 * side, z);
                b.part([3.7 * side, 1.0, z], [1.15, 1.45, 0.22], 0.0, Shape::Panel);
                b.bar(
                    [3.7 * side, 2.45],
                    [3.7 * side, 8.0],
                    z,
                    0.12,
                    0.25,
                    Shape::Girder,
                );
            }
            1 => {
                // Offset gantry and a replaceable maintenance screen.
                b.portal(0.8 * side, 0.5, z);
                b.part([0.8 * side, 0.5, z], [2.7, 2.7, 0.22], 0.0, Shape::Panel);
            }
            2 => {
                // Three independent cooling pipes with dog-leg elbows.
                for i in 0..3 {
                    let y = -3.6 + i as f32 * 1.0;
                    b.bar(
                        [-9.2, y],
                        [2.8 * side, y],
                        z + i as f32 * 1.2,
                        0.27,
                        0.27,
                        Shape::Pipe,
                    );
                    b.bar(
                        [2.8 * side, y],
                        [2.8 * side, y + 1.4],
                        z + i as f32 * 1.2,
                        0.27,
                        0.27,
                        Shape::Pipe,
                    );
                }
                b.part([-6.5, 3.0, z + 3.0], [0.5, 2.7, 2.5], 0.0, Shape::Duct);
                b.part([-5.0, 5.2, z + 3.0], [1.9, 0.5, 2.5], 0.0, Shape::Duct);
            }
            3 => {
                // Thin rectangular ductwork attached to opposite walls.
                for (x, y, dz) in [(-6.0, -2.8, 0.0), (6.0, 3.0, 12.0)] {
                    b.part([x, y, z + dz], [3.5, 0.65, 2.6], 0.0, Shape::Duct);
                    b.part(
                        [x.signum() * 3.0, y + 1.1, z + dz],
                        [0.5, 1.7, 2.6],
                        0.0,
                        Shape::Duct,
                    );
                    b.part(
                        [x.signum() * 3.0, y - 1.4, z + dz],
                        [1.3, 0.65, 0.2],
                        0.0,
                        Shape::Panel,
                    );
                    b.bar(
                        [x, y],
                        [x, y.signum() * 8.5],
                        z + dz,
                        0.14,
                        0.2,
                        Shape::Girder,
                    );
                }
            }
            4 => {
                // Open equipment rack and cable tray, no solid backing.
                for x in [-6.2, -2.8] {
                    b.bar([x, -7.5], [x, 4.5], z, 0.18, 1.4, Shape::Girder);
                }
                for y in [-5.5, -1.0, 3.5] {
                    b.bar([-6.2, y], [-2.8, y], z, 0.18, 1.4, Shape::Girder);
                    b.part([-4.5, y + 1.1, z], [1.4, 0.9, 0.24], 0.0, Shape::Panel);
                }
                for y in [4.8, 5.4] {
                    b.bar([-9.0, y], [7.0, y], z + 6.0, 0.12, 0.6, Shape::Girder);
                }
                for x in -8..=7 {
                    b.bar(
                        [x as f32, 4.8],
                        [x as f32, 5.4],
                        z + 6.0,
                        0.05,
                        0.6,
                        Shape::Girder,
                    );
                }
            }
            _ => {
                // Deep service collar, diagonal corner braces and a suspended screen.
                b.portal(-side, 0.0, z);
                b.truss(5.4, z + 9.0);
                b.part([-side, 0.0, z + 1.6], [2.6, 2.5, 0.24], 0.0, Shape::Panel);
            }
        }
    }
    // Subtract all three shoot-through openings from each steel bulkhead.
    for (index, (z, x, y)) in BULKHEADS.into_iter().enumerate() {
        b.group = 26 + index;
        let openings = gate_openings(x, y);
        let mut levels = vec![-11.0, 11.0];
        for [_, py, _, hy] in openings {
            levels.extend([py - hy, py + hy]);
        }
        levels.sort_by(f32::total_cmp);
        levels.dedup();
        for band in levels.windows(2) {
            let mid = (band[0] + band[1]) * 0.5;
            let mut gaps: Vec<_> = openings
                .iter()
                .filter(|o| (mid - o[1]).abs() < o[3])
                .map(|o| (o[0] - o[2], o[0] + o[2]))
                .collect();
            gaps.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut left = -11.0;
            gaps.push((11.0, 11.0));
            for (l, r) in gaps {
                if l > left {
                    b.part(
                        [(left + l) * 0.5, mid, z],
                        [(l - left) * 0.5, (band[1] - band[0]) * 0.5, 1.6],
                        0.0,
                        Shape::Duct,
                    );
                }
                left = left.max(r);
            }
        }
        for [px, py, hx, hy] in openings {
            b.part([px, py, z], [hx, hy, 0.24], 0.0, Shape::Panel);
            // Frames stay outside the opening, including the two new routes.
            for dx in [-hx - 0.17, hx + 0.17] {
                b.bar(
                    [px + dx, py - hy - 0.3],
                    [px + dx, py + hy + 0.3],
                    z - 1.8,
                    0.14,
                    0.15,
                    Shape::Girder,
                );
            }
            for dy in [-hy - 0.17, hy + 0.17] {
                b.bar(
                    [px - hx, py + dy],
                    [px + hx, py + dy],
                    z - 1.8,
                    0.14,
                    0.15,
                    Shape::Girder,
                );
            }
        }
    }
    b.items
}
impl Default for Game {
    fn default() -> Self {
        Self {
            ship_half: SHIP_HALF,
            phase: Phase::Briefing,
            pos: [0.0; 2],
            velocity: [0.0; 2],
            camera: [0.0; 2],
            bank: 0.0,
            pitch: 0.0,
            distance: 0.0,
            travel: 0.0,
            speed: 36.0,
            time: 0.0,
            hull: 3,
            score: 0,
            destroyed: 0,
            reactor_hp: 6,
            cooldown: 0.0,
            invulnerability: 0.0,
            flash: 0.0,
            end_time: 0.0,
            muzzle: 0.0,
            obstacles: route(),
            shots: vec![],
            particles: vec![],
            shots_fired: 0,
            rear_attack: None,
            rear_encounters: 0,
        }
    }
}
impl crate::minigames::common::host::AttemptOutcome for Game {
    fn attempt_outcome(&self) -> Option<crate::minigames::common::host::MinigameResult> {
        use crate::minigames::common::host::MinigameResult;
        match self.phase {
            // Reactor destruction ends the challenge; the outro is navigation/presentation.
            Phase::Explosion | Phase::Outro | Phase::Won => Some(MinigameResult::Victory),
            Phase::Lost => Some(MinigameResult::Defeat),
            _ => None,
        }
    }
}

impl Game {
    pub fn start(&mut self) {
        let half = self.ship_half;
        *self = Self::default();
        self.ship_half = half;
        self.phase = Phase::Running;
    }
    pub fn intro(&mut self) {
        let half = self.ship_half;
        *self = Self::default();
        self.ship_half = half;
        self.phase = Phase::Intro;
    }
    pub fn finish_video(&mut self) {
        self.phase = match self.phase {
            Phase::Intro => Phase::Briefing,
            Phase::Outro => Phase::Won,
            phase => phase,
        };
    }
    pub fn hull_half(&self) -> [f32; 3] {
        let h = self.ship_half;
        let y = h[1] * self.pitch.cos().abs() + h[2] * self.pitch.sin().abs();
        [
            h[0] * self.bank.cos().abs() + y * self.bank.sin().abs(),
            h[0] * self.bank.sin().abs() + y * self.bank.cos().abs(),
            h[1] * self.pitch.sin().abs() + h[2] * self.pitch.cos().abs(),
        ]
    }
    pub fn attacking(&self) -> bool {
        self.distance >= ATTACK_Z
    }
    pub fn toggle_pause(&mut self) {
        self.phase = match self.phase {
            Phase::Running => Phase::Paused,
            Phase::Paused => Phase::Running,
            p => p,
        };
    }
    fn burst(&mut self, pos: [f32; 3], count: usize, debris: bool) {
        for i in 0..count {
            let seed = i as f32 * 2.399_963 + self.time * 7.0;
            self.particles.push(Particle {
                pos,
                velocity: [
                    seed.cos() * 5.0,
                    (seed * 1.71).sin() * 5.0,
                    (seed * 0.73).cos() * 8.0,
                ],
                age: 0.0,
                life: if debris { 1.7 } else { 0.55 },
                size: if debris {
                    0.08 + (i % 5) as f32 * 0.045
                } else {
                    0.045
                },
                seed,
                debris,
            });
        }
    }
    fn damage(&mut self) {
        if self.invulnerability > 0.0 {
            return;
        }
        self.hull = self.hull.saturating_sub(1);
        self.flash = 1.0;
        self.invulnerability = 1.15;
        self.burst([self.pos[0], self.pos[1], self.distance], 28, true);
        if self.hull == 0 {
            self.phase = Phase::Lost;
            self.end_time = 0.0;
        }
    }
    fn particles(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.age += dt;
            for i in 0..3 {
                p.pos[i] += p.velocity[i] * dt;
            }
            p.velocity[1] -= dt * 1.8;
        }
        self.particles
            .retain(|p| p.age < p.life && p.pos[2] > self.distance - 16.0);
    }
    pub fn update(&mut self, dt: f32, input: Controls) {
        if matches!(self.phase, Phase::Explosion | Phase::Lost) {
            self.end_time += dt;
            self.particles(dt);
            self.flash = (self.flash - dt * 2.0).max(0.0);
            if self.phase == Phase::Explosion && self.end_time >= 1.15 {
                self.phase = Phase::Outro;
            }
            return;
        }
        if self.phase != Phase::Running {
            return;
        }
        self.time += dt;
        self.flash = (self.flash - dt * 3.0).max(0.0);
        self.invulnerability = (self.invulnerability - dt).max(0.0);
        self.cooldown = (self.cooldown - dt).max(0.0);
        self.muzzle = (self.muzzle - dt * 12.0).max(0.0);
        let old = [self.pos[0], self.pos[1], self.distance];
        let length = (input.x * input.x + input.y * input.y).sqrt().max(1.0);
        for (i, value) in [input.x / length, input.y / length].into_iter().enumerate() {
            self.velocity[i] += (value * 9.8 - self.velocity[i]) * (1.0 - (-dt * 13.0).exp());
            self.pos[i] += self.velocity[i] * dt;
        }
        let target_speed =
            36.0 + self.distance / ATTACK_Z * 7.0 + if input.boost { 19.0 } else { 0.0 };
        self.speed += (target_speed - self.speed) * (1.0 - (-dt * 3.5).exp());
        self.travel += self.speed * dt;
        self.distance = (self.distance + self.speed * dt).min(ATTACK_Z);
        self.bank += (-self.velocity[0] * 0.044 - self.bank) * (1.0 - (-dt * 9.0).exp());
        self.pitch += (self.velocity[1] * 0.017 - self.pitch) * (1.0 - (-dt * 8.0).exp());
        let half = self.hull_half();
        let mut touched = false;
        for i in 0..8 {
            let a = (i as f32 + 0.5) * TAU / 8.0;
            let n = [a.cos(), a.sin()];
            let excess = self.pos[0] * n[0]
                + self.pos[1] * n[1]
                + half[0] * n[0].abs()
                + half[1] * n[1].abs()
                - (WALL_APOTHEM - 0.4);
            if excess > 0.0 {
                self.pos[0] -= n[0] * excess;
                self.pos[1] -= n[1] * excess;
                let outward = self.velocity[0] * n[0] + self.velocity[1] * n[1];
                if outward > 0.0 {
                    self.velocity[0] -= outward * n[0];
                    self.velocity[1] -= outward * n[1];
                }
                touched = true;
            }
        }
        if touched {
            self.damage();
        }
        for i in 0..2 {
            self.camera[i] += (self.pos[i] - self.camera[i]) * (1.0 - (-dt * 5.0).exp());
        }
        if input.fire && self.cooldown <= 0.0 && self.phase == Phase::Running {
            let torpedo = self.attacking();
            self.shots.push(Shot {
                pos: [
                    self.pos[0],
                    self.pos[1],
                    self.distance + self.hull_half()[2] + 0.3,
                ],
                origin_z: self.distance,
                torpedo,
            });
            self.shots_fired += 1;
            self.cooldown = if torpedo { 0.42 } else { 0.15 };
            self.muzzle = 1.0;
        }
        for o in &mut self.obstacles {
            o.flash = (o.flash - dt * 6.0).max(0.0);
        }
        let shots = std::mem::take(&mut self.shots);
        for mut shot in shots {
            let previous = shot.pos;
            shot.pos[2] += (if shot.torpedo { 145.0 } else { 245.0 }) * dt;
            let hit = self
                .obstacles
                .iter()
                .enumerate()
                .filter_map(|(i, o)| o.hit(previous, shot.pos, [0.06; 3]).map(|t| (i, t)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((index, t)) = hit {
                let o = &mut self.obstacles[index];
                let destroyed = if o.material == Material::Cargo {
                    o.hp = o.hp.saturating_sub(1);
                    o.flash = 1.0;
                    o.hp == 0
                } else {
                    false
                };
                let p = [
                    shot.pos[0],
                    shot.pos[1],
                    previous[2] + (shot.pos[2] - previous[2]) * t,
                ];
                if destroyed {
                    self.destroyed += 1;
                    self.score += 250;
                }
                self.burst(p, if destroyed { 34 } else { 9 }, destroyed);
            } else if previous[2] < REACTOR_Z && shot.pos[2] >= REACTOR_Z {
                // Straight shots only: no homing, targeting assistance or alignment correction.
                if shot.torpedo
                    && self.reactor_hp > 0
                    && shot.pos[0].hypot(shot.pos[1]) < CORE_RADIUS
                {
                    self.reactor_hp -= 1;
                    self.score += 150;
                    self.burst([shot.pos[0], shot.pos[1], REACTOR_Z - 2.0], 20, false);
                    if self.reactor_hp == 0 {
                        self.phase = Phase::Explosion;
                        self.end_time = 0.0;
                        self.score += 3000;
                        self.burst([0.0, 0.0, REACTOR_Z], 160, true);
                    }
                }
            } else if shot.pos[2] - shot.origin_z < 270.0 {
                self.shots.push(shot);
            }
        }
        let now = [self.pos[0], self.pos[1], self.distance];
        let mut collision = false;
        for o in &mut self.obstacles {
            if !o.contacted && o.hit(old, now, half).is_some() {
                o.contacted = true;
                collision = true;
            }
        }
        if collision {
            let groups: Vec<_> = self
                .obstacles
                .iter()
                .filter(|o| o.contacted)
                .map(|o| o.group)
                .collect();
            for o in &mut self.obstacles {
                if groups.contains(&o.group) {
                    o.contacted = true;
                }
            }
        }
        if collision && self.phase == Phase::Running {
            self.damage();
        }
        if self.phase == Phase::Running {
            self.update_rear(dt, old, now, half);
        }
        self.particles(dt);
    }
    fn update_rear(&mut self, dt: f32, old: [f32; 3], now: [f32; 3], half: [f32; 3]) {
        if self.rear_attack.is_none()
            && self.rear_encounters < REAR_ENCOUNTERS.len()
            && self.distance >= REAR_ENCOUNTERS[self.rear_encounters]
            && !self.attacking()
        {
            self.rear_attack = Some(RearAttack {
                target: self.pos,
                age: 0.0,
                resolved: false,
            });
            self.rear_encounters += 1;
        }
        if let Some(mut attack) = self.rear_attack {
            let before = attack.age;
            attack.age += dt;
            let impact = REAR_WARNING + REAR_FLIGHT;
            if !attack.resolved && attack.age >= impact {
                attack.resolved = true;
                let t = ((impact - before) / dt).clamp(0.0, 1.0);
                let x = old[0] + (now[0] - old[0]) * t;
                let y = old[1] + (now[1] - old[1]) * t;
                if (x - attack.target[0]).abs() < half[0] + REAR_RADIUS
                    && (y - attack.target[1]).abs() < half[1] + REAR_RADIUS
                {
                    self.damage();
                }
            }
            self.rear_attack = if attack.age < impact + 0.65 {
                Some(attack)
            } else {
                None
            };
        }
    }
}
/// Offline QA only; never used by normal player controls.
pub fn diagnostic_pilot(g: &Game) -> Controls {
    let mut target = [0.0; 2];
    if !g.attacking() {
        if let Some(next) = g
            .obstacles
            .iter()
            .filter(|o| o.alive() && o.pos[2] + o.half[2] > g.distance - 2.0)
            .min_by(|a, b| a.pos[2].total_cmp(&b.pos[2]))
        {
            let z = next.pos[2];
            let mut best = f32::MAX;
            for ix in -6..=6 {
                for iy in -6..=6 {
                    let p = [ix as f32, iy as f32];
                    if p[0].hypot(p[1]) > 7.0 {
                        continue;
                    }
                    let blocked = g.obstacles.iter().any(|o| {
                        let margin = [
                            g.ship_half[0] * 1.35,
                            g.ship_half[1] + g.ship_half[0] * 0.45 + g.ship_half[2] * 0.15,
                            g.ship_half[2] + 0.15,
                        ];
                        let crosses = o
                            .hit([p[0], p[1], z - 5.0], [p[0], p[1], z + 24.0], margin)
                            .is_some();
                        let aimable = o.material == Material::Cargo
                            && (p[0] - o.pos[0]).abs() < o.half[0] - 0.3
                            && (p[1] - o.pos[1]).abs() < o.half[1] - 0.3;
                        crosses && !aimable
                    });
                    let threatened = g.rear_attack.is_some_and(|a| {
                        !a.resolved
                            && (p[0] - a.target[0]).abs() < 1.8
                            && (p[1] - a.target[1]).abs() < 1.4
                    });
                    if !blocked && !threatened {
                        let cost = (p[0] - g.pos[0]).powi(2)
                            + (p[1] - g.pos[1]).powi(2)
                            + (p[0] * p[0] + p[1] * p[1]) * 0.12;
                        if cost < best {
                            best = cost;
                            target = p;
                        }
                    }
                }
            }
        }
    }
    Controls {
        x: ((target[0] - g.pos[0]) * 2.8 - g.velocity[0] * 0.35).clamp(-1.0, 1.0),
        y: ((target[1] - g.pos[1]) * 2.8 - g.velocity[1] * 0.35).clamp(-1.0, 1.0),
        fire: true,
        boost: false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn empty() -> Game {
        let mut g = Game::default();
        g.start();
        g.obstacles.clear();
        g
    }
    fn obstacle(z: f32, m: Material) -> Obstacle {
        Obstacle {
            pos: [0.0, 0.0, z],
            half: [2.0; 3],
            material: m,
            hp: 3,
            flash: 0.0,
            contacted: false,
            seed: 0,
            shape: Shape::Panel,
            angle: 0.0,
            group: 0,
        }
    }
    #[test]
    fn selected_hull_survives_restart_and_bounds_the_rendered_pose() {
        use raylib::prelude::{Matrix, Vector3};
        let mut g = empty();
        g.ship_half = [0.9, 0.6, 1.4];
        g.intro();
        g.finish_video();
        g.start();
        assert_eq!(g.ship_half, [0.9, 0.6, 1.4]);
        g.bank = 0.42;
        g.pitch = -0.17;
        let bound = g.hull_half();
        let pose = Matrix::rotate_x(g.pitch) * Matrix::rotate_z(g.bank);
        for sx in [-1.0, 1.0] {
            for sy in [-1.0, 1.0] {
                for sz in [-1.0, 1.0] {
                    let p = Vector3::new(
                        sx * g.ship_half[0],
                        sy * g.ship_half[1],
                        sz * g.ship_half[2],
                    )
                    .transform_with(pose);
                    assert!(
                        p.x.abs() <= bound[0] + 0.0001
                            && p.y.abs() <= bound[1] + 0.0001
                            && p.z.abs() <= bound[2] + 0.0001
                    );
                }
            }
        }
        g.obstacles.clear();
        g.distance = 0.0;
        g.pos = [0.0; 2];
        g.update(
            STEP,
            Controls {
                fire: true,
                ..Controls::default()
            },
        );
        assert!(g.shots[0].pos[2] > g.distance + g.hull_half()[2]);
    }
    #[test]
    fn intro_returns_to_menu_and_menu_waits_for_start() {
        let mut g = Game::default();
        g.intro();
        g.finish_video(); // same transition for natural completion and Enter skip
        assert_eq!(g.phase, Phase::Briefing);
        g.update(
            1.0,
            Controls {
                fire: true,
                ..Controls::default()
            },
        );
        assert_eq!(g.distance, 0.0);
        assert!(g.shots.is_empty());
        g.start();
        assert_eq!(g.phase, Phase::Running);
        g.phase = Phase::Outro;
        g.finish_video();
        assert_eq!(g.phase, Phase::Won);
    }
    #[test]
    fn added_bulkheads_allow_thin_tall_and_wide_hulls_at_both_speeds() {
        for half in [
            [0.80, 0.20, 1.435],
            [0.39, 0.685, 0.85],
            [0.985, 0.675, 0.64],
            [0.84, 0.685, 1.01],
        ] {
            for boost in [false, true] {
                let mut g = Game {
                    ship_half: half,
                    ..Game::default()
                };
                g.start();
                for _ in 0..120 * 110 {
                    let mut input = diagnostic_pilot(&g);
                    input.boost = boost;
                    let hull = g.hull;
                    g.update(STEP, input);
                    if g.hull < hull {
                        println!(
                            "DAMAGE half={half:?} boost={boost} z={} pos={:?}",
                            g.distance, g.pos
                        );
                    }
                    if matches!(g.phase, Phase::Outro | Phase::Lost) {
                        break;
                    }
                }
                assert_eq!(g.phase, Phase::Outro);
                assert_eq!(g.hull, 3, "half {half:?} turbo {boost}");
            }
        }
    }
    #[test]
    fn six_full_bulkheads_keep_originals_and_add_four_separate_encounters() {
        let items = route();
        assert_eq!(BULKHEADS.len(), 6);
        assert_eq!(
            BULKHEADS
                .iter()
                .filter(|(z, _, _)| *z > REACTOR_Z / 2.0 && *z < 2055.0)
                .count(),
            2
        );
        for original in [2210.0, 2340.0] {
            assert!(BULKHEADS.iter().any(|(z, _, _)| *z == original));
        }
        for (i, (z, _, _)) in BULKHEADS.iter().enumerate() {
            let pieces: Vec<_> = items.iter().filter(|o| o.group == 26 + i).collect();
            assert!(pieces.iter().all(|o| (o.pos[2] - z).abs() < 2.0));
            // A solid section must still block shots and hull after all panels are broken.
            let solids: Vec<_> = pieces
                .iter()
                .filter(|o| o.material == Material::Armor)
                .collect();
            assert!(solids.iter().any(|o| o
                .hit([7.0, 0.0, z - 5.0], [7.0, 0.0, z + 5.0], [0.06; 3])
                .is_some()));
        }
        assert_eq!(
            items
                .iter()
                .filter(|o| o.group >= 26 && o.material == Material::Cargo)
                .count(),
            18
        );
    }
    #[test]
    fn all_six_bulkheads_have_independent_flyable_breakable_routes() {
        for group in 26..26 + BULKHEADS.len() {
            let gate: Vec<_> = route().into_iter().filter(|o| o.group == group).collect();
            let panels: Vec<_> = gate
                .iter()
                .filter(|o| o.material == Material::Cargo)
                .collect();
            assert_eq!(panels.len(), 3);
            for target in &panels {
                for boost in [false, true] {
                    let mut g = empty();
                    g.obstacles = gate.clone();
                    g.rear_encounters = 4;
                    g.distance = target.pos[2] - 65.0;
                    g.pos = [target.pos[0], target.pos[1]];
                    for _ in 0..300 {
                        g.update(
                            STEP,
                            Controls {
                                fire: true,
                                boost,
                                ..Controls::default()
                            },
                        );
                    }
                    assert!(g.distance > target.pos[2] + 4.0);
                    assert_eq!(
                        g.hull, 3,
                        "gate {group}, target {:?}, boost {boost}",
                        target.pos
                    );
                    assert_eq!(g.destroyed, 1);
                    assert_eq!(
                        g.obstacles
                            .iter()
                            .filter(|o| o.material == Material::Cargo && o.alive())
                            .count(),
                        2
                    );
                    assert!(g.obstacles.iter().all(|o| o
                        .hit(
                            [target.pos[0], target.pos[1], target.pos[2] - 5.0],
                            [target.pos[0], target.pos[1], target.pos[2] + 5.0],
                            [1.35, 1.0, 1.5]
                        )
                        .is_none()));
                }
            }
        }
    }
    #[test]
    fn open_lattice_has_real_holes_and_solid_diagonals() {
        let mut b = Builder {
            items: vec![],
            group: 0,
        };
        b.truss(0.0, 20.0);
        // Center of a triangular bay is empty; the diagonal crosses its top corner.
        let ray = |x, y| ([x, y, 10.0], [x, y, 30.0]);
        let (a, c) = ray(-7.5, -0.48);
        assert!(b.items.iter().all(|o| o.hit(a, c, [0.06; 3]).is_none()));
        let (a, c) = ray(-7.5, 0.0);
        assert!(b.items.iter().any(|o| o.hit(a, c, [0.06; 3]).is_some()));
        // Main flight opening above the complete lattice fits the banked ship.
        assert!(b.items.iter().all(|o| o
            .hit([-2.0, 3.0, 10.0], [-2.0, 3.0, 30.0], [1.35, 1.0, 1.5])
            .is_none()));
    }
    #[test]
    fn rotated_member_projects_hull_and_preserves_nearest_impact() {
        let mut b = Builder {
            items: vec![],
            group: 0,
        };
        b.bar([-3.0, -3.0], [3.0, 3.0], 20.0, 0.15, 0.2, Shape::Girder);
        let diagonal = &b.items[0];
        assert!(diagonal
            .hit([0.0, 0.0, 0.0], [0.0, 0.0, 40.0], [0.0; 3])
            .is_some());
        assert!(diagonal
            .hit([-2.0, 2.0, 0.0], [-2.0, 2.0, 40.0], [0.0; 3])
            .is_none());
        assert!(diagonal
            .hit([0.0, 1.0, 0.0], [0.0, 1.0, 40.0], [1.35, 1.0, 1.5])
            .is_some());
        let front = diagonal.hit([0.0; 3], [0.0, 0.0, 40.0], [0.06; 3]).unwrap();
        assert!(
            front
                < obstacle(25.0, Material::Cargo)
                    .hit([0.0; 3], [0.0, 0.0, 40.0], [0.06; 3])
                    .unwrap()
        );
    }
    #[test]
    fn destroyed_screen_leaves_portal_and_only_opens_its_own_space() {
        let mut b = Builder {
            items: vec![],
            group: 0,
        };
        b.portal(0.0, 0.0, 20.0);
        b.part([0.0, 0.0, 20.0], [2.8, 2.7, 0.24], 0.0, Shape::Panel);
        let hit = |o: &Obstacle, x| o.hit([x, 0.0, 10.0], [x, 0.0, 30.0], SHIP_HALF).is_some();
        assert!(b.items.iter().any(|o| hit(o, 0.0)));
        b.items.last_mut().unwrap().hp = 0;
        assert!(b.items.iter().all(|o| !hit(o, 0.0)));
        assert!(b.items.iter().any(|o| hit(o, 4.4)));
    }
    #[test]
    fn structural_group_causes_only_one_contact() {
        let mut g = empty();
        let mut b = Builder {
            items: vec![],
            group: 7,
        };
        b.bar([-3.0, 0.0], [3.0, 0.0], 10.0, 0.3, 0.3, Shape::Girder);
        b.bar([-3.0, 0.0], [3.0, 0.0], 80.0, 0.3, 0.3, Shape::Girder);
        g.obstacles = b.items;
        for _ in 0..360 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.hull, 2);
    }
    #[test]
    fn independent_axes_and_center() {
        let mut g = empty();
        assert_eq!(g.pos, [0.0; 2]);
        for _ in 0..60 {
            g.update(
                STEP,
                Controls {
                    x: 1.0,
                    ..Controls::default()
                },
            );
        }
        assert!(g.pos[0] > 3.0 && g.pos[1] == 0.0);
        for _ in 0..60 {
            g.update(
                STEP,
                Controls {
                    y: 1.0,
                    ..Controls::default()
                },
            );
        }
        assert!(g.pos[1] > 3.0);
    }
    #[test]
    fn diagonal_speed() {
        let (mut a, mut b) = (empty(), empty());
        for _ in 0..40 {
            a.update(
                STEP,
                Controls {
                    x: 1.0,
                    ..Controls::default()
                },
            );
            b.update(
                STEP,
                Controls {
                    x: 1.0,
                    y: 1.0,
                    ..Controls::default()
                },
            );
        }
        assert!((a.pos[0] - b.pos[0].hypot(b.pos[1])).abs() < 0.001);
    }
    #[test]
    fn swept_collision() {
        assert!(segment_box([0.0, 0.0, -50.0], [0.0, 0.0, 50.0], [0.0; 3], [1.0; 3]).is_some());
        assert!(segment_box([2.0, 0.0, -50.0], [2.0, 0.0, 50.0], [0.0; 3], [1.0; 3]).is_none());
    }
    #[test]
    fn one_contact_one_hull() {
        let mut g = empty();
        g.obstacles.push(obstacle(8.0, Material::Armor));
        for _ in 0..70 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.hull, 2);
    }
    #[test]
    fn armor_occludes_cargo() {
        let mut g = empty();
        g.obstacles = vec![
            obstacle(110.0, Material::Cargo),
            obstacle(85.0, Material::Armor),
        ];
        for _ in 0..120 {
            g.update(
                STEP,
                Controls {
                    fire: true,
                    ..Controls::default()
                },
            );
        }
        assert_eq!(g.obstacles[0].hp, 3);
        assert_eq!(g.destroyed, 0);
    }
    #[test]
    fn destroyed_cargo_no_collision() {
        let mut g = empty();
        g.obstacles.push(obstacle(65.0, Material::Cargo));
        for _ in 0..220 {
            g.update(
                STEP,
                Controls {
                    fire: true,
                    ..Controls::default()
                },
            );
        }
        assert_eq!(g.destroyed, 1);
        assert_eq!(g.hull, 3);
        assert!(!g.obstacles[0].alive());
    }
    #[test]
    fn pause() {
        let mut g = empty();
        g.toggle_pause();
        g.update(
            2.0,
            Controls {
                x: 1.0,
                fire: true,
                ..Controls::default()
            },
        );
        assert_eq!(g.distance, 0.0);
        assert_eq!(g.time, 0.0);
        assert!(g.shots.is_empty());
        g.toggle_pause();
        g.update(STEP, Controls::default());
        assert!(g.distance > 0.0);
    }
    #[test]
    fn walls() {
        let mut g = empty();
        for _ in 0..180 {
            g.update(
                STEP,
                Controls {
                    x: 1.0,
                    y: 1.0,
                    ..Controls::default()
                },
            );
        }
        assert!(g.hull < 3);
        assert!(g.pos[0].hypot(g.pos[1]) < TUNNEL_RADIUS);
    }
    #[test]
    fn manual_reactor_and_outro() {
        let mut g = empty();
        g.distance = ATTACK_Z;
        g.pos = [5.0, 0.0];
        for _ in 0..400 {
            g.update(
                STEP,
                Controls {
                    fire: true,
                    ..Controls::default()
                },
            );
        }
        assert_eq!(g.reactor_hp, 6);
        assert_eq!(g.distance, ATTACK_Z);
        assert!(g.travel > 100.0);
        g.pos = [0.0; 2];
        for _ in 0..700 {
            g.update(
                STEP,
                Controls {
                    fire: true,
                    ..Controls::default()
                },
            );
        }
        assert_eq!(g.phase, Phase::Outro);
        assert_eq!(g.reactor_hp, 0);
        use crate::{
            gameplay::{rewards::record_result, selection::{GameState, ShipCatalog}},
            minigames::common::host::AttemptReporter,
            ships::ShipsConfig,
        };
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
        let mut adventure = GameState::new_game("minecraft", &catalog).unwrap();
        let mut reporter = AttemptReporter::default();
        let mut observer = |result| {
            record_result(&mut adventure, "starwars", result, &catalog)?;
            Ok(())
        };
        // Award before the outro ends, including if the player restarts it.
        reporter.observe(&g, &mut observer).unwrap();
        reporter.observe(&g, &mut observer).unwrap();
        g.intro();
        reporter.observe(&g, &mut observer).unwrap();
        g.finish_video();
        g.start();
        for _ in 0..3 {
            g.invulnerability = 0.0;
            g.damage();
        }
        assert_eq!(g.phase, Phase::Lost);
        reporter.observe(&g, &mut observer).unwrap();
        assert_eq!(adventure.progress().token_count(), 1);
        assert!(adventure.is_unlocked("starwars"));
        assert_eq!(adventure.active_ship(), "minecraft");
    }
    #[test]
    fn approach_lasers_cannot_trigger_reactor_early() {
        let mut g = empty();
        g.distance = ATTACK_Z - 70.0;
        g.shots.push(Shot {
            pos: [0.0, 0.0, REACTOR_Z - 0.2],
            origin_z: ATTACK_Z - 80.0,
            torpedo: false,
        });
        g.update(STEP, Controls::default());
        assert_eq!(g.reactor_hp, 6);
        assert_eq!(g.phase, Phase::Running);
    }
    #[test]
    fn route_with_turbo() {
        let mut g = Game::default();
        g.start();
        for _ in 0..120 * 85 {
            let mut input = diagnostic_pilot(&g);
            input.boost = true;
            g.update(STEP, input);
            if g.phase == Phase::Outro {
                break;
            }
        }
        assert_eq!(g.phase, Phase::Outro);
        assert_eq!(g.hull, 3);
        assert_eq!(g.distance, ATTACK_Z);
        assert_eq!(g.rear_encounters, 4);
    }
    #[test]
    fn rear_warning_locks_position_and_can_be_evaded() {
        let mut g = empty();
        g.distance = REAR_ENCOUNTERS[0];
        g.update(STEP, Controls::default());
        let target = g.rear_attack.unwrap().target;
        for _ in 0..100 {
            g.update(
                STEP,
                Controls {
                    x: 0.5,
                    ..Controls::default()
                },
            );
        }
        assert_eq!(g.rear_attack.unwrap().target, target);
        for _ in 0..220 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.hull, 3);
        assert!(g.rear_attack.unwrap().resolved);
    }
    #[test]
    fn rear_bolt_damages_once_after_warning() {
        let mut g = empty();
        g.distance = REAR_ENCOUNTERS[0];
        for _ in 0..190 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.hull, 3);
        assert!(g.rear_attack.unwrap().warning());
        for _ in 0..150 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.hull, 2);
        assert_eq!(g.rear_encounters, 1);
        for _ in 0..60 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.hull, 2);
    }
    #[test]
    fn rear_pause_and_restart() {
        let mut g = empty();
        g.distance = REAR_ENCOUNTERS[0];
        g.update(STEP, Controls::default());
        let age = g.rear_attack.unwrap().age;
        g.toggle_pause();
        g.update(1.0, Controls::default());
        assert_eq!(g.rear_attack.unwrap().age, age);
        g.start();
        assert!(g.rear_attack.is_none());
        assert_eq!(g.rear_encounters, 0);
    }
    #[test]
    fn restart() {
        let mut g = empty();
        g.hull = 0;
        g.phase = Phase::Lost;
        g.start();
        assert_eq!(g.hull, 3);
        assert!(g.obstacles.len() > 60);
        assert_eq!(g.pos, [0.0; 2]);
        g.intro();
        assert_eq!(g.phase, Phase::Intro);
    }
    #[test]
    fn complete_route() {
        let mut g = Game::default();
        g.start();
        for _ in 0..120 * 100 {
            let input = diagnostic_pilot(&g);
            g.update(STEP, input);
            if g.phase == Phase::Outro {
                break;
            }
        }
        assert_eq!(
            g.phase,
            Phase::Outro,
            "distance {} hull {}",
            g.distance,
            g.hull
        );
        assert_eq!(g.hull, 3);
        assert!(g.destroyed >= 2);
        assert_eq!(g.rear_encounters, 4);
    }
    #[test]
    fn no_auto_pilot() {
        let mut g = Game::default();
        g.start();
        for _ in 0..120 * 20 {
            g.update(STEP, Controls::default());
        }
        assert_eq!(g.pos, [0.0; 2]);
        assert!(g.hull < 3);
        assert_eq!(g.shots_fired, 0);
    }
}
