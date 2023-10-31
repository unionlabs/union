// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

#[test]
fn required_option() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, long)]
        arg: i32,
    }
    assert_eq!(
        Opt { arg: 42 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a42"]))
    );
    assert_eq!(
        Opt { arg: 42 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "42"]))
    );
    assert_eq!(
        Opt { arg: 42 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--arg", "42"]))
    );
    assert!(Opt::clap().get_matches_from_safe(&["test"]).is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a42", "-a24"])
        .is_err());
}

#[test]
fn optional_option() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short)]
        arg: Option<i32>,
    }
    assert_eq!(
        Opt { arg: Some(42) },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a42"]))
    );
    assert_eq!(
        Opt { arg: None },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a42", "-a24"])
        .is_err());
}

#[test]
fn option_with_default() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, default_value = "42")]
        arg: i32,
    }
    assert_eq!(
        Opt { arg: 24 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a24"]))
    );
    assert_eq!(
        Opt { arg: 42 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a42", "-a24"])
        .is_err());
}

#[test]
fn option_with_raw_default() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, default_value = "42")]
        arg: i32,
    }
    assert_eq!(
        Opt { arg: 24 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a24"]))
    );
    assert_eq!(
        Opt { arg: 42 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a42", "-a24"])
        .is_err());
}

#[test]
fn options() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, long)]
        arg: Vec<i32>,
    }
    assert_eq!(
        Opt { arg: vec![24] },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a24"]))
    );
    assert_eq!(
        Opt { arg: vec![] },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert_eq!(
        Opt { arg: vec![24, 42] },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a24", "--arg", "42"]))
    );
}

#[test]
fn empy_default_value() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, default_value = "")]
        arg: String,
    }
    assert_eq!(Opt { arg: "".into() }, Opt::from_iter(&["test"]));
    assert_eq!(
        Opt { arg: "foo".into() },
        Opt::from_iter(&["test", "-afoo"])
    );
}

#[test]
fn option_from_str() {
    #[derive(Debug, PartialEq)]
    struct A;

    impl<'a> From<&'a str> for A {
        fn from(_: &str) -> A {
            A
        }
    }

    #[derive(Debug, StructOpt, PartialEq)]
    struct Opt {
        #[structopt(parse(from_str))]
        a: Option<A>,
    }

    assert_eq!(Opt { a: None }, Opt::from_iter(&["test"]));
    assert_eq!(Opt { a: Some(A) }, Opt::from_iter(&["test", "foo"]));
}

#[test]
fn optional_argument_for_optional_option() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short)]
        #[allow(clippy::option_option)]
        arg: Option<Option<i32>>,
    }
    assert_eq!(
        Opt {
            arg: Some(Some(42))
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a42"]))
    );
    assert_eq!(
        Opt { arg: Some(None) },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]))
    );
    assert_eq!(
        Opt { arg: None },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a42", "-a24"])
        .is_err());
}

#[test]
fn two_option_options() {
    #[derive(StructOpt, PartialEq, Debug)]
    #[allow(clippy::option_option)]
    struct Opt {
        #[structopt(short)]
        arg: Option<Option<i32>>,

        #[structopt(long)]
        field: Option<Option<String>>,
    }
    assert_eq!(
        Opt {
            arg: Some(Some(42)),
            field: Some(Some("f".into()))
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a42", "--field", "f"]))
    );
    assert_eq!(
        Opt {
            arg: Some(Some(42)),
            field: Some(None)
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a42", "--field"]))
    );
    assert_eq!(
        Opt {
            arg: Some(None),
            field: Some(None)
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "--field"]))
    );
    assert_eq!(
        Opt {
            arg: Some(None),
            field: Some(Some("f".into()))
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "--field", "f"]))
    );
    assert_eq!(
        Opt {
            arg: None,
            field: Some(None)
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--field"]))
    );
    assert_eq!(
        Opt {
            arg: None,
            field: None
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
}

#[test]
fn optional_vec() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short)]
        arg: Option<Vec<i32>>,
    }
    assert_eq!(
        Opt { arg: Some(vec![1]) },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "1"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1, 2])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a1", "-a2"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1, 2])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a1", "-a2", "-a"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1, 2])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a1", "-a", "-a2"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1, 2])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "1", "2"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1, 2, 3])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "1", "2", "-a", "3"]))
    );

    assert_eq!(
        Opt { arg: Some(vec![]) },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]))
    );

    assert_eq!(
        Opt { arg: Some(vec![]) },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "-a"]))
    );

    assert_eq!(
        Opt { arg: None },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
}

#[test]
fn two_optional_vecs() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short)]
        arg: Option<Vec<i32>>,

        #[structopt(short)]
        b: Option<Vec<i32>>,
    }

    assert_eq!(
        Opt {
            arg: Some(vec![1]),
            b: Some(vec![])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "1", "-b"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1]),
            b: Some(vec![])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "-b", "-a1"]))
    );

    assert_eq!(
        Opt {
            arg: Some(vec![1, 2]),
            b: Some(vec![1, 2])
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a1", "-a2", "-b1", "-b2"]))
    );

    assert_eq!(
        Opt { arg: None, b: None },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
}
