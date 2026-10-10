fn main() {
    if let Err(error) = copy_code_cli::run() {
        eprintln!("cpc: {error:#}");
        std::process::exit(1);
    }
}
