//! Error reporting
#![allow(deprecated)]

// Std
use std::{
    borrow::Cow,
    convert::From,
    error,
    fmt::{self, Debug, Display, Formatter},
    io::{self, BufRead},
    result::Result as StdResult,
};

// Internal
use crate::output::fmt::Colorizer;
use crate::output::fmt::Stream;
use crate::parser::features::suggestions;
use crate::util::{color::ColorChoice, safe_exit, SUCCESS_CODE, USAGE_CODE};
use crate::AppSettings;
use crate::Command;

mod context;
mod kind;

pub use context::ContextKind;
pub use context::ContextValue;
pub use kind::ErrorKind;

/// Short hand for [`Result`] type
///
/// [`Result`]: std::result::Result
pub type Result<T, E = Error> = StdResult<T, E>;

/// Command Line Argument Parser Error
///
/// See [`Command::error`] to create an error.
///
/// [`Command::error`]: crate::Command::error
#[derive(Debug)]
pub struct Error {
    inner: Box<ErrorInner>,
    /// Deprecated, replaced with [`Error::kind()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.0", note = "Replaced with `Error::kind()`")
    )]
    pub kind: ErrorKind,
    /// Deprecated, replaced with [`Error::context()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.0", note = "Replaced with `Error::context()`")
    )]
    pub info: Vec<String>,
}

#[derive(Debug)]
struct ErrorInner {
    kind: ErrorKind,
    context: Vec<(ContextKind, ContextValue)>,
    message: Option<Message>,
    source: Option<Box<dyn error::Error + Send + Sync>>,
    help_flag: Option<&'static str>,
    color_when: ColorChoice,
    wait_on_exit: bool,
    backtrace: Option<Backtrace>,
}

impl Error {
    /// Create an unformatted error
    ///
    /// This is for you need to pass the error up to
    /// a place that has access to the `Command` at which point you can call [`Error::format`].
    ///
    /// Prefer [`Command::error`] for generating errors.
    ///
    /// [`Command::error`]: crate::Command::error
    pub fn raw(kind: ErrorKind, message: impl std::fmt::Display) -> Self {
        Self::new(kind).set_message(message.to_string())
    }

    /// Format the existing message with the Command's context
    #[must_use]
    pub fn format(mut self, cmd: &mut Command) -> Self {
        cmd._build_self();
        let usage = cmd.render_usage();
        if let Some(message) = self.inner.message.as_mut() {
            message.format(cmd, usage);
        }
        self.with_cmd(cmd)
    }

    /// Type of error for programmatic processing
    pub fn kind(&self) -> ErrorKind {
        self.inner.kind
    }

    /// Additional information to further qualify the error
    pub fn context(&self) -> impl Iterator<Item = (ContextKind, &ContextValue)> {
        self.inner.context.iter().map(|(k, v)| (*k, v))
    }

    /// Should the message be written to `stdout` or not?
    #[inline]
    pub fn use_stderr(&self) -> bool {
        self.stream() == Stream::Stderr
    }

