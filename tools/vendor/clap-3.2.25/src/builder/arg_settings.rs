#![allow(deprecated)]

// Std
use std::ops::BitOr;
#[cfg(feature = "yaml")]
use std::str::FromStr;

// Third party
use bitflags::bitflags;

#[allow(unused)]
use crate::Arg;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgFlags(Flags);

impl Default for ArgFlags {
    fn default() -> Self {
        Self::empty()
    }
}

/// Various settings that apply to arguments and may be set, unset, and checked via getter/setter
/// methods [`Arg::setting`], [`Arg::unset_setting`], and [`Arg::is_set`]. This is what the
/// [`Arg`] methods which accept a `bool` use internally.
///
/// [`Arg`]: crate::Arg
/// [`Arg::setting`]: crate::Arg::setting()
/// [`Arg::unset_setting`]: crate::Arg::unset_setting()
/// [`Arg::is_set`]: crate::Arg::is_set()
#[derive(Debug, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum ArgSettings {
    /// Deprecated, replaced with [`Arg::required`] and [`Arg::is_required_set`]
    ///
    /// Derive: replace `#[clap(setting = Required)]` with `#[clap(required = true)]`
    ///
    /// Builder: replace `arg.setting(Required)` with `arg.required(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::required` and `Arg::is_required_set`

Derive: replace `#[clap(setting = Required)]` with `#[clap(required = true)]`

Builder: replace `arg.setting(Required)` with `arg.required(true)`
"
        )
    )]
    Required,
    /// Deprecated, replaced with [`Arg::multiple_values`] and [`Arg::is_multiple_values_set`]
    ///
    /// Derive: replace `#[clap(setting = MultipleValues)]` with `#[clap(multiple_values = true)]`
    ///
    /// Builder: replace `arg.setting(MultipleValues)` with `arg.multiple_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::multiple_values` and `Arg::`is_multiple_values_set`

Derive: replace `#[clap(setting = MultipleValues)]` with `#[clap(multiple_values = true)]`

Builder: replace `arg.setting(MultipleValues)` with `arg.multiple_values(true)`
"
        )
    )]
    MultipleValues,
    /// Deprecated, replaced with [`Arg::action`] ([Issue #3772](https://github.com/clap-rs/clap/issues/3772))
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::action` (Issue #3772)

Builder: replace `arg.setting(MultipleOccurrences)` with `arg.action(ArgAction::Append)` when taking a value and `arg.action(ArgAction::Count)` with `matches.get_count` when not
"
        )
    )]
    MultipleOccurrences,
    /// Deprecated, see [`ArgSettings::MultipleOccurrences`] (most likely what you want) and
    /// [`ArgSettings::MultipleValues`]
    ///
    /// Derive: replace `#[clap(setting = Multiple)]` with `#[clap(multiple_values = true, multiple_occurrences = true)]`
    ///
    /// Builder: replace `arg.setting(Multiple)` with `arg.multiple_values(true).multiple_occurrences(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Split into `Arg::multiple_occurrences` (most likely what you want)  and `Arg::multiple_values`

Derive: replace `#[clap(setting = Multiple)]` with `#[clap(multiple_values = true, multiple_occurrences = true)]`

Builder: replace `arg.setting(Multiple)` with `arg.multiple_values(true).multiple_occurrences(true)`
"
        )
    )]
    #[doc(hidden)]
    Multiple,
    /// Deprecated, replaced with [`Arg::value_parser(NonEmptyStringValueParser::new())`]
    ///
    /// Derive: replace `#[clap(setting = ForbidEmptyValues)]` with `#[clap(value_parser = NonEmptyStringValueParser::new())]`
    ///
    /// Builder: replace `arg.setting(Multiple)` with `arg.value_parser(NonEmptyStringValueParser::new())`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::value_parser(NonEmptyStringValueParser::new())`

Derive: replace `#[clap(setting = ForbidEmptyValues)]` with `#[clap(value_parser = NonEmptyStringValueParser::new())]`

Builder: replace `arg.setting(Multiple)` with `arg.value_parser(NonEmptyStringValueParser::new())`
"
        )
    )]
    ForbidEmptyValues,
    /// Deprecated, replaced with [`Arg::global`] and [`Arg::is_global_set`]
    ///
    /// Derive: replace `#[clap(setting = Global)]` with `#[clap(global = true)]`
    ///
    /// Builder: replace `arg.setting(Global)` with `arg.global(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::global` and `Arg::is_global_set`

