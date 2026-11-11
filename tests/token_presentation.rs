use planet_hoppers::{
    gameplay::{progress::Franchise, rewards::{record_result, RewardOutcome}, selection::{GameState, ShipCatalog}},
    minigames::common::host::MinigameResult,
    scenes::token_collection::{CollectionLayout, TokenPresentation, abduction_length, returns_abducted, ABDUCTION_SECONDS, TOAST_SECONDS, CEREMONY_SECONDS, TOKENS, slot_index},
    ships::ShipsConfig,
};
use std::path::Path;

fn root() -> &'static Path { Path::new(env!("CARGO_MANIFEST_DIR")) }
fn fixture(initial: &str) -> (GameState, ShipCatalog) {
    let ships = ShipsConfig::load(&root().join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root().join("config/ship-selection.json"), &ships).unwrap();
    (GameState::new_game(initial, &catalog).unwrap(), catalog)
}

#[test]
fn six_real_pngs_have_canonical_unique_slots_and_original_transparency() {
    use raylib::prelude::Image;
    let mut paths = std::collections::BTreeSet::new();
    for (index, art) in TOKENS.iter().enumerate() {
        assert_eq!(slot_index(art.franchise), index);
        assert_eq!(Franchise::from_id(art.franchise.as_str()).unwrap(), art.franchise);
        assert!(paths.insert(art.png));
        let image = Image::load_image(root().join(art.png).to_str().unwrap()).unwrap();
        assert_eq!((image.width, image.height), (256, 256));
        let pixels = image.get_image_data();
        assert!(pixels.iter().any(|p| p.a == 0));
        assert!(pixels.iter().any(|p| p.a == 255));
    }
    assert_eq!(paths.len(), Franchise::ALL.len());
}

#[test]
fn initial_ship_has_no_medal_and_its_win_has_no_new_ship_announcement() {
    let (mut game, catalog) = fixture("minecraft");
    let mut ui = TokenPresentation::default();
    assert_eq!(game.progress().token_count(), 0);
    assert!(game.is_unlocked("minecraft"));
    let reward = record_result(&mut game, "minecraft", MinigameResult::Victory, &catalog).unwrap();
    assert!(ui.enqueue(Franchise::Minecraft, reward, "End Ship"));
    ui.advance(0.1, true);
    assert_eq!(ui.active().unwrap().ship_name, None);
    assert_eq!(game.progress().token_count(), 1);
}

#[test]
fn notice_is_queued_once_and_survives_replay_loss_and_abandonment() {
    let (mut game, catalog) = fixture("minecraft");
    let mut ui = TokenPresentation::default();
    let reward = record_result(&mut game, "southpark", MinigameResult::Victory, &catalog).unwrap();
    assert!(ui.enqueue(Franchise::SouthPark, reward, "Butters"));
    assert!(!ui.enqueue(Franchise::SouthPark, reward, "Butters"));
    for result in [MinigameResult::Victory, MinigameResult::Defeat, MinigameResult::Abandoned] {
        let reward = record_result(&mut game, "southpark", result, &catalog).unwrap();
        assert!(!ui.enqueue(Franchise::SouthPark, reward, "Butters"));
    }
    ui.advance(1000.0, false); // Still inside the minigame, or behind a scene fade.
    assert!(!ui.is_busy());
    assert_eq!(ui.pending_count(), 1);
    ui.advance(0.1, true);
    assert_eq!(ui.abduction(), Some(Franchise::SouthPark));
    assert!(ui.active().is_none());
    ui.advance(ABDUCTION_SECONDS, true);
    assert_eq!(ui.active().unwrap().ship_name.as_deref(), Some("Butters"));
    let before = ui.elapsed();
    ui.advance(1000.0, false);
    assert_eq!(ui.elapsed(), before);
    ui.advance(CEREMONY_SECONDS, true);
    assert!(!ui.is_busy());
    assert!(!ui.enqueue(Franchise::SouthPark, reward, "Butters"));
    assert_eq!(game.progress().token_count(), 1);
    assert_eq!(game.active_ship(), "minecraft");
}

#[test]
fn fifo_notices_never_own_a_separate_token_counter() {
    let (mut game, catalog) = fixture("minecraft");
    let mut ui = TokenPresentation::default();
    for art in &TOKENS {
        let result = record_result(&mut game, art.franchise.as_str(), MinigameResult::Victory, &catalog).unwrap();
        assert!(ui.enqueue(art.franchise, result, art.name));
    }
    assert_eq!(ui.pending_count(), 6);
    for art in &TOKENS {
        ui.advance(0.0, true);
        if art.franchise != Franchise::Minecraft && returns_abducted(art.franchise) {
            assert_eq!(ui.abduction(), Some(art.franchise));
            ui.advance(abduction_length(art.franchise, false), true);
        }
        assert_eq!(ui.active().unwrap().franchise, art.franchise);
        ui.advance(CEREMONY_SECONDS, true);
        if art.franchise != Franchise::Minecraft {
            assert!(!ui.is_busy());
            assert_eq!(ui.toast().unwrap().franchise, art.franchise);
            ui.advance(TOAST_SECONDS, true);
        }
        assert!(game.progress().is_complete());
        assert_eq!(game.progress().token_count(), 6);
    }
    assert_eq!(ui.pending_count(), 0);
    assert!(!ui.is_busy());
}

