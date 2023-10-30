// Note: this requires the `cargo` feature

use clap::{arg, command};

fn main() {
    let matches = command!()
        .arg(arg!([NAME]).default_value("alice"))
        .get_matches();

    println!(
        "NAME: {:?}",
        matches
            .get_one::<String>("NAME")
            .expect("default ensures there is always a value")
    );
}
