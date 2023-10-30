use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    println!("name: {:?}", cli.name.as_deref());
}
