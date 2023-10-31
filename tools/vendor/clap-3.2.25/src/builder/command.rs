#![allow(deprecated)]

// Std
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::ops::Index;
use std::path::Path;

// Third Party
#[cfg(feature = "yaml")]
use yaml_rust::Yaml;

// Internal
use crate::builder::app_settings::{AppFlags, AppSettings};
use crate::builder::arg_settings::ArgSettings;
use crate::builder::{arg::ArgProvider, Arg, ArgGroup, ArgPredicate};
use crate::error::ErrorKind;
use crate::error::Result as ClapResult;
use crate::mkeymap::MKeyMap;
use crate::output::fmt::Stream;
use crate::output::{fmt::Colorizer, Help, HelpWriter, Usage};
use crate::parser::{ArgMatcher, ArgMatches, Parser};
use crate::util::ChildGraph;
use crate::util::{color::ColorChoice, Id, Key};
use crate::PossibleValue;
use crate::{Error, INTERNAL_ERROR_MSG};

#[cfg(debug_assertions)]
use crate::builder::debug_asserts::assert_app;

/// Build a command-line interface.
///
/// This includes defining arguments, subcommands, parser behavior, and help output.
/// Once all configuration is complete,
/// the [`Command::get_matches`] family of methods starts the runtime-parsing
/// process. These methods then return information about the user supplied
/// arguments (or lack thereof).
///
/// When deriving a [`Parser`][crate::Parser], you can use
/// [`CommandFactory::command`][crate::CommandFactory::command] to access the
/// `Command`.
///
/// - [Basic API][crate::App#basic-api]
/// - [Application-wide Settings][crate::App#application-wide-settings]
/// - [Command-specific Settings][crate::App#command-specific-settings]
/// - [Subcommand-specific Settings][crate::App#subcommand-specific-settings]
/// - [Reflection][crate::App#reflection]
///
/// # Examples
///
/// ```no_run
/// # use clap::{Command, Arg};
/// let m = Command::new("My Program")
///     .author("Me, me@mail.com")
///     .version("1.0.2")
///     .about("Explains in brief what the program does")
///     .arg(
///         Arg::new("in_file")
///     )
///     .after_help("Longer explanation to appear after the options when \
///                  displaying the help information from --help or -h")
///     .get_matches();
///
/// // Your program logic starts here...
/// ```
/// [`App::get_matches`]: Command::get_matches()
pub type Command<'help> = App<'help>;

/// Deprecated, replaced with [`Command`]
#[cfg_attr(
    feature = "deprecated",
    deprecated(since = "3.1.0", note = "Replaced with `Command`")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App<'help> {
    id: Id,
    name: String,
    long_flag: Option<&'help str>,
    short_flag: Option<char>,
    display_name: Option<String>,
    bin_name: Option<String>,
    author: Option<&'help str>,
    version: Option<&'help str>,
    long_version: Option<&'help str>,
    about: Option<&'help str>,
    long_about: Option<&'help str>,
    before_help: Option<&'help str>,
    before_long_help: Option<&'help str>,
    after_help: Option<&'help str>,
    after_long_help: Option<&'help str>,
    aliases: Vec<(&'help str, bool)>,           // (name, visible)
    short_flag_aliases: Vec<(char, bool)>,      // (name, visible)
    long_flag_aliases: Vec<(&'help str, bool)>, // (name, visible)
    usage_str: Option<&'help str>,
    usage_name: Option<String>,
    help_str: Option<&'help str>,
    disp_ord: Option<usize>,
    term_w: Option<usize>,
    max_w: Option<usize>,
    template: Option<&'help str>,
    settings: AppFlags,
    g_settings: AppFlags,
    args: MKeyMap<'help>,
    subcommands: Vec<App<'help>>,
    replacers: HashMap<&'help str, &'help [&'help str]>,
    groups: Vec<ArgGroup<'help>>,
    current_help_heading: Option<&'help str>,
    current_disp_ord: Option<usize>,
    subcommand_value_name: Option<&'help str>,
    subcommand_heading: Option<&'help str>,
}

/// # Basic API
impl<'help> App<'help> {
    /// Creates a new instance of an `Command`.
    ///
    /// It is common, but not required, to use binary name as the `name`. This
    /// name will only be displayed to the user when they request to print
    /// version or help and usage information.
    ///
    /// See also [`command!`](crate::command!) and [`crate_name!`](crate::crate_name!).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("My Program")
    /// # ;
    /// ```
    pub fn new<S: Into<String>>(name: S) -> Self {
        /// The actual implementation of `new`, non-generic to save code size.
        ///
        /// If we don't do this rustc will unnecessarily generate multiple versions
        /// of this code.
        fn new_inner<'help>(name: String) -> App<'help> {
            App {
                id: Id::from(&*name),
                name,
                ..Default::default()
            }
            .arg(
                Arg::new("help")
                    .long("help")
                    .help("Print help information")
                    .global(true)
                    .generated(),
            )
            .arg(
                Arg::new("version")
                    .long("version")
                    .help("Print version information")
                    .global(true)
                    .generated(),
            )
        }

        new_inner(name.into())
    }

    /// Adds an [argument] to the list of valid possibilities.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, arg, Arg};
    /// Command::new("myprog")
    ///     // Adding a single "flag" argument with a short and help text, using Arg::new()
    ///     .arg(
    ///         Arg::new("debug")
    ///            .short('d')
    ///            .help("turns on debugging mode")
    ///     )
    ///     // Adding a single "option" argument with a short, a long, and help text using the less
    ///     // verbose Arg::from()
    ///     .arg(
    ///         arg!(-c --config <CONFIG> "Optionally sets a config file to use")
    ///     )
    /// # ;
    /// ```
    /// [argument]: Arg
    #[must_use]
    pub fn arg<A: Into<Arg<'help>>>(mut self, a: A) -> Self {
        let mut arg = a.into();
        if let Some(current_disp_ord) = self.current_disp_ord.as_mut() {
            if !arg.is_positional() && arg.provider != ArgProvider::Generated {
                let current = *current_disp_ord;
                arg.disp_ord.set_implicit(current);
                *current_disp_ord = current + 1;
            }
        }

        arg.help_heading.get_or_insert(self.current_help_heading);
        self.args.push(arg);
        self
    }

    /// Adds multiple [arguments] to the list of valid possibilities.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, arg, Arg};
    /// Command::new("myprog")
    ///     .args(&[
    ///         arg!("[debug] -d 'turns on debugging info'"),
    ///         Arg::new("input").help("the input file to use")
    ///     ])
    /// # ;
    /// ```
    /// [arguments]: Arg
    #[must_use]
    pub fn args<I, T>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arg<'help>>,
    {
        let args = args.into_iter();
        let (lower, _) = args.size_hint();
        self.args.reserve(lower);

        for arg in args {
            self = self.arg(arg);
        }
        self
    }

    /// Allows one to mutate an [`Arg`] after it's been added to a [`Command`].
    ///
    /// This can be useful for modifying the auto-generated help or version arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    ///
    /// let mut cmd = Command::new("foo")
    ///     .arg(Arg::new("bar")
    ///         .short('b'))
    ///     .mut_arg("bar", |a| a.short('B'));
    ///
    /// let res = cmd.try_get_matches_from_mut(vec!["foo", "-b"]);
    ///
    /// // Since we changed `bar`'s short to "B" this should err as there
    /// // is no `-b` anymore, only `-B`
    ///
    /// assert!(res.is_err());
    ///
    /// let res = cmd.try_get_matches_from_mut(vec!["foo", "-B"]);
    /// assert!(res.is_ok());
    /// ```
    #[must_use]
    pub fn mut_arg<T, F>(mut self, arg_id: T, f: F) -> Self
    where
        F: FnOnce(Arg<'help>) -> Arg<'help>,
        T: Key + Into<&'help str>,
    {
        let arg_id: &str = arg_id.into();
        let id = Id::from(arg_id);

        let mut a = self.args.remove_by_name(&id).unwrap_or_else(|| Arg {
            id,
            name: arg_id,
            ..Arg::default()
        });

        if a.provider == ArgProvider::Generated {
            a.provider = ArgProvider::GeneratedMutated;
        }

        self.args.push(f(a));
        self
    }

    /// Allows one to mutate a [`Command`] after it's been added as a subcommand.
    ///
    /// This can be useful for modifying auto-generated arguments of nested subcommands with
    /// [`Command::mut_arg`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    ///
    /// let mut cmd = Command::new("foo")
    ///         .subcommand(Command::new("bar"))
    ///         .mut_subcommand("bar", |subcmd| subcmd.disable_help_flag(true));
    ///
    /// let res = cmd.try_get_matches_from_mut(vec!["foo", "bar", "--help"]);
    ///
    /// // Since we disabled the help flag on the "bar" subcommand, this should err.
    ///
    /// assert!(res.is_err());
    ///
    /// let res = cmd.try_get_matches_from_mut(vec!["foo", "bar"]);
    /// assert!(res.is_ok());
    /// ```
    #[must_use]
    pub fn mut_subcommand<'a, T, F>(mut self, subcmd_id: T, f: F) -> Self
    where
        F: FnOnce(App<'help>) -> App<'help>,
        T: Into<&'a str>,
    {
        let subcmd_id: &str = subcmd_id.into();
        let id = Id::from(subcmd_id);

        let pos = self.subcommands.iter().position(|s| s.id == id);

        let subcmd = if let Some(idx) = pos {
            self.subcommands.remove(idx)
        } else {
            App::new(subcmd_id)
        };

        self.subcommands.push(f(subcmd));
        self
    }

    /// Adds an [`ArgGroup`] to the application.
    ///
    /// [`ArgGroup`]s are a family of related arguments.
    /// By placing them in a logical group, you can build easier requirement and exclusion rules.
    ///
    /// Example use cases:
    /// - Make an entire [`ArgGroup`] required, meaning that one (and *only*
    ///   one) argument from that group must be present at runtime.
    /// - Name an [`ArgGroup`] as a conflict to another argument.
    ///   Meaning any of the arguments that belong to that group will cause a failure if present with
    ///   the conflicting argument.
    /// - Ensure exclusion between arguments.
    /// - Extract a value from a group instead of determining exactly which argument was used.
    ///
    /// # Examples
    ///
    /// The following example demonstrates using an [`ArgGroup`] to ensure that one, and only one,
    /// of the arguments from the specified group is present at runtime.
    ///
    /// ```no_run
    /// # use clap::{Command, arg, ArgGroup};
    /// Command::new("cmd")
    ///     .arg(arg!("--set-ver [ver] 'set the version manually'"))
    ///     .arg(arg!("--major 'auto increase major'"))
    ///     .arg(arg!("--minor 'auto increase minor'"))
    ///     .arg(arg!("--patch 'auto increase patch'"))
    ///     .group(ArgGroup::new("vers")
    ///          .args(&["set-ver", "major", "minor","patch"])
    ///          .required(true))
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn group<G: Into<ArgGroup<'help>>>(mut self, group: G) -> Self {
        self.groups.push(group.into());
        self
    }

    /// Adds multiple [`ArgGroup`]s to the [`Command`] at once.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, arg, ArgGroup};
    /// Command::new("cmd")
    ///     .arg(arg!("--set-ver [ver] 'set the version manually'"))
    ///     .arg(arg!("--major         'auto increase major'"))
    ///     .arg(arg!("--minor         'auto increase minor'"))
    ///     .arg(arg!("--patch         'auto increase patch'"))
    ///     .arg(arg!("-c [FILE]       'a config file'"))
    ///     .arg(arg!("-i [IFACE]      'an interface'"))
    ///     .groups(&[
    ///         ArgGroup::new("vers")
    ///             .args(&["set-ver", "major", "minor","patch"])
    ///             .required(true),
    ///         ArgGroup::new("input")
    ///             .args(&["c", "i"])
    ///     ])
    /// # ;
    /// ```
    #[must_use]
    pub fn groups<I, T>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<ArgGroup<'help>>,
    {
        for g in groups.into_iter() {
            self = self.group(g.into());
        }
        self
    }

    /// Adds a subcommand to the list of valid possibilities.
    ///
    /// Subcommands are effectively sub-[`Command`]s, because they can contain their own arguments,
    /// subcommands, version, usage, etc. They also function just like [`Command`]s, in that they get
    /// their own auto generated help, version, and usage.
    ///
    /// A subcommand's [`Command::name`] will be used for:
    /// - The argument the user passes in
    /// - Programmatically looking up the subcommand
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, arg};
    /// Command::new("myprog")
    ///     .subcommand(Command::new("config")
    ///         .about("Controls configuration features")
    ///         .arg(arg!("<config> 'Required configuration file to use'")))
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn subcommand<S: Into<App<'help>>>(mut self, subcmd: S) -> Self {
        self.subcommands.push(subcmd.into());
        self
    }

    /// Adds multiple subcommands to the list of valid possibilities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, };
    /// # Command::new("myprog")
    /// .subcommands( vec![
    ///        Command::new("config").about("Controls configuration functionality")
    ///                                 .arg(Arg::new("config_file")),
    ///        Command::new("debug").about("Controls debug functionality")])
    /// # ;
    /// ```
    /// [`IntoIterator`]: std::iter::IntoIterator
    #[must_use]
    pub fn subcommands<I, T>(mut self, subcmds: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<App<'help>>,
    {
        for subcmd in subcmds.into_iter() {
            self.subcommands.push(subcmd.into());
        }
        self
    }

    /// Catch problems earlier in the development cycle.
    ///
    /// Most error states are handled as asserts under the assumption they are programming mistake
    /// and not something to handle at runtime.  Rather than relying on tests (manual or automated)
    /// that exhaustively test your CLI to ensure the asserts are evaluated, this will run those
    /// asserts in a way convenient for running as a test.
    ///
    /// **Note::** This will not help with asserts in [`ArgMatches`], those will need exhaustive
    /// testing of your CLI.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ArgAction};
    /// fn cmd() -> Command<'static> {
    ///     Command::new("foo")
    ///         .arg(
    ///             Arg::new("bar").short('b').action(ArgAction::SetTrue)
    ///         )
    /// }
    ///
    /// #[test]
    /// fn verify_app() {
    ///     cmd().debug_assert();
    /// }
    ///
    /// fn main() {
    ///     let m = cmd().get_matches_from(vec!["foo", "-b"]);
    ///     println!("{}", *m.get_one::<bool>("bar").expect("defaulted by clap"));
    /// }
    /// ```
    pub fn debug_assert(mut self) {
        self._build_all();
    }

    /// Custom error message for post-parsing validation
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, ErrorKind};
    /// let mut cmd = Command::new("myprog");
    /// let err = cmd.error(ErrorKind::InvalidValue, "Some failure case");
    /// ```
    pub fn error(&mut self, kind: ErrorKind, message: impl std::fmt::Display) -> Error {
        Error::raw(kind, message).format(self)
    }

    /// Parse [`env::args_os`], exiting on failure.
    ///
    /// # Panics
    ///
    /// If contradictory arguments or settings exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let matches = Command::new("myprog")
    ///     // Args and options go here...
    ///     .get_matches();
    /// ```
    /// [`env::args_os`]: std::env::args_os()
    /// [`App::try_get_matches_from_mut`]: Command::try_get_matches_from_mut()
    #[inline]
    pub fn get_matches(self) -> ArgMatches {
        self.get_matches_from(&mut env::args_os())
    }

    /// Parse [`env::args_os`], exiting on failure.
    ///
    /// Like [`App::get_matches`] but doesn't consume the `Command`.
    ///
    /// # Panics
    ///
    /// If contradictory arguments or settings exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let mut cmd = Command::new("myprog")
    ///     // Args and options go here...
    ///     ;
    /// let matches = cmd.get_matches_mut();
    /// ```
    /// [`env::args_os`]: std::env::args_os()
    /// [`App::get_matches`]: Command::get_matches()
    pub fn get_matches_mut(&mut self) -> ArgMatches {
        self.try_get_matches_from_mut(&mut env::args_os())
            .unwrap_or_else(|e| e.exit())
    }

    /// Parse [`env::args_os`], returning a [`clap::Result`] on failure.
    ///
    /// **NOTE:** This method WILL NOT exit when `--help` or `--version` (or short versions) are
    /// used. It will return a [`clap::Error`], where the [`kind`] is a
    /// [`ErrorKind::DisplayHelp`] or [`ErrorKind::DisplayVersion`] respectively. You must call
    /// [`Error::exit`] or perform a [`std::process::exit`].
    ///
    /// # Panics
    ///
    /// If contradictory arguments or settings exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let matches = Command::new("myprog")
    ///     // Args and options go here...
    ///     .try_get_matches()
    ///     .unwrap_or_else(|e| e.exit());
    /// ```
    /// [`env::args_os`]: std::env::args_os()
    /// [`Error::exit`]: crate::Error::exit()
    /// [`std::process::exit`]: std::process::exit()
    /// [`clap::Result`]: Result
    /// [`clap::Error`]: crate::Error
    /// [`kind`]: crate::Error
    /// [`ErrorKind::DisplayHelp`]: crate::ErrorKind::DisplayHelp
    /// [`ErrorKind::DisplayVersion`]: crate::ErrorKind::DisplayVersion
    #[inline]
    pub fn try_get_matches(self) -> ClapResult<ArgMatches> {
        // Start the parsing
        self.try_get_matches_from(&mut env::args_os())
    }

    /// Parse the specified arguments, exiting on failure.
    ///
    /// **NOTE:** The first argument will be parsed as the binary name unless
    /// [`Command::no_binary_name`] is used.
    ///
    /// # Panics
    ///
    /// If contradictory arguments or settings exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let arg_vec = vec!["my_prog", "some", "args", "to", "parse"];
    ///
    /// let matches = Command::new("myprog")
    ///     // Args and options go here...
    ///     .get_matches_from(arg_vec);
    /// ```
    /// [`App::get_matches`]: Command::get_matches()
    /// [`clap::Result`]: Result
    /// [`Vec`]: std::vec::Vec
    pub fn get_matches_from<I, T>(mut self, itr: I) -> ArgMatches
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        self.try_get_matches_from_mut(itr).unwrap_or_else(|e| {
            drop(self);
            e.exit()
        })
    }

    /// Parse the specified arguments, returning a [`clap::Result`] on failure.
    ///
    /// **NOTE:** This method WILL NOT exit when `--help` or `--version` (or short versions) are
    /// used. It will return a [`clap::Error`], where the [`kind`] is a [`ErrorKind::DisplayHelp`]
    /// or [`ErrorKind::DisplayVersion`] respectively. You must call [`Error::exit`] or
    /// perform a [`std::process::exit`] yourself.
    ///
    /// **NOTE:** The first argument will be parsed as the binary name unless
    /// [`Command::no_binary_name`] is used.
    ///
    /// # Panics
    ///
    /// If contradictory arguments or settings exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let arg_vec = vec!["my_prog", "some", "args", "to", "parse"];
    ///
    /// let matches = Command::new("myprog")
    ///     // Args and options go here...
    ///     .try_get_matches_from(arg_vec)
    ///     .unwrap_or_else(|e| e.exit());
    /// ```
    /// [`App::get_matches_from`]: Command::get_matches_from()
    /// [`App::try_get_matches`]: Command::try_get_matches()
    /// [`Error::exit`]: crate::Error::exit()
    /// [`std::process::exit`]: std::process::exit()
    /// [`clap::Error`]: crate::Error
    /// [`Error::exit`]: crate::Error::exit()
    /// [`kind`]: crate::Error
    /// [`ErrorKind::DisplayHelp`]: crate::ErrorKind::DisplayHelp
    /// [`ErrorKind::DisplayVersion`]: crate::ErrorKind::DisplayVersion
    /// [`clap::Result`]: Result
    pub fn try_get_matches_from<I, T>(mut self, itr: I) -> ClapResult<ArgMatches>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        self.try_get_matches_from_mut(itr)
    }

    /// Parse the specified arguments, returning a [`clap::Result`] on failure.
    ///
    /// Like [`App::try_get_matches_from`] but doesn't consume the `Command`.
    ///
    /// **NOTE:** This method WILL NOT exit when `--help` or `--version` (or short versions) are
    /// used. It will return a [`clap::Error`], where the [`kind`] is a [`ErrorKind::DisplayHelp`]
    /// or [`ErrorKind::DisplayVersion`] respectively. You must call [`Error::exit`] or
    /// perform a [`std::process::exit`] yourself.
    ///
    /// **NOTE:** The first argument will be parsed as the binary name unless
    /// [`Command::no_binary_name`] is used.
    ///
    /// # Panics
    ///
    /// If contradictory arguments or settings exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let arg_vec = vec!["my_prog", "some", "args", "to", "parse"];
    ///
    /// let mut cmd = Command::new("myprog");
    ///     // Args and options go here...
    /// let matches = cmd.try_get_matches_from_mut(arg_vec)
    ///     .unwrap_or_else(|e| e.exit());
    /// ```
    /// [`App::try_get_matches_from`]: Command::try_get_matches_from()
    /// [`clap::Result`]: Result
    /// [`clap::Error`]: crate::Error
    /// [`kind`]: crate::Error
    pub fn try_get_matches_from_mut<I, T>(&mut self, itr: I) -> ClapResult<ArgMatches>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut raw_args = clap_lex::RawArgs::new(itr.into_iter());
        let mut cursor = raw_args.cursor();

        if self.settings.is_set(AppSettings::Multicall) {
            if let Some(argv0) = raw_args.next_os(&mut cursor) {
                let argv0 = Path::new(&argv0);
                if let Some(command) = argv0.file_stem().and_then(|f| f.to_str()) {
                    // Stop borrowing command so we can get another mut ref to it.
                    let command = command.to_owned();
                    debug!(
                        "Command::try_get_matches_from_mut: Parsed command {} from argv",
                        command
                    );

                    debug!("Command::try_get_matches_from_mut: Reinserting command into arguments so subcommand parser matches it");
                    raw_args.insert(&cursor, &[&command]);
                    debug!("Command::try_get_matches_from_mut: Clearing name and bin_name so that displayed command name starts with applet name");
                    self.name.clear();
                    self.bin_name = None;
                    return self._do_parse(&mut raw_args, cursor);
                }
            }
        };

        // Get the name of the program (argument 1 of env::args()) and determine the
        // actual file
        // that was used to execute the program. This is because a program called
        // ./target/release/my_prog -a
        // will have two arguments, './target/release/my_prog', '-a' but we don't want
        // to display
        // the full path when displaying help messages and such
        if !self.settings.is_set(AppSettings::NoBinaryName) {
            if let Some(name) = raw_args.next_os(&mut cursor) {
                let p = Path::new(name);

                if let Some(f) = p.file_name() {
                    if let Some(s) = f.to_str() {
                        if self.bin_name.is_none() {
                            self.bin_name = Some(s.to_owned());
                        }
                    }
                }
            }
        }

        self._do_parse(&mut raw_args, cursor)
    }

    /// Prints the short help message (`-h`) to [`io::stdout()`].
    ///
    /// See also [`Command::print_long_help`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// let mut cmd = Command::new("myprog");
    /// cmd.print_help();
    /// ```
    /// [`io::stdout()`]: std::io::stdout()
    pub fn print_help(&mut self) -> io::Result<()> {
        self._build_self();
        let color = self.color_help();

        let mut c = Colorizer::new(Stream::Stdout, color);
        let usage = Usage::new(self);
        Help::new(HelpWriter::Buffer(&mut c), self, &usage, false).write_help()?;
        c.print()
    }

    /// Prints the long help message (`--help`) to [`io::stdout()`].
    ///
    /// See also [`Command::print_help`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// let mut cmd = Command::new("myprog");
    /// cmd.print_long_help();
    /// ```
    /// [`io::stdout()`]: std::io::stdout()
    /// [`BufWriter`]: std::io::BufWriter
    /// [`-h` (short)]: Arg::help()
    /// [`--help` (long)]: Arg::long_help()
    pub fn print_long_help(&mut self) -> io::Result<()> {
        self._build_self();
        let color = self.color_help();

        let mut c = Colorizer::new(Stream::Stdout, color);
        let usage = Usage::new(self);
        Help::new(HelpWriter::Buffer(&mut c), self, &usage, true).write_help()?;
        c.print()
    }

    /// Writes the short help message (`-h`) to a [`io::Write`] object.
    ///
    /// See also [`Command::write_long_help`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// use std::io;
    /// let mut cmd = Command::new("myprog");
    /// let mut out = io::stdout();
    /// cmd.write_help(&mut out).expect("failed to write to stdout");
    /// ```
    /// [`io::Write`]: std::io::Write
    /// [`-h` (short)]: Arg::help()
    /// [`--help` (long)]: Arg::long_help()
    pub fn write_help<W: io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self._build_self();

        let usage = Usage::new(self);
        Help::new(HelpWriter::Normal(w), self, &usage, false).write_help()?;
        w.flush()
    }

    /// Writes the long help message (`--help`) to a [`io::Write`] object.
    ///
    /// See also [`Command::write_help`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// use std::io;
    /// let mut cmd = Command::new("myprog");
    /// let mut out = io::stdout();
    /// cmd.write_long_help(&mut out).expect("failed to write to stdout");
    /// ```
    /// [`io::Write`]: std::io::Write
    /// [`-h` (short)]: Arg::help()
    /// [`--help` (long)]: Arg::long_help()
    pub fn write_long_help<W: io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self._build_self();

        let usage = Usage::new(self);
        Help::new(HelpWriter::Normal(w), self, &usage, true).write_help()?;
        w.flush()
    }

    /// Version message rendered as if the user ran `-V`.
    ///
    /// See also [`Command::render_long_version`].
    ///
    /// ### Coloring
    ///
    /// This function does not try to color the message nor it inserts any [ANSI escape codes].
    ///
    /// ### Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// use std::io;
    /// let cmd = Command::new("myprog");
    /// println!("{}", cmd.render_version());
    /// ```
    /// [`io::Write`]: std::io::Write
    /// [`-V` (short)]: Command::version()
    /// [`--version` (long)]: Command::long_version()
    /// [ANSI escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code
    pub fn render_version(&self) -> String {
        self._render_version(false)
    }

    /// Version message rendered as if the user ran `--version`.
    ///
    /// See also [`Command::render_version`].
    ///
    /// ### Coloring
    ///
    /// This function does not try to color the message nor it inserts any [ANSI escape codes].
    ///
    /// ### Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// use std::io;
    /// let cmd = Command::new("myprog");
    /// println!("{}", cmd.render_long_version());
    /// ```
    /// [`io::Write`]: std::io::Write
    /// [`-V` (short)]: Command::version()
    /// [`--version` (long)]: Command::long_version()
    /// [ANSI escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code
    pub fn render_long_version(&self) -> String {
        self._render_version(true)
    }

    /// Usage statement
    ///
    /// ### Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// use std::io;
    /// let mut cmd = Command::new("myprog");
    /// println!("{}", cmd.render_usage());
    /// ```
    pub fn render_usage(&mut self) -> String {
        // If there are global arguments, or settings we need to propagate them down to subcommands
        // before parsing incase we run into a subcommand
        self._build_self();

        Usage::new(self).create_usage_with_title(&[])
    }
}

