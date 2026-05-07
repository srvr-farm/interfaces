fn main() {
    if let Err(error) = ifs::run() {
        eprintln!("{error:?}");
        std::process::exit(1);
    }
}
