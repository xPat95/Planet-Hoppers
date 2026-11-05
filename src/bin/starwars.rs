fn main() {
    if let Err(e) = run() {
        eprintln!("Star Wars: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use planet_hoppers::{
        gameplay::selection::ShipCatalog, minigames::starwars, ships::ShipsConfig,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = ShipsConfig::load(&root.join("config/ships.json"))?;
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &config)?;
    let mut args = std::env::args().skip(1);
    let mut forwarded = vec![];
    let mut selected = config.active().unwrap().clone();
    while let Some(arg) = args.next() {
        if arg == "--ship" {
            let id = args.next().ok_or("--ship necesita una identidad")?;
            selected = if let Some(choice) = catalog.choice(&id) {
                catalog.model(choice, &config)
            } else {
                config
                    .ships
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or("Nave desconocida")?
                    .clone()
            };
        } else {
            forwarded.push(arg);
        }
    }
    let result = starwars::run(root, &selected, forwarded.into_iter())?;
    println!("STARWARS_RETURN {result:?}");
    Ok(())
}
