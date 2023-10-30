# StructOpt

[![Build status](https://travis-ci.com/TeXitoi/structopt.svg?branch=master)](https://app.travis-ci.com/github/TeXitoi/structopt) [![](https://img.shields.io/crates/v/structopt.svg)](https://crates.io/crates/structopt) [![](https://docs.rs/structopt/badge.svg)](https://docs.rs/structopt)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Parse command line arguments by defining a struct.  It combines [clap](https://crates.io/crates/clap) with custom derive.

## Maintenance

As clap v3 is now out, and the structopt features are integrated into (almost as-is), structopt is now in maintenance mode: no new feature will be added.

Bugs will be fixed, and documentation improvements will be accepted.

## Documentation

Find it on [Docs.rs](https://docs.rs/structopt).  You can also check the [examples](https://github.com/TeXitoi/structopt/tree/master/examples) and the [changelog](https://github.com/TeXitoi/structopt/blob/master/CHANGELOG.md).

## Example

Add `structopt` to your dependencies of your `Cargo.toml`:
```toml
[dependencies]
structopt = "0.3"
```

And then, in your rust file:
```rust
use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag. The name of the
    // argument will be, by default, based on the name of the field.
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Set speed
    #[structopt(short, long, default_value = "42")]
    speed: f64,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    // the long option will be translated by default to kebab case,
    // i.e. `--nb-cars`.
    /// Number of cars
    #[structopt(short = "c", long)]
    nb_cars: Option<i32>,

    /// admin_level to consider
    #[structopt(short, long)]
    level: Vec<String>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
}
```

Using this example:
```
$ ./basic
error: The following required arguments were not provided:
    --output <output>

USAGE:
    basic --output <output> --speed <speed>

For more information try --help
$ ./basic --help
basic 0.3.0
Guillaume Pinot <texitoi@texitoi.eu>, others
A basic example

USAGE:
    basic [FLAGS] [OPTIONS] --output <output> [--] [file]...

FLAGS:
    -d, --debug      Activate debug mode
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)

OPTIONS:
    -l, --level <level>...     admin_level to consider
    -c, --nb-cars <nb-cars>    Number of cars
    -o, --output <output>      Output file
    -s, --speed <speed>        Set speed [default: 42]

ARGS:
    <file>...    Files to process
$ ./basic -o foo.txt
Opt {
    debug: false,
    verbose: 0,
    speed: 42.0,
    output: "foo.txt",
    nb_cars: None,
    level: [],
    files: [],
}
$ ./basic -o foo.txt -dvvvs 1337 -l alice -l bob --nb-cars 4 bar.txt baz.txt
Opt {
    debug: true,
    verbose: 3,
    speed: 1337.0,
    output: "foo.txt",
    nb_cars: Some(
        4,
    ),
    level: [
        "alice",
        "bob",
    ],
    files: [
        "bar.txt",
        "baz.txt",
    ],
}
```

## StructOpt rustc version policy

- Minimum rustc version modification must be specified in the [changelog](https://github.com/TeXitoi/structopt/blob/master/CHANGELOG.md) and in the [travis configuration](https://github.com/TeXitoi/structopt/blob/master/.travis.yml).
- Contributors can increment minimum rustc version without any justification if the new version is required by the latest version of one of StructOpt's dependencies (`cargo update` will not fail on StructOpt).
- Contributors can increment minimum rustc version if the library user experience is improved.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
