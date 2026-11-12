use planet_hoppers::minigames::smilingfriends::{quality, Dialogue, Game, Quality, Step};
use std::path::PathBuf;

fn dialogue() -> Dialogue {
    Dialogue::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/smilingfriends-dialogue.json")).unwrap()
}

/// Índice de la mejor (o peor) respuesta de la pregunta actual.
fn pick(game: &Game, best: bool) -> usize {
    let Step::Prompt { encounter, round } = game.step else { panic!("no hay pregunta: {:?}", game.step) };
    let c = &game.dialogue.encounters[encounter].rounds[round].choices;
    let idx = (0..c.len()).max_by_key(|&i| if best { c[i].laugh as i64 } else { -(c[i].laugh as i64) });
    idx.unwrap()
}

fn skip_to_prompt(game: &mut Game) {
    while !matches!(game.step, Step::Prompt { .. }) && !game.finished() {
        game.advance();
    }
}

#[test]
fn el_dialogo_es_valido() {
    let d = dialogue();
    assert_eq!(d.encounters.len(), 3);
    assert!(d.boss.lines.len() >= 2);
}

#[test]
fn con_las_mejores_respuestas_se_gana_sin_strikes() {
    let mut game = Game::new(dialogue());
    let mut laughed = 0;
    while !game.finished() {
        match game.step {
            Step::Prompt { .. } => game.choose(pick(&game, true)),
            Step::Laughing { .. } => {
                laughed += 1;
                game.advance();
            }
            _ => game.advance(),
        }
    }
    assert_eq!(game.step, Step::Victory);
    assert_eq!(laughed, 3);
    assert_eq!(game.strikes, 0);
}

#[test]
fn tres_respuestas_malas_pierden() {
    let mut game = Game::new(dialogue());
    skip_to_prompt(&mut game);
    for _ in 0..3 {
        let c = pick(&game, false);
        assert_eq!(quality(&game.dialogue.encounters[0].rounds[0].choices[c]), Quality::Bad);
        game.choose(c);
        game.advance();
        skip_to_prompt(&mut game);
    }
    assert_eq!(game.step, Step::Defeat);
}

#[test]
fn sin_risa_al_final_es_strike_y_se_repite_la_persona() {
    let mut game = Game::new(dialogue());
    skip_to_prompt(&mut game);
    // Solo respuestas "más o menos": no llena el medidor ni suma strikes hasta el final
    loop {
        let Step::Prompt { encounter, round } = game.step else { break };
        let c = &game.dialogue.encounters[encounter].rounds[round].choices;
        let meh = (0..3).find(|&i| quality(&c[i]) == Quality::Meh).unwrap();
        game.choose(meh);
        game.advance();
    }
    assert_eq!(game.step, Step::Failed { encounter: 0 });
    assert_eq!(game.strikes, 1);
    game.advance();
    assert_eq!(game.step, Step::Intro { encounter: 0 });
    assert_eq!(game.meter(), 0.0);
}

#[test]
fn elegir_fuera_de_una_pregunta_no_hace_nada() {
    let mut game = Game::new(dialogue());
    game.choose(0);
    assert_eq!(game.step, Step::Boss { line: 0 });
    assert_eq!(game.strikes, 0);
}

#[test]
fn con_desmond_solo_la_montana_rusa_sirve() {
    // Llegar a la primera pregunta de Desmond (la tercera escena) con las mejores respuestas
    let d = dialogue();
    let desmond = d.encounters.iter().position(|e| e.cast[0].name == "DESMOND").unwrap();
    for wrong in [1, 2] {
        let mut game = Game::new(dialogue());
        while game.step != (Step::Prompt { encounter: desmond, round: 0 }) {
            if matches!(game.step, Step::Prompt { .. }) {
                let best = pick(&game, true);
                game.choose(best);
            } else {
                game.advance();
            }
        }
        let strikes = game.strikes;
        game.choose(wrong);
        game.advance();
        // Cualquier otra respuesta pierde la escena en el acto (con su explosión), con un solo strike
        assert_eq!(game.step, Step::Failed { encounter: desmond });
        assert_eq!(game.strikes, strikes + 1);
        assert_eq!(game.failed_at, Some(0));
    }
}
