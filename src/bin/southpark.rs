fn main() {
    if let Err(error) = planet_hoppers::scenes::southpark_dev::run() {
        eprintln!("South Park: {error}");
        std::process::exit(1);
    }
}
