// Std
use std::any::Any;
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display};
use std::iter::{Cloned, Flatten, Map};
use std::slice::Iter;
use std::str::FromStr;

// Third Party
use indexmap::IndexMap;

// Internal
use crate::parser::AnyValue;
use crate::parser::AnyValueId;
use crate::parser::MatchedArg;
use crate::parser::MatchesError;
use crate::parser::ValueSource;
use crate::util::{Id, Key};
use crate::Error;
use crate::INTERNAL_ERROR_MSG;

/// Container for parse results.
///
/// Used to get information about the arguments that were supplied to the program at runtime by
/// the user. New instances of this struct are obtained by using the [`Command::get_matches`] family of
/// methods.
///
/// # Examples
///
/// ```no_run
/// # use clap::{Command, Arg, ValueSource};
/// let matches = Command::new("MyApp")
///     .arg(Arg::new("out")
///         .long("output")
///         .required(true)
///         .takes_value(true)
///         .default_value("-"))
///     .arg(Arg::new("cfg")
///         .short('c')
///         .takes_value(true))
///     .get_matches(); // builds the instance of ArgMatches
///
/// // to get information about the "cfg" argument we created, such as the value supplied we use
/// // various ArgMatches methods, such as [ArgMatches::get_one]
/// if let Some(c) = matches.get_one::<String>("cfg") {
///     println!("Value for -c: {}", c);
/// }
///
/// // The ArgMatches::get_one method returns an Option because the user may not have supplied
/// // that argument at runtime. But if we specified that the argument was "required" as we did
/// // with the "out" argument, we can safely unwrap because `clap` verifies that was actually
/// // used at runtime.
/// println!("Value for --output: {}", matches.get_one::<String>("out").unwrap());
///
/// // You can check the presence of an argument's values
/// if matches.contains_id("out") {
///     // However, if you want to know where the value came from
///     if matches.value_source("out").expect("checked contains_id") == ValueSource::CommandLine {
///         println!("`out` set by user");
///     } else {
///         println!("`out` is defaulted");
///     }
/// }
/// ```
/// [`Command::get_matches`]: crate::Command::get_matches()
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArgMatches {
    #[cfg(debug_assertions)]
    pub(crate) valid_args: Vec<Id>,
    #[cfg(debug_assertions)]
    pub(crate) valid_subcommands: Vec<Id>,
    #[cfg(debug_assertions)]
    pub(crate) disable_asserts: bool,
    pub(crate) args: IndexMap<Id, MatchedArg>,
    pub(crate) subcommand: Option<Box<SubCommand>>,
}

