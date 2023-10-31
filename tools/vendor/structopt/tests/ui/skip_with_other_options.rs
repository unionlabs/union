use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "test")]
pub struct Opt {
    #[structopt(long)]
    a: u32,
    #[structopt(skip, long)]
    b: u32,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
