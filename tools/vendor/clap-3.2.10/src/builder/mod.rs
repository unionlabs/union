//! Define [`Command`] line [arguments][`Arg`]

#[macro_use]
mod macros;

mod action;
mod app_settings;
mod arg;
mod arg_group;
mod arg_predicate;
mod arg_settings;
mod command;
mod possible_value;
mod usage_parser;
mod value_hint;
mod value_parser;

#[cfg(feature = "regex")]
mod regex;

#[cfg(debug_assertions)]
mod debug_asserts;

#[cfg(test)]
mod tests;

pub use action::ArgAction;
pub use app_settings::{AppFlags, AppSettings};
pub use arg::Arg;
pub use arg_group::ArgGroup;
pub use arg_settings::{ArgFlags, ArgSettings};
pub use command::Command;
pub use possible_value::PossibleValue;
pub use value_hint::ValueHint;
pub use value_parser::PossibleValuesParser;
pub use value_parser::RangedI64ValueParser;
pub use value_parser::RangedU64ValueParser;
pub use value_parser::StringValueParser;
pub use value_parser::TypedValueParser;
pub use value_parser::ValueParser;
pub use value_parser::ValueParserFactory;
pub use value_parser::_AnonymousValueParser;
pub use value_parser::_AutoValueParser;
pub use value_parser::via_prelude;
pub use value_parser::BoolValueParser;
pub use value_parser::BoolishValueParser;
pub use value_parser::EnumValueParser;
pub use value_parser::FalseyValueParser;
pub use value_parser::NonEmptyStringValueParser;
pub use value_parser::OsStringValueParser;
pub use value_parser::PathBufValueParser;

#[allow(deprecated)]
pub use command::App;

#[cfg(feature = "regex")]
pub use self::regex::RegexRef;

pub(crate) use action::CountType;
pub(crate) use arg::display_arg_val;
pub(crate) use arg_predicate::ArgPredicate;
