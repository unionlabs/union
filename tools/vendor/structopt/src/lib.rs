// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! This crate defines the `StructOpt` trait and its custom derive.
//!
//! ## Maintenance
//!
//! As clap v3 is now out, and the structopt features are integrated
//! into (almost as-is), structopt is now in maintenance mode: no new
//! feature will be added.
//!
//! Bugs will be fixed, and documentation improvements will be accepted.
//!
//! ## Features
//!
//! If you want to disable all the `clap` features (colors,
//! suggestions, ..) add `default-features = false` to the `structopt`
//! dependency:
//!
//! ```toml
//! [dependencies]
//! structopt = { version = "0.3", default-features = false }
//! ```
//!
//! Support for [`paw`](https://github.com/rust-cli/paw) (the
//! `Command line argument paw-rser abstraction for main`) is disabled
//! by default, but can be enabled in the `structopt` dependency
//! with the feature `paw`:
//!
//! ```toml
//! [dependencies]
//! structopt = { version = "0.3", features = [ "paw" ] }
//! paw = "1.0"
//! ```
//!
//! # Table of Contents
//!
//! - [How to `derive(StructOpt)`](#how-to-derivestructopt)
//! - [Attributes](#attributes)
//!     - [Raw methods](#raw-methods)
//!     - [Magical methods](#magical-methods)
//! - Arguments
//!     - [Type magic](#type-magic)
//!     - [Specifying argument types](#specifying-argument-types)
//!     - [Default values](#default-values)
//!     - [Help messages](#help-messages)
//!     - [Environment variable fallback](#environment-variable-fallback)
//! - [Skipping fields](#skipping-fields)
//! - [Subcommands](#subcommands)
//!     - [Optional subcommands](#optional-subcommands)
//!     - [External subcommands](#external-subcommands)
//!     - [Flattening subcommands](#flattening-subcommands)
//! - [Flattening](#flattening)
//! - [Custom string parsers](#custom-string-parsers)
//! - [Generics](#generics)
//!
//!
//!
//! ## How to `derive(StructOpt)`
//!
//! First, let's look at the example:
//!
//! ```
//! use std::path::PathBuf;
//! use structopt::StructOpt;
//!
//! #[derive(Debug, StructOpt)]
//! #[structopt(name = "example", about = "An example of StructOpt usage.")]
//! struct Opt {
//!     /// Activate debug mode
//!     // short and long flags (-d, --debug) will be deduced from the field's name
//!     #[structopt(short, long)]
//!     debug: bool,
//!
//!     /// Set speed
//!     // we don't want to name it "speed", need to look smart
//!     #[structopt(short = "v", long = "velocity", default_value = "42")]
//!     speed: f64,
//!
//!     /// Input file
//!     #[structopt(parse(from_os_str))]
//!     input: PathBuf,
//!
//!     /// Output file, stdout if not present
//!     #[structopt(parse(from_os_str))]
//!     output: Option<PathBuf>,
//!
//!     /// Where to write the output: to `stdout` or `file`
//!     #[structopt(short)]
//!     out_type: String,
//!
//!     /// File name: only required when `out-type` is set to `file`
//!     #[structopt(name = "FILE", required_if("out-type", "file"))]
//!     file_name: Option<String>,
//! }
//!
//! fn main() {
//! #   /*
//!     let opt = Opt::from_args();
//! #   */
//! #   let opt = Opt::from_iter(&["binary", "-o", "stdout", "input"]);
//!     println!("{:?}", opt);
//! }
//! ```
//!
//! So `derive(StructOpt)` tells Rust to generate a command line parser,
//! and the various `structopt` attributes are simply
//! used for additional parameters.
//!
//! First, define a struct, whatever its name. This structure
//! corresponds to a `clap::App`, its fields correspond to `clap::Arg`
//! (unless they're [subcommands](#subcommands)),
//! and you can adjust these apps and args by `#[structopt(...)]` [attributes](#attributes).
//!
//! **Note:**
//! _________________
//! Keep in mind that `StructOpt` trait is more than just `from_args` method.
//! It has a number of additional features, including access to underlying
//! `clap::App` via `StructOpt::clap()`. See the
//! [trait's reference documentation](trait.StructOpt.html).
//! _________________
//!
//! ## Attributes
//!
//! You can control the way `structopt` translates your struct into an actual
//! [`clap::App`] invocation via `#[structopt(...)]` attributes.
//!
//! The attributes fall into two categories:
//! - `structopt`'s own [magical methods](#magical-methods).
//!
//!    They are used by `structopt` itself. They come mostly in
//!    `attr = ["whatever"]` form, but some `attr(args...)` also exist.
//!
//! - [`raw` attributes](#raw-methods).
//!
//!     They represent explicit `clap::Arg/App` method calls.
//!     They are what used to be explicit `#[structopt(raw(...))]` attrs in pre-0.3 `structopt`
//!
//! Every `structopt attribute` looks like comma-separated sequence of methods:
//! ```
//! # #[derive(structopt::StructOpt)] struct S {
//! #
//! #[structopt(
//!     short, // method with no arguments - always magical
//!     long = "--long-option", // method with one argument
//!     required_if("out", "file"), // method with one and more args
//!     parse(from_os_str = path::to::parser) // some magical methods have their own syntax
//! )]
//! #
//! # s: () } mod path { pub(crate) mod to { pub(crate) fn parser(_: &std::ffi::OsStr) {} }}
//! ```
//!
//! `#[structopt(...)]` attributes can be placed on top of `struct`, `enum`,
//! `struct` field or `enum` variant. Attributes on top of `struct` or `enum`
//! represent `clap::App` method calls, field or variant attributes correspond
//! to `clap::Arg` method calls.
//!
//! In other words, the `Opt` struct from the example above
//! will be turned into this (*details omitted*):
//!
//! ```
//! # use structopt::clap::{Arg, App};
//! App::new("example")
//!     .version("0.2.0")
//!     .about("An example of StructOpt usage.")
//! .arg(Arg::with_name("debug")
//!     .help("Activate debug mode")
//!     .short("debug")
//!     .long("debug"))
//! .arg(Arg::with_name("speed")
//!     .help("Set speed")
//!     .short("v")
//!     .long("velocity")
//!     .default_value("42"))
//! // and so on
//! # ;
//! ```
//!
//! ## Raw methods
//!
//! They are the reason why `structopt` is so flexible. **Every and each method from
//! `clap::App/Arg` can be used this way!** See the [`clap::App`
//! methods](https://docs.rs/clap/2/clap/struct.App.html) and [`clap::Arg`
//! methods](https://docs.rs/clap/2/clap/struct.Arg.html).
//!
//! ```
//! # #[derive(structopt::StructOpt)] struct S {
//! #
//! #[structopt(
//!     global = true, // name = arg form, neat for one-arg methods
//!     required_if("out", "file") // name(arg1, arg2, ...) form.
//! )]
//! #
//! # s: String }
//! ```
//!
//! The first form can only be used for methods which take only one argument.
//! The second form must be used with multi-arg methods, but can also be used with
//! single-arg methods. These forms are identical otherwise.
//!
//! As long as `method_name` is not one of the magical methods -
//! it will be translated into a mere method call.
//!
//! **Note:**
//! _________________
//!
//! "Raw methods" are direct replacement for pre-0.3 structopt's
//! `#[structopt(raw(...))]` attributes, any time you would have used a `raw()` attribute
//! in 0.2 you should use raw method in 0.3.
//!
//! Unfortunately, old raw attributes collide with `clap::Arg::raw` method. To explicitly
//! warn users of this change we allow `#[structopt(raw())]` only with `true` or `false`
//! literals (this method is supposed to be called only with `true` anyway).
//! __________________
//!
//! ## Magical methods
//!
//! They are the reason why `structopt` is so easy to use and convenient in most cases.
//! Many of them have defaults, some of them get used even if not mentioned.
//!
//! Methods may be used on "top level" (on top of a `struct`, `enum` or `enum` variant)
//! and/or on "field-level" (on top of a `struct` field or *inside* of an enum variant).
//! Top level (non-magical) methods correspond to `App::method` calls, field-level methods
//! are `Arg::method` calls.
//!
//! ```ignore
//! #[structopt(top_level)]
//! struct Foo {
//!     #[structopt(field_level)]
//!     field: u32
//! }
//!
//! #[structopt(top_level)]
//! enum Bar {
//!     #[structopt(top_level)]
//!     Pineapple {
//!         #[structopt(field_level)]
//!         chocolate: String
//!     },
//!
//!     #[structopt(top_level)]
//!     Orange,
//! }
//! ```
//!
//! - `name`: `[name = expr]`
//!   - On top level: `App::new(expr)`.
//!
//!     The binary name displayed in help messages. Defaults to the crate name given by Cargo.
//!
//!   - On field-level: `Arg::with_name(expr)`.
//!
//!     The name for the argument the field stands for, this name appears in help messages.
//!     Defaults to a name, deduced from a field, see also
//!     [`rename_all`](#specifying-argument-types).
//!
//! - `version`: `[version = "version"]`
//!
//!     Usable only on top level: `App::version("version" or env!(CARGO_PKG_VERSION))`.
//!
//!     The version displayed in help messages.
//!     Defaults to the crate version given by Cargo. If `CARGO_PKG_VERSION` is not
//!     set no `.version()` calls will be generated unless requested.
//!
//! - `no_version`: `no_version`
//!
//!     Usable only on top level. Prevents default `App::version` call, i.e
//!     when no `version = "version"` mentioned.
//!
//! - `author`: `author [= "author"]`
//!
//!     Usable only on top level: `App::author("author" or env!(CARGO_PKG_AUTHORS))`.
//!
//!     Author/maintainer of the binary, this name appears in help messages.
//!     Defaults to the crate author given by cargo, but only when `author` explicitly mentioned.
//!
//! - `about`: `about [= "about"]`
//!
//!     Usable only on top level: `App::about("about" or env!(CARGO_PKG_DESCRIPTION))`.
//!
//!     Short description of the binary, appears in help messages.
//!     Defaults to the crate description given by cargo,
//!     but only when `about` explicitly mentioned.
//!
//! - [`short`](#specifying-argument-types): `short [= "short-opt-name"]`
//!
//!     Usable only on field-level.
//!
//! - [`long`](#specifying-argument-types): `long [= "long-opt-name"]`
//!
//!     Usable only on field-level.
//!
//! - [`default_value`](#default-values): `default_value [= "default value"]`
//!
//!     Usable only on field-level.
//!
//! - [`rename_all`](#specifying-argument-types):
//!     [`rename_all = "kebab"/"snake"/"screaming-snake"/"camel"/"pascal"/"verbatim"/"lower"/"upper"]`
//!
//!     Usable both on top level and field level.
//!
//! - [`parse`](#custom-string-parsers): `parse(type [= path::to::parser::fn])`
//!
//!     Usable only on field-level.
//!
//! - [`skip`](#skipping-fields): `skip [= expr]`
//!
//!     Usable only on field-level.
//!
//! - [`flatten`](#flattening): `flatten`
//!
//!     Usable on field-level or single-typed tuple variants.
//!
//! - [`subcommand`](#subcommands): `subcommand`
//!
//!     Usable only on field-level.
//!
//! - [`external_subcommand`](#external-subcommands)
//!
//!     Usable only on enum variants.
//!
//! - [`env`](#environment-variable-fallback): `env [= str_literal]`
//!
//!     Usable only on field-level.
//!
//! - [`rename_all_env`](#auto-deriving-environment-variables):
//!     [`rename_all_env = "kebab"/"snake"/"screaming-snake"/"camel"/"pascal"/"verbatim"/"lower"/"upper"]`
//!
//!     Usable both on top level and field level.
//!
//! - [`verbatim_doc_comment`](#doc-comment-preprocessing-and-structoptverbatim_doc_comment):
//!     `verbatim_doc_comment`
//!
//!     Usable both on top level and field level.
//!
//! ## Type magic
//!
//! One of major things that makes `structopt` so awesome is its type magic.
//! Do you want optional positional argument? Use `Option<T>`! Or perhaps optional argument
//! that optionally takes value (`[--opt=[val]]`)? Use `Option<Option<T>>`!
//!
//! Here is the table of types and `clap` methods they correspond to:
//!
//! Type                         | Effect                                            | Added method call to `clap::Arg`
//! -----------------------------|---------------------------------------------------|--------------------------------------
//! `bool`                       | `true` if the flag is present                     | `.takes_value(false).multiple(false)`
//! `Option<T: FromStr>`         | optional positional argument or option            | `.takes_value(true).multiple(false)`
//! `Option<Option<T: FromStr>>` | optional option with optional value               | `.takes_value(true).multiple(false).min_values(0).max_values(1)`
//! `Vec<T: FromStr>`            | list of options or the other positional arguments | `.takes_value(true).multiple(true)`
//! `Option<Vec<T: FromStr>`     | optional list of options                          | `.takes_values(true).multiple(true).min_values(0)`
//! `T: FromStr`                 | required option or positional argument            | `.takes_value(true).multiple(false).required(!has_default)`
//!
//! The `FromStr` trait is used to convert the argument to the given
//! type, and the `Arg::validator` method is set to a method using
//! `to_string()` (`FromStr::Err` must implement `std::fmt::Display`).
//! If you would like to use a custom string parser other than `FromStr`, see
//! the [same titled section](#custom-string-parsers) below.
//!
//! **Important:**
//! _________________
//! Pay attention that *only literal occurrence* of this types is special, for example
//! `Option<T>` is special while `::std::option::Option<T>` is not.
//!
//! If you need to avoid special casing you can make a `type` alias and
//! use it in place of the said type.
//! _________________
//!
//! **Note:**
//! _________________
//! `bool` cannot be used as positional argument unless you provide an explicit parser.
//! If you need a positional bool, for example to parse `true` or `false`, you must
//! annotate the field with explicit [`#[structopt(parse(...))]`](#custom-string-parsers).
//! _________________
//!
//! Thus, the `speed` argument is generated as:
//!
//! ```
//! # fn parse_validator<T>(_: String) -> Result<(), String> { unimplemented!() }
//! clap::Arg::with_name("speed")
//!     .takes_value(true)
//!     .multiple(false)
//!     .required(false)
//!     .validator(parse_validator::<f64>)
//!     .short("v")
//!     .long("velocity")
//!     .help("Set speed")
//!     .default_value("42");
//! ```
//!
//! ## Specifying argument types
//!
//! There are three types of arguments that can be supplied to each
//! (sub-)command:
//!
//!  - short (e.g. `-h`),
//!  - long (e.g. `--help`)
//!  - and positional.
//!
//! Like clap, structopt defaults to creating positional arguments.
//!
//! If you want to generate a long argument you can specify either
//! `long = $NAME`, or just `long` to get a long flag generated using
//! the field name.  The generated casing style can be modified using
//! the `rename_all` attribute. See the `rename_all` example for more.
//!
//! For short arguments, `short` will use the first letter of the
//! field name by default, but just like the long option it's also
//! possible to use a custom letter through `short = $LETTER`.
//!
//! If an argument is renamed using `name = $NAME` any following call to
//! `short` or `long` will use the new name.
//!
//! **Attention**: If these arguments are used without an explicit name
//! the resulting flag is going to be renamed using `kebab-case` if the
//! `rename_all` attribute was not specified previously. The same is true
//! for subcommands with implicit naming through the related data structure.
//!
//! ```
//! use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! #[structopt(rename_all = "kebab-case")]
//! struct Opt {
//!     /// This option can be specified with something like `--foo-option
//!     /// value` or `--foo-option=value`
//!     #[structopt(long)]
//!     foo_option: String,
//!
//!     /// This option can be specified with something like `-b value` (but
//!     /// not `--bar-option value`).
//!     #[structopt(short)]
//!     bar_option: String,
//!
//!     /// This option can be specified either `--baz value` or `-z value`.
//!     #[structopt(short = "z", long = "baz")]
//!     baz_option: String,
//!
//!     /// This option can be specified either by `--custom value` or
//!     /// `-c value`.
//!     #[structopt(name = "custom", long, short)]
//!     custom_option: String,
//!
//!     /// This option is positional, meaning it is the first unadorned string
//!     /// you provide (multiple others could follow).
//!     my_positional: String,
//!
//!     /// This option is skipped and will be filled with the default value
//!     /// for its type (in this case 0).
//!     #[structopt(skip)]
//!     skipped: u32,
//! }
//!
//! # Opt::from_iter(
//! #    &["test", "--foo-option", "", "-b", "", "--baz", "", "--custom", "", "positional"]);
//! ```
//!
//! ## Default values
//!
//! In clap, default values for options can be specified via [`Arg::default_value`].
//!
//! Of course, you can use as a raw method:
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! struct Opt {
//!     #[structopt(default_value = "", long)]
//!     prefix: String,
//! }
//! ```
//!
//! This is quite mundane and error-prone to type the `"..."` default by yourself,
//! especially when the Rust ecosystem uses the [`Default`] trait for that.
//! It would be wonderful to have `structopt` to take the `Default_default` and fill it
//! for you. And yes, `structopt` can do that.
//!
//! Unfortunately, `default_value` takes `&str` but `Default::default`
//! gives us some `Self` value. We need to map `Self` to `&str` somehow.
//!
//! `structopt` solves this problem via [`ToString`] trait.
//!
//! To be able to use auto-default the type must implement *both* `Default` and `ToString`:
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! struct Opt {
//!     // just leave the `= "..."` part and structopt will figure it for you
//!     #[structopt(default_value, long)]
//!     prefix: String, // `String` implements both `Default` and `ToString`
//! }
//! ```
//!
//! [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
//! [`ToString`]: https://doc.rust-lang.org/std/string/trait.ToString.html
//! [`Arg::default_value`]: https://docs.rs/clap/2.33.0/clap/struct.Arg.html#method.default_value
//!
//!
//! ## Help messages
//!
//! In clap, help messages for the whole binary can be specified
//! via [`App::about`] and [`App::long_about`] while help messages
//! for individual arguments can be specified via [`Arg::help`] and [`Arg::long_help`]".
//!
//! `long_*` variants are used when user calls the program with
//! `--help` and "short" variants are used with `-h` flag. In `structopt`,
//! you can use them via [raw methods](#raw-methods), for example:
//!
//! ```
//! # use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! #[structopt(about = "I am a program and I work, just pass `-h`")]
//! struct Foo {
//!     #[structopt(short, help = "Pass `-h` and you'll see me!")]
//!     bar: String,
//! }
//! ```
//!
//! For convenience, doc comments can be used instead of raw methods
//! (this example works exactly like the one above):
//!
//! ```
//! # use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! /// I am a program and I work, just pass `-h`
//! struct Foo {
//!     /// Pass `-h` and you'll see me!
//!     bar: String,
//! }
//! ```
//!
//! Doc comments on [top-level](#magical-methods) will be turned into
//! `App::about/long_about` call (see below), doc comments on field-level are
//! `Arg::help/long_help` calls.
//!
//! **Important:**
//! _________________
//!
//! Raw methods have priority over doc comments!
//!
//! **Top level doc comments always generate `App::about/long_about` calls!**
//! If you really want to use the `App::help/long_help` methods (you likely don't),
//! use a raw method to override the `App::about` call generated from the doc comment.
//! __________________
//!
//! ### `long_help` and `--help`
//!
//! A message passed to [`App::long_about`] or [`Arg::long_help`] will be displayed whenever
//! your program is called with `--help` instead of `-h`. Of course, you can
//! use them via raw methods as described [above](#help-messages).
//!
//! The more convenient way is to use a so-called "long" doc comment:
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! /// Hi there, I'm Robo!
//! ///
//! /// I like beeping, stumbling, eating your electricity,
//! /// and making records of you singing in a shower.
//! /// Pay up, or I'll upload it to youtube!
//! struct Robo {
//!     /// Call my brother SkyNet.
//!     ///
//!     /// I am artificial superintelligence. I won't rest
//!     /// until I'll have destroyed humanity. Enjoy your
//!     /// pathetic existence, you mere mortals.
//!     #[structopt(long)]
//!     kill_all_humans: bool,
//! }
//! ```
//!
//! A long doc comment consists of three parts:
//! * Short summary
//! * A blank line (whitespace only)
//! * Detailed description, all the rest
//!
//! In other words, "long" doc comment consists of two or more paragraphs,
//! with the first being a summary and the rest being the detailed description.
//!
//! **A long comment will result in two method calls**, `help(<summary>)` and
//! `long_help(<whole comment>)`, so clap will display the summary with `-h`
//! and the whole help message on `--help` (see below).
//!
//! So, the example above will be turned into this (details omitted):
//! ```
//! clap::App::new("<name>")
//!     .about("Hi there, I'm Robo!")
//!     .long_about("Hi there, I'm Robo!\n\n\
//!                  I like beeping, stumbling, eating your electricity,\
//!                  and making records of you singing in a shower.\
//!                  Pay up or I'll upload it to youtube!")
//! // args...
//! # ;
//! ```
//!
//! ### `-h` vs `--help` (A.K.A `help()` vs `long_help()`)
//!
//! The `-h` flag is not the same as `--help`.
//!
//! -h corresponds to `Arg::help/App::about` and requests short "summary" messages
//! while --help corresponds to `Arg::long_help/App::long_about` and requests more
//! detailed, descriptive messages.
//!
//! It is entirely up to `clap` what happens if you used only one of
//! [`Arg::help`]/[`Arg::long_help`], see `clap`'s documentation for these methods.
//!
//! As of clap v2.33, if only a short message ([`Arg::help`]) or only
//! a long ([`Arg::long_help`]) message is provided, clap will use it
//! for both -h and --help. The same logic applies to `about/long_about`.
//!
//! ### Doc comment preprocessing and `#[structopt(verbatim_doc_comment)]`
//!
//! `structopt` applies some preprocessing to doc comments to ease the most common uses:
//!
//! * Strip leading and trailing whitespace from every line, if present.
//!
//! * Strip leading and trailing blank lines, if present.
//!
//! * Interpret each group of non-empty lines as a word-wrapped paragraph.
//!
//!   We replace newlines within paragraphs with spaces to allow the output
//!   to be re-wrapped to the terminal width.
//!
//! * Strip any excess blank lines so that there is exactly one per paragraph break.
//!
//! * If the first paragraph ends in exactly one period,
//!   remove the trailing period (i.e. strip trailing periods but not trailing ellipses).
//!
//! Sometimes you don't want this preprocessing to apply, for example the comment contains
//! some ASCII art or markdown tables, you would need to preserve LFs along with
//! blank lines and the leading/trailing whitespace. You can ask `structopt` to preserve them
//! via `#[structopt(verbatim_doc_comment)]` attribute.
//!
//! **This attribute must be applied to each field separately**, there's no global switch.
//!
//! **Important:**
//! ______________
//! Keep in mind that `structopt` will *still* remove one leading space from each
//! line, even if this attribute is present, to allow for a space between
//! `///` and the content.
//!
//! Also, `structopt` will *still* remove leading and trailing blank lines so
//! these formats are equivalent:
//!
//! ```
//! /** This is a doc comment
//!
//! Hello! */
//!
//! /**
//! This is a doc comment
//!
//! Hello!
//! */
//!
//! /// This is a doc comment
//! ///
//! /// Hello!
//! #
//! # mod m {}
//! ```
//! ______________
//!
//! [`App::about`]:      https://docs.rs/clap/2/clap/struct.App.html#method.about
//! [`App::long_about`]: https://docs.rs/clap/2/clap/struct.App.html#method.long_about
//! [`Arg::help`]:       https://docs.rs/clap/2/clap/struct.Arg.html#method.help
//! [`Arg::long_help`]:  https://docs.rs/clap/2/clap/struct.Arg.html#method.long_help
//!
//! ## Environment variable fallback
//!
//! It is possible to specify an environment variable fallback option for an arguments
//! so that its value is taken from the specified environment variable if not
//! given through the command-line:
//!
//! ```
//! # use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! struct Foo {
//!     #[structopt(short, long, env = "PARAMETER_VALUE")]
//!     parameter_value: String,
//! }
//! ```
//!
//! By default, values from the environment are shown in the help output (i.e. when invoking
//! `--help`):
//!
//! ```shell
//! $ cargo run -- --help
//! ...
//! OPTIONS:
//!   -p, --parameter-value <parameter-value>     [env: PARAMETER_VALUE=env_value]
//! ```
//!
//! In some cases this may be undesirable, for example when being used for passing
//! credentials or secret tokens. In those cases you can use `hide_env_values` to avoid
//! having structopt emit the actual secret values:
//! ```
//! # use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! struct Foo {
//!     #[structopt(long = "secret", env = "SECRET_VALUE", hide_env_values = true)]
//!     secret_value: String,
//! }
//! ```
//!
//! ### Auto-deriving environment variables
//!
//! Environment variables tend to be called after the corresponding `struct`'s field,
//! as in example above. The field is `secret_value` and the env var is "SECRET_VALUE";
//! the name is the same, except casing is different.
//!
//! It's pretty tedious and error-prone to type the same name twice,
//! so you can ask `structopt` to do that for you.
//!
//! ```
//! # use structopt::StructOpt;
//!
//! #[derive(StructOpt)]
//! struct Foo {
//!     #[structopt(long = "secret", env)]
//!     secret_value: String,
//! }
//! ```
//!
//! It works just like `#[structopt(short/long)]`: if `env` is not set to some concrete
//! value the value will be derived from the field's name. This is controlled by
//! `#[structopt(rename_all_env)]`.
//!
//! `rename_all_env` works exactly as `rename_all` (including overriding)
//! except default casing is `SCREAMING_SNAKE_CASE` instead of `kebab-case`.
//!
//! ## Skipping fields
//!
//! Sometimes you may want to add a field to your `Opt` struct that is not
//! a command line option and `clap` should know nothing about it. You can ask
//! `structopt` to skip the field entirely via `#[structopt(skip = value)]`
//! (`value` must implement `Into<FieldType>`)
//! or `#[structopt(skip)]` if you want assign the field with `Default::default()`
//! (obviously, the field's type must implement `Default`).
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! pub struct Opt {
//!     #[structopt(long, short)]
//!     number: u32,
//!
//!     // these fields are to be assigned with Default::default()
//!
//!     #[structopt(skip)]
//!     k: String,
//!     #[structopt(skip)]
//!     v: Vec<u32>,
//!
//!     // these fields get set explicitly
//!
//!     #[structopt(skip = vec![1, 2, 3])]
//!     k2: Vec<u32>,
//!     #[structopt(skip = "cake")] // &str implements Into<String>
//!     v2: String,
//! }
//! ```
//!
//! ## Subcommands
//!
//! Some applications, especially large ones, split their functionality
//! through the use of "subcommands". Each of these act somewhat like a separate
//! command, but is part of the larger group.
//! One example is `git`, which has subcommands such as `add`, `commit`,
//! and `clone`, to mention just a few.
//!
//! `clap` has this functionality, and `structopt` supports it through enums:
//!
//! ```
//! # use structopt::StructOpt;
//!
//! # use std::path::PathBuf;
//! #[derive(StructOpt)]
//! #[structopt(about = "the stupid content tracker")]
//! enum Git {
//!     Add {
//!         #[structopt(short)]
//!         interactive: bool,
//!         #[structopt(short)]
//!         patch: bool,
//!         #[structopt(parse(from_os_str))]
//!         files: Vec<PathBuf>,
//!     },
//!     Fetch {
//!         #[structopt(long)]
//!         dry_run: bool,
//!         #[structopt(long)]
//!         all: bool,
//!         repository: Option<String>,
//!     },
//!     Commit {
//!         #[structopt(short)]
//!         message: Option<String>,
//!         #[structopt(short)]
//!         all: bool,
//!     },
//! }
//! ```
//!
//! Using `derive(StructOpt)` on an enum instead of a struct will produce
//! a `clap::App` that only takes subcommands. So `git add`, `git fetch`,
//! and `git commit` would be commands allowed for the above example.
//!
//! `structopt` also provides support for applications where certain flags
//! need to apply to all subcommands, as well as nested subcommands:
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! struct MakeCookie {
//!     #[structopt(name = "supervisor", default_value = "Puck", long = "supervisor")]
//!     supervising_faerie: String,
//!     /// The faerie tree this cookie is being made in.
//!     tree: Option<String>,
//!     #[structopt(subcommand)] // Note that we mark a field as a subcommand
//!     cmd: Command,
//! }
//!
//! #[derive(StructOpt)]
//! enum Command {
//!     /// Pound acorns into flour for cookie dough.
//!     Pound {
//!         acorns: u32,
//!     },
//!     /// Add magical sparkles -- the secret ingredient!
//!     Sparkle {
//!         #[structopt(short, parse(from_occurrences))]
//!         magicality: u64,
//!         #[structopt(short)]
//!         color: String,
//!     },
//!     Finish(Finish),
//! }
//!
//! // Subcommand can also be externalized by using a 1-uple enum variant
//! #[derive(StructOpt)]
//! struct Finish {
//!     #[structopt(short)]
//!     time: u32,
//!     #[structopt(subcommand)] // Note that we mark a field as a subcommand
//!     finish_type: FinishType,
//! }
//!
//! // subsubcommand!
//! #[derive(StructOpt)]
//! enum FinishType {
//!     Glaze {
//!         applications: u32,
//!     },
//!     Powder {
//!         flavor: String,
//!         dips: u32,
//!     }
//! }
//! ```
//!
//! Marking a field with `structopt(subcommand)` will add the subcommands of the
//! designated enum to the current `clap::App`. The designated enum *must* also
//! be derived `StructOpt`. So the above example would take the following
//! commands:
//!
//! + `make-cookie pound 50`
//! + `make-cookie sparkle -mmm --color "green"`
//! + `make-cookie finish 130 glaze 3`
//!
//! ### Optional subcommands
//!
//! Subcommands may be optional:
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! struct Foo {
//!     file: String,
//!     #[structopt(subcommand)]
//!     cmd: Option<Command>,
//! }
//!
//! #[derive(StructOpt)]
//! enum Command {
//!     Bar,
//!     Baz,
//!     Quux,
//! }
//! ```
//!
//! ### External subcommands
//!
//! Sometimes you want to support not only the set of well-known subcommands
//! but you also want to allow other, user-driven subcommands. `clap` supports
//! this via [`AppSettings::AllowExternalSubcommands`].
//!
//! `structopt` provides it's own dedicated syntax for that:
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(Debug, PartialEq, StructOpt)]
//! struct Opt {
//!     #[structopt(subcommand)]
//!     sub: Subcommands,
//! }
//!
//! #[derive(Debug, PartialEq, StructOpt)]
//! enum Subcommands {
//!     // normal subcommand
//!     Add,
//!
//!     // `external_subcommand` tells structopt to put
//!     // all the extra arguments into this Vec
//!     #[structopt(external_subcommand)]
//!     Other(Vec<String>),
//! }
//!
//! // normal subcommand
//! assert_eq!(
//!     Opt::from_iter(&["test", "add"]),
//!     Opt {
//!         sub: Subcommands::Add
//!     }
//! );
//!
//! assert_eq!(
//!     Opt::from_iter(&["test", "git", "status"]),
//!     Opt {
//!         sub: Subcommands::Other(vec!["git".into(), "status".into()])
//!     }
//! );
//!
//! // Please note that if you'd wanted to allow "no subcommands at all" case
//! // you should have used `sub: Option<Subcommands>` above
//! assert!(Opt::from_iter_safe(&["test"]).is_err());
//! ```
//!
//! In other words, you just add an extra tuple variant marked with
//! `#[structopt(subcommand)]`, and its type must be either
//! `Vec<String>` or `Vec<OsString>`. `structopt` will detect `String` in this context
//! and use appropriate `clap` API.
//!
//! [`AppSettings::AllowExternalSubcommands`]: https://docs.rs/clap/2.32.0/clap/enum.AppSettings.html#variant.AllowExternalSubcommands
//!
//! ### Flattening subcommands
//!
//! It is also possible to combine multiple enums of subcommands into one.
//! All the subcommands will be on the same level.
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! enum BaseCli {
//!     Ghost10 {
//!         arg1: i32,
//!     }
//! }
//!
//! #[derive(StructOpt)]
//! enum Opt {
//!     #[structopt(flatten)]
//!     BaseCli(BaseCli),
//!     Dex {
//!         arg2: i32,
//!     },
//! }
//! ```
//!
//! ```shell
//! cli ghost10 42
//! cli dex 42
//! ```
//!
//! ## Flattening
//!
//! It can sometimes be useful to group related arguments in a substruct,
//! while keeping the command-line interface flat. In these cases you can mark
//! a field as `flatten` and give it another type that derives `StructOpt`:
//!
//! ```
//! # use structopt::StructOpt;
//! #[derive(StructOpt)]
//! struct Cmdline {
//!     /// switch on verbosity
//!     #[structopt(short)]
//!     verbose: bool,
//!     #[structopt(flatten)]
//!     daemon_opts: DaemonOpts,
//! }
//!
//! #[derive(StructOpt)]
//! struct DaemonOpts {
//!     /// daemon user
//!     #[structopt(short)]
//!     user: String,
//!     /// daemon group
//!     #[structopt(short)]
//!     group: String,
//! }
//! ```
//!
//! In this example, the derived `Cmdline` parser will support the options `-v`,
//! `-u` and `-g`.
//!
//! This feature also makes it possible to define a `StructOpt` struct in a
//! library, parse the corresponding arguments in the main argument parser, and
//! pass off this struct to a handler provided by that library.
//!
//! ## Custom string parsers
//!
//! If the field type does not have a `FromStr` implementation, or you would
//! like to provide a custom parsing scheme other than `FromStr`, you may
//! provide a custom string parser using `parse(...)` like this:
//!
//! ```
//! # use structopt::StructOpt;
//! use std::num::ParseIntError;
//! use std::path::PathBuf;
//!
//! fn parse_hex(src: &str) -> Result<u32, ParseIntError> {
//!     u32::from_str_radix(src, 16)
//! }
//!
//! #[derive(StructOpt)]
//! struct HexReader {
//!     #[structopt(short, parse(try_from_str = parse_hex))]
//!     number: u32,
//!     #[structopt(short, parse(from_os_str))]
//!     output: PathBuf,
//! }
//! ```
//!
//! There are five kinds of custom parsers:
//!
//! | Kind              | Signature                             | Default                         |
//! |-------------------|---------------------------------------|---------------------------------|
//! | `from_str`        | `fn(&str) -> T`                       | `::std::convert::From::from`    |
//! | `try_from_str`    | `fn(&str) -> Result<T, E>`            | `::std::str::FromStr::from_str` |
//! | `from_os_str`     | `fn(&OsStr) -> T`                     | `::std::convert::From::from`    |
//! | `try_from_os_str` | `fn(&OsStr) -> Result<T, OsString>`   | (no default function)           |
//! | `from_occurrences`| `fn(u64) -> T`                        | `value as T`                    |
//! | `from_flag`       | `fn(bool) -> T`                       | `::std::convert::From::from`    |
//!
//! The `from_occurrences` parser is special. Using `parse(from_occurrences)`
//! results in the _number of flags occurrences_ being stored in the relevant
//! field or being passed to the supplied function. In other words, it converts
//! something like `-vvv` to `3`. This is equivalent to
//! `.takes_value(false).multiple(true)`. Note that the default parser can only
//! be used with fields of integer types (`u8`, `usize`, `i64`, etc.).
//!
//! The `from_flag` parser is also special. Using `parse(from_flag)` or
//! `parse(from_flag = some_func)` will result in the field being treated as a
//! flag even if it does not have type `bool`.
//!
//! When supplying a custom string parser, `bool` will not be treated specially:
//!
//! Type        | Effect            | Added method call to `clap::Arg`
//! ------------|-------------------|--------------------------------------
//! `Option<T>` | optional argument | `.takes_value(true).multiple(false)`
//! `Vec<T>`    | list of arguments | `.takes_value(true).multiple(true)`
//! `T`         | required argument | `.takes_value(true).multiple(false).required(!has_default)`
//!
//! In the `try_from_*` variants, the function will run twice on valid input:
//! once to validate, and once to parse. Hence, make sure the function is
//! side-effect-free.
//!
//! ## Generics
//!
//! Generic structs and enums can be used. They require explicit trait bounds
//! on any generic types that will be used by the `StructOpt` derive macro. In
//! some cases, associated types will require additional bounds. See the usage
//! of `FromStr` below for an example of this.
//!
//! ```
//! # use structopt::StructOpt;
//! use std::{fmt, str::FromStr};
//!
//! // a struct with single custom argument
//! #[derive(StructOpt)]
//! struct GenericArgs<T: FromStr> where <T as FromStr>::Err: fmt::Display + fmt::Debug {
//!     generic_arg_1: String,
//!     generic_arg_2: String,
//!     custom_arg_1: T,
//! }
//! ```
//!
//! or
//!
//! ```
//! # use structopt::StructOpt;
//! // a struct with multiple custom arguments in a substructure
//! #[derive(StructOpt)]
//! struct GenericArgs<T: StructOpt> {
//!     generic_arg_1: String,
//!     generic_arg_2: String,
//!     #[structopt(flatten)]
//!     custom_args: T,
//! }
//! ```

// those mains are for a reason
#![allow(clippy::needless_doctest_main)]

#[doc(hidden)]
pub use structopt_derive::*;

use std::ffi::OsString;

/// Re-exports
pub use clap;
#[cfg(feature = "paw")]
pub use paw_dep as paw;

/// **This is NOT PUBLIC API**.
#[doc(hidden)]
pub use lazy_static;

/// A struct that is converted from command line arguments.
pub trait StructOpt {
    /// Returns [`clap::App`] corresponding to the struct.
    fn clap<'a, 'b>() -> clap::App<'a, 'b>;

    /// Builds the struct from [`clap::ArgMatches`]. It's guaranteed to succeed
    /// if `matches` originates from an `App` generated by [`StructOpt::clap`] called on
    /// the same type, otherwise it must panic.
    fn from_clap(matches: &clap::ArgMatches<'_>) -> Self;

    /// Builds the struct from the command line arguments ([`std::env::args_os`]).
    /// Calls [`clap::Error::exit`] on failure, printing the error message and aborting the program.
    fn from_args() -> Self
    where
        Self: Sized,
    {
        Self::from_clap(&Self::clap().get_matches())
    }

    /// Builds the struct from the command line arguments ([`std::env::args_os`]).
    /// Unlike [`StructOpt::from_args`], returns [`clap::Error`] on failure instead of aborting the program,
    /// so calling [`.exit`][clap::Error::exit] is up to you.
    fn from_args_safe() -> Result<Self, clap::Error>
    where
        Self: Sized,
    {
        Self::clap()
            .get_matches_safe()
            .map(|matches| Self::from_clap(&matches))
    }

    /// Gets the struct from any iterator such as a `Vec` of your making.
    /// Print the error message and quit the program in case of failure.
    ///
    /// **NOTE**: The first argument will be parsed as the binary name unless
    /// [`clap::AppSettings::NoBinaryName`] has been used.
    fn from_iter<I>(iter: I) -> Self
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        Self::from_clap(&Self::clap().get_matches_from(iter))
    }

    /// Gets the struct from any iterator such as a `Vec` of your making.
    ///
    /// Returns a [`clap::Error`] in case of failure. This does *not* exit in the
    /// case of `--help` or `--version`, to achieve the same behavior as
    /// [`from_iter()`][StructOpt::from_iter] you must call [`.exit()`][clap::Error::exit] on the error value.
    ///
    /// **NOTE**: The first argument will be parsed as the binary name unless
    /// [`clap::AppSettings::NoBinaryName`] has been used.
    fn from_iter_safe<I>(iter: I) -> Result<Self, clap::Error>
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        Ok(Self::from_clap(&Self::clap().get_matches_from_safe(iter)?))
    }
}

/// This trait is NOT API. **SUBJECT TO CHANGE WITHOUT NOTICE!**.
#[doc(hidden)]
pub trait StructOptInternal: StructOpt {
    fn augment_clap<'a, 'b>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
        app
    }

    fn is_subcommand() -> bool {
        false
    }

    fn from_subcommand<'a, 'b>(_sub: (&'b str, Option<&'b clap::ArgMatches<'a>>)) -> Option<Self>
    where
        Self: std::marker::Sized,
    {
        None
    }
}

impl<T: StructOpt> StructOpt for Box<T> {
    fn clap<'a, 'b>() -> clap::App<'a, 'b> {
        <T as StructOpt>::clap()
    }

    fn from_clap(matches: &clap::ArgMatches<'_>) -> Self {
        Box::new(<T as StructOpt>::from_clap(matches))
    }
}

impl<T: StructOptInternal> StructOptInternal for Box<T> {
    #[doc(hidden)]
    fn is_subcommand() -> bool {
        <T as StructOptInternal>::is_subcommand()
    }

    #[doc(hidden)]
    fn from_subcommand<'a, 'b>(sub: (&'b str, Option<&'b clap::ArgMatches<'a>>)) -> Option<Self> {
        <T as StructOptInternal>::from_subcommand(sub).map(Box::new)
    }

    #[doc(hidden)]
    fn augment_clap<'a, 'b>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
        <T as StructOptInternal>::augment_clap(app)
    }
}
