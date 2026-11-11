use planet_hoppers::{
    gameplay::{progress::Franchise, rewards::record_result, selection::{GameState, ShipCatalog, ShipSelection, SelectionMode}, save},
    minigames::common::host::MinigameResult,
    scenes::{abduction::{ART, cover_source}, token_collection::{TokenPresentation, RewardPhase, abduction_length, abduction_seconds, returns_abducted, ABDUCTION_SECONDS, CEREMONY_SECONDS, TOAST_SECONDS}},
    ships::ShipsConfig,
};
use std::path::Path;

fn root() -> &'static Path { Path::new(env!("CARGO_MANIFEST_DIR")) }
fn catalog() -> ShipCatalog {
    let ships = ShipsConfig::load(&root().join("config/ships.json")).unwrap();
    ShipCatalog::load(&root().join("config/ship-selection.json"), &ships).unwrap()
}

#[test]
fn all_initial_ships_abduct_once_without_rewards_even_after_reload() {
    let catalog = catalog();
    for initial in Franchise::ALL {
        let game = GameState::new_game(initial.as_str(), &catalog).unwrap();
        let mut ui = TokenPresentation::default();
        assert!(ui.enqueue_initial(initial));
        assert!(!ui.enqueue_initial(initial));
        ui.advance(0.0,true);
        assert_eq!(ui.abduction(),Some(initial));
        assert!(ui.is_busy());
        assert!(ui.active().is_none());
        ui.advance(abduction_seconds(initial),true);
        assert!(!ui.is_busy());
        assert!(ui.toast().is_none());
        let mut restored: TokenPresentation = serde_json::from_str(&serde_json::to_string(&ui).unwrap()).unwrap();
        assert!(!restored.enqueue_initial(initial));
        assert_eq!(game.progress().token_count(),0);
        assert_eq!(game.unlocked_count(),1);
        for (index, choice) in catalog.choices.iter().enumerate() {
            let selection = ShipSelection { index, mode: SelectionMode::Docked };
            assert_eq!(selection.can_confirm(&catalog,Some(&game)),choice.id == initial.as_str());
            assert_eq!(selection.confirm(&catalog,Some(&game)).is_ok(),choice.id == initial.as_str());
        }
    }
}

#[test]
fn every_pair_of_initial_ship_and_planet_obeys_strict_stage_order() {
    let catalog = catalog();
    for initial in Franchise::ALL {
        for planet in Franchise::ALL {
            let mut game = GameState::new_game(initial.as_str(), &catalog).unwrap();
            let mut ui = TokenPresentation::default();
            let outcome = record_result(&mut game,planet.as_str(),MinigameResult::Victory,&catalog).unwrap();
            assert!(ui.enqueue(planet,outcome,"Nave"));
            ui.advance(100.0,false);
            assert!(ui.phase().is_none());
            ui.advance(0.0,true);
            if initial != planet && returns_abducted(planet) {
                assert_eq!(ui.phase(),Some(RewardPhase::Abduction));
                assert!(ui.active().is_none());
                assert!(ui.toast().is_none());
                ui.advance(abduction_length(planet,false),true);
            }
            assert_eq!(ui.phase(),Some(RewardPhase::Token));
            assert!(ui.is_busy());
            assert!(ui.toast().is_none());
            ui.advance(CEREMONY_SECONDS,true);
            assert!(!ui.is_busy());
            if initial != planet {
                assert_eq!(ui.phase(),Some(RewardPhase::Toast));
                ui.advance(TOAST_SECONDS,true);
            }
            assert!(ui.phase().is_none());
            let replay = record_result(&mut game,planet.as_str(),MinigameResult::Victory,&catalog).unwrap();
            assert!(!ui.enqueue(planet,replay,"Nave"));
            assert_eq!(game.progress().token_count(),1);
            assert_eq!(game.active_ship(),initial.as_str());
        }
    }
}

