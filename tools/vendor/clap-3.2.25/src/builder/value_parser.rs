use std::convert::TryInto;
use std::ops::RangeBounds;

use crate::parser::AnyValue;
use crate::parser::AnyValueId;

/// Parse/validate argument values
///
/// Specified with [`Arg::value_parser`][crate::Arg::value_parser].
///
/// `ValueParser` defines how to convert a raw argument value into a validated and typed value for
/// use within an application.
///
/// See
/// - [`value_parser!`][crate::value_parser] for automatically selecting an implementation for a given type
/// - [`ValueParser::new`] for additional [`TypedValueParser`] that can be used
///
/// # Example
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("color")
///             .long("color")
///             .value_parser(["always", "auto", "never"])
///             .default_value("auto")
///     )
///     .arg(
///         clap::Arg::new("hostname")
///             .long("hostname")
///             .value_parser(clap::builder::NonEmptyStringValueParser::new())
///             .takes_value(true)
///             .required(true)
///     )
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(clap::value_parser!(u16).range(3000..))
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(
///     ["cmd", "--hostname", "rust-lang.org", "--port", "3001"]
/// ).unwrap();
///
/// let color: &String = m.get_one("color")
///     .expect("default");
/// assert_eq!(color, "auto");
///
/// let hostname: &String = m.get_one("hostname")
///     .expect("required");
/// assert_eq!(hostname, "rust-lang.org");
///
/// let port: u16 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
pub struct ValueParser(ValueParserInner);

enum ValueParserInner {
    // Common enough to optimize and for possible values
    Bool,
    // Common enough to optimize
    String,
    // Common enough to optimize
    OsString,
    // Common enough to optimize
    PathBuf,
    Other(Box<dyn AnyValueParser>),
}

impl ValueParser {
    /// Custom parser for argument values
    ///
    /// To create a custom parser, see [`TypedValueParser`]
    ///
    /// Pre-existing implementations include:
    /// - [`EnumValueParser`] and  [`PossibleValuesParser`] for static enumerated values
    /// - [`BoolishValueParser`] and [`FalseyValueParser`] for alternative `bool` implementations
    /// - [`RangedI64ValueParser`] and [`RangedU64ValueParser`]
    /// - [`NonEmptyStringValueParser`]
    ///
    /// # Example
    ///
    /// ```rust
    /// type EnvVar = (String, Option<String>);
    /// fn parse_env_var(env: &str) -> Result<EnvVar, std::io::Error> {
    ///     if let Some((var, value)) = env.split_once('=') {
    ///         Ok((var.to_owned(), Some(value.to_owned())))
    ///     } else {
    ///         Ok((env.to_owned(), None))
    ///     }
    /// }
    ///
    /// let mut cmd = clap::Command::new("raw")
    ///     .arg(
    ///         clap::Arg::new("env")
    ///             .value_parser(clap::builder::ValueParser::new(parse_env_var))
    ///             .required(true)
    ///     );
    ///
    /// let m = cmd.try_get_matches_from_mut(["cmd", "key=value"]).unwrap();
    /// let port: &EnvVar = m.get_one("env")
    ///     .expect("required");
    /// assert_eq!(*port, ("key".into(), Some("value".into())));
    /// ```
    pub fn new<P>(other: P) -> Self
    where
        P: TypedValueParser,
        P::Value: Send + Sync + Clone,
    {
        Self(ValueParserInner::Other(Box::new(other)))
    }

    /// [`bool`] parser for argument values
    ///
    /// See also:
    /// - [`BoolishValueParser`] for different human readable bool representations
    /// - [`FalseyValueParser`] for assuming non-false is true
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cmd = clap::Command::new("raw")
    ///     .arg(
    ///         clap::Arg::new("download")
    ///             .value_parser(clap::value_parser!(bool))
    ///             .required(true)
    ///     );
    ///
    /// let m = cmd.try_get_matches_from_mut(["cmd", "true"]).unwrap();
    /// let port: bool = *m.get_one("download")
    ///     .expect("required");
    /// assert_eq!(port, true);
    ///
    /// assert!(cmd.try_get_matches_from_mut(["cmd", "forever"]).is_err());
    /// ```
    pub const fn bool() -> Self {
        Self(ValueParserInner::Bool)
    }

    /// [`String`] parser for argument values
    ///
    /// See also:
    /// - [`NonEmptyStringValueParser`]
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cmd = clap::Command::new("raw")
    ///     .arg(
    ///         clap::Arg::new("port")
    ///             .value_parser(clap::value_parser!(String))
    ///             .required(true)
    ///     );
    ///
    /// let m = cmd.try_get_matches_from_mut(["cmd", "80"]).unwrap();
    /// let port: &String = m.get_one("port")
    ///     .expect("required");
    /// assert_eq!(port, "80");
    /// ```
    pub const fn string() -> Self {
        Self(ValueParserInner::String)
    }

    /// [`OsString`][std::ffi::OsString] parser for argument values
    ///
    /// # Example
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```rust")]
    /// # use clap::{Command, Arg, builder::ValueParser};
    /// use std::ffi::OsString;
    /// use std::os::unix::ffi::{OsStrExt,OsStringExt};
    /// let r = Command::new("myprog")
    ///     .arg(
    ///         Arg::new("arg")
    ///         .required(true)
    ///         .value_parser(ValueParser::os_string())
    ///     )
    ///     .try_get_matches_from(vec![
    ///         OsString::from("myprog"),
    ///         OsString::from_vec(vec![0xe9])
    ///     ]);
    ///
    /// assert!(r.is_ok());
    /// let m = r.unwrap();
    /// let arg: &OsString = m.get_one("arg")
    ///     .expect("required");
    /// assert_eq!(arg.as_bytes(), &[0xe9]);
    /// ```
    pub const fn os_string() -> Self {
        Self(ValueParserInner::OsString)
    }

    /// [`PathBuf`][std::path::PathBuf] parser for argument values
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use std::path::Path;
    /// let mut cmd = clap::Command::new("raw")
    ///     .arg(
    ///         clap::Arg::new("output")
    ///             .value_parser(clap::value_parser!(PathBuf))
    ///             .required(true)
    ///     );
    ///
    /// let m = cmd.try_get_matches_from_mut(["cmd", "hello.txt"]).unwrap();
    /// let port: &PathBuf = m.get_one("output")
    ///     .expect("required");
    /// assert_eq!(port, Path::new("hello.txt"));
    ///
    /// assert!(cmd.try_get_matches_from_mut(["cmd", ""]).is_err());
    /// ```
    pub const fn path_buf() -> Self {
        Self(ValueParserInner::PathBuf)
    }
}

impl ValueParser {
    /// Parse into a `AnyValue`
    ///
    /// When `arg` is `None`, an external subcommand value is being parsed.
    pub(crate) fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<AnyValue, crate::Error> {
        self.any_value_parser().parse_ref(cmd, arg, value)
    }

    /// Describes the content of `AnyValue`
    pub fn type_id(&self) -> AnyValueId {
        self.any_value_parser().type_id()
    }

    /// Reflect on enumerated value properties
    ///
    /// Error checking should not be done with this; it is mostly targeted at user-facing
    /// applications like errors and completion.
    pub fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        self.any_value_parser().possible_values()
    }

    fn any_value_parser(&self) -> &dyn AnyValueParser {
        match &self.0 {
            ValueParserInner::Bool => &BoolValueParser {},
            ValueParserInner::String => &StringValueParser {},
            ValueParserInner::OsString => &OsStringValueParser {},
            ValueParserInner::PathBuf => &PathBufValueParser {},
            ValueParserInner::Other(o) => o.as_ref(),
        }
    }
}

