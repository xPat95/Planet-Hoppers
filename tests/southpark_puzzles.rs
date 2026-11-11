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
fn bus_clues_have_exactly_one_solution_and_attempts_are_bounded() {
    let mut solutions = vec![];
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let order = [a, b, c, d];
                    let mut unique = order;
                    unique.sort();
                    if unique != [0, 1, 2, 3] {
                        continue;
                    }
                    let index = |n| order.iter().position(|x| *x == n).unwrap();
                    if index(0) < index(3)
                        && index(0) == index(2) + 1
                        && index(1) != 0
                        && index(1) != 2
                    {
                        solutions.push(order);
                    }
                }
            }
        }
    }
    assert_eq!(solutions, vec![BUS_SOLUTION]);
    let mut p = playing(ThreatKind::Bus);
    for i in BUS_SOLUTION {
        p.click(i);
    }
    assert_eq!(p.result, Some(true));
    let mut p = playing(ThreatKind::Bus);
    p.click(3);
    p.click(3);
    let Board::Bus(b) = &p.board else { panic!() };
    assert_eq!(b.entered, vec![3]);
    for i in [2, 1, 0] {
        p.click(i);
    }
    assert!(p.result.is_none());
    for _ in 0..2 {
        for i in [3, 2, 1, 0] {
            p.click(i);
        }
    }
    assert_eq!(p.result, Some(false));
}
#[test]
fn piano_blocks_demo_input_and_plays_five_increasing_rounds() {
    let mut p = playing(ThreatKind::Piano);
    p.click(4);
    let Board::Piano(k) = &p.board else { panic!() };
    assert_eq!((k.input, k.errors), (0, 0));
    for round in 0..5 {
        p.tick(6.0);
        let Board::Piano(k) = &p.board else { panic!() };
        assert!(!k.demonstrating);
        assert_eq!(k.length(), round + 2);
        let notes = k.notes[..k.length()].to_vec();
        for n in notes {
            p.click(n);
        }
    }
    assert_eq!(p.result, Some(true));
}
#[test]
fn piano_mistakes_repeat_round_and_third_drops_the_piano() {
    let mut p = playing(ThreatKind::Piano);
    p.tick(6.0);
    p.click(1);
    p.click(3);
    for error in 1..=3 {
        p.tick(6.0);
        p.click(4);
        let Board::Piano(k) = &p.board else { panic!() };
        assert_eq!(k.round, 1);
        assert_eq!(k.errors, error);
        assert!(k.demonstrating);
    }
    assert_eq!(p.result, Some(false));
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
    for i in BUS_SOLUTION {
        p.click(i);
    }
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