/// # Arguments
impl ArgMatches {
    /// Gets the value of a specific option or positional argument.
    ///
    /// i.e. an argument that [takes an additional value][crate::Arg::takes_value] at runtime.
    ///
    /// Returns an error if the wrong type was used.
    ///
    /// Returns `None` if the option wasn't present.
    ///
    /// *NOTE:* This will always return `Some(value)` if [`default_value`] has been set.
    /// [`ArgMatches::value_source`] can be used to check if a value is present at runtime.
    ///
    /// # Panic
    ///
    /// If the argument definition and access mismatch.  To handle this case programmatically, see
    /// [`ArgMatches::try_get_one`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, value_parser};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("port")
    ///         .value_parser(value_parser!(usize))
    ///         .takes_value(true)
    ///         .required(true))
    ///     .get_matches_from(vec!["myapp", "2020"]);
    ///
    /// let port: usize = *m
    ///     .get_one("port")
    ///     .expect("`port`is required");
    /// assert_eq!(port, 2020);
    /// ```
    /// [option]: crate::Arg::takes_value()
    /// [positional]: crate::Arg::index()
    /// [`default_value`]: crate::Arg::default_value()
    #[track_caller]
    pub fn get_one<T: Any + Clone + Send + Sync + 'static>(&self, id: &str) -> Option<&T> {
        let internal_id = Id::from(id);
        MatchesError::unwrap(&internal_id, self.try_get_one(id))
    }

    /// Gets the value of a specific [`ArgAction::Count`][crate::ArgAction::Count] flag
    ///
    /// # Panic
    ///
    /// If the argument's action is not [`ArgAction::Count`][crate::ArgAction::Count]
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
    /// assert_eq!(
    ///     matches.get_count("flag"),
    ///     2
    /// );
    /// ```
    #[track_caller]
    pub fn get_count(&self, id: &str) -> u8 {
        *self
            .get_one::<u8>(id)
            .expect("ArgAction::Count is defaulted")
    }

    /// Gets the value of a specific [`ArgAction::SetTrue`][crate::ArgAction::SetTrue] or [`ArgAction::SetFalse`][crate::ArgAction::SetFalse] flag
    ///
    /// # Panic
    ///
    /// If the argument's action is not [`ArgAction::SetTrue`][crate::ArgAction::SetTrue] or [`ArgAction::SetFalse`][crate::ArgAction::SetFalse]
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
    /// assert_eq!(
    ///     matches.get_flag("flag"),
    ///     true
    /// );
    /// ```
    #[track_caller]
    pub fn get_flag(&self, id: &str) -> bool {
        *self
            .get_one::<bool>(id)
            .expect("ArgAction::SetTrue / ArgAction::SetFalse is defaulted")
    }

    /// Iterate over values of a specific option or positional argument.
    ///
    /// i.e. an argument that takes multiple values at runtime.
    ///
    /// Returns an error if the wrong type was used.
    ///
    /// Returns `None` if the option wasn't present.
    ///
    /// # Panic
    ///
    /// If the argument definition and access mismatch.  To handle this case programmatically, see
    /// [`ArgMatches::try_get_many`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, value_parser, ArgAction};
    /// let m = Command::new("myprog")
    ///     .arg(Arg::new("ports")
    ///         .action(ArgAction::Append)
    ///         .value_parser(value_parser!(usize))
    ///         .short('p')
    ///         .takes_value(true)
    ///         .required(true))
    ///     .get_matches_from(vec![
    ///         "myprog", "-p", "22", "-p", "80", "-p", "2020"
    ///     ]);
    /// let vals: Vec<usize> = m.get_many("ports")
    ///     .expect("`port`is required")
    ///     .copied()
    ///     .collect();
    /// assert_eq!(vals, [22, 80, 2020]);
    /// ```
    #[track_caller]
    pub fn get_many<T: Any + Clone + Send + Sync + 'static>(
        &self,
        id: &str,
    ) -> Option<ValuesRef<T>> {
        let internal_id = Id::from(id);
        MatchesError::unwrap(&internal_id, self.try_get_many(id))
    }

    /// Iterate over the original argument values.
    ///
    /// An `OsStr` on Unix-like systems is any series of bytes, regardless of whether or not they
    /// contain valid UTF-8. Since [`String`]s in Rust are guaranteed to be valid UTF-8, a valid
    /// filename on a Unix system as an argument value may contain invalid UTF-8.
    ///
    /// Returns `None` if the option wasn't present.
    ///
    /// # Panic
    ///
    /// If the argument definition and access mismatch.  To handle this case programmatically, see
    /// [`ArgMatches::try_get_raw`].
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```")]
    /// # use clap::{Command, arg, value_parser};
    /// # use std::ffi::{OsStr,OsString};
    /// # use std::os::unix::ffi::{OsStrExt,OsStringExt};
    /// use std::path::PathBuf;
    ///
    /// let m = Command::new("utf8")
    ///     .arg(arg!(<arg> ... "some arg").value_parser(value_parser!(PathBuf)))
    ///     .get_matches_from(vec![OsString::from("myprog"),
    ///                                 // "Hi"
    ///                                 OsString::from_vec(vec![b'H', b'i']),
    ///                                 // "{0xe9}!"
    ///                                 OsString::from_vec(vec![0xe9, b'!'])]);
    ///
    /// let mut itr = m.get_raw("arg")
    ///     .expect("`port`is required")
    ///     .into_iter();
    /// assert_eq!(itr.next(), Some(OsStr::new("Hi")));
    /// assert_eq!(itr.next(), Some(OsStr::from_bytes(&[0xe9, b'!'])));
    /// assert_eq!(itr.next(), None);
    /// ```
    /// [`Iterator`]: std::iter::Iterator
    /// [`OsSt`]: std::ffi::OsStr
    /// [values]: OsValues
    /// [`String`]: std::string::String
    #[track_caller]
    pub fn get_raw(&self, id: &str) -> Option<RawValues<'_>> {
        let internal_id = Id::from(id);
        MatchesError::unwrap(&internal_id, self.try_get_raw(id))
    }

    /// Returns the value of a specific option or positional argument.
    ///
    /// i.e. an argument that [takes an additional value][crate::Arg::takes_value] at runtime.
    ///
    /// Returns an error if the wrong type was used.  No item will have been removed.
    ///
    /// Returns `None` if the option wasn't present.
    ///
    /// *NOTE:* This will always return `Some(value)` if [`default_value`] has been set.
    /// [`ArgMatches::value_source`] can be used to check if a value is present at runtime.
    ///
    /// # Panic
    ///
    /// If the argument definition and access mismatch.  To handle this case programmatically, see
    /// [`ArgMatches::try_remove_one`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, value_parser};
    /// let mut m = Command::new("myprog")
    ///     .arg(Arg::new("file")
    ///         .required(true)
    ///         .takes_value(true))
    ///     .get_matches_from(vec![
    ///         "myprog", "file.txt",
    ///     ]);
    /// let vals: String = m.remove_one("file")
    ///     .expect("`file`is required");
    /// assert_eq!(vals, "file.txt");
    /// ```
    /// [option]: crate::Arg::takes_value()
    /// [positional]: crate::Arg::index()
    /// [`default_value`]: crate::Arg::default_value()
    #[track_caller]
    pub fn remove_one<T: Any + Clone + Send + Sync + 'static>(&mut self, id: &str) -> Option<T> {
        let internal_id = Id::from(id);
        MatchesError::unwrap(&internal_id, self.try_remove_one(id))
    }

    /// Return values of a specific option or positional argument.
    ///
    /// i.e. an argument that takes multiple values at runtime.
    ///
    /// Returns an error if the wrong type was used.  No item will have been removed.
    ///
    /// Returns `None` if the option wasn't present.
    ///
    /// # Panic
    ///
    /// If the argument definition and access mismatch.  To handle this case programmatically, see
    /// [`ArgMatches::try_remove_many`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, value_parser, ArgAction};
    /// let mut m = Command::new("myprog")
    ///     .arg(Arg::new("file")
    ///         .action(ArgAction::Append)
    ///         .multiple_values(true)
    ///         .required(true)
    ///         .takes_value(true))
    ///     .get_matches_from(vec![
    ///         "myprog", "file1.txt", "file2.txt", "file3.txt", "file4.txt",
    ///     ]);
    /// let vals: Vec<String> = m.remove_many("file")
    ///     .expect("`file`is required")
    ///     .collect();
    /// assert_eq!(vals, ["file1.txt", "file2.txt", "file3.txt", "file4.txt"]);
    /// ```
    #[track_caller]
    pub fn remove_many<T: Any + Clone + Send + Sync + 'static>(
        &mut self,
        id: &str,
    ) -> Option<Values2<T>> {
        let internal_id = Id::from(id);
        MatchesError::unwrap(&internal_id, self.try_remove_many(id))
    }

    /// Check if values are present for the argument or group id
    ///
    /// *NOTE:* This will always return `true` if [`default_value`] has been set.
    /// [`ArgMatches::value_source`] can be used to check if a value is present at runtime.
    ///
    /// # Panics
    ///
    /// If `id` is is not a valid argument or group name.  To handle this case programmatically, see
    /// [`ArgMatches::try_contains_id`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myprog")
    ///     .arg(Arg::new("debug")
    ///         .short('d'))
    ///     .get_matches_from(vec![
    ///         "myprog", "-d"
    ///     ]);
    ///
    /// assert!(m.contains_id("debug"));
    /// ```
    ///
    /// [`default_value`]: crate::Arg::default_value()
    pub fn contains_id(&self, id: &str) -> bool {
        let internal_id = Id::from(id);
        MatchesError::unwrap(&internal_id, self.try_contains_id(id))
    }

    /// Check if any args were present on the command line
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let mut cmd = Command::new("myapp")
    ///     .arg(Arg::new("output")
    ///         .takes_value(true));
    ///
    /// let m = cmd
    ///     .try_get_matches_from_mut(vec!["myapp", "something"])
    ///     .unwrap();
    /// assert!(m.args_present());
    ///
    /// let m = cmd
    ///     .try_get_matches_from_mut(vec!["myapp"])
    ///     .unwrap();
    /// assert!(! m.args_present());
    pub fn args_present(&self) -> bool {
        !self.args.is_empty()
    }

    /// Deprecated, replaced with [`ArgMatches::get_one()`]
    ///
    /// Replace `m.value_of(...)` with `m.get_one::<String>(...).map(|s| s.as_str())`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgMatches::get_one()`

Replace `m.value_of(...)` with `m.get_one::<String>(...).map(|s| s.as_str())`
"
        )
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value_of<T: Key>(&self, id: T) -> Option<&str> {
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = unwrap_string_arg(&id, arg.first()?);
        Some(v)
    }

    /// Deprecated, replaced with [`ArgMatches::get_one()`]
    ///
    /// Replace `m.value_of(...)` with `m.get_one::<String>(...).map(|s| s.as_str())`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgMatches::get_one()`

Replace `m.value_of(...)` with `m.get_one::<String>(...).map(|s| s.as_str())`
"
        )
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value_of_lossy<T: Key>(&self, id: T) -> Option<Cow<'_, str>> {
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = unwrap_os_string_arg(&id, arg.first()?);
        Some(v.to_string_lossy())
    }

    /// Deprecated, replaced with [`ArgMatches::get_one()`]
    ///
    /// Replace `m.value_of(...)` with `m.get_one::<OsString>(...).map(|s| s.as_os_str())`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgMatches::get_one()`

Replace `m.value_of(...)` with `m.get_one::<OsString>(...).map(|s| s.as_os_str())`
"
        )
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value_of_os<T: Key>(&self, id: T) -> Option<&OsStr> {
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = unwrap_os_string_arg(&id, arg.first()?);
        Some(v)
    }

    /// Deprecated, replaced with [`ArgMatches::get_many()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgMatches::get_many()`