/// # Application-wide Settings
///
/// These settings will apply to the top-level command and all subcommands, by default.  Some
/// settings can be overridden in subcommands.
impl<'help> App<'help> {
    /// Specifies that the parser should not assume the first argument passed is the binary name.
    ///
    /// This is normally the case when using a "daemon" style mode.  For shells / REPLs, see
    /// [`Command::multicall`][App::multicall].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, arg};
    /// let m = Command::new("myprog")
    ///     .no_binary_name(true)
    ///     .arg(arg!(<cmd> ... "commands to run"))
    ///     .get_matches_from(vec!["command", "set"]);
    ///
    /// let cmds: Vec<&str> = m.values_of("cmd").unwrap().collect();
    /// assert_eq!(cmds, ["command", "set"]);
    /// ```
    /// [`try_get_matches_from_mut`]: crate::Command::try_get_matches_from_mut()
    #[inline]
    pub fn no_binary_name(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::NoBinaryName)
        } else {
            self.unset_global_setting(AppSettings::NoBinaryName)
        }
    }

    /// Try not to fail on parse errors, like missing option values.
    ///
    /// **Note:** Make sure you apply it as `global_setting` if you want this setting
    /// to be propagated to subcommands and sub-subcommands!
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, arg};
    /// let cmd = Command::new("cmd")
    ///   .ignore_errors(true)
    ///   .arg(arg!(-c --config <FILE> "Sets a custom config file").required(false))
    ///   .arg(arg!(-x --stuff <FILE> "Sets a custom stuff file").required(false))
    ///   .arg(arg!(f: -f "Flag"));
    ///
    /// let r = cmd.try_get_matches_from(vec!["cmd", "-c", "file", "-f", "-x"]);
    ///
    /// assert!(r.is_ok(), "unexpected error: {:?}", r);
    /// let m = r.unwrap();
    /// assert_eq!(m.value_of("config"), Some("file"));
    /// assert!(m.is_present("f"));
    /// assert_eq!(m.value_of("stuff"), None);
    /// ```
    #[inline]
    pub fn ignore_errors(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::IgnoreErrors)
        } else {
            self.unset_global_setting(AppSettings::IgnoreErrors)
        }
    }

    /// Deprecated, replaced with [`ArgAction::Set`][super::ArgAction::Set]
    ///
    /// The new actions (`ArgAction::Set`, `ArgAction::SetTrue`) do this by default.
    ///
    /// See `ArgAction::StoreValue` and `ArgAction::IncOccurrence` for how to migrate
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.2.0",
            note = "Replaced with `Arg::action(ArgAction::...)`

The new actions (`ArgAction::Set`, `ArgAction::SetTrue`) do this by default.

See `ArgAction::StoreValue` and `ArgAction::IncOccurrence` for how to migrate
"
        )
    )]
    pub fn args_override_self(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::AllArgsOverrideSelf)
        } else {
            self.unset_global_setting(AppSettings::AllArgsOverrideSelf)
        }
    }

    /// Disables the automatic delimiting of values after `--` or when [`Command::trailing_var_arg`]
    /// was used.
    ///
    /// **NOTE:** The same thing can be done manually by setting the final positional argument to
    /// [`Arg::use_value_delimiter(false)`]. Using this setting is safer, because it's easier to locate
    /// when making changes.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .dont_delimit_trailing_values(true)
    ///     .get_matches();
    /// ```
    ///
    /// [`Arg::use_value_delimiter(false)`]: crate::Arg::use_value_delimiter()
    #[inline]
    pub fn dont_delimit_trailing_values(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::DontDelimitTrailingValues)
        } else {
            self.unset_global_setting(AppSettings::DontDelimitTrailingValues)
        }
    }

    /// Sets when to color output.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// **NOTE:** Default behaviour is [`ColorChoice::Auto`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, ColorChoice};
    /// Command::new("myprog")
    ///     .color(ColorChoice::Never)
    ///     .get_matches();
    /// ```
    /// [`ColorChoice::Auto`]: crate::ColorChoice::Auto
    #[cfg(feature = "color")]
    #[inline]
    #[must_use]
    pub fn color(self, color: ColorChoice) -> Self {
        #![allow(deprecated)]
        let cmd = self
            .unset_global_setting(AppSettings::ColorAuto)
            .unset_global_setting(AppSettings::ColorAlways)
            .unset_global_setting(AppSettings::ColorNever);
        match color {
            ColorChoice::Auto => cmd.global_setting(AppSettings::ColorAuto),
            ColorChoice::Always => cmd.global_setting(AppSettings::ColorAlways),
            ColorChoice::Never => cmd.global_setting(AppSettings::ColorNever),
        }
    }

    /// Sets the terminal width at which to wrap help messages.
    ///
    /// Using `0` will ignore terminal widths and use source formatting.
    ///
    /// Defaults to current terminal width when `wrap_help` feature flag is enabled.  If the flag
    /// is disabled or it cannot be determined, the default is 100.
    ///
    /// **NOTE:** This setting applies globally and *not* on a per-command basis.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .term_width(80)
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn term_width(mut self, width: usize) -> Self {
        self.term_w = Some(width);
        self
    }

    /// Sets the maximum terminal width at which to wrap help messages.
    ///
    /// This only applies when setting the current terminal width.  See [`Command::term_width`] for
    /// more details.
    ///
    /// Using `0` will ignore terminal widths and use source formatting.
    ///
    /// **NOTE:** This setting applies globally and *not* on a per-command basis.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .max_term_width(100)
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn max_term_width(mut self, w: usize) -> Self {
        self.max_w = Some(w);
        self
    }

    /// Disables `-V` and `--version` flag.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, ErrorKind};
    /// let res = Command::new("myprog")
    ///     .disable_version_flag(true)
    ///     .try_get_matches_from(vec![
    ///         "myprog", "-V"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind(), ErrorKind::UnknownArgument);
    /// ```
    #[inline]
    pub fn disable_version_flag(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::DisableVersionFlag)
        } else {
            self.unset_global_setting(AppSettings::DisableVersionFlag)
        }
    }

    /// Specifies to use the version of the current command for all [`subcommands`].
    ///
    /// Defaults to `false`; subcommands have independent version strings from their parents.
    ///
    /// **Note:** Make sure you apply it as `global_setting` if you want this setting
    /// to be propagated to subcommands and sub-subcommands!
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .version("v1.1")
    ///     .propagate_version(true)
    ///     .subcommand(Command::new("test"))
    ///     .get_matches();
    /// // running `$ myprog test --version` will display
    /// // "myprog-test v1.1"
    /// ```
    ///
    /// [`subcommands`]: crate::Command::subcommand()
    #[inline]
    pub fn propagate_version(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::PropagateVersion)
        } else {
            self.unset_global_setting(AppSettings::PropagateVersion)
        }
    }

    /// Places the help string for all arguments and subcommands on the line after them.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .next_line_help(true)
    ///     .get_matches();
    /// ```
    #[inline]
    pub fn next_line_help(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::NextLineHelp)
        } else {
            self.unset_global_setting(AppSettings::NextLineHelp)
        }
    }

    /// Disables `-h` and `--help` flag.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, ErrorKind};
    /// let res = Command::new("myprog")
    ///     .disable_help_flag(true)
    ///     .try_get_matches_from(vec![
    ///         "myprog", "-h"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind(), ErrorKind::UnknownArgument);
    /// ```
    #[inline]
    pub fn disable_help_flag(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::DisableHelpFlag)
        } else {
            self.unset_global_setting(AppSettings::DisableHelpFlag)
        }
    }

    /// Disables the `help` [`subcommand`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, ErrorKind};
    /// let res = Command::new("myprog")
    ///     .disable_help_subcommand(true)
    ///     // Normally, creating a subcommand causes a `help` subcommand to automatically
    ///     // be generated as well
    ///     .subcommand(Command::new("test"))
    ///     .try_get_matches_from(vec![
    ///         "myprog", "help"
    ///     ]);
    /// assert!(res.is_err());
    /// assert_eq!(res.unwrap_err().kind(), ErrorKind::UnknownArgument);
    /// ```
    ///
    /// [`subcommand`]: crate::Command::subcommand()
    #[inline]
    pub fn disable_help_subcommand(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::DisableHelpSubcommand)
        } else {
            self.unset_global_setting(AppSettings::DisableHelpSubcommand)
        }
    }

    /// Disables colorized help messages.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .disable_colored_help(true)
    ///     .get_matches();
    /// ```
    #[inline]
    pub fn disable_colored_help(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::DisableColoredHelp)
        } else {
            self.unset_global_setting(AppSettings::DisableColoredHelp)
        }
    }

    /// Panic if help descriptions are omitted.
    ///
    /// **NOTE:** When deriving [`Parser`][crate::Parser], you could instead check this at
    /// compile-time with `#![deny(missing_docs)]`
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .help_expected(true)
    ///     .arg(
    ///         Arg::new("foo").help("It does foo stuff")
    ///         // As required via `help_expected`, a help message was supplied
    ///      )
    /// #    .get_matches();
    /// ```
    ///
    /// # Panics
    ///
    /// ```rust,no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myapp")
    ///     .help_expected(true)
    ///     .arg(
    ///         Arg::new("foo")
    ///         // Someone forgot to put .about("...") here
    ///         // Since the setting `help_expected` is activated, this will lead to
    ///         // a panic (if you are in debug mode)
    ///     )
    /// #   .get_matches();
    ///```
    #[inline]
    pub fn help_expected(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::HelpExpected)
        } else {
            self.unset_global_setting(AppSettings::HelpExpected)
        }
    }

    /// Disables the automatic collapsing of positional args into `[ARGS]` inside the usage string.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .dont_collapse_args_in_usage(true)
    ///     .get_matches();
    /// ```
    #[inline]
    pub fn dont_collapse_args_in_usage(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::DontCollapseArgsInUsage)
        } else {
            self.unset_global_setting(AppSettings::DontCollapseArgsInUsage)
        }
    }

    /// Tells `clap` *not* to print possible values when displaying help information.
    ///
    /// This can be useful if there are many values, or they are explained elsewhere.
    ///
    /// To set this per argument, see
    /// [`Arg::hide_possible_values`][crate::Arg::hide_possible_values].
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    #[inline]
    pub fn hide_possible_values(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::HidePossibleValues)
        } else {
            self.unset_global_setting(AppSettings::HidePossibleValues)
        }
    }

    /// Allow partial matches of long arguments or their [aliases].
    ///
    /// For example, to match an argument named `--test`, one could use `--t`, `--te`, `--tes`, and
    /// `--test`.
    ///
    /// **NOTE:** The match *must not* be ambiguous at all in order to succeed. i.e. to match
    /// `--te` to `--test` there could not also be another argument or alias `--temp` because both
    /// start with `--te`
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// [aliases]: crate::Command::aliases()
    #[inline]
    pub fn infer_long_args(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::InferLongArgs)
        } else {
            self.unset_global_setting(AppSettings::InferLongArgs)
        }
    }

    /// Allow partial matches of [subcommand] names and their [aliases].
    ///
    /// For example, to match a subcommand named `test`, one could use `t`, `te`, `tes`, and
    /// `test`.
    ///
    /// **NOTE:** The match *must not* be ambiguous at all in order to succeed. i.e. to match `te`
    /// to `test` there could not also be a subcommand or alias `temp` because both start with `te`
    ///
    /// **CAUTION:** This setting can interfere with [positional/free arguments], take care when
    /// designing CLIs which allow inferred subcommands and have potential positional/free
    /// arguments whose values could start with the same characters as subcommands. If this is the
    /// case, it's recommended to use settings such as [`Command::args_conflicts_with_subcommands`] in
    /// conjunction with this setting.
    ///
    /// **NOTE:** This choice is propagated to all child subcommands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let m = Command::new("prog")
    ///     .infer_subcommands(true)
    ///     .subcommand(Command::new("test"))
    ///     .get_matches_from(vec![
    ///         "prog", "te"
    ///     ]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    ///
    /// [subcommand]: crate::Command::subcommand()
    /// [positional/free arguments]: crate::Arg::index()
    /// [aliases]: crate::Command::aliases()
    #[inline]
    pub fn infer_subcommands(self, yes: bool) -> Self {
        if yes {
            self.global_setting(AppSettings::InferSubcommands)
        } else {
            self.unset_global_setting(AppSettings::InferSubcommands)
        }
    }
}