    pub(crate) fn stream(&self) -> Stream {
        match self.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Stream::Stdout,
            _ => Stream::Stderr,
        }
    }

    /// Prints the error and exits.
    ///
    /// Depending on the error kind, this either prints to `stderr` and exits with a status of `2`
    /// or prints to `stdout` and exits with a status of `0`.
    pub fn exit(&self) -> ! {
        if self.use_stderr() {
            // Swallow broken pipe errors
            let _ = self.print();

            if self.inner.wait_on_exit {
                wlnerr!("\nPress [ENTER] / [RETURN] to continue...");
                let mut s = String::new();
                let i = io::stdin();
                i.lock().read_line(&mut s).unwrap();
            }

            safe_exit(USAGE_CODE);
        }

        // Swallow broken pipe errors
        let _ = self.print();
        safe_exit(SUCCESS_CODE)
    }

    /// Prints formatted and colored error to `stdout` or `stderr` according to its error kind
    ///
    /// # Example
    /// ```no_run
    /// use clap::Command;
    ///
    /// match Command::new("Command").try_get_matches() {
    ///     Ok(matches) => {
    ///         // do_something
    ///     },
    ///     Err(err) => {
    ///         err.print().expect("Error writing Error");
    ///         // do_something
    ///     },
    /// };
    /// ```
    pub fn print(&self) -> io::Result<()> {
        self.formatted().print()
    }

    /// Deprecated, replaced with [`Command::error`]
    ///
    /// [`Command::error`]: crate::Command::error
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `Command::error`")
    )]
    #[doc(hidden)]
    pub fn with_description(description: String, kind: ErrorKind) -> Self {
        Error::raw(kind, description)
    }

    fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                kind,
                context: Vec::new(),
                message: None,
                source: None,
                help_flag: None,
                color_when: ColorChoice::Never,
                wait_on_exit: false,
                backtrace: Backtrace::new(),
            }),
            kind,
            info: vec![],
        }
    }

    #[inline(never)]
    fn for_app(kind: ErrorKind, cmd: &Command, colorizer: Colorizer, info: Vec<String>) -> Self {
        Self::new(kind)
            .set_message(colorizer)
            .with_cmd(cmd)
            .set_info(info)
    }

    pub(crate) fn with_cmd(self, cmd: &Command) -> Self {
        self.set_wait_on_exit(cmd.is_set(AppSettings::WaitOnError))
            .set_color(cmd.get_color())
            .set_help_flag(get_help_flag(cmd))
    }

    pub(crate) fn set_message(mut self, message: impl Into<Message>) -> Self {
        self.inner.message = Some(message.into());
        self
    }

    pub(crate) fn set_info(mut self, info: Vec<String>) -> Self {
        self.info = info;
        self
    }

    pub(crate) fn set_source(mut self, source: Box<dyn error::Error + Send + Sync>) -> Self {
        self.inner.source = Some(source);
        self
    }

    pub(crate) fn set_color(mut self, color_when: ColorChoice) -> Self {
        self.inner.color_when = color_when;
        self
    }

    pub(crate) fn set_help_flag(mut self, help_flag: Option<&'static str>) -> Self {
        self.inner.help_flag = help_flag;
        self
    }

    pub(crate) fn set_wait_on_exit(mut self, yes: bool) -> Self {
        self.inner.wait_on_exit = yes;
        self
    }

    /// Does not verify if `ContextKind` is already present
    #[inline(never)]
    pub(crate) fn insert_context_unchecked(
        mut self,
        kind: ContextKind,
        value: ContextValue,
    ) -> Self {
        self.inner.context.push((kind, value));
        self
    }

    /// Does not verify if `ContextKind` is already present
    #[inline(never)]
    pub(crate) fn extend_context_unchecked<const N: usize>(
        mut self,
        context: [(ContextKind, ContextValue); N],
    ) -> Self {
        self.inner.context.extend(context);
        self
    }

    #[inline(never)]
    fn get_context(&self, kind: ContextKind) -> Option<&ContextValue> {
        self.inner
            .context
            .iter()
            .find_map(|(k, v)| (*k == kind).then(|| v))
    }

    pub(crate) fn display_help(cmd: &Command, colorizer: Colorizer) -> Self {
        Self::for_app(ErrorKind::DisplayHelp, cmd, colorizer, vec![])
    }

    pub(crate) fn display_help_error(cmd: &Command, colorizer: Colorizer) -> Self {
        Self::for_app(
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
            cmd,
            colorizer,
            vec![],
        )
    }

    pub(crate) fn display_version(cmd: &Command, colorizer: Colorizer) -> Self {
        Self::for_app(ErrorKind::DisplayVersion, cmd, colorizer, vec![])
    }

    pub(crate) fn argument_conflict(
        cmd: &Command,
        arg: String,
        mut others: Vec<String>,
        usage: String,
    ) -> Self {
        let info = others.clone();
        let others = match others.len() {
            0 => ContextValue::None,
            1 => ContextValue::String(others.pop().unwrap()),
            _ => ContextValue::Strings(others),
        };
        Self::new(ErrorKind::ArgumentConflict)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::PriorArg, others),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn empty_value(cmd: &Command, good_vals: &[&str], arg: String) -> Self {
        let info = vec![arg.clone()];
        let mut err = Self::new(ErrorKind::EmptyValue)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([(ContextKind::InvalidArg, ContextValue::String(arg))]);
        if !good_vals.is_empty() {
            err = err.insert_context_unchecked(
                ContextKind::ValidValue,
                ContextValue::Strings(good_vals.iter().map(|s| (*s).to_owned()).collect()),
            );
        }
        err
    }

    pub(crate) fn no_equals(cmd: &Command, arg: String, usage: String) -> Self {
        let info = vec![arg.clone()];
        Self::new(ErrorKind::NoEquals)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn invalid_value(
        cmd: &Command,
        bad_val: String,
        good_vals: &[&str],
        arg: String,
    ) -> Self {
        let mut info = vec![arg.clone(), bad_val.clone()];
        info.extend(good_vals.iter().map(|s| (*s).to_owned()));

        let suggestion = suggestions::did_you_mean(&bad_val, good_vals.iter()).pop();
        let mut err = Self::new(ErrorKind::InvalidValue)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::InvalidValue, ContextValue::String(bad_val)),
                (
                    ContextKind::ValidValue,
                    ContextValue::Strings(good_vals.iter().map(|s| (*s).to_owned()).collect()),
                ),
            ]);
        if let Some(suggestion) = suggestion {
            err = err.insert_context_unchecked(
                ContextKind::SuggestedValue,
                ContextValue::String(suggestion),
            );
        }
        err
    }

    pub(crate) fn invalid_subcommand(
        cmd: &Command,
        subcmd: String,
        did_you_mean: String,
        name: String,
        usage: String,
    ) -> Self {
        let info = vec![subcmd.clone()];
        let suggestion = format!("{} -- {}", name, subcmd);
        Self::new(ErrorKind::InvalidSubcommand)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidSubcommand, ContextValue::String(subcmd)),
                (
                    ContextKind::SuggestedSubcommand,
                    ContextValue::String(did_you_mean),
                ),
                (
                    ContextKind::SuggestedCommand,
                    ContextValue::String(suggestion),
                ),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn unrecognized_subcommand(cmd: &Command, subcmd: String, usage: String) -> Self {
        let info = vec![subcmd.clone()];
        Self::new(ErrorKind::UnrecognizedSubcommand)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidSubcommand, ContextValue::String(subcmd)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn missing_required_argument(
        cmd: &Command,
        required: Vec<String>,
        usage: String,
    ) -> Self {
        let info = required.clone();
        Self::new(ErrorKind::MissingRequiredArgument)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::Strings(required)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn missing_subcommand(cmd: &Command, name: String, usage: String) -> Self {
        let info = vec![];
        Self::new(ErrorKind::MissingSubcommand)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidSubcommand, ContextValue::String(name)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn invalid_utf8(cmd: &Command, usage: String) -> Self {
        let info = vec![];
        Self::new(ErrorKind::InvalidUtf8)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([(ContextKind::Usage, ContextValue::String(usage))])
    }

    pub(crate) fn too_many_occurrences(
        cmd: &Command,
        arg: String,
        max_occurs: usize,
        curr_occurs: usize,
        usage: String,
    ) -> Self {
        let info = vec![arg.clone(), curr_occurs.to_string(), max_occurs.to_string()];
        Self::new(ErrorKind::TooManyOccurrences)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (
                    ContextKind::MaxOccurrences,
                    ContextValue::Number(max_occurs as isize),
                ),
                (
                    ContextKind::ActualNumValues,
                    ContextValue::Number(curr_occurs as isize),
                ),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn too_many_values(cmd: &Command, val: String, arg: String, usage: String) -> Self {
        let info = vec![arg.clone(), val.clone()];
        Self::new(ErrorKind::TooManyValues)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::InvalidValue, ContextValue::String(val)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn too_few_values(
        cmd: &Command,
        arg: String,
        min_vals: usize,
        curr_vals: usize,
        usage: String,
    ) -> Self {
        let info = vec![arg.clone(), curr_vals.to_string(), min_vals.to_string()];
        Self::new(ErrorKind::TooFewValues)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (
                    ContextKind::MinValues,
                    ContextValue::Number(min_vals as isize),
                ),
                (
                    ContextKind::ActualNumValues,
                    ContextValue::Number(curr_vals as isize),
                ),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn value_validation(
        arg: String,
        val: String,
        err: Box<dyn error::Error + Send + Sync>,
    ) -> Self {
        let info = vec![arg.clone(), val.to_string(), err.to_string()];
        Self::new(ErrorKind::ValueValidation)
            .set_info(info)
            .set_source(err)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::InvalidValue, ContextValue::String(val)),
            ])
    }

    pub(crate) fn wrong_number_of_values(
        cmd: &Command,
        arg: String,
        num_vals: usize,
        curr_vals: usize,
        usage: String,
    ) -> Self {
        let info = vec![arg.clone(), curr_vals.to_string(), num_vals.to_string()];
        Self::new(ErrorKind::WrongNumberOfValues)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (
                    ContextKind::ExpectedNumValues,
                    ContextValue::Number(num_vals as isize),
                ),
                (
                    ContextKind::ActualNumValues,
                    ContextValue::Number(curr_vals as isize),
                ),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn unexpected_multiple_usage(cmd: &Command, arg: String, usage: String) -> Self {
        let info = vec![arg.clone()];
        Self::new(ErrorKind::UnexpectedMultipleUsage)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn unknown_argument(
        cmd: &Command,
        arg: String,
        did_you_mean: Option<(String, Option<String>)>,
        usage: String,
    ) -> Self {
        let info = vec![arg.clone()];
        let mut err = Self::new(ErrorKind::UnknownArgument)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ]);
        if let Some((flag, sub)) = did_you_mean {
            err = err.insert_context_unchecked(
                ContextKind::SuggestedArg,
                ContextValue::String(format!("--{}", flag)),
            );
            if let Some(sub) = sub {
                err = err.insert_context_unchecked(
                    ContextKind::SuggestedSubcommand,
                    ContextValue::String(sub),
                );
            }
        }
        err
    }

    pub(crate) fn unnecessary_double_dash(cmd: &Command, arg: String, usage: String) -> Self {
        let info = vec![arg.clone()];
        Self::new(ErrorKind::UnknownArgument)
            .with_cmd(cmd)
            .set_info(info)
            .extend_context_unchecked([
                (ContextKind::InvalidArg, ContextValue::String(arg)),
                (ContextKind::TrailingArg, ContextValue::Bool(true)),
                (ContextKind::Usage, ContextValue::String(usage)),
            ])
    }

    pub(crate) fn argument_not_found_auto(arg: String) -> Self {
        let info = vec![arg.clone()];
        Self::new(ErrorKind::ArgumentNotFound)
            .set_info(info)
            .extend_context_unchecked([(ContextKind::InvalidArg, ContextValue::String(arg))])
    }

    fn formatted(&self) -> Cow<'_, Colorizer> {
        if let Some(message) = self.inner.message.as_ref() {
            message.formatted()
        } else {
            let mut c = Colorizer::new(self.stream(), self.inner.color_when);

            start_error(&mut c);

            if !self.write_dynamic_context(&mut c) {
                if let Some(msg) = self.kind().as_str() {
                    c.none(msg.to_owned());
                } else if let Some(source) = self.inner.source.as_ref() {
                    c.none(source.to_string());
                } else {
                    c.none("Unknown cause");
                }
            }

            let usage = self.get_context(ContextKind::Usage);
            if let Some(ContextValue::String(usage)) = usage {
                put_usage(&mut c, usage);
            }

            try_help(&mut c, self.inner.help_flag);

            Cow::Owned(c)
        }
    }

    #[must_use]
    fn write_dynamic_context(&self, c: &mut Colorizer) -> bool {
        match self.kind() {
            ErrorKind::ArgumentConflict => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let prior_arg = self.get_context(ContextKind::PriorArg);
                if let (Some(ContextValue::String(invalid_arg)), Some(prior_arg)) =
                    (invalid_arg, prior_arg)
                {
                    c.none("The argument '");
                    c.warning(invalid_arg);
                    c.none("' cannot be used with");

                    match prior_arg {
                        ContextValue::Strings(values) => {
                            c.none(":");
                            for v in values {
                                c.none("\n    ");
                                c.warning(&**v);
                            }
                        }
                        ContextValue::String(value) => {
                            c.none(" '");
                            c.warning(value);
                            c.none("'");
                        }
                        _ => {
                            c.none(" one or more of the other specified arguments");
                        }
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::EmptyValue => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                    c.none("The argument '");
                    c.warning(invalid_arg);
                    c.none("' requires a value but none was supplied");

                    let possible_values = self.get_context(ContextKind::ValidValue);
                    if let Some(ContextValue::Strings(possible_values)) = possible_values {
                        c.none("\n\t[possible values: ");
                        if let Some((last, elements)) = possible_values.split_last() {
                            for v in elements {
                                c.good(escape(v));
                                c.none(", ");
                            }
                            c.good(escape(last));
                        }
                        c.none("]");
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::NoEquals => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                    c.none("Equal sign is needed when assigning values to '");
                    c.warning(invalid_arg);
                    c.none("'.");
                    true
                } else {
                    false
                }
            }
            ErrorKind::InvalidValue => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let invalid_value = self.get_context(ContextKind::InvalidValue);
                if let (
                    Some(ContextValue::String(invalid_arg)),
                    Some(ContextValue::String(invalid_value)),
                ) = (invalid_arg, invalid_value)
                {
                    c.none(quote(invalid_value));
                    c.none(" isn't a valid value for '");
                    c.warning(invalid_arg);
                    c.none("'");

                    let possible_values = self.get_context(ContextKind::ValidValue);
                    if let Some(ContextValue::Strings(possible_values)) = possible_values {
                        c.none("\n\t[possible values: ");
                        if let Some((last, elements)) = possible_values.split_last() {
                            for v in elements {
                                c.good(escape(v));
                                c.none(", ");
                            }
                            c.good(escape(last));
                        }
                        c.none("]");
                    }

                    let suggestion = self.get_context(ContextKind::SuggestedValue);
                    if let Some(ContextValue::String(suggestion)) = suggestion {
                        c.none("\n\n\tDid you mean ");
                        c.good(quote(suggestion));
                        c.none("?");
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::InvalidSubcommand => {
                let invalid_sub = self.get_context(ContextKind::InvalidSubcommand);
                if let Some(ContextValue::String(invalid_sub)) = invalid_sub {
                    c.none("The subcommand '");
                    c.warning(invalid_sub);
                    c.none("' wasn't recognized");

                    let valid_sub = self.get_context(ContextKind::SuggestedSubcommand);
                    if let Some(ContextValue::String(valid_sub)) = valid_sub {
                        c.none("\n\n\tDid you mean ");
                        c.good(valid_sub);
                        c.none("?");
                    }

                    let suggestion = self.get_context(ContextKind::SuggestedCommand);
                    if let Some(ContextValue::String(suggestion)) = suggestion {
                        c.none(
            "\n\nIf you believe you received this message in error, try re-running with '",
        );
                        c.good(suggestion);
                        c.none("'");
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::UnrecognizedSubcommand => {
                let invalid_sub = self.get_context(ContextKind::InvalidSubcommand);
                if let Some(ContextValue::String(invalid_sub)) = invalid_sub {
                    c.none("The subcommand '");
                    c.warning(invalid_sub);
                    c.none("' wasn't recognized");
                    true
                } else {
                    false
                }
            }
            ErrorKind::MissingRequiredArgument => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                if let Some(ContextValue::Strings(invalid_arg)) = invalid_arg {
                    c.none("The following required arguments were not provided:");
                    for v in invalid_arg {
                        c.none("\n    ");
                        c.good(&**v);
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::MissingSubcommand => {
                let invalid_sub = self.get_context(ContextKind::InvalidSubcommand);
                if let Some(ContextValue::String(invalid_sub)) = invalid_sub {
                    c.none("'");
                    c.warning(invalid_sub);
                    c.none("' requires a subcommand but one was not provided");
                    true
                } else {
                    false
                }
            }
            ErrorKind::InvalidUtf8 => false,
            ErrorKind::TooManyOccurrences => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let actual_num_occurs = self.get_context(ContextKind::ActualNumOccurrences);
                let max_occurs = self.get_context(ContextKind::MaxOccurrences);
                if let (
                    Some(ContextValue::String(invalid_arg)),
                    Some(ContextValue::Number(actual_num_occurs)),
                    Some(ContextValue::Number(max_occurs)),
                ) = (invalid_arg, actual_num_occurs, max_occurs)
                {
                    let were_provided = Error::singular_or_plural(*actual_num_occurs as usize);
                    c.none("The argument '");
                    c.warning(invalid_arg);
                    c.none("' allows at most ");
                    c.warning(max_occurs.to_string());
                    c.none(" occurrences but ");
                    c.warning(actual_num_occurs.to_string());
                    c.none(were_provided);
                    true
                } else {
                    false
                }
            }
            ErrorKind::TooManyValues => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let invalid_value = self.get_context(ContextKind::InvalidValue);
                if let (
                    Some(ContextValue::String(invalid_arg)),
                    Some(ContextValue::String(invalid_value)),
                ) = (invalid_arg, invalid_value)
                {
                    c.none("The value '");
                    c.warning(invalid_value);
                    c.none("' was provided to '");
                    c.warning(invalid_arg);
                    c.none("' but it wasn't expecting any more values");
                    true
                } else {
                    false
                }
            }
            ErrorKind::TooFewValues => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let actual_num_values = self.get_context(ContextKind::ActualNumValues);
                let min_values = self.get_context(ContextKind::MinValues);
                if let (
                    Some(ContextValue::String(invalid_arg)),
                    Some(ContextValue::Number(actual_num_values)),
                    Some(ContextValue::Number(min_values)),
                ) = (invalid_arg, actual_num_values, min_values)
                {
                    let were_provided = Error::singular_or_plural(*actual_num_values as usize);
                    c.none("The argument '");
                    c.warning(invalid_arg);
                    c.none("' requires at least ");
                    c.warning(min_values.to_string());
                    c.none(" values but only ");
                    c.warning(actual_num_values.to_string());
                    c.none(were_provided);
                    true
                } else {
                    false
                }
            }
            ErrorKind::ValueValidation => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let invalid_value = self.get_context(ContextKind::InvalidValue);
                if let (
                    Some(ContextValue::String(invalid_arg)),
                    Some(ContextValue::String(invalid_value)),
                ) = (invalid_arg, invalid_value)
                {
                    c.none("Invalid value ");
                    c.warning(quote(invalid_value));
                    c.none(" for '");
                    c.warning(invalid_arg);
                    if let Some(source) = self.inner.source.as_deref() {
                        c.none("': ");
                        c.none(source.to_string());
                    } else {
                        c.none("'");
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::WrongNumberOfValues => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                let actual_num_values = self.get_context(ContextKind::ActualNumValues);
                let num_values = self.get_context(ContextKind::ExpectedNumValues);
                if let (
                    Some(ContextValue::String(invalid_arg)),
                    Some(ContextValue::Number(actual_num_values)),
                    Some(ContextValue::Number(num_values)),
                ) = (invalid_arg, actual_num_values, num_values)
                {
                    let were_provided = Error::singular_or_plural(*actual_num_values as usize);
                    c.none("The argument '");
                    c.warning(invalid_arg);
                    c.none("' requires ");
                    c.warning(num_values.to_string());
                    c.none(" values, but ");
                    c.warning(actual_num_values.to_string());
                    c.none(were_provided);
                    true
                } else {
                    false
                }
            }
            ErrorKind::UnexpectedMultipleUsage => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                    c.none("The argument '");
                    c.warning(invalid_arg.to_string());
                    c.none("' was provided more than once, but cannot be used multiple times");
                    true
                } else {
                    false
                }
            }
            ErrorKind::UnknownArgument => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                    c.none("Found argument '");
                    c.warning(invalid_arg.to_string());
                    c.none("' which wasn't expected, or isn't valid in this context");

                    let valid_sub = self.get_context(ContextKind::SuggestedSubcommand);
                    let valid_arg = self.get_context(ContextKind::SuggestedArg);
                    match (valid_sub, valid_arg) {
                        (
                            Some(ContextValue::String(valid_sub)),
                            Some(ContextValue::String(valid_arg)),
                        ) => {
                            c.none("\n\n\tDid you mean ");
                            c.none("to put '");
                            c.good(valid_arg);
                            c.none("' after the subcommand '");
                            c.good(valid_sub);
                            c.none("'?");
                        }
                        (None, Some(ContextValue::String(valid_arg))) => {
                            c.none("\n\n\tDid you mean '");
                            c.good(valid_arg);
                            c.none("'?");
                        }
                        (_, _) => {}
                    }

                    let invalid_arg = self.get_context(ContextKind::InvalidArg);
                    if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                        if invalid_arg.starts_with('-') {
                            c.none(format!(
                                "\n\n\tIf you tried to supply `{}` as a value rather than a flag, use `-- {}`",
                                invalid_arg, invalid_arg
                            ));
                        }

                        let trailing_arg = self.get_context(ContextKind::TrailingArg);
                        if trailing_arg == Some(&ContextValue::Bool(true)) {
                            c.none(format!(
                            "\n\n\tIf you tried to supply `{}` as a subcommand, remove the '--' before it.",
                            invalid_arg
                        ));
                        }
                    }
                    true
                } else {
                    false
                }
            }
            ErrorKind::ArgumentNotFound => {
                let invalid_arg = self.get_context(ContextKind::InvalidArg);
                if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                    c.none("The argument '");
                    c.warning(invalid_arg.to_string());
                    c.none("' wasn't found");
                    true
                } else {
                    false
                }
            }
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | ErrorKind::DisplayVersion
            | ErrorKind::Io
            | ErrorKind::Format => false,
        }
    }

    /// Returns the singular or plural form on the verb to be based on the argument's value.
    fn singular_or_plural(n: usize) -> &'static str {
        if n > 1 {
            " were provided"
        } else {
            " was provided"
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::raw(ErrorKind::Io, e)
    }
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Self {
        Error::raw(ErrorKind::Format, e)
    }
}

impl error::Error for Error {
    #[allow(trivial_casts)]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| e.as_ref() as _)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Assuming `self.message` already has a trailing newline, from `try_help` or similar
        write!(f, "{}", self.formatted())?;
        if let Some(backtrace) = self.inner.backtrace.as_ref() {
            writeln!(f)?;
            writeln!(f, "Backtrace:")?;
            writeln!(f, "{}", backtrace)?;
        }
        Ok(())
    }
}

