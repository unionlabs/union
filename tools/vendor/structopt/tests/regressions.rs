use structopt::StructOpt;

mod utils;
use utils::*;

#[test]
fn invisible_group_issue_439() {
    macro_rules! m {
        ($bool:ty) => {
            #[derive(Debug, StructOpt)]
            struct Opts {
                #[structopt(long = "x")]
                x: $bool,
            }
        };
    }

    m!(bool);

    let help = get_long_help::<Opts>();

    assert!(help.contains("--x"));
    assert!(!help.contains("--x <x>"));
    Opts::from_iter_safe(&["test", "--x"]).unwrap();
}

#[test]
fn issue_447() {
    macro_rules! Command {
        ( $name:ident, [
        #[$meta:meta] $var:ident($inner:ty)
      ] ) => {
            #[derive(Debug, PartialEq, structopt::StructOpt)]
            enum $name {
                #[$meta]
                $var($inner),
            }
        };
    }

    Command! {GitCmd, [
      #[structopt(external_subcommand)]
      Ext(Vec<String>)
    ]}
}
