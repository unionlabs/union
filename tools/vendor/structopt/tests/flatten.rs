// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

mod utils;

#[test]
fn flatten() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Common {
        arg: i32,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(flatten)]
        common: Common,
    }
    assert_eq!(
        Opt {
            common: Common { arg: 42 }
        },
        Opt::from_iter(&["test", "42"])
    );
    assert!(Opt::clap().get_matches_from_safe(&["test"]).is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "42", "24"])
        .is_err());
}

#[test]
#[should_panic]
fn flatten_twice() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Common {
        arg: i32,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(flatten)]
        c1: Common,
        // Defines "arg" twice, so this should not work.
        #[structopt(flatten)]
        c2: Common,
    }
    Opt::from_iter(&["test", "42", "43"]);
}

#[test]
fn flatten_in_subcommand() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Common {
        arg: i32,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Add {
        #[structopt(short)]
        interactive: bool,
        #[structopt(flatten)]
        common: Common,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    enum Opt {
        Fetch {
            #[structopt(short)]
            all: bool,
            #[structopt(flatten)]
            common: Common,
        },

        Add(Add),
    }

    assert_eq!(
        Opt::Fetch {
            all: false,
            common: Common { arg: 42 }
        },
        Opt::from_iter(&["test", "fetch", "42"])
    );
    assert_eq!(
        Opt::Add(Add {
            interactive: true,
            common: Common { arg: 43 }
        }),
        Opt::from_iter(&["test", "add", "-i", "43"])
    );
}

#[test]
fn merge_subcommands_with_flatten() {
    #[derive(StructOpt, PartialEq, Debug)]
    enum BaseCli {
        Command1(Command1),
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Command1 {
        arg1: i32,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Command2 {
        arg2: i32,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    enum Opt {
        #[structopt(flatten)]
        BaseCli(BaseCli),
        Command2(Command2),
    }

    assert_eq!(
        Opt::BaseCli(BaseCli::Command1(Command1 { arg1: 42 })),
        Opt::from_iter(&["test", "command1", "42"])
    );
    assert_eq!(
        Opt::Command2(Command2 { arg2: 43 }),
        Opt::from_iter(&["test", "command2", "43"])
    );
}

#[test]
#[should_panic = "structopt misuse: You likely tried to #[flatten] a struct \
                  that contains #[subcommand]. This is forbidden."]
fn subcommand_in_flatten() {
    #[derive(Debug, StructOpt)]
    pub enum Struct1 {
        #[structopt(flatten)]
        Struct1(Struct2),
    }

    #[derive(Debug, StructOpt)]
    pub struct Struct2 {
        #[structopt(subcommand)]
        command_type: Enum3,
    }

    #[derive(Debug, StructOpt)]
    pub enum Enum3 {
        Command { args: Vec<String> },
    }

    Struct1::from_iter(&["test", "command", "foo"]);
}

#[test]
fn flatten_doc_comment() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Common {
        /// This is an arg. Arg means "argument". Command line argument.
        arg: i32,
    }

    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        /// The very important comment that clippy had me put here.
        /// It knows better.
        #[structopt(flatten)]
        common: Common,
    }
    assert_eq!(
        Opt {
            common: Common { arg: 42 }
        },
        Opt::from_iter(&["test", "42"])
    );

    let help = utils::get_help::<Opt>();
    assert!(help.contains("This is an arg."));
    assert!(!help.contains("The very important"));
}
