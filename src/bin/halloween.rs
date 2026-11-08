fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    match planet_hoppers::minigames::halloween::run_visit(root, std::env::args().skip(1), &mut |_| Ok(())) {
        Ok(result) => println!("HALLOWEEN_RETURN {result:?}"),
        Err(error) => {
            eprintln!("Halloween: {error}");
            std::process::exit(1);
        }
    }
}
