/// Behavior of arguments when they are encountered while parsing
///
/// # Examples
///
/// ```rust
/// # use clap::Command;
/// # use clap::Arg;
/// let cmd = Command::new("mycmd")
///     .arg(
///         Arg::new("special-help")
///             .short('?')
///             .action(clap::ArgAction::Help)
///     );
///
/// // Existing help still exists
/// let err = cmd.clone().try_get_matches_from(["mycmd", "-h"]).unwrap_err();
/// assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
///
/// // New help available
/// let err = cmd.try_get_matches_from(["mycmd", "-?"]).unwrap_err();
/// assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
#[allow(missing_copy_implementations)] // In the future, we may accept `Box<dyn ...>`
pub enum ArgAction {
    /// When encountered, store the associated value(s) in [`ArgMatches`][crate::ArgMatches]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("flag")
    ///             .long("flag")
    ///             .action(clap::ArgAction::Set)
    ///     );
    ///
    /// let matches = cmd.try_get_matches_from(["mycmd", "--flag", "value"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_many::<String>("flag").unwrap_or_default().map(|v| v.as_str()).collect::<Vec<_>>(),
    ///     vec!["value"]
    /// );
    /// ```
    Set,
    /// When encountered, store the associated value(s) in [`ArgMatches`][crate::ArgMatches]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("flag")
    ///             .long("flag")
    ///             .action(clap::ArgAction::Append)
    ///     );
    ///
    /// let matches = cmd.try_get_matches_from(["mycmd", "--flag", "value1", "--flag", "value2"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_many::<String>("flag").unwrap_or_default().map(|v| v.as_str()).collect::<Vec<_>>(),
    ///     vec!["value1", "value2"]
    /// );
    /// ```
    Append,
    /// Deprecated, replaced with [`ArgAction::Set`] or [`ArgAction::Append`]
    ///
    /// Builder: Instead of `arg.action(ArgAction::StoreValue)`,
    /// - Use `arg.action(ArgAction::Set)` for single-occurrence arguments
    /// - Use `arg.action(ArgAction::Append)` for multiple-occurrence arguments
    ///
    /// Derive: opt-in to the new behavior with `#[clap(action)]`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgAction::Set` or `ArgAction::Append`

Derive: opt-in to the new behavior with `#[clap(action)]`

Builder: Instead of `arg.action(ArgAction::StoreValue)`,
- Use `arg.action(ArgAction::Set)` for single-occurrence arguments
- Use `arg.action(ArgAction::Append)` for multiple-occurrence arguments
"
        )
    )]
    StoreValue,
    /// Deprecated, replaced with [`ArgAction::SetTrue`] or [`ArgAction::Count`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgAction::SetTrue` or `ArgAction::Count`

Derive: opt-in to the new behavior with `#[clap(action)]`

