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

#[derive(StructOpt, PartialEq, Debug)]
enum Opt {
    /// Fetch stuff from GitHub
    Fetch {
        #[structopt(long)]
        all: bool,
        #[structopt(short, long)]
        /// Overwrite local branches.
        force: bool,
        repo: String,
    },

    Add {
        #[structopt(short, long)]
        interactive: bool,
        #[structopt(short, long)]
        verbose: bool,
    },
}

#[test]
fn test_fetch() {
    assert_eq!(
        Opt::Fetch {
            all: true,
            force: false,
            repo: "origin".to_string()
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "fetch", "--all", "origin"]))
    );
    assert_eq!(
        Opt::Fetch {
            all: false,
            force: true,
            repo: "origin".to_string()
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "fetch", "-f", "origin"]))
    );
}

#[test]
fn test_add() {
    assert_eq!(
        Opt::Add {
            interactive: false,
            verbose: false
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "add"]))
    );
    assert_eq!(
        Opt::Add {
            interactive: true,
            verbose: true
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "add", "-i", "-v"]))
    );
}

#[test]
fn test_no_parse() {
    let result = Opt::clap().get_matches_from_safe(&["test", "badcmd", "-i", "-v"]);
    assert!(result.is_err());

    let result = Opt::clap().get_matches_from_safe(&["test", "add", "--badoption"]);
    assert!(result.is_err());

    let result = Opt::clap().get_matches_from_safe(&["test"]);
    assert!(result.is_err());
}

#[derive(StructOpt, PartialEq, Debug)]
enum Opt2 {
    DoSomething { arg: String },
}

#[test]
/// This test is specifically to make sure that hyphenated subcommands get
/// processed correctly.
fn test_hyphenated_subcommands() {
    assert_eq!(
        Opt2::DoSomething {
            arg: "blah".to_string()
        },
        Opt2::from_clap(&Opt2::clap().get_matches_from(&["test", "do-something", "blah"]))
    );
}

#[derive(StructOpt, PartialEq, Debug)]
enum Opt3 {
    Add,
    Init,
    Fetch,
}

#[test]
fn test_null_commands() {
    assert_eq!(
        Opt3::Add,
        Opt3::from_clap(&Opt3::clap().get_matches_from(&["test", "add"]))
    );
    assert_eq!(
        Opt3::Init,
        Opt3::from_clap(&Opt3::clap().get_matches_from(&["test", "init"]))
    );
    assert_eq!(
        Opt3::Fetch,
        Opt3::from_clap(&Opt3::clap().get_matches_from(&["test", "fetch"]))
    );
}

#[derive(StructOpt, PartialEq, Debug)]
#[structopt(about = "Not shown")]
struct Add {
    file: String,
}
/// Not shown
#[derive(StructOpt, PartialEq, Debug)]
struct Fetch {
    remote: String,
}
#[derive(StructOpt, PartialEq, Debug)]
enum Opt4 {
    // Not shown
    /// Add a file
    Add(Add),
    Init,
    /// download history from remote
    Fetch(Fetch),
}

#[test]
fn test_tuple_commands() {
    assert_eq!(
        Opt4::Add(Add {
            file: "f".to_string()
        }),
        Opt4::from_clap(&Opt4::clap().get_matches_from(&["test", "add", "f"]))
    );
    assert_eq!(
        Opt4::Init,
        Opt4::from_clap(&Opt4::clap().get_matches_from(&["test", "init"]))
    );
    assert_eq!(
        Opt4::Fetch(Fetch {
            remote: "origin".to_string()
        }),
        Opt4::from_clap(&Opt4::clap().get_matches_from(&["test", "fetch", "origin"]))
    );

    let output = get_long_help::<Opt4>();

    assert!(output.contains("download history from remote"));
    assert!(output.contains("Add a file"));
    assert!(!output.contains("Not shown"));
}