/// Convert a [`TypedValueParser`] to [`ValueParser`]
///
/// # Example
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("hostname")
///             .long("hostname")
///             .value_parser(clap::builder::NonEmptyStringValueParser::new())
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(
///     ["cmd", "--hostname", "rust-lang.org"]
/// ).unwrap();
///
/// let hostname: &String = m.get_one("hostname")
///     .expect("required");
/// assert_eq!(hostname, "rust-lang.org");
/// ```
impl<P> From<P> for ValueParser
where
    P: TypedValueParser + Send + Sync + 'static,
    P::Value: Send + Sync + Clone,
{
    fn from(p: P) -> Self {
        Self::new(p)
    }
}

impl From<_AnonymousValueParser> for ValueParser {
    fn from(p: _AnonymousValueParser) -> Self {
        p.0
    }
}

/// Create an `i64` [`ValueParser`] from a `N..M` range
///
/// See [`RangedI64ValueParser`] for more control over the output type.
///
/// See also [`RangedU64ValueParser`]
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(3000..4000)
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "3001"]).unwrap();
/// let port: i64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
impl From<std::ops::Range<i64>> for ValueParser {
    fn from(value: std::ops::Range<i64>) -> Self {
        let inner = RangedI64ValueParser::<i64>::new().range(value.start..value.end);
        Self::from(inner)
    }
}

/// Create an `i64` [`ValueParser`] from a `N..=M` range
///
/// See [`RangedI64ValueParser`] for more control over the output type.
///
/// See also [`RangedU64ValueParser`]
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(3000..=4000)
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "3001"]).unwrap();
/// let port: i64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
impl From<std::ops::RangeInclusive<i64>> for ValueParser {
    fn from(value: std::ops::RangeInclusive<i64>) -> Self {
        let inner = RangedI64ValueParser::<i64>::new().range(value.start()..=value.end());
        Self::from(inner)
    }
}

/// Create an `i64` [`ValueParser`] from a `N..` range
///
/// See [`RangedI64ValueParser`] for more control over the output type.
///
/// See also [`RangedU64ValueParser`]
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(3000..)
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "3001"]).unwrap();
/// let port: i64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
impl From<std::ops::RangeFrom<i64>> for ValueParser {
    fn from(value: std::ops::RangeFrom<i64>) -> Self {
        let inner = RangedI64ValueParser::<i64>::new().range(value.start..);
        Self::from(inner)
    }
}

/// Create an `i64` [`ValueParser`] from a `..M` range
///
/// See [`RangedI64ValueParser`] for more control over the output type.
///
/// See also [`RangedU64ValueParser`]
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(..3000)
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "80"]).unwrap();
/// let port: i64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 80);
/// ```
impl From<std::ops::RangeTo<i64>> for ValueParser {
    fn from(value: std::ops::RangeTo<i64>) -> Self {
        let inner = RangedI64ValueParser::<i64>::new().range(..value.end);
        Self::from(inner)
    }
}

/// Create an `i64` [`ValueParser`] from a `..=M` range
///
/// See [`RangedI64ValueParser`] for more control over the output type.
///
/// See also [`RangedU64ValueParser`]
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(..=3000)
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "80"]).unwrap();
/// let port: i64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 80);
/// ```
impl From<std::ops::RangeToInclusive<i64>> for ValueParser {
    fn from(value: std::ops::RangeToInclusive<i64>) -> Self {
        let inner = RangedI64ValueParser::<i64>::new().range(..=value.end);
        Self::from(inner)
    }
}

/// Create an `i64` [`ValueParser`] from a `..` range
///
/// See [`RangedI64ValueParser`] for more control over the output type.
///
/// See also [`RangedU64ValueParser`]
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(..)
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "3001"]).unwrap();
/// let port: i64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
impl From<std::ops::RangeFull> for ValueParser {
    fn from(value: std::ops::RangeFull) -> Self {
        let inner = RangedI64ValueParser::<i64>::new().range(value);
        Self::from(inner)
    }
}

/// Create a [`ValueParser`] with [`PossibleValuesParser`]
///
/// See [`PossibleValuesParser`] for more flexibility in creating the
/// [`PossibleValue`][crate::PossibleValue]s.
///
/// # Examples
///
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("color")
///             .long("color")
///             .value_parser(["always", "auto", "never"])
///             .default_value("auto")
///     );
///
/// let m = cmd.try_get_matches_from_mut(
///     ["cmd", "--color", "never"]
/// ).unwrap();
///
/// let color: &String = m.get_one("color")
///     .expect("default");
/// assert_eq!(color, "never");
/// ```
impl<P, const C: usize> From<[P; C]> for ValueParser
where
    P: Into<super::PossibleValue<'static>>,
{
    fn from(values: [P; C]) -> Self {
        let inner = PossibleValuesParser::from(values);
        Self::from(inner)
    }
}

