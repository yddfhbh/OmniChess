use omnichess::cli::run_cli;
use std::io;

fn main() {
    if let Err(error) = run_cli(io::stdin().lock(), &mut io::stdout()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
