#[cfg(feature = "yaml")]
use std::collections::BTreeMap;
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
use std::os::unix::ffi::OsStrExt;
use std::{
    env,
    ffi::{OsStr, OsString},
    rc::Rc,
};

#[cfg(feature = "yaml")]
use yaml_rust::Yaml;

#[cfg(any(target_os = "windows", target_arch = "wasm32"))]
use crate::osstringext::OsStrExt3;
use crate::{
    args::{
        arg_builder::{Base, Switched, Valued},
        settings::ArgSettings,
    },
    map::VecMap,
    usage_parser::UsageParser,
};

/// The abstract representation of a command line argument. Used to set all the options and
/// relationships that define a valid argument for the program.
///
/// There are two methods for constructing [`Arg`]s, using the builder pattern and setting options
/// manually, or using a usage string which is far less verbose but has fewer options. You can also
/// use a combination of the two methods to achieve the best of both worlds.
///
/// # Examples
///
/// ```rust
/// # use clap::Arg;
/// // Using the traditional builder pattern and setting each option manually
/// let cfg = Arg::with_name("config")
///       .short("c")
///       .long("config")
///       .takes_value(true)
///       .value_name("FILE")
///       .help("Provides a config file to myprog");
/// // Using a usage string (setting a similar argument to the one above)
/// let input = Arg::from_usage("-i, --input=[FILE] 'Provides an input file to the program'");
/// ```
/// [`Arg`]: ./struct.Arg.html
#[allow(missing_debug_implementations)]
#[derive(Default, Clone)]
pub struct Arg<'a, 'b>
where
    'a: 'b,
{
    #[doc(hidden)]
    pub b: Base<'a, 'b>,
    #[doc(hidden)]
    pub s: Switched<'b>,
    #[doc(hidden)]
    pub v: Valued<'a, 'b>,
    #[doc(hidden)]
    pub index: Option<u64>,
    #[doc(hidden)]
    pub r_ifs: Option<Vec<(&'a str, &'b str)>>,
}

impl<'a, 'b> Arg<'a, 'b> {
    /// Creates a new instance of [`Arg`] using a unique string name. The name will be used to get
    /// information about whether or not the argument was used at runtime, get values, set
    /// relationships with other args, etc..
    ///
    /// **NOTE:** In the case of arguments that take values (i.e. [`Arg::takes_value(true)`])
    /// and positional arguments (i.e. those without a preceding `-` or `--`) the name will also
    /// be displayed when the user prints the usage/help information of the program.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    /// # ;
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg`]: ./struct.Arg.html
    pub fn with_name(n: &'a str) -> Self {
        Arg {
            b: Base::new(n),
            ..Default::default()
        }
    }

    /// Creates a new instance of [`Arg`] from a .yml (YAML) file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #[macro_use]
    /// # extern crate clap;
    /// # use clap::Arg;
    /// # fn main() {
    /// let yml = load_yaml!("arg.yml");
    /// let arg = Arg::from_yaml(yml);
    /// # }
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    #[cfg(feature = "yaml")]
    pub fn from_yaml(y: &BTreeMap<Yaml, Yaml>) -> Arg {
        // We WANT this to panic on error...so expect() is good.
        let name_yml = y.keys().nth(0).unwrap();
        let name_str = name_yml.as_str().unwrap();
        let mut a = Arg::with_name(name_str);
        let arg_settings = y.get(name_yml).unwrap().as_hash().unwrap();

        for (k, v) in arg_settings.iter() {
            a = match k.as_str().unwrap() {
                "short" => yaml_to_str!(a, v, short),
                "long" => yaml_to_str!(a, v, long),
                "aliases" => yaml_vec_or_str!(v, a, alias),
                "help" => yaml_to_str!(a, v, help),
                "long_help" => yaml_to_str!(a, v, long_help),
                "required" => yaml_to_bool!(a, v, required),
                "required_if" => yaml_tuple2!(a, v, required_if),
                "required_ifs" => yaml_tuple2!(a, v, required_if),
                "takes_value" => yaml_to_bool!(a, v, takes_value),
                "index" => yaml_to_u64!(a, v, index),
                "global" => yaml_to_bool!(a, v, global),
                "multiple" => yaml_to_bool!(a, v, multiple),
                "hidden" => yaml_to_bool!(a, v, hidden),
                "next_line_help" => yaml_to_bool!(a, v, next_line_help),
                "empty_values" => yaml_to_bool!(a, v, empty_values),
                "group" => yaml_to_str!(a, v, group),
                "number_of_values" => yaml_to_u64!(a, v, number_of_values),
                "max_values" => yaml_to_u64!(a, v, max_values),
                "min_values" => yaml_to_u64!(a, v, min_values),
                "value_name" => yaml_to_str!(a, v, value_name),
                "use_delimiter" => yaml_to_bool!(a, v, use_delimiter),
                "allow_hyphen_values" => yaml_to_bool!(a, v, allow_hyphen_values),
                "last" => yaml_to_bool!(a, v, last),
                "require_delimiter" => yaml_to_bool!(a, v, require_delimiter),
                "value_delimiter" => yaml_to_str!(a, v, value_delimiter),
                "required_unless" => yaml_to_str!(a, v, required_unless),
                "display_order" => yaml_to_usize!(a, v, display_order),
                "default_value" => yaml_to_str!(a, v, default_value),
                "default_value_if" => yaml_tuple3!(a, v, default_value_if),
                "default_value_ifs" => yaml_tuple3!(a, v, default_value_if),
                "env" => yaml_to_str!(a, v, env),
                "value_names" => yaml_vec_or_str!(v, a, value_name),
                "groups" => yaml_vec_or_str!(v, a, group),
                "requires" => yaml_vec_or_str!(v, a, requires),
                "requires_if" => yaml_tuple2!(a, v, requires_if),
                "requires_ifs" => yaml_tuple2!(a, v, requires_if),
                "conflicts_with" => yaml_vec_or_str!(v, a, conflicts_with),
                "overrides_with" => yaml_vec_or_str!(v, a, overrides_with),
                "possible_values" => yaml_vec_or_str!(v, a, possible_value),
                "case_insensitive" => yaml_to_bool!(a, v, case_insensitive),
                "required_unless_one" => yaml_vec_or_str!(v, a, required_unless),
                "required_unless_all" => {
                    a = yaml_vec_or_str!(v, a, required_unless);
                    a.setb(ArgSettings::RequiredUnlessAll);
                    a
                }
                s => panic!(
                    "Unknown Arg setting '{}' in YAML file for arg '{}'",
                    s, name_str
                ),
            }
        }

        a
    }

    /// Creates a new instance of [`Arg`] from a usage string. Allows creation of basic settings
    /// for the [`Arg`]. The syntax is flexible, but there are some rules to follow.
    ///
    /// **NOTE**: Not all settings may be set using the usage string method. Some properties are
    /// only available via the builder pattern.
    ///
    /// **NOTE**: Only ASCII values are officially supported in [`Arg::from_usage`] strings. Some
    /// UTF-8 codepoints may work just fine, but this is not guaranteed.
    ///
    /// # Syntax
    ///
    /// Usage strings typically following the form:
    ///
    /// ```notrust
    /// [explicit name] [short] [long] [value names] [help string]
    /// ```
    ///
    /// This is not a hard rule as the attributes can appear in other orders. There are also
    /// several additional sigils which denote additional settings. Below are the details of each
    /// portion of the string.
    ///
    /// ### Explicit Name
    ///
    /// This is an optional field, if it's omitted the argument will use one of the additional
    /// fields as the name using the following priority order:
    ///
    ///  * Explicit Name (This always takes precedence when present)
    ///  * Long
    ///  * Short
    ///  * Value Name
    ///
    /// `clap` determines explicit names as the first string of characters between either `[]` or
    /// `<>` where `[]` has the dual notation of meaning the argument is optional, and `<>` meaning
    /// the argument is required.
    ///
    /// Explicit names may be followed by:
    ///  * The multiple denotation `...`
    ///
    /// Example explicit names as follows (`ename` for an optional argument, and `rname` for a
    /// required argument):
    ///
    /// ```notrust
    /// [ename] -s, --long 'some flag'
    /// <rname> -r, --longer 'some other flag'
    /// ```
    ///
    /// ### Short
    ///
    /// This is set by placing a single character after a leading `-`.
    ///
    /// Shorts may be followed by
    ///  * The multiple denotation `...`
    ///  * An optional comma `,` which is cosmetic only
    ///  * Value notation
    ///
    /// Example shorts are as follows (`-s`, and `-r`):
    ///
    /// ```notrust
    /// -s, --long 'some flag'
    /// <rname> -r [val], --longer 'some option'
    /// ```
    ///
    /// ### Long
    ///
    /// This is set by placing a word (no spaces) after a leading `--`.
    ///
    /// Shorts may be followed by
    ///  * The multiple denotation `...`
    ///  * Value notation
    ///
    /// Example longs are as follows (`--some`, and `--rapid`):
    ///
    /// ```notrust
    /// -s, --some 'some flag'
    /// --rapid=[FILE] 'some option'
    /// ```
    ///
    /// ### Values (Value Notation)
    ///
    /// This is set by placing a word(s) between `[]` or `<>` optionally after `=` (although this
    /// is cosmetic only and does not affect functionality). If an explicit name has **not** been
    /// set, using `<>` will denote a required argument, and `[]` will denote an optional argument
    ///
    /// Values may be followed by
    ///  * The multiple denotation `...`
    ///  * More Value notation
    ///
    /// More than one value will also implicitly set the arguments number of values, i.e. having
    /// two values, `--option [val1] [val2]` specifies that in order for option to be satisified it
    /// must receive exactly two values
    ///
    /// Example values are as follows (`FILE`, and `SPEED`):
    ///
    /// ```notrust
    /// -s, --some [FILE] 'some option'
    /// --rapid=<SPEED>... 'some required multiple option'
    /// ```
    ///
    /// ### Help String
    ///
    /// The help string is denoted between a pair of single quotes `''` and may contain any
    /// characters.
    ///
    /// Example help strings are as follows:
    ///
    /// ```notrust
    /// -s, --some [FILE] 'some option'
    /// --rapid=<SPEED>... 'some required multiple option'
    /// ```
    ///
    /// ### Additional Sigils
    ///
    /// Multiple notation `...` (three consecutive dots/periods) specifies that this argument may
    /// be used multiple times. Do not confuse multiple occurrences (`...`) with multiple values.
    /// `--option val1 val2` is a single occurrence with multiple values. `--flag --flag` is
    /// multiple occurrences (and then you can obviously have instances of both as well)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// App::new("prog")
    ///     .args(&[
    ///         Arg::from_usage("--config <FILE> 'a required file for the configuration and no short'"),
    ///         Arg::from_usage("-d, --debug... 'turns on debugging information and allows multiples'"),
    ///         Arg::from_usage("[input] 'an optional input file to use'")
    /// ])
    /// # ;
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    /// [`Arg::from_usage`]: ./struct.Arg.html#method.from_usage
    pub fn from_usage(u: &'a str) -> Self {
        let parser = UsageParser::from_usage(u);
        parser.parse()
    }

    /// Sets the short version of the argument without the preceding `-`.
    ///
    /// By default `clap` automatically assigns `V` and `h` to the auto-generated `version` and
    /// `help` arguments respectively. You may use the uppercase `V` or lowercase `h` for your own
    /// arguments, in which case `clap` simply will not assign those to the auto-generated
    /// `version` or `help` arguments.
    ///
    /// **NOTE:** Any leading `-` characters will be stripped, and only the first
    /// non `-` character will be used as the [`short`] version
    ///
    /// # Examples
    ///
    /// To set [`short`] use a single valid UTF-8 code point. If you supply a leading `-` such as
    /// `-c`, the `-` will be stripped.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .short("c")
    /// # ;
    /// ```
    ///
    /// Setting [`short`] allows using the argument via a single hyphen (`-`) such as `-c`
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("config")
    ///         .short("c"))
    ///     .get_matches_from(vec![
    ///         "prog", "-c"
    ///     ]);
    ///
    /// assert!(m.is_present("config"));
    /// ```
    /// [`short`]: ./struct.Arg.html#method.short
    pub fn short<S: AsRef<str>>(mut self, s: S) -> Self {
        self.s.short = s.as_ref().trim_left_matches(|c| c == '-').chars().next();
        self
    }

