// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::clap;
use structopt::StructOpt;

#[test]
fn required_argument() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        arg: i32,
    }
    assert_eq!(Opt { arg: 42 }, Opt::from_iter(&["test", "42"]));
    assert!(Opt::clap().get_matches_from_safe(&["test"]).is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "42", "24"])
        .is_err());
}

#[test]
fn optional_argument() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        arg: Option<i32>,
    }
    assert_eq!(Opt { arg: Some(42) }, Opt::from_iter(&["test", "42"]));
    assert_eq!(Opt { arg: None }, Opt::from_iter(&["test"]));
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "42", "24"])
        .is_err());
}

#[test]
fn argument_with_default() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(default_value = "42")]
        arg: i32,
    }
    assert_eq!(Opt { arg: 24 }, Opt::from_iter(&["test", "24"]));
    assert_eq!(Opt { arg: 42 }, Opt::from_iter(&["test"]));
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "42", "24"])
        .is_err());
}

#[test]
fn arguments() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        arg: Vec<i32>,
    }
    assert_eq!(Opt { arg: vec![24] }, Opt::from_iter(&["test", "24"]));
    assert_eq!(Opt { arg: vec![] }, Opt::from_iter(&["test"]));
    assert_eq!(
        Opt { arg: vec![24, 42] },
        Opt::from_iter(&["test", "24", "42"])
    );
}

#[test]
fn arguments_safe() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        arg: Vec<i32>,
    }
    assert_eq!(
        Opt { arg: vec![24] },
        Opt::from_iter_safe(&["test", "24"]).unwrap()
    );
    assert_eq!(Opt { arg: vec![] }, Opt::from_iter_safe(&["test"]).unwrap());
    assert_eq!(
        Opt { arg: vec![24, 42] },
        Opt::from_iter_safe(&["test", "24", "42"]).unwrap()
    );

    assert_eq!(
        clap::ErrorKind::ValueValidation,
        Opt::from_iter_safe(&["test", "NOPE"]).err().unwrap().kind
    );
}
