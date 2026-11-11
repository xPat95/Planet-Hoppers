//! Encounter data and placement, independent of the current challenge mechanics.
use super::{locations::EventSite, navigation::event_route, threats::ThreatKind, SouthParkScene};
use raylib::prelude::*;

pub const NAMES: [&str; 8] = [
    "Tolkien", "Tweek", "Craig", "Clyde", "Jimmy", "Timmy", "Wendy", "Bebe",
];
pub const MEMBERS: [&[usize]; 5] = [&[0], &[1, 2], &[3], &[4, 5], &[6, 7]];
pub const KINDS: [ThreatKind; 5] = [
    ThreatKind::Ufo,
    ThreatKind::Snowman,
    ThreatKind::Bus,
    ThreatKind::Piano,
    ThreatKind::Sewer,
];
pub const COLORS: [Color; 5] = [
    Color::PURPLE,
    Color::SKYBLUE,
    Color::ORANGE,
    Color::GREEN,
    Color::PINK,
];
#[derive(Clone)]
pub struct Group {
    pub positions: Vec<Vector3>,
    pub site: EventSite,
}
#[derive(Clone)]
pub struct Adventure {
    pub groups: Vec<Group>,
    pub completed: [bool; 5],
    pub active: Option<usize>,
    pub dialogue: Option<(usize, usize)>,
    pub spanish: bool,
}
impl Adventure {
    pub fn build(scene: &SouthParkScene, sites: &[EventSite]) -> Result<Self, String> {
        let mut groups: Vec<Group> = vec![];
        for (i, kind) in KINDS.iter().enumerate() {
            let mut chosen = None;
            for site in sites.iter().filter(|s| s.compatible.contains(kind)) {
                let base = Vector3::new(site.entry[0], site.entry[1], site.entry[2]);
                if groups
                    .iter()
                    .any(|g| (g.positions[0] - base).length() < 35.0)
                {
                    continue;
                }
                for side in [-4.0, 4.0, -6.0, 6.0] {
                    let positions: Vec<_> = MEMBERS[i]
                        .iter()
                        .enumerate()
                        .map(|(j, _)| base + Vector3::new(j as f32 * 1.5, 0.0, side))
                        .collect();
                    if positions.iter().all(|p| {
                        scene.world.can_occupy(*p, 0.65, 1.8)
                            && event_route(
                                &scene.roads,
                                &scene.world,
                                scene.player.feet,
                                *p,
                                scene.player.config.radius,
                                scene.player.config.body_height,
                            )
                            .is_ok()
                    }) {
                        chosen = Some(Group {
                            positions,
                            site: site.clone(),
                        });
                        break;
                    }
                }
                if chosen.is_some() {
                    break;
                }
            }
            groups.push(chosen.ok_or_else(|| format!("No accessible encounter for {kind:?}"))?);
        }
        Ok(Self {
            groups,
            completed: [false; 5],
            active: None,
            dialogue: None,
            spanish: false,
        })
    }
    pub fn text<'a>(&self, en: &'a str, es: &'a str) -> &'a str {
        if self.spanish {
            es
        } else {
            en
        }
    }
    pub fn nearby(&self, scene: &SouthParkScene) -> Option<usize> {
        self.groups
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.completed[*i] && self.active.is_none())
            .find_map(|(i, g)| {
                g.positions
                    .iter()
                    .any(|p| {
                        let delta = *p + Vector3::up() - scene.player.eye();
                        delta.length() < scene.interaction_distance + 1.0
                            && scene
                                .world
                                .ray_distance_ignoring(
                                    scene.player.eye(),
                                    delta.normalized(),
                                    delta.length(),
                                    None,
                                )
                                .is_none()
                    })
                    .then_some(i)
            })
    }
}
pub struct Line {
    pub speaker: usize,
    pub en: &'static str,
    pub es: &'static str,
    pub voice: bool,
}
macro_rules! line {
    ($s:expr,$en:expr,$es:expr) => {
        Line {
            speaker: $s,
            en: $en,
            es: $es,
            voice: false,
        }
    };
    ($s:expr,$en:expr,$es:expr,voice) => {
        Line {
            speaker: $s,
            en: $en,
            es: $es,
            voice: true,
        }
    };
}
pub fn script(group: usize) -> &'static [Line] {
    match group {
        0 => &[
            line!(
                0,
                "Oh, hi there! You came just in time.",
                "¡Hola! Llegaste justo a tiempo."
            ),
            line!(
                0,
                "You see, aliens are coming.",
                "Mira, vienen los extraterrestres."
            ),
            line!(
                0,
                "Yeah, yeah. Aliens.",
                "Sí, sí. Extraterrestres. Otro día normal aquí."
            ),
            line!(
                0,
                "I think they want to abduct us all.",
                "Creo que quieren secuestrarnos a todos."
            ),
            line!(
                0,
                "Look, they're getting Kenny.",
                "¡Mira, se están llevando a Kenny!"
            ),
            line!(
                0,
                "Oh Lord, they're going to kill Kenny!",
                "¡Dios mío, van a matar a Kenny!"
            ),
            line!(
                0,
                "Hurry up! Follow them and turn off the lights to save him!",
                "¡Rápido! ¡Síguelos y apaga las luces para salvarlo!"
            ),
        ],
        1 => &[
            line!(
                1,
                "OH MY GOD! IT'S TOO MUCH PRESSURE!",
                "¡DIOS MÍO! ¡ES DEMASIADA PRESIÓN!",
                voice
            ),
            line!(2, "Calm down, honey!", "¡Tranquilo, cariño!"),
            line!(2, "Hey, you! Listen!", "¡Oye, tú! ¡Escucha!"),
            line!(2, "You know Kenny, right?", "Conoces a Kenny, ¿verdad?"),
            line!(2, "He's FREEZING!", "¡Se está CONGELANDO!"),
            line!(
                2,
                "Hurry! You need to save him!",
                "¡Date prisa! ¡Tienes que salvarlo!"
            ),
            line!(
                2,
                "Connect those cables to turn on the heaters!",
                "¡Conecta esos cables para encender los calefactores!"
            ),
            line!(
                2,
                "You have 30 seconds before Kenny dies!",
                "¡Tienes 30 segundos antes de que Kenny muera!"
            ),
            line!(1, "TOO MUCH PRESSURE!", "¡DEMASIADA PRESIÓN!"),
        ],
        2 => &[
            line!(3, "Hah.", "Ja."),
            line!(3, "Oh, didn't see you there.", "Ah, no te había visto."),
            line!(
                3,
                "I think that bus lost control or something.",
                "Creo que ese autobús perdió el control o algo así."
            ),
            line!(
                3,
                "You might wanna turn off the switches before something happens.",
                "Igual deberías apagar los interruptores antes de que pase algo."
            ),
            line!(
                3,
                "Me? Nah, I don't really care.",
                "¿Yo? Nah, me da bastante igual."
            ),
            line!(
                3,
                "I think Kenny was walking somewhere near that bus.",
                "Creo que Kenny andaba por ahí, cerca del autobús."
            ),
            line!(3, "Anyway, bye-bye.", "En fin. Adiós, adiós."),
        ],
        3 => &[
            line!(
                4,
                "Wow, what a great audience!",
                "¡Vaya, qué público tan estupendo!",
                voice
            ),
            line!(5, "TIMMEH!", "¡TIMMEH!", voice),
            line!(
                4,
                "H-h-hi, fella! I a-a-am Jimmy.",
                "¡H-h-hola, amigo! S-s-soy Jimmy."
            ),
            line!(4, "And this is my friend...", "Y este es mi amigo..."),
            line!(5, "TIMMEH!", "¡TIMMEH!", voice),
            line!(4, "T-t-Timmy, right here.", "T-t-Timmy, aquí presente."),
            line!(4, "You see, we saw a...", "Verás, vimos un..."),
            line!(4, "We saw a...", "Vimos un..."),
            line!(4, "We saw a. piiiii....", "Vimos un. piiiii...."),
            line!(
                4,
                "We saw a p-p-piano hanging over there.",
                "Vimos un p-p-piano colgando por allá."
            ),
            line!(4, "We...", "Noso..."),
            line!(4, "We...", "Noso..."),
            line!(
                4,
                "We thought you could h-h-help Kenny!",
                "¡Pensamos que podrías a-a-ayudar a Kenny!"
            ),
            line!(5, "TIMMEH!", "¡TIMMEH!", voice),
            line!(
                4,
                "Remember the sequence and stop the piano from falling!",
                "¡Recuerda la secuencia y evita que caiga el piano!"
            ),
        ],
        _ => &[
            line!(
                6,
                "Hey! Hey, you! You gotta help me!",
                "¡Oye! ¡Tú! ¡Tienes que ayudarme!"
            ),
            line!(
                6,
                "That kid, Kenny, Stan's friend... he's gonna fall!",
                "¡Ese niño, Kenny, el amigo de Stan... se va a caer!"
            ),
            line!(
                7,
                "Yeah, we saw him walking through the city, with...",
                "Sí, lo vimos paseando por la ciudad, con..."
            ),
            line!(
                6,
                "ALL THE MANHOLES OPEN! HE'S GONNA FALL!",
                "¡TODAS LAS ALCANTARILLAS ABIERTAS! ¡SE VA A CAER!"
            ),
            line!(
                7,
                "Calm down, Wendy. You only care because he's Stan's friend.",
                "Tranquila, Wendy. Solo te importa porque es amigo de Stan."
            ),
            line!(
                6,
                "Anyway, you gotta help him! Hurry up!",
                "¡Como sea, tienes que ayudarlo! ¡Rápido!"
            ),
        ],
    }
}