/// # Command-specific Settings
///
/// These apply only to the current command and are not inherited by subcommands.
impl<'help> App<'help> {
    /// (Re)Sets the program's name.
    ///
    /// See [`Command::new`] for more details.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use clap::{Command, load_yaml};
    /// let yaml = load_yaml!("cmd.yaml");
    /// let cmd = Command::from(yaml)
    ///     .name(crate_name!());
    ///
    /// // continued logic goes here, such as `cmd.get_matches()` etc.
    /// ```
    #[must_use]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    /// Overrides the runtime-determined name of the binary for help and error messages.
    ///
    /// This should only be used when absolutely necessary, such as when the binary name for your
    /// application is misleading, or perhaps *not* how the user should invoke your program.
    ///
    /// **Pro-tip:** When building things such as third party `cargo`
    /// subcommands, this setting **should** be used!
    ///
    /// **NOTE:** This *does not* change or set the name of the binary file on
    /// disk. It only changes what clap thinks the name is for the purposes of
    /// error or help messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("My Program")
    ///      .bin_name("my_binary")
    /// # ;
    /// ```
    #[must_use]
    pub fn bin_name<S: Into<String>>(mut self, name: S) -> Self {
        self.bin_name = Some(name.into());
        self
    }

    /// Overrides the runtime-determined display name of the program for help and error messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("My Program")
    ///      .display_name("my_program")
    /// # ;
    /// ```
    #[must_use]
    pub fn display_name<S: Into<String>>(mut self, name: S) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the author(s) for the help message.
    ///
    /// **Pro-tip:** Use `clap`s convenience macro [`crate_authors!`] to
    /// automatically set your application's author(s) to the same thing as your
    /// crate at compile time.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///      .author("Me, me@mymain.com")
    /// # ;
    /// ```
    /// [`crate_authors!`]: ./macro.crate_authors!.html
    #[must_use]
    pub fn author<S: Into<&'help str>>(mut self, author: S) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the program's description for the short help (`-h`).
    ///
    /// If [`Command::long_about`] is not specified, this message will be displayed for `--help`.
    ///
    /// **NOTE:** Only `Command::about` (short format) is used in completion
    /// script generation in order to be concise.
    ///
    /// See also [`crate_description!`](crate::crate_description!).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .about("Does really amazing things for great people")
    /// # ;
    /// ```
    #[must_use]
    pub fn about<O: Into<Option<&'help str>>>(mut self, about: O) -> Self {
        self.about = about.into();
        self
    }

    /// Sets the program's description for the long help (`--help`).
    ///
    /// If [`Command::about`] is not specified, this message will be displayed for `-h`.
    ///
    /// **NOTE:** Only [`Command::about`] (short format) is used in completion
    /// script generation in order to be concise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .long_about(
    /// "Does really amazing things to great people. Now let's talk a little
    ///  more in depth about how this subcommand really works. It may take about
    ///  a few lines of text, but that's ok!")
    /// # ;
    /// ```
    /// [`App::about`]: Command::about()
    #[must_use]
    pub fn long_about<O: Into<Option<&'help str>>>(mut self, long_about: O) -> Self {
        self.long_about = long_about.into();
        self
    }

    /// Free-form help text for after auto-generated short help (`-h`).
    ///
    /// This is often used to describe how to use the arguments, caveats to be noted, or license
    /// and contact information.
    ///
    /// If [`Command::after_long_help`] is not specified, this message will be displayed for `--help`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .after_help("Does really amazing things for great people... but be careful with -R!")
    /// # ;
    /// ```
    ///
    #[must_use]
    pub fn after_help<S: Into<&'help str>>(mut self, help: S) -> Self {
        self.after_help = Some(help.into());
        self
    }

    /// Free-form help text for after auto-generated long help (`--help`).
    ///
    /// This is often used to describe how to use the arguments, caveats to be noted, or license
    /// and contact information.
    ///
    /// If [`Command::after_help`] is not specified, this message will be displayed for `-h`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .after_long_help("Does really amazing things to great people... but be careful with -R, \
    ///                      like, for real, be careful with this!")
    /// # ;
    /// ```
    #[must_use]
    pub fn after_long_help<S: Into<&'help str>>(mut self, help: S) -> Self {
        self.after_long_help = Some(help.into());
        self
    }

    /// Free-form help text for before auto-generated short help (`-h`).
    ///
    /// This is often used for header, copyright, or license information.
    ///
    /// If [`Command::before_long_help`] is not specified, this message will be displayed for `--help`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .before_help("Some info I'd like to appear before the help info")
    /// # ;
    /// ```
    #[must_use]
    pub fn before_help<S: Into<&'help str>>(mut self, help: S) -> Self {
        self.before_help = Some(help.into());
        self
    }

    /// Free-form help text for before auto-generated long help (`--help`).
    ///
    /// This is often used for header, copyright, or license information.
    ///
    /// If [`Command::before_help`] is not specified, this message will be displayed for `-h`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .before_long_help("Some verbose and long info I'd like to appear before the help info")
    /// # ;
    /// ```
    #[must_use]
    pub fn before_long_help<S: Into<&'help str>>(mut self, help: S) -> Self {
        self.before_long_help = Some(help.into());
        self
    }

    /// Sets the version for the short version (`-V`) and help messages.
    ///
    /// If [`Command::long_version`] is not specified, this message will be displayed for `--version`.
    ///
    /// **Pro-tip:** Use `clap`s convenience macro [`crate_version!`] to
    /// automatically set your application's version to the same thing as your
    /// crate at compile time.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .version("v0.1.24")
    /// # ;
    /// ```
    /// [`crate_version!`]: ./macro.crate_version!.html
    #[must_use]
    pub fn version<S: Into<&'help str>>(mut self, ver: S) -> Self {
        self.version = Some(ver.into());
        self
    }

    /// Sets the version for the long version (`--version`) and help messages.
    ///
    /// If [`Command::version`] is not specified, this message will be displayed for `-V`.
    ///
    /// **Pro-tip:** Use `clap`s convenience macro [`crate_version!`] to
    /// automatically set your application's version to the same thing as your
    /// crate at compile time.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .long_version(
    /// "v0.1.24
    ///  commit: abcdef89726d
    ///  revision: 123
    ///  release: 2
    ///  binary: myprog")
    /// # ;
    /// ```
    /// [`crate_version!`]: ./macro.crate_version!.html
    #[must_use]
    pub fn long_version<S: Into<&'help str>>(mut self, ver: S) -> Self {
        self.long_version = Some(ver.into());
        self
    }

    /// Overrides the `clap` generated usage string for help and error messages.
    ///
    /// **NOTE:** Using this setting disables `clap`s "context-aware" usage
    /// strings. After this setting is set, this will be *the only* usage string
    /// displayed to the user!
    ///
    /// **NOTE:** Multiple usage lines may be present in the usage argument, but
    /// some rules need to be followed to ensure the usage lines are formatted
    /// correctly by the default help formatter:
    ///
    /// - Do not indent the first usage line.
    /// - Indent all subsequent usage lines with four spaces.
    /// - The last line must not end with a newline.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .override_usage("myapp [-clDas] <some_file>")
    /// # ;
    /// ```
    ///
    /// Or for multiple usage lines:
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .override_usage(
    ///         "myapp -X [-a] [-b] <file>\n    \
    ///          myapp -Y [-c] <file1> <file2>\n    \
    ///          myapp -Z [-d|-e]"
    ///     )
    /// # ;
    /// ```
    ///
    /// [`ArgMatches::usage`]: ArgMatches::usage()
    #[must_use]
    pub fn override_usage<S: Into<&'help str>>(mut self, usage: S) -> Self {
        self.usage_str = Some(usage.into());
        self
    }

    /// Overrides the `clap` generated help message (both `-h` and `--help`).
    ///
    /// This should only be used when the auto-generated message does not suffice.
    ///
    /// **NOTE:** This **only** replaces the help message for the current
    /// command, meaning if you are using subcommands, those help messages will
    /// still be auto-generated unless you specify a [`Command::override_help`] for
    /// them as well.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myapp")
    ///     .override_help("myapp v1.0\n\
    ///            Does awesome things\n\
    ///            (C) me@mail.com\n\n\
    ///
    ///            USAGE: myapp <opts> <command>\n\n\
    ///
    ///            Options:\n\
    ///            -h, --help       Display this message\n\
    ///            -V, --version    Display version info\n\
    ///            -s <stuff>       Do something with stuff\n\
    ///            -v               Be verbose\n\n\
    ///
    ///            Commands:\n\
    ///            help             Print this message\n\
    ///            work             Do some work")
    /// # ;
    /// ```
    #[must_use]
    pub fn override_help<S: Into<&'help str>>(mut self, help: S) -> Self {
        self.help_str = Some(help.into());
        self
    }

    /// Sets the help template to be used, overriding the default format.
    ///
    /// **NOTE:** The template system is by design very simple. Therefore, the
    /// tags have to be written in the lowercase and without spacing.
    ///
    /// Tags are given inside curly brackets.
    ///
    /// Valid tags are:
    ///
    ///   * `{name}`                - Display name for the (sub-)command.
    ///   * `{bin}`                 - Binary name.
    ///   * `{version}`             - Version number.
    ///   * `{author}`              - Author information.
    ///   * `{author-with-newline}` - Author followed by `\n`.
    ///   * `{author-section}`      - Author preceded and followed by `\n`.
    ///   * `{about}`               - General description (from [`Command::about`] or
    ///                               [`Command::long_about`]).
    ///   * `{about-with-newline}`  - About followed by `\n`.
    ///   * `{about-section}`       - About preceded and followed by '\n'.
    ///   * `{usage-heading}`       - Automatically generated usage heading.
    ///   * `{usage}`               - Automatically generated or given usage string.
    ///   * `{all-args}`            - Help for all arguments (options, flags, positional
    ///                               arguments, and subcommands) including titles.
    ///   * `{options}`             - Help for options.
    ///   * `{positionals}`         - Help for positional arguments.
    ///   * `{subcommands}`         - Help for subcommands.
    ///   * `{after-help}`          - Help from [`App::after_help`] or [`Command::after_long_help`].
    ///   * `{before-help}`         - Help from [`App::before_help`] or [`Command::before_long_help`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .version("1.0")
    ///     .help_template("{bin} ({version}) - {usage}")
    /// # ;
    /// ```
    /// [`App::about`]: Command::about()
    /// [`App::long_about`]: Command::long_about()
    /// [`App::after_help`]: Command::after_help()
    /// [`App::after_long_help`]: Command::after_long_help()
    /// [`App::before_help`]: Command::before_help()
    /// [`App::before_long_help`]: Command::before_long_help()
    #[must_use]
    pub fn help_template<S: Into<&'help str>>(mut self, s: S) -> Self {
        self.template = Some(s.into());
        self
    }

    /// Apply a setting for the current command or subcommand.
    ///
    /// See [`Command::global_setting`] to apply a setting to this command and all subcommands.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, AppSettings};
    /// Command::new("myprog")
    ///     .setting(AppSettings::SubcommandRequired)
    ///     .setting(AppSettings::AllowLeadingHyphen)
    /// # ;
    /// ```
    /// or
    /// ```no_run
    /// # use clap::{Command, AppSettings};
    /// Command::new("myprog")
    ///     .setting(AppSettings::SubcommandRequired | AppSettings::AllowLeadingHyphen)
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn setting<F>(mut self, setting: F) -> Self
    where
        F: Into<AppFlags>,
    {
        self.settings.insert(setting.into());
        self
    }

    /// Remove a setting for the current command or subcommand.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, AppSettings};
    /// Command::new("myprog")
    ///     .unset_setting(AppSettings::SubcommandRequired)
    ///     .setting(AppSettings::AllowLeadingHyphen)
    /// # ;
    /// ```
    /// or
    /// ```no_run
    /// # use clap::{Command, AppSettings};
    /// Command::new("myprog")
    ///     .unset_setting(AppSettings::SubcommandRequired | AppSettings::AllowLeadingHyphen)
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn unset_setting<F>(mut self, setting: F) -> Self
    where
        F: Into<AppFlags>,
    {
        self.settings.remove(setting.into());
        self
    }

    /// Apply a setting for the current command and all subcommands.
    ///
    /// See [`Command::setting`] to apply a setting only to this command.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, AppSettings};
    /// Command::new("myprog")
    ///     .global_setting(AppSettings::AllowNegativeNumbers)
    /// # ;
    /// ```
    #[inline]
    #[must_use]
    pub fn global_setting(mut self, setting: AppSettings) -> Self {
        self.settings.set(setting);
        self.g_settings.set(setting);
        self
    }

    /// Remove a setting and stop propagating down to subcommands.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, AppSettings};
    /// Command::new("myprog")
    ///     .unset_global_setting(AppSettings::AllowNegativeNumbers)
    /// # ;
    /// ```
    /// [global]: Command::global_setting()
    #[inline]
    #[must_use]
    pub fn unset_global_setting(mut self, setting: AppSettings) -> Self {
        self.settings.unset(setting);
        self.g_settings.unset(setting);
        self
    }

    /// Deprecated, replaced with [`Command::next_help_heading`]
    #[inline]
    #[must_use]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.0", note = "Replaced with `App::next_help_heading`")
    )]
    pub fn help_heading<O>(self, heading: O) -> Self
    where
        O: Into<Option<&'help str>>,
    {
        self.next_help_heading(heading)
    }

    /// Set the default section heading for future args.
    ///
    /// This will be used for any arg that hasn't had [`Arg::help_heading`] called.
    ///
    /// This is useful if the default `OPTIONS` or `ARGS` headings are
    /// not specific enough for one's use case.
    ///
    /// For subcommands, see [`Command::subcommand_help_heading`]
    ///
    /// [`App::arg`]: Command::arg()
    /// [`Arg::help_heading`]: crate::Arg::help_heading()
    #[inline]
    #[must_use]
    pub fn next_help_heading<O>(mut self, heading: O) -> Self
    where
        O: Into<Option<&'help str>>,
    {
        self.current_help_heading = heading.into();
        self
    }

    /// Change the starting value for assigning future display orders for ags.
    ///
    /// This will be used for any arg that hasn't had [`Arg::display_order`] called.
    #[inline]
    #[must_use]
    pub fn next_display_order(mut self, disp_ord: impl Into<Option<usize>>) -> Self {
        self.current_disp_ord = disp_ord.into();
        self
    }

    /// Replaces an argument or subcommand used on the CLI at runtime with other arguments or subcommands.
    ///
    /// **Note:** This is gated behind [`unstable-replace`](https://github.com/clap-rs/clap/issues/2836)
    ///
    /// When this method is used, `name` is removed from the CLI, and `target`
    /// is inserted in its place. Parsing continues as if the user typed
    /// `target` instead of `name`.
    ///
    /// This can be used to create "shortcuts" for subcommands, or if a
    /// particular argument has the semantic meaning of several other specific
    /// arguments and values.
    ///
    /// # Examples
    ///
    /// We'll start with the "subcommand short" example. In this example, let's
    /// assume we have a program with a subcommand `module` which can be invoked
    /// via `cmd module`. Now let's also assume `module` also has a subcommand
    /// called `install` which can be invoked `cmd module install`. If for some
    /// reason users needed to be able to reach `cmd module install` via the
    /// short-hand `cmd install`, we'd have several options.
    ///
    /// We *could* create another sibling subcommand to `module` called
    /// `install`, but then we would need to manage another subcommand and manually
    /// dispatch to `cmd module install` handling code. This is error prone and
    /// tedious.
    ///
    /// We could instead use [`Command::replace`] so that, when the user types `cmd
    /// install`, `clap` will replace `install` with `module install` which will
    /// end up getting parsed as if the user typed the entire incantation.
    ///
    /// ```rust
    /// # use clap::Command;
    /// let m = Command::new("cmd")
    ///     .subcommand(Command::new("module")
    ///         .subcommand(Command::new("install")))
    ///     .replace("install", &["module", "install"])
    ///     .get_matches_from(vec!["cmd", "install"]);
    ///
    /// assert!(m.subcommand_matches("module").is_some());
    /// assert!(m.subcommand_matches("module").unwrap().subcommand_matches("install").is_some());
    /// ```
    ///
    /// Now let's show an argument example!
    ///
    /// Let's assume we have an application with two flags `--save-context` and
    /// `--save-runtime`. But often users end up needing to do *both* at the
    /// same time. We can add a third flag `--save-all` which semantically means
    /// the same thing as `cmd --save-context --save-runtime`. To implement that,
    /// we have several options.
    ///
    /// We could create this third argument and manually check if that argument
    /// and in our own consumer code handle the fact that both `--save-context`
    /// and `--save-runtime` *should* have been used. But again this is error
    /// prone and tedious. If we had code relying on checking `--save-context`
    /// and we forgot to update that code to *also* check `--save-all` it'd mean
    /// an error!
    ///
    /// Luckily we can use [`Command::replace`] so that when the user types
    /// `--save-all`, `clap` will replace that argument with `--save-context
    /// --save-runtime`, and parsing will continue like normal. Now all our code
    /// that was originally checking for things like `--save-context` doesn't
    /// need to change!
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ArgAction};
    /// let m = Command::new("cmd")
    ///     .arg(Arg::new("save-context")
    ///         .long("save-context")
    ///         .action(ArgAction::SetTrue))
    ///     .arg(Arg::new("save-runtime")
    ///         .long("save-runtime")
    ///         .action(ArgAction::SetTrue))
    ///     .replace("--save-all", &["--save-context", "--save-runtime"])
    ///     .get_matches_from(vec!["cmd", "--save-all"]);
    ///
    /// assert!(*m.get_one::<bool>("save-context").expect("defaulted by clap"));
    /// assert!(*m.get_one::<bool>("save-runtime").expect("defaulted by clap"));
    /// ```
    ///
    /// This can also be used with options, for example if our application with
    /// `--save-*` above also had a `--format=TYPE` option. Let's say it
    /// accepted `txt` or `json` values. However, when `--save-all` is used,
    /// only `--format=json` is allowed, or valid. We could change the example
    /// above to enforce this:
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ArgAction};
    /// let m = Command::new("cmd")
    ///     .arg(Arg::new("save-context")
    ///         .long("save-context")
    ///         .action(ArgAction::SetTrue))
    ///     .arg(Arg::new("save-runtime")
    ///         .long("save-runtime")
    ///         .action(ArgAction::SetTrue))
    ///     .arg(Arg::new("format")
    ///         .long("format")
    ///         .takes_value(true)
    ///         .value_parser(["txt", "json"]))
    ///     .replace("--save-all", &["--save-context", "--save-runtime", "--format=json"])
    ///     .get_matches_from(vec!["cmd", "--save-all"]);
    ///
    /// assert!(*m.get_one::<bool>("save-context").expect("defaulted by clap"));
    /// assert!(*m.get_one::<bool>("save-runtime").expect("defaulted by clap"));
    /// assert_eq!(m.value_of("format"), Some("json"));
    /// ```
    ///
    /// [`App::replace`]: Command::replace()
    #[inline]
    #[cfg(feature = "unstable-replace")]
    #[must_use]
    pub fn replace(mut self, name: &'help str, target: &'help [&'help str]) -> Self {
        self.replacers.insert(name, target);
        self
    }

    /// Exit gracefully if no arguments are present (e.g. `$ myprog`).
    ///
    /// **NOTE:** [`subcommands`] count as arguments
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command};
    /// Command::new("myprog")
    ///     .arg_required_else_help(true);
    /// ```
    ///
    /// [`subcommands`]: crate::Command::subcommand()
    /// [`Arg::default_value`]: crate::Arg::default_value()
    #[inline]
    pub fn arg_required_else_help(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::ArgRequiredElseHelp)
        } else {
            self.unset_setting(AppSettings::ArgRequiredElseHelp)
        }
    }

    /// Specifies that leading hyphens are allowed in all argument *values* (e.g. `-10`).
    ///
    /// Otherwise they will be parsed as another flag or option.  See also
    /// [`Command::allow_negative_numbers`].
    ///
    /// **NOTE:** Use this setting with caution as it silences certain circumstances which would
    /// otherwise be an error (such as accidentally forgetting to specify a value for leading
    /// option). It is preferred to set this on a per argument basis, via [`Arg::allow_hyphen_values`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Arg, Command};
    /// // Imagine you needed to represent negative numbers as well, such as -10
    /// let m = Command::new("nums")
    ///     .allow_hyphen_values(true)
    ///     .arg(Arg::new("neg"))
    ///     .get_matches_from(vec![
    ///         "nums", "-20"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("neg"), Some("-20"));
    /// # ;
    /// ```
    /// [`Arg::allow_hyphen_values`]: crate::Arg::allow_hyphen_values()
    #[inline]
    pub fn allow_hyphen_values(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::AllowHyphenValues)
        } else {
            self.unset_setting(AppSettings::AllowHyphenValues)
        }
    }

    /// Allows negative numbers to pass as values.
    ///
    /// This is similar to [`Command::allow_hyphen_values`] except that it only allows numbers,
    /// all other undefined leading hyphens will fail to parse.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let res = Command::new("myprog")
    ///     .allow_negative_numbers(true)
    ///     .arg(Arg::new("num"))
    ///     .try_get_matches_from(vec![
    ///         "myprog", "-20"
    ///     ]);
    /// assert!(res.is_ok());
    /// let m = res.unwrap();
    /// assert_eq!(m.value_of("num").unwrap(), "-20");
    /// ```
    #[inline]
    pub fn allow_negative_numbers(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::AllowNegativeNumbers)
        } else {
            self.unset_setting(AppSettings::AllowNegativeNumbers)
        }
    }

    /// Specifies that the final positional argument is a "VarArg" and that `clap` should not
    /// attempt to parse any further args.
    ///
    /// The values of the trailing positional argument will contain all args from itself on.
    ///
    /// **NOTE:** The final positional argument **must** have [`Arg::multiple_values(true)`] or the usage
    /// string equivalent.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, arg};
    /// let m = Command::new("myprog")
    ///     .trailing_var_arg(true)
    ///     .arg(arg!(<cmd> ... "commands to run"))
    ///     .get_matches_from(vec!["myprog", "arg1", "-r", "val1"]);
    ///
    /// let trail: Vec<&str> = m.values_of("cmd").unwrap().collect();
    /// assert_eq!(trail, ["arg1", "-r", "val1"]);
    /// ```
    /// [`Arg::multiple_values(true)`]: crate::Arg::multiple_values()
    pub fn trailing_var_arg(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::TrailingVarArg)
        } else {
            self.unset_setting(AppSettings::TrailingVarArg)
        }
    }

    /// Allows one to implement two styles of CLIs where positionals can be used out of order.
    ///
    /// The first example is a CLI where the second to last positional argument is optional, but
    /// the final positional argument is required. Such as `$ prog [optional] <required>` where one
    /// of the two following usages is allowed:
    ///
    /// * `$ prog [optional] <required>`
    /// * `$ prog <required>`
    ///
    /// This would otherwise not be allowed. This is useful when `[optional]` has a default value.
    ///
    /// **Note:** when using this style of "missing positionals" the final positional *must* be
    /// [required] if `--` will not be used to skip to the final positional argument.
    ///
    /// **Note:** This style also only allows a single positional argument to be "skipped" without
    /// the use of `--`. To skip more than one, see the second example.
    ///
    /// The second example is when one wants to skip multiple optional positional arguments, and use
    /// of the `--` operator is OK (but not required if all arguments will be specified anyways).
    ///
    /// For example, imagine a CLI which has three positional arguments `[foo] [bar] [baz]...` where
    /// `baz` accepts multiple values (similar to man `ARGS...` style training arguments).
    ///
    /// With this setting the following invocations are posisble:
    ///
    /// * `$ prog foo bar baz1 baz2 baz3`
    /// * `$ prog foo -- baz1 baz2 baz3`
    /// * `$ prog -- baz1 baz2 baz3`
    ///
    /// # Examples
    ///
    /// Style number one from above:
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// // Assume there is an external subcommand named "subcmd"
    /// let m = Command::new("myprog")
    ///     .allow_missing_positional(true)
    ///     .arg(Arg::new("arg1"))
    ///     .arg(Arg::new("arg2")
    ///         .required(true))
    ///     .get_matches_from(vec![
    ///         "prog", "other"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("arg1"), None);
    /// assert_eq!(m.value_of("arg2"), Some("other"));
    /// ```
    ///
    /// Now the same example, but using a default value for the first optional positional argument
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// // Assume there is an external subcommand named "subcmd"
    /// let m = Command::new("myprog")
    ///     .allow_missing_positional(true)
    ///     .arg(Arg::new("arg1")
    ///         .default_value("something"))
    ///     .arg(Arg::new("arg2")
    ///         .required(true))
    ///     .get_matches_from(vec![
    ///         "prog", "other"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("arg1"), Some("something"));
    /// assert_eq!(m.value_of("arg2"), Some("other"));
    /// ```
    ///
    /// Style number two from above:
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// // Assume there is an external subcommand named "subcmd"
    /// let m = Command::new("myprog")
    ///     .allow_missing_positional(true)
    ///     .arg(Arg::new("foo"))
    ///     .arg(Arg::new("bar"))
    ///     .arg(Arg::new("baz").takes_value(true).multiple_values(true))
    ///     .get_matches_from(vec![
    ///         "prog", "foo", "bar", "baz1", "baz2", "baz3"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("foo"), Some("foo"));
    /// assert_eq!(m.value_of("bar"), Some("bar"));
    /// assert_eq!(m.values_of("baz").unwrap().collect::<Vec<_>>(), &["baz1", "baz2", "baz3"]);
    /// ```
    ///
    /// Now nofice if we don't specify `foo` or `baz` but use the `--` operator.
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// // Assume there is an external subcommand named "subcmd"
    /// let m = Command::new("myprog")
    ///     .allow_missing_positional(true)
    ///     .arg(Arg::new("foo"))
    ///     .arg(Arg::new("bar"))
    ///     .arg(Arg::new("baz").takes_value(true).multiple_values(true))
    ///     .get_matches_from(vec![
    ///         "prog", "--", "baz1", "baz2", "baz3"
    ///     ]);
    ///
    /// assert_eq!(m.value_of("foo"), None);
    /// assert_eq!(m.value_of("bar"), None);
    /// assert_eq!(m.values_of("baz").unwrap().collect::<Vec<_>>(), &["baz1", "baz2", "baz3"]);
    /// ```
    ///
    /// [required]: crate::Arg::required()
    #[inline]
    pub fn allow_missing_positional(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::AllowMissingPositional)
        } else {
            self.unset_setting(AppSettings::AllowMissingPositional)
        }
    }
}

