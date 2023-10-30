use clap::Parser;

#[derive(Parser)]
#[clap(name = "MyApp")]
#[clap(author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(version = "1.0")]
#[clap(about = "Does awesome things", long_about = None)]
struct Cli {
    #[clap(long, value_parser)]
    two: String,
    #[clap(long, value_parser)]
    one: String,
}

fn main() {
    let cli = Cli::parse();

    println!("two: {:?}", cli.two);
    println!("one: {:?}", cli.one);
}