Derive: replace `#[clap(setting = Global)]` with `#[clap(global = true)]`

Builder: replace `arg.setting(Global)` with `arg.global(true)`
"
        )
    )]
    Global,
    /// Deprecated, replaced with [`Arg::hide`] and [`Arg::is_hide_set`]
    ///
    /// Derive: replace `#[clap(setting = Hidden)]` with `#[clap(hide = true)]`
    ///
    /// Builder: replace `arg.setting(Hidden)` with `arg.hide(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide` and `Arg::is_hide_set`

Derive: replace `#[clap(setting = Hidden)]` with `#[clap(hide = true)]`

Builder: replace `arg.setting(Hidden)` with `arg.hide(true)`
"
        )
    )]
    Hidden,
    /// Deprecated, replaced with [`Arg::takes_value`] and [`Arg::is_takes_value_set`]
    ///
    /// Derive: this setting shouldn't be needed
    ///
    /// Builder: replace `arg.setting(TakesValue)` with `arg.takes_value(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::takes_value` and `Arg::is_takes_value_set`

Derive: this setting shouldn't be needed

Builder: replace `arg.setting(TakesValue)` with `arg.takes_value(true)`
"
        )
    )]
    TakesValue,
    /// Deprecated, replaced with [`Arg::use_value_delimiter`] and
    /// [`Arg::is_use_value_delimiter_set`]
    ///
    /// Derive: replace `#[clap(setting = UseValueDelimiter)]` with `#[clap(use_value_delimiter = true)]`
    ///
    /// Builder: replace `arg.setting(UseValueDelimiter)` with `arg.use_value_delimiter(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::use_value_delimiter` and `Arg::is_use_value_delimiter_set`

Derive: replace `#[clap(setting = UseValueDelimiter)]` with `#[clap(use_value_delimiter = true)]`

Builder: replace `arg.setting(UseValueDelimiter)` with `arg.use_value_delimiter(true)`
"
        )
    )]
    UseValueDelimiter,
    /// Deprecated, replaced with [`Arg::next_line_help`] and [`Arg::is_next_line_help_set`]
    ///
    /// Derive: replace `#[clap(setting = NextLineHelp)]` with `#[clap(next_line_help = true)]`
    ///
    /// Builder: replace `arg.setting(NextLineHelp)` with `arg.next_line_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::next_line_help` and `Arg::is_next_line_help_set`

Derive: replace `#[clap(setting = NextLineHelp)]` with `#[clap(next_line_help = true)]`

Builder: replace `arg.setting(NextLineHelp)` with `arg.next_line_help(true)`
"
        )
    )]
    NextLineHelp,
    /// Deprecated, replaced with [`Arg::require_value_delimiter`] and
    /// [`Arg::is_require_value_delimiter_set`]
    ///
    /// Derive: replace `#[clap(setting = RequireDelimiter)]` with `#[clap(require_value_delimiter = true)]`
    ///
    /// Builder: replace `arg.setting(RequireDelimiter)` with `arg.require_value_delimiter(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::require_value_delimiter` and `Arg::is_require_value_delimiter_set`

Derive: replace `#[clap(setting = RequireDelimiter)]` with `#[clap(require_value_delimiter = true)]`

Builder: replace `arg.setting(RequireDelimiter)` with `arg.require_value_delimiter(true)`
"
        )
    )]
    RequireDelimiter,
    /// Deprecated, replaced with [`Arg::hide_possible_values`] and
    /// [`Arg::is_hide_possible_values_set`]
    ///
    /// Derive: replace `#[clap(setting = HidePossibleValues)]` with `#[clap(hide_possible_values = true)]`
    ///
    /// Builder: replace `arg.setting(HidePossibleValues)` with `arg.hide_possible_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide_possible_values` and `Arg::is_hide_possible_values_set`

Derive: replace `#[clap(setting = HidePossibleValues)]` with `#[clap(hide_possible_values = true)]`

Builder: replace `arg.setting(HidePossibleValues)` with `arg.hide_possible_values(true)`
"
        )
    )]
    HidePossibleValues,
    /// Deprecated, replaced with [`Arg::allow_hyphen_values`] and
    /// [`Arg::is_allow_hyphen_values_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowHyphenValues)]` with `#[clap(allow_hyphen_values = true)]`
    ///
    /// Builder: replace `arg.setting(AllowHyphenValues)` with `arg.allow_hyphen_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::allow_hyphen_values` and `Arg::is_allow_hyphen_values_set`