"
        )
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn values_of<T: Key>(&self, id: T) -> Option<Values> {
        #![allow(deprecated)]
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = Values {
            iter: arg.vals_flatten().map(unwrap_string),
            len: arg.num_vals(),
        };
        Some(v)
    }

    /// Get an [`Iterator`] over groups of values of a specific option.
    ///
    /// specifically grouped by the occurrences of the options.
    ///
    /// Each group is a `Vec<&str>` containing the arguments passed to a single occurrence
    /// of the option.
    ///
    /// If the option doesn't support multiple occurrences, or there was only a single occurrence,
    /// the iterator will only contain a single item.
    ///
    /// Returns `None` if the option wasn't present.
    ///
    /// # Panics
    ///
    /// If the value is invalid UTF-8.
    ///
    /// If `id` is not a valid argument or group id.
    ///
    /// # Examples
    /// ```rust
    /// # use clap::{Command,Arg, ArgAction};
    /// let m = Command::new("myprog")
    ///     .arg(Arg::new("exec")
    ///         .short('x')
    ///         .min_values(1)
    ///         .action(ArgAction::Append)
    ///         .value_terminator(";"))
    ///     .get_matches_from(vec![
    ///         "myprog", "-x", "echo", "hi", ";", "-x", "echo", "bye"]);
    /// let vals: Vec<Vec<&str>> = m.grouped_values_of("exec").unwrap().collect();
    /// assert_eq!(vals, [["echo", "hi"], ["echo", "bye"]]);
    /// ```
    /// [`Iterator`]: std::iter::Iterator
    #[cfg(feature = "unstable-grouped")]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn grouped_values_of<T: Key>(&self, id: T) -> Option<GroupedValues> {
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = GroupedValues {
            iter: arg.vals().map(|g| g.iter().map(unwrap_string).collect()),
            len: arg.vals().len(),
        };
        Some(v)
    }

    /// Deprecated, replaced with [`ArgMatches::get_many()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_many()`")
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn values_of_lossy<T: Key>(&self, id: T) -> Option<Vec<String>> {
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = arg
            .vals_flatten()
            .map(|v| unwrap_os_string_arg(&id, v).to_string_lossy().into_owned())
            .collect();
        Some(v)
    }

    /// Deprecated, replaced with [`ArgMatches::get_many()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_many()`")
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn values_of_os<T: Key>(&self, id: T) -> Option<OsValues> {
        #![allow(deprecated)]
        let id = Id::from(id);
        let arg = self.get_arg(&id)?;
        let v = OsValues {
            iter: arg.vals_flatten().map(unwrap_os_string),
            len: arg.num_vals(),
        };
        Some(v)
    }

    /// Deprecated, replaced with [`ArgMatches::get_one()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_one()`")
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value_of_t<R>(&self, name: &str) -> Result<R, Error>
    where
        R: FromStr,
        <R as FromStr>::Err: Display,
    {
        #![allow(deprecated)]
        let v = self
            .value_of(name)
            .ok_or_else(|| Error::argument_not_found_auto(name.to_string()))?;
        v.parse::<R>().map_err(|e| {
            let message = format!(
                "The argument '{}' isn't a valid value for '{}': {}",
                v, name, e
            );

            Error::value_validation(name.to_string(), v.to_string(), message.into())
        })
    }

    /// Deprecated, replaced with [`ArgMatches::get_one()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_one()`")
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value_of_t_or_exit<R>(&self, name: &str) -> R
    where
        R: FromStr,
        <R as FromStr>::Err: Display,
    {
        #![allow(deprecated)]
        self.value_of_t(name).unwrap_or_else(|e| e.exit())
    }

    /// Deprecated, replaced with [`ArgMatches::get_many()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_many()`")
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn values_of_t<R>(&self, name: &str) -> Result<Vec<R>, Error>
    where
        R: FromStr,
        <R as FromStr>::Err: Display,
    {
        #![allow(deprecated)]
        let v = self
            .values_of(name)
            .ok_or_else(|| Error::argument_not_found_auto(name.to_string()))?;
        v.map(|v| {
            v.parse::<R>().map_err(|e| {
                let message = format!("The argument '{}' isn't a valid value: {}", v, e);

                Error::value_validation(name.to_string(), v.to_string(), message.into())
            })
        })
        .collect()
    }

    /// Deprecated, replaced with [`ArgMatches::get_many()`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_many()`")
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn values_of_t_or_exit<R>(&self, name: &str) -> Vec<R>
    where
        R: FromStr,
        <R as FromStr>::Err: Display,
    {
        #![allow(deprecated)]
        self.values_of_t(name).unwrap_or_else(|e| e.exit())
    }

    /// Deprecated, replaced with [`ArgAction::SetTrue`][crate::ArgAction] or
    /// [`ArgMatches::contains_id`].
    ///
    /// With `ArgAction::SetTrue` becoming the new flag default in clap v4, flags will always be present.
    /// To make the migration easier, we've renamed this function so people can identify when they actually
    /// care about an arg being present or if they need to update to the new way to check a flag's
    /// presence.
    ///
    /// For flags, call `arg.action(ArgAction::SetTrue)` and lookup the value with `matches.get_flag(...)`
    ///
    /// For presence, replace `matches.is_present(...)` with `matches.contains_id(...)`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with either `ArgAction::SetTrue` or `ArgMatches::contains_id(...)` to avoid confusion over new semantics

With `ArgAction::SetTrue` becoming the new flag default in clap v4, flags will always be present.
To make the migration easier, we've renamed this function so people can identify when they actually
care about an arg being present or if they need to update to the new way to check a flag's
presence.

For flags, call `arg.action(ArgAction::SetTrue)` and lookup the value with `matches.get_flag(...)`

For presence, replace `matches.is_present(...)` with `matches.contains_id(...)`
"
        )
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn is_present<T: Key>(&self, id: T) -> bool {
        let id = Id::from(id);

        #[cfg(debug_assertions)]
        self.get_arg(&id);

        self.args.contains_key(&id)
    }

    /// Report where argument value came from
    ///
    /// # Panics
    ///
    /// If `id` is is not a valid argument or group id.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ValueSource};
    /// let m = Command::new("myprog")
    ///     .arg(Arg::new("debug")
    ///         .short('d'))
    ///     .get_matches_from(vec![
    ///         "myprog", "-d"
    ///     ]);
    ///
    /// assert_eq!(m.value_source("debug"), Some(ValueSource::CommandLine));
    /// ```
    ///
    /// [`default_value`]: crate::Arg::default_value()
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn value_source<T: Key>(&self, id: T) -> Option<ValueSource> {
        let id = Id::from(id);

        let value = self.get_arg(&id);

        value.and_then(MatchedArg::source)
    }

    /// Deprecated, replaced with  [`ArgAction::Count`][crate::ArgAction],
    /// [`ArgMatches::get_many`]`.len()`, or [`ArgMatches::value_source`].
    ///
    /// `occurrences_of`s meaning was overloaded and we are replacing it with more direct approaches.
    ///
    /// For counting flags, call `arg.action(ArgAction::Count)` and lookup the value with `matches.get_count(...)`
    ///
    /// To check if a user explicitly passed in a value, check the source with `matches.value_source(...)`
    ///
    /// To see how many values there are, call `matches.get_many::<T>(...).map(|v| v.len())`
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with either `ArgAction::Count`, `ArgMatches::get_many(...).len()`, or `ArgMatches::value_source`

