// Copyright ⓒ 2015-2018 Kevin B. Knapp
//
// `clap_complete` is distributed under the terms of both the MIT license and the Apache License
// (Version 2.0).
// See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files in this repository
// for more information.

#![doc(html_logo_url = "https://raw.githubusercontent.com/clap-rs/clap/master/assets/clap.png")]
#![doc = include_str!("../README.md")]
#![warn(missing_docs, trivial_casts, unused_allocation, trivial_numeric_casts)]
#![forbid(unsafe_code)]
#![allow(clippy::needless_doctest_main)]

//! ## Quick Start
//!
//! - For generating at compile-time, see [`generate_to`]
//! - For generating at runtime, see [`generate`]
//!
//! [`Shell`] is a convenience `enum` for an argument value type that implements `Generator`
//! for each natively-supported shell type.
//!
//! ## Example
//!
//! ```rust,no_run
//! use clap::{Command, Arg, ValueHint, value_parser, ArgAction};
//! use clap_complete::{generate, Generator, Shell};
//! use std::io;
//!
//! fn build_cli() -> Command {
//!     Command::new("example")
//!          .arg(Arg::new("file")
//!              .help("some input file")
//!                 .value_hint(ValueHint::AnyPath),
//!         )
//!        .arg(
//!            Arg::new("generator")
//!                .long("generate")
//!                .action(ArgAction::Set)
//!                .value_parser(value_parser!(Shell)),
//!        )
//! }
//!
//! fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
//!     generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
//! }
//!
//! fn main() {
//!     let matches = build_cli().get_matches();
//!
//!     if let Some(generator) = matches.get_one::<Shell>("generator").copied() {
//!         let mut cmd = build_cli();
//!         eprintln!("Generating completion file for {generator}...");
//!         print_completions(generator, &mut cmd);
//!     }
//! }
//! ```

const INTERNAL_ERROR_MSG: &str = "Fatal internal error. Please consider filing a bug \
                                  report at https://github.com/clap-rs/clap/issues";

#[macro_use]
#[allow(missing_docs)]
mod macros;

pub mod generator;
pub mod shells;

pub use generator::generate;
pub use generator::generate_to;
pub use generator::Generator;
pub use shells::Shell;

#[cfg(feature = "unstable-dynamic")]
pub mod dynamic;
