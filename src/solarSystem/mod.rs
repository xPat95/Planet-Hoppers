//! Sistema solar: el Sol en el centro y seis planetas en órbitas circulares.
//!
//! Solo lógica, sin ventana: posiciones, órbitas, traslación, rotación propia (fases 1 y 2), formas de choque y
//! proximidad (fases 5 y 6). Los valores salen de `config/solar-system.json` y todo avanza con deltaTime, así que
//! la velocidad no depende de los FPS. La escena `scenes::solar_system` lo dibuja con los modelos de cada planeta.
use raylib::prelude::{Matrix, Vector3};
use serde::Deserialize;
use std::{collections::HashSet, f32::consts::TAU, path::Path};

pub mod scale;

/// Franquicias del juego, en el mismo orden que las carpetas de `assets/models/planets/`.
pub const PLANET_IDS: [&str; 6] = [
    "minecraft",
    "southpark",
    "smilingfriends",
    "halloween",
    "regularshow",
    "starwars",
];

#[derive(Debug, Clone, Deserialize)]
pub struct SunConfig {
    pub radius: f32,
    pub color: String,
    /// Grados por segundo sobre su propio eje.
    pub rotation_speed_degrees: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanetConfig {
    pub id: String,
    pub name: String,
    /// Distancia del centro del Sol al centro del planeta.
    pub orbit_radius: f32,
    pub radius: f32,
    /// Traslación: grados por segundo alrededor del Sol (positivo = antihorario visto desde arriba).
    pub orbit_speed_degrees: f32,
    /// Rotación propia: grados por segundo (negativo = gira al revés).
    pub rotation_speed_degrees: f32,
    /// Inclinación del eje respecto a la vertical.
    pub axial_tilt_degrees: f32,
    /// Ángulo inicial en la órbita, para que no arranquen todos alineados.
    pub start_angle_degrees: f32,
    /// Color de la esfera temporal y de su etiqueta (#rrggbb).
    pub color: String,
    /// GLB del planeta (relativo a la raíz del repositorio). Sin modelo se dibuja la esfera temporal.
    #[serde(default)]
    pub model: Option<String>,
    /// Versión liviana para cuando se ve de lejos.
    #[serde(default)]
    pub model_far: Option<String>,
    /// Giro extra del modelo (grados en X, Y, Z) para orientarlo sin tocar el archivo original.
    #[serde(default)]
    pub model_rotation_degrees: [f32; 3],
    /// Minijuego que abre "Explorar planeta" (`None` si todavía no existe).
    #[serde(default)]
    pub minigame: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolarSystemConfig {
    pub sun: SunConfig,
    pub planets: Vec<PlanetConfig>,
    /// Alcance del botón "Explorar planeta", en radios del planeta medidos desde su superficie.
    #[serde(default = "default_interaction")]
    pub interaction_distance: f32,
}

fn default_interaction() -> f32 {
    0.9
}

impl SolarSystemConfig {
    /// Carga unidades de diseño y devuelve dimensiones y velocidades de juego.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config: Self = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        config.sun.radius *= scale::WORLD_SCALE;
        for planet in &mut config.planets {
            planet.radius *= scale::WORLD_SCALE;
            planet.orbit_radius *= scale::WORLD_SCALE;
            planet.orbit_speed_degrees /= scale::travel_scale();
        }
        config.validate()?;
        Ok(config)
    }

    /// Revisa que estén los seis planetas y que ninguna esfera choque con el Sol ni con su vecina.
    pub fn validate(&self) -> Result<(), String> {
        if !(self.sun.radius > 0.0) {
            return Err("El radio del Sol tiene que ser positivo".into());
        }
        hex_color(&self.sun.color)?;
        let ids: HashSet<&str> = self.planets.iter().map(|p| p.id.as_str()).collect();
        if self.planets.len() != PLANET_IDS.len() || PLANET_IDS.iter().any(|id| !ids.contains(id)) {
            return Err(format!("Tienen que estar los seis planetas: {}", PLANET_IDS.join(", ")));
        }
        let mut inner_edge = self.sun.radius;
        for p in &self.planets {
            if !(p.radius > 0.0) {
                return Err(format!("{}: el radio tiene que ser positivo", p.id));
            }
            hex_color(&p.color).map_err(|e| format!("{}: {e}", p.id))?;
            // Las órbitas van de adentro hacia afuera y cada planeta cabe entero entre sus vecinos
            if p.orbit_radius - p.radius <= inner_edge {
                return Err(format!("{}: la órbita {} choca con lo que está adentro", p.id, p.orbit_radius));
            }
            inner_edge = p.orbit_radius + p.radius;
        }
        Ok(())
    }
}

/// Convierte "#rrggbb" en sus tres canales.
pub fn hex_color(hex: &str) -> Result<[u8; 3], String> {
    let digits = hex.strip_prefix('#').filter(|d| d.len() == 6);
    let parse = |i: usize| digits.and_then(|d| u8::from_str_radix(&d[i..i + 2], 16).ok());
    match (parse(0), parse(2), parse(4)) {
        (Some(r), Some(g), Some(b)) => Ok([r, g, b]),
        _ => Err(format!("Color inválido: {hex} (se espera #rrggbb)")),
    }
}

#[derive(Debug, Clone)]
pub struct Planet {
    pub config: PlanetConfig,
    /// Ángulo actual en la órbita (radianes, entre 0 y 2π).
    pub orbit_angle: f32,
    /// Ángulo actual sobre su propio eje (radianes, entre 0 y 2π).
    pub spin_angle: f32,
    /// Semiejes de la forma de choque en el espacio del planeta (Y = su eje). Empieza como una esfera del radio;
    /// la escena la ajusta a la caja del modelo (el disco de Smiling Friends es más plano que una esfera).
    pub collision_radii: Vector3,
}

impl Planet {
    /// Centro del planeta. Las órbitas son círculos en el plano XZ alrededor del origen.
    pub fn position(&self) -> Vector3 {
        let r = self.config.orbit_radius;
        Vector3::new(r * self.orbit_angle.cos(), 0.0, -r * self.orbit_angle.sin())
    }

