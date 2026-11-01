fn main() {
    if let Err(error) = planet_hoppers::scenes::minecraft_dev::run() {
        eprintln!("Minecraft: {error}");
        std::process::exit(1);
    }
}