Derive: replace `#[clap(setting = AllowHyphenValues)]` with `#[clap(allow_hyphen_values = true)]`

Builder: replace `arg.setting(AllowHyphenValues)` with `arg.allow_hyphen_values(true)`
"
        )
    )]
    AllowHyphenValues,
    /// Deprecated, replaced with [`Arg::allow_hyphen_values`] and
    /// [`Arg::is_allow_hyphen_values_set`]
    ///
    /// Derive: replace `#[clap(setting = AllowLeadingHyphen)]` with `#[clap(allow_hyphen_values = true)]`
    ///
    /// Builder: replace `arg.setting(AllowLeadingHyphen)` with `arg.allow_hyphen_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Arg::allow_hyphen_values` and `Arg::is_allow_hyphen_values_set`

Derive: replace `#[clap(setting = AllowLeadingHyphen)]` with `#[clap(allow_hyphen_values = true)]`

Builder: replace `arg.setting(AllowLeadingHyphen)` with `arg.allow_hyphen_values(true)`
"
        )
    )]
    #[doc(hidden)]
    AllowLeadingHyphen,
    /// Deprecated, replaced with [`Arg::require_equals`] and [`Arg::is_require_equals_set`]
    ///
    /// Derive: replace `#[clap(setting = RequireEquals)]` with `#[clap(require_equals = true)]`
    ///
    /// Builder: replace `arg.setting(RequireEquals)` with `arg.require_equals(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::require_equals` and `Arg::is_require_equals_set`

Derive: replace `#[clap(setting = RequireEquals)]` with `#[clap(require_equals = true)]`

Builder: replace `arg.setting(RequireEquals)` with `arg.require_equals(true)`
"
        )
    )]
    RequireEquals,
    /// Deprecated, replaced with [`Arg::last`] and [`Arg::is_last_set`]
    ///
    /// Derive: replace `#[clap(setting = Last)]` with `#[clap(last = true)]`
    ///
    /// Builder: replace `arg.setting(Last)` with `arg.last(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::last` and `Arg::is_last_set`

Derive: replace `#[clap(setting = Last)]` with `#[clap(last = true)]`

Builder: replace `arg.setting(Last)` with `arg.last(true)`
"
        )
    )]
    Last,
    /// Deprecated, replaced with [`Arg::hide_default_value`] and [`Arg::is_hide_default_value_set`]
    ///
    /// Derive: replace `#[clap(setting = HideDefaultValue)]` with `#[clap(hide_default_value = true)]`
    ///
    /// Builder: replace `arg.setting(HideDefaultValue)` with `arg.hide_default_value(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide_default_value` and `Arg::is_hide_default_value_set`

Derive: replace `#[clap(setting = HideDefaultValue)]` with `#[clap(hide_default_value = true)]`

Builder: replace `arg.setting(HideDefaultValue)` with `arg.hide_default_value(true)`
"
        )
    )]
    HideDefaultValue,
    /// Deprecated, replaced with [`Arg::ignore_case`] and [`Arg::is_ignore_case_set`]
    ///
    /// Derive: replace `#[clap(setting = IgnoreCase)]` with `#[clap(ignore_case = true)]`
    ///
    /// Builder: replace `arg.setting(IgnoreCase)` with `arg.ignore_case(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::ignore_case` and `Arg::is_ignore_case_set`

Derive: replace `#[clap(setting = IgnoreCase)]` with `#[clap(ignore_case = true)]`

Builder: replace `arg.setting(IgnoreCase)` with `arg.ignore_case(true)`
"
        )
    )]
    IgnoreCase,
    /// Deprecated, replaced with [`Arg::ignore_case`] and [`Arg::is_ignore_case_set`]
    ///
    /// Derive: replace `#[clap(setting = CaseInsensitive)]` with `#[clap(ignore_case = true)]`
    ///
    /// Builder: replace `arg.setting(CaseInsensitive)` with `arg.ignore_case(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `Arg::ignore_case` and `Arg::is_ignore_case_set`

Derive: replace `#[clap(setting = CaseInsensitive)]` with `#[clap(ignore_case = true)]`