    /// Eje de rotación propia: la vertical inclinada hacia +X.
    pub fn spin_axis(&self) -> Vector3 {
        let tilt = self.config.axial_tilt_degrees.to_radians();
        Vector3::new(tilt.sin(), tilt.cos(), 0.0)
    }

    /// Giro del planeta (del espacio del planeta al mundo): primero sobre su eje y después la inclinación hacia +X.
    pub fn orientation(&self) -> Matrix {
        Matrix::rotate_y(self.spin_angle) * Matrix::rotate_z(-self.config.axial_tilt_degrees.to_radians())
    }

    /// Velocidad del centro del planeta en su órbita (unidades por segundo): tangente al círculo.
    pub fn velocity(&self) -> Vector3 {
        let w = self.config.orbit_speed_degrees.to_radians() * self.config.orbit_radius;
        Vector3::new(-w * self.orbit_angle.sin(), 0.0, -w * self.orbit_angle.cos())
    }

    /// Distancia "en radios" de un punto a la forma del planeta: 1 en la superficie, menos de 1 adentro.
    /// `margin` agranda la forma (por ejemplo, el radio de la nave).
    pub fn shape_distance(&self, point: Vector3, margin: f32) -> f32 {
        let local = (point - self.position()).transform_with(self.orientation().transposed());
        let r = self.collision_radii + Vector3::one() * margin;
        Vector3::new(local.x / r.x, local.y / r.y, local.z / r.z).length()
    }

    /// Si el punto está dentro de la forma (agrandada en `margin`), el punto más cercano sobre su borde.
    pub fn push_out(&self, point: Vector3, margin: f32) -> Option<Vector3> {
        let d = self.shape_distance(point, margin);
        if d >= 1.0 {
            return None;
        }
        let center = self.position();
        // En el centro exacto no hay dirección: se sale hacia arriba
        let rel = if d < 1e-4 { self.spin_axis() * 1e-3 } else { point - center };
        let k = 1.0 / self.shape_distance(center + rel, margin);
        Some(center + rel * k)
    }
}

#[derive(Debug, Clone)]
pub struct SolarSystem {
    pub sun: SunConfig,
    pub sun_spin_angle: f32,
    pub planets: Vec<Planet>,
    pub interaction_distance: f32,
}

impl SolarSystem {
    pub fn new(config: &SolarSystemConfig) -> Self {
        Self {
            sun: config.sun.clone(),
            sun_spin_angle: 0.0,
            planets: config
                .planets
                .iter()
                .map(|p| Planet {
                    config: p.clone(),
                    orbit_angle: p.start_angle_degrees.to_radians().rem_euclid(TAU),
                    spin_angle: 0.0,
                    collision_radii: Vector3::one() * p.radius,
                })
                .collect(),
            interaction_distance: config.interaction_distance,
        }
    }

    /// Centro del sistema: el Sol está fijo en el origen.
    pub fn center(&self) -> Vector3 {
        Vector3::zero()
    }

    /// Punto de partida en medio del espacio libre entre el Sol y la primera órbita.
    pub fn departure_position(&self) -> Vector3 {
        let first = &self.planets[0];
        let distance = (self.sun.radius + first.config.orbit_radius - first.config.radius) * 0.5;
        first.position().normalized() * distance
    }

    /// Avanza la traslación y la rotación `dt` segundos. Todo se multiplica por dt, así que un paso de un segundo
    /// deja a los planetas donde mismo que sesenta pasos de 1/60.
    pub fn update(&mut self, dt: f32) {
        self.sun_spin_angle = advance(self.sun_spin_angle, self.sun.rotation_speed_degrees, dt);
        for p in &mut self.planets {
            p.orbit_angle = advance(p.orbit_angle, p.config.orbit_speed_degrees, dt);
            p.spin_angle = advance(p.spin_angle, p.config.rotation_speed_degrees, dt);
        }
    }

    pub fn planet(&self, id: &str) -> Option<&Planet> {
        self.planets.iter().find(|p| p.config.id == id)
    }

    /// Planeta que se puede explorar desde `point`: el más cercano cuya superficie está dentro del alcance.
    pub fn planet_in_reach(&self, point: Vector3) -> Option<usize> {
        self.planets
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.shape_distance(point, 0.0)))
            .filter(|(_, d)| *d <= 1.0 + self.interaction_distance)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }
}

fn advance(angle: f32, degrees_per_second: f32, dt: f32) -> f32 {
    (angle + degrees_per_second.to_radians() * dt).rem_euclid(TAU)
}