Builder: Instead of `arg.action(ArgAction::IncOccurrence)`,
- Use `arg.action(ArgAction::SetTrue)` if you just care if its set, then switch `matches.is_present` to `matches.get_flag`
- Use `arg.action(ArgAction::Count)` if you care how many times its set, then switch `matches.occurrences_of` to `matches.get_count`
"
        )
    )]
    IncOccurrence,
    /// When encountered, act as if `"true"` was encountered on the command-line
    ///
    /// If no [`default_value`][super::Arg::default_value] is set, it will be `false`.
    ///
    /// No value is allowed. To optionally accept a value, see
    /// [`Arg::default_missing_value`][super::Arg::default_missing_value]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("flag")
    ///             .long("flag")
    ///             .action(clap::ArgAction::SetTrue)
    ///     );
    ///
    /// let matches = cmd.clone().try_get_matches_from(["mycmd", "--flag", "--flag"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_one::<bool>("flag").copied(),
    ///     Some(true)
    /// );
    ///
    /// let matches = cmd.try_get_matches_from(["mycmd"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_one::<bool>("flag").copied(),
    ///     Some(false)
    /// );
    /// ```
    SetTrue,
    /// When encountered, act as if `"false"` was encountered on the command-line
    ///
    /// If no [`default_value`][super::Arg::default_value] is set, it will be `true`.
    ///
    /// No value is allowed. To optionally accept a value, see
    /// [`Arg::default_missing_value`][super::Arg::default_missing_value]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("flag")
    ///             .long("flag")
    ///             .action(clap::ArgAction::SetFalse)
    ///     );
    ///
    /// let matches = cmd.clone().try_get_matches_from(["mycmd", "--flag", "--flag"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_one::<bool>("flag").copied(),
    ///     Some(false)
    /// );
    ///
    /// let matches = cmd.try_get_matches_from(["mycmd"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_one::<bool>("flag").copied(),
    ///     Some(true)
    /// );
    /// ```
    SetFalse,
    /// When encountered, increment a `u8` counter
    ///
    /// If no [`default_value`][super::Arg::default_value] is set, it will be `0`.
    ///
    /// No value is allowed. To optionally accept a value, see
    /// [`Arg::default_missing_value`][super::Arg::default_missing_value]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("flag")
    ///             .long("flag")
    ///             .action(clap::ArgAction::Count)
    ///     );
    ///
    /// let matches = cmd.clone().try_get_matches_from(["mycmd", "--flag", "--flag"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_count("flag"),
    ///     2
    /// );
    ///
    /// let matches = cmd.try_get_matches_from(["mycmd"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(matches.occurrences_of("flag"), 0);
    /// assert_eq!(
    ///     matches.get_count("flag"),
    ///     0
    /// );
    /// ```
    Count,
    /// When encountered, display [`Command::print_help`][super::App::print_help]
    ///
    /// Depending on the flag, [`Command::print_long_help`][super::App::print_long_help] may be shown
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("special-help")
    ///             .short('?')
    ///             .action(clap::ArgAction::Help)
    ///     );
    ///
    /// // Existing help still exists
    /// let err = cmd.clone().try_get_matches_from(["mycmd", "-h"]).unwrap_err();
    /// assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    ///
    /// // New help available
    /// let err = cmd.try_get_matches_from(["mycmd", "-?"]).unwrap_err();
    /// assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    /// ```
    Help,
    /// When encountered, display [`Command::version`][super::App::version]
    ///
    /// Depending on the flag, [`Command::long_version`][super::App::long_version] may be shown
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// let cmd = Command::new("mycmd")
    ///     .version("1.0.0")
    ///     .arg(
    ///         Arg::new("special-version")
    ///             .long("special-version")
    ///             .action(clap::ArgAction::Version)
    ///     );
    ///
    /// // Existing help still exists
    /// let err = cmd.clone().try_get_matches_from(["mycmd", "--version"]).unwrap_err();
    /// assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    ///
    /// // New help available
    /// let err = cmd.try_get_matches_from(["mycmd", "--special-version"]).unwrap_err();
    /// assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    /// ```
    Version,
}

impl ArgAction {
    /// Returns whether this action accepts values on the command-line
    ///
    /// [`default_values`][super::Arg::default_values] and [`env`][super::Arg::env] may still be
    /// processed.
    pub fn takes_values(&self) -> bool {
        match self {
            Self::Set => true,
            Self::Append => true,
            #[allow(deprecated)]
            Self::StoreValue => true,
            #[allow(deprecated)]
            Self::IncOccurrence => false,
            Self::SetTrue => false,
            Self::SetFalse => false,
            Self::Count => false,
            Self::Help => false,
            Self::Version => false,
        }
    }

    pub(crate) fn default_value(&self) -> Option<&'static std::ffi::OsStr> {
        match self {
            Self::Set => None,
            Self::Append => None,
            #[allow(deprecated)]
            Self::StoreValue => None,
            #[allow(deprecated)]
            Self::IncOccurrence => None,
            Self::SetTrue => Some(std::ffi::OsStr::new("false")),
            Self::SetFalse => Some(std::ffi::OsStr::new("true")),
            Self::Count => Some(std::ffi::OsStr::new("0")),
            Self::Help => None,
            Self::Version => None,
        }
    }

    pub(crate) fn default_value_parser(&self) -> Option<super::ValueParser> {
        match self {
            Self::Set => None,
            Self::Append => None,
            #[allow(deprecated)]
            Self::StoreValue => None,
            #[allow(deprecated)]
            Self::IncOccurrence => None,
            Self::SetTrue => Some(super::ValueParser::bool()),
            Self::SetFalse => Some(super::ValueParser::bool()),
            Self::Count => Some(crate::value_parser!(u8).into()),
            Self::Help => None,
            Self::Version => None,
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn value_type_id(&self) -> Option<crate::parser::AnyValueId> {
        use crate::parser::AnyValueId;

        match self {
            Self::Set => None,
            Self::Append => None,
            #[allow(deprecated)]
            Self::StoreValue => None,
            #[allow(deprecated)]
            Self::IncOccurrence => None,
            Self::SetTrue => Some(AnyValueId::of::<bool>()),
            Self::SetFalse => Some(AnyValueId::of::<bool>()),
            Self::Count => Some(AnyValueId::of::<CountType>()),
            Self::Help => None,
            Self::Version => None,
        }
    }
}

pub(crate) type CountType = u8;
