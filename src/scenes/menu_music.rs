//! La música y los sonidos de afuera de los minijuegos. "Sky Warriors" suena en la pantalla de inicio, el hangar y el
//! sistema solar; además, un golpe al cambiar de nave en el hangar y un crescendo mientras carga el sistema solar.
//! Cada una
//! de esas pantallas abre su propia ventana y su propio dispositivo de audio (se cierra antes de entrar a un minijuego,
//! que abre el suyo), así que la canción recuerda por dónde iba y sigue desde ahí. Las pruebas automáticas van en
//! silencio. Si no hay dispositivo de audio o falta el archivo, se sigue sin música.
use raylib::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

const TRACK: &str = "assets/audio/music/Menu/sky_warriors.mp3";
const VOLUME: f32 = 0.35;
const SWITCH: (&str, f32) = ("assets/audio/sfx/Menu/ship_switch.mp3", 0.4);
const RISER: (&str, f32) = ("assets/audio/sfx/Menu/solar_riser.mp3", 0.5);
/// Por dónde iba la canción (los bits del `f32`, en segundos).
static POSITION: AtomicU32 = AtomicU32::new(0);

pub struct MenuMusic<'a> {
    music: Option<Music<'a>>,
}

impl<'a> MenuMusic<'a> {
    pub fn start(device: Option<&'a RaylibAudio>, root: &Path) -> Self {
        let music = device.and_then(|d| d.new_music(&root.join(TRACK).to_string_lossy()).ok()).map(|mut m| {
            m.looping = true;
            m.set_volume(VOLUME);
            m.play_stream();
            let position = f32::from_bits(POSITION.load(Ordering::Relaxed));
            if position > 0.0 {
                m.seek_stream(position);
            }
            m
        });
        Self { music }
    }

    /// Calla la canción (mientras suena el video de una abducción) o la sigue desde donde quedó.
    pub fn set_paused(&self, paused: bool) {
        if let Some(m) = &self.music {
            if paused { m.pause_stream() } else { m.resume_stream() }
        }
    }

    /// Cada cuadro: le da más audio al dispositivo y anota por dónde va.
    pub fn update(&self) {
        if let Some(m) = &self.music {
            m.update_stream();
            POSITION.store(m.get_time_played().to_bits(), Ordering::Relaxed);
        }
    }
}

/// Un sonido suelto (el golpe del hangar, el crescendo de la carga), listo para tocar.
pub fn effect<'a>(device: Option<&'a RaylibAudio>, root: &Path, (file, volume): (&str, f32)) -> Option<Sound<'a>> {
    device.and_then(|d| d.new_sound(&root.join(file).to_string_lossy()).ok()).inspect(|s| s.set_volume(volume))
}

pub fn ship_switch<'a>(device: Option<&'a RaylibAudio>, root: &Path) -> Option<Sound<'a>> {
    effect(device, root, SWITCH)
}

pub fn riser<'a>(device: Option<&'a RaylibAudio>, root: &Path) -> Option<Sound<'a>> {
    effect(device, root, RISER)
}
