fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let legacy = args
        .iter()
        .any(|a| matches!(a.as_str(), "--smoke" | "--smoke-mining"));
    let result = if legacy {
        planet_hoppers::scenes::minecraft_dev::run()
    } else {
        planet_hoppers::scenes::minecraft_rush::run(args)
    };
    if let Err(error) = result {
        eprintln!("Minecraft: {error}");
        std::process::exit(1);
    }
}
