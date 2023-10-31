// Copyright ⓒ 2015-2016 Kevin B. Knapp and [`clap-rs` contributors](https://github.com/clap-rs/clap/graphs/contributors).
// Licensed under the MIT license
// (see LICENSE or <http://opensource.org/licenses/MIT>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.

//! > **Command Line Argument Parser for Rust**
//!
//! Quick Links:
//! - Derive [tutorial][_derive::_tutorial] and [reference][_derive]
//! - Builder [tutorial][_tutorial] and [reference](index.html)
//! - [Cookbook][_cookbook]
//! - [FAQ][_faq]
//! - [Discussions](https://github.com/clap-rs/clap/discussions)
//!
//! ## Aspirations
//!
//! - Out of the box, users get a polished CLI experience
//!   - Including common argument behavior, help generation, suggested fixes for users, colored output, [shell completions](https://github.com/clap-rs/clap/tree/master/clap_complete), etc
//! - Flexible enough to port your existing CLI interface
//!   - However, we won't necessarily streamline support for each use case
//! - Reasonable parse performance
//! - Resilient maintainership, including
//!   - Willing to break compatibility rather than batching up breaking changes in large releases
//!   - Leverage feature flags to keep to one active branch
//!   - Being under [WG-CLI](https://github.com/rust-cli/team/) to increase the bus factor
//! - We follow semver and will wait about 6-9 months between major breaking changes
//! - We will support the last two minor Rust releases (MSRV, currently 1.56.1)
//!
//! While these aspirations can be at odds with fast build times and low binary
//! size, we will still strive to keep these reasonable for the flexibility you
//! get.  Check out the
//! [argparse-benchmarks](https://github.com/rust-cli/argparse-benchmarks-rs) for
//! CLI parsers optimized for other use cases.
//!
//! ## Example
//!
//! Run
//! ```console
//! $ cargo add clap --features derive
//! ```
//! *(See also [feature flag reference][_features])*
//!
//! Then define your CLI in `main.rs`:
#![cfg_attr(not(feature = "derive"), doc = " ```ignore")]
#![cfg_attr(feature = "derive", doc = " ```no_run")]
#![doc = include_str!("../examples/demo.rs")]
//! ```
//!
//! And try it out:
#![doc = include_str!("../examples/demo.md")]
//!
//! See also the derive [tutorial][_derive::_tutorial] and [reference][_derive]
//!
//! ### Related Projects
//!
//! Augment clap:
//! - [wild](https://crates.io/crates/wild) for supporting wildcards (`*`) on Windows like you do Linux
//! - [argfile](https://crates.io/crates/argfile) for loading additional arguments from a file (aka response files)
//! - [shadow-rs](https://crates.io/crates/shadow-rs) for generating `Command::long_version`
//! - [clap_mangen](https://crates.io/crates/clap_mangen) for generating man page source (roff)
//! - [clap_complete](https://crates.io/crates/clap_complete) for shell completion support
//!
//! CLI Helpers
//! - [clap-verbosity-flag](https://crates.io/crates/clap-verbosity-flag)
//! - [clap-cargo](https://crates.io/crates/clap-cargo)
//! - [concolor-clap](https://crates.io/crates/concolor-clap)
//!
//! Testing
//! - [`trycmd`](https://crates.io/crates/trycmd):  Bulk snapshot testing
//! - [`snapbox`](https://crates.io/crates/snapbox):  Specialized snapshot testing
//! - [`assert_cmd`](https://crates.io/crates/assert_cmd) and [`assert_fs`](https://crates.io/crates/assert_fs): Customized testing
//!
//! Documentation:
//! - [Command-line Apps for Rust](https://rust-cli.github.io/book/index.html) book
//!

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/clap-rs/clap/master/assets/clap.png")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_allocation,
    trivial_numeric_casts,
    clippy::single_char_pattern
)]
#![forbid(unsafe_code)]
// HACK https://github.com/rust-lang/rust-clippy/issues/7290
#![allow(clippy::single_component_path_imports)]
#![allow(clippy::branches_sharing_code)]
// Doesn't allow for debug statements, etc to be unique
#![allow(clippy::if_same_then_else)]