impl std::fmt::Debug for ValueParser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self.0 {
            ValueParserInner::Bool => f.debug_struct("ValueParser::bool").finish(),
            ValueParserInner::String => f.debug_struct("ValueParser::string").finish(),
            ValueParserInner::OsString => f.debug_struct("ValueParser::os_string").finish(),
            ValueParserInner::PathBuf => f.debug_struct("ValueParser::path_buf").finish(),
            ValueParserInner::Other(o) => write!(f, "ValueParser::other({:?})", o.type_id()),
        }
    }
}

impl Clone for ValueParser {
    fn clone(&self) -> Self {
        Self(match &self.0 {
            ValueParserInner::Bool => ValueParserInner::Bool,
            ValueParserInner::String => ValueParserInner::String,
            ValueParserInner::OsString => ValueParserInner::OsString,
            ValueParserInner::PathBuf => ValueParserInner::PathBuf,
            ValueParserInner::Other(o) => ValueParserInner::Other(o.clone_any()),
        })
    }
}

/// A type-erased wrapper for [`TypedValueParser`].
trait AnyValueParser: Send + Sync + 'static {
    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<AnyValue, crate::Error>;

    fn parse(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<AnyValue, crate::Error>;

    /// Describes the content of `AnyValue`
    fn type_id(&self) -> AnyValueId;

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>>;

    fn clone_any(&self) -> Box<dyn AnyValueParser>;
}

impl<T, P> AnyValueParser for P
where
    T: std::any::Any + Clone + Send + Sync + 'static,
    P: TypedValueParser<Value = T>,
{
    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<AnyValue, crate::Error> {
        let value = TypedValueParser::parse_ref(self, cmd, arg, value)?;
        Ok(AnyValue::new(value))
    }

    fn parse(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<AnyValue, crate::Error> {
        let value = TypedValueParser::parse(self, cmd, arg, value)?;
        Ok(AnyValue::new(value))
    }

    fn type_id(&self) -> AnyValueId {
        AnyValueId::of::<T>()
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        P::possible_values(self)
    }

    fn clone_any(&self) -> Box<dyn AnyValueParser> {
        Box::new(self.clone())
    }
}

/// Parse/validate argument values
pub trait TypedValueParser: Clone + Send + Sync + 'static {
    /// Argument's value type
    type Value;

    /// Parse the argument value
    ///
    /// When `arg` is `None`, an external subcommand value is being parsed.
    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error>;

    /// Parse the argument value
    ///
    /// When `arg` is `None`, an external subcommand value is being parsed.
    fn parse(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, crate::Error> {
        self.parse_ref(cmd, arg, &value)
    }

    /// Reflect on enumerated value properties
    ///
    /// Error checking should not be done with this; it is mostly targeted at user-facing
    /// applications like errors and completion.
    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        None
    }

    /// Adapt a `TypedValueParser` from one value to another
    ///
    /// # Example
    ///
    /// ```rust
    /// # use clap::Command;
    /// # use clap::Arg;
    /// # use clap::builder::TypedValueParser as _;
    /// # use clap::builder::BoolishValueParser;
    /// let cmd = Command::new("mycmd")
    ///     .arg(
    ///         Arg::new("flag")
    ///             .long("flag")
    ///             .action(clap::ArgAction::Set)
    ///             .value_parser(
    ///                 BoolishValueParser::new()
    ///                 .map(|b| -> usize {
    ///                     if b { 10 } else { 5 }
    ///                 })
    ///             )
    ///     );
    ///
    /// let matches = cmd.clone().try_get_matches_from(["mycmd", "--flag=true", "--flag=true"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(
    ///     matches.get_one::<usize>("flag").copied(),
    ///     Some(10)
    /// );
    ///
    /// let matches = cmd.try_get_matches_from(["mycmd", "--flag=false"]).unwrap();
    /// assert!(matches.contains_id("flag"));
    /// assert_eq!(
    ///     matches.get_one::<usize>("flag").copied(),
    ///     Some(5)
    /// );
    /// ```
    fn map<T, F>(self, func: F) -> MapValueParser<Self, F>
    where
        T: Send + Sync + Clone,
        F: Fn(Self::Value) -> T + Clone,
    {
        MapValueParser::new(self, func)
    }
}

impl<F, T, E> TypedValueParser for F
where
    F: Fn(&str) -> Result<T, E> + Clone + Send + Sync + 'static,
    E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = value.to_str().ok_or_else(|| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        let value = (self)(value).map_err(|e| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            crate::Error::value_validation(arg, value.to_owned(), e.into()).with_cmd(cmd)
        })?;
        Ok(value)
    }
}

/// Implementation for [`ValueParser::string`]
///
/// Useful for composing new [`TypedValueParser`]s
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct StringValueParser {}

impl StringValueParser {
    /// Implementation for [`ValueParser::string`]
    pub fn new() -> Self {
        Self {}
    }
}

impl TypedValueParser for StringValueParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        cmd: &crate::Command,
        _arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, crate::Error> {
        let value = value.into_string().map_err(|_| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        Ok(value)
    }
}

impl Default for StringValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation for [`ValueParser::os_string`]
///
/// Useful for composing new [`TypedValueParser`]s
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct OsStringValueParser {}

impl OsStringValueParser {
    /// Implementation for [`ValueParser::os_string`]
    pub fn new() -> Self {
        Self {}
    }
}

impl TypedValueParser for OsStringValueParser {
    type Value = std::ffi::OsString;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        _cmd: &crate::Command,
        _arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, crate::Error> {
        Ok(value)
    }
}

impl Default for OsStringValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation for [`ValueParser::path_buf`]
///
/// Useful for composing new [`TypedValueParser`]s
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct PathBufValueParser {}

impl PathBufValueParser {
    /// Implementation for [`ValueParser::path_buf`]
    pub fn new() -> Self {
        Self {}
    }
}

impl TypedValueParser for PathBufValueParser {
    type Value = std::path::PathBuf;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, crate::Error> {
        if value.is_empty() {
            return Err(crate::Error::empty_value(
                cmd,
                &[],
                arg.map(ToString::to_string)
                    .unwrap_or_else(|| "...".to_owned()),
            ));
        }
        Ok(Self::Value::from(value))
    }
}

