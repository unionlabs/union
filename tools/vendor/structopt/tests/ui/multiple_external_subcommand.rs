use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(external_subcommand)]
    Run(Vec<String>),

    #[structopt(external_subcommand)]
    Other(Vec<String>)
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
