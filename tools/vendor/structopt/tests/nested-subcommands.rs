// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

#[derive(StructOpt, PartialEq, Debug)]
struct Opt {
    #[structopt(short, long)]
    force: bool,
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u64,
    #[structopt(subcommand)]
    cmd: Sub,
}

#[derive(StructOpt, PartialEq, Debug)]
enum Sub {
    Fetch {},
    Add {},
}

#[derive(StructOpt, PartialEq, Debug)]
struct Opt2 {
    #[structopt(short, long)]
    force: bool,
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u64,
    #[structopt(subcommand)]
    cmd: Option<Sub>,
}

#[test]
fn test_no_cmd() {
    let result = Opt::clap().get_matches_from_safe(&["test"]);
    assert!(result.is_err());

    assert_eq!(
        Opt2 {
            force: false,
            verbose: 0,
            cmd: None
        },
        Opt2::from_clap(&Opt2::clap().get_matches_from(&["test"]))
    );
}

#[test]
fn test_fetch() {
    assert_eq!(
        Opt {
            force: false,
            verbose: 3,
            cmd: Sub::Fetch {}
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-vvv", "fetch"]))
    );
    assert_eq!(
        Opt {
            force: true,
            verbose: 0,
            cmd: Sub::Fetch {}
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--force", "fetch"]))
    );
}

#[test]
fn test_add() {
    assert_eq!(
        Opt {
            force: false,
            verbose: 0,
            cmd: Sub::Add {}
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "add"]))
    );
    assert_eq!(
        Opt {
            force: false,
            verbose: 2,
            cmd: Sub::Add {}
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-vv", "add"]))
    );
}

#[test]
fn test_badinput() {
    let result = Opt::clap().get_matches_from_safe(&["test", "badcmd"]);
    assert!(result.is_err());
    let result = Opt::clap().get_matches_from_safe(&["test", "add", "--verbose"]);
    assert!(result.is_err());
    let result = Opt::clap().get_matches_from_safe(&["test", "--badopt", "add"]);
    assert!(result.is_err());
    let result = Opt::clap().get_matches_from_safe(&["test", "add", "--badopt"]);
    assert!(result.is_err());
}

#[derive(StructOpt, PartialEq, Debug)]
struct Opt3 {
    #[structopt(short, long)]
    all: bool,
    #[structopt(subcommand)]
    cmd: Sub2,
}

#[derive(StructOpt, PartialEq, Debug)]
enum Sub2 {
    Foo {
        file: String,
        #[structopt(subcommand)]
        cmd: Sub3,
    },
    Bar {},
}

#[derive(StructOpt, PartialEq, Debug)]
enum Sub3 {
    Baz {},
    Quux {},
}

#[test]
fn test_subsubcommand() {
    assert_eq!(
        Opt3 {
            all: true,
            cmd: Sub2::Foo {
                file: "lib.rs".to_string(),
                cmd: Sub3::Quux {}
            }
        },
        Opt3::from_clap(
            &Opt3::clap().get_matches_from(&["test", "--all", "foo", "lib.rs", "quux"])
        )
    );
}

#[derive(StructOpt, PartialEq, Debug)]
enum SubSubCmdWithOption {
    Remote {
        #[structopt(subcommand)]
        cmd: Option<Remote>,
    },
    Stash {
        #[structopt(subcommand)]
        cmd: Stash,
    },
}
#[derive(StructOpt, PartialEq, Debug)]
enum Remote {
    Add { name: String, url: String },
    Remove { name: String },
}

#[derive(StructOpt, PartialEq, Debug)]
enum Stash {
    Save,
    Pop,
}

#[test]
fn sub_sub_cmd_with_option() {
    fn make(args: &[&str]) -> Option<SubSubCmdWithOption> {
        SubSubCmdWithOption::clap()
            .get_matches_from_safe(args)
            .ok()
            .map(|m| SubSubCmdWithOption::from_clap(&m))
    }
    assert_eq!(
        Some(SubSubCmdWithOption::Remote { cmd: None }),
        make(&["", "remote"])
    );
    assert_eq!(
        Some(SubSubCmdWithOption::Remote {
            cmd: Some(Remote::Add {
                name: "origin".into(),
                url: "http".into()
            })
        }),
        make(&["", "remote", "add", "origin", "http"])
    );
    assert_eq!(
        Some(SubSubCmdWithOption::Stash { cmd: Stash::Save }),
        make(&["", "stash", "save"])
    );
    assert_eq!(None, make(&["", "stash"]));
}