/// # Subcommand-specific Settings
impl<'help> App<'help> {
    /// Sets the short version of the subcommand flag without the preceding `-`.
    ///
    /// Allows the subcommand to be used as if it were an [`Arg::short`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use clap::{Command, Arg, ArgAction};
    /// let matches = Command::new("pacman")
    ///     .subcommand(
    ///         Command::new("sync").short_flag('S').arg(
    ///             Arg::new("search")
    ///                 .short('s')
    ///                 .long("search")
    ///                 .action(ArgAction::SetTrue)
    ///                 .help("search remote repositories for matching strings"),
    ///         ),
    ///     )
    ///     .get_matches_from(vec!["pacman", "-Ss"]);
    ///
    /// assert_eq!(matches.subcommand_name().unwrap(), "sync");
    /// let sync_matches = matches.subcommand_matches("sync").unwrap();
    /// assert!(*sync_matches.get_one::<bool>("search").expect("defaulted by clap"));
    /// ```
    /// [`Arg::short`]: Arg::short()
    #[must_use]
    pub fn short_flag(mut self, short: char) -> Self {
        self.short_flag = Some(short);
        self
    }

    /// Sets the long version of the subcommand flag without the preceding `--`.
    ///
    /// Allows the subcommand to be used as if it were an [`Arg::long`].
    ///
    /// **NOTE:** Any leading `-` characters will be stripped.
    ///
    /// # Examples
    ///
    /// To set `long_flag` use a word containing valid UTF-8 codepoints. If you supply a double leading
    /// `--` such as `--sync` they will be stripped. Hyphens in the middle of the word; however,
    /// will *not* be stripped (i.e. `sync-file` is allowed).
    ///
    /// ```
    /// # use clap::{Command, Arg, ArgAction};
    /// let matches = Command::new("pacman")
    ///     .subcommand(
    ///         Command::new("sync").long_flag("sync").arg(
    ///             Arg::new("search")
    ///                 .short('s')
    ///                 .long("search")
    ///                 .action(ArgAction::SetTrue)
    ///                 .help("search remote repositories for matching strings"),
    ///         ),
    ///     )
    ///     .get_matches_from(vec!["pacman", "--sync", "--search"]);
    ///
    /// assert_eq!(matches.subcommand_name().unwrap(), "sync");
    /// let sync_matches = matches.subcommand_matches("sync").unwrap();
    /// assert!(*sync_matches.get_one::<bool>("search").expect("defaulted by clap"));
    /// ```
    ///
    /// [`Arg::long`]: Arg::long()
    #[must_use]
    pub fn long_flag(mut self, long: &'help str) -> Self {
        #[cfg(feature = "unstable-v4")]
        {
            self.long_flag = Some(long);
        }
        #[cfg(not(feature = "unstable-v4"))]
        {
            self.long_flag = Some(long.trim_start_matches(|c| c == '-'));
        }
        self
    }