#[cfg(not(feature = "std"))]
compile_error!("`std` feature is currently required to build `clap`");

pub use crate::builder::ArgAction;
pub use crate::builder::Command;
pub use crate::builder::{Arg, ArgGroup};
pub use crate::error::Error;
pub use crate::parser::ArgMatches;
#[cfg(feature = "color")]
pub use crate::util::color::ColorChoice;
#[cfg(not(feature = "color"))]
#[allow(unused_imports)]
pub(crate) use crate::util::color::ColorChoice;

pub use crate::derive::{Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};

#[allow(deprecated)]
pub use crate::builder::App;
pub use crate::builder::{AppFlags, AppSettings, ArgFlags, ArgSettings, PossibleValue, ValueHint};
pub use crate::error::{ErrorKind, Result};
#[allow(deprecated)]
pub use crate::parser::{Indices, OsValues, ValueSource, Values};

#[cfg(feature = "yaml")]
#[doc(hidden)]
#[cfg_attr(
    feature = "deprecated",
    deprecated(
        since = "3.0.0",
        note = "Deprecated in Issue #3087, maybe clap::Parser would fit your use case?"
    )
)]
#[doc(hidden)]
pub use yaml_rust::YamlLoader;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use clap_derive::{self, *};

/// Deprecated, replaced with [`CommandFactory`]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `CommandFactory`")
)]
pub use CommandFactory as IntoApp;
/// Deprecated, replaced with [`Parser`]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.0.0", note = "Replaced with `Parser`")
)]
#[doc(hidden)]
pub use Parser as StructOpt;
/// Deprecated, replaced with [`ValueEnum`]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.2.0", note = "Replaced with `ValueEnum`")
)]
pub use ValueEnum as ArgEnum;

#[cfg(feature = "unstable-doc")]
pub mod _cookbook;
#[cfg(feature = "unstable-doc")]
pub mod _derive;
#[cfg(feature = "unstable-doc")]
pub mod _faq;
#[cfg(feature = "unstable-doc")]
pub mod _features;
#[cfg(feature = "unstable-doc")]
pub mod _tutorial;

#[doc(hidden)]
pub mod __macro_refs {
    #[cfg(any(feature = "derive", feature = "cargo"))]
    #[doc(hidden)]
    pub use once_cell;
}

#[macro_use]
#[allow(missing_docs)]
mod macros;

mod derive;

#[cfg(feature = "regex")]
pub use crate::builder::RegexRef;

pub mod builder;
pub mod error;
pub mod parser;

mod mkeymap;
mod output;
mod util;

const INTERNAL_ERROR_MSG: &str = "Fatal internal error. Please consider filing a bug \
                                  report at https://github.com/clap-rs/clap/issues";
const INVALID_UTF8: &str = "unexpected invalid UTF-8 code point";

/// Deprecated, replaced with [`Command::new`], unless you were looking for [Subcommand]
#[cfg_attr(
    feature = "deprecated",
    deprecated(
        since = "3.0.0",
        note = "Replaced with `Command::new` unless you intended the `Subcommand` trait"
    )
)]
#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct SubCommand {}

#[allow(deprecated)]
impl SubCommand {
    /// Deprecated, replaced with [`Command::new`].
    /// Did you mean Subcommand (lower-case c)?
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `Command::new`")
    )]
    #[doc(hidden)]
    pub fn with_name<'help>(name: &str) -> App<'help> {
        Command::new(name)
    }

    /// Deprecated in [Issue #3087](https://github.com/clap-rs/clap/issues/3087), maybe [`clap::Parser`][crate::Parser] would fit your use case?
    #[cfg(feature = "yaml")]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Deprecated in Issue #3087, maybe clap::Parser would fit your use case?"
        )
    )]
    #[doc(hidden)]
    pub fn from_yaml(yaml: &yaml_rust::Yaml) -> App {
        #![allow(deprecated)]
        Command::from_yaml(yaml)
    }
}
