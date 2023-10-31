use clap::{arg, command, AppSettings, ArgAction};

fn main() {
    let matches = command!() // requires `cargo` feature
        .global_setting(AppSettings::DeriveDisplayOrder)
        .allow_negative_numbers(true)
        .arg(arg!(--two <VALUE>).action(ArgAction::Set))
        .arg(arg!(--one <VALUE>).action(ArgAction::Set))
        .get_matches();

    println!(
        "two: {:?}",
        matches.get_one::<String>("two").expect("required")
    );
    println!(
        "one: {:?}",
        matches.get_one::<String>("one").expect("required")
    );
}
