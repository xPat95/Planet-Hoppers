use planet_hoppers::minigames::southpark::{puzzles::*, threats::ThreatKind};

fn playing(kind: ThreatKind) -> PuzzleRun {
    let mut p = PuzzleRun::new(kind, 0);
    p.tick(FADE_SECONDS);
    p.start();
    assert!(p.interactive());
    p
}
fn solve_heaters(p: &mut PuzzleRun) {
    for (i, target) in HEATER_SOLUTION.into_iter().enumerate() {
        for _ in 0..4 {
            let Board::Heaters(h) = &p.board else {
                panic!()
            };
            if h.tiles[i] == target || p.result.is_some() {
                break;
            }
            p.click(i);
        }
    }
}
#[test]
fn circuit_reaches_all_three_destinations_by_compatible_ports() {
    let mut p = playing(ThreatKind::Snowman);
    assert_eq!(p.result, None);
    solve_heaters(&mut p);
    assert_eq!(p.result, Some(true));
    let Board::Heaters(h) = &mut p.board else {
        panic!()
    };
    // An unrelated tile need not match the reference solution.
    h.tiles[4] = rotate(h.tiles[4]);
    h.recompute();
    assert!(h.destinations.iter().all(|v| *v));
    // A neighbour touching a powered tile without a reciprocal port is not connected.
    h.tiles[1] = 1;
    h.recompute();
    assert_eq!(h.destinations, [false; 3]);
    h.tiles = HEATER_SOLUTION;
    h.tiles[0] &= !8;
    h.recompute();
    assert_eq!(h.powered, [false; 16]);
}
#[test]
fn clocks_start_only_after_transition_and_explicit_start_and_pause_freezes() {
    let mut p = PuzzleRun::new(ThreatKind::Snowman, 1);
    p.start();
    assert!(!p.started);
    p.tick(200.0);
    assert_eq!(p.elapsed, 0.0);
    p.tick(200.0);
    assert_eq!(p.result, None);
    p.start();
    p.tick(10.0);
    assert!((p.elapsed - 10.0).abs() < 0.01);
    p.paused = true;
    p.tick(200.0);
    p.click(0);
    assert!((p.elapsed - 10.0).abs() < 0.01);
    p.paused = false;
    p.tick(20.1);
    assert_eq!(p.result, Some(false));
}
#[test]
fn ufo_reflexes_win_and_idle_or_wrong_clicks_lose() {
    for dt in [0.05, 0.10, 0.20] {
        let mut p = playing(ThreatKind::Ufo);
        while p.result.is_none() {
            if let Board::Ufo(u) = &p.board {
                let target = u.target;
                p.click(target);
            }
            p.tick(dt);
        }
        assert_eq!(p.result, Some(true));
        assert!((p.elapsed - 20.0).abs() < 0.06);
    }
    let mut idle = playing(ThreatKind::Ufo);
    idle.tick(20.0);
    assert_eq!(idle.result, Some(false));
    let mut wrong = playing(ThreatKind::Ufo);
    for _ in 0..10 {
        if let Board::Ufo(u) = &wrong.board {
            wrong.click((u.target + 1) % 6);
        }
    }
    assert_eq!(wrong.result, Some(false));
}
#[test]
fn murdoku_has_the_original_unique_solution_and_one_confirmation() {
    use planet_hoppers::minigames::southpark::murdoku::{solutions,SOLUTION,DANGEROUS};
    assert_eq!(solutions(),vec![(SOLUTION,DANGEROUS)]);
    let mut p=playing(ThreatKind::Bus);p.confirm_bus();assert!(p.result.is_none());
    for (i,cell) in SOLUTION.into_iter().enumerate() {p.select_bus_piece(i);p.place_bus_piece(cell);}
    p.accuse_bus_driver(DANGEROUS);p.confirm_bus();assert_eq!(p.result,Some(true));
    p.accuse_bus_driver(1);p.clear_bus();p.confirm_bus();assert_eq!(p.result,Some(true));
}
#[test]
fn piano_has_eight_distinct_rounds_at_constant_speed() {
    let mut p=playing(ThreatKind::Piano);let mut patterns=vec![];
    p.click(4);let Board::Piano(k)=&p.board else {panic!()};assert_eq!((k.input,k.errors),(0,0));
    for length in [3,4,5,5,6,6,7,7] {
        p.tick(6.);let Board::Piano(k)=&p.board else {panic!()};
        assert_eq!(k.length(),length);assert_eq!(k.illumination(),0.45);
        let notes=k.notes[..length].to_vec();assert!(!patterns.contains(&notes));patterns.push(notes.clone());
        for n in notes {p.click(n);}
    }
    assert_eq!(p.result,Some(true));
}
#[test]
fn piano_repeats_the_same_pattern_then_loses_on_second_error() {
    let mut p=playing(ThreatKind::Piano);
    let Board::Piano(k)=&p.board else {panic!()};let notes=k.notes;
    for error in 1..=2 {
        p.tick(6.);p.click((notes[0]+1)%5);
        let Board::Piano(k)=&p.board else {panic!()};assert_eq!(k.notes,notes);assert_eq!(k.errors,error);
    }
    assert_eq!(p.result,Some(false));
}
#[test]
fn manholes_are_fair_and_reachable_with_fixed_time_steps() {
    let mut p = playing(ThreatKind::Sewer);
    let mut first_open = [None; 7];
    while p.result.is_none() {
        let Board::Sewers(s) = &p.board else { panic!() };
        let open = s.open;
        for i in 0..7 {
            if open[i] {
                first_open[i].get_or_insert(p.elapsed);
                p.click(i);
            }
        }
        p.tick(0.1);
    }
    assert_eq!(p.result, Some(true));
    for i in 0..7 {
        assert!(HOLES[i] * 28.0 - first_open[i].unwrap() > 3.7);
    }
    let mut idle = playing(ThreatKind::Sewer);
    idle.tick(28.0);
    assert_eq!(idle.result, Some(false));
    for i in 0..100 {
        let p = route_point(i as f32 / 99.0);
        assert!((0.0..=1.0).contains(&p.0) && (0.0..=1.0).contains(&p.1));
    }
}
#[test]
fn every_restart_clears_timers_results_inputs_and_pending_feedback() {
    for kind in ThreatKind::ALL {
        let mut p = playing(kind);
        p.tick(2.0);
        p.click(0);
        p.paused = true;
        p.restart();
        assert!(!p.started && !p.paused && !p.leaving && !p.committed);
        assert_eq!((p.elapsed, p.animation, p.feedback), (0.0, 0.0, 0.0));
        assert_eq!(p.last_hit, None);
        assert_eq!(p.result, None);
        assert_eq!(
            format!("{:?}", p.board),
            format!("{:?}", PuzzleRun::new(kind, 0).board)
        );
    }
}
#[test]
fn results_ignore_additional_clicks_and_invalid_indices() {
    let mut p = playing(ThreatKind::Bus);
    p.click(usize::MAX);
    use planet_hoppers::minigames::southpark::murdoku::{SOLUTION,DANGEROUS};
    for (i,cell) in SOLUTION.into_iter().enumerate() {p.select_bus_piece(i);p.place_bus_piece(cell);}
    p.accuse_bus_driver(DANGEROUS);p.confirm_bus();
    let before = format!("{:?}", p.board);
    p.click(1);
    p.tick(200.0);
    assert_eq!(p.result, Some(true));
    assert_eq!(before, format!("{:?}", p.board));
}
#[test]
fn musical_mute_survives_new_puzzles_without_muting_death_or_success() {
    use planet_hoppers::audio::{AudioConfig, AudioController, AudioCue, MusicMode};
    let mut audio = AudioController::load(
        None,
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
        AudioConfig::default(),
    );
    audio.set_background_muted(true);
    for _ in 0..5 {
        audio.restart_challenge_music();
        audio.set_music(MusicMode::Challenge);
    }
    assert!(!audio
        .audit
        .music_transitions
        .iter()
        .any(|s| s == "save_kenny"));
    audio.cue(AudioCue::DeathVoice);
    audio.cue(AudioCue::Success);
    assert_eq!(audio.audit.plays["they_killed_kenny"], 1);
    assert_eq!(audio.audit.plays["success"], 1);
    audio.set_background_muted(false);
    audio.set_music(MusicMode::Challenge);
    let transitions = audio.audit.music_transitions.len();
    audio.set_music(MusicMode::Challenge);
    assert_eq!(audio.audit.music_transitions.len(), transitions);
    assert_eq!(audio.audit.music_transitions.last().unwrap(), "save_kenny");
}
