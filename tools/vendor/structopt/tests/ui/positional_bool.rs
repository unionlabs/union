use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    verbose: bool,
}

fn main() {
    Opt::from_args();
}