impl Default for PathBufValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse an [`ValueEnum`][crate::ValueEnum] value.
///
/// See also:
/// - [`PossibleValuesParser`]
///
/// # Example
///
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
///
/// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// enum ColorChoice {
///     Always,
///     Auto,
///     Never,
/// }
///
/// impl clap::ValueEnum for ColorChoice {
///     fn value_variants<'a>() -> &'a [Self] {
///         &[Self::Always, Self::Auto, Self::Never]
///     }
///
///     fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
///         match self {
///             Self::Always => Some(clap::PossibleValue::new("always")),
///             Self::Auto => Some(clap::PossibleValue::new("auto")),
///             Self::Never => Some(clap::PossibleValue::new("never")),
///         }
///     }
/// }
///
/// // Usage
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("color")
///             .value_parser(clap::builder::EnumValueParser::<ColorChoice>::new())
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "always"]).unwrap();
/// let port: ColorChoice = *m.get_one("color")
///     .expect("required");
/// assert_eq!(port, ColorChoice::Always);
///
/// // Semantics
/// let value_parser = clap::builder::EnumValueParser::<ColorChoice>::new();
/// // or
/// let value_parser = clap::value_parser!(ColorChoice);
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).is_err());
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("always")).unwrap(), ColorChoice::Always);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("auto")).unwrap(), ColorChoice::Auto);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("never")).unwrap(), ColorChoice::Never);
/// ```
#[derive(Clone, Debug)]
pub struct EnumValueParser<E: crate::ValueEnum + Clone + Send + Sync + 'static>(
    std::marker::PhantomData<E>,
);

impl<E: crate::ValueEnum + Clone + Send + Sync + 'static> EnumValueParser<E> {
    /// Parse an [`ValueEnum`][crate::ValueEnum]
    pub fn new() -> Self {
        let phantom: std::marker::PhantomData<E> = Default::default();
        Self(phantom)
    }
}

impl<E: crate::ValueEnum + Clone + Send + Sync + 'static> TypedValueParser for EnumValueParser<E> {
    type Value = E;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let ignore_case = arg.map(|a| a.is_ignore_case_set()).unwrap_or(false);
        let possible_vals = || {
            E::value_variants()
                .iter()
                .filter_map(|v| v.to_possible_value())
                .filter(|v| !v.is_hide_set())
                .map(|v| v.get_name())
                .collect::<Vec<_>>()
        };

        let value = value.to_str().ok_or_else(|| {
            crate::Error::invalid_value(
                cmd,
                value.to_string_lossy().into_owned(),
                &possible_vals(),
                arg.map(ToString::to_string)
                    .unwrap_or_else(|| "...".to_owned()),
            )
        })?;
        let value = E::value_variants()
            .iter()
            .find(|v| {
                v.to_possible_value()
                    .expect("ValueEnum::value_variants contains only values with a corresponding ValueEnum::to_possible_value")
                    .matches(value, ignore_case)
            })
            .ok_or_else(|| {
            crate::Error::invalid_value(
                cmd,
                value.to_owned(),
                &possible_vals(),
                arg.map(ToString::to_string)
                    .unwrap_or_else(|| "...".to_owned()),
            )
            })?
            .clone();
        Ok(value)
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        Some(Box::new(
            E::value_variants()
                .iter()
                .filter_map(|v| v.to_possible_value()),
        ))
    }
}

impl<E: crate::ValueEnum + Clone + Send + Sync + 'static> Default for EnumValueParser<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify the value is from an enumerated set of [`PossibleValue`][crate::PossibleValue].
///
/// See also:
/// - [`EnumValueParser`]
///
/// # Example
///
/// Usage:
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("color")
///             .value_parser(clap::builder::PossibleValuesParser::new(["always", "auto", "never"]))
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "always"]).unwrap();
/// let port: &String = m.get_one("color")
///     .expect("required");
/// assert_eq!(port, "always");
/// ```
///
/// Semantics:
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
/// let value_parser = clap::builder::PossibleValuesParser::new(["always", "auto", "never"]);
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).is_err());
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("always")).unwrap(), "always");
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("auto")).unwrap(), "auto");
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("never")).unwrap(), "never");
/// ```
#[derive(Clone, Debug)]
pub struct PossibleValuesParser(Vec<super::PossibleValue<'static>>);

impl PossibleValuesParser {
    /// Verify the value is from an enumerated set pf [`PossibleValue`][crate::PossibleValue].
    pub fn new(values: impl Into<PossibleValuesParser>) -> Self {
        values.into()
    }
}

impl TypedValueParser for PossibleValuesParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<String, crate::Error> {
        let value = value.into_string().map_err(|_| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;

        let ignore_case = arg.map(|a| a.is_ignore_case_set()).unwrap_or(false);
        if self.0.iter().any(|v| v.matches(&value, ignore_case)) {
            Ok(value)
        } else {
            let possible_vals = self
                .0
                .iter()
                .filter(|v| !v.is_hide_set())
                .map(crate::builder::PossibleValue::get_name)
                .collect::<Vec<_>>();

            Err(crate::Error::invalid_value(
                cmd,
                value,
                &possible_vals,
                arg.map(ToString::to_string)
                    .unwrap_or_else(|| "...".to_owned()),
            ))
        }
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        Some(Box::new(self.0.iter().cloned()))
    }
}

impl<I, T> From<I> for PossibleValuesParser
where
    I: IntoIterator<Item = T>,
    T: Into<super::PossibleValue<'static>>,
{
    fn from(values: I) -> Self {
        Self(values.into_iter().map(|t| t.into()).collect())
    }
}

/// Parse number that fall within a range of values
///
/// # Example
///
/// Usage:
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(clap::value_parser!(u16).range(3000..))
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "3001"]).unwrap();
/// let port: u16 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
///
/// Semantics:
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
/// let value_parser = clap::builder::RangedI64ValueParser::<i32>::new().range(-1..200);
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("-200")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("300")).is_err());
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("-1")).unwrap(), -1);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("0")).unwrap(), 0);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("50")).unwrap(), 50);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct RangedI64ValueParser<T: std::convert::TryFrom<i64> + Clone + Send + Sync = i64> {
    bounds: (std::ops::Bound<i64>, std::ops::Bound<i64>),
    target: std::marker::PhantomData<T>,
}

impl<T: std::convert::TryFrom<i64> + Clone + Send + Sync> RangedI64ValueParser<T> {
    /// Select full range of `i64`
    pub fn new() -> Self {
        Self::from(..)
    }

