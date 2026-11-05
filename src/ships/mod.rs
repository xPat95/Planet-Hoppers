//! Naves: modelos disponibles y vuelo por el sistema solar (fase 5).
//!
//! Solo lógica, sin ventana. La nave avanza hacia donde apunta su nariz, con aceleración y turbo; choca contra el
//! Sol y la forma de cada planeta; y cerca de un planeta lo acompaña en su órbita (lo "arrastra"), así que es
//! posible alcanzar planetas en movimiento y quedarse junto a ellos. Todo usa deltaTime.
//! El flujo normal recibe la propiedad de naves desde `gameplay::selection`; el acceso directo de desarrollo
//! usa la nave activa de `config/ships.json` y conserva su catálogo completo.
use crate::solar_system::{scale::travel_scale, SolarSystem};
use raylib::prelude::Vector3;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ShipModel {
    pub id: String,
    pub name: String,
    /// GLB u OBJ relativo a la raíz del repositorio.
    pub model: String,
    /// Largo en el juego (el modelo se escala a esto).
    pub length: f32,
    /// Giro extra para que la nariz quede hacia -Z y el techo hacia +Y (grados en X, Y, Z).
    #[serde(default)]
    pub model_rotation_degrees: [f32; 3],
}

impl ShipModel {
    /// Flight framing only: the slender N-1 needs a little more screen space.
    /// Model fitting, hangar scale and physical dimensions stay unchanged.
    pub fn camera_length(&self) -> f32 {
        if self.model.contains("/starwars/") {
            self.length * 0.60
        } else {
            self.length
        }
    }
    /// Correct the prepared N-1's +Z nose only while loading the flight view.
    pub fn flight_rotation_degrees(&self) -> [f32; 3] {
        let mut rotation = self.model_rotation_degrees;
        if self.model.contains("/starwars/") {
            rotation[1] += 180.0;
        }
        rotation
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlightConfig {
    /// Velocidad normal y con turbo (unidades por segundo).
    pub cruise_speed: f32,
    pub boost_speed: f32,
    /// Cuánto cambia la velocidad por segundo al acelerar o frenar.
    pub acceleration: f32,
    /// Giro con el teclado (grados por segundo).
    pub turn_speed_degrees: f32,
    pub mouse_sensitivity: f32,
    /// Radio de choque de la nave.
    pub radius: f32,
    /// Hasta qué distancia (en radios del planeta, desde su centro) la nave acompaña la órbita del planeta.
    pub carry_range: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShipsConfig {
    pub active_ship: String,
    pub ships: Vec<ShipModel>,
    pub flight: FlightConfig,
}

impl ShipsConfig {
    /// Carga unidades de diseño y devuelve velocidades de juego; no escala las naves.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config: Self = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        let travel = travel_scale();
        config.flight.cruise_speed *= travel;
        config.flight.boost_speed *= travel;
        config.flight.acceleration *= travel;
        if config.active().is_none() {
            return Err(format!("active_ship '{}' no está en la lista de naves", config.active_ship).into());
        }
        let f = &config.flight;
        if !(f.cruise_speed > 0.0 && f.boost_speed >= f.cruise_speed && f.acceleration > 0.0 && f.radius > 0.0) {
            return Err("Velocidades, aceleración y radio de la nave tienen que ser positivos".into());
        }
        Ok(config)
    }

    pub fn active(&self) -> Option<&ShipModel> {
        self.ships.iter().find(|s| s.id == self.active_ship)
    }
}

/// Lo que pide el jugador en este cuadro.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlightInput {
    /// Ascenso vertical en el eje Y del mundo, independiente de la orientación.
    pub ascend: bool,
    /// -1 (frenar / reversa) a 1 (acelerar).
    pub throttle: f32,
    pub boost: bool,
    /// Giro con el teclado: -1 a 1 en cada eje (izquierda/derecha, abajo/arriba).
    pub turn: (f32, f32),
    /// Movimiento del mouse en píxeles (x, y).
    pub mouse: (f32, f32),
    /// Si hay un punto al que apuntar (piloto automático), la nariz gira sola hacia él.
    pub aim_at: Option<Vector3>,
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub position: Vector3,
    /// Rumbo (alrededor de Y) e inclinación de la nariz, en radianes.
    pub yaw: f32,
    pub pitch: f32,
    /// Velocidad hacia adelante (negativa = reversa).
    pub speed: f32,
    /// Ladeo visual al girar (radianes); no cambia el rumbo.
    pub bank: f32,
    /// Planeta que la está arrastrando en su órbita, si hay.
    pub carried_by: Option<usize>,
    /// Chocó en este cuadro.
    pub bumped: bool,
}

const PITCH_LIMIT: f32 = 1.4;

impl Ship {
    pub fn new(position: Vector3, yaw: f32) -> Self {
        Self { position, yaw, pitch: 0.0, speed: 0.0, bank: 0.0, carried_by: None, bumped: false }
    }