Builder: replace `arg.setting(CaseInsensitive)` with `arg.ignore_case(true)`
"
        )
    )]
    #[doc(hidden)]
    CaseInsensitive,
    /// Deprecated, replaced with [`Arg::hide_env`] and [`Arg::is_hide_env_set`]
    ///
    /// Derive: replace `#[clap(setting = HideEnv)]` with `#[clap(hide_env = true)]`
    ///
    /// Builder: replace `arg.setting(HideEnv)` with `arg.hide_env(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide_env` and `Arg::is_hide_env_set`

Derive: replace `#[clap(setting = HideEnv)]` with `#[clap(hide_env = true)]`

Builder: replace `arg.setting(HideEnv)` with `arg.hide_env(true)`
"
        )
    )]
    #[cfg(feature = "env")]
    HideEnv,
    /// Deprecated, replaced with [`Arg::hide_env_values`] and [`Arg::is_hide_env_values_set`]
    ///
    /// Derive: replace `#[clap(setting = HideEnvValues)]` with `#[clap(hide_env_values = true)]`
    ///
    /// Builder: replace `arg.setting(HideEnvValues)` with `arg.hide_env_values(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide_env_values` and `Arg::is_hide_env_values_set`

Derive: replace `#[clap(setting = HideEnvValues)]` with `#[clap(hide_env_values = true)]`

Builder: replace `arg.setting(HideEnvValues)` with `arg.hide_env_values(true)`
"
        )
    )]
    #[cfg(feature = "env")]
    HideEnvValues,
    /// Deprecated, replaced with [`Arg::hide_short_help`] and [`Arg::is_hide_short_help_set`]
    ///
    /// Derive: replace `#[clap(setting = HiddenShortHelp)]` with `#[clap(hide_short_help = true)]`
    ///
    /// Builder: replace `arg.setting(HiddenShortHelp)` with `arg.hide_short_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide_short_help` and `Arg::is_hide_short_help_set`

Derive: replace `#[clap(setting = HiddenShortHelp)]` with `#[clap(hide_short_help = true)]`

Builder: replace `arg.setting(HiddenShortHelp)` with `arg.hide_short_help(true)`
"
        )
    )]
    HiddenShortHelp,
    /// Deprecated, replaced with [`Arg::hide_long_help`] and [`Arg::is_hide_long_help_set`]
    ///
    /// Derive: replace `#[clap(setting = HiddenLongHelp)]` with `#[clap(hide_long_help = true)]`
    ///
    /// Builder: replace `arg.setting(HiddenLongHelp)` with `arg.hide_long_help(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::hide_long_help` and `Arg::is_hide_long_help_set`

Derive: replace `#[clap(setting = HiddenLongHelp)]` with `#[clap(hide_long_help = true)]`

Builder: replace `arg.setting(HiddenLongHelp)` with `arg.hide_long_help(true)`
"
        )
    )]
    HiddenLongHelp,
    /// Deprecated, replaced with [`Arg::value_parser`]
    ///
    /// Derive: replace `#[clap(setting = AllowInvalidUtf8)]` with `#[clap(action)]` (which opts-in to the
    /// new clap v4 behavior which gets the type via `value_parser!`)
    ///
    /// Builder: replace `arg.setting(AllowInvalidUtf8)` with `arg.value_parser(value_parser!(T))` where
    /// `T` is the type of interest, like `OsString` or `PathBuf`, and `matches.value_of_os` with
    /// `matches.get_one::<T>` or `matches.values_of_os` with `matches.get_many::<T>`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `value_parser`

Derive: replace `#[clap(setting = AllowInvalidUtf8)]` with `#[clap(action)]` (which opts-in to the
new clap v4 behavior which gets the type via `value_parser!`)

Builder: replace `arg.setting(AllowInvalidUtf8)` with `arg.value_parser(value_parser!(T))` where
`T` is the type of interest, like `OsString` or `PathBuf`, and `matches.value_of_os` with
`matches.get_one::<T>` or `matches.values_of_os` with `matches.get_many::<T>`
"
        )
    )]
    AllowInvalidUtf8,
    /// Deprecated, replaced with [`Arg::exclusive`] and [`Arg::is_exclusive_set`]
    ///
    /// Derive: replace `#[clap(setting = Exclusive)]` with `#[clap(exclusive = true)]`
    ///
    /// Builder: replace `arg.setting(Exclusive)` with `arg.exclusive(true)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `Arg::exclusive` and `Arg::is_exclusive_set`