    /// Sets a hidden alias to this subcommand.
    ///
    /// This allows the subcommand to be accessed via *either* the original name, or this given
    /// alias. This is more efficient and easier than creating multiple hidden subcommands as one
    /// only needs to check for the existence of this command, and not all aliased variants.
    ///
    /// **NOTE:** Aliases defined with this method are *hidden* from the help
    /// message. If you're looking for aliases that will be displayed in the help
    /// message, see [`Command::visible_alias`].
    ///
    /// **NOTE:** When using aliases and checking for the existence of a
    /// particular subcommand within an [`ArgMatches`] struct, one only needs to
    /// search for the original name and not all aliases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///     .subcommand(Command::new("test")
    ///         .alias("do-stuff"))
    ///     .get_matches_from(vec!["myprog", "do-stuff"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::visible_alias`]: Command::visible_alias()
    #[must_use]
    pub fn alias<S: Into<&'help str>>(mut self, name: S) -> Self {
        self.aliases.push((name.into(), false));
        self
    }

    /// Add an alias, which functions as  "hidden" short flag subcommand
    ///
    /// This will automatically dispatch as if this subcommand was used. This is more efficient,
    /// and easier than creating multiple hidden subcommands as one only needs to check for the
    /// existence of this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").short_flag('t')
    ///                 .short_flag_alias('d'))
    ///             .get_matches_from(vec!["myprog", "-d"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    #[must_use]
    pub fn short_flag_alias(mut self, name: char) -> Self {
        assert!(name != '-', "short alias name cannot be `-`");
        self.short_flag_aliases.push((name, false));
        self
    }

    /// Add an alias, which functions as a "hidden" long flag subcommand.
    ///
    /// This will automatically dispatch as if this subcommand was used. This is more efficient,
    /// and easier than creating multiple hidden subcommands as one only needs to check for the
    /// existence of this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").long_flag("test")
    ///                 .long_flag_alias("testing"))
    ///             .get_matches_from(vec!["myprog", "--testing"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    #[must_use]
    pub fn long_flag_alias(mut self, name: &'help str) -> Self {
        self.long_flag_aliases.push((name, false));
        self
    }

    /// Sets multiple hidden aliases to this subcommand.
    ///
    /// This allows the subcommand to be accessed via *either* the original name or any of the
    /// given aliases. This is more efficient, and easier than creating multiple hidden subcommands
    /// as one only needs to check for the existence of this command and not all aliased variants.
    ///
    /// **NOTE:** Aliases defined with this method are *hidden* from the help
    /// message. If looking for aliases that will be displayed in the help
    /// message, see [`Command::visible_aliases`].
    ///
    /// **NOTE:** When using aliases and checking for the existence of a
    /// particular subcommand within an [`ArgMatches`] struct, one only needs to
    /// search for the original name and not all aliases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myprog")
    ///     .subcommand(Command::new("test")
    ///         .aliases(&["do-stuff", "do-tests", "tests"]))
    ///         .arg(Arg::new("input")
    ///             .help("the file to add")
    ///             .required(false))
    ///     .get_matches_from(vec!["myprog", "do-tests"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::visible_aliases`]: Command::visible_aliases()
    #[must_use]
    pub fn aliases(mut self, names: &[&'help str]) -> Self {
        self.aliases.extend(names.iter().map(|n| (*n, false)));
        self
    }

    /// Add aliases, which function as "hidden" short flag subcommands.
    ///
    /// These will automatically dispatch as if this subcommand was used. This is more efficient,
    /// and easier than creating multiple hidden subcommands as one only needs to check for the
    /// existence of this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///     .subcommand(Command::new("test").short_flag('t')
    ///         .short_flag_aliases(&['a', 'b', 'c']))
    ///         .arg(Arg::new("input")
    ///             .help("the file to add")
    ///             .required(false))
    ///     .get_matches_from(vec!["myprog", "-a"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    #[must_use]
    pub fn short_flag_aliases(mut self, names: &[char]) -> Self {
        for s in names {
            assert!(s != &'-', "short alias name cannot be `-`");
            self.short_flag_aliases.push((*s, false));
        }
        self
    }

    /// Add aliases, which function as "hidden" long flag subcommands.
    ///
    /// These will automatically dispatch as if this subcommand was used. This is more efficient,
    /// and easier than creating multiple hidden subcommands as one only needs to check for the
    /// existence of this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").long_flag("test")
    ///                 .long_flag_aliases(&["testing", "testall", "test_all"]))
    ///                 .arg(Arg::new("input")
    ///                             .help("the file to add")
    ///                             .required(false))
    ///             .get_matches_from(vec!["myprog", "--testing"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    #[must_use]
    pub fn long_flag_aliases(mut self, names: &[&'help str]) -> Self {
        for s in names {
            self.long_flag_aliases.push((s, false));
        }
        self
    }

    /// Sets a visible alias to this subcommand.
    ///
    /// This allows the subcommand to be accessed via *either* the
    /// original name or the given alias. This is more efficient and easier
    /// than creating hidden subcommands as one only needs to check for
    /// the existence of this command and not all aliased variants.
    ///
    /// **NOTE:** The alias defined with this method is *visible* from the help
    /// message and displayed as if it were just another regular subcommand. If
    /// looking for an alias that will not be displayed in the help message, see
    /// [`Command::alias`].
    ///
    /// **NOTE:** When using aliases and checking for the existence of a
    /// particular subcommand within an [`ArgMatches`] struct, one only needs to
    /// search for the original name and not all aliases.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// let m = Command::new("myprog")
    ///     .subcommand(Command::new("test")
    ///         .visible_alias("do-stuff"))
    ///     .get_matches_from(vec!["myprog", "do-stuff"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::alias`]: Command::alias()
    #[must_use]
    pub fn visible_alias<S: Into<&'help str>>(mut self, name: S) -> Self {
        self.aliases.push((name.into(), true));
        self
    }

    /// Add an alias, which functions as  "visible" short flag subcommand
    ///
    /// This will automatically dispatch as if this subcommand was used. This is more efficient,
    /// and easier than creating multiple hidden subcommands as one only needs to check for the
    /// existence of this command, and not all variants.
    ///
    /// See also [`Command::short_flag_alias`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").short_flag('t')
    ///                 .visible_short_flag_alias('d'))
    ///             .get_matches_from(vec!["myprog", "-d"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::short_flag_alias`]: Command::short_flag_alias()
    #[must_use]
    pub fn visible_short_flag_alias(mut self, name: char) -> Self {
        assert!(name != '-', "short alias name cannot be `-`");
        self.short_flag_aliases.push((name, true));
        self
    }

    /// Add an alias, which functions as a "visible" long flag subcommand.
    ///
    /// This will automatically dispatch as if this subcommand was used. This is more efficient,
    /// and easier than creating multiple hidden subcommands as one only needs to check for the
    /// existence of this command, and not all variants.
    ///
    /// See also [`Command::long_flag_alias`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").long_flag("test")
    ///                 .visible_long_flag_alias("testing"))
    ///             .get_matches_from(vec!["myprog", "--testing"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::long_flag_alias`]: Command::long_flag_alias()
    #[must_use]
    pub fn visible_long_flag_alias(mut self, name: &'help str) -> Self {
        self.long_flag_aliases.push((name, true));
        self
    }

    /// Sets multiple visible aliases to this subcommand.
    ///
    /// This allows the subcommand to be accessed via *either* the
    /// original name or any of the given aliases. This is more efficient and easier
    /// than creating multiple hidden subcommands as one only needs to check for
    /// the existence of this command and not all aliased variants.
    ///
    /// **NOTE:** The alias defined with this method is *visible* from the help
    /// message and displayed as if it were just another regular subcommand. If
    /// looking for an alias that will not be displayed in the help message, see
    /// [`Command::alias`].
    ///
    /// **NOTE:** When using aliases, and checking for the existence of a
    /// particular subcommand within an [`ArgMatches`] struct, one only needs to
    /// search for the original name and not all aliases.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///     .subcommand(Command::new("test")
    ///         .visible_aliases(&["do-stuff", "tests"]))
    ///     .get_matches_from(vec!["myprog", "do-stuff"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::alias`]: Command::alias()
    #[must_use]
    pub fn visible_aliases(mut self, names: &[&'help str]) -> Self {
        self.aliases.extend(names.iter().map(|n| (*n, true)));
        self
    }

    /// Add aliases, which function as *visible* short flag subcommands.
    ///
    /// See [`Command::short_flag_aliases`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").short_flag('b')
    ///                 .visible_short_flag_aliases(&['t']))
    ///             .get_matches_from(vec!["myprog", "-t"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::short_flag_aliases`]: Command::short_flag_aliases()
    #[must_use]
    pub fn visible_short_flag_aliases(mut self, names: &[char]) -> Self {
        for s in names {
            assert!(s != &'-', "short alias name cannot be `-`");
            self.short_flag_aliases.push((*s, true));
        }
        self
    }

    /// Add aliases, which function as *visible* long flag subcommands.
    ///
    /// See [`Command::long_flag_aliases`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg, };
    /// let m = Command::new("myprog")
    ///             .subcommand(Command::new("test").long_flag("test")
    ///                 .visible_long_flag_aliases(&["testing", "testall", "test_all"]))
    ///             .get_matches_from(vec!["myprog", "--testing"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`App::long_flag_aliases`]: Command::long_flag_aliases()
    #[must_use]
    pub fn visible_long_flag_aliases(mut self, names: &[&'help str]) -> Self {
        for s in names {
            self.long_flag_aliases.push((s, true));
        }
        self
    }

    /// Set the placement of this subcommand within the help.
    ///
    /// Subcommands with a lower value will be displayed first in the help message.  Subcommands
    /// with duplicate display orders will be displayed in alphabetical order.
    ///
    /// This is helpful when one would like to emphasize frequently used subcommands, or prioritize
    /// those towards the top of the list.
    ///
    /// **NOTE:** The default is 999 for all subcommands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, };
    /// let m = Command::new("cust-ord")
    ///     .subcommand(Command::new("alpha") // typically subcommands are grouped
    ///                                                // alphabetically by name. Subcommands
    ///                                                // without a display_order have a value of
    ///                                                // 999 and are displayed alphabetically with
    ///                                                // all other 999 subcommands
    ///         .about("Some help and text"))
    ///     .subcommand(Command::new("beta")
    ///         .display_order(1)   // In order to force this subcommand to appear *first*
    ///                             // all we have to do is give it a value lower than 999.
    ///                             // Any other subcommands with a value of 1 will be displayed
    ///                             // alphabetically with this one...then 2 values, then 3, etc.
    ///         .about("I should be first!"))
    ///     .get_matches_from(vec![
    ///         "cust-ord", "--help"
    ///     ]);
    /// ```
    ///
    /// The above example displays the following help message
    ///
    /// ```text
    /// cust-ord
    ///
    /// USAGE:
    ///     cust-ord [OPTIONS]
    ///
    /// OPTIONS:
    ///     -h, --help       Print help information
    ///     -V, --version    Print version information
    ///
    /// SUBCOMMANDS:
    ///     beta    I should be first!
    ///     alpha   Some help and text
    /// ```
    #[inline]
    #[must_use]
    pub fn display_order(mut self, ord: usize) -> Self {
        self.disp_ord = Some(ord);
        self
    }

    /// Specifies that this [`subcommand`] should be hidden from help messages
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .subcommand(
    ///         Command::new("test").hide(true)
    ///     )
    /// # ;
    /// ```
    ///
    /// [`subcommand`]: crate::Command::subcommand()
    #[inline]
    pub fn hide(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::Hidden)
        } else {
            self.unset_setting(AppSettings::Hidden)
        }
    }

    /// If no [`subcommand`] is present at runtime, error and exit gracefully.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, ErrorKind};
    /// let err = Command::new("myprog")
    ///     .subcommand_required(true)
    ///     .subcommand(Command::new("test"))
    ///     .try_get_matches_from(vec![
    ///         "myprog",
    ///     ]);
    /// assert!(err.is_err());
    /// assert_eq!(err.unwrap_err().kind(), ErrorKind::MissingSubcommand);
    /// # ;
    /// ```
    ///
    /// [`subcommand`]: crate::Command::subcommand()
    pub fn subcommand_required(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::SubcommandRequired)
        } else {
            self.unset_setting(AppSettings::SubcommandRequired)
        }
    }

    /// Assume unexpected positional arguments are a [`subcommand`].
    ///
    /// Arguments will be stored in the `""` argument in the [`ArgMatches`]
    ///
    /// **NOTE:** Use this setting with caution,
    /// as a truly unexpected argument (i.e. one that is *NOT* an external subcommand)
    /// will **not** cause an error and instead be treated as a potential subcommand.
    /// One should check for such cases manually and inform the user appropriately.
    ///
    /// **NOTE:** A built-in subcommand will be parsed as an external subcommand when escaped with
    /// `--`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// // Assume there is an external subcommand named "subcmd"
    /// let m = Command::new("myprog")
    ///     .allow_external_subcommands(true)
    ///     .get_matches_from(vec![
    ///         "myprog", "subcmd", "--option", "value", "-fff", "--flag"
    ///     ]);
    ///
    /// // All trailing arguments will be stored under the subcommand's sub-matches using an empty
    /// // string argument name
    /// match m.subcommand() {
    ///     Some((external, ext_m)) => {
    ///          let ext_args: Vec<&str> = ext_m.values_of("").unwrap().collect();
    ///          assert_eq!(external, "subcmd");
    ///          assert_eq!(ext_args, ["--option", "value", "-fff", "--flag"]);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// [`subcommand`]: crate::Command::subcommand()
    /// [`ArgMatches`]: crate::ArgMatches
    /// [`ErrorKind::UnknownArgument`]: crate::ErrorKind::UnknownArgument
    pub fn allow_external_subcommands(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::AllowExternalSubcommands)
        } else {
            self.unset_setting(AppSettings::AllowExternalSubcommands)
        }
    }

    /// Specifies that external subcommands that are invalid UTF-8 should *not* be treated as an error.
    ///
    /// **NOTE:** Using external subcommand argument values with invalid UTF-8 requires using
    /// [`ArgMatches::values_of_os`] or [`ArgMatches::values_of_lossy`] for those particular
    /// arguments which may contain invalid UTF-8 values
    ///
    /// **NOTE:** Setting this requires [`Command::allow_external_subcommands`]
    ///
    /// # Platform Specific
    ///
    /// Non Windows systems only
    ///
    /// # Examples
    ///
    #[cfg_attr(not(unix), doc = " ```ignore")]
    #[cfg_attr(unix, doc = " ```")]
    /// # use clap::Command;
    /// // Assume there is an external subcommand named "subcmd"
    /// let m = Command::new("myprog")
    ///     .allow_invalid_utf8_for_external_subcommands(true)
    ///     .allow_external_subcommands(true)
    ///     .get_matches_from(vec![
    ///         "myprog", "subcmd", "--option", "value", "-fff", "--flag"
    ///     ]);
    ///
    /// // All trailing arguments will be stored under the subcommand's sub-matches using an empty
    /// // string argument name
    /// match m.subcommand() {
    ///     Some((external, ext_m)) => {
    ///          let ext_args: Vec<&std::ffi::OsStr> = ext_m.values_of_os("").unwrap().collect();
    ///          assert_eq!(external, "subcmd");
    ///          assert_eq!(ext_args, ["--option", "value", "-fff", "--flag"]);
    ///     },
    ///     _ => {},
    /// }
    /// ```
    ///
    /// [`ArgMatches::values_of_os`]: crate::ArgMatches::values_of_os()
    /// [`ArgMatches::values_of_lossy`]: crate::ArgMatches::values_of_lossy()
    /// [`subcommands`]: crate::Command::subcommand()
    pub fn allow_invalid_utf8_for_external_subcommands(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
        } else {
            self.unset_setting(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
        }
    }

    /// Specifies that use of an argument prevents the use of [`subcommands`].
    ///
    /// By default `clap` allows arguments between subcommands such
    /// as `<cmd> [cmd_args] <subcmd> [subcmd_args] <subsubcmd> [subsubcmd_args]`.
    ///
    /// This setting disables that functionality and says that arguments can
    /// only follow the *final* subcommand. For instance using this setting
    /// makes only the following invocations possible:
    ///
    /// * `<cmd> <subcmd> <subsubcmd> [subsubcmd_args]`
    /// * `<cmd> <subcmd> [subcmd_args]`
    /// * `<cmd> [cmd_args]`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::Command;
    /// Command::new("myprog")
    ///     .args_conflicts_with_subcommands(true);
    /// ```
    ///
    /// [`subcommands`]: crate::Command::subcommand()
    pub fn args_conflicts_with_subcommands(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::ArgsNegateSubcommands)
        } else {
            self.unset_setting(AppSettings::ArgsNegateSubcommands)
        }
    }

    /// Prevent subcommands from being consumed as an arguments value.
    ///
    /// By default, if an option taking multiple values is followed by a subcommand, the
    /// subcommand will be parsed as another value.
    ///
    /// ```text
    /// cmd --foo val1 val2 subcommand
    ///           --------- ----------
    ///             values   another value
    /// ```
    ///
    /// This setting instructs the parser to stop when encountering a subcommand instead of
    /// greedily consuming arguments.
    ///
    /// ```text
    /// cmd --foo val1 val2 subcommand
    ///           --------- ----------
    ///             values   subcommand
    /// ```
    ///
    /// **Note:** Make sure you apply it as `global_setting` if you want this setting
    /// to be propagated to subcommands and sub-subcommands!
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{Command, Arg};
    /// let cmd = Command::new("cmd").subcommand(Command::new("sub")).arg(
    ///     Arg::new("arg")
    ///         .long("arg")
    ///         .multiple_values(true)
    ///         .takes_value(true),
    /// );
    ///
    /// let matches = cmd
    ///     .clone()
    ///     .try_get_matches_from(&["cmd", "--arg", "1", "2", "3", "sub"])
    ///     .unwrap();
    /// assert_eq!(
    ///     matches.values_of("arg").unwrap().collect::<Vec<_>>(),
    ///     &["1", "2", "3", "sub"]
    /// );
    /// assert!(matches.subcommand_matches("sub").is_none());
    ///
    /// let matches = cmd
    ///     .subcommand_precedence_over_arg(true)
    ///     .try_get_matches_from(&["cmd", "--arg", "1", "2", "3", "sub"])
    ///     .unwrap();
    /// assert_eq!(
    ///     matches.values_of("arg").unwrap().collect::<Vec<_>>(),
    ///     &["1", "2", "3"]
    /// );
    /// assert!(matches.subcommand_matches("sub").is_some());
    /// ```
    pub fn subcommand_precedence_over_arg(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::SubcommandPrecedenceOverArg)
        } else {
            self.unset_setting(AppSettings::SubcommandPrecedenceOverArg)
        }
    }

    /// Allows [`subcommands`] to override all requirements of the parent command.
    ///
    /// For example, if you had a subcommand or top level application with a required argument
    /// that is only required as long as there is no subcommand present,
    /// using this setting would allow you to set those arguments to [`Arg::required(true)`]
    /// and yet receive no error so long as the user uses a valid subcommand instead.
    ///
    /// **NOTE:** This defaults to false (using subcommand does *not* negate requirements)
    ///
    /// # Examples
    ///
    /// This first example shows that it is an error to not use a required argument
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ErrorKind};
    /// let err = Command::new("myprog")
    ///     .subcommand_negates_reqs(true)
    ///     .arg(Arg::new("opt").required(true))
    ///     .subcommand(Command::new("test"))
    ///     .try_get_matches_from(vec![
    ///         "myprog"
    ///     ]);
    /// assert!(err.is_err());
    /// assert_eq!(err.unwrap_err().kind(), ErrorKind::MissingRequiredArgument);
    /// # ;
    /// ```
    ///
    /// This next example shows that it is no longer error to not use a required argument if a
    /// valid subcommand is used.
    ///
    /// ```rust
    /// # use clap::{Command, Arg, ErrorKind};
    /// let noerr = Command::new("myprog")
    ///     .subcommand_negates_reqs(true)
    ///     .arg(Arg::new("opt").required(true))
    ///     .subcommand(Command::new("test"))
    ///     .try_get_matches_from(vec![
    ///         "myprog", "test"
    ///     ]);
    /// assert!(noerr.is_ok());
    /// # ;
    /// ```
    ///
    /// [`Arg::required(true)`]: crate::Arg::required()
    /// [`subcommands`]: crate::Command::subcommand()
    pub fn subcommand_negates_reqs(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::SubcommandsNegateReqs)
        } else {
            self.unset_setting(AppSettings::SubcommandsNegateReqs)
        }
    }

    /// Multiple-personality program dispatched on the binary name (`argv[0]`)
    ///
    /// A "multicall" executable is a single executable
    /// that contains a variety of applets,
    /// and decides which applet to run based on the name of the file.
    /// The executable can be called from different names by creating hard links
    /// or symbolic links to it.
    ///
    /// This is desirable for:
    /// - Easy distribution, a single binary that can install hardlinks to access the different
    ///   personalities.
    /// - Minimal binary size by sharing common code (e.g. standard library, clap)
    /// - Custom shells or REPLs where there isn't a single top-level command
    ///
    /// Setting `multicall` will cause
    /// - `argv[0]` to be stripped to the base name and parsed as the first argument, as if
    ///   [`Command::no_binary_name`][App::no_binary_name] was set.
    /// - Help and errors to report subcommands as if they were the top-level command
    ///
    /// When the subcommand is not present, there are several strategies you may employ, depending
    /// on your needs:
    /// - Let the error percolate up normally
    /// - Print a specialized error message using the
    ///   [`Error::context`][crate::Error::context]
    /// - Print the [help][App::write_help] but this might be ambiguous
    /// - Disable `multicall` and re-parse it
    /// - Disable `multicall` and re-parse it with a specific subcommand
    ///
    /// When detecting the error condition, the [`ErrorKind`] isn't sufficient as a sub-subcommand
    /// might report the same error.  Enable
    /// [`allow_external_subcommands`][App::allow_external_subcommands] if you want to specifically
    /// get the unrecognized binary name.
    ///
    /// **NOTE:** Multicall can't be used with [`no_binary_name`] since they interpret
    /// the command name in incompatible ways.
    ///
    /// **NOTE:** The multicall command cannot have arguments.
    ///
    /// **NOTE:** Applets are slightly semantically different from subcommands,
    /// so it's recommended to use [`Command::subcommand_help_heading`] and
    /// [`Command::subcommand_value_name`] to change the descriptive text as above.
    ///
    /// # Examples
    ///
    /// `hostname` is an example of a multicall executable.
    /// Both `hostname` and `dnsdomainname` are provided by the same executable
    /// and which behaviour to use is based on the executable file name.
    ///
    /// This is desirable when the executable has a primary purpose
    /// but there is related functionality that would be convenient to provide
    /// and implement it to be in the same executable.
    ///
    /// The name of the cmd is essentially unused
    /// and may be the same as the name of a subcommand.
    ///
    /// The names of the immediate subcommands of the Command
    /// are matched against the basename of the first argument,
    /// which is conventionally the path of the executable.
    ///
    /// This does not allow the subcommand to be passed as the first non-path argument.
    ///
    /// ```rust
    /// # use clap::{Command, ErrorKind};
    /// let mut cmd = Command::new("hostname")
    ///     .multicall(true)
    ///     .subcommand(Command::new("hostname"))
    ///     .subcommand(Command::new("dnsdomainname"));
    /// let m = cmd.try_get_matches_from_mut(&["/usr/bin/hostname", "dnsdomainname"]);
    /// assert!(m.is_err());
    /// assert_eq!(m.unwrap_err().kind(), ErrorKind::UnknownArgument);
    /// let m = cmd.get_matches_from(&["/usr/bin/dnsdomainname"]);
    /// assert_eq!(m.subcommand_name(), Some("dnsdomainname"));
    /// ```
    ///
    /// Busybox is another common example of a multicall executable
    /// with a subcommmand for each applet that can be run directly,
    /// e.g. with the `cat` applet being run by running `busybox cat`,
    /// or with `cat` as a link to the `busybox` binary.
    ///
    /// This is desirable when the launcher program has additional options
    /// or it is useful to run the applet without installing a symlink
    /// e.g. to test the applet without installing it
    /// or there may already be a command of that name installed.
    ///
    /// To make an applet usable as both a multicall link and a subcommand
    /// the subcommands must be defined both in the top-level Command
    /// and as subcommands of the "main" applet.
    ///
    /// ```rust
    /// # use clap::Command;
    /// fn applet_commands() -> [Command<'static>; 2] {
    ///     [Command::new("true"), Command::new("false")]
    /// }
    /// let mut cmd = Command::new("busybox")
    ///     .multicall(true)
    ///     .subcommand(
    ///         Command::new("busybox")
    ///             .subcommand_value_name("APPLET")
    ///             .subcommand_help_heading("APPLETS")
    ///             .subcommands(applet_commands()),
    ///     )
    ///     .subcommands(applet_commands());
    /// // When called from the executable's canonical name
    /// // its applets can be matched as subcommands.
    /// let m = cmd.try_get_matches_from_mut(&["/usr/bin/busybox", "true"]).unwrap();
    /// assert_eq!(m.subcommand_name(), Some("busybox"));
    /// assert_eq!(m.subcommand().unwrap().1.subcommand_name(), Some("true"));
    /// // When called from a link named after an applet that applet is matched.
    /// let m = cmd.get_matches_from(&["/usr/bin/true"]);
    /// assert_eq!(m.subcommand_name(), Some("true"));
    /// ```
    ///
    /// [`no_binary_name`]: crate::Command::no_binary_name
    /// [`App::subcommand_value_name`]: crate::Command::subcommand_value_name
    /// [`App::subcommand_help_heading`]: crate::Command::subcommand_help_heading
    #[inline]
    pub fn multicall(self, yes: bool) -> Self {
        if yes {
            self.setting(AppSettings::Multicall)
        } else {
            self.unset_setting(AppSettings::Multicall)
        }
    }

    /// Sets the value name used for subcommands when printing usage and help.
    ///
    /// By default, this is "SUBCOMMAND".
    ///
    /// See also [`Command::subcommand_help_heading`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .subcommand(Command::new("sub1"))
    ///     .print_help()
    /// # ;
    /// ```
    ///
    /// will produce
    ///
    /// ```text
    /// myprog
    ///
    /// USAGE:
    ///     myprog [SUBCOMMAND]
    ///
    /// OPTIONS:
    ///     -h, --help       Print help information
    ///     -V, --version    Print version information
    ///
    /// SUBCOMMANDS:
    ///     help    Print this message or the help of the given subcommand(s)
    ///     sub1
    /// ```
    ///
    /// but usage of `subcommand_value_name`
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .subcommand(Command::new("sub1"))
    ///     .subcommand_value_name("THING")
    ///     .print_help()
    /// # ;
    /// ```
    ///
    /// will produce
    ///
    /// ```text
    /// myprog
    ///
    /// USAGE:
    ///     myprog [THING]
    ///
    /// OPTIONS:
    ///     -h, --help       Print help information
    ///     -V, --version    Print version information
    ///
    /// SUBCOMMANDS:
    ///     help    Print this message or the help of the given subcommand(s)
    ///     sub1
    /// ```
    #[must_use]
    pub fn subcommand_value_name<S>(mut self, value_name: S) -> Self
    where
        S: Into<&'help str>,
    {
        self.subcommand_value_name = Some(value_name.into());
        self
    }

    /// Sets the help heading used for subcommands when printing usage and help.
    ///
    /// By default, this is "SUBCOMMANDS".
    ///
    /// See also [`Command::subcommand_value_name`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .subcommand(Command::new("sub1"))
    ///     .print_help()
    /// # ;
    /// ```
    ///
    /// will produce
    ///
    /// ```text
    /// myprog
    ///
    /// USAGE:
    ///     myprog [SUBCOMMAND]
    ///
    /// OPTIONS:
    ///     -h, --help       Print help information
    ///     -V, --version    Print version information
    ///
    /// SUBCOMMANDS:
    ///     help    Print this message or the help of the given subcommand(s)
    ///     sub1
    /// ```
    ///
    /// but usage of `subcommand_help_heading`
    ///
    /// ```no_run
    /// # use clap::{Command, Arg};
    /// Command::new("myprog")
    ///     .subcommand(Command::new("sub1"))
    ///     .subcommand_help_heading("THINGS")
    ///     .print_help()
    /// # ;
    /// ```
    ///
    /// will produce
    ///
    /// ```text
    /// myprog
    ///
    /// USAGE:
    ///     myprog [SUBCOMMAND]
    ///
    /// OPTIONS:
    ///     -h, --help       Print help information
    ///     -V, --version    Print version information
    ///
    /// THINGS:
    ///     help    Print this message or the help of the given subcommand(s)
    ///     sub1
    /// ```
    #[must_use]
    pub fn subcommand_help_heading<T>(mut self, heading: T) -> Self
    where
        T: Into<&'help str>,
    {
        self.subcommand_heading = Some(heading.into());
        self
    }
}

