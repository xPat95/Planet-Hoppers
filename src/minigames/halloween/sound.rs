//! Música y sonidos del minijuego de Halloween. "Exploring" (Jeremy Korpas) suena en la historieta, la práctica y la
//! advertencia, y otra vez en el baile del final; "Conspiracy Theory" (Rod Kim), en el juego. Mientras una criatura se
//! acerca se la oye: cada una con su propia voz (de las de su tipo, que se turnan), más fuerte cuanto más cerca y del
//! lado de donde viene, aunque todavía no se la vea. Cada criatura grita cuando se tira a la cara (de verdad, con un
//! alarido encima; de mentira, con una voz enojada) y el grito se apaga apenas termina el susto; la gente grita cuando
//! cae el cartel de cada oleada y se apaga con el cartel. Si no hay dispositivo de audio o falta un archivo, el juego
//! sigue sin sonido.
use super::{Kind, Session, State};
use raylib::prelude::*;
use std::path::Path;

/// Qué música va: la de la intro, la del juego, o ninguna (pausado: baja).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Track {
    Intro,
    Game,
}

const INTRO_VOLUME: f32 = 0.35;
const GAME_VOLUME: f32 = 0.45;
/// En pausa, la música sigue pero bajita.
const PAUSED_VOLUME: f32 = 0.12;
/// Las voces de cada tipo (en el orden de `Kind`): archivo, ganancia (medida para que todas suenen parejo, un promedio
/// de unos -16 dB; las más bajitas se quedan en 2,2 para no saturar) y hasta qué segundo se oye: después solo queda una
/// cola casi muda, así que ahí se corta y se pasa a la siguiente, y la criatura nunca se queda callada.
const VOICES: [&[(&str, f32, f32)]; 3] = [
    &[("devil_laugh_1.mp3", 2.05, 2.5), ("devil_laugh_2.mp3", 1.6, 4.5), ("devil_evil_woman.mp3", 2.2, 2.0)],
    &[("ghost_i_remember.mp3", 1.55, 3.0), ("ghost_evil_woman.mp3", 2.2, 2.5), ("ghost_not_enough.mp3", 1.3, 3.0)],
    &[("demon_possessed.mp3", 1.6, 5.5), ("demon_tensing.mp3", 2.0, 2.0), ("alien_grunt_hiss.mp3", 1.4, 4.5)],
];
/// Cada vez que suena una voz, va con un tono al azar entre estos (más grave y lenta, o más aguda y rápida), lejos del
/// anterior de su tipo: así la misma voz no suena igual dos veces.
const VOICE_PITCH: (f32, f32) = (0.78, 1.25);
const PITCH_APART: f32 = 0.1;
/// Qué tan rápido siguen las voces al volumen que les toca, y se apagan cuando no hay nadie (por segundo).
const VOICE_FADE: f32 = 6.0;
/// El grito de cada criatura (en el orden de `Kind`), con su volumen.
const ROARS: [(&str, f32); 3] = [("alien_roar.mp3", 0.8), ("zombie_roar.mp3", 0.8), ("alien_grunt_hiss.mp3", 0.85)];
/// Los tonos con que se turna el grito de cada criatura (más grave, normal, más agudo), para que no suene igual dos
/// veces seguidas.
const ROAR_PITCHES: [f32; 3] = [0.82, 1.0, 1.2];
/// Volumen del alarido o la voz de encima, cuánto dura el susto y cuánto tarda en apagarse después.
const SCREAM_VOLUME: f32 = 0.7;
const ANNOYED_VOLUME: f32 = 0.75;
const STING_HOLD: f32 = 0.55;
const STING_FADE: f32 = 0.45;
/// La gente que grita: suena lo que dura el cartel de la oleada y se apaga.
const CROWD_VOLUME: f32 = 0.55;
const CROWD_HOLD: f32 = 2.6;
const CROWD_FADE: f32 = 0.8;

/// Una voz cargada: de qué criatura es ahora (ninguna: libre, o apagándose), hace cuánto empezó y el volumen que tiene.
struct Voice<'a> {
    sound: Sound<'a>,
    gain: f32,
    audible: f32,
    owner: Option<u32>,
    age: f32,
    level: f32,
    /// El tono con que está sonando (también cambia lo que tarda: más agudo, más rápido).
    pitch: f32,
}

/// Un grito que suena: de qué criatura, si es de verdad y hace cuánto empezó.
struct Sting {
    roar: usize,
    real: bool,
    age: f32,
}

