use std::{borrow::Cow, iter};

use crate::util::eq_ignore_case;

/// A possible value of an argument.
///
/// This is used for specifying [possible values] of [Args].
///
/// **NOTE:** This struct is likely not needed for most usecases as it is only required to
/// [hide] single values from help messages and shell completions or to attach [help] to possible values.
///
/// # Examples
///
/// ```rust
/// # use clap::{Arg, PossibleValue};
/// let cfg = Arg::new("config")
///     .takes_value(true)
///     .value_name("FILE")
///     .value_parser([
///         PossibleValue::new("fast"),
///         PossibleValue::new("slow").help("slower than fast"),
///         PossibleValue::new("secret speed").hide(true)
///     ]);
/// ```
/// [Args]: crate::Arg
/// [possible values]: crate::builder::ValueParser::possible_values
/// [hide]: PossibleValue::hide()
/// [help]: PossibleValue::help()
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PossibleValue<'help> {
    name: &'help str,
    help: Option<&'help str>,
    aliases: Vec<&'help str>, // (name, visible)
    hide: bool,
}

impl<'help> PossibleValue<'help> {
    /// Create a [`PossibleValue`] with its name.
    ///
    /// The name will be used to decide whether this value was provided by the user to an argument.
    ///
    /// **NOTE:** In case it is not [hidden] it will also be shown in help messages for arguments
    /// that use it as a [possible value] and have not hidden them through [`Arg::hide_possible_values(true)`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::PossibleValue;
    /// PossibleValue::new("fast")
    /// # ;
    /// ```
    /// [hidden]: PossibleValue::hide
    /// [possible value]: crate::Arg::possible_values
    /// [`Arg::hide_possible_values(true)`]: crate::Arg::hide_possible_values()
    pub fn new(name: &'help str) -> Self {
        PossibleValue {
            name,
            ..Default::default()
        }
    }

    /// Sets the help description of the value.
    ///
    /// This is typically displayed in completions (where supported) and should be a short, one-line
    /// description.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::PossibleValue;
    /// PossibleValue::new("slow")
    ///     .help("not fast")
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn help(mut self, help: &'help str) -> Self {
        self.help = Some(help);
        self
    }

    /// Hides this value from help and shell completions.
    ///
    /// This is an alternative to hiding through [`Arg::hide_possible_values(true)`], if you only
    /// want to hide some values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::PossibleValue;
    /// PossibleValue::new("secret")
    ///     .hide(true)
    /// # ;
    /// ```
    /// [`Arg::hide_possible_values(true)`]: crate::Arg::hide_possible_values()
    #[inline]
    #[must_use]
    pub fn hide(mut self, yes: bool) -> Self {
        self.hide = yes;
        self
    }

    /// Sets a *hidden* alias for this argument value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::PossibleValue;
    /// PossibleValue::new("slow")
    ///     .alias("not-fast")
    /// # ;
    /// ```
    #[must_use]
    pub fn alias(mut self, name: &'help str) -> Self {
        self.aliases.push(name);
        self
    }

    /// Sets multiple *hidden* aliases for this argument value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::PossibleValue;
    /// PossibleValue::new("slow")
    ///     .aliases(["not-fast", "snake-like"])
    /// # ;
    /// ```
    #[must_use]
    pub fn aliases<I>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = &'help str>,
    {
        self.aliases.extend(names.into_iter());
        self
    }
}

/// Reflection
impl<'help> PossibleValue<'help> {
    /// Get the name of the argument value
    #[inline]
    pub fn get_name(&self) -> &'help str {
        self.name
    }

    /// Get the help specified for this argument, if any
    #[inline]
    pub fn get_help(&self) -> Option<&'help str> {
        self.help
    }

    /// Get the help specified for this argument, if any and the argument
    /// value is not hidden
    #[inline]
    #[cfg(feature = "unstable-v4")]
    pub(crate) fn get_visible_help(&self) -> Option<&'help str> {
        if !self.hide {
            self.help
        } else {
            None
        }
    }

    /// Deprecated, replaced with [`PossibleValue::is_hide_set`]
    #[inline]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.0", note = "Replaced with `PossibleValue::is_hide_set`")
    )]
    pub fn is_hidden(&self) -> bool {
        self.is_hide_set()
    }

    /// Report if [`PossibleValue::hide`] is set
    #[inline]
    pub fn is_hide_set(&self) -> bool {
        self.hide
    }

    /// Report if PossibleValue is not hidden and has a help message
    pub(crate) fn should_show_help(&self) -> bool {
        !self.hide && self.help.is_some()
    }

    /// Get the name if argument value is not hidden, `None` otherwise
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.4",
            note = "Use `PossibleValue::is_hide_set` and `PossibleValue::get_name`"
        )
    )]
    pub fn get_visible_name(&self) -> Option<&'help str> {
        if self.hide {
            None
        } else {
            Some(self.name)
        }
    }

    /// Get the name if argument value is not hidden, `None` otherwise,
    /// but wrapped in quotes if it contains whitespace
    pub(crate) fn get_visible_quoted_name(&self) -> Option<Cow<'help, str>> {
        if !self.hide {
            Some(if self.name.contains(char::is_whitespace) {
                format!("{:?}", self.name).into()
            } else {
                self.name.into()
            })
        } else {
            None
        }
    }

    /// Returns all valid values of the argument value.
    ///
    /// Namely the name and all aliases.
    pub fn get_name_and_aliases(&self) -> impl Iterator<Item = &'help str> + '_ {
        iter::once(&self.name).chain(&self.aliases).copied()
    }

    /// Tests if the value is valid for this argument value
    ///
    /// The value is valid if it is either the name or one of the aliases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::PossibleValue;
    /// let arg_value = PossibleValue::new("fast").alias("not-slow");
    ///
    /// assert!(arg_value.matches("fast", false));
    /// assert!(arg_value.matches("not-slow", false));
    ///
    /// assert!(arg_value.matches("FAST", true));
    /// assert!(!arg_value.matches("FAST", false));
    /// ```
    pub fn matches(&self, value: &str, ignore_case: bool) -> bool {
        if ignore_case {
            self.get_name_and_aliases()
                .any(|name| eq_ignore_case(name, value))
        } else {
            self.get_name_and_aliases().any(|name| name == value)
        }
    }
}

impl<'help> From<&'help str> for PossibleValue<'help> {
    fn from(s: &'help str) -> Self {
        Self::new(s)
    }
}

impl<'help> From<&'help &'help str> for PossibleValue<'help> {
    fn from(s: &'help &'help str) -> Self {
        Self::new(s)
    }
}
