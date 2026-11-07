use planet_hoppers::minigames::regularshow::{autoplay, simulation::*};

fn game(stage: Stage) -> Game {
    let mut g = Game::new();
    g.start();
    g.load_stage(stage);
    g
}
fn step(g: &mut Game, seconds: f32, input: Input) {
    for _ in 0..(seconds * 120.0).ceil() as u32 {
        g.update(1.0 / 120.0, [input, Input::default()]);
    }
}

#[test]
fn strikes_respect_depth_facing_and_hit_only_once() {
    for (dx, dy, expected) in [(60.0, 0.0, 16), (-60.0, 0.0, 30), (60.0, 45.0, 30)] {
        let mut g = game(Stage::Tutorial);
        g.enemies = vec![Fighter::new(
            Kind::Rookie,
            g.players[0].x + dx,
            g.players[0].y + dy,
        )];
        g.update(
            1.0 / 120.0,
            [
                Input {
                    punch: true,
                    ..Input::default()
                },
                Input::default(),
            ],
        );
        step(&mut g, 0.45, Input::default());
        assert_eq!(g.enemies[0].hp, expected, "dx={dx} dy={dy}");
    }
}

#[test]
fn pause_freezes_ai_attack_timers_particles_and_shockwaves() {
    let mut g = game(Stage::Hammer);
    step(&mut g, 2.0, Input::default());
    g.toggle_pause();
    let before = (
        g.elapsed,
        g.enemies[0].x,
        g.enemies[0].action,
        g.players[0].hp,
    );
    step(
        &mut g,
        10.0,
        Input {
            kick: true,
            ..Input::default()
        },
    );
    assert_eq!(
        before,
        (
            g.elapsed,
            g.enemies[0].x,
            g.enemies[0].action,
            g.players[0].hp
        )
    );
    g.toggle_pause();
    assert_eq!(g.phase, Phase::Playing);
}

#[test]
fn shockwaves_can_be_jumped_or_dodged_and_never_hit_twice() {
    for (input, health) in [
        (Input::default(), 78),
        (
            Input {
                jump: true,
                ..Input::default()
            },
            100,
        ),
        (
            Input {
                dodge: true,
                x: -1.0,
                ..Input::default()
            },
            100,
        ),
    ] {
        let mut g = game(Stage::Hammer);
        g.players[0].cooldown = 0.0;
        let p = &g.players[0];
        g.waves.push(Shockwave {
            x: p.x - 55.0,
            y: p.y,
            radius: 0.0,
            hit: [false; 2],
        });
        g.update(1.0 / 120.0, [input, Input::default()]);
        step(&mut g, 0.7, Input::default());
        assert_eq!(g.players[0].hp, health, "{input:?}");
    }
}

#[test]
fn hammer_announces_his_slam_and_enrages_at_half_health() {
    let mut g = game(Stage::Hammer);
    g.enemies[0].action = Action::Idle;
    g.stage_time = 4.0;
    g.enemies[0].cooldown = 0.0;
    g.update(1.0 / 60.0, [Input::default(); 2]);
    assert!(matches!(
        g.enemies[0].action,
        Action::Strike {
            attack: Attack::Slam,
            fired: false,
            ..
        }
    ));
    assert!(g.waves.is_empty());
    step(&mut g, 0.8, Input::default());
    assert!(g.waves.is_empty());
    step(&mut g, 0.4, Input::default());
    assert_eq!(g.slams, 1);
    assert!(!g.waves.is_empty());
    g.enemies[0].hp = g.enemies[0].max_hp / 2;
    assert!(g.enemies[0].enraged());
}

#[test]
fn retry_restores_stage_health_props_and_stage_start_score() {
    let mut g = game(Stage::Tutorial);
    g.score = 900;
    g.load_stage(Stage::Hammer);
    g.score = 1500;
    g.players[0].hp = 1;
    g.props[0].state = PropState::Held(0);
    g.enemies[0].hp = 1;
    g.phase = Phase::Defeat;
    g.retry();
    assert_eq!(g.stage, Stage::Hammer);
    assert_eq!(g.phase, Phase::Playing);
    assert_eq!(g.score, 900);
    assert_eq!(g.players[0].hp, 100);
    assert_eq!(g.props[0].state, PropState::Ready);
    assert_eq!(g.enemies[0].hp, g.enemies[0].max_hp);
    g.score = 1800;
    g.retry();
    assert_eq!(g.score, 900);
}

#[test]
fn coop_requires_both_players_down_and_scales_enemies() {
    let mut g = Game::new();
    g.coop = true;
    g.start();
    assert_eq!(g.players.len(), 2);
    assert_eq!(g.enemies[0].hp, 45);
    g.players[0].hp = 0;
    g.players[0].action = Action::Down;
    g.update(1.0 / 60.0, [Input::default(); 2]);
    assert_eq!(g.phase, Phase::Playing);
    g.players[1].hp = 0;
    g.players[1].action = Action::Down;
    g.update(1.0 / 60.0, [Input::default(); 2]);
    assert_eq!(g.phase, Phase::Defeat);
}