#[test]
fn enum_in_enum_subsubcommand() {
    #[derive(StructOpt, Debug, PartialEq)]
    pub enum Opt {
        Daemon(DaemonCommand),
    }

    #[derive(StructOpt, Debug, PartialEq)]
    pub enum DaemonCommand {
        Start,
        Stop,
    }

    let result = Opt::clap().get_matches_from_safe(&["test"]);
    assert!(result.is_err());

    let result = Opt::clap().get_matches_from_safe(&["test", "daemon"]);
    assert!(result.is_err());

    let result = Opt::from_iter(&["test", "daemon", "start"]);
    assert_eq!(Opt::Daemon(DaemonCommand::Start), result);
}

#[test]
fn flatten_enum() {
    #[derive(StructOpt, Debug, PartialEq)]
    struct Opt {
        #[structopt(flatten)]
        sub_cmd: SubCmd,
    }
    #[derive(StructOpt, Debug, PartialEq)]
    enum SubCmd {
        Foo,
        Bar,
    }

    assert!(Opt::from_iter_safe(&["test"]).is_err());
    assert_eq!(
        Opt::from_iter(&["test", "foo"]),
        Opt {
            sub_cmd: SubCmd::Foo
        }
    );
}

#[test]
fn external_subcommand() {
    #[derive(Debug, PartialEq, StructOpt)]
    struct Opt {
        #[structopt(subcommand)]
        sub: Subcommands,
    }

    #[derive(Debug, PartialEq, StructOpt)]
    enum Subcommands {
        Add,
        Remove,
        #[structopt(external_subcommand)]
        Other(Vec<String>),
    }

    assert_eq!(
        Opt::from_iter(&["test", "add"]),
        Opt {
            sub: Subcommands::Add
        }
    );

    assert_eq!(
        Opt::from_iter(&["test", "remove"]),
        Opt {
            sub: Subcommands::Remove
        }
    );

    assert_eq!(
        Opt::from_iter(&["test", "git", "status"]),
        Opt {
            sub: Subcommands::Other(vec!["git".into(), "status".into()])
        }
    );

    assert!(Opt::from_iter_safe(&["test"]).is_err());
}

#[test]
fn external_subcommand_os_string() {
    use std::ffi::OsString;

    #[derive(Debug, PartialEq, StructOpt)]
    struct Opt {
        #[structopt(subcommand)]
        sub: Subcommands,
    }

    #[derive(Debug, PartialEq, StructOpt)]
    enum Subcommands {
        #[structopt(external_subcommand)]
        Other(Vec<OsString>),
    }

    assert_eq!(
        Opt::from_iter(&["test", "git", "status"]),
        Opt {
            sub: Subcommands::Other(vec!["git".into(), "status".into()])
        }
    );

    assert!(Opt::from_iter_safe(&["test"]).is_err());
}

#[test]
fn external_subcommand_optional() {
    #[derive(Debug, PartialEq, StructOpt)]
    struct Opt {
        #[structopt(subcommand)]
        sub: Option<Subcommands>,
    }

    #[derive(Debug, PartialEq, StructOpt)]
    enum Subcommands {
        #[structopt(external_subcommand)]
        Other(Vec<String>),
    }

    assert_eq!(
        Opt::from_iter(&["test", "git", "status"]),
        Opt {
            sub: Some(Subcommands::Other(vec!["git".into(), "status".into()]))
        }
    );

    assert_eq!(Opt::from_iter(&["test"]), Opt { sub: None });
}

#[test]
fn skip_subcommand() {
    #[derive(Debug, PartialEq, StructOpt)]
    struct Opt {
        #[structopt(subcommand)]
        sub: Subcommands,
    }

    #[derive(Debug, PartialEq, StructOpt)]
    enum Subcommands {
        Add,
        Remove,

        #[allow(dead_code)]
        #[structopt(skip)]
        Skip,
    }

    assert_eq!(
        Opt::from_iter(&["test", "add"]),
        Opt {
            sub: Subcommands::Add
        }
    );

    assert_eq!(
        Opt::from_iter(&["test", "remove"]),
        Opt {
            sub: Subcommands::Remove
        }
    );

    let res = Opt::from_iter_safe(&["test", "skip"]);
    assert!(
        matches!(
            res,
            Err(clap::Error {
                kind: clap::ErrorKind::UnknownArgument,
                ..
            })
        ),
        "Unexpected result: {:?}",
        res
    );
}