`occurrences_of`s meaning was overloaded and we are replacing it with more direct approaches.

For counting flags, call `arg.action(ArgAction::Count)` and lookup the value with `matches.get_count(...)`

To check if a user explicitly passed in a value, check the source with `matches.value_source(...)`

To see how many values there are, call `matches.get_many::<T>(...).map(|v| v.len())`
"
        )
    )]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn occurrences_of<T: Key>(&self, id: T) -> u64 {
        #![allow(deprecated)]
        self.get_arg(&Id::from(id))
            .map_or(0, |a| a.get_occurrences())
    }

    /// The first index of that an argument showed up.
    ///
    /// Indices are similar to argv indices, but are not exactly 1:1.
    ///
    /// For flags (i.e. those arguments which don't have an associated value), indices refer
    /// to occurrence of the switch, such as `-f`, or `--flag`. However, for options the indices
    /// refer to the *values* `-o val` would therefore not represent two distinct indices, only the
    /// index for `val` would be recorded. This is by design.
    ///
    /// Besides the flag/option discrepancy, the primary difference between an argv index and clap
    /// index, is that clap continues counting once all arguments have properly separated, whereas
    /// an argv index does not.
    ///
    /// The examples should clear this up.
    ///
    /// *NOTE:* If an argument is allowed multiple times, this method will only give the *first*
    /// index.  See [`ArgMatches::indices_of`].
    ///
    /// # Panics
    ///
    /// If `id` is is not a valid argument or group id.
    ///
    /// # Examples
    ///
    /// The argv indices are listed in the comments below. See how they correspond to the clap
    /// indices. Note that if it's not listed in a clap index, this is because it's not saved in
    /// in an `ArgMatches` struct for querying.
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-f", "-o", "val"]);
    ///            // ARGV indices: ^0       ^1    ^2    ^3
    ///            // clap indices:          ^1          ^3
    ///
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("option"), Some(3));
    /// ```
    ///
    /// Now notice, if we use one of the other styles of options:
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-f", "-o=val"]);
    ///            // ARGV indices: ^0       ^1    ^2
    ///            // clap indices:          ^1       ^3
    ///
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("option"), Some(3));
    /// ```
    ///
    /// Things become much more complicated, or clear if we look at a more complex combination of
    /// flags. Let's also throw in the final option style for good measure.
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("flag2")
    ///         .short('F'))
    ///     .arg(Arg::new("flag3")
    ///         .short('z'))
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-fzF", "-oval"]);
    ///            // ARGV indices: ^0      ^1       ^2
    ///            // clap indices:         ^1,2,3    ^5
    ///            //
    ///            // clap sees the above as 'myapp -f -z -F -o val'
    ///            //                         ^0    ^1 ^2 ^3 ^4 ^5
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("flag2"), Some(3));
    /// assert_eq!(m.index_of("flag3"), Some(2));
    /// assert_eq!(m.index_of("option"), Some(5));
    /// ```
    ///
    /// One final combination of flags/options to see how they combine:
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("flag")
    ///         .short('f'))
    ///     .arg(Arg::new("flag2")
    ///         .short('F'))
    ///     .arg(Arg::new("flag3")
    ///         .short('z'))
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .takes_value(true))
    ///     .get_matches_from(vec!["myapp", "-fzFoval"]);
    ///            // ARGV indices: ^0       ^1
    ///            // clap indices:          ^1,2,3^5
    ///            //
    ///            // clap sees the above as 'myapp -f -z -F -o val'
    ///            //                         ^0    ^1 ^2 ^3 ^4 ^5
    /// assert_eq!(m.index_of("flag"), Some(1));
    /// assert_eq!(m.index_of("flag2"), Some(3));
    /// assert_eq!(m.index_of("flag3"), Some(2));
    /// assert_eq!(m.index_of("option"), Some(5));
    /// ```
    ///
    /// The last part to mention is when values are sent in multiple groups with a [delimiter].
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .use_value_delimiter(true)
    ///         .multiple_values(true))
    ///     .get_matches_from(vec!["myapp", "-o=val1,val2,val3"]);
    ///            // ARGV indices: ^0       ^1
    ///            // clap indices:             ^2   ^3   ^4
    ///            //
    ///            // clap sees the above as 'myapp -o val1 val2 val3'
    ///            //                         ^0    ^1 ^2   ^3   ^4
    /// assert_eq!(m.index_of("option"), Some(2));
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2, 3, 4]);
    /// ```
    /// [delimiter]: crate::Arg::value_delimiter()
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn index_of<T: Key>(&self, id: T) -> Option<usize> {
        let arg = self.get_arg(&Id::from(id))?;
        let i = arg.get_index(0)?;
        Some(i)
    }

    /// All indices an argument appeared at when parsing.
    ///
    /// Indices are similar to argv indices, but are not exactly 1:1.
    ///
    /// For flags (i.e. those arguments which don't have an associated value), indices refer
    /// to occurrence of the switch, such as `-f`, or `--flag`. However, for options the indices
    /// refer to the *values* `-o val` would therefore not represent two distinct indices, only the
    /// index for `val` would be recorded. This is by design.
    ///
    /// *NOTE:* For more information about how clap indices compared to argv indices, see
    /// [`ArgMatches::index_of`]
    ///
    /// # Panics
    ///
    /// If `id` is is not a valid argument or group id.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .use_value_delimiter(true)
    ///         .multiple_values(true))
    ///     .get_matches_from(vec!["myapp", "-o=val1,val2,val3"]);
    ///            // ARGV indices: ^0       ^1
    ///            // clap indices:             ^2   ^3   ^4
    ///            //
    ///            // clap sees the above as 'myapp -o val1 val2 val3'
    ///            //                         ^0    ^1 ^2   ^3   ^4
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2, 3, 4]);
    /// ```
    ///
    /// Another quick example is when flags and options are used together
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ArgAction};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .takes_value(true)
    ///         .action(ArgAction::Append))
    ///     .arg(Arg::new("flag")
    ///         .short('f')
    ///         .action(ArgAction::Count))
    ///     .get_matches_from(vec!["myapp", "-o", "val1", "-f", "-o", "val2", "-f"]);
    ///            // ARGV indices: ^0       ^1    ^2      ^3    ^4    ^5      ^6
    ///            // clap indices:                ^2      ^3          ^5      ^6
    ///
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2, 5]);
    /// assert_eq!(m.indices_of("flag").unwrap().collect::<Vec<_>>(), &[6]);
    /// ```
    ///
    /// One final example, which is an odd case; if we *don't* use  value delimiter as we did with
    /// the first example above instead of `val1`, `val2` and `val3` all being distinc values, they
    /// would all be a single value of `val1,val2,val3`, in which case they'd only receive a single
    /// index.
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myapp")
    ///     .arg(Arg::new("option")
    ///         .short('o')
    ///         .takes_value(true)
    ///         .multiple_values(true))
    ///     .get_matches_from(vec!["myapp", "-o=val1,val2,val3"]);
    ///            // ARGV indices: ^0       ^1
    ///            // clap indices:             ^2
    ///            //
    ///            // clap sees the above as 'myapp -o "val1,val2,val3"'
    ///            //                         ^0    ^1  ^2
    /// assert_eq!(m.indices_of("option").unwrap().collect::<Vec<_>>(), &[2]);
    /// ```
    /// [`ArgMatches::index_of`]: ArgMatches::index_of()
    /// [delimiter]: Arg::value_delimiter()
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn indices_of<T: Key>(&self, id: T) -> Option<Indices<'_>> {
        let arg = self.get_arg(&Id::from(id))?;
        let i = Indices {
            iter: arg.indices(),
            len: arg.num_vals(),
        };
        Some(i)
    }

    #[inline]
    #[doc(hidden)]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `ArgMatches::try_get_one()`

