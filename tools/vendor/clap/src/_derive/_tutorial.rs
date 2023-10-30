// Contributing
//
// New example code:
// - Please update the corresponding section in the derive tutorial
// - Building: They must be added to `Cargo.toml` with the appropriate `required-features`.
// - Testing: Ensure there is a markdown file with [trycmd](https://docs.rs/trycmd) syntax
//
// See also the general CONTRIBUTING

//! # Documentation: Derive Tutorial
//!
//! 1. [Quick Start](#quick-start)
//! 2. [Configuring the Parser](#configuring-the-parser)
//! 3. [Adding Arguments](#adding-arguments)
//!     1. [Positionals](#positionals)
//!     2. [Options](#options)
//!     3. [Flags](#flags)
//!     4. [Subcommands](#subcommands)
//!     5. [Defaults](#defaults)
//! 4. Validation
//!     1. [Enumerated values](#enumerated-values)
//!     2. [Validated values](#validated-values)
//!     3. [Argument Relations](#argument-relations)
//!     4. [Custom Validation](#custom-validation)
//! 5. [Testing](#testing)
//! 6. [Next Steps](#next-steps)
//!
//! See also
//! - [FAQ: When should I use the builder vs derive APIs?][crate::_faq#when-should-i-use-the-builder-vs-derive-apis]
//! - The [cookbook][crate::_cookbook] for more application-focused examples
//!
//! ## Quick Start
//!
//! You can create an application declaratively with a `struct` and some
//! attributes.
//!
//! First, ensure `clap` is available with the [`derive` feature flag][crate::_features]:
//! ```console
//! $ cargo add clap --features derive
//! ```
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/01_quick.rs")]
//! ```
//!
#![doc = include_str!("../../examples/tutorial_derive/01_quick.md")]
//!
//! ## Configuring the Parser
//!
//! You use derive [`Parser`][crate::Parser] to start building a parser.
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/02_apps.rs")]
//! ```
//!
#![doc = include_str!("../../examples/tutorial_derive/02_apps.md")]
//!
//! You can use [`#[command(author, version, about)]` attribute defaults][super#command-attributes] to fill these fields in from your `Cargo.toml` file.
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/02_crate.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/02_crate.md")]
//!
//! You can use attributes to change the application level behavior of clap.  Any [`Command`][crate::Command] builder function can be used as an attribute, like [`Command::next_line_help`].
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/02_app_settings.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/02_app_settings.md")]
//!
//! ## Adding Arguments
//!
//! ### Positionals
//!
//! You can have users specify values by their position on the command-line:
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_03_positional.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_03_positional.md")]
//!
//! Note that the [default `ArgAction` is `Set`][super#arg-types].  To
//! accept multiple values, override the [action][Arg::action] with [`Append`][crate::ArgAction::Append] via `Vec`:
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_03_positional_mult.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_03_positional_mult.md")]
//!
//! ### Options
//!
//! You can name your arguments with a flag:
//! - Order doesn't matter
//! - They can be optional
//! - Intent is clearer
//!
//! The [`#[arg(short = 'n')]`][Arg::short] and [`#[arg(long = "name")]`][Arg::long] attributes that define
//! the flags are [`Arg`][crate::Args] methods that are derived from the field name when no value
//! is specified ([`#[arg(short)]` and `#[arg(long)]`][super#arg-attributes]).
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_02_option.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_02_option.md")]
//!
//! Note that the [default `ArgAction` is `Set`][super#arg-types].  To
//! accept multiple occurrences, override the [action][Arg::action] with [`Append`][crate::ArgAction::Append] via `Vec`:
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_02_option_mult.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_02_option_mult.md")]
//!
//! ### Flags
//!
//! Flags can also be switches that can be on/off:
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_01_flag_bool.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_01_flag_bool.md")]
//!
//! Note that the [default `ArgAction` for a `bool` field is
//! `SetTrue`][super#arg-types].  To accept multiple flags, override the [action][Arg::action] with
//! [`Count`][crate::ArgAction::Count]:
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_01_flag_count.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_01_flag_count.md")]
//!
//! ### Subcommands
//!
//! Subcommands are derived with `#[derive(Subcommand)]` and be added via [`#[command(subcommand)]` attribute][super#command-attributes]. Each
//! instance of a [Subcommand][crate::Subcommand] can have its own version, author(s), Args, and even its own
//! subcommands.
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_04_subcommands.rs")]
//! ```
//! We used a struct-variant to define the `add` subcommand.
//! Alternatively, you can use a struct for your subcommand's arguments:
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_04_subcommands_alt.rs")]
//! ```
//!
#![doc = include_str!("../../examples/tutorial_derive/03_04_subcommands.md")]
//!
//! ### Defaults
//!
//! We've previously showed that arguments can be [`required`][crate::Arg::required] or optional.
//! When optional, you work with a `Option` and can `unwrap_or`.  Alternatively, you can
//! set [`#[arg(default_value_t)]`][super#arg-attributes].
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/03_05_default_values.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/03_05_default_values.md")]
//!
//! ## Validation
//!
//! An appropriate default parser/validator will be selected for the field's type.  See
//! [`value_parser!`][crate::value_parser!] for more details.
//!
//! ### Enumerated values
//!
//! For example, if you have arguments of specific values you want to test for, you can derive
//! [`ValueEnum`][super#valueenum-attributes]
//! (any [`PossibleValue`] builder function can be used as variant attributes).
//!
//! This allows you specify the valid values for that argument. If the user does not use one of
//! those specific values, they will receive a graceful exit with error message informing them
//! of the mistake, and what the possible valid values are
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/04_01_enum.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/04_01_enum.md")]
//!
//! ### Validated values
//!
//! More generally, you can validate and parse into any data type with [`Arg::value_parser`].
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/04_02_parse.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/04_02_parse.md")]
//!
//! A [custom parser][TypedValueParser] can be used to improve the error messages or provide additional validation:
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/04_02_validate.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/04_02_validate.md")]
//!
//! See [`Arg::value_parser`][crate::Arg::value_parser] for more details.
//!
//! ### Argument Relations
//!
//! You can declare dependencies or conflicts between [`Arg`][crate::Arg]s or even
//! [`ArgGroup`][crate::ArgGroup]s.
//!
//! [`ArgGroup`][crate::ArgGroup]s  make it easier to declare relations instead of having to list
//! each individually, or when you want a rule to apply "any but not all" arguments.
//!
//! Perhaps the most common use of [`ArgGroup`][crate::ArgGroup]s is to require one and *only* one
//! argument to be present out of a given set. Imagine that you had multiple arguments, and you
//! want one of them to be required, but making all of them required isn't feasible because perhaps
//! they conflict with each other.
//!
//! [`ArgGroup`][crate::ArgGroup]s are automatically created for a `struct` with its
//! [`ArgGroup::id`][crate::ArgGroup::id] being the struct's name.
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/04_03_relations.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/04_03_relations.md")]
//!
//! ### Custom Validation
//!
//! As a last resort, you can create custom errors with the basics of clap's formatting.
//!
//! ```rust
#![doc = include_str!("../../examples/tutorial_derive/04_04_custom.rs")]
//! ```
#![doc = include_str!("../../examples/tutorial_derive/04_04_custom.md")]
//!
//! ## Testing
//!
//! clap reports most development errors as `debug_assert!`s.  Rather than checking every
//! subcommand, you should have a test that calls
//! [`Command::debug_assert`][crate::Command::debug_assert]:
//! ```rust,no_run
#![doc = include_str!("../../examples/tutorial_derive/05_01_assert.rs")]
//! ```
//!
//! ## Next Steps
//!
//! - [Cookbook][crate::_cookbook] for application-focused examples
//! - Explore more features in the [Derive reference][super]
//!   - See also [`Command`], [`Arg`], [`ArgGroup`], and [`PossibleValue`] builder functions which
//!     can be used as attributes
//!
//! For support, see [Discussions](https://github.com/clap-rs/clap/discussions)

#![allow(unused_imports)]
use crate::builder::Arg;
use crate::builder::ArgGroup;
use crate::builder::Command;
use crate::builder::PossibleValue;
use crate::builder::TypedValueParser;
