clap
====

[![Crates.io](https://img.shields.io/crates/v/clap.svg)](https://crates.io/crates/clap) [![Crates.io](https://img.shields.io/crates/d/clap.svg)](https://crates.io/crates/clap) [![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/clap-rs/clap/blob/master/LICENSE-MIT) [![Coverage Status](https://coveralls.io/repos/kbknapp/clap-rs/badge.svg?branch=master&service=github)](https://coveralls.io/github/kbknapp/clap-rs?branch=master) [![Join the chat at https://gitter.im/kbknapp/clap-rs](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/kbknapp/clap-rs?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Linux: [![Build Status](https://travis-ci.org/clap-rs/clap.svg?branch=master)](https://travis-ci.org/clap-rs/clap)
Windows: [![Build status](https://ci.appveyor.com/api/projects/status/ejg8c33dn31nhv36/branch/master?svg=true)](https://ci.appveyor.com/project/kbknapp/clap-rs/branch/master)

Command Line Argument Parser for Rust

It is a simple-to-use, efficient, and full-featured library for parsing command line arguments and subcommands when writing console/terminal applications.

* [documentation](https://docs.rs/clap/)
* [website](https://clap.rs/)
* [video tutorials](https://www.youtube.com/playlist?list=PLza5oFLQGTl2Z5T8g1pRkIynR3E0_pc7U)

Table of Contents
=================

* [About](#about)
* [FAQ](#faq)
* [Features](#features)
* [Quick Example](#quick-example)
* [Try it!](#try-it)
  * [Pre-Built Test](#pre-built-test)
  * [BYOB (Build Your Own Binary)](#byob-build-your-own-binary)
* [Usage](#usage)
  * [Optional Dependencies / Features](#optional-dependencies--features)
  * [Dependencies Tree](#dependencies-tree)
  * [More Information](#more-information)
    * [Video Tutorials](#video-tutorials)
* [How to Contribute](#how-to-contribute)
  * [Compatibility Policy](#compatibility-policy)
    * [Minimum Version of Rust](#minimum-version-of-rust)
* [Related Crates](#related-crates)
* [License](#license)
* [Recent Breaking Changes](#recent-breaking-changes)
  * [Deprecations](#deprecations)

Created by [gh-md-toc](https://github.com/ekalinin/github-markdown-toc)

## About

`clap` is used to parse *and validate* the string of command line arguments provided by a user at runtime. You provide the list of valid possibilities, and `clap` handles the rest. This means you focus on your *applications* functionality, and less on the parsing and validating of arguments.

`clap` provides many things 'for free' (with no configuration) including the traditional version and help switches (or flags) along with associated messages. If you are using subcommands, `clap` will also auto-generate a `help` subcommand and separate associated help messages.

Once `clap` parses the user provided string of arguments, it returns the matches along with any applicable values. If the user made an error or typo, `clap` informs them with a friendly message and exits gracefully (or returns a `Result` type and allows you to perform any clean up prior to exit). Because of this, you can make reasonable assumptions in your code about the validity of the arguments prior to your applications main execution.

## FAQ

For a full FAQ and more in depth details, see [the wiki page](https://github.com/clap-rs/clap/wiki/FAQ)

### Comparisons

First, let me say that these comparisons are highly subjective, and not meant in a critical or harsh manner. All the argument parsing libraries out there (to include `clap`) have their own strengths and weaknesses. Sometimes it just comes down to personal taste when all other factors are equal. When in doubt, try them all and pick one that you enjoy :) There's plenty of room in the Rust community for multiple implementations!

#### How does `clap` compare to [getopts](https://github.com/rust-lang-nursery/getopts)?

`getopts` is a very basic, fairly minimalist argument parsing library. This isn't a bad thing, sometimes you don't need tons of features, you just want to parse some simple arguments, and have some help text generated for you based on valid arguments you specify. The downside to this approach is that you must manually implement most of the common features (such as checking to display help messages, usage strings, etc.). If you want a highly custom argument parser, and don't mind writing the majority of the functionality yourself, `getopts` is an excellent base.

`getopts` also doesn't allocate much, or at all. This gives it a very small performance boost. Although, as you start implementing additional features, that boost quickly disappears.

Personally, I find many, many uses of `getopts` are manually implementing features that `clap` provides by default. Using `clap` simplifies your codebase allowing you to focus on your application, and not argument parsing.

#### How does `clap` compare to [docopt.rs](https://github.com/docopt/docopt.rs)?

I first want to say I'm a big a fan of BurntSushi's work, the creator of `Docopt.rs`. I aspire to produce the quality of libraries that this man does! When it comes to comparing these two libraries they are very different. `docopt` tasks you with writing a help message, and then it parsers that message for you to determine all valid arguments and their use. Some people LOVE this approach, others do not. If you're willing to write a detailed help message, it's nice that you can stick that in your program and have `docopt` do the rest. On the downside, it's far less flexible.

`docopt` is also excellent at translating arguments into Rust types automatically. There is even a syntax extension which will do all this for you, if you're willing to use a nightly compiler (use of a stable compiler requires you to somewhat manually translate from arguments to Rust types). To use BurntSushi's words, `docopt` is also a sort of black box. You get what you get, and it's hard to tweak implementation or customize the experience for your use case.

Because `docopt` is doing a ton of work to parse your help messages and determine what you were trying to communicate as valid arguments, it's also one of the more heavy weight parsers performance-wise. For most applications this isn't a concern and this isn't to say `docopt` is slow, in fact far from it. This is just something to keep in mind while comparing.

#### All else being equal, what are some reasons to use `clap`? (The Pitch)

`clap` is as fast, and as lightweight as possible while still giving all the features you'd expect from a modern argument parser. In fact, for the amount and type of features `clap` offers it remains about as fast as `getopts`. If you use `clap` when just need some simple arguments parsed, you'll find it's a walk in the park. `clap` also makes it possible to represent extremely complex, and advanced requirements, without too much thought. `clap` aims to be intuitive, easy to use, and fully capable for wide variety use cases and needs.

#### All else being equal, what are some reasons *not* to use `clap`? (The Anti Pitch)

Depending on the style in which you choose to define the valid arguments, `clap` can be very verbose. `clap` also offers so many fine-tuning knobs and dials, that learning everything can seem overwhelming. I strive to keep the simple cases simple, but when turning all those custom dials it can get complex. `clap` is also opinionated about parsing. Even though so much can be tweaked and tuned with `clap` (and I'm adding more all the time), there are still certain features which `clap` implements in specific ways which may be contrary to some users use-cases. Finally, `clap` is "stringly typed" when referring to arguments which can cause typos in code. This particular paper-cut is being actively worked on, and should be gone in v3.x.

## Features

Below are a few of the features which `clap` supports, full descriptions and usage can be found in the [documentation](https://docs.rs/clap/) and [examples/](examples) directory

* **Auto-generated Help, Version, and Usage information**
  - Can optionally be fully, or partially overridden if you want a custom help, version, or usage statements
* **Auto-generated completion scripts at compile time (Bash, Zsh, Fish, and PowerShell)**
  - Even works through many multiple levels of subcommands
  - Works with options which only accept certain values
  - Works with subcommand aliases
* **Flags / Switches** (i.e. bool fields)
  - Both short and long versions supported (i.e. `-f` and `--flag` respectively)
  - Supports combining short versions (i.e. `-fBgoZ` is the same as `-f -B -g -o -Z`)
  - Supports multiple occurrences (i.e. `-vvv` or `-v -v -v`)
* **Positional Arguments** (i.e. those which are based off an index from the program name)
  - Supports multiple values (i.e. `myprog <file>...` such as `myprog file1.txt file2.txt` being two values for the same "file" argument)
  - Supports Specific Value Sets (See below)
  - Can set value parameters (such as the minimum number of values, the maximum number of values, or the exact number of values)
  - Can set custom validations on values to extend the argument parsing capability to truly custom domains
* **Option Arguments** (i.e. those that take values)
  - Both short and long versions supported (i.e. `-o value`, `-ovalue`, `-o=value` and `--option value` or `--option=value` respectively)
  - Supports multiple values (i.e. `-o <val1> -o <val2>` or `-o <val1> <val2>`)
  - Supports delimited values (i.e. `-o=val1,val2,val3`, can also change the delimiter)
  - Supports Specific Value Sets (See below)
  - Supports named values so that the usage/help info appears as `-o <FILE> <INTERFACE>` etc. for when you require specific multiple values
  - Can set value parameters (such as the minimum number of values, the maximum number of values, or the exact number of values)
  - Can set custom validations on values to extend the argument parsing capability to truly custom domains
* **Sub-Commands** (i.e. `git add <file>` where `add` is a sub-command of `git`)
  - Support their own sub-arguments, and sub-sub-commands independent of the parent
  - Get their own auto-generated Help, Version, and Usage independent of parent
* **Support for building CLIs from YAML** - This keeps your Rust source nice and tidy and makes supporting localized translation very simple!
* **Requirement Rules**: Arguments can define the following types of requirement rules
  - Can be required by default
  - Can be required only if certain arguments are present
  - Can require other arguments to be present
  - Can be required only if certain values of other arguments are used
* **Confliction Rules**: Arguments can optionally define the following types of exclusion rules
  - Can be disallowed when certain arguments are present
  - Can disallow use of other arguments when present
* **Groups**: Arguments can be made part of a group
  - Fully compatible with other relational rules (requirements, conflicts, and overrides) which allows things like requiring the use of any arg in a group, or denying the use of an entire group conditionally
* **Specific Value Sets**: Positional or Option Arguments can define a specific set of allowed values (i.e. imagine a `--mode` option which may *only* have one of two values `fast` or `slow` such as `--mode fast` or `--mode slow`)
* **Default Values**
  - Also supports conditional default values (i.e. a default which only applies if specific arguments are used, or specific values of those arguments)
* **Automatic Version from Cargo.toml**: `clap` is fully compatible with Rust's `env!()` macro for automatically setting the version of your application to the version in your Cargo.toml. See [09_auto_version example](examples/09_auto_version.rs) for how to do this (Thanks to [jhelwig](https://github.com/jhelwig) for pointing this out)
* **Typed Values**: You can use several convenience macros provided by `clap` to get typed values (i.e. `i32`, `u8`, etc.) from positional or option arguments so long as the type you request implements `std::str::FromStr` See the [12_typed_values example](examples/12_typed_values.rs). You can also use `clap`s `arg_enum!` macro to create an enum with variants that automatically implement `std::str::FromStr`. See [13a_enum_values_automatic example](examples/13a_enum_values_automatic.rs) for details
* **Suggestions**: Suggests corrections when the user enters a typo. For example, if you defined a `--myoption` argument, and the user mistakenly typed `--moyption` (notice `y` and `o` transposed), they would receive a `Did you mean '--myoption'?` error and exit gracefully. This also works for subcommands and flags. (Thanks to [Byron](https://github.com/Byron) for the implementation) (This feature can optionally be disabled, see 'Optional Dependencies / Features')
* **Colorized Errors (Non Windows OS only)**: Error message are printed in in colored text (this feature can optionally be disabled, see 'Optional Dependencies / Features').
* **Global Arguments**: Arguments can optionally be defined once, and be available to all child subcommands. There values will also be propagated up/down throughout all subcommands.
* **Custom Validations**: You can define a function to use as a validator of argument values. Imagine defining a function to validate IP addresses, or fail parsing upon error. This means your application logic can be solely focused on *using* values.
* **POSIX Compatible Conflicts/Overrides** - In POSIX args can be conflicting, but not fail parsing because whichever arg comes *last* "wins" so to speak. This allows things such as aliases (i.e. `alias ls='ls -l'` but then using `ls -C` in your terminal which ends up passing `ls -l -C` as the final arguments. Since `-l` and `-C` aren't compatible, this effectively runs `ls -C` in `clap` if you choose...`clap` also supports hard conflicts that fail parsing). (Thanks to [Vinatorul](https://github.com/Vinatorul)!)
* Supports the Unix `--` meaning, only positional arguments follow

## Quick Example

The following examples show a quick example of some of the very basic functionality of `clap`. For more advanced usage, such as requirements, conflicts, groups, multiple values and occurrences see the [documentation](https://docs.rs/clap/), [examples/](examples) directory of this repository or the [video tutorials](https://www.youtube.com/playlist?list=PLza5oFLQGTl2Z5T8g1pRkIynR3E0_pc7U).

 **NOTE:** All of these examples are functionally the same, but show different styles in which to use `clap`. These different styles are purely a matter of personal preference.

The first example shows a method using the 'Builder Pattern' which allows more advanced configuration options (not shown in this small example), or even dynamically generating arguments when desired.

```rust
// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
extern crate clap;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("My Super Program")
                          .version("1.0")
                          .author("Kevin K. <kbknapp@gmail.com>")
                          .about("Does awesome things")
                          .arg(Arg::with_name("config")
                               .short("c")
                               .long("config")
                               .value_name("FILE")
                               .help("Sets a custom config file")
                               .takes_value(true))
                          .arg(Arg::with_name("INPUT")
                               .help("Sets the input file to use")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity"))
                          .subcommand(SubCommand::with_name("test")
                                      .about("controls testing features")
                                      .version("1.3")
                                      .author("Someone E. <someone_else@other.com>")
                                      .arg(Arg::with_name("debug")
                                          .short("d")
                                          .help("print debug information verbosely")))
                          .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("test") {
        if matches.is_present("debug") {
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }
    }

    // more program logic goes here...
}
```

One could also optionally declare their CLI in YAML format and keep your Rust source tidy
or support multiple localized translations by having different YAML files for each localization.

First, create the `cli.yml` file to hold your CLI options, but it could be called anything we like:

```yaml
name: myapp
version: "1.0"
author: Kevin K. <kbknapp@gmail.com>
about: Does awesome things
args:
    - config:
        short: c
        long: config
        value_name: FILE
        help: Sets a custom config file
        takes_value: true
    - INPUT:
        help: Sets the input file to use
        required: true
        index: 1
    - verbose:
        short: v
        multiple: true
        help: Sets the level of verbosity
subcommands:
    - test:
        about: controls testing features
        version: "1.3"
        author: Someone E. <someone_else@other.com>
        args:
            - debug:
                short: d
                help: print debug information
```

Since this feature requires additional dependencies that not everyone may want, it is *not* compiled in by default and we need to enable a feature flag in Cargo.toml:

Simply change your `clap = "2.34"` to `clap = {version = "2.34", features = ["yaml"]}`.

Finally we create our `main.rs` file just like we would have with the previous two examples:

```rust
// (Full example with detailed comments in examples/17_yaml.rs)
//
// This example demonstrates clap's building from YAML style of creating arguments which is far
// more clean, but takes a very small performance hit compared to the other two methods.
#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Same as previous examples...
}
```

If you were to compile any of the above programs and run them with the flag `--help` or `-h` (or `help` subcommand, since we defined `test` as a subcommand) the following would be output

```sh
$ myprog --help
My Super Program 1.0
Kevin K. <kbknapp@gmail.com>
Does awesome things

USAGE:
    MyApp [FLAGS] [OPTIONS] <INPUT> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>    Sets a custom config file

ARGS:
    INPUT    The input file to use

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    test    Controls testing features
```

**NOTE:** You could also run `myapp test --help` or `myapp help test` to see the help message for the `test` subcommand.

There are also two other methods to create CLIs. Which style you choose is largely a matter of personal preference. The two other methods are:

* Using [usage strings (examples/01a_quick_example.rs)](examples/01a_quick_example.rs) similar to (but not exact) docopt style usage statements. This is far less verbose than the above methods, but incurs a slight runtime penalty.
* Using [a macro (examples/01c_quick_example.rs)](examples/01c_quick_example.rs) which is like a hybrid of the builder and usage string style. It's less verbose, but doesn't incur the runtime penalty of the usage string style. The downside is that it's harder to debug, and more opaque.

Examples of each method can be found in the [examples/](examples) directory of this repository.

## Try it!

### Pre-Built Test

To try out the pre-built examples, use the following steps:

* Clone the repository `$ git clone https://github.com/clap-rs/clap && cd clap-rs/`
* Compile the example `$ cargo build --example <EXAMPLE>`
* Run the help info `$ ./target/debug/examples/<EXAMPLE> --help`
* Play with the arguments!
* You can also do a onetime run via `$ cargo run --example <EXAMPLE> -- [args to example]`

### BYOB (Build Your Own Binary)

To test out `clap`'s default auto-generated help/version follow these steps:
* Create a new cargo project `$ cargo new fake --bin && cd fake`
* Add `clap` to your `Cargo.toml`

```toml
[dependencies]
clap = "2"
```

* Add the following to your `src/main.rs`

```rust
extern crate clap;
use clap::App;

fn main() {
  App::new("fake").version("v1.0-beta").get_matches();
}
```

* Build your program `$ cargo build --release`
* Run with help or version `$ ./target/release/fake --help` or `$ ./target/release/fake --version`

## Usage

For full usage, add `clap` as a dependency in your `Cargo.toml` () to use from crates.io:

```toml
[dependencies]
clap = "~2.34"
```

(**note**: If you are concerned with supporting a minimum version of Rust that is *older* than the current stable Rust minus 2 stable releases, it's recommended to use the `~major.minor.patch` style versions in your `Cargo.toml` which will only update the patch version automatically. For more information see the [Compatibility Policy](#compatibility-policy))

Then add `extern crate clap;` to your crate root.

Define a list of valid arguments for your program (see the [documentation](https://docs.rs/clap/) or [examples/](examples) directory of this repo)

Then run `cargo build` or `cargo update && cargo build` for your project.

### Optional Dependencies / Features

#### Features enabled by default

* **"suggestions"**: Turns on the `Did you mean '--myoption'?` feature for when users make typos. (builds dependency `strsim`)
* **"color"**: Turns on colored error messages. This feature only works on non-Windows OSs. (builds dependency `ansi-term` only on non-Windows targets)
* **"vec_map"**: Use [`VecMap`](https://crates.io/crates/vec_map) internally instead of a [`BTreeMap`](https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html). This feature provides a _slight_ performance improvement. (builds dependency `vec_map`)

To disable these, add this to your `Cargo.toml`:

```toml
[dependencies.clap]
version = "2.34"
default-features = false
```

You can also selectively enable only the features you'd like to include, by adding:

```toml
[dependencies.clap]
version = "2.34"
default-features = false

# Cherry-pick the features you'd like to use
features = [ "suggestions", "color" ]
```

#### Opt-in features

* **"yaml"**: Enables building CLIs from YAML documents. (builds dependency `yaml-rust`)
* **"unstable"**: Enables unstable `clap` features that may change from release to release
* **"wrap_help"**: Turns on the help text wrapping feature, based on the terminal size. (builds dependency `term-size`)

### Dependencies Tree

The following graphic depicts `clap`s dependency graph (generated using [cargo-graph](https://github.com/kbknapp/cargo-graph)).

 * **Dashed** Line: Optional dependency
 * **Red** Color: **NOT** included by default (must use cargo `features` to enable)
 * **Blue** Color: Dev dependency, only used while developing.

![clap dependencies](clap_dep_graph.png)

### More Information

You can find complete documentation on the [docs.rs](https://docs.rs/clap/) for this project.

You can also find usage examples in the [examples/](examples) directory of this repo.

#### Video Tutorials

There's also the video tutorial series [Argument Parsing with Rust v2](https://www.youtube.com/playlist?list=PLza5oFLQGTl2Z5T8g1pRkIynR3E0_pc7U).

These videos slowly trickle out as I finish them and currently a work in progress.

## How to Contribute

Details on how to contribute can be found in the [CONTRIBUTING.md](.github/CONTRIBUTING.md) file.

### Compatibility Policy

Because `clap` takes SemVer and compatibility seriously, this is the official policy regarding breaking changes and minimum required versions of Rust.

`clap` will pin the minimum required version of Rust to the CI builds. Bumping the minimum version of Rust is considered a minor breaking change, meaning *at a minimum* the minor version of `clap` will be bumped.

In order to keep from being surprised of breaking changes, it is **highly** recommended to use the `~major.minor.patch` style in your `Cargo.toml` only if you wish to target a version of Rust that is *older* than current stable minus two releases:

```toml
[dependencies]
clap = "~2.34"
```

This will cause *only* the patch version to be updated upon a `cargo update` call, and therefore cannot break due to new features, or bumped minimum versions of Rust.

#### Warning about '~' Dependencies

Using `~` can cause issues in certain circumstances.

From @alexcrichton:

Right now Cargo's version resolution is pretty naive, it's just a brute-force search of the solution space, returning the first resolvable graph. This also means that it currently won't terminate until it proves there is not possible resolvable graph. This leads to situations where workspaces with multiple binaries, for example, have two different dependencies such as:

```toml,no_sync

# In one Cargo.toml
[dependencies]
clap = "~2.34.0"

# In another Cargo.toml
[dependencies]
clap = "2.34.0"
```

This is inherently an unresolvable crate graph in Cargo right now. Cargo requires there's only one major version of a crate, and being in the same workspace these two crates must share a version. This is impossible in this location, though, as these version constraints cannot be met.

#### Minimum Version of Rust

`clap` will officially support current stable Rust, minus two releases, but may work with prior releases as well. For example, current stable Rust at the time of this writing is 1.41.0, meaning `clap` is guaranteed to compile with 1.39.0 and beyond.

At the 1.42.0 stable release, `clap` will be guaranteed to compile with 1.40.0 and beyond, etc.

Upon bumping the minimum version of Rust (assuming it's within the stable-2 range), it *must* be clearly annotated in the `CHANGELOG.md`

#### Breaking Changes

`clap` takes a similar policy to Rust and will bump the major version number upon breaking changes with only the following exceptions:

 * The breaking change is to fix a security concern
 * The breaking change is to be fixing a bug (i.e. relying on a bug as a feature)
 * The breaking change is a feature isn't used in the wild, or all users of said feature have given approval *prior* to the change

#### Compatibility with Wasm

A best effort is made to ensure that `clap` will work on projects targeting `wasm32-unknown-unknown`. However there is no dedicated CI build
covering this specific target.

## License

`clap` is licensed under the MIT license. Please read the [LICENSE-MIT](LICENSE-MIT) file in this repository for more information.

## Related Crates

There are several excellent crates which can be used with `clap`, I recommend checking them all out! If you've got a crate that would be a good fit to be used with `clap` open an issue and let me know, I'd love to add it!

* [`structopt`](https://github.com/TeXitoi/structopt) - This crate allows you to define a struct, and build a CLI from it! No more "stringly typed" and it uses `clap` behind the scenes! (*Note*: There is work underway to pull this crate into mainline `clap`).
* [`assert_cli`](https://github.com/assert-rs/assert_cli) - This crate allows you test your CLIs in a very intuitive and functional way!

## Recent Breaking Changes

`clap` follows semantic versioning, so breaking changes should only happen upon major version bumps. The only exception to this rule is breaking changes that happen due to implementation that was deemed to be a bug, security concerns, or it can be reasonably proved to affect no code. For the full details, see [CHANGELOG.md](./CHANGELOG.md).

As of 2.27.0:

* Argument values now take precedence over subcommand names. This only arises by using unrestrained multiple values and subcommands together where the subcommand name can coincide with one of the multiple values. Such as `$ prog <files>... <subcommand>`. The fix is to place restraints on number of values, or disallow the use of `$ prog <prog-args> <subcommand>` structure.

As of 2.0.0 (From 1.x)

* **Fewer lifetimes! Yay!**
 * `App<'a, 'b, 'c, 'd, 'e, 'f>` => `App<'a, 'b>`
 * `Arg<'a, 'b, 'c, 'd, 'e, 'f>` => `Arg<'a, 'b>`
 * `ArgMatches<'a, 'b>` => `ArgMatches<'a>`
* **Simply Renamed**
 * `App::arg_group` => `App::group`
 * `App::arg_groups` => `App::groups`
 * `ArgGroup::add` => `ArgGroup::arg`
 * `ArgGroup::add_all` => `ArgGroup::args`
 * `ClapError` => `Error`
  * struct field `ClapError::error_type` => `Error::kind`
 * `ClapResult` => `Result`
 * `ClapErrorType` => `ErrorKind`
* **Removed Deprecated Functions and Methods**
 * `App::subcommands_negate_reqs`
 * `App::subcommand_required`
 * `App::arg_required_else_help`
 * `App::global_version(bool)`
 * `App::versionless_subcommands`
 * `App::unified_help_messages`
 * `App::wait_on_error`
 * `App::subcommand_required_else_help`
 * `SubCommand::new`
 * `App::error_on_no_subcommand`
 * `Arg::new`
 * `Arg::mutually_excludes`
 * `Arg::mutually_excludes_all`
 * `Arg::mutually_overrides_with`
 * `simple_enum!`
* **Renamed Error Variants**
 * `InvalidUnicode` => `InvalidUtf8`
 * `InvalidArgument` => `UnknownArgument`
* **Usage Parser**
 * Value names can now be specified inline, i.e. `-o, --option <FILE> <FILE2> 'some option which takes two files'`
 * **There is now a priority of order to determine the name** - This is perhaps the biggest breaking change. See the documentation for full details. Prior to this change, the value name took precedence. **Ensure your args are using the proper names (i.e. typically the long or short and NOT the value name) throughout the code**
* `ArgMatches::values_of` returns an `Values` now which implements `Iterator` (should not break any code)
* `crate_version!` returns `&'static str` instead of `String`

### Deprecations

Old method names will be left around for several minor version bumps, or one major version bump.

As of 2.27.0:

* **AppSettings::PropagateGlobalValuesDown:**  this setting deprecated and is no longer required to propagate values down or up