/// # Reflection
impl<'help> App<'help> {
    #[inline]
    pub(crate) fn get_usage_name(&self) -> Option<&str> {
        self.usage_name.as_deref()
    }

    /// Get the name of the binary.
    #[inline]
    pub fn get_display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// Get the name of the binary.
    #[inline]
    pub fn get_bin_name(&self) -> Option<&str> {
        self.bin_name.as_deref()
    }

    /// Set binary name. Uses `&mut self` instead of `self`.
    pub fn set_bin_name<S: Into<String>>(&mut self, name: S) {
        self.bin_name = Some(name.into());
    }

    /// Get the name of the cmd.
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the version of the cmd.
    #[inline]
    pub fn get_version(&self) -> Option<&'help str> {
        self.version
    }

    /// Get the long version of the cmd.
    #[inline]
    pub fn get_long_version(&self) -> Option<&'help str> {
        self.long_version
    }

    /// Get the authors of the cmd.
    #[inline]
    pub fn get_author(&self) -> Option<&'help str> {
        self.author
    }

    /// Get the short flag of the subcommand.
    #[inline]
    pub fn get_short_flag(&self) -> Option<char> {
        self.short_flag
    }

    /// Get the long flag of the subcommand.
    #[inline]
    pub fn get_long_flag(&self) -> Option<&'help str> {
        self.long_flag
    }

    /// Get the help message specified via [`Command::about`].
    ///
    /// [`App::about`]: Command::about()
    #[inline]
    pub fn get_about(&self) -> Option<&'help str> {
        self.about
    }

    /// Get the help message specified via [`Command::long_about`].
    ///
    /// [`App::long_about`]: Command::long_about()
    #[inline]
    pub fn get_long_about(&self) -> Option<&'help str> {
        self.long_about
    }

    /// Deprecated, replaced with [`Command::get_next_help_heading`]
    #[inline]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.0", note = "Replaced with `App::get_next_help_heading`")
    )]
    pub fn get_help_heading(&self) -> Option<&'help str> {
        self.get_next_help_heading()
    }

    /// Get the custom section heading specified via [`Command::help_heading`].
    ///
    /// [`App::help_heading`]: Command::help_heading()
    #[inline]
    pub fn get_next_help_heading(&self) -> Option<&'help str> {
        self.current_help_heading
    }

    /// Iterate through the *visible* aliases for this subcommand.
    #[inline]
    pub fn get_visible_aliases(&self) -> impl Iterator<Item = &'help str> + '_ {
        self.aliases.iter().filter(|(_, vis)| *vis).map(|a| a.0)
    }

    /// Iterate through the *visible* short aliases for this subcommand.
    #[inline]
    pub fn get_visible_short_flag_aliases(&self) -> impl Iterator<Item = char> + '_ {
        self.short_flag_aliases
            .iter()
            .filter(|(_, vis)| *vis)
            .map(|a| a.0)
    }

    /// Iterate through the *visible* long aliases for this subcommand.
    #[inline]
    pub fn get_visible_long_flag_aliases(&self) -> impl Iterator<Item = &'help str> + '_ {
        self.long_flag_aliases
            .iter()
            .filter(|(_, vis)| *vis)
            .map(|a| a.0)
    }

    /// Iterate through the set of *all* the aliases for this subcommand, both visible and hidden.
    #[inline]
    pub fn get_all_aliases(&self) -> impl Iterator<Item = &str> + '_ {
        self.aliases.iter().map(|a| a.0)
    }

    /// Iterate through the set of *all* the short aliases for this subcommand, both visible and hidden.
    #[inline]
    pub fn get_all_short_flag_aliases(&self) -> impl Iterator<Item = char> + '_ {
        self.short_flag_aliases.iter().map(|a| a.0)
    }

    /// Iterate through the set of *all* the long aliases for this subcommand, both visible and hidden.
    #[inline]
    pub fn get_all_long_flag_aliases(&self) -> impl Iterator<Item = &'help str> + '_ {
        self.long_flag_aliases.iter().map(|a| a.0)
    }

    /// Check if the given [`AppSettings`] variant is currently set on the `Command`.
    ///
    /// This checks both [local] and [global settings].
    ///
    /// [local]: Command::setting()
    /// [global settings]: Command::global_setting()
    #[inline]
    pub fn is_set(&self, s: AppSettings) -> bool {
        self.settings.is_set(s) || self.g_settings.is_set(s)
    }

    /// Should we color the output?
    #[inline(never)]
    pub fn get_color(&self) -> ColorChoice {
        debug!("Command::color: Color setting...");

        if cfg!(feature = "color") {
            #[allow(deprecated)]
            if self.is_set(AppSettings::ColorNever) {
                debug!("Never");
                ColorChoice::Never
            } else if self.is_set(AppSettings::ColorAlways) {
                debug!("Always");
                ColorChoice::Always
            } else {
                debug!("Auto");
                ColorChoice::Auto
            }
        } else {
            ColorChoice::Never
        }
    }

    /// Iterate through the set of subcommands, getting a reference to each.
    #[inline]
    pub fn get_subcommands(&self) -> impl Iterator<Item = &App<'help>> {
        self.subcommands.iter()
    }

    /// Iterate through the set of subcommands, getting a mutable reference to each.
    #[inline]
    pub fn get_subcommands_mut(&mut self) -> impl Iterator<Item = &mut App<'help>> {
        self.subcommands.iter_mut()
    }

    /// Returns `true` if this `Command` has subcommands.
    #[inline]
    pub fn has_subcommands(&self) -> bool {
        !self.subcommands.is_empty()
    }

    /// Returns the help heading for listing subcommands.
    #[inline]
    pub fn get_subcommand_help_heading(&self) -> Option<&str> {
        self.subcommand_heading
    }

    /// Deprecated, replaced with [`App::get_subcommand_help_heading`]
    #[inline]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.1.0",
            note = "Replaced with `App::get_subcommand_help_heading`"
        )
    )]
    pub fn get_subommand_help_heading(&self) -> Option<&str> {
        self.get_subcommand_help_heading()
    }

    /// Returns the subcommand value name.
    #[inline]
    pub fn get_subcommand_value_name(&self) -> Option<&str> {
        self.subcommand_value_name
    }

    /// Returns the help heading for listing subcommands.
    #[inline]
    pub fn get_before_help(&self) -> Option<&str> {
        self.before_help
    }

    /// Returns the help heading for listing subcommands.
    #[inline]
    pub fn get_before_long_help(&self) -> Option<&str> {
        self.before_long_help
    }

    /// Returns the help heading for listing subcommands.
    #[inline]
    pub fn get_after_help(&self) -> Option<&str> {
        self.after_help
    }

    /// Returns the help heading for listing subcommands.
    #[inline]
    pub fn get_after_long_help(&self) -> Option<&str> {
        self.after_long_help
    }

    /// Find subcommand such that its name or one of aliases equals `name`.
    ///
    /// This does not recurse through subcommands of subcommands.
    #[inline]
    pub fn find_subcommand<T>(&self, name: &T) -> Option<&App<'help>>
    where
        T: PartialEq<str> + ?Sized,
    {
        self.get_subcommands().find(|s| s.aliases_to(name))
    }

    /// Find subcommand such that its name or one of aliases equals `name`, returning
    /// a mutable reference to the subcommand.
    ///
    /// This does not recurse through subcommands of subcommands.
    #[inline]
    pub fn find_subcommand_mut<T>(&mut self, name: &T) -> Option<&mut App<'help>>
    where
        T: PartialEq<str> + ?Sized,
    {
        self.get_subcommands_mut().find(|s| s.aliases_to(name))
    }

    /// Iterate through the set of groups.
    #[inline]
    pub fn get_groups(&self) -> impl Iterator<Item = &ArgGroup<'help>> {
        self.groups.iter()
    }

    /// Iterate through the set of arguments.
    #[inline]
    pub fn get_arguments(&self) -> impl Iterator<Item = &Arg<'help>> {
        self.args.args()
    }

    /// Iterate through the *positionals* arguments.
    #[inline]
    pub fn get_positionals(&self) -> impl Iterator<Item = &Arg<'help>> {
        self.get_arguments().filter(|a| a.is_positional())
    }

    /// Iterate through the *options*.
    pub fn get_opts(&self) -> impl Iterator<Item = &Arg<'help>> {
        self.get_arguments()
            .filter(|a| a.is_takes_value_set() && !a.is_positional())
    }

    /// Get a list of all arguments the given argument conflicts with.
    ///
    /// If the provided argument is declared as global, the conflicts will be determined
    /// based on the propagation rules of global arguments.
    ///
    /// ### Panics
    ///
    /// If the given arg contains a conflict with an argument that is unknown to
    /// this `Command`.
    pub fn get_arg_conflicts_with(&self, arg: &Arg) -> Vec<&Arg<'help>> // FIXME: This could probably have been an iterator
    {
        if arg.is_global_set() {
            self.get_global_arg_conflicts_with(arg)
        } else {
            let mut result = Vec::new();
            for id in arg.blacklist.iter() {
                if let Some(arg) = self.find(id) {
                    result.push(arg);
                } else if let Some(group) = self.find_group(id) {
                    result.extend(
                        self.unroll_args_in_group(&group.id)
                            .iter()
                            .map(|id| self.find(id).expect(INTERNAL_ERROR_MSG)),
                    );
                } else {
                    panic!("Command::get_arg_conflicts_with: The passed arg conflicts with an arg unknown to the cmd");
                }
            }
            result
        }
    }

    // Get a unique list of all arguments of all commands and continuous subcommands the given argument conflicts with.
    //
    // This behavior follows the propagation rules of global arguments.
    // It is useful for finding conflicts for arguments declared as global.
    //
    // ### Panics
    //
    // If the given arg contains a conflict with an argument that is unknown to
    // this `App`.
    fn get_global_arg_conflicts_with(&self, arg: &Arg) -> Vec<&Arg<'help>> // FIXME: This could probably have been an iterator
    {
        arg.blacklist
            .iter()
            .map(|id| {
                self.args
                    .args()
                    .chain(
                        self.get_subcommands_containing(arg)
                            .iter()
                            .flat_map(|x| x.args.args()),
                    )
                    .find(|arg| arg.id == *id)
                    .expect(
                        "Command::get_arg_conflicts_with: \
                    The passed arg conflicts with an arg unknown to the cmd",
                    )
            })
            .collect()
    }

    // Get a list of subcommands which contain the provided Argument
    //
    // This command will only include subcommands in its list for which the subcommands
    // parent also contains the Argument.
    //
    // This search follows the propagation rules of global arguments.
    // It is useful to finding subcommands, that have inherited a global argument.
    //
    // **NOTE:** In this case only Sucommand_1 will be included
    //   Subcommand_1 (contains Arg)
    //     Subcommand_1.1 (doesn't contain Arg)
    //       Subcommand_1.1.1 (contains Arg)
    //
    fn get_subcommands_containing(&self, arg: &Arg) -> Vec<&App<'help>> {
        let mut vec = std::vec::Vec::new();
        for idx in 0..self.subcommands.len() {
            if self.subcommands[idx].args.args().any(|ar| ar.id == arg.id) {
                vec.push(&self.subcommands[idx]);
                vec.append(&mut self.subcommands[idx].get_subcommands_containing(arg));
            }
        }
        vec
    }

    /// Report whether [`Command::no_binary_name`] is set
    pub fn is_no_binary_name_set(&self) -> bool {
        self.is_set(AppSettings::NoBinaryName)
    }

    /// Report whether [`Command::ignore_errors`] is set
    pub(crate) fn is_ignore_errors_set(&self) -> bool {
        self.is_set(AppSettings::IgnoreErrors)
    }

    /// Report whether [`Command::dont_delimit_trailing_values`] is set
    pub fn is_dont_delimit_trailing_values_set(&self) -> bool {
        self.is_set(AppSettings::DontDelimitTrailingValues)
    }

    /// Report whether [`Command::disable_version_flag`] is set
    pub fn is_disable_version_flag_set(&self) -> bool {
        self.is_set(AppSettings::DisableVersionFlag)
    }

    /// Report whether [`Command::propagate_version`] is set
    pub fn is_propagate_version_set(&self) -> bool {
        self.is_set(AppSettings::PropagateVersion)
    }

    /// Report whether [`Command::next_line_help`] is set
    pub fn is_next_line_help_set(&self) -> bool {
        self.is_set(AppSettings::NextLineHelp)
    }

    /// Report whether [`Command::disable_help_flag`] is set
    pub fn is_disable_help_flag_set(&self) -> bool {
        self.is_set(AppSettings::DisableHelpFlag)
    }

    /// Report whether [`Command::disable_help_subcommand`] is set
    pub fn is_disable_help_subcommand_set(&self) -> bool {
        self.is_set(AppSettings::DisableHelpSubcommand)
    }

    /// Report whether [`Command::disable_colored_help`] is set
    pub fn is_disable_colored_help_set(&self) -> bool {
        self.is_set(AppSettings::DisableColoredHelp)
    }

    /// Report whether [`Command::help_expected`] is set
    #[cfg(debug_assertions)]
    pub(crate) fn is_help_expected_set(&self) -> bool {
        self.is_set(AppSettings::HelpExpected)
    }

    /// Report whether [`Command::dont_collapse_args_in_usage`] is set
    pub fn is_dont_collapse_args_in_usage_set(&self) -> bool {
        self.is_set(AppSettings::DontCollapseArgsInUsage)
    }

    /// Report whether [`Command::infer_long_args`] is set
    pub(crate) fn is_infer_long_args_set(&self) -> bool {
        self.is_set(AppSettings::InferLongArgs)
    }

    /// Report whether [`Command::infer_subcommands`] is set
    pub(crate) fn is_infer_subcommands_set(&self) -> bool {
        self.is_set(AppSettings::InferSubcommands)
    }

    /// Report whether [`Command::arg_required_else_help`] is set
    pub fn is_arg_required_else_help_set(&self) -> bool {
        self.is_set(AppSettings::ArgRequiredElseHelp)
    }

    /// Report whether [`Command::allow_hyphen_values`] is set
    pub(crate) fn is_allow_hyphen_values_set(&self) -> bool {
        self.is_set(AppSettings::AllowHyphenValues)
    }

    /// Report whether [`Command::allow_negative_numbers`] is set
    pub fn is_allow_negative_numbers_set(&self) -> bool {
        self.is_set(AppSettings::AllowNegativeNumbers)
    }

    /// Report whether [`Command::trailing_var_arg`] is set
    pub fn is_trailing_var_arg_set(&self) -> bool {
        self.is_set(AppSettings::TrailingVarArg)
    }

    /// Report whether [`Command::allow_missing_positional`] is set
    pub fn is_allow_missing_positional_set(&self) -> bool {
        self.is_set(AppSettings::AllowMissingPositional)
    }

    /// Report whether [`Command::hide`] is set
    pub fn is_hide_set(&self) -> bool {
        self.is_set(AppSettings::Hidden)
    }

    /// Report whether [`Command::subcommand_required`] is set
    pub fn is_subcommand_required_set(&self) -> bool {
        self.is_set(AppSettings::SubcommandRequired)
    }

    /// Report whether [`Command::allow_external_subcommands`] is set
    pub fn is_allow_external_subcommands_set(&self) -> bool {
        self.is_set(AppSettings::AllowExternalSubcommands)
    }

    /// Report whether [`Command::allow_invalid_utf8_for_external_subcommands`] is set
    pub fn is_allow_invalid_utf8_for_external_subcommands_set(&self) -> bool {
        self.is_set(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
    }

    /// Configured parser for values passed to an external subcommand
    ///
    /// # Example
    ///
    /// ```rust
    /// let cmd = clap::Command::new("raw")
    ///     .allow_external_subcommands(true)
    ///     .allow_invalid_utf8_for_external_subcommands(true);
    /// let value_parser = cmd.get_external_subcommand_value_parser();
    /// println!("{:?}", value_parser);
    /// ```
    pub fn get_external_subcommand_value_parser(&self) -> Option<&super::ValueParser> {
        if !self.is_allow_external_subcommands_set() {
            None
        } else if self.is_allow_invalid_utf8_for_external_subcommands_set() {
            static DEFAULT: super::ValueParser = super::ValueParser::os_string();
            Some(&DEFAULT)
        } else {
            static DEFAULT: super::ValueParser = super::ValueParser::string();
            Some(&DEFAULT)
        }
    }

    /// Report whether [`Command::args_conflicts_with_subcommands`] is set
    pub fn is_args_conflicts_with_subcommands_set(&self) -> bool {
        self.is_set(AppSettings::ArgsNegateSubcommands)
    }

    /// Report whether [`Command::subcommand_precedence_over_arg`] is set
    pub fn is_subcommand_precedence_over_arg_set(&self) -> bool {
        self.is_set(AppSettings::SubcommandPrecedenceOverArg)
    }

    /// Report whether [`Command::subcommand_negates_reqs`] is set
    pub fn is_subcommand_negates_reqs_set(&self) -> bool {
        self.is_set(AppSettings::SubcommandsNegateReqs)
    }

    /// Report whether [`Command::multicall`] is set
    pub fn is_multicall_set(&self) -> bool {
        self.is_set(AppSettings::Multicall)
    }
}

