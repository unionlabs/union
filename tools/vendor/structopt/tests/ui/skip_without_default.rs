// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

#[derive(Debug)]
enum Kind {
    A,
    B,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "test")]
pub struct Opt {
    #[structopt(short)]
    number: u32,
    #[structopt(skip)]
    k: Kind,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
