// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod utils;

use structopt::StructOpt;
use utils::*;

#[test]
fn no_author_version_about() {
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(name = "foo", no_version)]
    struct Opt {}

    let output = get_long_help::<Opt>();
    assert!(output.starts_with("foo \n\nUSAGE:"));
}

#[test]
fn use_env() {
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(author, about)]
    struct Opt {}

    let output = get_long_help::<Opt>();
    assert!(output.starts_with("structopt 0."));
    assert!(output.contains("Guillaume Pinot <texitoi@texitoi.eu>, others"));
    assert!(output.contains("Parse command line argument by defining a struct."));
}

#[test]
fn explicit_version_not_str() {
    const VERSION: &str = "custom version";

    #[derive(StructOpt)]
    #[structopt(version = VERSION)]
    pub struct Opt {}

    let output = get_long_help::<Opt>();
    assert!(output.contains("custom version"));
}

#[test]
fn no_version_gets_propagated() {
    #[derive(StructOpt, PartialEq, Debug)]
    #[structopt(no_version)]
    enum Action {
        Move,
    }

    let output = get_subcommand_long_help::<Action>("move");
    assert_eq!(output.lines().next(), Some("test-move "));
}