    /// Narrow the supported range
    pub fn range<B: RangeBounds<i64>>(mut self, range: B) -> Self {
        // Consideration: when the user does `value_parser!(u8).range()`
        // - Avoid programming mistakes by accidentally expanding the range
        // - Make it convenient to limit the range like with `..10`
        let start = match range.start_bound() {
            l @ std::ops::Bound::Included(i) => {
                debug_assert!(
                    self.bounds.contains(i),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            l @ std::ops::Bound::Excluded(i) => {
                debug_assert!(
                    self.bounds.contains(&i.saturating_add(1)),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            std::ops::Bound::Unbounded => self.bounds.start_bound().cloned(),
        };
        let end = match range.end_bound() {
            l @ std::ops::Bound::Included(i) => {
                debug_assert!(
                    self.bounds.contains(i),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            l @ std::ops::Bound::Excluded(i) => {
                debug_assert!(
                    self.bounds.contains(&i.saturating_sub(1)),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            std::ops::Bound::Unbounded => self.bounds.end_bound().cloned(),
        };
        self.bounds = (start, end);
        self
    }

    fn format_bounds(&self) -> String {
        let mut result = match self.bounds.0 {
            std::ops::Bound::Included(i) => i.to_string(),
            std::ops::Bound::Excluded(i) => i.saturating_add(1).to_string(),
            std::ops::Bound::Unbounded => i64::MIN.to_string(),
        };
        result.push_str("..");
        match self.bounds.1 {
            std::ops::Bound::Included(i) => {
                result.push('=');
                result.push_str(&i.to_string());
            }
            std::ops::Bound::Excluded(i) => {
                result.push_str(&i.to_string());
            }
            std::ops::Bound::Unbounded => {
                result.push_str(&i64::MAX.to_string());
            }
        }
        result
    }
}

impl<T: std::convert::TryFrom<i64> + Clone + Send + Sync + 'static> TypedValueParser
    for RangedI64ValueParser<T>
where
    <T as std::convert::TryFrom<i64>>::Error: Send + Sync + 'static + std::error::Error + ToString,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        raw_value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = raw_value.to_str().ok_or_else(|| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        let value = value.parse::<i64>().map_err(|err| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            crate::Error::value_validation(
                arg,
                raw_value.to_string_lossy().into_owned(),
                err.into(),
            )
            .with_cmd(cmd)
        })?;
        if !self.bounds.contains(&value) {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            return Err(crate::Error::value_validation(
                arg,
                raw_value.to_string_lossy().into_owned(),
                format!("{} is not in {}", value, self.format_bounds()).into(),
            )
            .with_cmd(cmd));
        }

        let value: Result<Self::Value, _> = value.try_into();
        let value = value.map_err(|err| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            crate::Error::value_validation(
                arg,
                raw_value.to_string_lossy().into_owned(),
                err.into(),
            )
            .with_cmd(cmd)
        })?;

        Ok(value)
    }
}

impl<T: std::convert::TryFrom<i64> + Clone + Send + Sync, B: RangeBounds<i64>> From<B>
    for RangedI64ValueParser<T>
{
    fn from(range: B) -> Self {
        Self {
            bounds: (range.start_bound().cloned(), range.end_bound().cloned()),
            target: Default::default(),
        }
    }
}

impl<T: std::convert::TryFrom<i64> + Clone + Send + Sync> Default for RangedI64ValueParser<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse number that fall within a range of values
///
/// # Example
///
/// Usage:
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("port")
///             .long("port")
///             .value_parser(clap::value_parser!(u64).range(3000..))
///             .takes_value(true)
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "--port", "3001"]).unwrap();
/// let port: u64 = *m.get_one("port")
///     .expect("required");
/// assert_eq!(port, 3001);
/// ```
///
/// Semantics:
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
/// let value_parser = clap::builder::RangedU64ValueParser::<u32>::new().range(0..200);
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("-200")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("300")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("-1")).is_err());
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("0")).unwrap(), 0);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("50")).unwrap(), 50);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct RangedU64ValueParser<T: std::convert::TryFrom<u64> = u64> {
    bounds: (std::ops::Bound<u64>, std::ops::Bound<u64>),
    target: std::marker::PhantomData<T>,
}

impl<T: std::convert::TryFrom<u64>> RangedU64ValueParser<T> {
    /// Select full range of `u64`
    pub fn new() -> Self {
        Self::from(..)
    }

    /// Narrow the supported range
    pub fn range<B: RangeBounds<u64>>(mut self, range: B) -> Self {
        // Consideration: when the user does `value_parser!(u8).range()`
        // - Avoid programming mistakes by accidentally expanding the range
        // - Make it convenient to limit the range like with `..10`
        let start = match range.start_bound() {
            l @ std::ops::Bound::Included(i) => {
                debug_assert!(
                    self.bounds.contains(i),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            l @ std::ops::Bound::Excluded(i) => {
                debug_assert!(
                    self.bounds.contains(&i.saturating_add(1)),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            std::ops::Bound::Unbounded => self.bounds.start_bound().cloned(),
        };
        let end = match range.end_bound() {
            l @ std::ops::Bound::Included(i) => {
                debug_assert!(
                    self.bounds.contains(i),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            l @ std::ops::Bound::Excluded(i) => {
                debug_assert!(
                    self.bounds.contains(&i.saturating_sub(1)),
                    "{} must be in {:?}",
                    i,
                    self.bounds
                );
                l.cloned()
            }
            std::ops::Bound::Unbounded => self.bounds.end_bound().cloned(),
        };
        self.bounds = (start, end);
        self
    }

    fn format_bounds(&self) -> String {
        let mut result = match self.bounds.0 {
            std::ops::Bound::Included(i) => i.to_string(),
            std::ops::Bound::Excluded(i) => i.saturating_add(1).to_string(),
            std::ops::Bound::Unbounded => u64::MIN.to_string(),
        };
        result.push_str("..");
        match self.bounds.1 {
            std::ops::Bound::Included(i) => {
                result.push('=');
                result.push_str(&i.to_string());
            }
            std::ops::Bound::Excluded(i) => {
                result.push_str(&i.to_string());
            }
            std::ops::Bound::Unbounded => {
                result.push_str(&u64::MAX.to_string());
            }
        }
        result
    }
}

impl<T: std::convert::TryFrom<u64> + Clone + Send + Sync + 'static> TypedValueParser
    for RangedU64ValueParser<T>
where
    <T as std::convert::TryFrom<u64>>::Error: Send + Sync + 'static + std::error::Error + ToString,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        raw_value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = raw_value.to_str().ok_or_else(|| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        let value = value.parse::<u64>().map_err(|err| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            crate::Error::value_validation(
                arg,
                raw_value.to_string_lossy().into_owned(),
                err.into(),
            )
            .with_cmd(cmd)
        })?;
        if !self.bounds.contains(&value) {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            return Err(crate::Error::value_validation(
                arg,
                raw_value.to_string_lossy().into_owned(),
                format!("{} is not in {}", value, self.format_bounds()).into(),
            )
            .with_cmd(cmd));
        }

        let value: Result<Self::Value, _> = value.try_into();
        let value = value.map_err(|err| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            crate::Error::value_validation(
                arg,
                raw_value.to_string_lossy().into_owned(),
                err.into(),
            )
            .with_cmd(cmd)
        })?;

        Ok(value)
    }
}