    /// Hacia donde apunta la nariz. Con rumbo 0 mira hacia -Z, como los modelos exportados.
    pub fn forward(&self) -> Vector3 {
        Vector3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    /// Apunta la nariz hacia `target` sin moverse.
    pub fn look_at(&mut self, target: Vector3) {
        let d = (target - self.position).normalized();
        self.yaw = (-d.x).atan2(-d.z);
        self.pitch = d.y.clamp(-1.0, 1.0).asin().clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Predicción sobre la órbita circular, compartida por el juego y las pruebas.
    pub fn autopilot_aim(&self, index: usize, flight: &FlightConfig, system: &SolarSystem) -> Vector3 {
        let p = &system.planets[index];
        let eta = (p.position() - self.position).length() / self.speed.abs().max(flight.cruise_speed);
        let angle = p.orbit_angle + p.config.orbit_speed_degrees.to_radians() * eta;
        let target = Vector3::new(angle.cos(), 0.0, -angle.sin()) * p.config.orbit_radius;
        self.autopilot_to(target, flight, system)
    }

    /// Ruta compartida para objetivos orbitales y puntos de interés fijos.
    pub fn autopilot_to(&self, target: Vector3, flight: &FlightConfig, system: &SolarSystem) -> Vector3 {
        // Un destino al otro lado del Sol necesita un rodeo. La ruta directa
        // dejaría al piloto empujando contra su colisión, especialmente en normal.
        let route = target - self.position;
        let t = (-self.position).dot(route) / route.dot(route).max(1.0);
        let closest = self.position + route * t.clamp(0.0, 1.0);
        let clearance = system.sun.radius * 1.35 + flight.radius;
        if (0.0..1.0).contains(&t) && closest.length() < clearance {
            let direction = route.normalized();
            let up = Vector3::up() - direction * direction.y;
            let lift = if up.dot(up) > 0.01 { up.normalized() } else { Vector3::right() };
            closest + lift * clearance * 1.5
        } else {
            target
        }
    }

    pub fn update(&mut self, dt: f32, input: &FlightInput, flight: &FlightConfig, system: &SolarSystem) {
        // Giro: mouse (ya es un ángulo por cuadro, no depende de dt) + teclado (grados por segundo)
        let turn = flight.turn_speed_degrees.to_radians() * dt;
        let yaw_step = -input.mouse.0 * flight.mouse_sensitivity - input.turn.0 * turn;
        let mut pitch_step = -input.mouse.1 * flight.mouse_sensitivity + input.turn.1 * turn;
        let mut yaw_step = yaw_step;
        if let Some(target) = input.aim_at {
            // Piloto automático: gira hacia el objetivo a la velocidad del teclado, sin pasarse
            let d = (target - self.position).normalized();
            let want_yaw = (-d.x).atan2(-d.z);
            let want_pitch = d.y.clamp(-1.0, 1.0).asin();
            let dyaw = (want_yaw - self.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            yaw_step += dyaw.clamp(-turn * 1.5, turn * 1.5);
            pitch_step += (want_pitch - self.pitch).clamp(-turn * 1.5, turn * 1.5);
        }
        self.yaw = (self.yaw + yaw_step).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + pitch_step).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        // Se ladea hacia el lado del giro y vuelve solo (suavizado que no depende de los FPS)
        let rate = if dt > 0.0 { yaw_step / dt } else { 0.0 };
        let target_bank = (rate * 0.35).clamp(-0.6, 0.6);
        self.bank += (target_bank - self.bank) * (1.0 - (-6.0 * dt).exp());

        // Velocidad: acelera hacia la pedida; sin acelerador va frenando poco a poco
        let top = if input.boost { flight.boost_speed } else { flight.cruise_speed };
        let wanted = if input.throttle >= 0.0 { input.throttle * top } else { input.throttle * flight.cruise_speed * 0.4 };
        let step = flight.acceleration * dt * if input.boost { 2.0 } else { 1.0 };
        let wanted = if input.throttle == 0.0 { self.speed * (-0.6 * dt).exp() } else { wanted };
        self.speed += (wanted - self.speed).clamp(-step, step);

        // Arrastre: dentro del alcance de un planeta, la nave se mueve con él además de su propio vuelo
        self.carried_by = system
            .planets
            .iter()
            .enumerate()
            .filter(|(_, p)| (self.position - p.position()).length() < p.config.radius * flight.carry_range)
            .min_by(|a, b| {
                let da = (self.position - a.1.position()).length() / a.1.config.radius;
                let db = (self.position - b.1.position()).length() / b.1.config.radius;
                da.total_cmp(&db)
            })
            .map(|(i, _)| i);
        let carry = self.carried_by.map_or(Vector3::zero(), |i| system.planets[i].velocity());
        let ascent = if input.ascend { Vector3::up() * top } else { Vector3::zero() };
        self.position += (self.forward() * self.speed + carry + ascent) * dt;
        self.resolve_collisions(flight, system);
    }

    /// Saca a la nave del Sol y de los planetas. Si iba hacia adentro pierde casi toda la velocidad.
    pub fn resolve_collisions(&mut self, flight: &FlightConfig, system: &SolarSystem) {
        self.bumped = false;
        let sun = system.center();
        let min = system.sun.radius + flight.radius;
        let rel = self.position - sun;
        if rel.length() < min {
            let normal = if rel.length() > 1e-4 { rel.normalized() } else { Vector3::up() };
            self.position = sun + normal * min;
            self.hit(normal);
        }
        for p in &system.planets {
            if let Some(out) = p.push_out(self.position, flight.radius) {
                let normal = (out - self.position).normalized();
                self.position = out;
                self.hit(normal);
            }
        }
    }

    fn hit(&mut self, normal: Vector3) {
        self.bumped = true;
        if (self.forward() * self.speed).dot(normal) < 0.0 {
            self.speed *= 0.1;
        }
    }
}