When helping to catch mistyped argument IDs, we overlooked the use cases for more dynamic lookups
of arguments, so we added `is_valid_arg` as a short-term hack.  Since then, we've generalized
`value_of` as `try_get_one` to give callers all the relevant information they need.

So replace
```
if matches.is_valid_arg(...) {
    matches.value_of(...)
}
```
with
```
if let Ok(value) = matches.try_get_one::<T>(...) {
}
```
"
        )
    )]
    pub fn is_valid_arg(&self, _id: impl Key) -> bool {
        #[cfg(debug_assertions)]
        {
            let id = Id::from(_id);
            self.disable_asserts || id == Id::empty_hash() || self.valid_args.contains(&id)
        }
        #[cfg(not(debug_assertions))]
        {
            true
        }
    }
}

/// # Subcommands
impl ArgMatches {
    /// The name and `ArgMatches` of the current [subcommand].
    ///
    /// Subcommand values are put in a child [`ArgMatches`]
    ///
    /// Returns `None` if the subcommand wasn't present at runtime,
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    ///  let app_m = Command::new("git")
    ///      .subcommand(Command::new("clone"))
    ///      .subcommand(Command::new("push"))
    ///      .subcommand(Command::new("commit"))
    ///      .get_matches();
    ///
    /// match app_m.subcommand() {
    ///     Some(("clone",  sub_m)) => {}, // clone was used
    ///     Some(("push",   sub_m)) => {}, // push was used
    ///     Some(("commit", sub_m)) => {}, // commit was used
    ///     _                       => {}, // Either no subcommand or one not tested for...
    /// }
    /// ```
    ///
    /// Another useful scenario is when you want to support third party, or external, subcommands.
    /// In these cases you can't know the subcommand name ahead of time, so use a variable instead
    /// with pattern matching!
    ///
    /// ```rust
    /// # use clap::Command;
    /// // Assume there is an external subcommand named "subcmd"
    /// let app_m = Command::new("myprog")
    ///     .allow_external_subcommands(true)
    ///     .get_matches_from(vec![
    ///         "myprog", "subcmd", "--option", "value", "-fff", "--flag"
    ///     ]);
    ///
    /// // All trailing arguments will be stored under the subcommand's sub-matches using an empty
    /// // string argument name
    /// match app_m.subcommand() {
    ///     Some((external, sub_m)) => {
    ///          let ext_args: Vec<&str> = sub_m.get_many::<String>("")
    ///             .unwrap().map(|s| s.as_str()).collect();
    ///          assert_eq!(external, "subcmd");
    ///          assert_eq!(ext_args, ["--option", "value", "-fff", "--flag"]);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    /// [subcommand]: crate::Command::subcommand
    #[inline]
    pub fn subcommand(&self) -> Option<(&str, &ArgMatches)> {
        self.subcommand.as_ref().map(|sc| (&*sc.name, &sc.matches))
    }

    /// Return the name and `ArgMatches` of the current [subcommand].
    ///
    /// Subcommand values are put in a child [`ArgMatches`]
    ///
    /// Returns `None` if the subcommand wasn't present at runtime,
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    ///  let mut app_m = Command::new("git")
    ///      .subcommand(Command::new("clone"))
    ///      .subcommand(Command::new("push"))
    ///      .subcommand(Command::new("commit"))
    ///      .subcommand_required(true)
    ///      .get_matches();
    ///
    /// let (name, sub_m) = app_m.remove_subcommand().expect("required");
    /// match (name.as_str(), sub_m) {
    ///     ("clone",  sub_m) => {}, // clone was used
    ///     ("push",   sub_m) => {}, // push was used
    ///     ("commit", sub_m) => {}, // commit was used
    ///     (name, _)         => unimplemented!("{}", name),
    /// }
    /// ```
    ///
    /// Another useful scenario is when you want to support third party, or external, subcommands.
    /// In these cases you can't know the subcommand name ahead of time, so use a variable instead
    /// with pattern matching!
    ///
    /// ```rust
    /// # use clap::Command;
    /// // Assume there is an external subcommand named "subcmd"
    /// let mut app_m = Command::new("myprog")
    ///     .allow_external_subcommands(true)
    ///     .get_matches_from(vec![
    ///         "myprog", "subcmd", "--option", "value", "-fff", "--flag"
    ///     ]);
    ///
    /// // All trailing arguments will be stored under the subcommand's sub-matches using an empty
    /// // string argument name
    /// match app_m.remove_subcommand() {
    ///     Some((external, mut sub_m)) => {
    ///          let ext_args: Vec<String> = sub_m.remove_many("")
    ///             .expect("`file`is required")
    ///             .collect();
    ///          assert_eq!(external, "subcmd");
    ///          assert_eq!(ext_args, ["--option", "value", "-fff", "--flag"]);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    /// [subcommand]: crate::Command::subcommand
    pub fn remove_subcommand(&mut self) -> Option<(String, ArgMatches)> {
        self.subcommand.take().map(|sc| (sc.name, sc.matches))
    }

    /// The `ArgMatches` for the current [subcommand].
    ///
    /// Subcommand values are put in a child [`ArgMatches`]
    ///
    /// Returns `None` if the subcommand wasn't present at runtime,
    ///
    /// # Panics
    ///
    /// If `id` is is not a valid subcommand.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ArgAction};
    /// let app_m = Command::new("myprog")
    ///     .arg(Arg::new("debug")
    ///         .short('d')
    ///         .action(ArgAction::SetTrue)
    ///     )
    ///     .subcommand(Command::new("test")
    ///         .arg(Arg::new("opt")
    ///             .long("option")
    ///             .takes_value(true)))
    ///     .get_matches_from(vec![
    ///         "myprog", "-d", "test", "--option", "val"
    ///     ]);
    ///
    /// // Both parent commands, and child subcommands can have arguments present at the same times
    /// assert!(*app_m.get_one::<bool>("debug").expect("defaulted by clap"));
    ///
    /// // Get the subcommand's ArgMatches instance
    /// if let Some(sub_m) = app_m.subcommand_matches("test") {
    ///     // Use the struct like normal
    ///     assert_eq!(sub_m.get_one::<String>("opt").map(|s| s.as_str()), Some("val"));
    /// }
    /// ```
    ///
    /// [subcommand]: crate::Command::subcommand
    /// [`Command`]: crate::Command
    pub fn subcommand_matches<T: Key>(&self, id: T) -> Option<&ArgMatches> {
        self.get_subcommand(&id.into()).map(|sc| &sc.matches)
    }