impl<T: std::convert::TryFrom<u64>, B: RangeBounds<u64>> From<B> for RangedU64ValueParser<T> {
    fn from(range: B) -> Self {
        Self {
            bounds: (range.start_bound().cloned(), range.end_bound().cloned()),
            target: Default::default(),
        }
    }
}

impl<T: std::convert::TryFrom<u64>> Default for RangedU64ValueParser<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation for [`ValueParser::bool`]
///
/// Useful for composing new [`TypedValueParser`]s
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct BoolValueParser {}

impl BoolValueParser {
    /// Implementation for [`ValueParser::bool`]
    pub fn new() -> Self {
        Self {}
    }

    fn possible_values() -> impl Iterator<Item = crate::PossibleValue<'static>> {
        ["true", "false"]
            .iter()
            .copied()
            .map(crate::PossibleValue::new)
    }
}

impl TypedValueParser for BoolValueParser {
    type Value = bool;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = if value == std::ffi::OsStr::new("true") {
            true
        } else if value == std::ffi::OsStr::new("false") {
            false
        } else {
            // Intentionally showing hidden as we hide all of them
            let possible_vals = Self::possible_values()
                .map(|v| v.get_name())
                .collect::<Vec<_>>();

            return Err(crate::Error::invalid_value(
                cmd,
                value.to_string_lossy().into_owned(),
                &possible_vals,
                arg.map(ToString::to_string)
                    .unwrap_or_else(|| "...".to_owned()),
            ));
        };
        Ok(value)
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        Some(Box::new(Self::possible_values()))
    }
}

impl Default for BoolValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse false-like string values, everything else is `true`
///
/// See also:
/// - [`ValueParser::bool`] for assuming non-false is true
/// - [`BoolishValueParser`] for different human readable bool representations
///
/// # Example
///
/// Usage:
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("append")
///             .value_parser(clap::builder::FalseyValueParser::new())
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "true"]).unwrap();
/// let port: bool = *m.get_one("append")
///     .expect("required");
/// assert_eq!(port, true);
/// ```
///
/// Semantics:
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
/// let value_parser = clap::builder::FalseyValueParser::new();
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).unwrap(), true);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("100")).unwrap(), true);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("false")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("No")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("oFF")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("0")).unwrap(), false);
/// ```
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct FalseyValueParser {}

impl FalseyValueParser {
    /// Parse false-like string values, everything else is `true`
    pub fn new() -> Self {
        Self {}
    }

    fn possible_values() -> impl Iterator<Item = crate::PossibleValue<'static>> {
        crate::util::TRUE_LITERALS
            .iter()
            .chain(crate::util::FALSE_LITERALS.iter())
            .copied()
            .map(|l| crate::PossibleValue::new(l).hide(l != "true" && l != "false"))
    }
}

impl TypedValueParser for FalseyValueParser {
    type Value = bool;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        _arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = value.to_str().ok_or_else(|| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        let value = if value.is_empty() {
            false
        } else {
            crate::util::str_to_bool(value).unwrap_or(true)
        };
        Ok(value)
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        Some(Box::new(Self::possible_values()))
    }
}

impl Default for FalseyValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse bool-like string values, everything else is `true`
///
/// See also:
/// - [`ValueParser::bool`] for different human readable bool representations
/// - [`FalseyValueParser`] for assuming non-false is true
///
/// # Example
///
/// Usage:
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("append")
///             .value_parser(clap::builder::BoolishValueParser::new())
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "true"]).unwrap();
/// let port: bool = *m.get_one("append")
///     .expect("required");
/// assert_eq!(port, true);
/// ```
///
/// Semantics:
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
/// let value_parser = clap::builder::BoolishValueParser::new();
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).is_err());
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("100")).is_err());
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("true")).unwrap(), true);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("Yes")).unwrap(), true);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("oN")).unwrap(), true);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("1")).unwrap(), true);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("false")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("No")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("oFF")).unwrap(), false);
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("0")).unwrap(), false);
/// ```
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct BoolishValueParser {}

impl BoolishValueParser {
    /// Parse bool-like string values, everything else is `true`
    pub fn new() -> Self {
        Self {}
    }

    fn possible_values() -> impl Iterator<Item = crate::PossibleValue<'static>> {
        crate::util::TRUE_LITERALS
            .iter()
            .chain(crate::util::FALSE_LITERALS.iter())
            .copied()
            .map(|l| crate::PossibleValue::new(l).hide(l != "true" && l != "false"))
    }
}

impl TypedValueParser for BoolishValueParser {
    type Value = bool;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = value.to_str().ok_or_else(|| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        let value = crate::util::str_to_bool(value).ok_or_else(|| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "...".to_owned());
            crate::Error::value_validation(arg, value.to_owned(), "value was not a boolean".into())
                .with_cmd(cmd)
        })?;
        Ok(value)
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::PossibleValue<'static>> + '_>> {
        Some(Box::new(Self::possible_values()))
    }
}