fn start_error(c: &mut Colorizer) {
    c.error("error:");
    c.none(" ");
}

fn put_usage(c: &mut Colorizer, usage: impl Into<String>) {
    c.none("\n\n");
    c.none(usage);
}

fn get_help_flag(cmd: &Command) -> Option<&'static str> {
    if !cmd.is_disable_help_flag_set() {
        Some("--help")
    } else if cmd.has_subcommands() && !cmd.is_disable_help_subcommand_set() {
        Some("help")
    } else {
        None
    }
}

fn try_help(c: &mut Colorizer, help: Option<&str>) {
    if let Some(help) = help {
        c.none("\n\nFor more information try ");
        c.good(help);
        c.none("\n");
    } else {
        c.none("\n");
    }
}

fn quote(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    format!("{:?}", s)
}

fn escape(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    if s.contains(char::is_whitespace) {
        quote(s)
    } else {
        s.to_owned()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    Raw(String),
    Formatted(Colorizer),
}

impl Message {
    fn format(&mut self, cmd: &Command, usage: String) {
        match self {
            Message::Raw(s) => {
                let mut c = Colorizer::new(Stream::Stderr, cmd.get_color());

                let mut message = String::new();
                std::mem::swap(s, &mut message);
                start_error(&mut c);
                c.none(message);
                put_usage(&mut c, usage);
                try_help(&mut c, get_help_flag(cmd));
                *self = Self::Formatted(c);
            }
            Message::Formatted(_) => {}
        }
    }

    fn formatted(&self) -> Cow<Colorizer> {
        match self {
            Message::Raw(s) => {
                let mut c = Colorizer::new(Stream::Stderr, ColorChoice::Never);
                start_error(&mut c);
                c.none(s);
                Cow::Owned(c)
            }
            Message::Formatted(c) => Cow::Borrowed(c),
        }
    }
}

impl From<String> for Message {
    fn from(inner: String) -> Self {
        Self::Raw(inner)
    }
}

impl From<Colorizer> for Message {
    fn from(inner: Colorizer) -> Self {
        Self::Formatted(inner)
    }
}

#[cfg(feature = "debug")]
#[derive(Debug)]
struct Backtrace(backtrace::Backtrace);

#[cfg(feature = "debug")]
impl Backtrace {
    fn new() -> Option<Self> {
        Some(Self(backtrace::Backtrace::new()))
    }
}

#[cfg(feature = "debug")]
impl Display for Backtrace {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // `backtrace::Backtrace` uses `Debug` instead of `Display`
        write!(f, "{:?}", self.0)
    }
}

#[cfg(not(feature = "debug"))]
#[derive(Debug)]
struct Backtrace;

#[cfg(not(feature = "debug"))]
impl Backtrace {
    fn new() -> Option<Self> {
        None
    }
}

#[cfg(not(feature = "debug"))]
impl Display for Backtrace {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}

#[test]
fn check_auto_traits() {
    static_assertions::assert_impl_all!(Error: Send, Sync, Unpin);
}