    /// The name of the current [subcommand].
    ///
    /// Returns `None` if the subcommand wasn't present at runtime,
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    ///  let app_m = Command::new("git")
    ///      .subcommand(Command::new("clone"))
    ///      .subcommand(Command::new("push"))
    ///      .subcommand(Command::new("commit"))
    ///      .get_matches();
    ///
    /// match app_m.subcommand_name() {
    ///     Some("clone")  => {}, // clone was used
    ///     Some("push")   => {}, // push was used
    ///     Some("commit") => {}, // commit was used
    ///     _              => {}, // Either no subcommand or one not tested for...
    /// }
    /// ```
    /// [subcommand]: crate::Command::subcommand
    /// [`Command`]: crate::Command
    #[inline]
    pub fn subcommand_name(&self) -> Option<&str> {
        self.subcommand.as_ref().map(|sc| &*sc.name)
    }

    /// Check if a subcommand can be queried
    ///
    /// By default, `ArgMatches` functions assert on undefined `Id`s to help catch programmer
    /// mistakes.  In some context, this doesn't work, so users can use this function to check
    /// before they do a query on `ArgMatches`.
    #[inline]
    #[doc(hidden)]
    pub fn is_valid_subcommand(&self, _id: impl Key) -> bool {
        #[cfg(debug_assertions)]
        {
            let id = Id::from(_id);
            self.disable_asserts || id == Id::empty_hash() || self.valid_subcommands.contains(&id)
        }
        #[cfg(not(debug_assertions))]
        {
            true
        }
    }
}

/// # Advanced
impl ArgMatches {
    /// Non-panicking version of [`ArgMatches::get_one`]
    pub fn try_get_one<T: Any + Clone + Send + Sync + 'static>(
        &self,
        id: &str,
    ) -> Result<Option<&T>, MatchesError> {
        let id = Id::from(id);
        let arg = self.try_get_arg_t::<T>(&id)?;
        let value = match arg.and_then(|a| a.first()) {
            Some(value) => value,
            None => {
                return Ok(None);
            }
        };
        Ok(value
            .downcast_ref::<T>()
            .map(Some)
            .expect(INTERNAL_ERROR_MSG)) // enforced by `try_get_arg_t`
    }

    /// Non-panicking version of [`ArgMatches::get_many`]
    pub fn try_get_many<T: Any + Clone + Send + Sync + 'static>(
        &self,
        id: &str,
    ) -> Result<Option<ValuesRef<T>>, MatchesError> {
        let id = Id::from(id);
        let arg = match self.try_get_arg_t::<T>(&id)? {
            Some(arg) => arg,
            None => return Ok(None),
        };
        let len = arg.num_vals();
        let values = arg.vals_flatten();
        let values = ValuesRef {
            // enforced by `try_get_arg_t`
            iter: values.map(|v| v.downcast_ref::<T>().expect(INTERNAL_ERROR_MSG)),
            len,
        };
        Ok(Some(values))
    }

    /// Non-panicking version of [`ArgMatches::get_raw`]
    pub fn try_get_raw(&self, id: &str) -> Result<Option<RawValues<'_>>, MatchesError> {
        let id = Id::from(id);
        let arg = match self.try_get_arg(&id)? {
            Some(arg) => arg,
            None => return Ok(None),
        };
        let len = arg.num_vals();
        let values = arg.raw_vals_flatten();
        let values = RawValues {
            iter: values.map(OsString::as_os_str),
            len,
        };
        Ok(Some(values))
    }

    /// Non-panicking version of [`ArgMatches::remove_one`]
    pub fn try_remove_one<T: Any + Clone + Send + Sync + 'static>(
        &mut self,
        id: &str,
    ) -> Result<Option<T>, MatchesError> {
        let id = Id::from(id);
        match self.try_remove_arg_t::<T>(&id)? {
            Some(values) => Ok(values
                .into_vals_flatten()
                // enforced by `try_get_arg_t`
                .map(|v| v.downcast_into::<T>().expect(INTERNAL_ERROR_MSG))
                .next()),
            None => Ok(None),
        }
    }

    /// Non-panicking version of [`ArgMatches::remove_many`]
    pub fn try_remove_many<T: Any + Clone + Send + Sync + 'static>(
        &mut self,
        id: &str,
    ) -> Result<Option<Values2<T>>, MatchesError> {
        let id = Id::from(id);
        let arg = match self.try_remove_arg_t::<T>(&id)? {
            Some(arg) => arg,
            None => return Ok(None),
        };
        let len = arg.num_vals();
        let values = arg.into_vals_flatten();
        let values = Values2 {
            // enforced by `try_get_arg_t`
            iter: values.map(|v| v.downcast_into::<T>().expect(INTERNAL_ERROR_MSG)),
            len,
        };
        Ok(Some(values))
    }

    /// Non-panicking version of [`ArgMatches::contains_id`]
    pub fn try_contains_id(&self, id: &str) -> Result<bool, MatchesError> {
        let id = Id::from(id);

        self.verify_arg(&id)?;

        let presence = self.args.contains_key(&id);
        Ok(presence)
    }
}

// Private methods
impl ArgMatches {
    #[inline]
    fn try_get_arg(&self, arg: &Id) -> Result<Option<&MatchedArg>, MatchesError> {
        self.verify_arg(arg)?;
        Ok(self.args.get(arg))
    }

