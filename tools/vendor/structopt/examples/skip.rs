//! How to use `#[structopt(skip)]`

use structopt::StructOpt;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    #[structopt(long, short)]
    number: u32,
    #[structopt(skip)]
    k: Kind,
    #[structopt(skip)]
    v: Vec<u32>,

    #[structopt(skip = Kind::A)]
    k2: Kind,
    #[structopt(skip = vec![1, 2, 3])]
    v2: Vec<u32>,
    #[structopt(skip = "cake")] // &str implements Into<String>
    s: String,
}

#[derive(Debug, PartialEq)]
enum Kind {
    A,
    B,
}

impl Default for Kind {
    fn default() -> Self {
        return Kind::B;
    }
}

fn main() {
    assert_eq!(
        Opt::from_iter(&["test", "-n", "10"]),
        Opt {
            number: 10,
            k: Kind::B,
            v: vec![],

            k2: Kind::A,
            v2: vec![1, 2, 3],
            s: String::from("cake")
        }
    );
}
