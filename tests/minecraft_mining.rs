use planet_hoppers::minigames::{
    common::{fps::PlayerInput, manifest::from_blender},
    minecraft::{
        mine::MineScene,
        mining::{BlockKind, MiningConfig, MiningEvent, MiningSession},
        MinecraftConfig,
    },
};
use raylib::prelude::Vector3;
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn setup() -> (MineScene, MiningSession) {
    let s = MineScene::load(
        &root().join("assets/models/minigames/minecraft/mining/minecraft_mine.scene.json"),
        MinecraftConfig::load(&root().join("config/minecraft-dev.json")).unwrap(),
    )
    .unwrap();
    let mut m = MiningSession::new(
        &s,
        MiningConfig::load(&root().join("config/minecraft-mining.json")).unwrap(),
    );
    let mut s = s;
    m.update(&mut s, false, true, 0.0);
    (s, m)
}
fn aim(s: &mut MineScene, id: &str) {
    let b = s
        .data
        .interactive_blocks
        .iter()
        .find(|b| b.id == id)
        .unwrap();
    s.player.feet = if let Some(p) = b.approach {
        from_blender(p)
    } else {
        from_blender([b.max[0] + 1.0, b.min[1] + 0.5, 0.0])
    };
    let center = from_blender([
        (b.min[0] + b.max[0]) * 0.5,
        (b.min[1] + b.max[1]) * 0.5,
        (b.min[2] + b.max[2]) * 0.5,
    ]);
    let d = (center - s.player.eye()).normalized();
    s.player.yaw = d.z.atan2(d.x);
    s.player.pitch = d.y.asin();
}
fn mine(s: &mut MineScene, m: &mut MiningSession, id: &str) {
    aim(s, id);
    m.update(s, false, true, 0.0);
    let kind = m.blocks.iter().find(|b| b.id == id).unwrap().kind;
    for _ in 0..(m.config.duration(kind) * 60.0).ceil() as usize + 2 {
        if !m.blocks.iter().find(|b| b.id == id).unwrap().alive {
            break;
        }
        m.update(s, true, true, 1.0 / 60.0);
    }
    assert!(!m.blocks.iter().find(|b| b.id == id).unwrap().alive, "{id}");
}

#[test]
fn all_seven_materials_are_designated_and_times_are_centralized() {
    let (s, m) = setup();
    assert_eq!(m.blocks.len(), 86);
    assert_eq!(s.targets.len(), 87);
    for (k, t) in [
        (BlockKind::Stone, 0.7),
        (BlockKind::Coal, 0.9),
        (BlockKind::Iron, 1.2),
        (BlockKind::Lapis, 1.3),
        (BlockKind::Redstone, 1.4),
        (BlockKind::Gold, 1.5),
        (BlockKind::Diamond, 1.8),
    ] {
        assert_eq!(m.config.duration(k), t);
        assert!(m.blocks.iter().any(|b| b.kind == k));
    }
}

#[test]
fn hold_advances_and_release_resets_progress_and_cracks() {
    let (mut s, mut m) = setup();
    aim(&mut s, "ORE_DIAMOND_11");
    m.update(&mut s, true, true, 0.8);
    assert!((m.progress() - 0.8 / 1.8).abs() < 0.001);
    assert!(m.crack_stage() >= 4);
    m.update(&mut s, false, true, 0.0);
    assert!(m.active.is_none());
    assert_eq!(m.crack_stage(), 0);
    m.update(&mut s, true, true, 0.1);
    assert!(m.progress() < 0.06);
}

#[test]
fn changing_target_discards_previous_work() {
    let (mut s, mut m) = setup();
    aim(&mut s, "ORE_DIAMOND_11");
    m.update(&mut s, true, true, 0.9);
    aim(&mut s, "ORE_DIAMOND_12");
    m.update(&mut s, true, true, 0.1);
    let active = m.active.unwrap();
    assert_eq!(m.blocks[active.block].id, "ORE_DIAMOND_12");
    assert!((active.elapsed - 0.1).abs() < 0.001);
}

#[test]
fn reach_direction_and_static_walls_prevent_mining() {
    let (mut s, mut m) = setup();
    aim(&mut s, "ORE_DIAMOND_11");
    let forward = s.player.direction();
    s.player.feet -= Vector3::new(forward.x, 0.0, forward.z) * 6.0;
    m.update(&mut s, true, true, 2.0);
    assert!(m.active.is_none());
    assert_eq!(m.diamonds, 0);
    let b = m
        .blocks
        .iter()
        .find(|b| b.id == "ORE_DIAMOND_11")
        .unwrap()
        .bounds;
    s.player.feet = b.center() + Vector3::new(2.0, -b.center().y, 0.0);
    let d = (b.center() - s.player.eye()).normalized();
    s.player.yaw = d.z.atan2(d.x);
    s.player.pitch = d.y.asin();
    m.update(&mut s, true, true, 2.0);
    assert!(m.active.is_none());
    assert!(m.blocks.iter().all(|b| b.alive));
}