impl Default for BoolishValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse non-empty string values
///
/// See also:
/// - [`ValueParser::string`]
///
/// # Example
///
/// Usage:
/// ```rust
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("append")
///             .value_parser(clap::builder::NonEmptyStringValueParser::new())
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "true"]).unwrap();
/// let port: &String = m.get_one("append")
///     .expect("required");
/// assert_eq!(port, "true");
/// ```
///
/// Semantics:
/// ```rust
/// # use std::ffi::OsStr;
/// # use clap::builder::TypedValueParser;
/// # let cmd = clap::Command::new("test");
/// # let arg = None;
/// let value_parser = clap::builder::NonEmptyStringValueParser::new();
/// assert_eq!(value_parser.parse_ref(&cmd, arg, OsStr::new("random")).unwrap(), "random");
/// assert!(value_parser.parse_ref(&cmd, arg, OsStr::new("")).is_err());
/// ```
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct NonEmptyStringValueParser {}

impl NonEmptyStringValueParser {
    /// Parse non-empty string values
    pub fn new() -> Self {
        Self {}
    }
}

impl TypedValueParser for NonEmptyStringValueParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        if value.is_empty() {
            return Err(crate::Error::empty_value(
                cmd,
                &[],
                arg.map(ToString::to_string)
                    .unwrap_or_else(|| "...".to_owned()),
            ));
        }
        let value = value.to_str().ok_or_else(|| {
            crate::Error::invalid_utf8(
                cmd,
                crate::output::Usage::new(cmd).create_usage_with_title(&[]),
            )
        })?;
        Ok(value.to_owned())
    }
}

impl Default for NonEmptyStringValueParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapt a `TypedValueParser` from one value to another
///
/// See [`TypedValueParser::map`]
#[derive(Clone, Debug)]
pub struct MapValueParser<P, F> {
    parser: P,
    func: F,
}

impl<P, F> MapValueParser<P, F> {
    fn new(parser: P, func: F) -> Self {
        Self { parser, func }
    }
}