    #[inline]
    fn try_get_arg_t<T: Any + Send + Sync + 'static>(
        &self,
        arg: &Id,
    ) -> Result<Option<&MatchedArg>, MatchesError> {
        let arg = match self.try_get_arg(arg)? {
            Some(arg) => arg,
            None => {
                return Ok(None);
            }
        };
        self.verify_arg_t::<T>(arg)?;
        Ok(Some(arg))
    }

    #[inline]
    fn try_remove_arg_t<T: Any + Send + Sync + 'static>(
        &mut self,
        arg: &Id,
    ) -> Result<Option<MatchedArg>, MatchesError> {
        self.verify_arg(arg)?;
        let matched = match self.args.remove(arg) {
            Some(matched) => matched,
            None => {
                return Ok(None);
            }
        };

        let expected = AnyValueId::of::<T>();
        let actual = matched.infer_type_id(expected);
        if actual == expected {
            Ok(Some(matched))
        } else {
            self.args.insert(arg.clone(), matched);
            Err(MatchesError::Downcast { actual, expected })
        }
    }

    fn verify_arg_t<T: Any + Send + Sync + 'static>(
        &self,
        arg: &MatchedArg,
    ) -> Result<(), MatchesError> {
        let expected = AnyValueId::of::<T>();
        let actual = arg.infer_type_id(expected);
        if expected == actual {
            Ok(())
        } else {
            Err(MatchesError::Downcast { actual, expected })
        }
    }

    #[inline]
    fn verify_arg(&self, _arg: &Id) -> Result<(), MatchesError> {
        #[cfg(debug_assertions)]
        {
            if self.disable_asserts || *_arg == Id::empty_hash() || self.valid_args.contains(_arg) {
            } else if self.valid_subcommands.contains(_arg) {
                debug!(
                    "Subcommand `{:?}` used where an argument or group name was expected.",
                    _arg
                );
                return Err(MatchesError::UnknownArgument {});
            } else {
                debug!(
                    "`{:?}` is not an id of an argument or a group.\n\
                     Make sure you're using the name of the argument itself \
                     and not the name of short or long flags.",
                    _arg
                );
                return Err(MatchesError::UnknownArgument {});
            }
        }
        Ok(())
    }

    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn get_arg(&self, arg: &Id) -> Option<&MatchedArg> {
        #[cfg(debug_assertions)]
        {
            if self.disable_asserts || *arg == Id::empty_hash() || self.valid_args.contains(arg) {
            } else if self.valid_subcommands.contains(arg) {
                panic!(
                    "Subcommand `{:?}` used where an argument or group name was expected.",
                    arg
                );
            } else {
                panic!(
                    "`{:?}` is not an id of an argument or a group.\n\
                     Make sure you're using the name of the argument itself \
                     and not the name of short or long flags.",
                    arg
                );
            }
        }

        self.args.get(arg)
    }

    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn get_subcommand(&self, id: &Id) -> Option<&SubCommand> {
        #[cfg(debug_assertions)]
        {
            if self.disable_asserts
                || *id == Id::empty_hash()
                || self.valid_subcommands.contains(id)
            {
            } else if self.valid_args.contains(id) {
                panic!(
                    "Argument or group `{:?}` used where a subcommand name was expected.",
                    id
                );
            } else {
                panic!("`{:?}` is not a name of a subcommand.", id);
            }
        }

        if let Some(ref sc) = self.subcommand {
            if sc.id == *id {
                return Some(sc);
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SubCommand {
    pub(crate) id: Id,
    pub(crate) name: String,
    pub(crate) matches: ArgMatches,
}

/// Iterate over multiple values for an argument via [`ArgMatches::remove_many`].
///
/// # Examples
///
/// ```rust
/// # use clap::{Command, Arg, ArgAction};
/// let mut m = Command::new("myapp")
///     .arg(Arg::new("output")
///         .short('o')
///         .action(ArgAction::Append)
///         .takes_value(true))
///     .get_matches_from(vec!["myapp", "-o", "val1", "-o", "val2"]);
///
/// let mut values = m.remove_many::<String>("output")
///     .unwrap();
///
/// assert_eq!(values.next(), Some(String::from("val1")));
/// assert_eq!(values.next(), Some(String::from("val2")));
/// assert_eq!(values.next(), None);
/// ```
#[derive(Clone, Debug)]
pub struct Values2<T> {
    #[allow(clippy::type_complexity)]
    iter: Map<Flatten<std::vec::IntoIter<Vec<AnyValue>>>, fn(AnyValue) -> T>,
    len: usize,
}

impl<T> Iterator for Values2<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for Values2<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> ExactSizeIterator for Values2<T> {}

/// Creates an empty iterator.
impl<T> Default for Values2<T> {
    fn default() -> Self {
        let empty: Vec<Vec<AnyValue>> = Default::default();
        Values2 {
            iter: empty.into_iter().flatten().map(|_| unreachable!()),
            len: 0,
        }
    }
}

/// Iterate over multiple values for an argument via [`ArgMatches::get_many`].
///
/// # Examples
///
/// ```rust
/// # use clap::{Command, Arg, ArgAction};
/// let m = Command::new("myapp")
///     .arg(Arg::new("output")
///         .short('o')
///         .action(ArgAction::Append)
///         .takes_value(true))
///     .get_matches_from(vec!["myapp", "-o", "val1", "-o", "val2"]);
///
/// let mut values = m.get_many::<String>("output")
///     .unwrap()
///     .map(|s| s.as_str());
///
/// assert_eq!(values.next(), Some("val1"));
/// assert_eq!(values.next(), Some("val2"));
/// assert_eq!(values.next(), None);
/// ```
#[derive(Clone, Debug)]
pub struct ValuesRef<'a, T> {
    #[allow(clippy::type_complexity)]
    iter: Map<Flatten<Iter<'a, Vec<AnyValue>>>, fn(&AnyValue) -> &T>,
    len: usize,
}

impl<'a, T: 'a> Iterator for ValuesRef<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for ValuesRef<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, T: 'a> ExactSizeIterator for ValuesRef<'a, T> {}

/// Creates an empty iterator.
impl<'a, T: 'a> Default for ValuesRef<'a, T> {
    fn default() -> Self {
        static EMPTY: [Vec<AnyValue>; 0] = [];
        ValuesRef {
            iter: EMPTY[..].iter().flatten().map(|_| unreachable!()),
            len: 0,
        }
    }
}

/// Iterate over raw argument values via [`ArgMatches::get_raw`].
///
/// # Examples
///
#[cfg_attr(not(unix), doc = " ```ignore")]
#[cfg_attr(unix, doc = " ```")]
/// # use clap::{Command, arg, value_parser};
/// use std::ffi::OsString;
/// use std::os::unix::ffi::{OsStrExt,OsStringExt};
///
/// let m = Command::new("utf8")
///     .arg(arg!(<arg> "some arg")
///         .value_parser(value_parser!(OsString)))
///     .get_matches_from(vec![OsString::from("myprog"),
///                             // "Hi {0xe9}!"
///                             OsString::from_vec(vec![b'H', b'i', b' ', 0xe9, b'!'])]);
/// assert_eq!(
///     &*m.get_raw("arg")
///         .unwrap()
///         .next().unwrap()
///         .as_bytes(),
///     [b'H', b'i', b' ', 0xe9, b'!']
/// );
/// ```
#[derive(Clone, Debug)]
pub struct RawValues<'a> {
    #[allow(clippy::type_complexity)]
    iter: Map<Flatten<Iter<'a, Vec<OsString>>>, fn(&OsString) -> &OsStr>,
    len: usize,
}

impl<'a> Iterator for RawValues<'a> {
    type Item = &'a OsStr;

    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> DoubleEndedIterator for RawValues<'a> {
    fn next_back(&mut self) -> Option<&'a OsStr> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for RawValues<'a> {}

/// Creates an empty iterator.
impl Default for RawValues<'_> {
    fn default() -> Self {
        static EMPTY: [Vec<OsString>; 0] = [];
        RawValues {
            iter: EMPTY[..].iter().flatten().map(|_| unreachable!()),
            len: 0,
        }
    }
}

// The following were taken and adapted from vec_map source
// repo: https://github.com/contain-rs/vec-map
// commit: be5e1fa3c26e351761b33010ddbdaf5f05dbcc33
// license: MIT - Copyright (c) 2015 The Rust Project Developers

/// Deprecated, replaced with [`ArgMatches::get_many()`]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_many()`")
)]
#[derive(Clone, Debug)]
pub struct Values<'a> {
    #[allow(clippy::type_complexity)]
    iter: Map<Flatten<Iter<'a, Vec<AnyValue>>>, for<'r> fn(&'r AnyValue) -> &'r str>,
    len: usize,
}

#[allow(deprecated)]
impl<'a> Iterator for Values<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

#[allow(deprecated)]
impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        self.iter.next_back()
    }
}

#[allow(deprecated)]
impl<'a> ExactSizeIterator for Values<'a> {}