/// Deprecated
impl<'help> App<'help> {
    /// Deprecated in [Issue #3087](https://github.com/clap-rs/clap/issues/3087), maybe [`clap::Parser`][crate::Parser] would fit your use case?
    #[cfg(feature = "yaml")]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Deprecated in Issue #3087, maybe clap::Parser would fit your use case?"
        )
    )]
    #[doc(hidden)]
    pub fn from_yaml(y: &'help Yaml) -> Self {
        #![allow(deprecated)]
        let yaml_file_hash = y.as_hash().expect("YAML file must be a hash");
        // We WANT this to panic on error...so expect() is good.
        let (mut a, yaml, err) = if let Some(name) = y["name"].as_str() {
            (App::new(name), yaml_file_hash, "cmd".into())
        } else {
            let (name_yaml, value_yaml) = yaml_file_hash
                .iter()
                .next()
                .expect("There must be one subcommand in the YAML file");
            let name_str = name_yaml
                .as_str()
                .expect("Subcommand name must be a string");

            (
                App::new(name_str),
                value_yaml.as_hash().expect("Subcommand must be a hash"),
                format!("subcommand '{}'", name_str),
            )
        };

        for (k, v) in yaml {
            a = match k.as_str().expect("App fields must be strings") {
                "version" => yaml_to_str!(a, v, version),
                "long_version" => yaml_to_str!(a, v, long_version),
                "author" => yaml_to_str!(a, v, author),
                "bin_name" => yaml_to_str!(a, v, bin_name),
                "about" => yaml_to_str!(a, v, about),
                "long_about" => yaml_to_str!(a, v, long_about),
                "before_help" => yaml_to_str!(a, v, before_help),
                "after_help" => yaml_to_str!(a, v, after_help),
                "template" => yaml_to_str!(a, v, help_template),
                "usage" => yaml_to_str!(a, v, override_usage),
                "help" => yaml_to_str!(a, v, override_help),
                "help_message" => yaml_to_str!(a, v, help_message),
                "version_message" => yaml_to_str!(a, v, version_message),
                "alias" => yaml_to_str!(a, v, alias),
                "aliases" => yaml_vec_or_str!(a, v, alias),
                "visible_alias" => yaml_to_str!(a, v, visible_alias),
                "visible_aliases" => yaml_vec_or_str!(a, v, visible_alias),
                "display_order" => yaml_to_usize!(a, v, display_order),
                "args" => {
                    if let Some(vec) = v.as_vec() {
                        for arg_yaml in vec {
                            a = a.arg(Arg::from_yaml(arg_yaml));
                        }
                    } else {
                        panic!("Failed to convert YAML value {:?} to a vec", v);
                    }
                    a
                }
                "subcommands" => {
                    if let Some(vec) = v.as_vec() {
                        for sc_yaml in vec {
                            a = a.subcommand(App::from_yaml(sc_yaml));
                        }
                    } else {
                        panic!("Failed to convert YAML value {:?} to a vec", v);
                    }
                    a
                }
                "groups" => {
                    if let Some(vec) = v.as_vec() {
                        for ag_yaml in vec {
                            a = a.group(ArgGroup::from(ag_yaml));
                        }
                    } else {
                        panic!("Failed to convert YAML value {:?} to a vec", v);
                    }
                    a
                }
                "setting" | "settings" => {
                    yaml_to_setting!(a, v, setting, AppSettings, "AppSetting", err)
                }
                "global_setting" | "global_settings" => {
                    yaml_to_setting!(a, v, global_setting, AppSettings, "AppSetting", err)
                }
                _ => a,
            }
        }

        a
    }

    /// Deprecated, replaced with [`Command::override_usage`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::override_usage`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn usage<S: Into<&'help str>>(self, usage: S) -> Self {
        self.override_usage(usage)
    }

    /// Deprecated, replaced with [`Command::override_help`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::override_help`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn help<S: Into<&'help str>>(self, help: S) -> Self {
        self.override_help(help)
    }

    /// Deprecated, replaced with [`Command::mut_arg`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::mut_arg`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn help_short(self, c: char) -> Self {
        self.mut_arg("help", |a| a.short(c))
    }

    /// Deprecated, replaced with [`Command::mut_arg`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::mut_arg`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn version_short(self, c: char) -> Self {
        self.mut_arg("version", |a| a.short(c))
    }

    /// Deprecated, replaced with [`Command::mut_arg`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::mut_arg`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn help_message(self, s: impl Into<&'help str>) -> Self {
        self.mut_arg("help", |a| a.help(s.into()))
    }

    /// Deprecated, replaced with [`Command::mut_arg`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::mut_arg`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn version_message(self, s: impl Into<&'help str>) -> Self {
        self.mut_arg("version", |a| a.help(s.into()))
    }

    /// Deprecated, replaced with [`Command::help_template`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::help_template`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn template<S: Into<&'help str>>(self, s: S) -> Self {
        self.help_template(s)
    }

    /// Deprecated, replaced with [`Command::setting(a| b)`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::setting(a | b)`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn settings(mut self, settings: &[AppSettings]) -> Self {
        for s in settings {
            self.settings.insert((*s).into());
        }
        self
    }

    /// Deprecated, replaced with [`Command::unset_setting(a| b)`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::unset_setting(a | b)`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn unset_settings(mut self, settings: &[AppSettings]) -> Self {
        for s in settings {
            self.settings.remove((*s).into());
        }
        self
    }

    /// Deprecated, replaced with [`Command::global_setting(a| b)`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::global_setting(a | b)`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn global_settings(mut self, settings: &[AppSettings]) -> Self {
        for s in settings {
            self.settings.insert((*s).into());
            self.g_settings.insert((*s).into());
        }
        self
    }

    /// Deprecated, replaced with [`Command::term_width`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::term_width`")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn set_term_width(self, width: usize) -> Self {
        self.term_width(width)
    }

    /// Deprecated in [Issue #3086](https://github.com/clap-rs/clap/issues/3086), see [`arg!`][crate::arg!].
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Deprecated in Issue #3086, see `clap::arg!")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn arg_from_usage(self, usage: &'help str) -> Self {
        #![allow(deprecated)]
        self.arg(Arg::from_usage(usage))
    }

    /// Deprecated in [Issue #3086](https://github.com/clap-rs/clap/issues/3086), see [`arg!`][crate::arg!].
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Deprecated in Issue #3086, see `clap::arg!")
    )]
    #[doc(hidden)]
    #[must_use]
    pub fn args_from_usage(mut self, usage: &'help str) -> Self {
        #![allow(deprecated)]
        for line in usage.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            self = self.arg(Arg::from_usage(l));
        }
        self
    }

    /// Deprecated, replaced with [`Command::render_version`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::render_version`")
    )]
    #[doc(hidden)]
    pub fn write_version<W: io::Write>(&self, w: &mut W) -> ClapResult<()> {
        write!(w, "{}", self.render_version()).map_err(From::from)
    }

    /// Deprecated, replaced with [`Command::render_long_version`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::render_long_version`")
    )]
    #[doc(hidden)]
    pub fn write_long_version<W: io::Write>(&self, w: &mut W) -> ClapResult<()> {
        write!(w, "{}", self.render_long_version()).map_err(From::from)
    }

    /// Deprecated, replaced with [`Command::try_get_matches`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::try_get_matches`")
    )]
    #[doc(hidden)]
    pub fn get_matches_safe(self) -> ClapResult<ArgMatches> {
        self.try_get_matches()
    }

    /// Deprecated, replaced with [`Command::try_get_matches_from`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.0.0", note = "Replaced with `App::try_get_matches_from`")
    )]
    #[doc(hidden)]
    pub fn get_matches_from_safe<I, T>(self, itr: I) -> ClapResult<ArgMatches>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        self.try_get_matches_from(itr)
    }

    /// Deprecated, replaced with [`Command::try_get_matches_from_mut`]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(
            since = "3.0.0",
            note = "Replaced with `App::try_get_matches_from_mut`"
        )
    )]
    #[doc(hidden)]
    pub fn get_matches_from_safe_borrow<I, T>(&mut self, itr: I) -> ClapResult<ArgMatches>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        self.try_get_matches_from_mut(itr)
    }
}

// Internally used only
impl<'help> App<'help> {
    pub(crate) fn get_id(&self) -> Id {
        self.id.clone()
    }

    pub(crate) fn get_override_usage(&self) -> Option<&str> {
        self.usage_str
    }

    pub(crate) fn get_override_help(&self) -> Option<&str> {
        self.help_str
    }

    pub(crate) fn get_help_template(&self) -> Option<&str> {
        self.template
    }

    pub(crate) fn get_term_width(&self) -> Option<usize> {
        self.term_w
    }

    pub(crate) fn get_max_term_width(&self) -> Option<usize> {
        self.max_w
    }

    pub(crate) fn get_replacement(&self, key: &str) -> Option<&[&str]> {
        self.replacers.get(key).copied()
    }

