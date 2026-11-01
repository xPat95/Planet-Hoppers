//! Escala del espacio: los JSON conservan las unidades de diseño originales.
//! Se aplica una sola vez al cargar las configuraciones, nunca a los minijuegos.

/// Cuerpos y órbitas crecen juntos; las naves conservan su tamaño.
pub const WORLD_SCALE: f32 = 12.0;

/// Los viajes duran sqrt(escala) veces más, en lugar de escala veces más.
/// Velocidades lineales y aceleración comparten este factor; las velocidades
/// angulares orbitales se dividen por él para mantener alcanzables los planetas.
pub fn travel_scale() -> f32 {
    WORLD_SCALE.sqrt()
}
