//! Minijuego de Smiling Friends: hacer reír a tres personas eligiendo respuestas.
//!
//! Solo lógica, sin ventana: datos del diálogo (`config/smilingfriends-dialogue.json`), medidor de risa, strikes,
//! reinicio de un encuentro y resultado. La pantalla (`scenes::smilingfriends_game`) pregunta qué mostrar con
//! `Game::step` y avisa las decisiones con `choose` y `advance`.
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct BossLine {
    /// Imagen del jefe para esta línea (de `assets/textures/minigames/smilingfriends/characters/`).
    pub pose: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Boss {
    pub name: String,
    pub background: String,
    pub lines: Vec<BossLine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub text: String,
    /// Cuánto llena el medidor (0 = respuesta mala: suma un strike).
    pub laugh: u32,
    /// Lo que contesta el personaje.
    pub reply: String,
    /// Reacción corta que salta en pantalla ("¡JA!", "*te mira fijo*").
    pub reaction: String,
    /// Quién contesta, si no es el que preguntó (índice en `cast`).
    #[serde(default)]
    pub speaker: Option<usize>,
    /// Texto de contexto de la respuesta (lo que hace o quiere decir; ver `Round::context`).
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Round {
    /// Quién pregunta (índice en `cast`).
    #[serde(default)]
    pub speaker: usize,
    /// Texto de contexto, en otro estilo, antes de lo que dice: describe lo que hace o lo que quiere decir
    /// (para los que casi no hablan, como Mr. Frog).
    #[serde(default)]
    pub context: Option<String>,
    /// Cambio de lugar desde esta pregunta: fondo y dibujo nuevos del primer personaje, y un cartel de transición
    /// ("MÁS TARDE, EN..."). Valen también para las preguntas siguientes y para el final de la escena.
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub scene_card: Option<String>,
    /// Nombre del lugar nuevo (el que se ve arriba a la izquierda).
    #[serde(default)]
    pub place: Option<String>,
    pub prompt: String,
    pub choices: Vec<Choice>,
}

/// Personaje en escena. Al reírse puede cambiar de dibujo y de nombre (el remate del chiste).
#[derive(Debug, Clone, Deserialize)]
pub struct Actor {
    pub name: String,
    pub image: String,
    #[serde(default)]
    pub laugh_image: Option<String>,
    #[serde(default)]
    pub laugh_name: Option<String>,
    /// Posición horizontal (0..1 del ancho de la pantalla) y alto (0..1 del alto), si no se usa la de siempre.
    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub size: Option<f32>,
    /// Altura de los pies (0..1 del alto de la pantalla, 1 = abajo de todo), para que uno chico no quede tapado.
    #[serde(default)]
    pub floor: Option<f32>,
    /// Alto y pies del dibujo del remate, si son otros.
    #[serde(default)]
    pub laugh_size: Option<f32>,
    #[serde(default)]
    pub laugh_floor: Option<f32>,
    #[serde(default)]
    pub laugh_x: Option<f32>,
}

/// Línea dicha por un personaje de la escena.
#[derive(Debug, Clone, Deserialize)]
pub struct Line {
    #[serde(default)]
    pub speaker: usize,
    #[serde(default)]
    pub context: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Encounter {
    /// Nombre de la escena ("EL BAR").
    pub name: String,
    pub background: String,
    pub cast: Vec<Actor>,
    /// Efecto especial de la escena: "disco" (luces de boliche) o "bomb" (bomba al fallar).
    #[serde(default)]
    pub effect: Option<String>,
    /// Imagen que salta a pantalla completa cuando se ríe (un primer plano feliz).
    #[serde(default)]
    pub laugh_card: Option<String>,
    /// Final en otro lugar y otro momento: al reírse pasa un cartel (`laugh_scene_card`, "TIEMPO DESPUÉS...") y
    /// el personaje aparece cambiado (`laugh_image`) en `laugh_background`.
    #[serde(default)]
    pub laugh_background: Option<String>,
    #[serde(default)]
    pub laugh_scene_card: Option<String>,
    /// Imagen detrás de los carteles de cambio de lugar de esta escena.
    #[serde(default)]
    pub card_backdrop: Option<String>,
    pub intro: Line,
    pub rounds: Vec<Round>,
    /// Lo que dice al morirse de risa.
    pub laugh_line: Line,
    /// Lo que dice si se acaban las preguntas sin que se ría (y el encuentro vuelve a empezar).
    pub fail_line: Line,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Dialogue {
    pub boss: Boss,
    pub encounters: Vec<Encounter>,
    /// Risa necesaria para que una persona se ría a carcajadas.
    pub laugh_goal: u32,
    /// Strikes para perder.
    pub max_strikes: u32,
    pub victory_line: String,
    pub defeat_line: String,
}

impl Dialogue {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let d: Self = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        d.validate()?;
        Ok(d)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.boss.lines.is_empty() {
            return Err("El jefe necesita al menos una línea".into());
        }
        if self.encounters.len() != 3 {
            return Err("Tienen que ser tres personas".into());
        }
        if self.laugh_goal == 0 || self.max_strikes == 0 {
            return Err("laugh_goal y max_strikes tienen que ser mayores que cero".into());
        }
        for e in &self.encounters {
            if e.cast.is_empty() {
                return Err(format!("{}: sin personajes", e.name));
            }
            let speakers = [e.intro.speaker, e.laugh_line.speaker, e.fail_line.speaker];
            let repliers = e.rounds.iter().flat_map(|r| r.choices.iter().filter_map(|c| c.speaker));
            if speakers.into_iter().chain(e.rounds.iter().map(|r| r.speaker)).chain(repliers).any(|k| k >= e.cast.len()) {
                return Err(format!("{}: habla un personaje que no está en escena", e.name));
            }
            if e.rounds.is_empty() {
                return Err(format!("{}: sin preguntas", e.name));
            }
            for (i, r) in e.rounds.iter().enumerate() {
                if r.choices.len() != 3 {
                    return Err(format!("{}, pregunta {}: tiene que tener tres respuestas", e.name, i + 1));
                }
            }
            // Tiene que poder ganarse: la mejor respuesta de cada pregunta alcanza para la risa completa
            let best: u32 = e.rounds.iter().map(|r| r.choices.iter().map(|c| c.laugh).max().unwrap_or(0)).sum();
            if best < self.laugh_goal {
                return Err(format!("{}: ni con las mejores respuestas llega a la risa ({best} de {})", e.name, self.laugh_goal));
            }
        }
        Ok(())
    }
}

/// Qué hay que mostrar ahora.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// El jefe dice su línea `line`.
    Boss { line: usize },
    /// Entra la persona `encounter` y dice su presentación.
    Intro { encounter: usize },
    /// Pregunta `round` de la persona `encounter`: esperando que el jugador elija.
    Prompt { encounter: usize, round: usize },
    /// La persona reacciona a la respuesta `choice`.
    Reaction { encounter: usize, round: usize, choice: usize },
    /// La persona se muere de risa.
    Laughing { encounter: usize },
    /// Se acabaron las preguntas sin risa: strike y el encuentro vuelve a empezar.
    Failed { encounter: usize },
    Victory,
    Defeat,
}

/// Qué tan buena fue una respuesta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Good,
    Meh,
    Bad,
}

/// Fondo y dibujo del primer personaje vigentes en la pregunta `round` (o al final, con `None`): los del último
/// cambio de lugar hasta ahí, o los de la escena.
pub fn scene_art(e: &Encounter, round: Option<usize>) -> (String, String) {
    let upto = round.map_or(e.rounds.len(), |r| r + 1);
    let mut bg = e.background.clone();
    let mut img = e.cast[0].image.clone();
    for r in &e.rounds[..upto] {
        if let Some(b) = &r.background {
            bg = b.clone();
        }
        if let Some(i) = &r.image {
            img = i.clone();
        }
    }
    (bg, img)
}

/// Nombre del lugar vigente en la pregunta `round` (o al final, con `None`).
pub fn place_name(e: &Encounter, round: Option<usize>) -> String {
    let upto = round.map_or(e.rounds.len(), |r| r + 1);
    e.rounds[..upto].iter().rev().find_map(|r| r.place.clone()).unwrap_or_else(|| e.name.clone())
}

pub fn quality(choice: &Choice) -> Quality {
    match choice.laugh {
        0 => Quality::Bad,
        1..=29 => Quality::Meh,
        _ => Quality::Good,
    }
}

pub struct Game {
    pub dialogue: Dialogue,
    pub step: Step,
    /// Risa acumulada de la persona actual.
    pub laugh: u32,
    pub strikes: u32,
}

impl Game {
    pub fn new(dialogue: Dialogue) -> Self {
        Self { dialogue, step: Step::Boss { line: 0 }, laugh: 0, strikes: 0 }
    }

    /// Fracción del medidor de risa (0..1).
    pub fn meter(&self) -> f32 {
        (self.laugh as f32 / self.dialogue.laugh_goal as f32).min(1.0)
    }

    /// El jugador elige una respuesta (solo durante una pregunta).
    pub fn choose(&mut self, choice: usize) {
        let Step::Prompt { encounter, round } = self.step else { return };
        let Some(c) = self.dialogue.encounters[encounter].rounds[round].choices.get(choice) else { return };
        self.laugh += c.laugh;
        if c.laugh == 0 {
            self.strikes += 1;
        }
        self.step = Step::Reaction { encounter, round, choice };
    }

    /// Sigue adelante (clic o Enter fuera de una pregunta).
    pub fn advance(&mut self) {
        let n_boss = self.dialogue.boss.lines.len();
        let n_enc = self.dialogue.encounters.len();
        self.step = match self.step {
            Step::Boss { line } if line + 1 < n_boss => Step::Boss { line: line + 1 },
            Step::Boss { .. } => Step::Intro { encounter: 0 },
            Step::Intro { encounter } => Step::Prompt { encounter, round: 0 },
            Step::Prompt { .. } => self.step, // Hay que elegir
            Step::Reaction { encounter, round, .. } => {
                if self.strikes >= self.dialogue.max_strikes {
                    Step::Defeat
                } else if self.laugh >= self.dialogue.laugh_goal {
                    Step::Laughing { encounter }
                } else if round + 1 < self.dialogue.encounters[encounter].rounds.len() {
                    Step::Prompt { encounter, round: round + 1 }
                } else {
                    self.strikes += 1;
                    Step::Failed { encounter }
                }
            }
            Step::Laughing { encounter } => {
                self.laugh = 0;
                if encounter + 1 < n_enc {
                    Step::Intro { encounter: encounter + 1 }
                } else {
                    Step::Victory
                }
            }
            Step::Failed { encounter } => {
                self.laugh = 0;
                if self.strikes >= self.dialogue.max_strikes {
                    Step::Defeat
                } else {
                    Step::Intro { encounter }
                }
            }
            Step::Victory | Step::Defeat => self.step,
        };
    }

    pub fn finished(&self) -> bool {
        matches!(self.step, Step::Victory | Step::Defeat)
    }
}
