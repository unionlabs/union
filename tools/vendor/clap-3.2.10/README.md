<!-- omit in TOC -->
# clap

> **Command Line Argument Parser for Rust**

[![Crates.io](https://img.shields.io/crates/v/clap?style=flat-square)](https://crates.io/crates/clap)
[![Crates.io](https://img.shields.io/crates/d/clap?style=flat-square)](https://crates.io/crates/clap)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/clap-rs/clap/blob/v3.2.10/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/clap-rs/clap/blob/v3.2.10/LICENSE-MIT)
[![Build Status](https://img.shields.io/github/workflow/status/clap-rs/clap/CI/staging?style=flat-square)](https://github.com/clap-rs/clap/actions/workflows/ci.yml?query=branch%3Astaging)
[![Coverage Status](https://img.shields.io/coveralls/github/clap-rs/clap/master?style=flat-square)](https://coveralls.io/github/clap-rs/clap?branch=master)
[![Contributors](https://img.shields.io/github/contributors/clap-rs/clap?style=flat-square)](https://github.com/clap-rs/clap/graphs/contributors)

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

1. [About](#about)
2. Tutorial: [Builder API](https://github.com/clap-rs/clap/blob/v3.2.10/examples/tutorial_builder/README.md),  [Derive API](https://github.com/clap-rs/clap/blob/v3.2.10/examples/tutorial_derive/README.md)
3. [Examples](https://github.com/clap-rs/clap/blob/v3.2.10/examples/README.md)
4. [API Reference](https://docs.rs/clap)
    - [Derive Reference](https://github.com/clap-rs/clap/blob/v3.2.10/examples/derive_ref/README.md)
    - [Feature Flags](#feature-flags)
5. [CHANGELOG](https://github.com/clap-rs/clap/blob/v3.2.10/CHANGELOG.md)
6. [FAQ](https://github.com/clap-rs/clap/blob/v3.2.10/docs/FAQ.md)
7. [Questions & Discussions](https://github.com/clap-rs/clap/discussions)
8. [Contributing](https://github.com/clap-rs/clap/blob/v3.2.10/CONTRIBUTING.md)
8. [Sponsors](#sponsors)

## About

Create your command-line parser, with all of the bells and whistles, declaratively or procedurally.

### Example

Add `clap` as a dependency with the `derive` feature enabled:
```console
cargo add clap -F derive
```

This allows using the
[Derive API](https://github.com/clap-rs/clap/blob/v3.2.10/examples/tutorial_derive/README.md)
which provides access to the [Builder API](https://github.com/clap-rs/clap/blob/v3.2.10/examples/tutorial_builder/README.md) as attributes on a `struct`:

<!-- Copied from examples/demo.{rs,md} -->
```rust,no_run
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name)
    }
}
```

```bash
$ demo --help
clap [..]
Simple program to greet a person

USAGE:
    demo[EXE] [OPTIONS] --name <NAME>

OPTIONS:
    -c, --count <COUNT>    Number of times to greet [default: 1]
    -h, --help             Print help information
    -n, --name <NAME>      Name of the person to greet
    -V, --version          Print version information
```
*(version number and `.exe` extension on windows replaced by placeholders)*

### Aspirations

- Out of the box, users get a polished CLI experience
  - Including common argument behavior, help generation, suggested fixes for users, colored output, [shell completions](https://github.com/clap-rs/clap/tree/master/clap_complete), etc
- Flexible enough to port your existing CLI interface
  - However, we won't necessarily streamline support for each use case
- Reasonable parse performance
- Resilient maintainership, including
  - Willing to break compatibility rather than batching up breaking changes in large releases
  - Leverage feature flags to keep to one active branch
  - Being under [WG-CLI](https://github.com/rust-cli/team/) to increase the bus factor
- We follow semver and will wait about 6-9 months between major breaking changes
- We will support the last two minor Rust releases (MSRV, currently 1.56.1)

While these aspirations can be at odds with fast build times and low binary
size, we will still strive to keep these reasonable for the flexibility you
get.  Check out the
[argparse-benchmarks](https://github.com/rust-cli/argparse-benchmarks-rs) for
CLI parsers optimized for other use cases.

### Selecting an API

Why use the declarative [Derive API](https://github.com/clap-rs/clap/blob/v3.2.10/examples/tutorial_derive/README.md):
- Easier to read, write, and modify
- Easier to keep the argument declaration and reading of argument in sync
- Easier to reuse, e.g. [clap-verbosity-flag](https://crates.io/crates/clap-verbosity-flag)

Why use the procedural [Builder API](https://github.com/clap-rs/clap/blob/v3.2.10/examples/tutorial_builder/README.md):
- Faster compile times if you aren't already using other procedural macros
- More flexible, e.g. you can look up how many times an argument showed up,
  what its values were, and what were the indexes of those values.  The Derive
  API can only report presence, number of occurrences, or values but no indices
  or combinations of data.

### Related Projects

- [wild](https://crates.io/crates/wild) for supporting wildcards (`*`) on Windows like you do Linux
- [argfile](https://crates.io/crates/argfile) for loading additional arguments from a file (aka response files)
- [shadow-rs](https://crates.io/crates/shadow-rs) for generating `Command::long_version`
- [clap_lex](https://crates.io/crates/clap_lex) for a lighter-weight, battle-tested CLI parser
- [clap_mangen](https://crates.io/crates/clap_mangen) for generating man page source (roff)
- [clap_complete](https://crates.io/crates/clap_complete) for shell completion support
- [clap-verbosity-flag](https://crates.io/crates/clap-verbosity-flag)
- [clap-cargo](https://crates.io/crates/clap-cargo)
- [concolor-clap](https://crates.io/crates/concolor-clap)
- [Command-line Apps for Rust](https://rust-cli.github.io/book/index.html) book
- [`trycmd`](https://crates.io/crates/trycmd):  Snapshot testing
  - Or for more control, [`assert_cmd`](https://crates.io/crates/assert_cmd) and [`assert_fs`](https://crates.io/crates/assert_fs)

## Feature Flags

### Default Features

* **std**: _Not Currently Used._ Placeholder for supporting `no_std` environments in a backwards compatible manner.
* **color**: Turns on colored error messages.
* **suggestions**: Turns on the `Did you mean '--myoption'?` feature for when users make typos.

#### Optional features

* **deprecated**: Guided experience to prepare for next breaking release (at different stages of development, this may become default)
* **derive**: Enables the custom derive (i.e. `#[derive(Parser)]`). Without this you must use one of the other methods of creating a `clap` CLI listed above.
* **cargo**: Turns on macros that read values from `CARGO_*` environment variables.
* **env**: Turns on the usage of environment variables during parsing.
* **regex**: Enables regex validators.
* **unicode**: Turns on support for unicode characters (including emoji) in arguments and help messages.
* **wrap_help**: Turns on the help text wrapping feature, based on the terminal size.

#### Experimental features

**Warning:** These may contain breaking changes between minor releases.

* **unstable-replace**: Enable [`Command::replace`](https://github.com/clap-rs/clap/issues/2836)
* **unstable-grouped**: Enable [`ArgMatches::grouped_values_of`](https://github.com/clap-rs/clap/issues/2924)
* **unstable-v4**: Preview features which will be stable on the v4.0 release

## Sponsors

<!-- omit in TOC -->
### Gold

[![](https://opencollective.com/clap/tiers/gold.svg?avatarHeight=36&width=600)](https://opencollective.com/clap)

<!-- omit in TOC -->
### Silver

[![](https://opencollective.com/clap/tiers/silver.svg?avatarHeight=36&width=600)](https://opencollective.com/clap)

<!-- omit in TOC -->
### Bronze

[![](https://opencollective.com/clap/tiers/bronze.svg?avatarHeight=36&width=600)](https://opencollective.com/clap)

<!-- omit in TOC -->
### Backer

[![](https://opencollective.com/clap/tiers/backer.svg?avatarHeight=36&width=600)](https://opencollective.com/clap)