#[test]
fn empty_outcomes_and_navigation_never_create_ceremonies() {
    let mut ui = TokenPresentation::default();
    for franchise in Franchise::ALL { assert!(!ui.enqueue(franchise, RewardOutcome::default(), "")); }
    ui.advance(0.0, true);
    ui.advance(f32::NAN, true);
    assert!(!ui.is_busy());
    assert_eq!(ui.pending_count(), 0);
}

#[test]
fn six_slots_remain_inside_every_supported_viewport() {
    for (width, height) in [(640.0,480.0),(960.0,640.0),(1280.0,800.0),(1920.0,1080.0),(1024.0,1024.0)] {
        let layout = CollectionLayout::new(width, height);
        for i in 0..6 {
            let [x, y, w, h] = layout.slot(i);
            let [cx, cy] = layout.medal_center(i);
            assert!(x >= 0.0 && y >= 0.0 && x + w <= width && y + h <= height);
            assert!(cx > x && cx < x + w && cy > y && cy < y + h);
            for j in (i + 1)..6 {
                let [xx, yy, ww, hh] = layout.slot(j);
                assert!(x+w <= xx || xx+ww <= x || y+h <= yy || yy+hh <= y);
            }
        }
    }
}

#[test]
#[ignore = "requires OpenGL; captures actual token PNGs and all ceremony stages"]
fn token_visual_smoke() {
    use raylib::{ffi, prelude::*};
    use planet_hoppers::{graphics::resources::save_frame, scenes::token_collection_ui::TokenTextures};
    fn font(path: &Path) -> Font {
        let path = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();
        let mut glyphs: Vec<i32> = (32..=255).collect();
        let raw = unsafe { ffi::LoadFontEx(path.as_ptr(), 64, glyphs.as_mut_ptr(), glyphs.len() as i32) };
        assert!(raw.texture.id != 0);
        unsafe { ffi::SetTextureFilter(raw.texture, TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
        unsafe { Font::from_raw(raw) }
    }
    let output = root().join("target/token-ui");
    std::fs::create_dir_all(&output).unwrap();
    let (mut rl, thread) = raylib::init().size(1280,800).resizable().title("Tokens / validación visual").build();
    rl.set_window_state(WindowState::default().set_window_hidden(true));
    rl.set_target_fps(0);
    let textures = TokenTextures::load(&mut rl, &thread, root()).unwrap();
    let regular = font(&root().join("assets/fonts/AdventPro-Regular.ttf"));
    let bold = font(&root().join("assets/fonts/AdventPro-Bold.ttf"));
    for (width,height) in [(640,480),(1280,800),(1920,1080)] {
        rl.set_window_size(width,height);
        let (mut game, catalog) = fixture("minecraft");
        for step in 0..7 {
            let mut presentation = TokenPresentation::default();
            let elapsed = match step { 2=>0.18, 3=>1.2, 4=>2.55, 5=>3.15, _=>0.0 };
            if step == 1 {
                record_result(&mut game,"minecraft",MinigameResult::Victory,&catalog).unwrap();
            }
            if (2..=5).contains(&step) {
                let reward = if step == 2 { record_result(&mut game,"southpark",MinigameResult::Victory,&catalog).unwrap() }
                    else { RewardOutcome {token_obtained:true,ship_unlocked:true,progress_completed:false} };
                presentation.enqueue(Franchise::SouthPark,reward,"Butters / nave de South Park");
                presentation.advance(ABDUCTION_SECONDS,true);
                presentation.advance(elapsed,true);
            }
            if step == 6 {
                presentation.enqueue(Franchise::SouthPark,
                    RewardOutcome { token_obtained:true, ship_unlocked:true, progress_completed:false },
                    "Butters / nave de South Park");
                presentation.advance(ABDUCTION_SECONDS,true);
                presentation.advance(CEREMONY_SECONDS,true);
                presentation.advance(0.5,true);
            }
            for frame in 0..3 {
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::new(3,9,24,255));
                textures.draw_hud(&mut d,&bold,game.progress(),width,height);
                if presentation.is_busy() { textures.draw_ceremony(&mut d,&regular,&bold,game.progress(),&presentation,width,height); }
                else if step != 6 { textures.draw_collection(&mut d,&regular,&bold,game.progress(),width,height,1.0,None); }
                textures.draw_ship_notice(&mut d,&bold,&presentation,width,height);
                if frame == 2 { save_frame(&output.join(format!("{width}x{height}-{step}.png")),width,height).unwrap(); }
            }
        }
    }
    println!("TOKEN_UI_SMOKE_PASS sizes=3 pngs=6 stages=7");
}