    pub(crate) fn get_keymap(&self) -> &MKeyMap<'help> {
        &self.args
    }

    fn get_used_global_args(&self, matches: &ArgMatches, global_arg_vec: &mut Vec<Id>) {
        global_arg_vec.extend(
            self.args
                .args()
                .filter(|a| a.is_global_set())
                .map(|ga| ga.id.clone()),
        );
        if let Some((id, matches)) = matches.subcommand() {
            if let Some(used_sub) = self.find_subcommand(id) {
                used_sub.get_used_global_args(matches, global_arg_vec);
            }
        }
    }

    fn _do_parse(
        &mut self,
        raw_args: &mut clap_lex::RawArgs,
        args_cursor: clap_lex::ArgCursor,
    ) -> ClapResult<ArgMatches> {
        debug!("Command::_do_parse");

        // If there are global arguments, or settings we need to propagate them down to subcommands
        // before parsing in case we run into a subcommand
        self._build_self();

        let mut matcher = ArgMatcher::new(self);

        // do the real parsing
        let mut parser = Parser::new(self);
        if let Err(error) = parser.get_matches_with(&mut matcher, raw_args, args_cursor) {
            if self.is_set(AppSettings::IgnoreErrors) {
                debug!("Command::_do_parse: ignoring error: {}", error);
            } else {
                return Err(error);
            }
        }

        let mut global_arg_vec = Default::default();
        self.get_used_global_args(&matcher, &mut global_arg_vec);

        matcher.propagate_globals(&global_arg_vec);

        Ok(matcher.into_inner())
    }

    #[doc(hidden)]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.10", note = "Replaced with `Command::build`")
    )]
    pub fn _build_all(&mut self) {
        self.build();
    }

    #[doc(hidden)]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.10", note = "Replaced with `Command::build`")
    )]
    pub fn _build(&mut self) {
        self._build_self()
    }

    #[doc(hidden)]
    #[cfg_attr(
        feature = "deprecated",
        deprecated(since = "3.1.13", note = "Replaced with `Command::build`")
    )]
    pub fn _build_bin_names(&mut self) {
        self._build_bin_names_internal();
    }

    /// Prepare for introspecting on all included [`Command`]s
    ///
    /// Call this on the top-level [`Command`] when done building and before reading state for
    /// cases like completions, custom help output, etc.
    pub fn build(&mut self) {
        self._build_recursive();
        self._build_bin_names_internal();
    }

    pub(crate) fn _build_recursive(&mut self) {
        self._build_self();
        for subcmd in self.get_subcommands_mut() {
            subcmd._build_recursive();
        }
    }

    pub(crate) fn _build_self(&mut self) {
        debug!("Command::_build: name={:?}", self.get_name());
        if !self.settings.is_set(AppSettings::Built) {
            // Make sure all the globally set flags apply to us as well
            self.settings = self.settings | self.g_settings;

            if self.is_multicall_set() {
                self.settings.insert(AppSettings::SubcommandRequired.into());
                self.settings.insert(AppSettings::DisableHelpFlag.into());
                self.settings.insert(AppSettings::DisableVersionFlag.into());
            }

            self._propagate();
            self._check_help_and_version();
            self._propagate_global_args();
            self._derive_display_order();

            let mut pos_counter = 1;
            let self_override = self.is_set(AppSettings::AllArgsOverrideSelf);
            let hide_pv = self.is_set(AppSettings::HidePossibleValues);
            let auto_help =
                !self.is_set(AppSettings::NoAutoHelp) && !self.is_disable_help_flag_set();
            let auto_version =
                !self.is_set(AppSettings::NoAutoVersion) && !self.is_disable_version_flag_set();
            for a in self.args.args_mut() {
                // Fill in the groups
                for g in &a.groups {
                    if let Some(ag) = self.groups.iter_mut().find(|grp| grp.id == *g) {
                        ag.args.push(a.id.clone());
                    } else {
                        let mut ag = ArgGroup::with_id(g.clone());
                        ag.args.push(a.id.clone());
                        self.groups.push(ag);
                    }
                }

                // Figure out implied settings
                if a.is_last_set() {
                    // if an arg has `Last` set, we need to imply DontCollapseArgsInUsage so that args
                    // in the usage string don't get confused or left out.
                    self.settings.set(AppSettings::DontCollapseArgsInUsage);
                }
                if hide_pv && a.is_takes_value_set() {
                    a.settings.set(ArgSettings::HidePossibleValues);
                }
                if self_override {
                    let self_id = a.id.clone();
                    a.overrides.push(self_id);
                }
                a._build();
                // HACK: Setting up action at this level while auto-help / disable help flag is
                // required.  Otherwise, most of this won't be needed because when we can break
                // compat, actions will reign supreme (default to `Store`)
                if a.action.is_none() {
                    if a.get_id() == "help" && auto_help && !a.is_takes_value_set() {
                        let action = super::ArgAction::Help;
                        a.action = Some(action);
                    } else if a.get_id() == "version" && auto_version && !a.is_takes_value_set() {
                        let action = super::ArgAction::Version;
                        a.action = Some(action);
                    } else if a.is_takes_value_set() {
                        let action = super::ArgAction::StoreValue;
                        a.action = Some(action);
                    } else {
                        let action = super::ArgAction::IncOccurrence;
                        a.action = Some(action);
                    }
                }
                if a.is_positional() && a.index.is_none() {
                    a.index = Some(pos_counter);
                    pos_counter += 1;
                }
            }

            self.args._build();

            #[cfg(debug_assertions)]
            assert_app(self);
            self.settings.set(AppSettings::Built);
        } else {
            debug!("Command::_build: already built");
        }
    }

    pub(crate) fn _build_subcommand(&mut self, name: &str) -> Option<&mut Self> {
        use std::fmt::Write;

        let mut mid_string = String::from(" ");
        if !self.is_subcommand_negates_reqs_set() && !self.is_args_conflicts_with_subcommands_set()
        {
            let reqs = Usage::new(self).get_required_usage_from(&[], None, true); // maybe Some(m)

            for s in &reqs {
                mid_string.push_str(s);
                mid_string.push(' ');
            }
        }
        let is_multicall_set = self.is_multicall_set();

        let sc = self.subcommands.iter_mut().find(|s| s.name == name)?;

        // Display subcommand name, short and long in usage
        let mut sc_names = sc.name.clone();
        let mut flag_subcmd = false;
        if let Some(l) = sc.long_flag {
            write!(sc_names, "|--{}", l).unwrap();
            flag_subcmd = true;
        }
        if let Some(s) = sc.short_flag {
            write!(sc_names, "|-{}", s).unwrap();
            flag_subcmd = true;
        }

        if flag_subcmd {
            sc_names = format!("{{{}}}", sc_names);
        }

        let usage_name = self
            .bin_name
            .as_ref()
            .map(|bin_name| format!("{}{}{}", bin_name, mid_string, sc_names))
            .unwrap_or(sc_names);
        sc.usage_name = Some(usage_name);

        // bin_name should be parent's bin_name + [<reqs>] + the sc's name separated by
        // a space
        let bin_name = format!(
            "{}{}{}",
            self.bin_name.as_ref().unwrap_or(&String::new()),
            if self.bin_name.is_some() { " " } else { "" },
            &*sc.name
        );
        debug!(
            "Command::_build_subcommand Setting bin_name of {} to {:?}",
            sc.name, bin_name
        );
        sc.bin_name = Some(bin_name);

        if sc.display_name.is_none() {
            let self_display_name = if is_multicall_set {
                self.display_name.as_deref().unwrap_or("")
            } else {
                self.display_name.as_deref().unwrap_or(&self.name)
            };
            let display_name = format!(
                "{}{}{}",
                self_display_name,
                if !self_display_name.is_empty() {
                    "-"
                } else {
                    ""
                },
                &*sc.name
            );
            debug!(
                "Command::_build_subcommand Setting display_name of {} to {:?}",
                sc.name, display_name
            );
            sc.display_name = Some(display_name);
        }

        // Ensure all args are built and ready to parse
        sc._build_self();

        Some(sc)
    }

    fn _build_bin_names_internal(&mut self) {
        debug!("Command::_build_bin_names");

        if !self.is_set(AppSettings::BinNameBuilt) {
            let mut mid_string = String::from(" ");
            if !self.is_subcommand_negates_reqs_set()
                && !self.is_args_conflicts_with_subcommands_set()
            {
                let reqs = Usage::new(self).get_required_usage_from(&[], None, true); // maybe Some(m)

                for s in &reqs {
                    mid_string.push_str(s);
                    mid_string.push(' ');
                }
            }
            let is_multicall_set = self.is_multicall_set();

            let self_bin_name = if is_multicall_set {
                self.bin_name.as_deref().unwrap_or("")
            } else {
                self.bin_name.as_deref().unwrap_or(&self.name)
            }
            .to_owned();

            for mut sc in &mut self.subcommands {
                debug!("Command::_build_bin_names:iter: bin_name set...");

                if sc.usage_name.is_none() {
                    use std::fmt::Write;
                    // Display subcommand name, short and long in usage
                    let mut sc_names = sc.name.clone();
                    let mut flag_subcmd = false;
                    if let Some(l) = sc.long_flag {
                        write!(sc_names, "|--{}", l).unwrap();
                        flag_subcmd = true;
                    }
                    if let Some(s) = sc.short_flag {
                        write!(sc_names, "|-{}", s).unwrap();
                        flag_subcmd = true;
                    }

                    if flag_subcmd {
                        sc_names = format!("{{{}}}", sc_names);
                    }

                    let usage_name = format!("{}{}{}", self_bin_name, mid_string, sc_names);
                    debug!(
                        "Command::_build_bin_names:iter: Setting usage_name of {} to {:?}",
                        sc.name, usage_name
                    );
                    sc.usage_name = Some(usage_name);
                } else {
                    debug!(
                        "Command::_build_bin_names::iter: Using existing usage_name of {} ({:?})",
                        sc.name, sc.usage_name
                    );
                }

                if sc.bin_name.is_none() {
                    let bin_name = format!(
                        "{}{}{}",
                        self_bin_name,
                        if !self_bin_name.is_empty() { " " } else { "" },
                        &*sc.name
                    );
                    debug!(
                        "Command::_build_bin_names:iter: Setting bin_name of {} to {:?}",
                        sc.name, bin_name
                    );
                    sc.bin_name = Some(bin_name);
                } else {
                    debug!(
                        "Command::_build_bin_names::iter: Using existing bin_name of {} ({:?})",
                        sc.name, sc.bin_name
                    );
                }

                if sc.display_name.is_none() {
                    let self_display_name = if is_multicall_set {
                        self.display_name.as_deref().unwrap_or("")
                    } else {
                        self.display_name.as_deref().unwrap_or(&self.name)
                    };
                    let display_name = format!(
                        "{}{}{}",
                        self_display_name,
                        if !self_display_name.is_empty() {
                            "-"
                        } else {
                            ""
                        },
                        &*sc.name
                    );
                    debug!(
                        "Command::_build_bin_names:iter: Setting display_name of {} to {:?}",
                        sc.name, display_name
                    );
                    sc.display_name = Some(display_name);
                } else {
                    debug!(
                        "Command::_build_bin_names::iter: Using existing display_name of {} ({:?})",
                        sc.name, sc.display_name
                    );
                }

                sc._build_bin_names_internal();
            }
            self.set(AppSettings::BinNameBuilt);
        } else {
            debug!("Command::_build_bin_names: already built");
        }
    }

    pub(crate) fn _panic_on_missing_help(&self, help_required_globally: bool) {
        if self.is_set(AppSettings::HelpExpected) || help_required_globally {
            let args_missing_help: Vec<String> = self
                .args
                .args()
                .filter(|arg| arg.help.is_none() && arg.long_help.is_none())
                .map(|arg| String::from(arg.name))
                .collect();

            assert!(args_missing_help.is_empty(),
                    "Command::help_expected is enabled for the Command {}, but at least one of its arguments does not have either `help` or `long_help` set. List of such arguments: {}",
                    self.name,
                    args_missing_help.join(", ")
                );
        }

        for sub_app in &self.subcommands {
            sub_app._panic_on_missing_help(help_required_globally);
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn two_args_of<F>(&self, condition: F) -> Option<(&Arg<'help>, &Arg<'help>)>
    where
        F: Fn(&Arg) -> bool,
    {
        two_elements_of(self.args.args().filter(|a: &&Arg| condition(a)))
    }

    // just in case
    #[allow(unused)]
    fn two_groups_of<F>(&self, condition: F) -> Option<(&ArgGroup, &ArgGroup)>
    where
        F: Fn(&ArgGroup) -> bool,
    {
        two_elements_of(self.groups.iter().filter(|a| condition(a)))
    }

    /// Propagate global args
    pub(crate) fn _propagate_global_args(&mut self) {
        debug!("Command::_propagate_global_args:{}", self.name);

        for sc in &mut self.subcommands {
            for a in self.args.args().filter(|a| a.is_global_set()) {
                let mut propagate = false;
                let is_generated = matches!(
                    a.provider,
                    ArgProvider::Generated | ArgProvider::GeneratedMutated
                );

                // Remove generated help and version args in the subcommand
                //
                // Don't remove if those args are further mutated
                if is_generated {
                    let generated_pos = sc
                        .args
                        .args()
                        .position(|x| x.id == a.id && x.provider == ArgProvider::Generated);

                    if let Some(index) = generated_pos {
                        debug!(
                            "Command::_propagate removing {}'s {:?}",
                            sc.get_name(),
                            a.id
                        );
                        sc.args.remove(index);
                        propagate = true;
                    }
                }

                if propagate || sc.find(&a.id).is_none() {
                    debug!(
                        "Command::_propagate pushing {:?} to {}",
                        a.id,
                        sc.get_name(),
                    );
                    sc.args.push(a.clone());
                }
            }
        }
    }

    /// Propagate settings
    pub(crate) fn _propagate(&mut self) {
        debug!("Command::_propagate:{}", self.name);
        let mut subcommands = std::mem::take(&mut self.subcommands);
        for sc in &mut subcommands {
            self._propagate_subcommand(sc);
        }
        self.subcommands = subcommands;
    }

    fn _propagate_subcommand(&self, sc: &mut Self) {
        // We have to create a new scope in order to tell rustc the borrow of `sc` is
        // done and to recursively call this method
        {
            if self.settings.is_set(AppSettings::PropagateVersion) {
                if sc.version.is_none() && self.version.is_some() {
                    sc.version = Some(self.version.unwrap());
                }
                if sc.long_version.is_none() && self.long_version.is_some() {
                    sc.long_version = Some(self.long_version.unwrap());
                }
            }

            sc.settings = sc.settings | self.g_settings;
            sc.g_settings = sc.g_settings | self.g_settings;
            sc.term_w = self.term_w;
            sc.max_w = self.max_w;
        }
    }

    #[allow(clippy::blocks_in_if_conditions)]
    pub(crate) fn _check_help_and_version(&mut self) {
        debug!("Command::_check_help_and_version: {}", self.name);

        if self.is_set(AppSettings::DisableHelpFlag)
            || self.args.args().any(|x| {
                x.provider == ArgProvider::User
                    && (x.long == Some("help") || x.id == Id::help_hash())
            })
            || self
                .subcommands
                .iter()
                .any(|sc| sc.long_flag == Some("help"))
        {
            debug!("Command::_check_help_and_version: Removing generated help");

            let generated_help_pos = self
                .args
                .args()
                .position(|x| x.id == Id::help_hash() && x.provider == ArgProvider::Generated);

            if let Some(index) = generated_help_pos {
                self.args.remove(index);
            }
        } else {
            let help = self
                .args
                .args()
                .find(|x| x.id == Id::help_hash())
                .expect(INTERNAL_ERROR_MSG);
            assert_ne!(help.provider, ArgProvider::User);

            if help.short.is_some() {
                if help.short == Some('h') {
                    if let Some(other_arg) = self
                        .args
                        .args()
                        .find(|x| x.id != Id::help_hash() && x.short == Some('h'))
                    {
                        panic!(
                            "`help`s `-h` conflicts with `{}`.

To change `help`s short, call `cmd.arg(Arg::new(\"help\")...)`.",
                            other_arg.name
                        );
                    }
                }
            } else if !(self.args.args().any(|x| x.short == Some('h'))
                || self.subcommands.iter().any(|sc| sc.short_flag == Some('h')))
            {
                let help = self
                    .args
                    .args_mut()
                    .find(|x| x.id == Id::help_hash())
                    .expect(INTERNAL_ERROR_MSG);
                help.short = Some('h');
            } else {
                debug!("Command::_check_help_and_version: Removing `-h` from help");
            }
        }

        // Determine if we should remove the generated --version flag
        //
        // Note that if only mut_arg() was used, the first expression will evaluate to `true`
        // however inside the condition block, we only check for Generated args, not
        // GeneratedMutated args, so the `mut_arg("version", ..) will be skipped and fall through
        // to the following condition below (Adding the short `-V`)
        if self.settings.is_set(AppSettings::DisableVersionFlag)
            || (self.version.is_none() && self.long_version.is_none())
            || self.args.args().any(|x| {
                x.provider == ArgProvider::User
                    && (x.long == Some("version") || x.id == Id::version_hash())
            })
            || self
                .subcommands
                .iter()
                .any(|sc| sc.long_flag == Some("version"))
        {
            debug!("Command::_check_help_and_version: Removing generated version");

            // This is the check mentioned above that only checks for Generated, not
            // GeneratedMutated args by design.
            let generated_version_pos = self
                .args
                .args()
                .position(|x| x.id == Id::version_hash() && x.provider == ArgProvider::Generated);

            if let Some(index) = generated_version_pos {
                self.args.remove(index);
            }
        }

        // If we still have a generated --version flag, determine if we can apply the short `-V`
        if self.args.args().any(|x| {
            x.id == Id::version_hash()
                && matches!(
                    x.provider,
                    ArgProvider::Generated | ArgProvider::GeneratedMutated
                )
        }) {
            let other_arg_has_short = self.args.args().any(|x| x.short == Some('V'));
            let version = self
                .args
                .args_mut()
                .find(|x| x.id == Id::version_hash())
                .expect(INTERNAL_ERROR_MSG);

            if !(version.short.is_some()
                || other_arg_has_short
                || self.subcommands.iter().any(|sc| sc.short_flag == Some('V')))
            {
                version.short = Some('V');
            }
        }

        if !self.is_set(AppSettings::DisableHelpSubcommand)
            && self.has_subcommands()
            && !self.subcommands.iter().any(|s| s.id == Id::help_hash())
        {
            debug!("Command::_check_help_and_version: Building help subcommand");
            let mut help_subcmd = App::new("help")
                .about("Print this message or the help of the given subcommand(s)")
                .arg(
                    Arg::new("subcommand")
                        .index(1)
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .value_name("SUBCOMMAND")
                        .help("The subcommand whose help message to display"),
                );
            self._propagate_subcommand(&mut help_subcmd);

            // The parser acts like this is set, so let's set it so we don't falsely
            // advertise it to the user
            help_subcmd.version = None;
            help_subcmd.long_version = None;
            help_subcmd = help_subcmd
                .setting(AppSettings::DisableHelpFlag)
                .unset_global_setting(AppSettings::PropagateVersion);

            self.subcommands.push(help_subcmd);
        }
    }

    pub(crate) fn _derive_display_order(&mut self) {
        debug!("Command::_derive_display_order:{}", self.name);

        if self.settings.is_set(AppSettings::DeriveDisplayOrder) {
            for a in self
                .args
                .args_mut()
                .filter(|a| !a.is_positional())
                .filter(|a| a.provider != ArgProvider::Generated)
            {
                a.disp_ord.make_explicit();
            }
            for (i, sc) in &mut self.subcommands.iter_mut().enumerate() {
                sc.disp_ord.get_or_insert(i);
            }
        }
        for sc in &mut self.subcommands {
            sc._derive_display_order();
        }
    }

    pub(crate) fn _render_version(&self, use_long: bool) -> String {
        debug!("Command::_render_version");

        let ver = if use_long {
            self.long_version.or(self.version).unwrap_or("")
        } else {
            self.version.or(self.long_version).unwrap_or("")
        };
        if let Some(bn) = self.bin_name.as_ref() {
            if bn.contains(' ') {
                // In case we're dealing with subcommands i.e. git mv is translated to git-mv
                format!("{} {}\n", bn.replace(' ', "-"), ver)
            } else {
                format!("{} {}\n", &self.name[..], ver)
            }
        } else {
            format!("{} {}\n", &self.name[..], ver)
        }
    }

    pub(crate) fn format_group(&self, g: &Id) -> String {
        let g_string = self
            .unroll_args_in_group(g)
            .iter()
            .filter_map(|x| self.find(x))
            .map(|x| {
                if x.is_positional() {
                    // Print val_name for positional arguments. e.g. <file_name>
                    x.name_no_brackets().to_string()
                } else {
                    // Print usage string for flags arguments, e.g. <--help>
                    x.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("|");
        format!("<{}>", &*g_string)
    }
}

/// A workaround:
/// <https://github.com/rust-lang/rust/issues/34511#issuecomment-373423999>
pub(crate) trait Captures<'a> {}
impl<'a, T> Captures<'a> for T {}

// Internal Query Methods
impl<'help> App<'help> {
    /// Iterate through the *flags* & *options* arguments.
    pub(crate) fn get_non_positionals(&self) -> impl Iterator<Item = &Arg<'help>> {
        self.get_arguments().filter(|a| !a.is_positional())
    }

    /// Iterate through the *positionals* that don't have custom heading.
    pub(crate) fn get_positionals_with_no_heading(&self) -> impl Iterator<Item = &Arg<'help>> {
        self.get_positionals()
            .filter(|a| a.get_help_heading().is_none())
    }

    /// Iterate through the *flags* & *options* that don't have custom heading.
    pub(crate) fn get_non_positionals_with_no_heading(&self) -> impl Iterator<Item = &Arg<'help>> {
        self.get_non_positionals()
            .filter(|a| a.get_help_heading().is_none())
    }

    pub(crate) fn find(&self, arg_id: &Id) -> Option<&Arg<'help>> {
        self.args.args().find(|a| a.id == *arg_id)
    }

    #[inline]
    pub(crate) fn contains_short(&self, s: char) -> bool {
        assert!(
            self.is_set(AppSettings::Built),
            "If App::_build hasn't been called, manually search through Arg shorts"
        );

        self.args.contains(s)
    }

    #[inline]
    pub(crate) fn set(&mut self, s: AppSettings) {
        self.settings.set(s)
    }

    #[inline]
    pub(crate) fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    pub(crate) fn has_positionals(&self) -> bool {
        self.args.keys().any(|x| x.is_position())
    }

    pub(crate) fn has_visible_subcommands(&self) -> bool {
        self.subcommands
            .iter()
            .any(|sc| sc.name != "help" && !sc.is_set(AppSettings::Hidden))
    }

    /// Check if this subcommand can be referred to as `name`. In other words,
    /// check if `name` is the name of this subcommand or is one of its aliases.
    #[inline]
    pub(crate) fn aliases_to<T>(&self, name: &T) -> bool
    where
        T: PartialEq<str> + ?Sized,
    {
        *name == *self.get_name() || self.get_all_aliases().any(|alias| *name == *alias)
    }

    /// Check if this subcommand can be referred to as `name`. In other words,
    /// check if `name` is the name of this short flag subcommand or is one of its short flag aliases.
    #[inline]
    pub(crate) fn short_flag_aliases_to(&self, flag: char) -> bool {
        Some(flag) == self.short_flag
            || self.get_all_short_flag_aliases().any(|alias| flag == alias)
    }

    /// Check if this subcommand can be referred to as `name`. In other words,
    /// check if `name` is the name of this long flag subcommand or is one of its long flag aliases.
    #[inline]
    pub(crate) fn long_flag_aliases_to<T>(&self, flag: &T) -> bool
    where
        T: PartialEq<str> + ?Sized,
    {
        match self.long_flag {
            Some(long_flag) => {
                flag == long_flag || self.get_all_long_flag_aliases().any(|alias| flag == alias)
            }
            None => self.get_all_long_flag_aliases().any(|alias| flag == alias),
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn id_exists(&self, id: &Id) -> bool {
        self.args.args().any(|x| x.id == *id) || self.groups.iter().any(|x| x.id == *id)
    }

    /// Iterate through the groups this arg is member of.
    pub(crate) fn groups_for_arg<'a>(&'a self, arg: &Id) -> impl Iterator<Item = Id> + 'a {
        debug!("Command::groups_for_arg: id={:?}", arg);
        let arg = arg.clone();
        self.groups
            .iter()
            .filter(move |grp| grp.args.iter().any(|a| a == &arg))
            .map(|grp| grp.id.clone())
    }

    pub(crate) fn find_group(&self, group_id: &Id) -> Option<&ArgGroup<'help>> {
        self.groups.iter().find(|g| g.id == *group_id)
    }

    /// Iterate through all the names of all subcommands (not recursively), including aliases.
    /// Used for suggestions.
    pub(crate) fn all_subcommand_names(&self) -> impl Iterator<Item = &str> + Captures<'help> {
        self.get_subcommands().flat_map(|sc| {
            let name = sc.get_name();
            let aliases = sc.get_all_aliases();
            std::iter::once(name).chain(aliases)
        })
    }

    pub(crate) fn required_graph(&self) -> ChildGraph<Id> {
        let mut reqs = ChildGraph::with_capacity(5);
        for a in self.args.args().filter(|a| a.is_required_set()) {
            reqs.insert(a.id.clone());
        }
        for group in &self.groups {
            if group.required {
                let idx = reqs.insert(group.id.clone());
                for a in &group.requires {
                    reqs.insert_child(idx, a.clone());
                }
            }
        }

        reqs
    }

    pub(crate) fn unroll_args_in_group(&self, group: &Id) -> Vec<Id> {
        debug!("Command::unroll_args_in_group: group={:?}", group);
        let mut g_vec = vec![group];
        let mut args = vec![];

        while let Some(g) = g_vec.pop() {
            for n in self
                .groups
                .iter()
                .find(|grp| grp.id == *g)
                .expect(INTERNAL_ERROR_MSG)
                .args
                .iter()
            {
                debug!("Command::unroll_args_in_group:iter: entity={:?}", n);
                if !args.contains(n) {
                    if self.find(n).is_some() {
                        debug!("Command::unroll_args_in_group:iter: this is an arg");
                        args.push(n.clone())
                    } else {
                        debug!("Command::unroll_args_in_group:iter: this is a group");
                        g_vec.push(n);
                    }
                }
            }
        }

        args
    }

    pub(crate) fn unroll_arg_requires<F>(&self, func: F, arg: &Id) -> Vec<Id>
    where
        F: Fn(&(ArgPredicate<'_>, Id)) -> Option<Id>,
    {
        let mut processed = vec![];
        let mut r_vec = vec![arg];
        let mut args = vec![];

        while let Some(a) = r_vec.pop() {
            if processed.contains(&a) {
                continue;
            }

            processed.push(a);

            if let Some(arg) = self.find(a) {
                for r in arg.requires.iter().filter_map(&func) {
                    if let Some(req) = self.find(&r) {
                        if !req.requires.is_empty() {
                            r_vec.push(&req.id)
                        }
                    }
                    args.push(r);
                }
            }
        }

        args
    }

    /// Find a flag subcommand name by short flag or an alias
    pub(crate) fn find_short_subcmd(&self, c: char) -> Option<&str> {
        self.get_subcommands()
            .find(|sc| sc.short_flag_aliases_to(c))
            .map(|sc| sc.get_name())
    }

    /// Find a flag subcommand name by long flag or an alias
    pub(crate) fn find_long_subcmd(&self, long: &str) -> Option<&str> {
        self.get_subcommands()
            .find(|sc| sc.long_flag_aliases_to(long))
            .map(|sc| sc.get_name())
    }

    pub(crate) fn get_display_order(&self) -> usize {
        self.disp_ord.unwrap_or(999)
    }

    pub(crate) fn write_help_err(
        &self,
        mut use_long: bool,
        stream: Stream,
    ) -> ClapResult<Colorizer> {
        debug!(
            "Parser::write_help_err: use_long={:?}, stream={:?}",
            use_long && self.use_long_help(),
            stream
        );

        use_long = use_long && self.use_long_help();
        let usage = Usage::new(self);

        let mut c = Colorizer::new(stream, self.color_help());
        Help::new(HelpWriter::Buffer(&mut c), self, &usage, use_long).write_help()?;
        Ok(c)
    }

    pub(crate) fn use_long_help(&self) -> bool {
        debug!("Command::use_long_help");
        // In this case, both must be checked. This allows the retention of
        // original formatting, but also ensures that the actual -h or --help
        // specified by the user is sent through. If hide_short_help is not included,
        // then items specified with hidden_short_help will also be hidden.
        let should_long = |v: &Arg| {
            v.long_help.is_some()
                || v.is_hide_long_help_set()
                || v.is_hide_short_help_set()
                || cfg!(feature = "unstable-v4")
                    && v.get_possible_values2()
                        .iter()
                        .any(PossibleValue::should_show_help)
        };

        // Subcommands aren't checked because we prefer short help for them, deferring to
        // `cmd subcmd --help` for more.
        self.get_long_about().is_some()
            || self.get_before_long_help().is_some()
            || self.get_after_long_help().is_some()
            || self.get_arguments().any(should_long)
    }

    // Should we color the help?
    pub(crate) fn color_help(&self) -> ColorChoice {
        #[cfg(feature = "color")]
        if self.is_disable_colored_help_set() {
            return ColorChoice::Never;
        }

        self.get_color()
    }
}

impl<'help> Default for App<'help> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            long_flag: Default::default(),
            short_flag: Default::default(),
            display_name: Default::default(),
            bin_name: Default::default(),
            author: Default::default(),
            version: Default::default(),
            long_version: Default::default(),
            about: Default::default(),
            long_about: Default::default(),
            before_help: Default::default(),
            before_long_help: Default::default(),
            after_help: Default::default(),
            after_long_help: Default::default(),
            aliases: Default::default(),
            short_flag_aliases: Default::default(),
            long_flag_aliases: Default::default(),
            usage_str: Default::default(),
            usage_name: Default::default(),
            help_str: Default::default(),
            disp_ord: Default::default(),
            term_w: Default::default(),
            max_w: Default::default(),
            template: Default::default(),
            settings: Default::default(),
            g_settings: Default::default(),
            args: Default::default(),
            subcommands: Default::default(),
            replacers: Default::default(),
            groups: Default::default(),
            current_help_heading: Default::default(),
            current_disp_ord: Some(0),
            subcommand_value_name: Default::default(),
            subcommand_heading: Default::default(),
        }
    }
}

impl<'help> Index<&'_ Id> for App<'help> {
    type Output = Arg<'help>;

    fn index(&self, key: &Id) -> &Self::Output {
        self.find(key).expect(INTERNAL_ERROR_MSG)
    }
}

impl fmt::Display for App<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn two_elements_of<I, T>(mut iter: I) -> Option<(T, T)>
where
    I: Iterator<Item = T>,
{
    let first = iter.next();
    let second = iter.next();

    match (first, second) {
        (Some(first), Some(second)) => Some((first, second)),
        _ => None,
    }
}

#[test]
fn check_auto_traits() {
    static_assertions::assert_impl_all!(Command: Send, Sync, Unpin);
}