impl<P, F, T> TypedValueParser for MapValueParser<P, F>
where
    P: TypedValueParser,
    P::Value: Send + Sync + Clone,
    F: Fn(P::Value) -> T + Clone + Send + Sync + 'static,
    T: Send + Sync + Clone,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, crate::Error> {
        let value = self.parser.parse_ref(cmd, arg, value)?;
        let value = (self.func)(value);
        Ok(value)
    }

    fn parse(
        &self,
        cmd: &crate::Command,
        arg: Option<&crate::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, crate::Error> {
        let value = self.parser.parse(cmd, arg, value)?;
        let value = (self.func)(value);
        Ok(value)
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = crate::builder::PossibleValue<'static>> + '_>> {
        self.parser.possible_values()
    }
}

/// Register a type with [value_parser!][crate::value_parser!]
///
/// # Example
///
/// ```rust
/// #[derive(Copy, Clone, Debug)]
/// pub struct Custom(u32);
///
/// impl clap::builder::ValueParserFactory for Custom {
///     type Parser = CustomValueParser;
///     fn value_parser() -> Self::Parser {
///         CustomValueParser
///     }
/// }
///
/// #[derive(Clone, Debug)]
/// pub struct CustomValueParser;
/// impl clap::builder::TypedValueParser for CustomValueParser {
///     type Value = Custom;
///
///     fn parse_ref(
///         &self,
///         cmd: &clap::Command,
///         arg: Option<&clap::Arg>,
///         value: &std::ffi::OsStr,
///     ) -> Result<Self::Value, clap::Error> {
///         let inner = clap::value_parser!(u32);
///         let val = inner.parse_ref(cmd, arg, value)?;
///         Ok(Custom(val))
///     }
/// }
///
/// let parser: CustomValueParser = clap::value_parser!(Custom);
/// ```
pub trait ValueParserFactory {
    /// Generated parser, usually [`ValueParser`].
    ///
    /// It should at least be a type that supports `Into<ValueParser>`.  A non-`ValueParser` type
    /// allows the caller to do further initialization on the parser.
    type Parser;

    /// Create the specified [`Self::Parser`]
    fn value_parser() -> Self::Parser;
}
impl ValueParserFactory for String {
    type Parser = ValueParser;
    fn value_parser() -> Self::Parser {
        ValueParser::string()
    }
}
impl ValueParserFactory for std::ffi::OsString {
    type Parser = ValueParser;
    fn value_parser() -> Self::Parser {
        ValueParser::os_string()
    }
}
impl ValueParserFactory for std::path::PathBuf {
    type Parser = ValueParser;
    fn value_parser() -> Self::Parser {
        ValueParser::path_buf()
    }
}
impl ValueParserFactory for bool {
    type Parser = ValueParser;
    fn value_parser() -> Self::Parser {
        ValueParser::bool()
    }
}
impl ValueParserFactory for u8 {
    type Parser = RangedI64ValueParser<u8>;
    fn value_parser() -> Self::Parser {
        let start: i64 = u8::MIN.into();
        let end: i64 = u8::MAX.into();
        RangedI64ValueParser::new().range(start..=end)
    }
}
impl ValueParserFactory for i8 {
    type Parser = RangedI64ValueParser<i8>;
    fn value_parser() -> Self::Parser {
        let start: i64 = i8::MIN.into();
        let end: i64 = i8::MAX.into();
        RangedI64ValueParser::new().range(start..=end)
    }
}
impl ValueParserFactory for u16 {
    type Parser = RangedI64ValueParser<u16>;
    fn value_parser() -> Self::Parser {
        let start: i64 = u16::MIN.into();
        let end: i64 = u16::MAX.into();
        RangedI64ValueParser::new().range(start..=end)
    }
}
impl ValueParserFactory for i16 {
    type Parser = RangedI64ValueParser<i16>;
    fn value_parser() -> Self::Parser {
        let start: i64 = i16::MIN.into();
        let end: i64 = i16::MAX.into();
        RangedI64ValueParser::new().range(start..=end)
    }
}
impl ValueParserFactory for u32 {
    type Parser = RangedI64ValueParser<u32>;
    fn value_parser() -> Self::Parser {
        let start: i64 = u32::MIN.into();
        let end: i64 = u32::MAX.into();
        RangedI64ValueParser::new().range(start..=end)
    }
}
impl ValueParserFactory for i32 {
    type Parser = RangedI64ValueParser<i32>;
    fn value_parser() -> Self::Parser {
        let start: i64 = i32::MIN.into();
        let end: i64 = i32::MAX.into();
        RangedI64ValueParser::new().range(start..=end)
    }
}
impl ValueParserFactory for i64 {
    type Parser = RangedI64ValueParser<i64>;
    fn value_parser() -> Self::Parser {
        RangedI64ValueParser::new()
    }
}
impl ValueParserFactory for u64 {
    type Parser = RangedU64ValueParser<u64>;
    fn value_parser() -> Self::Parser {
        RangedU64ValueParser::new()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct _AutoValueParser<T>(std::marker::PhantomData<T>);

impl<T> _AutoValueParser<T> {
    #[doc(hidden)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Default::default())
    }
}

/// Unstable [`ValueParser`]
///
/// Implementation may change to more specific instance in the future
#[doc(hidden)]
#[derive(Debug)]
pub struct _AnonymousValueParser(ValueParser);

#[doc(hidden)]
pub mod via_prelude {
    use super::*;

    #[doc(hidden)]
    pub trait _ValueParserViaFactory: private::_ValueParserViaFactorySealed {
        type Parser;
        fn value_parser(&self) -> Self::Parser;
    }
    impl<P: ValueParserFactory> _ValueParserViaFactory for &&_AutoValueParser<P> {
        type Parser = P::Parser;
        fn value_parser(&self) -> Self::Parser {
            P::value_parser()
        }
    }

    #[doc(hidden)]
    pub trait _ValueParserViaValueEnum: private::_ValueParserViaValueEnumSealed {
        type Output;

        fn value_parser(&self) -> Self::Output;
    }
    impl<E: crate::ValueEnum + Clone + Send + Sync + 'static> _ValueParserViaValueEnum
        for &_AutoValueParser<E>
    {
        type Output = EnumValueParser<E>;

        fn value_parser(&self) -> Self::Output {
            EnumValueParser::<E>::new()
        }
    }

    #[doc(hidden)]
    pub trait _ValueParserViaFromStr: private::_ValueParserViaFromStrSealed {
        fn value_parser(&self) -> _AnonymousValueParser;
    }
    impl<FromStr> _ValueParserViaFromStr for _AutoValueParser<FromStr>
    where
        FromStr: std::str::FromStr + std::any::Any + Clone + Send + Sync + 'static,
        <FromStr as std::str::FromStr>::Err:
            Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        fn value_parser(&self) -> _AnonymousValueParser {
            let func: fn(&str) -> Result<FromStr, <FromStr as std::str::FromStr>::Err> =
                FromStr::from_str;
            _AnonymousValueParser(ValueParser::new(func))
        }
    }
}

/// Select a [`ValueParser`] implementation from the intended type
///
/// To register a custom type with this macro, implement [`ValueParserFactory`].
///
/// # Example
///
/// Usage:
/// ```rust
/// # use std::path::PathBuf;
/// # use std::path::Path;
/// let mut cmd = clap::Command::new("raw")
///     .arg(
///         clap::Arg::new("output")
///             .value_parser(clap::value_parser!(PathBuf))
///             .required(true)
///     );
///
/// let m = cmd.try_get_matches_from_mut(["cmd", "file.txt"]).unwrap();
/// let port: &PathBuf = m.get_one("output")
///     .expect("required");
/// assert_eq!(port, Path::new("file.txt"));
/// ```
///
/// Supported types:
/// ```rust
/// // Built-in types
/// let parser = clap::value_parser!(String);
/// assert_eq!(format!("{:?}", parser), "ValueParser::string");
/// let parser = clap::value_parser!(std::ffi::OsString);
/// assert_eq!(format!("{:?}", parser), "ValueParser::os_string");
/// let parser = clap::value_parser!(std::path::PathBuf);
/// assert_eq!(format!("{:?}", parser), "ValueParser::path_buf");
/// let _parser = clap::value_parser!(u16).range(3000..);
/// let _parser = clap::value_parser!(u64).range(3000..);
///
/// // FromStr types
/// let parser = clap::value_parser!(usize);
/// assert_eq!(format!("{:?}", parser), "_AnonymousValueParser(ValueParser::other(usize))");
///
/// // ValueEnum types
/// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// enum ColorChoice {
///     Always,
///     Auto,
///     Never,
/// }
/// impl clap::ValueEnum for ColorChoice {
///     // ...
/// #     fn value_variants<'a>() -> &'a [Self] {
/// #         &[Self::Always, Self::Auto, Self::Never]
/// #     }
/// #     fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
/// #         match self {
/// #             Self::Always => Some(clap::PossibleValue::new("always")),
/// #             Self::Auto => Some(clap::PossibleValue::new("auto")),
/// #             Self::Never => Some(clap::PossibleValue::new("never")),
/// #         }
/// #     }
/// }
/// let _parser = clap::value_parser!(ColorChoice);
/// ```
#[macro_export]
macro_rules! value_parser {
    ($name:ty) => {{
        use $crate::builder::via_prelude::*;
        let auto = $crate::builder::_AutoValueParser::<$name>::new();
        (&&&auto).value_parser()
    }};
}

mod private {
    use super::*;

    pub trait _ValueParserViaSelfSealed {}
    impl<P: Into<ValueParser>> _ValueParserViaSelfSealed for &&&_AutoValueParser<P> {}

    pub trait _ValueParserViaFactorySealed {}
    impl<P: ValueParserFactory> _ValueParserViaFactorySealed for &&_AutoValueParser<P> {}

    pub trait _ValueParserViaValueEnumSealed {}
    impl<E: crate::ValueEnum> _ValueParserViaValueEnumSealed for &_AutoValueParser<E> {}

    pub trait _ValueParserViaFromStrSealed {}
    impl<FromStr> _ValueParserViaFromStrSealed for _AutoValueParser<FromStr>
    where
        FromStr: std::str::FromStr + std::any::Any + Send + Sync + 'static,
        <FromStr as std::str::FromStr>::Err:
            Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ensure_typed_applies_to_parse() {
        fn parse(_: &str) -> Result<usize, std::io::Error> {
            Ok(10)
        }
        let cmd = crate::Command::new("cmd");
        let arg = None;
        assert_eq!(
            TypedValueParser::parse_ref(&parse, &cmd, arg, std::ffi::OsStr::new("foo")).unwrap(),
            10
        );
    }
}