#[test]
fn menu_and_lost_capture_cancel_and_require_button_release() {
    let (mut s, mut m) = setup();
    aim(&mut s, "ORE_DIAMOND_11");
    m.update(&mut s, true, true, 0.7);
    m.update(&mut s, true, false, 0.1);
    assert!(m.active.is_none());
    m.update(&mut s, true, true, 2.0);
    assert_eq!(m.diamonds, 0);
    assert!(m.active.is_none());
    m.update(&mut s, false, true, 0.0);
    m.update(&mut s, true, true, 0.2);
    assert!(m.active.is_some());
}

#[test]
fn diamond_removes_target_collider_and_cannot_award_twice() {
    let (mut s, mut m) = setup();
    let count = s.world.obstacles.len();
    mine(&mut s, &mut m, "ORE_DIAMOND_11");
    assert_eq!(m.diamonds, 1);
    assert_eq!(s.world.obstacles.len(), count - 1);
    assert!(!s.targets.iter().any(|t| t.id == "ORE_DIAMOND_11"));
    assert!(!s.world.obstacles.iter().any(|c| c.id == "ORE_DIAMOND_11"));
    for _ in 0..300 {
        m.update(&mut s, true, true, 1.0 / 60.0);
    }
    assert_eq!(m.diamonds, 1);
    assert_eq!(
        m.drain_events()
            .iter()
            .filter(|e| matches!(e, MiningEvent::DiamondCollected { .. }))
            .count(),
        1
    );
}

#[test]
fn all_common_minerals_break_without_awarding_diamonds() {
    let (mut s, mut m) = setup();
    for kind in [
        BlockKind::Coal,
        BlockKind::Iron,
        BlockKind::Gold,
        BlockKind::Lapis,
        BlockKind::Redstone,
    ] {
        let id = s
            .data
            .interactive_blocks
            .iter()
            .find(|b| BlockKind::from_manifest(&b.kind) == Some(kind) && b.occluded_by.is_empty())
            .unwrap()
            .id
            .clone();
        mine(&mut s, &mut m, &id);
        assert_eq!(m.diamonds, 0);
        assert!(!s.world.obstacles.iter().any(|c| c.id == id));
    }
    assert_eq!(
        m.drain_events()
            .iter()
            .filter(|e| matches!(e, MiningEvent::CommonOreCollected { .. }))
            .count(),
        5
    );
}

#[test]
fn common_ore_slots_have_no_overlapping_structural_collision() {
    let (s, m) = setup();
    for b in &m.blocks {
        assert!(
            s.world
                .obstacles
                .iter()
                .filter(|c| c.id != b.id)
                .all(|c| !c.bounds.overlaps(b.bounds)),
            "{}",
            b.id
        );
    }
}

#[test]
fn opening_stone_gate_exposes_diamonds_and_player_can_cross() {
    let (mut s, mut m) = setup();
    let before = s.accessibility(false).safe_reachable_diamonds.len();
    let ids: Vec<_> = m
        .blocks
        .iter()
        .filter(|b| b.gate.as_deref() == Some("B_gate"))
        .map(|b| b.id.clone())
        .collect();
    for id in ids {
        mine(&mut s, &mut m, &id);
    }
    let start = Vector3::new(-7.0, 0.0, -32.5);
    let end = s
        .world
        .move_body(start, Vector3::new(-7.0, 0.0, 0.0), 0.35, 1.8);
    assert!(end.x < -13.0);
    assert!(s.world.can_occupy(end, 0.35, 1.8));
    assert_eq!(
        s.accessibility(false).safe_reachable_diamonds.len(),
        before + 2
    );
    assert_eq!(
        m.drain_events()
            .iter()
            .filter(|e| matches!(e, MiningEvent::PassageUnlocked { .. }))
            .count(),
        1
    );
}

#[test]
fn a_removed_front_block_allows_rays_to_reach_the_upper_gate_block() {
    let (mut s, mut m) = setup();
    mine(&mut s, &mut m, "BREAK_B_gate_32_0");
    mine(&mut s, &mut m, "BREAK_B_gate_32_1");
    assert!(!s.targets.iter().any(|t| t.id == "BREAK_B_gate_32_0"));
    assert!(s.targets.iter().any(|t| t.id == "BREAK_B_gate_32_2"));
}