pub struct Sounds<'a> {
    intro: Option<Music<'a>>,
    game: Option<Music<'a>>,
    playing: Option<Track>,
    roars: [Option<Sound<'a>>; 3],
    crowd: Option<Sound<'a>>,
    crowd_age: Option<f32>,
    scream: Option<Sound<'a>>,
    annoyed: Option<Sound<'a>>,
    sting: Option<Sting>,
    /// Qué tono de `ROAR_PITCHES` le toca al próximo grito de cada criatura.
    next_roar: [usize; 3],
    /// Las voces de cada tipo de criatura, cuál toca después y el último tono de cada tipo.
    voices: [Vec<Voice<'a>>; 3],
    next_voice: [usize; 3],
    last_pitch: [f32; 3],
    rng: u32,
    muted: bool,
}

/// Qué tan fuerte se oye una criatura que viene, según la distancia: se la oye ya de lejos y del todo en la cara.
fn closeness(distance: f32, far: f32, hit: f32) -> f32 {
    let k = ((far - distance) / (far - hit)).clamp(0.0, 1.0);
    0.22 + 0.78 * k.powf(1.3)
}

/// Cuánto queda de un sonido que se mantiene `hold` segundos y después se apaga en `fade`.
fn fading(age: f32, hold: f32, fade: f32) -> f32 {
    (1.0 - (age - hold) / fade).clamp(0.0, 1.0)
}

impl<'a> Sounds<'a> {
    pub fn load(device: Option<&'a RaylibAudio>, root: &Path, muted: bool) -> Self {
        let dir = root.join("assets/audio");
        let music = |name: &str| {
            device.and_then(|d| d.new_music(&dir.join("music/Halloween").join(name).to_string_lossy()).ok()).map(|mut m| {
                m.looping = true;
                m
            })
        };
        let sound = |name: &str, volume: f32| {
            device.and_then(|d| d.new_sound(&dir.join("sfx/Halloween").join(name).to_string_lossy()).ok()).inspect(|s| {
                s.set_volume(if muted { 0.0 } else { volume });
            })
        };
        let voices = |k: usize| VOICES[k].iter().filter_map(|&(name, gain, audible)| {
            sound(name, 0.0).map(|sound| Voice { sound, gain, audible, owner: None, age: 0.0, level: 0.0, pitch: 1.0 })
        }).collect();
        Self {
            intro: music("exploring.mp3"),
            game: music("conspiracy_theory.mp3"),
            playing: None,
            roars: ROARS.map(|(name, volume)| sound(name, volume)),
            crowd: sound("crowd_yelling.mp3", CROWD_VOLUME),
            crowd_age: None,
            scream: sound("scare_scream.mp3", SCREAM_VOLUME),
            annoyed: sound("fright_annoyed.mp3", ANNOYED_VOLUME),
            sting: None,
            next_roar: [1, 0, 2],
            voices: [voices(0), voices(1), voices(2)],
            next_voice: [0, 1, 1],
            last_pitch: [1.0; 3],
            rng: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos()) | 1,
            muted,
        }
    }

    /// Las voces de las criaturas que se acercan: cada una tiene la suya, que sigue su distancia (qué tan fuerte) y su
    /// lado (según hacia dónde mira la bruja, `yaw` en grados). La de una que ya no viene se apaga de a poco.
    pub fn approach(&mut self, s: &Session, yaw: f32, dt: f32, paused: bool) {
        let far = s.balance.spawn_radius;
        for (k, kind) in [Kind::Devil, Kind::Ghost, Kind::Demon].into_iter().enumerate() {
            let mut coming: Vec<_> = s.creatures.iter().filter(|c| c.kind == kind && c.state == State::Approaching && !paused).collect();
            coming.sort_by(|a, b| a.distance().total_cmp(&b.distance()));
            let voices = &mut self.voices[k];
            // Las voces cuya criatura ya no viene quedan libres; las que llegaron a su cola muda se cortan
            for v in voices.iter_mut() {
                v.age += dt * v.pitch;
                if v.owner.is_some_and(|id| !coming.iter().any(|c| c.id == id)) {
                    v.owner = None;
                } else if v.owner.is_some() && v.age >= v.audible {
                    v.owner = None;
                    v.level = 0.0;
                    v.sound.stop();
                }
            }
            // Las que no tienen voz (primero las más cercanas) toman la siguiente libre
            for c in &coming {
                if voices.iter().any(|v| v.owner == Some(c.id)) || voices.is_empty() {
                    continue;
                }
                let n = voices.len();
                let start = self.next_voice[k];
                let free = (0..n).map(|i| (start + i) % n).find(|&i| voices[i].owner.is_none() && !voices[i].sound.is_playing());
                if let Some(i) = free {
                    // Un tono al azar, lejos del anterior de este tipo
                    let mut pitch = 1.0;
                    for _ in 0..8 {
                        self.rng ^= self.rng << 13;
                        self.rng ^= self.rng >> 17;
                        self.rng ^= self.rng << 5;
                        pitch = VOICE_PITCH.0 + (VOICE_PITCH.1 - VOICE_PITCH.0) * (self.rng >> 8) as f32 / (1u32 << 24) as f32;
                        if (pitch - self.last_pitch[k]).abs() >= PITCH_APART {
                            break;
                        }
                    }
                    self.last_pitch[k] = pitch;
                    let v = &mut voices[i];
                    v.pitch = pitch;
                    v.sound.set_pitch(pitch);
                    v.owner = Some(c.id);
                    v.age = 0.0;
                    v.level = closeness(c.distance(), far, s.balance.hit_radius);
                    v.sound.play();
                    self.next_voice[k] = i + 1;
                }
            }
            for v in voices.iter_mut() {
                let owner = v.owner.and_then(|id| coming.iter().find(|c| c.id == id));
                let target = owner.map_or(0.0, |c| closeness(c.distance(), far, s.balance.hit_radius));
                v.level += (target - v.level) * (dt * VOICE_FADE).min(1.0);
                v.sound.set_volume(if self.muted { 0.0 } else { v.level * v.gain });
                if let Some(c) = owner {
                    // De qué lado viene, respecto de hacia dónde mira (1 en el raylib es todo a la izquierda)
                    let side = (c.pos.0.atan2(c.pos.1) + yaw.to_radians()).sin();
                    v.sound.set_pan((0.5 - 0.4 * side).clamp(0.05, 0.95));
                } else if v.level < 0.01 {
                    v.sound.stop();
                }
            }
        }
    }

    /// Cada cuadro: qué música va y si está en pausa (al cambiar de música, la nueva empieza desde el principio), y los
    /// gritos que se van apagando.
    pub fn update(&mut self, track: Track, paused: bool, dt: f32) {
        if self.playing != Some(track) {
            for (t, m) in [(Track::Intro, &self.intro), (Track::Game, &self.game)] {
                if let Some(m) = m {
                    if t == track {
                        m.seek_stream(0.0);
                        m.play_stream();
                    } else {
                        m.stop_stream();
                    }
                }
            }
            self.playing = Some(track);
        }
        let playing = if track == Track::Intro { INTRO_VOLUME } else { GAME_VOLUME };
        let volume = if self.muted { 0.0 } else if paused { PAUSED_VOLUME } else { playing };
        for m in [&self.intro, &self.game].into_iter().flatten() {
            m.set_volume(volume);
            m.update_stream();
        }
        let hush = if self.muted { 0.0 } else { 1.0 };
        let mut sting_over = false;
        if let Some(st) = &mut self.sting {
            st.age += dt;
            let left = fading(st.age, STING_HOLD, STING_FADE);
            let (over, over_volume) = if st.real { (&self.scream, SCREAM_VOLUME) } else { (&self.annoyed, ANNOYED_VOLUME) };
            if let Some(s) = &self.roars[st.roar] {
                s.set_volume(ROARS[st.roar].1 * left * hush);
            }
            if let Some(s) = over {
                s.set_volume(over_volume * left * hush);
            }
            sting_over = left <= 0.0;
        }
        if sting_over {
            self.stop_sting();
        }
        if let Some(age) = &mut self.crowd_age {
            *age += dt;
            let left = fading(*age, CROWD_HOLD, CROWD_FADE);
            if let Some(s) = &self.crowd {
                s.set_volume(CROWD_VOLUME * left * hush);
                if left <= 0.0 {
                    s.stop();
                }
            }
            if left <= 0.0 {
                self.crowd_age = None;
            }
        }
    }

    /// Corta del todo el grito que está sonando.
    fn stop_sting(&mut self) {
        if let Some(st) = self.sting.take() {
            let over = if st.real { &self.scream } else { &self.annoyed };
            for s in [&self.roars[st.roar], over].into_iter().flatten() {
                s.stop();
            }
        }
    }

    /// El grito de una criatura que se tira a la cara: si es de verdad, con un alarido encima; si es un susto de
    /// mentira, con una voz enojada. Dura lo que el susto y se apaga.
    pub fn roar(&mut self, kind: Kind, real: bool) {
        self.stop_sting();
        let roar = match kind {
            Kind::Devil => 0,
            Kind::Ghost => 1,
            Kind::Demon => 2,
        };
        let hush = if self.muted { 0.0 } else { 1.0 };
        let (over, over_volume) = if real { (&self.scream, SCREAM_VOLUME) } else { (&self.annoyed, ANNOYED_VOLUME) };
        if let Some(s) = over {
            s.set_volume(over_volume * hush);
            s.play();
        }
        if let Some(s) = &self.roars[roar] {
            s.set_volume(ROARS[roar].1 * hush);
            s.set_pitch(ROAR_PITCHES[self.next_roar[roar]]);
            s.play();
            self.next_roar[roar] = (self.next_roar[roar] + 1) % ROAR_PITCHES.len();
        }
        self.sting = Some(Sting { roar, real, age: 0.0 });
    }

    /// La gente que grita cuando empieza una oleada.
    pub fn crowd(&mut self) {
        if let Some(s) = &self.crowd {
            s.stop();
            s.set_volume(if self.muted { 0.0 } else { CROWD_VOLUME });
            s.play();
            self.crowd_age = Some(0.0);
        }
    }
}