/// Creates an empty iterator.
#[allow(deprecated)]
impl<'a> Default for Values<'a> {
    fn default() -> Self {
        static EMPTY: [Vec<AnyValue>; 0] = [];
        Values {
            iter: EMPTY[..].iter().flatten().map(|_| unreachable!()),
            len: 0,
        }
    }
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct GroupedValues<'a> {
    #[allow(clippy::type_complexity)]
    iter: Map<Iter<'a, Vec<AnyValue>>, fn(&Vec<AnyValue>) -> Vec<&str>>,
    len: usize,
}

impl<'a> Iterator for GroupedValues<'a> {
    type Item = Vec<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> DoubleEndedIterator for GroupedValues<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for GroupedValues<'a> {}

/// Creates an empty iterator. Used for `unwrap_or_default()`.
impl<'a> Default for GroupedValues<'a> {
    fn default() -> Self {
        #![allow(deprecated)]
        static EMPTY: [Vec<AnyValue>; 0] = [];
        GroupedValues {
            iter: EMPTY[..].iter().map(|_| unreachable!()),
            len: 0,
        }
    }
}

/// Deprecated, replaced with [`ArgMatches::get_many()`]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.2.0", note = "Replaced with `ArgMatches::get_many()`")
)]
#[derive(Clone, Debug)]
pub struct OsValues<'a> {
    #[allow(clippy::type_complexity)]
    iter: Map<Flatten<Iter<'a, Vec<AnyValue>>>, fn(&AnyValue) -> &OsStr>,
    len: usize,
}

#[allow(deprecated)]
impl<'a> Iterator for OsValues<'a> {
    type Item = &'a OsStr;

    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

#[allow(deprecated)]
impl<'a> DoubleEndedIterator for OsValues<'a> {
    fn next_back(&mut self) -> Option<&'a OsStr> {
        self.iter.next_back()
    }
}

#[allow(deprecated)]
impl<'a> ExactSizeIterator for OsValues<'a> {}

/// Creates an empty iterator.
#[allow(deprecated)]
impl Default for OsValues<'_> {
    fn default() -> Self {
        static EMPTY: [Vec<AnyValue>; 0] = [];
        OsValues {
            iter: EMPTY[..].iter().flatten().map(|_| unreachable!()),
            len: 0,
        }
    }
}

/// Iterate over indices for where an argument appeared when parsing, via [`ArgMatches::indices_of`]
///
/// # Examples
///
/// ```rust
/// # use clap::{Command, Arg};
/// let m = Command::new("myapp")
///     .arg(Arg::new("output")
///         .short('o')
///         .multiple_values(true)
///         .takes_value(true))
///     .get_matches_from(vec!["myapp", "-o", "val1", "val2"]);
///
/// let mut indices = m.indices_of("output").unwrap();
///
/// assert_eq!(indices.next(), Some(2));
/// assert_eq!(indices.next(), Some(3));
/// assert_eq!(indices.next(), None);
/// ```
/// [`ArgMatches::indices_of`]: ArgMatches::indices_of()
#[derive(Clone, Debug)]
pub struct Indices<'a> {
    iter: Cloned<Iter<'a, usize>>,
    len: usize,
}

impl<'a> Iterator for Indices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> DoubleEndedIterator for Indices<'a> {
    fn next_back(&mut self) -> Option<usize> {
        self.iter.next_back()
    }
}

impl<'a> ExactSizeIterator for Indices<'a> {}

/// Creates an empty iterator.
impl<'a> Default for Indices<'a> {
    fn default() -> Self {
        static EMPTY: [usize; 0] = [];
        // This is never called because the iterator is empty:
        Indices {
            iter: EMPTY[..].iter().cloned(),
            len: 0,
        }
    }
}

#[cfg_attr(debug_assertions, track_caller)]
#[inline]
fn unwrap_string(value: &AnyValue) -> &str {
    match value.downcast_ref::<String>() {
        Some(value) => value,
        None => {
            panic!("Must use `_os` lookups with `Arg::allow_invalid_utf8`",)
        }
    }
}

#[cfg_attr(debug_assertions, track_caller)]
#[inline]
fn unwrap_string_arg<'v>(id: &Id, value: &'v AnyValue) -> &'v str {
    match value.downcast_ref::<String>() {
        Some(value) => value,
        None => {
            panic!(
                "Must use `_os` lookups with `Arg::allow_invalid_utf8` at `{:?}`",
                id
            )
        }
    }
}

#[cfg_attr(debug_assertions, track_caller)]
#[inline]
fn unwrap_os_string(value: &AnyValue) -> &OsStr {
    match value.downcast_ref::<OsString>() {
        Some(value) => value,
        None => {
            panic!("Must use `Arg::allow_invalid_utf8` with `_os` lookups",)
        }
    }
}

#[cfg_attr(debug_assertions, track_caller)]
#[inline]
fn unwrap_os_string_arg<'v>(id: &Id, value: &'v AnyValue) -> &'v OsStr {
    match value.downcast_ref::<OsString>() {
        Some(value) => value,
        None => {
            panic!(
                "Must use `Arg::allow_invalid_utf8` with `_os` lookups at `{:?}`",
                id
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_auto_traits() {
        static_assertions::assert_impl_all!(ArgMatches: Send, Sync, Unpin);
    }

    #[test]
    fn test_default_values() {
        #![allow(deprecated)]
        let mut values: Values = Values::default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_osvalues() {
        #![allow(deprecated)]
        let mut values: OsValues = OsValues::default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_raw_values() {
        let mut values: RawValues = Default::default();
        assert_eq!(values.next(), None);
    }

    #[test]
    fn test_default_indices() {
        let mut indices: Indices = Indices::default();
        assert_eq!(indices.next(), None);
    }

    #[test]
    fn test_default_indices_with_shorter_lifetime() {
        let matches = ArgMatches::default();
        let mut indices = matches.indices_of("").unwrap_or_default();
        assert_eq!(indices.next(), None);
    }

    #[test]
    fn values_exact_size() {
        let l = crate::Command::new("test")
            .arg(
                crate::Arg::new("POTATO")
                    .takes_value(true)
                    .multiple_values(true)
                    .required(true),
            )
            .try_get_matches_from(["test", "one"])
            .unwrap()
            .get_many::<String>("POTATO")
            .expect("present")
            .count();
        assert_eq!(l, 1);
    }

    #[test]
    fn os_values_exact_size() {
        let l = crate::Command::new("test")
            .arg(
                crate::Arg::new("POTATO")
                    .takes_value(true)
                    .multiple_values(true)
                    .value_parser(crate::builder::ValueParser::os_string())
                    .required(true),
            )
            .try_get_matches_from(["test", "one"])
            .unwrap()
            .get_many::<std::ffi::OsString>("POTATO")
            .expect("present")
            .count();
        assert_eq!(l, 1);
    }

    #[test]
    fn indices_exact_size() {
        let l = crate::Command::new("test")
            .arg(
                crate::Arg::new("POTATO")
                    .takes_value(true)
                    .multiple_values(true)
                    .required(true),
            )
            .try_get_matches_from(["test", "one"])
            .unwrap()
            .indices_of("POTATO")
            .expect("present")
            .len();
        assert_eq!(l, 1);
    }
}
