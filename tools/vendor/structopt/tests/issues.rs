// https://github.com/TeXitoi/structopt/issues/{NUMBER}

mod utils;
use utils::*;

use structopt::StructOpt;

#[test]
fn issue_151() {
    use structopt::{clap::ArgGroup, StructOpt};

    #[derive(StructOpt, Debug)]
    #[structopt(group = ArgGroup::with_name("verb").required(true).multiple(true))]
    struct Opt {
        #[structopt(long, group = "verb")]
        foo: bool,
        #[structopt(long, group = "verb")]
        bar: bool,
    }

    #[derive(Debug, StructOpt)]
    struct Cli {
        #[structopt(flatten)]
        a: Opt,
    }

    assert!(Cli::clap().get_matches_from_safe(&["test"]).is_err());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--foo"])
        .is_ok());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--bar"])
        .is_ok());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--zebra"])
        .is_err());
    assert!(Cli::clap()
        .get_matches_from_safe(&["test", "--foo", "--bar"])
        .is_ok());
}

#[test]
fn issue_289() {
    use structopt::{clap::AppSettings, StructOpt};

    #[derive(StructOpt)]
    #[structopt(setting = AppSettings::InferSubcommands)]
    enum Args {
        SomeCommand(SubSubCommand),
        AnotherCommand,
    }

    #[derive(StructOpt)]
    #[structopt(setting = AppSettings::InferSubcommands)]
    enum SubSubCommand {
        TestCommand,
    }

    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some-command", "test-command"])
        .is_ok());
    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some", "test-command"])
        .is_ok());
    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some-command", "test"])
        .is_ok());
    assert!(Args::clap()
        .get_matches_from_safe(&["test", "some", "test"])
        .is_ok());
}

#[test]
fn issue_324() {
    fn my_version() -> &'static str {
        "MY_VERSION"
    }

    #[derive(StructOpt)]
    #[structopt(version = my_version())]
    struct Opt {
        #[structopt(subcommand)]
        _cmd: Option<SubCommand>,
    }

    #[derive(StructOpt)]
    enum SubCommand {
        Start,
    }

    let help = get_long_help::<Opt>();
    assert!(help.contains("MY_VERSION"));
}

#[test]
fn issue_359() {
    #[derive(Debug, PartialEq, StructOpt)]
    struct Opt {
        #[structopt(subcommand)]
        sub: Subcommands,
    }

    #[derive(Debug, PartialEq, StructOpt)]
    enum Subcommands {
        Add,

        #[structopt(external_subcommand)]
        Other(Vec<String>),
    }

    assert_eq!(
        Opt {
            sub: Subcommands::Other(vec!["only_one_arg".into()])
        },
        Opt::from_iter(&["test", "only_one_arg"])
    );
}

#[test]
fn issue_418() {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    struct Opts {
        #[structopt(subcommand)]
        /// The command to run
        command: Command,
    }

    #[derive(Debug, StructOpt)]
    enum Command {
        /// Reticulate the splines
        #[structopt(visible_alias = "ret")]
        Reticulate {
            /// How many splines
            num_splines: u8,
        },
        /// Frobnicate the rest
        #[structopt(visible_alias = "frob")]
        Frobnicate,
    }

    let help = get_long_help::<Opts>();
    assert!(help.contains("Reticulate the splines [aliases: ret]"));
}

#[test]
fn issue_490() {
    use std::iter::FromIterator;
    use std::str::FromStr;
    use structopt::StructOpt;

    struct U16ish;
    impl FromStr for U16ish {
        type Err = ();
        fn from_str(_: &str) -> Result<Self, Self::Err> {
            unimplemented!()
        }
    }
    impl<'a> FromIterator<&'a U16ish> for Vec<u16> {
        fn from_iter<T: IntoIterator<Item = &'a U16ish>>(_: T) -> Self {
            unimplemented!()
        }
    }

    #[derive(StructOpt, Debug)]
    struct Opt {
        opt_vec: Vec<u16>,
        #[structopt(long)]
        opt_opt_vec: Option<Vec<u16>>,
    }

    // Assert that it compiles
}
