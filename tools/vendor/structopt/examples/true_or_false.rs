//! How to parse `--foo=true --bar=false` and turn them into bool.

use structopt::StructOpt;

fn true_or_false(s: &str) -> Result<bool, &'static str> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("expected `true` or `false`"),
    }
}

#[derive(StructOpt, Debug, PartialEq)]
struct Opt {
    // Default parser for `try_from_str` is FromStr::from_str.
    // `impl FromStr for bool` parses `true` or `false` so this
    // works as expected.
    #[structopt(long, parse(try_from_str))]
    foo: bool,

    // Of course, this could be done with an explicit parser function.
    #[structopt(long, parse(try_from_str = true_or_false))]
    bar: bool,

    // `bool` can be positional only with explicit `parse(...)` annotation
    #[structopt(parse(try_from_str))]
    boom: bool,
}

fn main() {
    assert_eq!(
        Opt::from_iter(&["test", "--foo=true", "--bar=false", "true"]),
        Opt {
            foo: true,
            bar: false,
            boom: true
        }
    );
    // no beauty, only truth and falseness
    assert!(Opt::from_iter_safe(&["test", "--foo=beauty"]).is_err());
}
