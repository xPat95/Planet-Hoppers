fn main() {
    if let Err(error) = planet_hoppers::scenes::smilingfriends_entry::run(std::env::args().skip(1))
    {
        eprintln!("Smiling Friends: {error}");
        std::process::exit(1);
    }
}
