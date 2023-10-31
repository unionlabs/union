use structopt::StructOpt;
use std::ffi::CString;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(external_subcommand)]
    Other(Vec<CString>)
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}