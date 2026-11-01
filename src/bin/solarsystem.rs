fn main() {
    if let Err(error) = planet_hoppers::scenes::solar_system::run() {
        eprintln!("Sistema solar: {error}");
        std::process::exit(1);
    }
}
