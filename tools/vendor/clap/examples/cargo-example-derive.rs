use clap::Parser;

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    ExampleDerive(ExampleDeriveArgs),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct ExampleDeriveArgs {
    #[arg(long)]
    manifest_path: Option<std::path::PathBuf>,
}

fn main() {
    let CargoCli::ExampleDerive(args) = CargoCli::parse();
    println!("{:?}", args.manifest_path);
}