#[test]
fn reset_restores_session_targets_solids_counter_and_safe_spawn() {
    let (mut s, mut m) = setup();
    let colliders = s.world.obstacles.len();
    let targets = s.targets.len();
    mine(&mut s, &mut m, "ORE_DIAMOND_11");
    mine(&mut s, &mut m, "BREAK_B_gate_32_0");
    m.reset_mining_session(&mut s);
    assert_eq!(m.diamonds, 0);
    assert_eq!(s.world.obstacles.len(), colliders);
    assert_eq!(s.targets.len(), targets);
    assert!(m.blocks.iter().all(|b| b.alive));
    assert!(m.active.is_none() && m.hovered.is_none());
    assert_eq!(m.crack_stage(), 0);
    assert!(s.grounded());
    assert_eq!(m.drain_events().len(), 1);
    aim(&mut s, "ORE_DIAMOND_11");
    m.update(&mut s, true, true, 2.0);
    assert_eq!(m.diamonds, 0);
    m.update(&mut s, false, true, 0.0);
    mine(&mut s, &mut m, "ORE_DIAMOND_11");
    assert_eq!(m.diamonds, 1);
}

#[test]
fn timing_and_audio_hooks_follow_elapsed_seconds_at_different_frame_rates() {
    for fps in [20, 30, 60, 144] {
        let (mut s, mut m) = setup();
        aim(&mut s, "ORE_DIAMOND_11");
        let mut elapsed = 0.0;
        while m.diamonds == 0 {
            m.update(&mut s, true, true, 1.0 / fps as f32);
            elapsed += 1.0 / fps as f32;
            assert!(elapsed < 2.0);
        }
        assert!((elapsed - 1.8).abs() < 1.0 / fps as f32 + 0.001);
        let events = m.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, MiningEvent::SwingStarted { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, MiningEvent::Impact { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, MiningEvent::BlockDestroyed { .. })));
    }
}

#[test]
fn mining_keeps_structural_floor_roof_and_supports_intact() {
    let (mut s, mut m) = setup();
    let structural: Vec<_> = s
        .world
        .obstacles
        .iter()
        .filter(|o| !m.blocks.iter().any(|b| b.id == o.id))
        .map(|o| o.id.clone())
        .collect();
    mine(&mut s, &mut m, "ORE_DIAMOND_11");
    assert!(structural
        .iter()
        .all(|id| s.world.obstacles.iter().any(|o| o.id == *id)));
    s.update(PlayerInput::default(), 0.0, true);
    assert!(s.grounded());
}

#[test]
fn concealed_common_ore_is_blocked_until_front_diamond_is_removed() {
    let (mut s, mut m) = setup();
    aim(&mut s, "ORE_REDSTONE_06");
    m.update(&mut s, true, true, 0.1);
    assert_eq!(
        m.blocks[m.active.as_ref().unwrap().block].id,
        "ORE_DIAMOND_07"
    );
    m.update(&mut s, false, true, 0.0);
    mine(&mut s, &mut m, "ORE_DIAMOND_07");
    mine(&mut s, &mut m, "ORE_REDSTONE_06");
    assert_eq!(m.diamonds, 1);
}

#[test]
fn every_designated_block_is_really_mineable_in_dependency_order() {
    let (mut s, mut m) = setup();
    for _ in 0..86 {
        let next = s
            .data
            .interactive_blocks
            .iter()
            .find(|b| {
                m.blocks.iter().any(|v| v.id == b.id && v.alive)
                    && b.occluded_by
                        .iter()
                        .all(|id| m.blocks.iter().any(|v| v.id == *id && !v.alive))
            })
            .unwrap()
            .id
            .clone();
        aim(&mut s, &next);
        assert!(s.grounded(), "Unsafe approach {next}");
        mine(&mut s, &mut m, &next);
    }
    assert_eq!(m.diamonds, 14);
    assert_eq!(s.targets.len(), 1);
    assert!(m.blocks.iter().all(|b| !b.alive));
    assert!(s.grounded());
    assert_eq!(s.accessibility(false).reachable_chambers.len(), 9);
}

#[test]
fn a_new_solid_obstruction_cancels_in_progress_mining_from_a_valid_position() {
    use planet_hoppers::minigames::common::collision::{Aabb, Obstacle};
    let (mut s, mut m) = setup();
    aim(&mut s, "ORE_DIAMOND_11");
    s.player.feet.x -= 1.5;
    let center = m
        .blocks
        .iter()
        .find(|b| b.id == "ORE_DIAMOND_11")
        .unwrap()
        .bounds
        .center();
    let d = (center - s.player.eye()).normalized();
    s.player.yaw = d.z.atan2(d.x);
    s.player.pitch = d.y.asin();
    assert!(s.grounded());
    m.update(&mut s, true, true, 0.7);
    assert!(m.active.is_some());
    s.world.obstacles.push(Obstacle {
        id: "test-solid-wall".into(),
        bounds: Aabb::new(
            Vector3::new(23.0, 0.0, -49.0),
            Vector3::new(23.1, 3.0, -48.0),
        ),
    });
    assert!(s.grounded());
    m.update(&mut s, true, true, 0.1);
    assert!(m.active.is_none());
    assert_eq!(m.diamonds, 0);
}
