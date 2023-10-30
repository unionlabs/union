use clap::{arg, command};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([NAME]).default_value("alice"))
        .get_matches();

    println!(
        "NAME: {:?}",
        matches
            .get_one::<String>("NAME")
            .expect("default ensures there is always a value")
    );
}