    /// Sets the long version of the argument without the preceding `--`.
    ///
    /// By default `clap` automatically assigns `version` and `help` to the auto-generated
    /// `version` and `help` arguments respectively. You may use the word `version` or `help` for
    /// the long form of your own arguments, in which case `clap` simply will not assign those to
    /// the auto-generated `version` or `help` arguments.
    ///
    /// **NOTE:** Any leading `-` characters will be stripped
    ///
    /// # Examples
    ///
    /// To set `long` use a word containing valid UTF-8 codepoints. If you supply a double leading
    /// `--` such as `--config` they will be stripped. Hyphens in the middle of the word, however,
    /// will *not* be stripped (i.e. `config-file` is allowed)
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("cfg")
    ///     .long("config")
    /// # ;
    /// ```
    ///
    /// Setting `long` allows using the argument via a double hyphen (`--`) such as `--config`
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config"))
    ///     .get_matches_from(vec![
    ///         "prog", "--config"
    ///     ]);
    ///
    /// assert!(m.is_present("cfg"));
    /// ```
    pub fn long(mut self, l: &'b str) -> Self {
        self.s.long = Some(l.trim_left_matches(|c| c == '-'));
        self
    }

    /// Allows adding a [`Arg`] alias, which function as "hidden" arguments that
    /// automatically dispatch as if this argument was used. This is more efficient, and easier
    /// than creating multiple hidden arguments as one only needs to check for the existence of
    /// this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///             .long("test")
    ///             .alias("alias")
    ///             .takes_value(true))
    ///        .get_matches_from(vec![
    ///             "prog", "--alias", "cool"
    ///         ]);
    /// assert!(m.is_present("test"));
    /// assert_eq!(m.value_of("test"), Some("cool"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    pub fn alias<S: Into<&'b str>>(mut self, name: S) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            als.push((name.into(), false));
        } else {
            self.s.aliases = Some(vec![(name.into(), false)]);
        }
        self
    }

    /// Allows adding [`Arg`] aliases, which function as "hidden" arguments that
    /// automatically dispatch as if this argument was used. This is more efficient, and easier
    /// than creating multiple hidden subcommands as one only needs to check for the existence of
    /// this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///                     .long("test")
    ///                     .aliases(&["do-stuff", "do-tests", "tests"])
    ///                     .help("the file to add")
    ///                     .required(false))
    ///             .get_matches_from(vec![
    ///                 "prog", "--do-tests"
    ///             ]);
    /// assert!(m.is_present("test"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    pub fn aliases(mut self, names: &[&'b str]) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            for n in names {
                als.push((n, false));
            }
        } else {
            self.s.aliases = Some(names.iter().map(|n| (*n, false)).collect::<Vec<_>>());
        }
        self
    }

    /// Allows adding a [`Arg`] alias that functions exactly like those defined with
    /// [`Arg::alias`], except that they are visible inside the help message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///                 .visible_alias("something-awesome")
    ///                 .long("test")
    ///                 .takes_value(true))
    ///        .get_matches_from(vec![
    ///             "prog", "--something-awesome", "coffee"
    ///         ]);
    /// assert!(m.is_present("test"));
    /// assert_eq!(m.value_of("test"), Some("coffee"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    /// [`App::alias`]: ./struct.Arg.html#method.alias
    pub fn visible_alias<S: Into<&'b str>>(mut self, name: S) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            als.push((name.into(), true));
        } else {
            self.s.aliases = Some(vec![(name.into(), true)]);
        }
        self
    }

    /// Allows adding multiple [`Arg`] aliases that functions exactly like those defined
    /// with [`Arg::aliases`], except that they are visible inside the help message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///             .arg(Arg::with_name("test")
    ///                 .long("test")
    ///                 .visible_aliases(&["something", "awesome", "cool"]))
    ///        .get_matches_from(vec![
    ///             "prog", "--awesome"
    ///         ]);
    /// assert!(m.is_present("test"));
    /// ```
    /// [`Arg`]: ./struct.Arg.html
    /// [`App::aliases`]: ./struct.Arg.html#method.aliases
    pub fn visible_aliases(mut self, names: &[&'b str]) -> Self {
        if let Some(ref mut als) = self.s.aliases {
            for n in names {
                als.push((n, true));
            }
        } else {
            self.s.aliases = Some(names.iter().map(|n| (*n, true)).collect::<Vec<_>>());
        }
        self
    }

    /// Sets the short help text of the argument that will be displayed to the user when they print
    /// the help information with `-h`. Typically, this is a short (one line) description of the
    /// arg.
    ///
    /// **NOTE:** If only `Arg::help` is provided, and not [`Arg::long_help`] but the user requests
    /// `--help` clap will still display the contents of `help` appropriately
    ///
    /// **NOTE:** Only `Arg::help` is used in completion script generation in order to be concise
    ///
    /// # Examples
    ///
    /// Any valid UTF-8 is allowed in the help text. The one exception is when one wishes to
    /// include a newline in the help text and have the following text be properly aligned with all
    /// the other help text.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .help("The config file used by the myprog")
    /// # ;
    /// ```
    ///
    /// Setting `help` displays a short message to the side of the argument when the user passes
    /// `-h` or `--help` (by default).
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    ///     --config     Some help text describing the --config arg
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    /// [`Arg::long_help`]: ./struct.Arg.html#method.long_help
    pub fn help(mut self, h: &'b str) -> Self {
        self.b.help = Some(h);
        self
    }

    /// Sets the long help text of the argument that will be displayed to the user when they print
    /// the help information with `--help`. Typically this a more detailed (multi-line) message
    /// that describes the arg.
    ///
    /// **NOTE:** If only `long_help` is provided, and not [`Arg::help`] but the user requests `-h`
    /// clap will still display the contents of `long_help` appropriately
    ///
    /// **NOTE:** Only [`Arg::help`] is used in completion script generation in order to be concise
    ///
    /// # Examples
    ///
    /// Any valid UTF-8 is allowed in the help text. The one exception is when one wishes to
    /// include a newline in the help text and have the following text be properly aligned with all
    /// the other help text.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .long_help(
    /// "The config file used by the myprog must be in JSON format
    /// with only valid keys and may not contain other nonsense
    /// that cannot be read by this program. Obviously I'm going on
    /// and on, so I'll stop now.")
    /// # ;
    /// ```
    ///
    /// Setting `help` displays a short message to the side of the argument when the user passes
    /// `-h` or `--help` (by default).
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .long_help(
    /// "The config file used by the myprog must be in JSON format
    /// with only valid keys and may not contain other nonsense
    /// that cannot be read by this program. Obviously I'm going on
    /// and on, so I'll stop now."))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    ///    --config
    ///         The config file used by the myprog must be in JSON format
    ///         with only valid keys and may not contain other nonsense
    ///         that cannot be read by this program. Obviously I'm going on
    ///         and on, so I'll stop now.
    ///
    /// -h, --help
    ///         Prints help information
    ///
    /// -V, --version
    ///         Prints version information
    /// ```
    /// [`Arg::help`]: ./struct.Arg.html#method.help
    pub fn long_help(mut self, h: &'b str) -> Self {
        self.b.long_help = Some(h);
        self
    }

    /// Specifies that this arg is the last, or final, positional argument (i.e. has the highest
    /// index) and is *only* able to be accessed via the `--` syntax (i.e. `$ prog args --
    /// last_arg`). Even, if no other arguments are left to parse, if the user omits the `--` syntax
    /// they will receive an [`UnknownArgument`] error. Setting an argument to `.last(true)` also
    /// allows one to access this arg early using the `--` syntax. Accessing an arg early, even with
    /// the `--` syntax is otherwise not possible.
    ///
    /// **NOTE:** This will change the usage string to look like `$ prog [FLAGS] [-- <ARG>]` if
    /// `ARG` is marked as `.last(true)`.
    ///
    /// **NOTE:** This setting will imply [`AppSettings::DontCollapseArgsInUsage`] because failing
    /// to set this can make the usage string very confusing.
    ///
    /// **NOTE**: This setting only applies to positional arguments, and has no affect on FLAGS /
    /// OPTIONS
    ///
    /// **CAUTION:** Setting an argument to `.last(true)` *and* having child subcommands is not
    /// recommended with the exception of *also* using [`AppSettings::ArgsNegateSubcommands`]
    /// (or [`AppSettings::SubcommandsNegateReqs`] if the argument marked `.last(true)` is also
    /// marked [`.required(true)`])
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("args")
    ///     .last(true)
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::last(true)`] ensures the arg has the highest [index] of all positional args
    /// and requires that the `--` syntax be used to access it early.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("first"))
    ///     .arg(Arg::with_name("second"))
    ///     .arg(Arg::with_name("third").last(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "one", "--", "three"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// let m = res.unwrap();
    /// assert_eq!(m.value_of("third"), Some("three"));
    /// assert!(m.value_of("second").is_none());
    /// ```
    ///
    /// Even if the positional argument marked `.last(true)` is the only argument left to parse,
    /// failing to use the `--` syntax results in an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("first"))
    ///     .arg(Arg::with_name("second"))
    ///     .arg(Arg::with_name("third").last(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "one", "two", "three"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::UnknownArgument);
    /// ```
    /// [`Arg::last(true)`]: ./struct.Arg.html#method.last
    /// [index]: ./struct.Arg.html#method.index
    /// [`AppSettings::DontCollapseArgsInUsage`]: ./enum.AppSettings.html#variant.DontCollapseArgsInUsage
    /// [`AppSettings::ArgsNegateSubcommands`]: ./enum.AppSettings.html#variant.ArgsNegateSubcommands
    /// [`AppSettings::SubcommandsNegateReqs`]: ./enum.AppSettings.html#variant.SubcommandsNegateReqs
    /// [`.required(true)`]: ./struct.Arg.html#method.required
    /// [`UnknownArgument`]: ./enum.ErrorKind.html#variant.UnknownArgument
    pub fn last(self, l: bool) -> Self {
        if l {
            self.set(ArgSettings::Last)
        } else {
            self.unset(ArgSettings::Last)
        }
    }

    /// Sets whether or not the argument is required by default. Required by default means it is
    /// required, when no other conflicting rules have been evaluated. Conflicting rules take
    /// precedence over being required. **Default:** `false`
    ///
    /// **NOTE:** Flags (i.e. not positional, or arguments that take values) cannot be required by
    /// default. This is simply because if a flag should be required, it should simply be implied
    /// as no additional information is required from user. Flags by their very nature are simply
    /// yes/no, or true/false.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required(true)
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required(true)`] requires that the argument be used at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required(true)
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required(true)`] and *not* supplying that argument is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required(true)
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::required(true)`]: ./struct.Arg.html#method.required
    pub fn required(self, r: bool) -> Self {
        if r {
            self.set(ArgSettings::Required)
        } else {
            self.unset(ArgSettings::Required)
        }
    }

    /// Requires that options use the `--option=val` syntax (i.e. an equals between the option and
    /// associated value) **Default:** `false`
    ///
    /// **NOTE:** This setting also removes the default of allowing empty values and implies
    /// [`Arg::empty_values(false)`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .long("config")
    ///     .takes_value(true)
    ///     .require_equals(true)
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::require_equals(true)`] requires that the option have an equals sign between
    /// it and the associated value.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .require_equals(true)
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config=file.conf"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::require_equals(true)`] and *not* supplying the equals will cause an error
    /// unless [`Arg::empty_values(true)`] is set.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .require_equals(true)
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::EmptyValue);
    /// ```
    /// [`Arg::require_equals(true)`]: ./struct.Arg.html#method.require_equals
    /// [`Arg::empty_values(true)`]: ./struct.Arg.html#method.empty_values
    /// [`Arg::empty_values(false)`]: ./struct.Arg.html#method.empty_values
    pub fn require_equals(mut self, r: bool) -> Self {
        if r {
            self.unsetb(ArgSettings::EmptyValues);
            self.set(ArgSettings::RequireEquals)
        } else {
            self.unset(ArgSettings::RequireEquals)
        }
    }

    /// Allows values which start with a leading hyphen (`-`)
    ///
    /// **WARNING**: Take caution when using this setting combined with [`Arg::multiple(true)`], as
    /// this becomes ambiguous `$ prog --arg -- -- val`. All three `--, --, val` will be values
    /// when the user may have thought the second `--` would constitute the normal, "Only
    /// positional args follow" idiom. To fix this, consider using [`Arg::number_of_values(1)`]
    ///
    /// **WARNING**: When building your CLIs, consider the effects of allowing leading hyphens and
    /// the user passing in a value that matches a valid short. For example `prog -opt -F` where
    /// `-F` is supposed to be a value, yet `-F` is *also* a valid short for another arg. Care should
    /// should be taken when designing these args. This is compounded by the ability to "stack"
    /// short args. I.e. if `-val` is supposed to be a value, but `-v`, `-a`, and `-l` are all valid
    /// shorts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("pattern")
    ///     .allow_hyphen_values(true)
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("pat")
    ///         .allow_hyphen_values(true)
    ///         .takes_value(true)
    ///         .long("pattern"))
    ///     .get_matches_from(vec![
    ///         "prog", "--pattern", "-file"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("pat"), Some("-file"));
    /// ```
    ///
    /// Not setting [`Arg::allow_hyphen_values(true)`] and supplying a value which starts with a
    /// hyphen is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("pat")
    ///         .takes_value(true)
    ///         .long("pattern"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--pattern", "-file"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::UnknownArgument);
    /// ```
    /// [`Arg::allow_hyphen_values(true)`]: ./struct.Arg.html#method.allow_hyphen_values
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`Arg::number_of_values(1)`]: ./struct.Arg.html#method.number_of_values
    pub fn allow_hyphen_values(self, a: bool) -> Self {
        if a {
            self.set(ArgSettings::AllowLeadingHyphen)
        } else {
            self.unset(ArgSettings::AllowLeadingHyphen)
        }
    }
    /// Sets an arg that override this arg's required setting. (i.e. this arg will be required
    /// unless this other argument is present).
    ///
    /// **Pro Tip:** Using [`Arg::required_unless`] implies [`Arg::required`] and is therefore not
    /// mandatory to also set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_unless("debug")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_unless(name)`] requires that the argument be used at runtime
    /// *unless* `name` is present. In the following example, the required argument is *not*
    /// provided, but it's not an error because the `unless` arg has been supplied.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless("dbg")
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required_unless(name)`] and *not* supplying `name` or this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless("dbg")
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::required_unless`]: ./struct.Arg.html#method.required_unless
    /// [`Arg::required`]: ./struct.Arg.html#method.required
    /// [`Arg::required_unless(name)`]: ./struct.Arg.html#method.required_unless
    pub fn required_unless(mut self, name: &'a str) -> Self {
        if let Some(ref mut vec) = self.b.r_unless {
            vec.push(name);
        } else {
            self.b.r_unless = Some(vec![name]);
        }
        self.required(true)
    }

    /// Sets args that override this arg's required setting. (i.e. this arg will be required unless
    /// all these other arguments are present).
    ///
    /// **NOTE:** If you wish for this argument to only be required if *one of* these args are
    /// present see [`Arg::required_unless_one`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_unless_all(&["cfg", "dbg"])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_unless_all(names)`] requires that the argument be used at runtime
    /// *unless* *all* the args in `names` are present. In the following example, the required
    /// argument is *not* provided, but it's not an error because all the `unless` args have been
    /// supplied.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_all(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug", "-i", "file"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required_unless_all(names)`] and *not* supplying *all* of `names` or this
    /// arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_all(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::required_unless_one`]: ./struct.Arg.html#method.required_unless_one
    /// [`Arg::required_unless_all(names)`]: ./struct.Arg.html#method.required_unless_all
    pub fn required_unless_all(mut self, names: &[&'a str]) -> Self {
        if let Some(ref mut vec) = self.b.r_unless {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.r_unless = Some(names.iter().copied().collect());
        }
        self.setb(ArgSettings::RequiredUnlessAll);
        self.required(true)
    }

    /// Sets args that override this arg's [required] setting. (i.e. this arg will be required
    /// unless *at least one of* these other arguments are present).
    ///
    /// **NOTE:** If you wish for this argument to only be required if *all of* these args are
    /// present see [`Arg::required_unless_all`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_unless_all(&["cfg", "dbg"])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_unless_one(names)`] requires that the argument be used at runtime
    /// *unless* *at least one of* the args in `names` are present. In the following example, the
    /// required argument is *not* provided, but it's not an error because one the `unless` args
    /// have been supplied.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_one(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// ```
    ///
    /// Setting [`Arg::required_unless_one(names)`] and *not* supplying *at least one of* `names`
    /// or this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_unless_one(&["dbg", "infile"])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("dbg")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("infile")
    ///         .short("i")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [required]: ./struct.Arg.html#method.required
    /// [`Arg::required_unless_one(names)`]: ./struct.Arg.html#method.required_unless_one
    /// [`Arg::required_unless_all`]: ./struct.Arg.html#method.required_unless_all
    pub fn required_unless_one(mut self, names: &[&'a str]) -> Self {
        if let Some(ref mut vec) = self.b.r_unless {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.r_unless = Some(names.iter().copied().collect());
        }
        self.required(true)
    }

    /// Sets a conflicting argument by name. I.e. when using this argument,
    /// the following argument can't be present and vice versa.
    ///
    /// **NOTE:** Conflicting rules take precedence over being required by default. Conflict rules
    /// only need to be set for one of the two arguments, they do not need to be set for each.
    ///
    /// **NOTE:** Defining a conflict is two-way, but does *not* need to defined for both arguments
    /// (i.e. if A conflicts with B, defining A.conflicts_with(B) is sufficient. You do not need
    /// need to also do B.conflicts_with(A))
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .conflicts_with("debug")
    /// # ;
    /// ```
    ///
    /// Setting conflicting argument, and having both arguments present at runtime is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .conflicts_with("debug")
    ///         .long("config"))
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--debug", "--config", "file.conf"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::ArgumentConflict);
    /// ```
    pub fn conflicts_with(mut self, name: &'a str) -> Self {
        if let Some(ref mut vec) = self.b.blacklist {
            vec.push(name);
        } else {
            self.b.blacklist = Some(vec![name]);
        }
        self
    }

    /// The same as [`Arg::conflicts_with`] but allows specifying multiple two-way conlicts per
    /// argument.
    ///
    /// **NOTE:** Conflicting rules take precedence over being required by default. Conflict rules
    /// only need to be set for one of the two arguments, they do not need to be set for each.
    ///
    /// **NOTE:** Defining a conflict is two-way, but does *not* need to defined for both arguments
    /// (i.e. if A conflicts with B, defining A.conflicts_with(B) is sufficient. You do not need
    /// need to also do B.conflicts_with(A))
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .conflicts_with_all(&["debug", "input"])
    /// # ;
    /// ```
    ///
    /// Setting conflicting argument, and having any of the arguments present at runtime with a
    /// conflicting argument is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .conflicts_with_all(&["debug", "input"])
    ///         .long("config"))
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf", "file.txt"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::ArgumentConflict);
    /// ```
    /// [`Arg::conflicts_with`]: ./struct.Arg.html#method.conflicts_with
    pub fn conflicts_with_all(mut self, names: &[&'a str]) -> Self {
        if let Some(ref mut vec) = self.b.blacklist {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.blacklist = Some(names.iter().copied().collect());
        }
        self
    }

    /// Sets a overridable argument by name. I.e. this argument and the following argument
    /// will override each other in POSIX style (whichever argument was specified at runtime
    /// **last** "wins")
    ///
    /// **NOTE:** When an argument is overridden it is essentially as if it never was used, any
    /// conflicts, requirements, etc. are evaluated **after** all "overrides" have been removed
    ///
    /// **WARNING:** Positional arguments cannot override themselves (or we would never be able
    /// to advance to the next positional). If a positional agument lists itself as an override,
    /// it is simply ignored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::from_usage("-f, --flag 'some flag'")
    ///         .conflicts_with("debug"))
    ///     .arg(Arg::from_usage("-d, --debug 'other flag'"))
    ///     .arg(Arg::from_usage("-c, --color 'third flag'")
    ///         .overrides_with("flag"))
    ///     .get_matches_from(vec![
    ///         "prog", "-f", "-d", "-c"]);
    ///             //    ^~~~~~~~~~~~^~~~~ flag is overridden by color
    ///
    /// assert!(m.is_present("color"));
    /// assert!(m.is_present("debug")); // even though flag conflicts with debug, it's as if flag
    ///                                 // was never used because it was overridden with color
    /// assert!(!m.is_present("flag"));
    /// ```
    /// Care must be taken when using this setting, and having an arg override with itself. This
    /// is common practice when supporting things like shell aliases, config files, etc.
    /// However, when combined with multiple values, it can get dicy.
    /// Here is how clap handles such situations:
    ///
    /// When a flag overrides itself, it's as if the flag was only ever used once (essentially
    /// preventing a "Unexpected multiple usage" error):
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("posix")
    ///             .arg(Arg::from_usage("--flag  'some flag'").overrides_with("flag"))
    ///             .get_matches_from(vec!["posix", "--flag", "--flag"]);
    /// assert!(m.is_present("flag"));
    /// assert_eq!(m.occurrences_of("flag"), 1);
    /// ```
    /// Making a arg `multiple(true)` and override itself is essentially meaningless. Therefore
    /// clap ignores an override of self if it's a flag and it already accepts multiple occurrences.
    ///
    /// ```
    /// # use clap::{App, Arg};
    /// let m = App::new("posix")
    ///             .arg(Arg::from_usage("--flag...  'some flag'").overrides_with("flag"))
    ///             .get_matches_from(vec!["", "--flag", "--flag", "--flag", "--flag"]);
    /// assert!(m.is_present("flag"));
    /// assert_eq!(m.occurrences_of("flag"), 4);
    /// ```
    /// Now notice with options (which *do not* set `multiple(true)`), it's as if only the last
    /// occurrence happened.
    ///
    /// ```
    /// # use clap::{App, Arg};
    /// let m = App::new("posix")
    ///             .arg(Arg::from_usage("--opt [val] 'some option'").overrides_with("opt"))
    ///             .get_matches_from(vec!["", "--opt=some", "--opt=other"]);
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 1);
    /// assert_eq!(m.value_of("opt"), Some("other"));
    /// ```
    ///
    /// Just like flags, options with `multiple(true)` set, will ignore the "override self" setting.
    ///
    /// ```
    /// # use clap::{App, Arg};
    /// let m = App::new("posix")
    ///             .arg(Arg::from_usage("--opt [val]... 'some option'")
    ///                 .overrides_with("opt"))
    ///             .get_matches_from(vec!["", "--opt", "first", "over", "--opt", "other", "val"]);
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 2);
    /// assert_eq!(m.values_of("opt").unwrap().collect::<Vec<_>>(), &["first", "over", "other", "val"]);
    /// ```
    ///
    /// A safe thing to do if you'd like to support an option which supports multiple values, but
    /// also is "overridable" by itself, is to use `use_delimiter(false)` and *not* use
    /// `multiple(true)` while telling users to seperate values with a comma (i.e. `val1,val2`)
    ///
    /// ```
    /// # use clap::{App, Arg};
    /// let m = App::new("posix")
    ///             .arg(Arg::from_usage("--opt [val] 'some option'")
    ///                 .overrides_with("opt")
    ///                 .use_delimiter(false))
    ///             .get_matches_from(vec!["", "--opt=some,other", "--opt=one,two"]);
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 1);
    /// assert_eq!(m.values_of("opt").unwrap().collect::<Vec<_>>(), &["one,two"]);
    /// ```
    pub fn overrides_with(mut self, name: &'a str) -> Self {
        if let Some(ref mut vec) = self.b.overrides {
            vec.push(name);
        } else {
            self.b.overrides = Some(vec![name]);
        }
        self
    }

    /// Sets multiple mutually overridable arguments by name. I.e. this argument and the following
    /// argument will override each other in POSIX style (whichever argument was specified at
    /// runtime **last** "wins")
    ///
    /// **NOTE:** When an argument is overridden it is essentially as if it never was used, any
    /// conflicts, requirements, etc. are evaluated **after** all "overrides" have been removed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::from_usage("-f, --flag 'some flag'")
    ///         .conflicts_with("color"))
    ///     .arg(Arg::from_usage("-d, --debug 'other flag'"))
    ///     .arg(Arg::from_usage("-c, --color 'third flag'")
    ///         .overrides_with_all(&["flag", "debug"]))
    ///     .get_matches_from(vec![
    ///         "prog", "-f", "-d", "-c"]);
    ///             //    ^~~~~~^~~~~~~~~ flag and debug are overridden by color
    ///
    /// assert!(m.is_present("color")); // even though flag conflicts with color, it's as if flag
    ///                                 // and debug were never used because they were overridden
    ///                                 // with color
    /// assert!(!m.is_present("debug"));
    /// assert!(!m.is_present("flag"));
    /// ```
    pub fn overrides_with_all(mut self, names: &[&'a str]) -> Self {
        if let Some(ref mut vec) = self.b.overrides {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.overrides = Some(names.iter().copied().collect());
        }
        self
    }

    /// Sets an argument by name that is required when this one is present I.e. when
    /// using this argument, the following argument *must* be present.
    ///
    /// **NOTE:** [Conflicting] rules and [override] rules take precedence over being required
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires("input")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires(name)`] requires that the argument be used at runtime if the
    /// defining argument is used. If the defining argument isn't used, the other argument isn't
    /// required
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires("input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use cfg, so input wasn't required
    /// ```
    ///
    /// Setting [`Arg::requires(name)`] and *not* supplying that argument is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires("input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    pub fn requires(mut self, name: &'a str) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            vec.push((None, name));
        } else {
            self.b.requires = Some(vec![(None, name)]);
        }
        self
    }

    /// Allows a conditional requirement. The requirement will only become valid if this arg's value
    /// equals `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// requires_if:
    ///     - [val, arg]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires_if("val", "arg")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires_if(val, arg)`] requires that the `arg` be used at runtime if the
    /// defining argument's value is equal to `val`. If the defining argument is anything other than
    /// `val`, the other argument isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_if("my.cfg", "other")
    ///         .long("config"))
    ///     .arg(Arg::with_name("other"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "some.cfg"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use --config=my.cfg, so other wasn't required
    /// ```
    ///
    /// Setting [`Arg::requires_if(val, arg)`] and setting the value to `val` but *not* supplying
    /// `arg` is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_if("my.cfg", "input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "my.cfg"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    pub fn requires_if(mut self, val: &'b str, arg: &'a str) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            vec.push((Some(val), arg));
        } else {
            self.b.requires = Some(vec![(Some(val), arg)]);
        }
        self
    }

    /// Allows multiple conditional requirements. The requirement will only become valid if this arg's value
    /// equals `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// requires_if:
    ///     - [val, arg]
    ///     - [val2, arg2]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires_ifs(&[
    ///         ("val", "arg"),
    ///         ("other_val", "arg2"),
    ///     ])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires_ifs(&["val", "arg"])`] requires that the `arg` be used at runtime if the
    /// defining argument's value is equal to `val`. If the defining argument's value is anything other
    /// than `val`, `arg` isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_ifs(&[
    ///             ("special.conf", "opt"),
    ///             ("other.conf", "other"),
    ///         ])
    ///         .long("config"))
    ///     .arg(Arg::with_name("opt")
    ///         .long("option")
    ///         .takes_value(true))
    ///     .arg(Arg::with_name("other"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "special.conf"
    ///     ]);
    ///
    /// assert!(res.is_err()); // We  used --config=special.conf so --option <val> is required
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    pub fn requires_ifs(mut self, ifs: &[(&'b str, &'a str)]) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            for &(val, arg) in ifs {
                vec.push((Some(val), arg));
            }
        } else {
            let mut vec = vec![];
            for &(val, arg) in ifs {
                vec.push((Some(val), arg));
            }
            self.b.requires = Some(vec);
        }
        self
    }

    /// Allows specifying that an argument is [required] conditionally. The requirement will only
    /// become valid if the specified `arg`'s value equals `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// required_if:
    ///     - [arg, val]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_if("other_arg", "value")
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_if(arg, val)`] makes this arg required if the `arg` is used at
    /// runtime and it's value is equal to `val`. If the `arg`'s value is anything other than `val`,
    /// this argument isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .required_if("other", "special")
    ///         .long("config"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--other", "not-special"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use --other=special, so "cfg" wasn't required
    /// ```
    ///
    /// Setting [`Arg::required_if(arg, val)`] and having `arg` used with a value of `val` but *not*
    /// using this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .required_if("other", "special")
    ///         .long("config"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .takes_value(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--other", "special"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [required]: ./struct.Arg.html#method.required
    pub fn required_if(mut self, arg: &'a str, val: &'b str) -> Self {
        if let Some(ref mut vec) = self.r_ifs {
            vec.push((arg, val));
        } else {
            self.r_ifs = Some(vec![(arg, val)]);
        }
        self
    }

    /// Allows specifying that an argument is [required] based on multiple conditions. The
    /// conditions are set up in a `(arg, val)` style tuple. The requirement will only become valid
    /// if one of the specified `arg`'s value equals it's corresponding `val`.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// required_if:
    ///     - [arg, val]
    ///     - [arg2, val2]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .required_ifs(&[
    ///         ("extra", "val"),
    ///         ("option", "spec")
    ///     ])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::required_ifs(&[(arg, val)])`] makes this arg required if any of the `arg`s
    /// are used at runtime and it's corresponding value is equal to `val`. If the `arg`'s value is
    /// anything other than `val`, this argument isn't required.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_ifs(&[
    ///             ("extra", "val"),
    ///             ("option", "spec")
    ///         ])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("extra")
    ///         .takes_value(true)
    ///         .long("extra"))
    ///     .arg(Arg::with_name("option")
    ///         .takes_value(true)
    ///         .long("option"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--option", "other"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use --option=spec, or --extra=val so "cfg" isn't required
    /// ```
    ///
    /// Setting [`Arg::required_ifs(&[(arg, val)])`] and having any of the `arg`s used with it's
    /// value of `val` but *not* using this arg is an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .required_ifs(&[
    ///             ("extra", "val"),
    ///             ("option", "spec")
    ///         ])
    ///         .takes_value(true)
    ///         .long("config"))
    ///     .arg(Arg::with_name("extra")
    ///         .takes_value(true)
    ///         .long("extra"))
    ///     .arg(Arg::with_name("option")
    ///         .takes_value(true)
    ///         .long("option"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--option", "spec"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [`Arg::requires(name)`]: ./struct.Arg.html#method.requires
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [required]: ./struct.Arg.html#method.required
    pub fn required_ifs(mut self, ifs: &[(&'a str, &'b str)]) -> Self {
        if let Some(ref mut vec) = self.r_ifs {
            for r_if in ifs {
                vec.push((r_if.0, r_if.1));
            }
        } else {
            let mut vec = vec![];
            for r_if in ifs {
                vec.push((r_if.0, r_if.1));
            }
            self.r_ifs = Some(vec);
        }
        self
    }

    /// Sets multiple arguments by names that are required when this one is present I.e. when
    /// using this argument, the following arguments *must* be present.
    ///
    /// **NOTE:** [Conflicting] rules and [override] rules take precedence over being required
    /// by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Arg;
    /// Arg::with_name("config")
    ///     .requires_all(&["input", "output"])
    /// # ;
    /// ```
    ///
    /// Setting [`Arg::requires_all(&[arg, arg2])`] requires that all the arguments be used at
    /// runtime if the defining argument is used. If the defining argument isn't used, the other
    /// argument isn't required
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires("input")
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .arg(Arg::with_name("output")
    ///         .index(2))
    ///     .get_matches_from_safe(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert!(res.is_ok()); // We didn't use cfg, so input and output weren't required
    /// ```
    ///
    /// Setting [`Arg::requires_all(&[arg, arg2])`] and *not* supplying all the arguments is an
    /// error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .takes_value(true)
    ///         .requires_all(&["input", "output"])
    ///         .long("config"))
    ///     .arg(Arg::with_name("input")
    ///         .index(1))
    ///     .arg(Arg::with_name("output")
    ///         .index(2))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config", "file.conf", "in.txt"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// // We didn't use output
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    /// ```
    /// [Conflicting]: ./struct.Arg.html#method.conflicts_with
    /// [override]: ./struct.Arg.html#method.overrides_with
    /// [`Arg::requires_all(&[arg, arg2])`]: ./struct.Arg.html#method.requires_all
    pub fn requires_all(mut self, names: &[&'a str]) -> Self {
        if let Some(ref mut vec) = self.b.requires {
            for s in names {
                vec.push((None, s));
            }
        } else {
            let mut vec = vec![];
            for s in names {
                vec.push((None, *s));
            }
            self.b.requires = Some(vec);
        }
        self
    }

    /// Specifies that the argument takes a value at run time.
    ///
    /// **NOTE:** values for arguments may be specified in any of the following methods
    ///
    /// * Using a space such as `-o value` or `--option value`
    /// * Using an equals and no space such as `-o=value` or `--option=value`
    /// * Use a short and no space such as `-ovalue`
    ///
    /// **NOTE:** By default, args which allow [multiple values] are delimited by commas, meaning
    /// `--option=val1,val2,val3` is three values for the `--option` argument. If you wish to
    /// change the delimiter to another character you can use [`Arg::value_delimiter(char)`],
    /// alternatively you can turn delimiting values **OFF** by using [`Arg::use_delimiter(false)`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .takes_value(true)
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true))
    ///     .get_matches_from(vec![
    ///         "prog", "--mode", "fast"
    ///     ]);
    ///
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast"));
    /// ```
    /// [`Arg::value_delimiter(char)`]: ./struct.Arg.html#method.value_delimiter
    /// [`Arg::use_delimiter(false)`]: ./struct.Arg.html#method.use_delimiter
    /// [multiple values]: ./struct.Arg.html#method.multiple
    pub fn takes_value(self, tv: bool) -> Self {
        if tv {
            self.set(ArgSettings::TakesValue)
        } else {
            self.unset(ArgSettings::TakesValue)
        }
    }

    /// Specifies if the possible values of an argument should be displayed in the help text or
    /// not. Defaults to `false` (i.e. show possible values)
    ///
    /// This is useful for args with many values, or ones which are explained elsewhere in the
    /// help text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .hide_possible_values(true)
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .possible_values(&["fast", "slow"])
    ///         .takes_value(true)
    ///         .hide_possible_values(true));
    ///
    /// ```
    ///
    /// If we were to run the above program with `--help` the `[values: fast, slow]` portion of
    /// the help text would be omitted.
    pub fn hide_possible_values(self, hide: bool) -> Self {
        if hide {
            self.set(ArgSettings::HidePossibleValues)
        } else {
            self.unset(ArgSettings::HidePossibleValues)
        }
    }

    /// Specifies if the default value of an argument should be displayed in the help text or
    /// not. Defaults to `false` (i.e. show default value)
    ///
    /// This is useful when default behavior of an arg is explained elsewhere in the help text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .hide_default_value(true)
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("connect")
    ///     .arg(Arg::with_name("host")
    ///         .long("host")
    ///         .default_value("localhost")
    ///         .hide_default_value(true));
    ///
    /// ```
    ///
    /// If we were to run the above program with `--help` the `[default: localhost]` portion of
    /// the help text would be omitted.
    pub fn hide_default_value(self, hide: bool) -> Self {
        if hide {
            self.set(ArgSettings::HideDefaultValue)
        } else {
            self.unset(ArgSettings::HideDefaultValue)
        }
    }

    /// Specifies the index of a positional argument **starting at** 1.
    ///
    /// **NOTE:** The index refers to position according to **other positional argument**. It does
    /// not define position in the argument list as a whole.
    ///
    /// **NOTE:** If no [`Arg::short`], or [`Arg::long`] have been defined, you can optionally
    /// leave off the `index` method, and the index will be assigned in order of evaluation.
    /// Utilizing the `index` method allows for setting indexes out of order
    ///
    /// **NOTE:** When utilized with [`Arg::multiple(true)`], only the **last** positional argument
    /// may be defined as multiple (i.e. with the highest index)
    ///
    /// # Panics
    ///
    /// Although not in this method directly, [`App`] will [`panic!`] if indexes are skipped (such
    /// as defining `index(1)` and `index(3)` but not `index(2)`, or a positional argument is
    /// defined as multiple and is not the highest index
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("config")
    ///     .index(1)
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .index(1))
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug"))
    ///     .get_matches_from(vec![
    ///         "prog", "--debug", "fast"
    ///     ]);
    ///
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast")); // notice index(1) means "first positional"
    ///                                               // *not* first argument
    /// ```
    /// [`Arg::short`]: ./struct.Arg.html#method.short
    /// [`Arg::long`]: ./struct.Arg.html#method.long
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`App`]: ./struct.App.html
    /// [`panic!`]: https://doc.rust-lang.org/std/macro.panic!.html
    pub fn index(mut self, idx: u64) -> Self {
        self.index = Some(idx);
        self
    }

    /// Specifies that the argument may appear more than once. For flags, this results
    /// in the number of occurrences of the flag being recorded. For example `-ddd` or `-d -d -d`
    /// would count as three occurrences. For options there is a distinct difference in multiple
    /// occurrences vs multiple values.
    ///
    /// For example, `--opt val1 val2` is one occurrence, but two values. Whereas
    /// `--opt val1 --opt val2` is two occurrences.
    ///
    /// **WARNING:**
    ///
    /// Setting `multiple(true)` for an [option] with no other details, allows multiple values
    /// **and** multiple occurrences because it isn't possible to have more occurrences than values
    /// for options. Because multiple values are allowed, `--option val1 val2 val3` is perfectly
    /// valid, be careful when designing a CLI where positional arguments are expected after a
    /// option which accepts multiple values, as `clap` will continue parsing *values* until it
    /// reaches the max or specific number of values defined, or another flag or option.
    ///
    /// **Pro Tip**:
    ///
    /// It's possible to define an option which allows multiple occurrences, but only one value per
    /// occurrence. To do this use [`Arg::number_of_values(1)`] in coordination with
    /// [`Arg::multiple(true)`].
    ///
    /// **WARNING:**
    ///
    /// When using args with `multiple(true)` on [options] or [positionals] (i.e. those args that
    /// accept values) and [subcommands], one needs to consider the possibility of an argument value
    /// being the same as a valid subcommand. By default `clap` will parse the argument in question
    /// as a value *only if* a value is possible at that moment. Otherwise it will be parsed as a
    /// subcommand. In effect, this means using `multiple(true)` with no additional parameters and
    /// a possible value that coincides with a subcommand name, the subcommand cannot be called
    /// unless another argument is passed first.
    ///
    /// As an example, consider a CLI with an option `--ui-paths=<paths>...` and subcommand `signer`
    ///
    /// The following would be parsed as values to `--ui-paths`.
    ///
    /// ```notrust
    /// $ program --ui-paths path1 path2 signer
    /// ```
    ///
    /// This is because `--ui-paths` accepts multiple values. `clap` will continue parsing values
    /// until another argument is reached and it knows `--ui-paths` is done.
    ///
    /// By adding additional parameters to `--ui-paths` we can solve this issue. Consider adding
    /// [`Arg::number_of_values(1)`] as discussed above. The following are all valid, and `signer`
    /// is parsed as both a subcommand and a value in the second case.
    ///
    /// ```notrust
    /// $ program --ui-paths path1 signer
    /// $ program --ui-paths path1 --ui-paths signer signer
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .short("d")
    ///     .multiple(true)
    /// # ;
    /// ```
    /// An example with flags
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("verbose")
    ///         .multiple(true)
    ///         .short("v"))
    ///     .get_matches_from(vec![
    ///         "prog", "-v", "-v", "-v"    // note, -vvv would have same result
    ///     ]);
    ///
    /// assert!(m.is_present("verbose"));
    /// assert_eq!(m.occurrences_of("verbose"), 3);
    /// ```
    ///
    /// An example with options
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .multiple(true)
    ///         .takes_value(true)
    ///         .short("F"))
    ///     .get_matches_from(vec![
    ///         "prog", "-F", "file1", "file2", "file3"
    ///     ]);
    ///
    /// assert!(m.is_present("file"));
    /// assert_eq!(m.occurrences_of("file"), 1); // notice only one occurrence
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3"]);
    /// ```
    /// This is functionally equivalent to the example above
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .multiple(true)
    ///         .takes_value(true)
    ///         .short("F"))
    ///     .get_matches_from(vec![
    ///         "prog", "-F", "file1", "-F", "file2", "-F", "file3"
    ///     ]);
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3"]);
    ///
    /// assert!(m.is_present("file"));
    /// assert_eq!(m.occurrences_of("file"), 3); // Notice 3 occurrences
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3"]);
    /// ```
    ///
    /// A common mistake is to define an option which allows multiples, and a positional argument
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .multiple(true)
    ///         .takes_value(true)
    ///         .short("F"))
    ///     .arg(Arg::with_name("word")
    ///         .index(1))
    ///     .get_matches_from(vec![
    ///         "prog", "-F", "file1", "file2", "file3", "word"
    ///     ]);
    ///
    /// assert!(m.is_present("file"));
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3", "word"]); // wait...what?!
    /// assert!(!m.is_present("word")); // but we clearly used word!
    /// ```
    /// The problem is clap doesn't know when to stop parsing values for "files". This is further
    /// compounded by if we'd said `word -F file1 file2` it would have worked fine, so it would
    /// appear to only fail sometimes...not good!
    ///
    /// A solution for the example above is to specify that `-F` only accepts one value, but is
    /// allowed to appear multiple times
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .multiple(true)
    ///         .takes_value(true)
    ///         .number_of_values(1)
    ///         .short("F"))
    ///     .arg(Arg::with_name("word")
    ///         .index(1))
    ///     .get_matches_from(vec![
    ///         "prog", "-F", "file1", "-F", "file2", "-F", "file3", "word"
    ///     ]);
    ///
    /// assert!(m.is_present("file"));
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3"]);
    /// assert!(m.is_present("word"));
    /// assert_eq!(m.value_of("word"), Some("word"));
    /// ```
    /// As a final example, notice if we define [`Arg::number_of_values(1)`] and try to run the
    /// problem example above, it would have been a runtime error with a pretty message to the
    /// user :)
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .multiple(true)
    ///         .takes_value(true)
    ///         .number_of_values(1)
    ///         .short("F"))
    ///     .arg(Arg::with_name("word")
    ///         .index(1))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2", "file3", "word"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::UnknownArgument);
    /// ```
    /// [option]: ./struct.Arg.html#method.takes_value
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [subcommands]: ./struct.SubCommand.html
    /// [positionals]: ./struct.Arg.html#method.index
    /// [`Arg::number_of_values(1)`]: ./struct.Arg.html#method.number_of_values
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn multiple(self, multi: bool) -> Self {
        if multi {
            self.set(ArgSettings::Multiple)
        } else {
            self.unset(ArgSettings::Multiple)
        }
    }

    /// Specifies a value that *stops* parsing multiple values of a give argument. By default when
    /// one sets [`multiple(true)`] on an argument, clap will continue parsing values for that
    /// argument until it reaches another valid argument, or one of the other more specific settings
    /// for multiple values is used (such as [`min_values`], [`max_values`] or
    /// [`number_of_values`]).
    ///
    /// **NOTE:** This setting only applies to [options] and [positional arguments]
    ///
    /// **NOTE:** When the terminator is passed in on the command line, it is **not** stored as one
    /// of the values
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("vals")
    ///     .takes_value(true)
    ///     .multiple(true)
    ///     .value_terminator(";")
    /// # ;
    /// ```
    /// The following example uses two arguments, a sequence of commands, and the location in which
    /// to perform them
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cmds")
    ///         .multiple(true)
    ///         .allow_hyphen_values(true)
    ///         .value_terminator(";"))
    ///     .arg(Arg::with_name("location"))
    ///     .get_matches_from(vec![
    ///         "prog", "find", "-type", "f", "-name", "special", ";", "/home/clap"
    ///     ]);
    /// let cmds: Vec<_> = m.values_of("cmds").unwrap().collect();
    /// assert_eq!(&cmds, &["find", "-type", "f", "-name", "special"]);
    /// assert_eq!(m.value_of("location"), Some("/home/clap"));
    /// ```
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [positional arguments]: ./struct.Arg.html#method.index
    /// [`multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`min_values`]: ./struct.Arg.html#method.min_values
    /// [`number_of_values`]: ./struct.Arg.html#method.number_of_values
    /// [`max_values`]: ./struct.Arg.html#method.max_values
    pub fn value_terminator(mut self, term: &'b str) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.terminator = Some(term);
        self
    }

    /// Specifies that an argument can be matched to all child [`SubCommand`]s.
    ///
    /// **NOTE:** Global arguments *only* propagate down, **not** up (to parent commands), however
    /// their values once a user uses them will be propagated back up to parents. In effect, this
    /// means one should *define* all global arguments at the top level, however it doesn't matter
    /// where the user *uses* the global argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .short("d")
    ///     .global(true)
    /// # ;
    /// ```
    ///
    /// For example, assume an application with two subcommands, and you'd like to define a
    /// `--verbose` flag that can be called on any of the subcommands and parent, but you don't
    /// want to clutter the source with three duplicate [`Arg`] definitions.
    ///
    /// ```rust
    /// # use clap::{App, Arg, SubCommand};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("verb")
    ///         .long("verbose")
    ///         .short("v")
    ///         .global(true))
    ///     .subcommand(SubCommand::with_name("test"))
    ///     .subcommand(SubCommand::with_name("do-stuff"))
    ///     .get_matches_from(vec![
    ///         "prog", "do-stuff", "--verbose"
    ///     ]);
    ///
    /// assert_eq!(m.subcommand_name(), Some("do-stuff"));
    /// let sub_m = m.subcommand_matches("do-stuff").unwrap();
    /// assert!(sub_m.is_present("verb"));
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [required]: ./struct.Arg.html#method.required
    /// [`ArgMatches`]: ./struct.ArgMatches.html
    /// [`ArgMatches::is_present("flag")`]: ./struct.ArgMatches.html#method.is_present
    /// [`Arg`]: ./struct.Arg.html
    pub fn global(self, g: bool) -> Self {
        if g {
            self.set(ArgSettings::Global)
        } else {
            self.unset(ArgSettings::Global)
        }
    }

    /// Allows an argument to accept explicitly empty values. An empty value must be specified at
    /// the command line with an explicit `""`, or `''`
    ///
    /// **NOTE:** Defaults to `true` (Explicitly empty values are allowed)
    ///
    /// **NOTE:** Implicitly sets [`Arg::takes_value(true)`] when set to `false`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .long("file")
    ///     .empty_values(false)
    /// # ;
    /// ```
    /// The default is to allow empty values, such as `--option ""` would be an empty value. But
    /// we can change to make empty values become an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .short("v")
    ///         .empty_values(false))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--config="
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::EmptyValue);
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    pub fn empty_values(mut self, ev: bool) -> Self {
        if ev {
            self.set(ArgSettings::EmptyValues)
        } else {
            self = self.set(ArgSettings::TakesValue);
            self.unset(ArgSettings::EmptyValues)
        }
    }

    /// Hides an argument from help message output.
    ///
    /// **NOTE:** Implicitly sets [`Arg::hidden_short_help(true)`] and [`Arg::hidden_long_help(true)`]
    /// when set to true
    ///
    /// **NOTE:** This does **not** hide the argument from usage strings on error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .hidden(true)
    /// # ;
    /// ```
    /// Setting `hidden(true)` will hide the argument when displaying help text
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .hidden(true)
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    /// [`Arg::hidden_short_help(true)`]: ./struct.Arg.html#method.hidden_short_help
    /// [`Arg::hidden_long_help(true)`]: ./struct.Arg.html#method.hidden_long_help
    pub fn hidden(self, h: bool) -> Self {
        if h {
            self.set(ArgSettings::Hidden)
        } else {
            self.unset(ArgSettings::Hidden)
        }
    }

    /// Specifies a list of possible values for this argument. At runtime, `clap` verifies that
    /// only one of the specified values was used, or fails with an error message.
    ///
    /// **NOTE:** This setting only applies to [options] and [positional arguments]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("mode")
    ///     .takes_value(true)
    ///     .possible_values(&["fast", "slow", "medium"])
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_values(&["fast", "slow", "medium"]))
    ///     .get_matches_from(vec![
    ///         "prog", "--mode", "fast"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast"));
    /// ```
    ///
    /// The next example shows a failed parse from using a value which wasn't defined as one of the
    /// possible values.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_values(&["fast", "slow", "medium"]))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--mode", "wrong"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::InvalidValue);
    /// ```
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [positional arguments]: ./struct.Arg.html#method.index
    pub fn possible_values(mut self, names: &[&'b str]) -> Self {
        if let Some(ref mut vec) = self.v.possible_vals {
            for s in names {
                vec.push(s);
            }
        } else {
            self.v.possible_vals = Some(names.iter().copied().collect());
        }
        self
    }

    /// Specifies a possible value for this argument, one at a time. At runtime, `clap` verifies
    /// that only one of the specified values was used, or fails with error message.
    ///
    /// **NOTE:** This setting only applies to [options] and [positional arguments]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("mode")
    ///     .takes_value(true)
    ///     .possible_value("fast")
    ///     .possible_value("slow")
    ///     .possible_value("medium")
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_value("fast")
    ///         .possible_value("slow")
    ///         .possible_value("medium"))
    ///     .get_matches_from(vec![
    ///         "prog", "--mode", "fast"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// assert_eq!(m.value_of("mode"), Some("fast"));
    /// ```
    ///
    /// The next example shows a failed parse from using a value which wasn't defined as one of the
    /// possible values.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("mode")
    ///         .long("mode")
    ///         .takes_value(true)
    ///         .possible_value("fast")
    ///         .possible_value("slow")
    ///         .possible_value("medium"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "--mode", "wrong"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::InvalidValue);
    /// ```
    /// [options]: ./struct.Arg.html#method.takes_value
    /// [positional arguments]: ./struct.Arg.html#method.index
    pub fn possible_value(mut self, name: &'b str) -> Self {
        if let Some(ref mut vec) = self.v.possible_vals {
            vec.push(name);
        } else {
            self.v.possible_vals = Some(vec![name]);
        }
        self
    }

    /// When used with [`Arg::possible_values`] it allows the argument value to pass validation even if
    /// the case differs from that of the specified `possible_value`.
    ///
    /// **Pro Tip:** Use this setting with [`arg_enum!`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// # use std::ascii::AsciiExt;
    /// let m = App::new("pv")
    ///     .arg(Arg::with_name("option")
    ///         .long("--option")
    ///         .takes_value(true)
    ///         .possible_value("test123")
    ///         .case_insensitive(true))
    ///     .get_matches_from(vec![
    ///         "pv", "--option", "TeSt123",
    ///     ]);
    ///
    /// assert!(m.value_of("option").unwrap().eq_ignore_ascii_case("test123"));
    /// ```
    ///
    /// This setting also works when multiple values can be defined:
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("pv")
    ///     .arg(Arg::with_name("option")
    ///         .short("-o")
    ///         .long("--option")
    ///         .takes_value(true)
    ///         .possible_value("test123")
    ///         .possible_value("test321")
    ///         .multiple(true)
    ///         .case_insensitive(true))
    ///     .get_matches_from(vec![
    ///         "pv", "--option", "TeSt123", "teST123", "tESt321"
    ///     ]);
    ///
    /// let matched_vals = m.values_of("option").unwrap().collect::<Vec<_>>();
    /// assert_eq!(&*matched_vals, &["TeSt123", "teST123", "tESt321"]);
    /// ```
    /// [`Arg::case_insensitive(true)`]: ./struct.Arg.html#method.possible_values
    /// [`arg_enum!`]: ./macro.arg_enum.html
    pub fn case_insensitive(self, ci: bool) -> Self {
        if ci {
            self.set(ArgSettings::CaseInsensitive)
        } else {
            self.unset(ArgSettings::CaseInsensitive)
        }
    }

    /// Specifies the name of the [`ArgGroup`] the argument belongs to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .long("debug")
    ///     .group("mode")
    /// # ;
    /// ```
    ///
    /// Multiple arguments can be a member of a single group and then the group checked as if it
    /// was one of said arguments.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug")
    ///         .group("mode"))
    ///     .arg(Arg::with_name("verbose")
    ///         .long("verbose")
    ///         .group("mode"))
    ///     .get_matches_from(vec![
    ///         "prog", "--debug"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// ```
    /// [`ArgGroup`]: ./struct.ArgGroup.html
    pub fn group(mut self, name: &'a str) -> Self {
        if let Some(ref mut vec) = self.b.groups {
            vec.push(name);
        } else {
            self.b.groups = Some(vec![name]);
        }
        self
    }

    /// Specifies the names of multiple [`ArgGroup`]'s the argument belongs to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .long("debug")
    ///     .groups(&["mode", "verbosity"])
    /// # ;
    /// ```
    ///
    /// Arguments can be members of multiple groups and then the group checked as if it
    /// was one of said arguments.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("debug")
    ///         .long("debug")
    ///         .groups(&["mode", "verbosity"]))
    ///     .arg(Arg::with_name("verbose")
    ///         .long("verbose")
    ///         .groups(&["mode", "verbosity"]))
    ///     .get_matches_from(vec![
    ///         "prog", "--debug"
    ///     ]);
    /// assert!(m.is_present("mode"));
    /// assert!(m.is_present("verbosity"));
    /// ```
    /// [`ArgGroup`]: ./struct.ArgGroup.html
    pub fn groups(mut self, names: &[&'a str]) -> Self {
        if let Some(ref mut vec) = self.b.groups {
            for s in names {
                vec.push(s);
            }
        } else {
            self.b.groups = Some(names.iter().copied().collect());
        }
        self
    }

    /// Specifies how many values are required to satisfy this argument. For example, if you had a
    /// `-f <file>` argument where you wanted exactly 3 'files' you would set
    /// `.number_of_values(3)`, and this argument wouldn't be satisfied unless the user provided
    /// 3 and only 3 values.
    ///
    /// **NOTE:** Does *not* require [`Arg::multiple(true)`] to be set. Setting
    /// [`Arg::multiple(true)`] would allow `-f <file> <file> <file> -f <file> <file> <file>` where
    /// as *not* setting [`Arg::multiple(true)`] would only allow one occurrence of this argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .short("f")
    ///     .number_of_values(3)
    /// # ;
    /// ```
    ///
    /// Not supplying the correct number of values is an error
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .number_of_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::WrongNumberOfValues);
    /// ```
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn number_of_values(mut self, qty: u64) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.num_vals = Some(qty);
        self
    }

    /// Allows one to perform a custom validation on the argument value. You provide a closure
    /// which accepts a [`String`] value, and return a [`Result`] where the [`Err(String)`] is a
    /// message displayed to the user.
    ///
    /// **NOTE:** The error message does *not* need to contain the `error:` portion, only the
    /// message as all errors will appear as
    /// `error: Invalid value for '<arg>': <YOUR MESSAGE>` where `<arg>` is replaced by the actual
    /// arg, and `<YOUR MESSAGE>` is the `String` you return as the error.
    ///
    /// **NOTE:** There is a small performance hit for using validators, as they are implemented
    /// with [`Rc`] pointers. And the value to be checked will be allocated an extra time in order
    /// to to be passed to the closure. This performance hit is extremely minimal in the grand
    /// scheme of things.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// fn has_at(v: String) -> Result<(), String> {
    ///     if v.contains("@") { return Ok(()); }
    ///     Err(String::from("The value did not contain the required @ sigil"))
    /// }
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .index(1)
    ///         .validator(has_at))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "some@file"
    ///     ]);
    /// assert!(res.is_ok());
    /// assert_eq!(res.unwrap().value_of("file"), Some("some@file"));
    /// ```
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`Err(String)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
    pub fn validator<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Result<(), String> + 'static,
    {
        self.v.validator = Some(Rc::new(f));
        self
    }

    /// Works identically to Validator but is intended to be used with values that could
    /// contain non UTF-8 formatted strings.
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```rust")]
    /// # use clap::{App, Arg};
    /// # use std::ffi::{OsStr, OsString};
    /// # use std::os::unix::ffi::OsStrExt;
    /// fn has_ampersand(v: &OsStr) -> Result<(), OsString> {
    ///     if v.as_bytes().iter().any(|b| *b == b'&') { return Ok(()); }
    ///     Err(OsString::from("The value did not contain the required & sigil"))
    /// }
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .index(1)
    ///         .validator_os(has_ampersand))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "Fish & chips"
    ///     ]);
    /// assert!(res.is_ok());
    /// assert_eq!(res.unwrap().value_of("file"), Some("Fish & chips"));
    /// ```
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    /// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`Err(String)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    /// [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
    pub fn validator_os<F>(mut self, f: F) -> Self
    where
        F: Fn(&OsStr) -> Result<(), OsString> + 'static,
    {
        self.v.validator_os = Some(Rc::new(f));
        self
    }

    /// Specifies the *maximum* number of values are for this argument. For example, if you had a
    /// `-f <file>` argument where you wanted up to 3 'files' you would set `.max_values(3)`, and
    /// this argument would be satisfied if the user provided, 1, 2, or 3 values.
    ///
    /// **NOTE:** This does *not* implicitly set [`Arg::multiple(true)`]. This is because
    /// `-o val -o val` is multiple occurrences but a single value and `-o val1 val2` is a single
    /// occurrence with multiple values. For positional arguments this **does** set
    /// [`Arg::multiple(true)`] because there is no way to determine the difference between multiple
    /// occurrences and multiple values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .short("f")
    ///     .max_values(3)
    /// # ;
    /// ```
    ///
    /// Supplying less than the maximum number of values is allowed
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .max_values(3)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// let m = res.unwrap();
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2"]);
    /// ```
    ///
    /// Supplying more than the maximum number of values is an error
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .max_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2", "file3"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::TooManyValues);
    /// ```
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn max_values(mut self, qty: u64) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.max_vals = Some(qty);
        self
    }

    /// Specifies the *minimum* number of values for this argument. For example, if you had a
    /// `-f <file>` argument where you wanted at least 2 'files' you would set
    /// `.min_values(2)`, and this argument would be satisfied if the user provided, 2 or more
    /// values.
    ///
    /// **NOTE:** This does not implicitly set [`Arg::multiple(true)`]. This is because
    /// `-o val -o val` is multiple occurrences but a single value and `-o val1 val2` is a single
    /// occurrence with multiple values. For positional arguments this **does** set
    /// [`Arg::multiple(true)`] because there is no way to determine the difference between multiple
    /// occurrences and multiple values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("file")
    ///     .short("f")
    ///     .min_values(3)
    /// # ;
    /// ```
    ///
    /// Supplying more than the minimum number of values is allowed
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .min_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1", "file2", "file3"
    ///     ]);
    ///
    /// assert!(res.is_ok());
    /// let m = res.unwrap();
    /// let files: Vec<_> = m.values_of("file").unwrap().collect();
    /// assert_eq!(files, ["file1", "file2", "file3"]);
    /// ```
    ///
    /// Supplying less than the minimum number of values is an error
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("file")
    ///         .takes_value(true)
    ///         .min_values(2)
    ///         .short("F"))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-F", "file1"
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind, ErrorKind::TooFewValues);
    /// ```
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn min_values(mut self, qty: u64) -> Self {
        self.v.min_vals = Some(qty);
        self.set(ArgSettings::TakesValue)
    }

    /// Specifies whether or not an argument should allow grouping of multiple values via a
    /// delimiter. I.e. should `--option=val1,val2,val3` be parsed as three values (`val1`, `val2`,
    /// and `val3`) or as a single value (`val1,val2,val3`). Defaults to using `,` (comma) as the
    /// value delimiter for all arguments that accept values (options and positional arguments)
    ///
    /// **NOTE:** The default is `false`. When set to `true` the default [`Arg::value_delimiter`]
    /// is the comma `,`.
    ///
    /// # Examples
    ///
    /// The following example shows the default behavior.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let delims = App::new("prog")
    ///     .arg(Arg::with_name("option")
    ///         .long("option")
    ///         .use_delimiter(true)
    ///         .takes_value(true))
    ///     .get_matches_from(vec![
    ///         "prog", "--option=val1,val2,val3",
    ///     ]);
    ///
    /// assert!(delims.is_present("option"));
    /// assert_eq!(delims.occurrences_of("option"), 1);
    /// assert_eq!(delims.values_of("option").unwrap().collect::<Vec<_>>(), ["val1", "val2", "val3"]);
    /// ```
    /// The next example shows the difference when turning delimiters off. This is the default
    /// behavior
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let nodelims = App::new("prog")
    ///     .arg(Arg::with_name("option")
    ///         .long("option")
    ///         .use_delimiter(false)
    ///         .takes_value(true))
    ///     .get_matches_from(vec![
    ///         "prog", "--option=val1,val2,val3",
    ///     ]);
    ///
    /// assert!(nodelims.is_present("option"));
    /// assert_eq!(nodelims.occurrences_of("option"), 1);
    /// assert_eq!(nodelims.value_of("option").unwrap(), "val1,val2,val3");
    /// ```
    /// [`Arg::value_delimiter`]: ./struct.Arg.html#method.value_delimiter
    pub fn use_delimiter(mut self, d: bool) -> Self {
        if d {
            if self.v.val_delim.is_none() {
                self.v.val_delim = Some(',');
            }
            self.setb(ArgSettings::TakesValue);
            self.setb(ArgSettings::UseValueDelimiter);
        } else {
            self.v.val_delim = None;
            self.unsetb(ArgSettings::UseValueDelimiter);
        }
        self.unset(ArgSettings::ValueDelimiterNotSet)
    }

    /// Specifies that *multiple values* may only be set using the delimiter. This means if an
    /// if an option is encountered, and no delimiter is found, it automatically assumed that no
    /// additional values for that option follow. This is unlike the default, where it is generally
    /// assumed that more values will follow regardless of whether or not a delimiter is used.
    ///
    /// **NOTE:** The default is `false`.
    ///
    /// **NOTE:** Setting this to true implies [`Arg::use_delimiter(true)`]
    ///
    /// **NOTE:** It's a good idea to inform the user that use of a delimiter is required, either
    /// through help text or other means.
    ///
    /// # Examples
    ///
    /// These examples demonstrate what happens when `require_delimiter(true)` is used. Notice
    /// everything works in this first example, as we use a delimiter, as expected.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let delims = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true)
    ///         .require_delimiter(true))
    ///     .get_matches_from(vec![
    ///         "prog", "-o", "val1,val2,val3",
    ///     ]);
    ///
    /// assert!(delims.is_present("opt"));
    /// assert_eq!(delims.values_of("opt").unwrap().collect::<Vec<_>>(), ["val1", "val2", "val3"]);
    /// ```
    /// In this next example, we will *not* use a delimiter. Notice it's now an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg, ErrorKind};
    /// let res = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true)
    ///         .require_delimiter(true))
    ///     .get_matches_from_safe(vec![
    ///         "prog", "-o", "val1", "val2", "val3",
    ///     ]);
    ///
    /// assert!(res.is_err());
    /// let err = res.unwrap_err();
    /// assert_eq!(err.kind, ErrorKind::UnknownArgument);
    /// ```
    /// What's happening is `-o` is getting `val1`, and because delimiters are required yet none
    /// were present, it stops parsing `-o`. At this point it reaches `val2` and because no
    /// positional arguments have been defined, it's an error of an unexpected argument.
    ///
    /// In this final example, we contrast the above with `clap`'s default behavior where the above
    /// is *not* an error.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let delims = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .multiple(true))
    ///     .get_matches_from(vec![
    ///         "prog", "-o", "val1", "val2", "val3",
    ///     ]);
    ///
    /// assert!(delims.is_present("opt"));
    /// assert_eq!(delims.values_of("opt").unwrap().collect::<Vec<_>>(), ["val1", "val2", "val3"]);
    /// ```
    /// [`Arg::use_delimiter(true)`]: ./struct.Arg.html#method.use_delimiter
    pub fn require_delimiter(mut self, d: bool) -> Self {
        if d {
            self = self.use_delimiter(true);
            self.unsetb(ArgSettings::ValueDelimiterNotSet);
            self.setb(ArgSettings::UseValueDelimiter);
            self.set(ArgSettings::RequireDelimiter)
        } else {
            self = self.use_delimiter(false);
            self.unsetb(ArgSettings::UseValueDelimiter);
            self.unset(ArgSettings::RequireDelimiter)
        }
    }

    /// Specifies the separator to use when values are clumped together, defaults to `,` (comma).
    ///
    /// **NOTE:** implicitly sets [`Arg::use_delimiter(true)`]
    ///
    /// **NOTE:** implicitly sets [`Arg::takes_value(true)`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("config")
    ///         .short("c")
    ///         .long("config")
    ///         .value_delimiter(";"))
    ///     .get_matches_from(vec![
    ///         "prog", "--config=val1;val2;val3"
    ///     ]);
    ///
    /// assert_eq!(m.values_of("config").unwrap().collect::<Vec<_>>(), ["val1", "val2", "val3"])
    /// ```
    /// [`Arg::use_delimiter(true)`]: ./struct.Arg.html#method.use_delimiter
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    pub fn value_delimiter(mut self, d: &str) -> Self {
        self.unsetb(ArgSettings::ValueDelimiterNotSet);
        self.setb(ArgSettings::TakesValue);
        self.setb(ArgSettings::UseValueDelimiter);
        self.v.val_delim = Some(
            d.chars()
                .next()
                .expect("Failed to get value_delimiter from arg"),
        );
        self
    }

    /// Specify multiple names for values of option arguments. These names are cosmetic only, used
    /// for help and usage strings only. The names are **not** used to access arguments. The values
    /// of the arguments are accessed in numeric order (i.e. if you specify two names `one` and
    /// `two` `one` will be the first matched value, `two` will be the second).
    ///
    /// This setting can be very helpful when describing the type of input the user should be
    /// using, such as `FILE`, `INTERFACE`, etc. Although not required, it's somewhat convention to
    /// use all capital letters for the value name.
    ///
    /// **Pro Tip:** It may help to use [`Arg::next_line_help(true)`] if there are long, or
    /// multiple value names in order to not throw off the help text alignment of all options.
    ///
    /// **NOTE:** This implicitly sets [`Arg::number_of_values`] if the number of value names is
    /// greater than one. I.e. be aware that the number of "names" you set for the values, will be
    /// the *exact* number of values required to satisfy this argument
    ///
    /// **NOTE:** implicitly sets [`Arg::takes_value(true)`]
    ///
    /// **NOTE:** Does *not* require or imply [`Arg::multiple(true)`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("speed")
    ///     .short("s")
    ///     .value_names(&["fast", "slow"])
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("io")
    ///         .long("io-files")
    ///         .value_names(&["INFILE", "OUTFILE"]))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    /// Running the above program produces the following output
    ///
    /// ```notrust
    /// valnames
    ///
    /// USAGE:
    ///    valnames [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     --io-files <INFILE> <OUTFILE>    Some help text
    /// ```
    /// [`Arg::next_line_help(true)`]: ./struct.Arg.html#method.next_line_help
    /// [`Arg::number_of_values`]: ./struct.Arg.html#method.number_of_values
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    pub fn value_names(mut self, names: &[&'b str]) -> Self {
        self.setb(ArgSettings::TakesValue);
        if self.is_set(ArgSettings::ValueDelimiterNotSet) {
            self.unsetb(ArgSettings::ValueDelimiterNotSet);
            self.setb(ArgSettings::UseValueDelimiter);
        }
        if let Some(ref mut vals) = self.v.val_names {
            let mut l = vals.len();
            for s in names {
                vals.insert(l, s);
                l += 1;
            }
        } else {
            let mut vm = VecMap::new();
            for (i, n) in names.iter().enumerate() {
                vm.insert(i, *n);
            }
            self.v.val_names = Some(vm);
        }
        self
    }

    /// Specifies the name for value of [option] or [positional] arguments inside of help
    /// documentation. This name is cosmetic only, the name is **not** used to access arguments.
    /// This setting can be very helpful when describing the type of input the user should be
    /// using, such as `FILE`, `INTERFACE`, etc. Although not required, it's somewhat convention to
    /// use all capital letters for the value name.
    ///
    /// **NOTE:** implicitly sets [`Arg::takes_value(true)`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("cfg")
    ///     .long("config")
    ///     .value_name("FILE")
    /// # ;
    /// ```
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("config")
    ///         .long("config")
    ///         .value_name("FILE"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    /// Running the above program produces the following output
    ///
    /// ```notrust
    /// valnames
    ///
    /// USAGE:
    ///    valnames [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     --config <FILE>     Some help text
    /// ```
    /// [option]: ./struct.Arg.html#method.takes_value
    /// [positional]: ./struct.Arg.html#method.index
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    pub fn value_name(mut self, name: &'b str) -> Self {
        self.setb(ArgSettings::TakesValue);
        if let Some(ref mut vals) = self.v.val_names {
            let l = vals.len();
            vals.insert(l, name);
        } else {
            let mut vm = VecMap::new();
            vm.insert(0, name);
            self.v.val_names = Some(vm);
        }
        self
    }

    /// Specifies the value of the argument when *not* specified at runtime.
    ///
    /// **NOTE:** If the user *does not* use this argument at runtime, [`ArgMatches::occurrences_of`]
    /// will return `0` even though the [`ArgMatches::value_of`] will return the default specified.
    ///
    /// **NOTE:** If the user *does not* use this argument at runtime [`ArgMatches::is_present`] will
    /// still return `true`. If you wish to determine whether the argument was used at runtime or
    /// not, consider [`ArgMatches::occurrences_of`] which will return `0` if the argument was *not*
    /// used at runtime.
    ///
    /// **NOTE:** This setting is perfectly compatible with [`Arg::default_value_if`] but slightly
    /// different. `Arg::default_value` *only* takes affect when the user has not provided this arg
    /// at runtime. `Arg::default_value_if` however only takes affect when the user has not provided
    /// a value at runtime **and** these other conditions are met as well. If you have set
    /// `Arg::default_value` and `Arg::default_value_if`, and the user **did not** provide a this
    /// arg at runtime, nor did were the conditions met for `Arg::default_value_if`, the
    /// `Arg::default_value` will be applied.
    ///
    /// **NOTE:** This implicitly sets [`Arg::takes_value(true)`].
    ///
    /// **NOTE:** This setting effectively disables `AppSettings::ArgRequiredElseHelp` if used in
    /// conjunction as it ensures that some argument will always be present.
    ///
    /// # Examples
    ///
    /// First we use the default value without providing any value at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .long("myopt")
    ///         .default_value("myval"))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("opt"), Some("myval"));
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 0);
    /// ```
    ///
    /// Next we provide a value at runtime to override the default.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .long("myopt")
    ///         .default_value("myval"))
    ///     .get_matches_from(vec![
    ///         "prog", "--myopt=non_default"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("opt"), Some("non_default"));
    /// assert!(m.is_present("opt"));
    /// assert_eq!(m.occurrences_of("opt"), 1);
    /// ```
    /// [`ArgMatches::occurrences_of`]: ./struct.ArgMatches.html#method.occurrences_of
    /// [`ArgMatches::value_of`]: ./struct.ArgMatches.html#method.value_of
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`ArgMatches::is_present`]: ./struct.ArgMatches.html#method.is_present
    /// [`Arg::default_value_if`]: ./struct.Arg.html#method.default_value_if
    pub fn default_value(self, val: &'a str) -> Self {
        self.default_value_os(OsStr::from_bytes(val.as_bytes()))
    }

    /// Provides a default value in the exact same manner as [`Arg::default_value`]
    /// only using [`OsStr`]s instead.
    /// [`Arg::default_value`]: ./struct.Arg.html#method.default_value
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    pub fn default_value_os(mut self, val: &'a OsStr) -> Self {
        self.setb(ArgSettings::TakesValue);
        self.v.default_val = Some(val);
        self
    }

    /// Specifies the value of the argument if `arg` has been used at runtime. If `val` is set to
    /// `None`, `arg` only needs to be present. If `val` is set to `"some-val"` then `arg` must be
    /// present at runtime **and** have the value `val`.
    ///
    /// **NOTE:** This setting is perfectly compatible with [`Arg::default_value`] but slightly
    /// different. `Arg::default_value` *only* takes affect when the user has not provided this arg
    /// at runtime. This setting however only takes affect when the user has not provided a value at
    /// runtime **and** these other conditions are met as well. If you have set `Arg::default_value`
    /// and `Arg::default_value_if`, and the user **did not** provide a this arg at runtime, nor did
    /// were the conditions met for `Arg::default_value_if`, the `Arg::default_value` will be
    /// applied.
    ///
    /// **NOTE:** This implicitly sets [`Arg::takes_value(true)`].
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows (`None` can be represented
    /// as `null` in YAML)
    ///
    /// ```yaml
    /// default_value_if:
    ///     - [arg, val, default]
    /// ```
    ///
    /// # Examples
    ///
    /// First we use the default value only if another arg is present at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("flag", None, "default"))
    ///     .get_matches_from(vec![
    ///         "prog", "--flag"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("default"));
    /// ```
    ///
    /// Next we run the same test, but without providing `--flag`.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("flag", None, "default"))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), None);
    /// ```
    ///
    /// Now lets only use the default value if `--opt` contains the value `special`.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .takes_value(true)
    ///         .long("opt"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("opt", Some("special"), "default"))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "special"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("default"));
    /// ```
    ///
    /// We can run the same test and provide any value *other than* `special` and we won't get a
    /// default value.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .takes_value(true)
    ///         .long("opt"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_if("opt", Some("special"), "default"))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "hahaha"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), None);
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::default_value`]: ./struct.Arg.html#method.default_value
    pub fn default_value_if(self, arg: &'a str, val: Option<&'b str>, default: &'b str) -> Self {
        self.default_value_if_os(
            arg,
            val.map(str::as_bytes).map(OsStr::from_bytes),
            OsStr::from_bytes(default.as_bytes()),
        )
    }

    /// Provides a conditional default value in the exact same manner as [`Arg::default_value_if`]
    /// only using [`OsStr`]s instead.
    /// [`Arg::default_value_if`]: ./struct.Arg.html#method.default_value_if
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    pub fn default_value_if_os(
        mut self,
        arg: &'a str,
        val: Option<&'b OsStr>,
        default: &'b OsStr,
    ) -> Self {
        self.setb(ArgSettings::TakesValue);
        if let Some(ref mut vm) = self.v.default_vals_ifs {
            let l = vm.len();
            vm.insert(l, (arg, val, default));
        } else {
            let mut vm = VecMap::new();
            vm.insert(0, (arg, val, default));
            self.v.default_vals_ifs = Some(vm);
        }
        self
    }

    /// Specifies multiple values and conditions in the same manner as [`Arg::default_value_if`].
    /// The method takes a slice of tuples in the `(arg, Option<val>, default)` format.
    ///
    /// **NOTE**: The conditions are stored in order and evaluated in the same order. I.e. the first
    /// if multiple conditions are true, the first one found will be applied and the ultimate value.
    ///
    /// **NOTE:** If using YAML the values should be laid out as follows
    ///
    /// ```yaml
    /// default_value_if:
    ///     - [arg, val, default]
    ///     - [arg2, null, default2]
    /// ```
    ///
    /// # Examples
    ///
    /// First we use the default value only if another arg is present at runtime.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("opt")
    ///         .long("opt")
    ///         .takes_value(true))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_ifs(&[
    ///             ("flag", None, "default"),
    ///             ("opt", Some("channal"), "chan"),
    ///         ]))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "channal"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("chan"));
    /// ```
    ///
    /// Next we run the same test, but without providing `--flag`.
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_ifs(&[
    ///             ("flag", None, "default"),
    ///             ("opt", Some("channal"), "chan"),
    ///         ]))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), None);
    /// ```
    ///
    /// We can also see that these values are applied in order, and if more than one condition is
    /// true, only the first evaluated "wins"
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag"))
    ///     .arg(Arg::with_name("opt")
    ///         .long("opt")
    ///         .takes_value(true))
    ///     .arg(Arg::with_name("other")
    ///         .long("other")
    ///         .default_value_ifs(&[
    ///             ("flag", None, "default"),
    ///             ("opt", Some("channal"), "chan"),
    ///         ]))
    ///     .get_matches_from(vec![
    ///         "prog", "--opt", "channal", "--flag"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("other"), Some("default"));
    /// ```
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::default_value`]: ./struct.Arg.html#method.default_value
    pub fn default_value_ifs(mut self, ifs: &[(&'a str, Option<&'b str>, &'b str)]) -> Self {
        for &(arg, val, default) in ifs {
            self = self.default_value_if_os(
                arg,
                val.map(str::as_bytes).map(OsStr::from_bytes),
                OsStr::from_bytes(default.as_bytes()),
            );
        }
        self
    }

    /// Provides multiple conditional default values in the exact same manner as
    /// [`Arg::default_value_ifs`] only using [`OsStr`]s instead.
    /// [`Arg::default_value_ifs`]: ./struct.Arg.html#method.default_value_ifs
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    pub fn default_value_ifs_os(mut self, ifs: &[(&'a str, Option<&'b OsStr>, &'b OsStr)]) -> Self {
        for &(arg, val, default) in ifs {
            self = self.default_value_if_os(arg, val, default);
        }
        self
    }

    /// Specifies that if the value is not passed in as an argument, that it should be retrieved
    /// from the environment, if available. If it is not present in the environment, then default
    /// rules will apply.
    ///
    /// **NOTE:** If the user *does not* use this argument at runtime, [`ArgMatches::occurrences_of`]
    /// will return `0` even though the [`ArgMatches::value_of`] will return the default specified.
    ///
    /// **NOTE:** If the user *does not* use this argument at runtime [`ArgMatches::is_present`] will
    /// return `true` if the variable is present in the environment . If you wish to determine whether
    /// the argument was used at runtime or not, consider [`ArgMatches::occurrences_of`] which will
    /// return `0` if the argument was *not* used at runtime.
    ///
    /// **NOTE:** This implicitly sets [`Arg::takes_value(true)`].
    ///
    /// **NOTE:** If [`Arg::multiple(true)`] is set then [`Arg::use_delimiter(true)`] should also be
    /// set. Otherwise, only a single argument will be returned from the environment variable. The
    /// default delimiter is `,` and follows all the other delimiter rules.
    ///
    /// # Examples
    ///
    /// In this example, we show the variable coming from the environment:
    ///
    /// ```rust
    /// # use std::env;
    /// # use clap::{App, Arg};
    ///
    /// env::set_var("MY_FLAG", "env");
    ///
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag")
    ///         .env("MY_FLAG"))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("flag"), Some("env"));
    /// ```
    ///
    /// In this example, we show the variable coming from an option on the CLI:
    ///
    /// ```rust
    /// # use std::env;
    /// # use clap::{App, Arg};
    ///
    /// env::set_var("MY_FLAG", "env");
    ///
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag")
    ///         .env("MY_FLAG"))
    ///     .get_matches_from(vec![
    ///         "prog", "--flag", "opt"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("flag"), Some("opt"));
    /// ```
    ///
    /// In this example, we show the variable coming from the environment even with the
    /// presence of a default:
    ///
    /// ```rust
    /// # use std::env;
    /// # use clap::{App, Arg};
    ///
    /// env::set_var("MY_FLAG", "env");
    ///
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag")
    ///         .env("MY_FLAG")
    ///         .default_value("default"))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("flag"), Some("env"));
    /// ```
    ///
    /// In this example, we show the use of multiple values in a single environment variable:
    ///
    /// ```rust
    /// # use std::env;
    /// # use clap::{App, Arg};
    ///
    /// env::set_var("MY_FLAG_MULTI", "env1,env2");
    ///
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("flag")
    ///         .long("flag")
    ///         .env("MY_FLAG_MULTI")
    ///         .multiple(true)
    ///         .use_delimiter(true))
    ///     .get_matches_from(vec![
    ///         "prog"
    ///     ]);
    ///
    /// assert_eq!(m.values_of("flag").unwrap().collect::<Vec<_>>(), vec!["env1", "env2"]);
    /// ```
    /// [`ArgMatches::occurrences_of`]: ./struct.ArgMatches.html#method.occurrences_of
    /// [`ArgMatches::value_of`]: ./struct.ArgMatches.html#method.value_of
    /// [`ArgMatches::is_present`]: ./struct.ArgMatches.html#method.is_present
    /// [`Arg::takes_value(true)`]: ./struct.Arg.html#method.takes_value
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`Arg::use_delimiter(true)`]: ./struct.Arg.html#method.use_delimiter
    pub fn env(self, name: &'a str) -> Self {
        self.env_os(OsStr::new(name))
    }

    /// Specifies that if the value is not passed in as an argument, that it should be retrieved
    /// from the environment if available in the exact same manner as [`Arg::env`] only using
    /// [`OsStr`]s instead.
    pub fn env_os(mut self, name: &'a OsStr) -> Self {
        self.setb(ArgSettings::TakesValue);

        self.v.env = Some((name, env::var_os(name)));
        self
    }

    /// @TODO @p2 @docs @release: write docs
    pub fn hide_env_values(self, hide: bool) -> Self {
        if hide {
            self.set(ArgSettings::HideEnvValues)
        } else {
            self.unset(ArgSettings::HideEnvValues)
        }
    }

    /// When set to `true` the help string will be displayed on the line after the argument and
    /// indented once. This can be helpful for arguments with very long or complex help messages.
    /// This can also be helpful for arguments with very long flag names, or many/long value names.
    ///
    /// **NOTE:** To apply this setting to all arguments consider using
    /// [`AppSettings::NextLineHelp`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("opt")
    ///         .long("long-option-flag")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .value_names(&["value1", "value2"])
    ///         .help("Some really long help and complex\n\
    ///                help that makes more sense to be\n\
    ///                on a line after the option")
    ///         .next_line_help(true))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays the following help message
    ///
    /// ```notrust
    /// nlh
    ///
    /// USAGE:
    ///     nlh [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     -o, --long-option-flag <value1> <value2>
    ///         Some really long help and complex
    ///         help that makes more sense to be
    ///         on a line after the option
    /// ```
    /// [`AppSettings::NextLineHelp`]: ./enum.AppSettings.html#variant.NextLineHelp
    pub fn next_line_help(mut self, nlh: bool) -> Self {
        if nlh {
            self.setb(ArgSettings::NextLineHelp);
        } else {
            self.unsetb(ArgSettings::NextLineHelp);
        }
        self
    }

    /// Allows custom ordering of args within the help message. Args with a lower value will be
    /// displayed first in the help message. This is helpful when one would like to emphasise
    /// frequently used args, or prioritize those towards the top of the list. Duplicate values
    /// **are** allowed. Args with duplicate display orders will be displayed in alphabetical
    /// order.
    ///
    /// **NOTE:** The default is 999 for all arguments.
    ///
    /// **NOTE:** This setting is ignored for [positional arguments] which are always displayed in
    /// [index] order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("a") // Typically args are grouped alphabetically by name.
    ///                              // Args without a display_order have a value of 999 and are
    ///                              // displayed alphabetically with all other 999 valued args.
    ///         .long("long-option")
    ///         .short("o")
    ///         .takes_value(true)
    ///         .help("Some help and text"))
    ///     .arg(Arg::with_name("b")
    ///         .long("other-option")
    ///         .short("O")
    ///         .takes_value(true)
    ///         .display_order(1)   // In order to force this arg to appear *first*
    ///                             // all we have to do is give it a value lower than 999.
    ///                             // Any other args with a value of 1 will be displayed
    ///                             // alphabetically with this one...then 2 values, then 3, etc.
    ///         .help("I should be first!"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays the following help message
    ///
    /// ```notrust
    /// cust-ord
    ///
    /// USAGE:
    ///     cust-ord [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// OPTIONS:
    ///     -O, --other-option <b>    I should be first!
    ///     -o, --long-option <a>     Some help and text
    /// ```
    /// [positional arguments]: ./struct.Arg.html#method.index
    /// [index]: ./struct.Arg.html#method.index
    pub fn display_order(mut self, ord: usize) -> Self {
        self.s.disp_ord = ord;
        self
    }

    /// Indicates that all parameters passed after this should not be parsed
    /// individually, but rather passed in their entirety. It is worth noting
    /// that setting this requires all values to come after a `--` to indicate they
    /// should all be captured. For example:
    ///
    /// ```notrust
    /// --foo something -- -v -v -v -b -b -b --baz -q -u -x
    /// ```
    /// Will result in everything after `--` to be considered one raw argument. This behavior
    /// may not be exactly what you are expecting and using [`AppSettings::TrailingVarArg`]
    /// may be more appropriate.
    ///
    /// **NOTE:** Implicitly sets [`Arg::multiple(true)`], [`Arg::allow_hyphen_values(true)`], and
    /// [`Arg::last(true)`] when set to `true`
    ///
    /// [`Arg::multiple(true)`]: ./struct.Arg.html#method.multiple
    /// [`Arg::allow_hyphen_values(true)`]: ./struct.Arg.html#method.allow_hyphen_values
    /// [`Arg::last(true)`]: ./struct.Arg.html#method.last
    /// [`AppSettings::TrailingVarArg`]: ./enum.AppSettings.html#variant.TrailingVarArg
    pub fn raw(self, raw: bool) -> Self {
        self.multiple(raw).allow_hyphen_values(raw).last(raw)
    }

    /// Hides an argument from short help message output.
    ///
    /// **NOTE:** This does **not** hide the argument from usage strings on error
    ///
    /// **NOTE:** Setting this option will cause next-line-help output style to be used
    /// when long help (`--help`) is called.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .hidden_short_help(true)
    /// # ;
    /// ```
    /// Setting `hidden_short_help(true)` will hide the argument when displaying short help text
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .hidden_short_help(true)
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "-h"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    ///
    /// However, when --help is called
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .hidden_short_help(true)
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// Then the following would be displayed
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    ///     --config     Some help text describing the --config arg
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    pub fn hidden_short_help(self, hide: bool) -> Self {
        if hide {
            self.set(ArgSettings::HiddenShortHelp)
        } else {
            self.unset(ArgSettings::HiddenShortHelp)
        }
    }

    /// Hides an argument from long help message output.
    ///
    /// **NOTE:** This does **not** hide the argument from usage strings on error
    ///
    /// **NOTE:** Setting this option will cause next-line-help output style to be used
    /// when long help (`--help`) is called.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// Arg::with_name("debug")
    ///     .hidden_long_help(true)
    /// # ;
    /// ```
    /// Setting `hidden_long_help(true)` will hide the argument when displaying long help text
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .hidden_long_help(true)
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    ///
    /// However, when -h is called
    ///
    /// ```rust
    /// # use clap::{App, Arg};
    /// let m = App::new("prog")
    ///     .arg(Arg::with_name("cfg")
    ///         .long("config")
    ///         .hidden_long_help(true)
    ///         .help("Some help text describing the --config arg"))
    ///     .get_matches_from(vec![
    ///         "prog", "-h"
    ///     ]);
    /// ```
    ///
    /// Then the following would be displayed
    ///
    /// ```notrust
    /// helptest
    ///
    /// USAGE:
    ///    helptest [FLAGS]
    ///
    /// FLAGS:
    ///     --config     Some help text describing the --config arg
    /// -h, --help       Prints help information
    /// -V, --version    Prints version information
    /// ```
    pub fn hidden_long_help(self, hide: bool) -> Self {
        if hide {
            self.set(ArgSettings::HiddenLongHelp)
        } else {
            self.unset(ArgSettings::HiddenLongHelp)
        }
    }

    /// Checks if one of the [`ArgSettings`] settings is set for the argument.
    ///
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn is_set(&self, s: ArgSettings) -> bool {
        self.b.is_set(s)
    }

    /// Sets one of the [`ArgSettings`] settings for the argument.
    ///
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn set(mut self, s: ArgSettings) -> Self {
        self.setb(s);
        self
    }

    /// Unsets one of the [`ArgSettings`] settings for the argument.
    ///
    /// [`ArgSettings`]: ./enum.ArgSettings.html
    pub fn unset(mut self, s: ArgSettings) -> Self {
        self.unsetb(s);
        self
    }

    #[doc(hidden)]
    pub fn setb(&mut self, s: ArgSettings) {
        self.b.set(s);
    }

    #[doc(hidden)]
    pub fn unsetb(&mut self, s: ArgSettings) {
        self.b.unset(s);
    }
}

impl<'a, 'b, 'z> From<&'z Arg<'a, 'b>> for Arg<'a, 'b> {
    fn from(a: &'z Arg<'a, 'b>) -> Self {
        Arg {
            b: a.b.clone(),
            v: a.v.clone(),
            s: a.s.clone(),
            index: a.index,
            r_ifs: a.r_ifs.clone(),
        }
    }
}

impl<'n, 'e> PartialEq for Arg<'n, 'e> {
    fn eq(&self, other: &Arg<'n, 'e>) -> bool {
        self.b == other.b
    }
}