Derive: replace `#[clap(setting = Exclusive)]` with `#[clap(exclusive = true)]`

Builder: replace `arg.setting(Exclusive)` with `arg.exclusive(true)`
"
        )
    )]
    Exclusive,
}

bitflags! {
    struct Flags: u32 {
        const REQUIRED         = 1;
        const MULTIPLE_OCC     = 1 << 1;
        const NO_EMPTY_VALS    = 1 << 2;
        const GLOBAL           = 1 << 3;
        const HIDDEN           = 1 << 4;
        const TAKES_VAL        = 1 << 5;
        const USE_DELIM        = 1 << 6;
        const NEXT_LINE_HELP   = 1 << 7;
        const REQ_DELIM        = 1 << 9;
        const DELIM_NOT_SET    = 1 << 10;
        const HIDE_POS_VALS    = 1 << 11;
        const ALLOW_TAC_VALS   = 1 << 12;
        const REQUIRE_EQUALS   = 1 << 13;
        const LAST             = 1 << 14;
        const HIDE_DEFAULT_VAL = 1 << 15;
        const CASE_INSENSITIVE = 1 << 16;
        #[cfg(feature = "env")]
        const HIDE_ENV_VALS    = 1 << 17;
        const HIDDEN_SHORT_H   = 1 << 18;
        const HIDDEN_LONG_H    = 1 << 19;
        const MULTIPLE_VALS    = 1 << 20;
        const MULTIPLE         = Self::MULTIPLE_OCC.bits | Self::MULTIPLE_VALS.bits;
        #[cfg(feature = "env")]
        const HIDE_ENV         = 1 << 21;
        const UTF8_NONE        = 1 << 22;
        const EXCLUSIVE        = 1 << 23;
        const NO_OP            = 0;
    }
}

impl_settings! { ArgSettings, ArgFlags,
    Required => Flags::REQUIRED,
    MultipleOccurrences => Flags::MULTIPLE_OCC,
    MultipleValues => Flags::MULTIPLE_VALS,
    Multiple => Flags::MULTIPLE,
    ForbidEmptyValues => Flags::NO_EMPTY_VALS,
    Global => Flags::GLOBAL,
    Hidden => Flags::HIDDEN,
    TakesValue => Flags::TAKES_VAL,
    UseValueDelimiter => Flags::USE_DELIM,
    NextLineHelp => Flags::NEXT_LINE_HELP,
    RequireDelimiter => Flags::REQ_DELIM,
    HidePossibleValues => Flags::HIDE_POS_VALS,
    AllowHyphenValues => Flags::ALLOW_TAC_VALS,
    AllowLeadingHyphen => Flags::ALLOW_TAC_VALS,
    RequireEquals => Flags::REQUIRE_EQUALS,
    Last => Flags::LAST,
    IgnoreCase => Flags::CASE_INSENSITIVE,
    CaseInsensitive => Flags::CASE_INSENSITIVE,
    #[cfg(feature = "env")]
    HideEnv => Flags::HIDE_ENV,
    #[cfg(feature = "env")]
    HideEnvValues => Flags::HIDE_ENV_VALS,
    HideDefaultValue => Flags::HIDE_DEFAULT_VAL,
    HiddenShortHelp => Flags::HIDDEN_SHORT_H,
    HiddenLongHelp => Flags::HIDDEN_LONG_H,
    AllowInvalidUtf8 => Flags::UTF8_NONE,
    Exclusive => Flags::EXCLUSIVE
}

