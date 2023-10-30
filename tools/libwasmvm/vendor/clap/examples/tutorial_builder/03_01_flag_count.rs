use clap::{arg, command, ArgAction};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!(-v - -verbose).action(ArgAction::Count))
        .get_matches();

    println!("verbose: {:?}", matches.get_count("verbose"));
}
