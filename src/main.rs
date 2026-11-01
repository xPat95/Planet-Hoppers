fn main() {
    if let Err(error) = planet_hoppers::scenes::front_end::run() {
        eprintln!("Planet Hoppers: {error}");
        std::process::exit(1);
    }
}
