// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

#[test]
fn skip_1() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short)]
        x: u32,
        #[structopt(skip)]
        s: u32,
    }

    assert!(Opt::from_iter_safe(&["test", "-x", "10", "20"]).is_err());
    assert_eq!(
        Opt::from_iter(&["test", "-x", "10"]),
        Opt {
            x: 10,
            s: 0, // default
        }
    );
}

#[test]
fn skip_2() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(short)]
        x: u32,
        #[structopt(skip)]
        ss: String,
        #[structopt(skip)]
        sn: u8,
        y: u32,
        #[structopt(skip)]
        sz: u16,
        t: u32,
    }

    assert_eq!(
        Opt::from_iter(&["test", "-x", "10", "20", "30"]),
        Opt {
            x: 10,
            ss: String::from(""),
            sn: 0,
            y: 20,
            sz: 0,
            t: 30,
        }
    );
}

#[test]
fn skip_enum() {
    #[derive(Debug, PartialEq)]
    #[allow(unused)]
    enum Kind {
        A,
        B,
    }

    impl Default for Kind {
        fn default() -> Self {
            return Kind::B;
        }
    }

    #[derive(StructOpt, Debug, PartialEq)]
    pub struct Opt {
        #[structopt(long, short)]
        number: u32,
        #[structopt(skip)]
        k: Kind,
        #[structopt(skip)]
        v: Vec<u32>,
    }

    assert_eq!(
        Opt::from_iter(&["test", "-n", "10"]),
        Opt {
            number: 10,
            k: Kind::B,
            v: vec![],
        }
    );
}

#[test]
fn skip_help_doc_comments() {
    #[derive(StructOpt, Debug, PartialEq)]
    pub struct Opt {
        #[structopt(skip, help = "internal_stuff")]
        a: u32,

        #[structopt(skip, long_help = "internal_stuff\ndo not touch")]
        b: u32,

        /// Not meant to be used by clap.
        ///
        /// I want a default here.
        #[structopt(skip)]
        c: u32,

        #[structopt(short, parse(try_from_str))]
        n: u32,
    }

    assert_eq!(
        Opt::from_iter(&["test", "-n", "10"]),
        Opt {
            n: 10,
            a: 0,
            b: 0,
            c: 0,
        }
    );
}

#[test]
fn skip_val() {
    #[derive(StructOpt, Debug, PartialEq)]
    pub struct Opt {
        #[structopt(long, short)]
        number: u32,

        #[structopt(skip = "key")]
        k: String,

        #[structopt(skip = vec![1, 2, 3])]
        v: Vec<u32>,
    }

    assert_eq!(
        Opt::from_iter(&["test", "-n", "10"]),
        Opt {
            number: 10,
            k: "key".into(),
            v: vec![1, 2, 3]
        }
    );
}
