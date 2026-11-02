use planet_hoppers::minigames::{
    common::{
        fps::PlayerInput,
        interaction::select_target,
        manifest::{from_blender, SceneManifest},
    },
    minecraft::{mine::MineScene, MinecraftConfig},
};
use raylib::prelude::Vector3;
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn scene() -> MineScene {
    MineScene::load(
        &root().join("assets/models/minigames/minecraft/mine/minecraft_mine.scene.json"),
        MinecraftConfig::load(&root().join("config/minecraft-dev.json")).unwrap(),
    )
    .unwrap()
}

#[test]
fn independent_mine_has_three_sectors_two_loops_and_human_scale() {
    let s = scene();
    assert!(s.grounded());
    assert_eq!(s.player.eye().y, 1.7);
    assert_eq!(s.data.graph.nodes.len(), 9);
    assert_eq!(s.data.graph.edges.len(), 10);
    assert_eq!(s.data.graph.edges.len() - s.data.graph.nodes.len() + 1, 2);
    assert_eq!(s.data.interactive_blocks.len(), 26);
    assert_eq!(s.data.lights.len(), 22);
    assert!(s.data.floor_cells.iter().all(|c| c.ceiling >= 3.0));
    for sector in ["A", "B", "C"] {
        assert!(s.data.graph.nodes.iter().any(|n| n.sector == sector));
    }
    for edge in &s.data.graph.edges {
        assert!(s.data.graph.nodes.iter().any(|n| n.id == edge.a));
        assert!(s.data.graph.nodes.iter().any(|n| n.id == edge.b));
        assert!(edge.requires_break.iter().all(|id| s
            .data
            .interactive_blocks
            .iter()
            .any(|b| b.id == *id && b.kind == "breakable")));
    }
}

#[test]
fn ten_diamonds_and_every_main_sector_have_safe_return_routes_without_entering_lava() {
    let s = scene();
    let audit = s.accessibility(false);
    assert_eq!(audit.safe_reachable_diamonds.len(), 10);
    assert_eq!(audit.reachable_chambers.len(), 7);
    for room in [
        "A_entry",
        "A_junction",
        "A_supply",
        "B_hub",
        "B_east",
        "C_lava",
        "C_vault",
    ] {
        assert!(audit.reachable_chambers.iter().any(|r| r == room), "{room}");
    }
    let all = s.accessibility(true);
    assert_eq!(all.safe_reachable_diamonds.len(), 14);
    assert_eq!(all.reachable_chambers.len(), 9);
    assert!(!s.world.obstacles.is_empty()); // Hypothetical validation never mutates live gate state.
    assert_eq!(
        s.world
            .obstacles
            .iter()
            .filter(|o| o.id.starts_with("BREAK_"))
            .count(),
        12
    );
}

#[test]
fn interactive_colliders_are_independent_without_hidden_static_overlap() {
    let s = scene();
    for block in &s.data.interactive_blocks {
        let own = s
            .world
            .obstacles
            .iter()
            .find(|o| o.id == block.collider_id)
            .unwrap();
        assert!(block.initial_visible && block.initial_solid);
        assert_eq!(block.object, block.id);
        for other in s.world.obstacles.iter().filter(|o| o.id != own.id) {
            assert!(
                !own.bounds.overlaps(other.bounds),
                "{} overlaps {}",
                own.id,
                other.id
            );
        }
    }
}

#[test]
fn shared_raycast_selects_each_diamond_and_solid_walls_occlude_it() {
    let s = scene();
    for block in s
        .data
        .interactive_blocks
        .iter()
        .filter(|b| b.kind == "diamond")
    {
        let p = block.approach.unwrap();
        let origin = from_blender([p[0], p[1], 1.7]);
        let target = s.targets.iter().position(|t| t.id == block.id).unwrap();
        let direction = (s.targets[target].bounds.center() - origin).normalized();
        assert_eq!(
            select_target(&s.targets, &s.world, origin, direction, 4.5),
            Some(target),
            "{}",
            block.id
        );
        assert_eq!(
            select_target(&s.targets, &s.world, origin, -direction, 0.2),
            None
        );
    }
    let target = &s.targets[1];
    let behind = target.bounds.center() + Vector3::new(0.0, 0.0, -5.0);
    assert_ne!(
        select_target(
            &s.targets,
            &s.world,
            behind,
            (target.bounds.center() - behind).normalized(),
            6.0
        ),
        Some(1)
    );
}

#[test]
fn walls_and_gate_blocks_prevent_large_sprint_tunnelling() {
    let s = scene();
    let wall = s.world.move_body(
        Vector3::new(3.5, 0.0, -1.0),
        Vector3::new(100.0, 0.0, 0.0),
        0.35,
        1.8,
    );
    assert!((wall.x - 4.6499).abs() < 0.002);
    let gate = s.world.move_body(
        Vector3::new(-7.0, 0.0, -32.5),
        Vector3::new(-100.0, 0.0, 0.0),
        0.35,
        1.8,
    );
    assert!((gate.x + 7.6499).abs() < 0.002);
    assert!(s.world.can_occupy(gate, 0.35, 1.8));
}

#[test]
fn e_does_not_extract_diamonds_and_inactive_controls_are_frozen() {
    let mut s = scene();
    let block = s
        .data
        .interactive_blocks
        .iter()
        .find(|b| b.id == "ORE_DIAMOND_11")
        .unwrap();
    let p = block.approach.unwrap();
    s.player.feet = from_blender(p);
    s.player.yaw = 0.0;
    s.player.pitch = -0.2;
    let colliders = s.world.obstacles.len();
    let old = s.player.feet;
    assert!(!s.update(
        PlayerInput {
            interact: true,
            ..Default::default()
        },
        0.0,
        true
    ));
    assert_eq!(s.world.obstacles.len(), colliders);
    assert_eq!(s.data.interactive_blocks.len(), 26);
    s.update(
        PlayerInput {
            forward: 1.0,
            sprint: true,
            ..Default::default()
        },
        0.1,
        false,
    );
    assert_eq!(s.player.feet, old);
}

#[test]
fn exit_and_future_surface_return_are_safe_and_lava_is_sensor_only() {
    let mut s = scene();
    s.player.yaw = std::f32::consts::FRAC_PI_2;
    assert!(s.update(
        PlayerInput {
            interact: true,
            ..Default::default()
        },
        0.0,
        true
    ));
    assert_eq!(s.data.lava_regions.len(), 2);
    assert!(s.data.lava_regions.iter().all(|r| !r.damage_enabled));
    let surface = SceneManifest::load(
        &root().join("assets/models/minigames/minecraft/minecraft_village.scene.json"),
    )
    .unwrap();
    let p = from_blender(s.data.exit.surface_position) - Vector3::up() * 1.7;
    assert!(surface.collision_world().unwrap().can_occupy(p, 0.35, 1.8));
    assert!(from_blender(s.data.exit.surface_forward).length() > 0.9);
}
