// Copyright ⓒ 2015-2016 Kevin B. Knapp and [`clap-rs` contributors](https://github.com/clap-rs/clap/graphs/contributors).
// Licensed under the MIT license
// (see LICENSE or <http://opensource.org/licenses/MIT>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
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
// Wanting consistency in our calls
#![allow(clippy::write_with_newline)]
// Gets in the way of logging
#![allow(clippy::let_and_return)]
// HACK https://github.com/rust-lang/rust-clippy/issues/7290
#![allow(clippy::single_component_path_imports)]
#![allow(clippy::branches_sharing_code)]
// Doesn't allow for debug statements, etc to be unique
#![allow(clippy::if_same_then_else)]
// Breaks up parallelism that clarifies intent
#![allow(clippy::collapsible_else_if)]

#[cfg(not(feature = "std"))]
compile_error!("`std` feature is currently required to build `clap`");

pub use crate::builder::ArgAction;
pub use crate::builder::Command;
pub use crate::builder::ValueHint;
pub use crate::builder::{Arg, ArgGroup};
pub use crate::parser::ArgMatches;
pub use crate::util::color::ColorChoice;
pub use crate::util::Id;

/// Command Line Argument Parser Error
///
/// See [`Command::error`] to create an error.
///
/// [`Command::error`]: crate::Command::error
pub type Error = crate::error::Error<crate::error::DefaultFormatter>;

pub use crate::derive::{Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};

#[doc(hidden)]
pub mod __macro_refs {
    #[cfg(feature = "cargo")]
    #[doc(hidden)]
    pub use once_cell;
}

#[macro_use]
#[allow(missing_docs)]
mod macros;

mod derive;

pub mod builder;
pub mod error;
pub mod parser;

mod mkeymap;
mod output;
mod util;

const INTERNAL_ERROR_MSG: &str = "Fatal internal error. Please consider filing a bug \
                                  report at https://github.com/clap-rs/clap/issues";
