use planet_hoppers::minigames::{
    common::{
        fps::PlayerInput,
        manifest::{from_blender, from_blender_bounds},
    },
    minecraft::{
        mine::MineScene,
        mining::{MiningConfig, MiningSession},
        rush::{RushConfig, RushCue, RushSession, RushState},
        MinecraftConfig,
    },
};
use raylib::prelude::Vector3;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn setup() -> (MineScene, MiningSession, RushSession) {
    let root = root();
    let s = MineScene::load(
        &root.join("assets/models/minigames/minecraft/rush/minecraft_mine.scene.json"),
        MinecraftConfig::load(&root.join("config/minecraft-dev.json")).unwrap(),
    )
    .unwrap();
    let mining = MiningSession::new(
        &s,
        MiningConfig::load(&root.join("config/minecraft-mining.json")).unwrap(),
    );
    let game =
        RushSession::new(RushConfig::load(&root.join("config/minecraft-rush.json")).unwrap());
    (s, mining, game)
}
#[test]
fn expanded_layout_more_than_doubles_and_every_sector_and_diamond_is_accessible() {
    let (s, _, _) = setup();
    assert!(s.data.floor_cells.len() >= 2200);
    assert_eq!(s.data.graph.nodes.len(), 14);
    assert_eq!(s.data.graph.edges.len(), 18);
    assert_eq!(
        s.data
            .interactive_blocks
            .iter()
            .filter(|b| b.kind == "diamond")
            .count(),
        16
    );
    let intact = s.accessibility(false);
    let opened = s.accessibility(true);
    assert!(intact.reachable_chambers.iter().any(|n| n == "D_cavern"));
    assert!(intact.safe_reachable_diamonds.len() >= 8);
    assert_eq!(opened.reachable_chambers.len(), 14);
    assert_eq!(opened.safe_reachable_diamonds.len(), 16);
}
#[test]
fn loading_pause_and_finished_states_never_advance_timer() {
    let (mut s, mut m, mut g) = setup();
    g.advance(20.0);
    assert_eq!(g.remaining, 150.0);
    g.ready();
    g.advance(5.0);
    g.pause();
    g.advance(60.0);
    assert_eq!(g.remaining, 145.0);
    g.resume();
    g.advance(1.0);
    assert_eq!(g.remaining, 144.0);
    m.diamonds = 8;
    g.inspect_world(&mut s, &mut m, true);
    assert_eq!(g.state, RushState::Victory);
    g.advance(1000.0);
    assert_eq!(g.remaining, 144.0);
}
#[test]
fn eight_diamonds_alone_do_not_win_and_exit_requires_time_and_target() {
    let (mut s, mut m, mut g) = setup();
    g.ready();
    g.inspect_world(&mut s, &mut m, true);
    assert_eq!(g.state, RushState::Running);
    assert!(g.notice.is_some());
    m.diamonds = 8;
    g.inspect_world(&mut s, &mut m, false);
    assert_eq!(g.state, RushState::Running);
    assert!(g.objective_met);
    g.inspect_world(&mut s, &mut m, true);
    assert_eq!(g.state, RushState::Victory);
    assert_eq!(g.drain_cues(), vec![RushCue::Success, RushCue::Victory]);
    g.inspect_world(&mut s, &mut m, true);
    assert!(g.drain_cues().is_empty());
}
#[test]
fn timeout_is_clamped_and_warning_and_defeat_fire_once() {
    let (_, _, mut g) = setup();
    g.ready();
    g.advance(119.9);
    assert!(!g.warning);
    g.advance(0.2);
    assert!(g.warning);
    assert_eq!(g.drain_cues(), vec![RushCue::Danger]);
    for _ in 0..50 {
        g.advance(0.1);
    }
    assert!(g.drain_cues().is_empty());
    g.advance(1000.0);
    assert_eq!(g.remaining, 0.0);
    assert_eq!(g.state, RushState::Defeat);
    assert_eq!(g.clock(), "00:00");
    assert_eq!(g.drain_cues(), vec![RushCue::GameOver]);
    g.advance(2.0);
    assert!(g.drain_cues().is_empty());
}
#[test]
fn all_lava_regions_penalize_once_and_return_to_verified_safe_ground() {
    let (mut s, mut m, mut g) = setup();
    g.ready();
    for i in 0..s.data.lava_regions.len() {
        g.lava_protection = 0.0;
        let r = &s.data.lava_regions[i];
        let expected = from_blender(r.safe_return.unwrap());
        s.player.feet = from_blender([
            (r.min[0] + r.max[0]) * 0.5,
            (r.min[1] + r.max[1]) * 0.5,
            0.0,
        ]);
        let before = g.remaining;
        g.inspect_world(&mut s, &mut m, false);
        assert_eq!(g.remaining, before - 10.0);
        assert_eq!(s.player.feet, expected);
        assert!(s.grounded());
        g.inspect_world(&mut s, &mut m, false);
        assert_eq!(g.remaining, before - 10.0);
        assert!(!s.data.lava_regions.iter().any(
            |r| from_blender_bounds(r.min, r.max).overlaps(s.world.body(s.player.feet, 0.35, 1.8))
        ));
    }
}
#[test]
fn lava_that_exhausts_time_loses_immediately_and_cannot_become_victory() {
    let (mut s, mut m, mut g) = setup();
    g.ready();
    g.remaining = 5.0;
    m.diamonds = 8;
    s.player.feet = Vector3::new(-2.5, 0.0, -52.5);
    g.inspect_world(&mut s, &mut m, true);
    assert_eq!(g.state, RushState::Defeat);
    assert_eq!(g.remaining, 0.0);
    assert_eq!(g.drain_cues(), vec![RushCue::GameOver]);
}
#[test]
fn retry_restores_clock_lava_alerts_blocks_and_counter() {
    let (mut s, mut m, mut g) = setup();
    g.ready();
    g.advance(125.0);
    g.lava_protection = 1.0;
    m.diamonds = 8;
    g.inspect_world(&mut s, &mut m, false);
    g.restart(&mut s, &mut m);
    assert_eq!(g.remaining, 150.0);
    assert_eq!(g.state, RushState::Running);
    assert!(!g.warning && !g.objective_met);
    assert_eq!(g.lava_protection, 0.0);
    assert!(g.notice.is_none());
    assert_eq!(m.diamonds, 0);
    assert!(m.blocks.iter().all(|b| b.alive));
    assert!(s.grounded());
}
#[test]
fn countdown_uses_elapsed_time_independent_of_frame_rate() {
    for fps in [20, 30, 60, 144] {
        let (_, _, mut g) = setup();
        g.ready();
        for _ in 0..fps * 10 {
            g.advance(1.0 / fps as f32);
        }
        assert!((g.remaining - 140.0).abs() < 0.015);
    }
}