#[test]
fn save_roundtrip_keeps_ownership_tokens_and_unfinished_stage_independent() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft",&catalog).unwrap();
    let mut ui = TokenPresentation::default();
    ui.enqueue_initial(Franchise::Minecraft);
    ui.advance(ABDUCTION_SECONDS,true);
    let outcome=record_result(&mut game,"halloween",MinigameResult::Victory,&catalog).unwrap();
    ui.enqueue(Franchise::Halloween,outcome,"Escoba");
    ui.advance(abduction_length(Franchise::Halloween,false),true);
    assert_eq!(ui.phase(),Some(RewardPhase::Token));
    let dir=root().join(format!("target/reward-save-{}",std::process::id()));
    let path=dir.join("adventure.json");
    save::store(&path,&game,&ui).unwrap();
    // Replacing an existing snapshot must work on Windows as well.
    save::store(&path,&game,&ui).unwrap();
    let restored=save::load(&path,&catalog).unwrap().unwrap();
    assert_eq!(restored.game,game);
    assert_eq!(restored.presentation.phase(),Some(RewardPhase::Token));
    assert!(restored.game.is_unlocked("minecraft"));
    assert!(!restored.game.progress().is_completed(Franchise::Minecraft));
    assert_eq!(restored.game.progress().token_count(),1);
    let mut ui=restored.presentation;
    assert!(!ui.enqueue_initial(Franchise::Minecraft));
    ui.advance(CEREMONY_SECONDS,true);
    assert_eq!(ui.phase(),Some(RewardPhase::Toast));
    save::store(&path,&restored.game,&ui).unwrap();
    assert_eq!(save::load(&path,&catalog).unwrap().unwrap().presentation.phase(),Some(RewardPhase::Toast));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn six_backgrounds_and_original_cast_exports_are_present_and_cover_without_stretching() {
    use raylib::prelude::Image;
    for art in &ART {
        let image=Image::load_image(root().join(format!("assets/images/abductions/{}.png",art.background)).to_str().unwrap()).unwrap();
        for (w,h) in [(640.0,480.0),(1920.0,1080.0),(2560.0,1080.0),(800.0,1200.0)] {
            let source=cover_source([image.width as f32,image.height as f32],[w,h]);
            assert!((source.width/source.height-w/h).abs()<0.001);
            assert!(source.x>=0.0 && source.y>=0.0);
            assert!(source.width<=image.width as f32+0.001 && source.height<=image.height as f32+0.001);
        }
        for name in art.characters {
            assert!(root().join(format!("assets/models/characters/{name}/{name}.blend")).is_file());
            let bytes=std::fs::read(root().join(format!("assets/models/characters/abductions/{name}.glb"))).unwrap();
            assert_eq!(&bytes[..4],b"glTF");
            let length=u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
            let doc: serde_json::Value=serde_json::from_slice(&bytes[20..20+length]).unwrap();
            assert!(!doc["meshes"].as_array().unwrap().is_empty());
            assert!(!doc["materials"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
#[ignore = "requires OpenGL; renders original models over all six backgrounds"]
fn abduction_visual_smoke() {
    use raylib::prelude::*;
    use planet_hoppers::{scenes::abduction::AbductionRenderer, graphics::resources::save_frame};
    let (mut rl,thread)=raylib::init().size(1280,800).resizable().title("Abducciones / verificación").build();
    rl.set_window_state(WindowState::default().set_window_hidden(true));
    rl.set_target_fps(0);
    let mut renderer=AbductionRenderer::load(&mut rl,&thread,root(),&Franchise::ALL).unwrap();
    let output=root().join("target/abductions");
    std::fs::create_dir_all(&output).unwrap();
    for (w,h) in [(640,480),(1280,800),(1920,1080)] {
        rl.set_window_size(w,h);
        for franchise in Franchise::ALL {
            for (stage,t) in [0.45,1.8,3.0,4.1].into_iter().enumerate() {
                for frame in 0..2 {
                    renderer.prepare(&mut rl,&thread).unwrap();
                    let mut d=rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    renderer.draw(&mut d,&thread,franchise,t,w,h);
                    if frame==1 { save_frame(&output.join(format!("{}-{w}-{stage}.png",franchise.as_str())),w,h).unwrap(); }
                }
            }
        }
    }
    println!("ABDUCTION_SMOKE_PASS franchises=6 sizes=3 stages=4");
}

#[test]
fn smiling_friends_video_lasts_its_frames_and_can_be_skipped() {
    // La duración coincide con el video empaquetado
    let frames = planet_hoppers::scenes::sfv::Frames::load(&root().join("assets/video/smilingfriends/abduction.sfv")).unwrap();
    assert!((frames.duration() - abduction_seconds(Franchise::SmilingFriends)).abs() < 1e-3);
    let mut ui = TokenPresentation::default();
    assert!(ui.enqueue_initial(Franchise::SmilingFriends));
    ui.advance(0.0,true);
    ui.advance(ABDUCTION_SECONDS,true);
    assert_eq!(ui.abduction(),Some(Franchise::SmilingFriends));
    ui.skip_abduction();
    ui.advance(0.0,true);
    assert!(ui.phase().is_none());
}