#[test]
fn boss_resists_bare_hits_and_furniture_can_be_recovered_after_a_miss() {
    let mut g = game(Stage::Hammer);
    g.players[0].x = g.props[0].x;
    g.players[0].y = g.props[0].y;
    g.enemies[0].x = g.players[0].x + 45.0;
    g.enemies[0].y = g.players[0].y;
    g.enemies[0].action = Action::Idle;
    g.enemies[0].cooldown = 20.0;
    let full = g.enemies[0].hp;
    for input in [
        Input {
            punch: true,
            ..Default::default()
        },
        Input {
            kick: true,
            ..Default::default()
        },
    ] {
        g.update(1.0 / 60.0, [input, Input::default()]);
        step(&mut g, 0.48, Input::default());
        assert_eq!(g.enemies[0].hp, full);
        assert_eq!(g.score, 0);
    }
    let grab = Input {
        grab: true,
        ..Default::default()
    };
    g.update(1.0 / 60.0, [grab, Input::default()]);
    assert_eq!(g.props[0].state, PropState::Held(0));
    g.players[0].facing = -1.0;
    step(
        &mut g,
        0.3,
        Input {
            punch: true,
            ..Default::default()
        },
    );
    assert_eq!(g.enemies[0].hp, full);
    assert!(matches!(g.props[0].state, PropState::Broken(_)));
    step(&mut g, 3.2, Input::default());
    assert_eq!(g.props[0].state, PropState::Ready);
}
#[test]
fn guard_blocks_a_facing_punch_but_leaves_the_back_exposed() {
    for (facing, expected) in [(1.0, 100), (-1.0, 95)] {
        let mut g = game(Stage::Tutorial);
        let p = &mut g.players[0];
        p.facing = facing;
        let mut attacker = Fighter::new(Kind::Rookie, p.x + 40.0, p.y);
        attacker.action = Action::Strike {
            attack: Attack::Punch,
            time: 0.94,
            fired: false,
        };
        g.enemies = vec![attacker];
        step(
            &mut g,
            0.04,
            Input {
                guard: true,
                ..Default::default()
            },
        );
        assert_eq!(g.players[0].hp, expected);
        assert_eq!(g.blocked[0] > 0.0, facing > 0.0);
    }
}

#[test]
fn all_three_levels_are_winnable_solo_and_in_coop() {
    for coop in [false, true] {
        let mut g = Game::new();
        g.coop = coop;
        g.start();
        let mut cleared = false;
        for frame in 0..36_000 {
            if g.phase == Phase::StageClear {
                g.advance();
            }
            assert_ne!(
                g.phase,
                Phase::Defeat,
                "coop={coop} stage={:?} frame={frame} boss_hp={}",
                g.stage,
                g.enemies[0].hp
            );
            if g.phase == Phase::ArcadeClear {
                cleared = true;
                break;
            }
            g.update(1.0 / 60.0, autoplay(&g));
        }
        assert!(
            cleared,
            "coop={coop} stage={:?} enemies={:?}",
            g.stage, g.enemies
        );
        assert_eq!(g.stage, Stage::LivingRoom);
        assert!(g.score > 0);
        assert!(g.slams > 0);
        let time = g.elapsed;
        step(&mut g, 10.0, Input::default());
        assert_eq!(g.elapsed, time);
        g.advance();
        assert_eq!(g.phase, Phase::ArcadeClear);
    }
}

#[test]
fn combat_outcome_is_stable_at_different_frame_rates() {
    for hz in [30.0, 60.0, 120.0] {
        let mut g = game(Stage::Hammer);
        for _ in 0..(hz * 240.0) as usize {
            if g.phase != Phase::Playing {
                break;
            }
            g.update(1.0 / hz, autoplay(&g));
        }
        assert_eq!(
            g.phase,
            Phase::StageClear,
            "{hz} Hz, hp={} boss_hp={}",
            g.players[0].hp,
            g.enemies[0].hp
        );
    }
}

#[test]
fn rookies_pursue_and_attack_without_waiting_for_tutorial_inputs() {
    let mut g = game(Stage::Tutorial);
    step(&mut g, 12.0, Input::default());
    assert!(g.players[0].hp < 100);
    assert!(g.enemies.iter().all(|e| e.x < 500.0));
}

#[test]
fn simultaneous_pickup_has_one_owner_and_a_down_player_releases_it() {
    let mut g = game(Stage::Hammer);
    g.coop = true;
    g.retry();
    for p in &mut g.players {
        p.x = g.props[0].x;
        p.y = g.props[0].y;
    }
    g.update(
        1.0 / 60.0,
        [Input {
            grab: true,
            ..Default::default()
        }; 2],
    );
    assert_eq!(g.props[0].state, PropState::Held(0));
    g.players[0].hp = 0;
    g.update(
        1.0 / 60.0,
        [
            Input::default(),
            Input {
                grab: true,
                ..Default::default()
            },
        ],
    );
    assert_eq!(g.props[0].state, PropState::Held(1));
}

#[test]
fn room_deliveries_wait_for_ten_playable_seconds_and_reset_on_retry() {
    let mut g = game(Stage::LivingRoom);
    let start_x = g.players[0].x;
    step(
        &mut g,
        2.0,
        Input {
            x: 1.0,
            ..Default::default()
        },
    );
    assert_eq!(g.stage_time, 0.0);
    assert_eq!(g.players[0].x, start_x);
    while g.intro_left > 0.0 {
        g.update(1.0 / 120.0, [Input::default(); 2]);
    }
    assert_eq!(g.stage_time, 0.0);
    step(&mut g, 9.9, Input::default());
    assert!(g.props.is_empty() && g.delivery.is_none());
    g.toggle_pause();
    step(&mut g, 8.0, Input::default());
    assert!(g.props.is_empty());
    g.toggle_pause();
    step(&mut g, 0.12, Input::default());
    let helper = g.delivery.as_ref().expect("first delivery");
    assert_eq!((helper.x, helper.y), (g.props[0].x, g.props[0].y));
    assert_eq!(g.props[0].state, PropState::Ready);
    step(&mut g, 1.8, Input::default());
    assert!(g.delivery.is_none());
    assert_eq!(g.props.len(), 1);
    g.retry();
    assert_eq!(g.intro_left, ROOM_INTRO);
    assert!(g.props.is_empty() && g.delivery.is_none());
}