/// Deprecated in [Issue #3087](https://github.com/clap-rs/clap/issues/3087), maybe [`clap::Parser`][crate::Parser] would fit your use case?
#[cfg(feature = "yaml")]
impl FromStr for ArgSettings {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        #[allow(deprecated)]
        #[allow(unreachable_patterns)]
        match &*s.to_ascii_lowercase() {
            "required" => Ok(ArgSettings::Required),
            "multipleoccurrences" => Ok(ArgSettings::MultipleOccurrences),
            "multiplevalues" => Ok(ArgSettings::MultipleValues),
            "multiple" => Ok(ArgSettings::Multiple),
            "forbidemptyvalues" => Ok(ArgSettings::ForbidEmptyValues),
            "global" => Ok(ArgSettings::Global),
            "hidden" => Ok(ArgSettings::Hidden),
            "takesvalue" => Ok(ArgSettings::TakesValue),
            "usevaluedelimiter" => Ok(ArgSettings::UseValueDelimiter),
            "nextlinehelp" => Ok(ArgSettings::NextLineHelp),
            "requiredelimiter" => Ok(ArgSettings::RequireDelimiter),
            "hidepossiblevalues" => Ok(ArgSettings::HidePossibleValues),
            "allowhyphenvalues" => Ok(ArgSettings::AllowHyphenValues),
            "allowleadinghypyhen" => Ok(ArgSettings::AllowLeadingHyphen),
            "requireequals" => Ok(ArgSettings::RequireEquals),
            "last" => Ok(ArgSettings::Last),
            "ignorecase" => Ok(ArgSettings::IgnoreCase),
            "caseinsensitive" => Ok(ArgSettings::CaseInsensitive),
            #[cfg(feature = "env")]
            "hideenv" => Ok(ArgSettings::HideEnv),
            #[cfg(feature = "env")]
            "hideenvvalues" => Ok(ArgSettings::HideEnvValues),
            "hidedefaultvalue" => Ok(ArgSettings::HideDefaultValue),
            "hiddenshorthelp" => Ok(ArgSettings::HiddenShortHelp),
            "hiddenlonghelp" => Ok(ArgSettings::HiddenLongHelp),
            "allowinvalidutf8" => Ok(ArgSettings::AllowInvalidUtf8),
            "exclusive" => Ok(ArgSettings::Exclusive),
            _ => Err(format!("unknown AppSetting: `{}`", s)),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[cfg(feature = "yaml")]
    fn arg_settings_fromstr() {
        use super::ArgSettings;

        assert_eq!(
            "allowhyphenvalues".parse::<ArgSettings>().unwrap(),
            ArgSettings::AllowHyphenValues
        );
        assert_eq!(
            "forbidemptyvalues".parse::<ArgSettings>().unwrap(),
            ArgSettings::ForbidEmptyValues
        );
        assert_eq!(
            "hidepossiblevalues".parse::<ArgSettings>().unwrap(),
            ArgSettings::HidePossibleValues
        );
        assert_eq!(
            "hidden".parse::<ArgSettings>().unwrap(),
            ArgSettings::Hidden
        );
        assert_eq!(
            "nextlinehelp".parse::<ArgSettings>().unwrap(),
            ArgSettings::NextLineHelp
        );
        assert_eq!(
            "requiredelimiter".parse::<ArgSettings>().unwrap(),
            ArgSettings::RequireDelimiter
        );
        assert_eq!(
            "required".parse::<ArgSettings>().unwrap(),
            ArgSettings::Required
        );
        assert_eq!(
            "takesvalue".parse::<ArgSettings>().unwrap(),
            ArgSettings::TakesValue
        );
        assert_eq!(
            "usevaluedelimiter".parse::<ArgSettings>().unwrap(),
            ArgSettings::UseValueDelimiter
        );
        assert_eq!(
            "requireequals".parse::<ArgSettings>().unwrap(),
            ArgSettings::RequireEquals
        );
        assert_eq!("last".parse::<ArgSettings>().unwrap(), ArgSettings::Last);
        assert_eq!(
            "hidedefaultvalue".parse::<ArgSettings>().unwrap(),
            ArgSettings::HideDefaultValue
        );
        assert_eq!(
            "ignorecase".parse::<ArgSettings>().unwrap(),
            ArgSettings::IgnoreCase
        );
        #[cfg(feature = "env")]
        assert_eq!(
            "hideenv".parse::<ArgSettings>().unwrap(),
            ArgSettings::HideEnv
        );
        #[cfg(feature = "env")]
        assert_eq!(
            "hideenvvalues".parse::<ArgSettings>().unwrap(),
            ArgSettings::HideEnvValues
        );
        assert_eq!(
            "hiddenshorthelp".parse::<ArgSettings>().unwrap(),
            ArgSettings::HiddenShortHelp
        );
        assert_eq!(
            "hiddenlonghelp".parse::<ArgSettings>().unwrap(),
            ArgSettings::HiddenLongHelp
        );
        assert_eq!(
            "allowinvalidutf8".parse::<ArgSettings>().unwrap(),
            ArgSettings::AllowInvalidUtf8
        );
        assert_eq!(
            "exclusive".parse::<ArgSettings>().unwrap(),
            ArgSettings::Exclusive
        );
        assert!("hahahaha".parse::<ArgSettings>().is_err());
    }
}
