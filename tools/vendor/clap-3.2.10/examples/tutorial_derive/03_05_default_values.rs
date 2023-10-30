use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(default_value_t = String::from("alice"), value_parser)]
    name: String,
}

fn main() {
    let cli = Cli::parse();

    println!("name: {:?}", cli.name);
}