#[test]
fn expanded_scene_preserves_selective_mining_for_every_block_and_full_reset() {
    let (mut s, mut m, mut g) = setup();
    g.ready();
    let original = s.world.obstacles.len();
    for _ in 0..88 {
        let b = s
            .data
            .interactive_blocks
            .iter()
            .find(|b| {
                m.blocks.iter().any(|v| v.id == b.id && v.alive)
                    && b.occluded_by
                        .iter()
                        .all(|id| m.blocks.iter().any(|v| v.id == *id && !v.alive))
            })
            .unwrap();
        let id = b.id.clone();
        s.player.feet = if let Some(p) = b.approach {
            from_blender(p)
        } else {
            from_blender([b.max[0] + 1.0, b.min[1] + 0.5, 0.0])
        };
        assert!(s.grounded(), "Unsafe reference for {id}");
        let target = m
            .blocks
            .iter()
            .find(|b| b.id == id)
            .unwrap()
            .bounds
            .center();
        let d = (target - s.player.eye()).normalized();
        s.player.yaw = d.z.atan2(d.x);
        s.player.pitch = d.y.asin();
        m.update(&mut s, false, true, 0.0);
        for _ in 0..40 {
            if !m.blocks.iter().find(|b| b.id == id).unwrap().alive {
                break;
            }
            m.update(&mut s, true, true, 0.05);
        }
        assert!(!m.blocks.iter().find(|b| b.id == id).unwrap().alive, "{id}");
    }
    assert_eq!(m.diamonds, 16);
    assert_eq!(s.targets.len(), 1);
    g.inspect_world(&mut s, &mut m, false);
    assert_eq!(g.state, RushState::Running);
    g.restart(&mut s, &mut m);
    assert_eq!(s.world.obstacles.len(), original);
    assert_eq!(m.diamonds, 0);
    assert!(m.blocks.iter().all(|b| b.alive));
    assert_eq!(s.targets.len(), 89);
}

