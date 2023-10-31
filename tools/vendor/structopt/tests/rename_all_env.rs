mod utils;

use structopt::StructOpt;
use utils::*;

#[test]
fn it_works() {
    #[derive(Debug, PartialEq, StructOpt)]
    #[structopt(rename_all_env = "kebab")]
    struct BehaviorModel {
        #[structopt(env)]
        be_nice: String,
    }

    let help = get_help::<BehaviorModel>();
    assert!(help.contains("[env: be-nice=]"));
}

#[test]
fn default_is_screaming() {
    #[derive(Debug, PartialEq, StructOpt)]
    struct BehaviorModel {
        #[structopt(env)]
        be_nice: String,
    }

    let help = get_help::<BehaviorModel>();
    assert!(help.contains("[env: BE_NICE=]"));
}

#[test]
fn overridable() {
    #[derive(Debug, PartialEq, StructOpt)]
    #[structopt(rename_all_env = "kebab")]
    struct BehaviorModel {
        #[structopt(env)]
        be_nice: String,

        #[structopt(rename_all_env = "pascal", env)]
        be_agressive: String,
    }

    let help = get_help::<BehaviorModel>();
    assert!(help.contains("[env: be-nice=]"));
    assert!(help.contains("[env: BeAgressive=]"));
}
