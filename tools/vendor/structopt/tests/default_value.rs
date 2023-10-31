use structopt::StructOpt;

mod utils;

use utils::*;

#[test]
fn auto_default_value() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(default_value)]
        arg: i32,
    }
    assert_eq!(Opt { arg: 0 }, Opt::from_iter(&["test"]));
    assert_eq!(Opt { arg: 1 }, Opt::from_iter(&["test", "1"]));

    let help = get_long_help::<Opt>();
    assert!(help.contains("[default: 0]"));
}