type Cell = (i32, i32);
fn safe_cells(s: &MineScene) -> BTreeSet<Cell> {
    s.data
        .floor_cells
        .iter()
        .filter_map(|c| {
            let p = from_blender([c.x as f32 + 0.5, c.y as f32 + 0.5, 0.0]);
            (s.world.can_occupy(p, 0.35, 1.8)
                && !s.data.lava_regions.iter().any(|r| {
                    from_blender_bounds(r.min, r.max).overlaps(s.world.body(p, 0.35, 1.8))
                }))
            .then_some((c.x, c.y))
        })
        .collect()
}
fn search(start: Cell, cells: &BTreeSet<Cell>) -> BTreeMap<Cell, Cell> {
    let mut prev = BTreeMap::new();
    prev.insert(start, start);
    let mut q = VecDeque::from([start]);
    while let Some((x, y)) = q.pop_front() {
        for n in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if cells.contains(&n) && !prev.contains_key(&n) {
                prev.insert(n, (x, y));
                q.push_back(n);
            }
        }
    }
    prev
}
fn path(prev: &BTreeMap<Cell, Cell>, mut target: Cell) -> Vec<Cell> {
    let mut p = vec![target];
    while prev[&target] != target {
        target = prev[&target];
        p.push(target);
    }
    p.reverse();
    p
}
fn walk(s: &mut MineScene, m: &mut MiningSession, g: &mut RushSession, cells: &[Cell]) -> f32 {
    let mut distance = 0.0;
    for &(x, y) in cells {
        let target = from_blender([x as f32 + 0.5, y as f32 + 0.5, 0.0]);
        let mut guard = 0;
        while (target - s.player.feet).length() > 0.002 {
            let d = target - s.player.feet;
            let before = s.player.feet;
            s.player.yaw = d.z.atan2(d.x);
            s.player.pitch = 0.0;
            g.advance(0.05);
            let input = PlayerInput {
                forward: (d.length() / (s.player.config.walk_speed * 0.05)).min(1.0),
                ..Default::default()
            };
            s.update(input, 0.05, true);
            m.update(s, false, true, 0.05);
            g.inspect_world(s, m, false);
            assert_eq!(g.state, RushState::Running);
            assert!(s.grounded());
            distance += (s.player.feet - before).length();
            guard += 1;
            assert!(guard < 20, "Blocked navigation step at {target:?}");
        }
    }
    distance
}
#[test]
fn two_real_controller_routes_can_mine_eight_and_return_inside_initial_time() {
    let mut reports = vec![];
    for deep_first in [false, true] {
        let (mut s, mut m, mut g) = setup();
        g.ready();
        let cells = safe_cells(&s);
        let mut distance = 0.0;
        let mut collected = vec![];
        for _ in 0..8 {
            let start = (
                s.player.feet.x.floor() as i32,
                (-s.player.feet.z).floor() as i32,
            );
            let prev = search(start, &cells);
            let mut choices: Vec<_> = s
                .data
                .interactive_blocks
                .iter()
                .filter(|b| b.kind == "diamond" && !collected.contains(&b.id))
                .filter_map(|b| {
                    let p = b.approach.unwrap();
                    let cell = (p[0].floor() as i32, p[1].floor() as i32);
                    prev.contains_key(&cell)
                        .then(|| (b.id.clone(), b.sector.clone(), path(&prev, cell)))
                })
                .collect();
            choices.sort_by_key(|(_, sector, p)| {
                (
                    (deep_first && collected.len() < 4 && !matches!(sector.as_str(), "D" | "E"))
                        as usize,
                    p.len(),
                )
            });
            let (id, _, route) = choices.remove(0);
            distance += walk(&mut s, &mut m, &mut g, &route);
            let target = m
                .blocks
                .iter()
                .find(|b| b.id == id)
                .unwrap()
                .bounds
                .center();
            let direction = (target - s.player.eye()).normalized();
            s.player.yaw = direction.z.atan2(direction.x);
            s.player.pitch = direction.y.asin();
            m.update(&mut s, false, true, 0.0);
            for _ in 0..40 {
                if !m.blocks.iter().find(|b| b.id == id).unwrap().alive {
                    break;
                }
                g.advance(0.05);
                m.update(&mut s, true, true, 0.05);
                g.inspect_world(&mut s, &mut m, false);
                assert_eq!(g.state, RushState::Running);
            }
            assert!(!m.blocks.iter().find(|b| b.id == id).unwrap().alive);
            collected.push(id);
        }
        assert_eq!(m.diamonds, 8);
        assert_eq!(g.state, RushState::Running);
        let start = (
            s.player.feet.x.floor() as i32,
            (-s.player.feet.z).floor() as i32,
        );
        let prev = search(start, &cells);
        distance += walk(&mut s, &mut m, &mut g, &path(&prev, (0, 0)));
        let d = (s.targets[0].bounds.center() - s.player.eye()).normalized();
        s.player.yaw = d.z.atan2(d.x);
        s.player.pitch = d.y.asin();
        let exit = s.update(
            PlayerInput {
                interact: true,
                ..Default::default()
            },
            0.0,
            true,
        );
        assert!(exit);
        g.inspect_world(&mut s, &mut m, exit);
        assert_eq!(g.state, RushState::Victory);
        reports.push(serde_json::json!({"strategy":if deep_first{"new sectors first"}else{"nearest safe ore"},"walked_metres":distance,"seconds_used":150.0-g.remaining,"remaining":g.remaining,"diamonds":collected,"method":"Real FPS swept collisions and mining, fixed 20 Hz; instant orientation, known safe routes; no manual playtest."}));
    }
    let report = serde_json::json!({"initial_seconds":150,"routes":reports});
    std::fs::create_dir_all(root().join("target")).unwrap();
    std::fs::write(
        root().join("target/diamond-rush-balance.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("BALANCE {report}");
}
