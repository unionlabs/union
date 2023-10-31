// Note: this requires the `cargo` feature

use clap::{arg, command, value_parser};

fn main() {
    let matches = cmd().get_matches();

    // Note, it's safe to call unwrap() because the arg is required
    let port: usize = *matches
        .get_one::<usize>("PORT")
        .expect("'PORT' is required and parsing will fail if its missing");
    println!("PORT = {}", port);
}

fn cmd() -> clap::Command<'static> {
    command!().arg(
        arg!(<PORT>)
            .help("Network port to use")
            .value_parser(value_parser!(usize)),
    )
}

#[test]
fn verify_app() {
    cmd().debug_assert();
}
