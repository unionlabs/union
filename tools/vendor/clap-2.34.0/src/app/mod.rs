mod help;
mod meta;
pub mod parser;
mod settings;
mod usage;
mod validator;

// Std
use std::result::Result as StdResult;
use std::{
    env,
    ffi::{OsStr, OsString},
    fmt,
    io::{self, BufRead, BufWriter, Write},
    path::Path,
    process,
    rc::Rc,
};

// Third Party
#[cfg(feature = "yaml")]
use yaml_rust::Yaml;

// Internal
use crate::errors::Result as ClapResult;
use crate::{
    app::{help::Help, parser::Parser},
    args::{AnyArg, Arg, ArgGroup, ArgMatcher, ArgMatches, ArgSettings},
    completions::Shell,
    map::{self, VecMap},
};
pub use settings::AppSettings;

/// Used to create a representation of a command line program and all possible command line
/// arguments. Application settings are set using the "builder pattern" with the
/// [`App::get_matches`] family of methods being the terminal methods that starts the
/// runtime-parsing process. These methods then return information about the user supplied
/// arguments (or lack there of).
///
/// **NOTE:** There aren't any mandatory "options" that one must set. The "options" may
/// also appear in any order (so long as one of the [`App::get_matches`] methods is the last method
/// called).
///
/// # Examples
///
/// ```no_run
/// # use clap::{App, Arg};
/// let m = App::new("My Program")
///     .author("Me, me@mail.com")
///     .version("1.0.2")
///     .about("Explains in brief what the program does")
///     .arg(
///         Arg::with_name("in_file").index(1)
///     )
///     .after_help("Longer explanation to appear after the options when \
///                  displaying the help information from --help or -h")
///     .get_matches();
///
/// // Your program logic starts here...
/// ```
/// [`App::get_matches`]: ./struct.App.html#method.get_matches
#[allow(missing_debug_implementations)]
pub struct App<'a, 'b>
where
    'a: 'b,
{
    #[doc(hidden)]
    pub p: Parser<'a, 'b>,
}

impl<'a, 'b> App<'a, 'b> {
    /// Creates a new instance of an application requiring a name. The name may be, but doesn't
    /// have to be same as the binary. The name will be displayed to the user when they request to
    /// print version or help and usage information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let prog = App::new("My Program")
    /// # ;
    /// ```
    pub fn new<S: Into<String>>(n: S) -> Self {
        App {
            p: Parser::with_name(n.into()),
        }
    }

    /// Get the name of the app
    pub fn get_name(&self) -> &str {
        &self.p.meta.name
    }

    /// Get the name of the binary
    pub fn get_bin_name(&self) -> Option<&str> {
        self.p.meta.bin_name.as_deref()
    }

    /// Creates a new instance of an application requiring a name, but uses the [`crate_authors!`]
    /// and [`crate_version!`] macros to fill in the [`App::author`] and [`App::version`] fields.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let prog = App::with_defaults("My Program")
    /// # ;
    /// ```
    /// [`crate_authors!`]: ./macro.crate_authors!.html
    /// [`crate_version!`]: ./macro.crate_version!.html
    /// [`App::author`]: ./struct.App.html#method.author
    /// [`App::version`]: ./struct.App.html#method.author
    #[deprecated(
        since = "2.14.1",
        note = "Can never work; use explicit App::author() and App::version() calls instead"
    )]
    pub fn with_defaults<S: Into<String>>(n: S) -> Self {
        let mut a = App {
            p: Parser::with_name(n.into()),
        };
        a.p.meta.author = Some("Kevin K. <kbknapp@gmail.com>");
        a.p.meta.version = Some("2.19.2");
        a
    }

    /// Creates a new instance of [`App`] from a .yml (YAML) file. A full example of supported YAML
    /// objects can be found in [`examples/17_yaml.rs`] and [`examples/17_yaml.yml`]. One great use
    /// for using YAML is when supporting multiple languages and dialects, as each language could
    /// be a distinct YAML file and determined at compiletime via `cargo` "features" in your
    /// `Cargo.toml`
    ///
    /// In order to use this function you must compile `clap` with the `features = ["yaml"]` in
    /// your settings for the `[dependencies.clap]` table of your `Cargo.toml`
    ///
    /// **NOTE:** Due to how the YAML objects are built there is a convenience macro for loading
    /// the YAML file at compile time (relative to the current file, like modules work). That YAML
    /// object can then be passed to this function.
    ///
    /// # Panics
    ///
    /// The YAML file must be properly formatted or this function will [`panic!`]. A good way to
    /// ensure this doesn't happen is to run your program with the `--help` switch. If this passes
    /// without error, you needn't worry because the YAML is properly formatted.
    ///
    /// # Examples
    ///
    /// The following example shows how to load a properly formatted YAML file to build an instance
    /// of an [`App`] struct.
    ///
    /// ```ignore
    /// # #[macro_use]
    /// # extern crate clap;
    /// # use clap::App;
    /// # fn main() {
    /// let yml = load_yaml!("app.yml");
    /// let app = App::from_yaml(yml);
    ///
    /// // continued logic goes here, such as `app.get_matches()` etc.
    /// # }
    /// ```
    /// [`App`]: ./struct.App.html
    /// [`examples/17_yaml.rs`]: https://github.com/clap-rs/clap/blob/v2.33.1/examples/17_yaml.rs
    /// [`examples/17_yaml.yml`]: https://github.com/clap-rs/clap/blob/v2.33.1/examples/17_yaml.yml
    /// [`panic!`]: https://doc.rust-lang.org/std/macro.panic!.html
    #[cfg(feature = "yaml")]
    pub fn from_yaml(yaml: &'a Yaml) -> App<'a, 'a> {
        App::from(yaml)
    }

    /// Sets a string of author(s) that will be displayed to the user when they
    /// request the help information with `--help` or `-h`.
    ///
    /// **Pro-tip:** Use `clap`s convenience macro [`crate_authors!`] to automatically set your
    /// application's author(s) to the same thing as your crate at compile time. See the [`examples/`]
    /// directory for more information
    ///
    /// See the [`examples/`]
    /// directory for more information
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///      .author("Me, me@mymain.com")
    /// # ;
    /// ```
    /// [`crate_authors!`]: ./macro.crate_authors!.html
    /// [`examples/`]: https://github.com/clap-rs/clap/tree/v2.33.1/examples
    pub fn author<S: Into<&'b str>>(mut self, author: S) -> Self {
        self.p.meta.author = Some(author.into());
        self
    }

    /// Overrides the system-determined binary name. This should only be used when absolutely
    /// necessary, such as when the binary name for your application is misleading, or perhaps
    /// *not* how the user should invoke your program.
    ///
    /// **Pro-tip:** When building things such as third party `cargo` subcommands, this setting
    /// **should** be used!
    ///
    /// **NOTE:** This command **should not** be used for [`SubCommand`]s.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("My Program")
    ///      .bin_name("my_binary")
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    pub fn bin_name<S: Into<String>>(mut self, name: S) -> Self {
        self.p.meta.bin_name = Some(name.into());
        self
    }

    /// Sets a string describing what the program does. This will be displayed when displaying help
    /// information with `-h`.
    ///
    /// **NOTE:** If only `about` is provided, and not [`App::long_about`] but the user requests
    /// `--help` clap will still display the contents of `about` appropriately
    ///
    /// **NOTE:** Only [`App::about`] is used in completion script generation in order to be
    /// concise
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .about("Does really amazing things to great people")
    /// # ;
    /// ```
    /// [`App::long_about`]: ./struct.App.html#method.long_about
    pub fn about<S: Into<&'b str>>(mut self, about: S) -> Self {
        self.p.meta.about = Some(about.into());
        self
    }

    /// Sets a string describing what the program does. This will be displayed when displaying help
    /// information.
    ///
    /// **NOTE:** If only `long_about` is provided, and not [`App::about`] but the user requests
    /// `-h` clap will still display the contents of `long_about` appropriately
    ///
    /// **NOTE:** Only [`App::about`] is used in completion script generation in order to be
    /// concise
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .long_about(
    /// "Does really amazing things to great people. Now let's talk a little
    ///  more in depth about how this subcommand really works. It may take about
    ///  a few lines of text, but that's ok!")
    /// # ;
    /// ```
    /// [`App::about`]: ./struct.App.html#method.about
    pub fn long_about<S: Into<&'b str>>(mut self, about: S) -> Self {
        self.p.meta.long_about = Some(about.into());
        self
    }

    /// Sets the program's name. This will be displayed when displaying help information.
    ///
    /// **Pro-top:** This function is particularly useful when configuring a program via
    /// [`App::from_yaml`] in conjunction with the [`crate_name!`] macro to derive the program's
    /// name from its `Cargo.toml`.
    ///
    /// # Examples
    /// ```ignore
    /// # #[macro_use]
    /// # extern crate clap;
    /// # use clap::App;
    /// # fn main() {
    /// let yml = load_yaml!("app.yml");
    /// let app = App::from_yaml(yml)
    ///     .name(crate_name!());
    ///
    /// // continued logic goes here, such as `app.get_matches()` etc.
    /// # }
    /// ```
    ///
    /// [`App::from_yaml`]: ./struct.App.html#method.from_yaml
    /// [`crate_name!`]: ./macro.crate_name.html
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.p.meta.name = name.into();
        self
    }

    /// Adds additional help information to be displayed in addition to auto-generated help. This
    /// information is displayed **after** the auto-generated help information. This is often used
    /// to describe how to use the arguments, or caveats to be noted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::App;
    /// App::new("myprog")
    ///     .after_help("Does really amazing things to great people...but be careful with -R")
    /// # ;
    /// ```
    pub fn after_help<S: Into<&'b str>>(mut self, help: S) -> Self {
        self.p.meta.more_help = Some(help.into());
        self
    }

    /// Adds additional help information to be displayed in addition to auto-generated help. This
    /// information is displayed **before** the auto-generated help information. This is often used
    /// for header information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::App;
    /// App::new("myprog")
    ///     .before_help("Some info I'd like to appear before the help info")
    /// # ;
    /// ```
    pub fn before_help<S: Into<&'b str>>(mut self, help: S) -> Self {
        self.p.meta.pre_help = Some(help.into());
        self
    }

    /// Sets a string of the version number to be displayed when displaying version or help
    /// information with `-V`.
    ///
    /// **NOTE:** If only `version` is provided, and not [`App::long_version`] but the user
    /// requests `--version` clap will still display the contents of `version` appropriately
    ///
    /// **Pro-tip:** Use `clap`s convenience macro [`crate_version!`] to automatically set your
    /// application's version to the same thing as your crate at compile time. See the [`examples/`]
    /// directory for more information
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .version("v0.1.24")
    /// # ;
    /// ```
    /// [`crate_version!`]: ./macro.crate_version!.html
    /// [`examples/`]: https://github.com/clap-rs/clap/tree/v2.33.1/examples
    /// [`App::long_version`]: ./struct.App.html#method.long_version
    pub fn version<S: Into<&'b str>>(mut self, ver: S) -> Self {
        self.p.meta.version = Some(ver.into());
        self
    }

    /// Sets a string of the version number to be displayed when displaying version or help
    /// information with `--version`.
    ///
    /// **NOTE:** If only `long_version` is provided, and not [`App::version`] but the user
    /// requests `-V` clap will still display the contents of `long_version` appropriately
    ///
    /// **Pro-tip:** Use `clap`s convenience macro [`crate_version!`] to automatically set your
    /// application's version to the same thing as your crate at compile time. See the [`examples/`]
    /// directory for more information
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .long_version(
    /// "v0.1.24
    ///  commit: abcdef89726d
    ///  revision: 123
    ///  release: 2
    ///  binary: myprog")
    /// # ;
    /// ```
    /// [`crate_version!`]: ./macro.crate_version!.html
    /// [`examples/`]: https://github.com/clap-rs/clap/tree/v2.33.1/examples
    /// [`App::version`]: ./struct.App.html#method.version
    pub fn long_version<S: Into<&'b str>>(mut self, ver: S) -> Self {
        self.p.meta.long_version = Some(ver.into());
        self
    }

    /// Sets a custom usage string to override the auto-generated usage string.
    ///
    /// This will be displayed to the user when errors are found in argument parsing, or when you
    /// call [`ArgMatches::usage`]
    ///
    /// **CAUTION:** Using this setting disables `clap`s "context-aware" usage strings. After this
    /// setting is set, this will be the only usage string displayed to the user!
    ///
    /// **NOTE:** You do not need to specify the "USAGE: \n\t" portion, as that will
    /// still be applied by `clap`, you only need to specify the portion starting
    /// with the binary name.
    ///
    /// **NOTE:** This will not replace the entire help message, *only* the portion
    /// showing the usage.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .usage("myapp [-clDas] <some_file>")
    /// # ;
    /// ```
    /// [`ArgMatches::usage`]: ./struct.ArgMatches.html#method.usage
    pub fn usage<S: Into<&'b str>>(mut self, usage: S) -> Self {
        self.p.meta.usage_str = Some(usage.into());
        self
    }

    /// Sets a custom help message and overrides the auto-generated one. This should only be used
    /// when the auto-generated message does not suffice.
    ///
    /// This will be displayed to the user when they use `--help` or `-h`
    ///
    /// **NOTE:** This replaces the **entire** help message, so nothing will be auto-generated.
    ///
    /// **NOTE:** This **only** replaces the help message for the current command, meaning if you
    /// are using subcommands, those help messages will still be auto-generated unless you
    /// specify a [`Arg::help`] for them as well.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myapp")
    ///     .help("myapp v1.0\n\
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
    ///            Commmands:\n\
    ///            help             Prints this message\n\
    ///            work             Do some work")
    /// # ;
    /// ```
    /// [`Arg::help`]: ./struct.Arg.html#method.help
    pub fn help<S: Into<&'b str>>(mut self, help: S) -> Self {
        self.p.meta.help_str = Some(help.into());
        self
    }

    /// Sets the [`short`] for the auto-generated `help` argument.
    ///
    /// By default `clap` automatically assigns `h`, but this can be overridden if you have a
    /// different argument which you'd prefer to use the `-h` short with. This can be done by
    /// defining your own argument with a lowercase `h` as the [`short`].
    ///
    /// `clap` lazily generates these `help` arguments **after** you've defined any arguments of
    /// your own.
    ///
    /// **NOTE:** Any leading `-` characters will be stripped, and only the first
    /// non `-` character will be used as the [`short`] version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .help_short("H") // Using an uppercase `H` instead of the default lowercase `h`
    /// # ;
    /// ```
    /// [`short`]: ./struct.Arg.html#method.short
    pub fn help_short<S: AsRef<str> + 'b>(mut self, s: S) -> Self {
        self.p.help_short(s.as_ref());
        self
    }

    /// Sets the [`short`] for the auto-generated `version` argument.
    ///
    /// By default `clap` automatically assigns `V`, but this can be overridden if you have a
    /// different argument which you'd prefer to use the `-V` short with. This can be done by
    /// defining your own argument with an uppercase `V` as the [`short`].
    ///
    /// `clap` lazily generates these `version` arguments **after** you've defined any arguments of
    /// your own.
    ///
    /// **NOTE:** Any leading `-` characters will be stripped, and only the first
    /// non `-` character will be used as the `short` version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .version_short("v") // Using a lowercase `v` instead of the default capital `V`
    /// # ;
    /// ```
    /// [`short`]: ./struct.Arg.html#method.short
    pub fn version_short<S: AsRef<str>>(mut self, s: S) -> Self {
        self.p.version_short(s.as_ref());
        self
    }

    /// Sets the help text for the auto-generated `help` argument.
    ///
    /// By default `clap` sets this to `"Prints help information"`, but if you're using a
    /// different convention for your help messages and would prefer a different phrasing you can
    /// override it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .help_message("Print help information") // Perhaps you want imperative help messages
    ///
    /// # ;
    /// ```
    pub fn help_message<S: Into<&'a str>>(mut self, s: S) -> Self {
        self.p.help_message = Some(s.into());
        self
    }

    /// Sets the help text for the auto-generated `version` argument.
    ///
    /// By default `clap` sets this to `"Prints version information"`, but if you're using a
    /// different convention for your help messages and would prefer a different phrasing then you
    /// can change it.
    ///
    /// # Examples
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .version_message("Print version information") // Perhaps you want imperative help messages
    /// # ;
    /// ```
    pub fn version_message<S: Into<&'a str>>(mut self, s: S) -> Self {
        self.p.version_message = Some(s.into());
        self
    }

    /// Sets the help template to be used, overriding the default format.
    ///
    /// Tags arg given inside curly brackets.
    ///
    /// Valid tags are:
    ///
    ///   * `{bin}`         - Binary name.
    ///   * `{version}`     - Version number.
    ///   * `{author}`      - Author information.
    ///   * `{about}`       - General description (from [`App::about`])
    ///   * `{usage}`       - Automatically generated or given usage string.
    ///   * `{all-args}`    - Help for all arguments (options, flags, positionals arguments,
    ///                       and subcommands) including titles.
    ///   * `{unified}`     - Unified help for options and flags. Note, you must *also* set
    ///                       [`AppSettings::UnifiedHelpMessage`] to fully merge both options and
    ///                       flags, otherwise the ordering is "best effort"
    ///   * `{flags}`       - Help for flags.
    ///   * `{options}`     - Help for options.
    ///   * `{positionals}` - Help for positionals arguments.
    ///   * `{subcommands}` - Help for subcommands.
    ///   * `{after-help}`  - Help from [`App::after_help`]
    ///   * `{before-help}`  - Help from [`App::before_help`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .version("1.0")
    ///     .template("{bin} ({version}) - {usage}")
    /// # ;
    /// ```
    /// **NOTE:** The template system is, on purpose, very simple. Therefore the tags have to be
    /// written in lowercase and without spacing.
    ///
    /// [`App::about`]: ./struct.App.html#method.about
    /// [`App::after_help`]: ./struct.App.html#method.after_help
    /// [`App::before_help`]: ./struct.App.html#method.before_help
    /// [`AppSettings::UnifiedHelpMessage`]: ./enum.AppSettings.html#variant.UnifiedHelpMessage
    pub fn template<S: Into<&'b str>>(mut self, s: S) -> Self {
        self.p.meta.template = Some(s.into());
        self
    }

    /// Enables a single command, or [`SubCommand`], level settings.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, AppSettings};
    /// App::new("myprog")
    ///     .setting(AppSettings::SubcommandRequired)
    ///     .setting(AppSettings::WaitOnError)
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`AppSettings`]: ./enum.AppSettings.html
    pub fn setting(mut self, setting: AppSettings) -> Self {
        self.p.set(setting);
        self
    }

    /// Enables multiple command, or [`SubCommand`], level settings
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, AppSettings};
    /// App::new("myprog")
    ///     .settings(&[AppSettings::SubcommandRequired,
    ///                  AppSettings::WaitOnError])
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`AppSettings`]: ./enum.AppSettings.html
    pub fn settings(mut self, settings: &[AppSettings]) -> Self {
        for s in settings {
            self.p.set(*s);
        }
        self
    }

    /// Enables a single setting that is propagated down through all child [`SubCommand`]s.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// **NOTE**: The setting is *only* propagated *down* and not up through parent commands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, AppSettings};
    /// App::new("myprog")
    ///     .global_setting(AppSettings::SubcommandRequired)
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`AppSettings`]: ./enum.AppSettings.html
    pub fn global_setting(mut self, setting: AppSettings) -> Self {
        self.p.set(setting);
        self.p.g_settings.set(setting);
        self
    }

    /// Enables multiple settings which are propagated *down* through all child [`SubCommand`]s.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// **NOTE**: The setting is *only* propagated *down* and not up through parent commands.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, AppSettings};
    /// App::new("myprog")
    ///     .global_settings(&[AppSettings::SubcommandRequired,
    ///                  AppSettings::ColoredHelp])
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`AppSettings`]: ./enum.AppSettings.html
    pub fn global_settings(mut self, settings: &[AppSettings]) -> Self {
        for s in settings {
            self.p.set(*s);
            self.p.g_settings.set(*s)
        }
        self
    }

    /// Disables a single command, or [`SubCommand`], level setting.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, AppSettings};
    /// App::new("myprog")
    ///     .unset_setting(AppSettings::ColorAuto)
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`AppSettings`]: ./enum.AppSettings.html
    pub fn unset_setting(mut self, setting: AppSettings) -> Self {
        self.p.unset(setting);
        self
    }

    /// Disables multiple command, or [`SubCommand`], level settings.
    ///
    /// See [`AppSettings`] for a full list of possibilities and examples.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, AppSettings};
    /// App::new("myprog")
    ///     .unset_settings(&[AppSettings::ColorAuto,
    ///                       AppSettings::AllowInvalidUtf8])
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`AppSettings`]: ./enum.AppSettings.html
    pub fn unset_settings(mut self, settings: &[AppSettings]) -> Self {
        for s in settings {
            self.p.unset(*s);
        }
        self
    }

    /// Sets the terminal width at which to wrap help messages. Defaults to `120`. Using `0` will
    /// ignore terminal widths and use source formatting.
    ///
    /// `clap` automatically tries to determine the terminal width on Unix, Linux, macOS and Windows
    /// if the `wrap_help` cargo "feature" has been used while compiling. If the terminal width
    /// cannot be determined, `clap` defaults to `120`.
    ///
    /// **NOTE:** This setting applies globally and *not* on a per-command basis.
    ///
    /// **NOTE:** This setting must be set **before** any subcommands are added!
    ///
    /// # Platform Specific
    ///
    /// Only Unix, Linux, macOS and Windows support automatic determination of terminal width.
    /// Even on those platforms, this setting is useful if for any reason the terminal width
    /// cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::App;
    /// App::new("myprog")
    ///     .set_term_width(80)
    /// # ;
    /// ```
    pub fn set_term_width(mut self, width: usize) -> Self {
        self.p.meta.term_w = Some(width);
        self
    }

    /// Sets the max terminal width at which to wrap help messages. Using `0` will ignore terminal
    /// widths and use source formatting.
    ///
    /// `clap` automatically tries to determine the terminal width on Unix, Linux, macOS and Windows
    /// if the `wrap_help` cargo "feature" has been used while compiling, but one might want to
    /// limit the size (e.g. when the terminal is running fullscreen).
    ///
    /// **NOTE:** This setting applies globally and *not* on a per-command basis.
    ///
    /// **NOTE:** This setting must be set **before** any subcommands are added!
    ///
    /// # Platform Specific
    ///
    /// Only Unix, Linux, macOS and Windows support automatic determination of terminal width.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::App;
    /// App::new("myprog")
    ///     .max_term_width(100)
    /// # ;
    /// ```
    pub fn max_term_width(mut self, w: usize) -> Self {
        self.p.meta.max_w = Some(w);
        self
    }

    /// Adds an [argument] to the list of valid possibilities.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     // Adding a single "flag" argument with a short and help text, using Arg::with_name()
    ///     .arg(
    ///         Arg::with_name("debug")
    ///            .short("d")
    ///            .help("turns on debugging mode")
    ///     )
    ///     // Adding a single "option" argument with a short, a long, and help text using the less
    ///     // verbose Arg::from_usage()
    ///     .arg(
    ///         Arg::from_usage("-c --config=[CONFIG] 'Optionally sets a config file to use'")
    ///     )
    /// # ;
    /// ```
    /// [argument]: ./struct.Arg.html
    pub fn arg<A: Into<Arg<'a, 'b>>>(mut self, a: A) -> Self {
        self.p.add_arg(a.into());
        self
    }

    /// Adds multiple [arguments] to the list of valid possibilities
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .args(
    ///         &[Arg::from_usage("[debug] -d 'turns on debugging info'"),
    ///          Arg::with_name("input").index(1).help("the input file to use")]
    ///     )
    /// # ;
    /// ```
    /// [arguments]: ./struct.Arg.html
    pub fn args(mut self, args: &[Arg<'a, 'b>]) -> Self {
        for arg in args {
            self.p.add_arg_ref(arg);
        }
        self
    }

    /// A convenience method for adding a single [argument] from a usage type string. The string
    /// used follows the same rules and syntax as [`Arg::from_usage`]
    ///
    /// **NOTE:** The downside to using this method is that you can not set any additional
    /// properties of the [`Arg`] other than what [`Arg::from_usage`] supports.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .arg_from_usage("-c --config=<FILE> 'Sets a configuration file to use'")
    /// # ;
    /// ```
    /// [argument]: ./struct.Arg.html
    /// [`Arg`]: ./struct.Arg.html
    /// [`Arg::from_usage`]: ./struct.Arg.html#method.from_usage
    pub fn arg_from_usage(mut self, usage: &'a str) -> Self {
        self.p.add_arg(Arg::from_usage(usage));
        self
    }

    /// Adds multiple [arguments] at once from a usage string, one per line. See
    /// [`Arg::from_usage`] for details on the syntax and rules supported.
    ///
    /// **NOTE:** Like [`App::arg_from_usage`] the downside is you only set properties for the
    /// [`Arg`]s which [`Arg::from_usage`] supports.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// App::new("myprog")
    ///     .args_from_usage(
    ///         "-c --config=[FILE] 'Sets a configuration file to use'
    ///          [debug]... -d 'Sets the debugging level'
    ///          <FILE> 'The input file to use'"
    ///     )
    /// # ;
    /// ```
    /// [arguments]: ./struct.Arg.html
    /// [`Arg::from_usage`]: ./struct.Arg.html#method.from_usage
    /// [`App::arg_from_usage`]: ./struct.App.html#method.arg_from_usage
    /// [`Arg`]: ./struct.Arg.html
    pub fn args_from_usage(mut self, usage: &'a str) -> Self {
        for line in usage.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            self.p.add_arg(Arg::from_usage(l));
        }
        self
    }

    /// Allows adding a [`SubCommand`] alias, which function as "hidden" subcommands that
    /// automatically dispatch as if this subcommand was used. This is more efficient, and easier
    /// than creating multiple hidden subcommands as one only needs to check for the existence of
    /// this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    /// let m = App::new("myprog")
    ///             .subcommand(SubCommand::with_name("test")
    ///                 .alias("do-stuff"))
    ///             .get_matches_from(vec!["myprog", "do-stuff"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    pub fn alias<S: Into<&'b str>>(mut self, name: S) -> Self {
        if let Some(ref mut als) = self.p.meta.aliases {
            als.push((name.into(), false));
        } else {
            self.p.meta.aliases = Some(vec![(name.into(), false)]);
        }
        self
    }

    /// Allows adding [`SubCommand`] aliases, which function as "hidden" subcommands that
    /// automatically dispatch as if this subcommand was used. This is more efficient, and easier
    /// than creating multiple hidden subcommands as one only needs to check for the existence of
    /// this command, and not all variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, SubCommand};
    /// let m = App::new("myprog")
    ///             .subcommand(SubCommand::with_name("test")
    ///                 .aliases(&["do-stuff", "do-tests", "tests"]))
    ///                 .arg(Arg::with_name("input")
    ///                             .help("the file to add")
    ///                             .index(1)
    ///                             .required(false))
    ///             .get_matches_from(vec!["myprog", "do-tests"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    pub fn aliases(mut self, names: &[&'b str]) -> Self {
        if let Some(ref mut als) = self.p.meta.aliases {
            for n in names {
                als.push((n, false));
            }
        } else {
            self.p.meta.aliases = Some(names.iter().map(|n| (*n, false)).collect::<Vec<_>>());
        }
        self
    }

    /// Allows adding a [`SubCommand`] alias that functions exactly like those defined with
    /// [`App::alias`], except that they are visible inside the help message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    /// let m = App::new("myprog")
    ///             .subcommand(SubCommand::with_name("test")
    ///                 .visible_alias("do-stuff"))
    ///             .get_matches_from(vec!["myprog", "do-stuff"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`App::alias`]: ./struct.App.html#method.alias
    pub fn visible_alias<S: Into<&'b str>>(mut self, name: S) -> Self {
        if let Some(ref mut als) = self.p.meta.aliases {
            als.push((name.into(), true));
        } else {
            self.p.meta.aliases = Some(vec![(name.into(), true)]);
        }
        self
    }

    /// Allows adding multiple [`SubCommand`] aliases that functions exactly like those defined
    /// with [`App::aliases`], except that they are visible inside the help message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    /// let m = App::new("myprog")
    ///             .subcommand(SubCommand::with_name("test")
    ///                 .visible_aliases(&["do-stuff", "tests"]))
    ///             .get_matches_from(vec!["myprog", "do-stuff"]);
    /// assert_eq!(m.subcommand_name(), Some("test"));
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`App::aliases`]: ./struct.App.html#method.aliases
    pub fn visible_aliases(mut self, names: &[&'b str]) -> Self {
        if let Some(ref mut als) = self.p.meta.aliases {
            for n in names {
                als.push((n, true));
            }
        } else {
            self.p.meta.aliases = Some(names.iter().map(|n| (*n, true)).collect::<Vec<_>>());
        }
        self
    }

    /// Adds an [`ArgGroup`] to the application. [`ArgGroup`]s are a family of related arguments.
    /// By placing them in a logical group, you can build easier requirement and exclusion rules.
    /// For instance, you can make an entire [`ArgGroup`] required, meaning that one (and *only*
    /// one) argument from that group must be present at runtime.
    ///
    /// You can also do things such as name an [`ArgGroup`] as a conflict to another argument.
    /// Meaning any of the arguments that belong to that group will cause a failure if present with
    /// the conflicting argument.
    ///
    /// Another added benefit of [`ArgGroup`]s is that you can extract a value from a group instead
    /// of determining exactly which argument was used.
    ///
    /// Finally, using [`ArgGroup`]s to ensure exclusion between arguments is another very common
    /// use
    ///
    /// # Examples
    ///
    /// The following example demonstrates using an [`ArgGroup`] to ensure that one, and only one,
    /// of the arguments from the specified group is present at runtime.
    ///
    /// ```no_run
    /// # use clap::{App, ArgGroup};
    /// App::new("app")
    ///     .args_from_usage(
    ///         "--set-ver [ver] 'set the version manually'
    ///          --major         'auto increase major'
    ///          --minor         'auto increase minor'
    ///          --patch         'auto increase patch'")
    ///     .group(ArgGroup::with_name("vers")
    ///          .args(&["set-ver", "major", "minor","patch"])
    ///          .required(true))
    /// # ;
    /// ```
    /// [`ArgGroup`]: ./struct.ArgGroup.html
    pub fn group(mut self, group: ArgGroup<'a>) -> Self {
        self.p.add_group(group);
        self
    }

    /// Adds multiple [`ArgGroup`]s to the [`App`] at once.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, ArgGroup};
    /// App::new("app")
    ///     .args_from_usage(
    ///         "--set-ver [ver] 'set the version manually'
    ///          --major         'auto increase major'
    ///          --minor         'auto increase minor'
    ///          --patch         'auto increase patch'
    ///          -c [FILE]       'a config file'
    ///          -i [IFACE]      'an interface'")
    ///     .groups(&[
    ///         ArgGroup::with_name("vers")
    ///             .args(&["set-ver", "major", "minor","patch"])
    ///             .required(true),
    ///         ArgGroup::with_name("input")
    ///             .args(&["c", "i"])
    ///     ])
    /// # ;
    /// ```
    /// [`ArgGroup`]: ./struct.ArgGroup.html
    /// [`App`]: ./struct.App.html
    pub fn groups(mut self, groups: &[ArgGroup<'a>]) -> Self {
        for g in groups {
            self = self.group(g.into());
        }
        self
    }

    /// Adds a [`SubCommand`] to the list of valid possibilities. Subcommands are effectively
    /// sub-[`App`]s, because they can contain their own arguments, subcommands, version, usage,
    /// etc. They also function just like [`App`]s, in that they get their own auto generated help,
    /// version, and usage.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg, SubCommand};
    /// App::new("myprog")
    ///     .subcommand(SubCommand::with_name("config")
    ///         .about("Controls configuration features")
    ///         .arg_from_usage("<config> 'Required configuration file to use'"))
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`App`]: ./struct.App.html
    pub fn subcommand(mut self, subcmd: App<'a, 'b>) -> Self {
        self.p.add_subcommand(subcmd);
        self
    }

    /// Adds multiple subcommands to the list of valid possibilities by iterating over an
    /// [`IntoIterator`] of [`SubCommand`]s
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, SubCommand};
    /// # App::new("myprog")
    /// .subcommands( vec![
    ///        SubCommand::with_name("config").about("Controls configuration functionality")
    ///                                 .arg(Arg::with_name("config_file").index(1)),
    ///        SubCommand::with_name("debug").about("Controls debug functionality")])
    /// # ;
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    /// [`IntoIterator`]: https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
    pub fn subcommands<I>(mut self, subcmds: I) -> Self
    where
        I: IntoIterator<Item = App<'a, 'b>>,
    {
        for subcmd in subcmds {
            self.p.add_subcommand(subcmd);
        }
        self
    }

    /// Allows custom ordering of [`SubCommand`]s within the help message. Subcommands with a lower
    /// value will be displayed first in the help message. This is helpful when one would like to
    /// emphasise frequently used subcommands, or prioritize those towards the top of the list.
    /// Duplicate values **are** allowed. Subcommands with duplicate display orders will be
    /// displayed in alphabetical order.
    ///
    /// **NOTE:** The default is 999 for all subcommands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, SubCommand};
    /// let m = App::new("cust-ord")
    ///     .subcommand(SubCommand::with_name("alpha") // typically subcommands are grouped
    ///                                                // alphabetically by name. Subcommands
    ///                                                // without a display_order have a value of
    ///                                                // 999 and are displayed alphabetically with
    ///                                                // all other 999 subcommands
    ///         .about("Some help and text"))
    ///     .subcommand(SubCommand::with_name("beta")
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
    ///     cust-ord [FLAGS] [OPTIONS]
    ///
    /// FLAGS:
    ///     -h, --help       Prints help information
    ///     -V, --version    Prints version information
    ///
    /// SUBCOMMANDS:
    ///     beta    I should be first!
    ///     alpha   Some help and text
    /// ```
    /// [`SubCommand`]: ./struct.SubCommand.html
    pub fn display_order(mut self, ord: usize) -> Self {
        self.p.meta.disp_ord = ord;
        self
    }

    /// Prints the full help message to [`io::stdout()`] using a [`BufWriter`] using the same
    /// method as if someone ran `-h` to request the help message
    ///
    /// **NOTE:** clap has the ability to distinguish between "short" and "long" help messages
    /// depending on if the user ran [`-h` (short)] or [`--help` (long)]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::App;
    /// let mut app = App::new("myprog");
    /// app.print_help();
    /// ```
    /// [`io::stdout()`]: https://doc.rust-lang.org/std/io/fn.stdout.html
    /// [`BufWriter`]: https://doc.rust-lang.org/std/io/struct.BufWriter.html
    /// [`-h` (short)]: ./struct.Arg.html#method.help
    /// [`--help` (long)]: ./struct.Arg.html#method.long_help
    pub fn print_help(&mut self) -> ClapResult<()> {
        // If there are global arguments, or settings we need to propagate them down to subcommands
        // before parsing incase we run into a subcommand
        self.p.propagate_globals();
        self.p.propagate_settings();
        self.p.derive_display_order();

        self.p.create_help_and_version();
        let out = io::stdout();
        let mut buf_w = BufWriter::new(out.lock());
        self.write_help(&mut buf_w)
    }

    /// Prints the full help message to [`io::stdout()`] using a [`BufWriter`] using the same
    /// method as if someone ran `--help` to request the help message
    ///
    /// **NOTE:** clap has the ability to distinguish between "short" and "long" help messages
    /// depending on if the user ran [`-h` (short)] or [`--help` (long)]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::App;
    /// let mut app = App::new("myprog");
    /// app.print_long_help();
    /// ```
    /// [`io::stdout()`]: https://doc.rust-lang.org/std/io/fn.stdout.html
    /// [`BufWriter`]: https://doc.rust-lang.org/std/io/struct.BufWriter.html
    /// [`-h` (short)]: ./struct.Arg.html#method.help
    /// [`--help` (long)]: ./struct.Arg.html#method.long_help
    pub fn print_long_help(&mut self) -> ClapResult<()> {
        let out = io::stdout();
        let mut buf_w = BufWriter::new(out.lock());
        self.write_long_help(&mut buf_w)
    }

    /// Writes the full help message to the user to a [`io::Write`] object in the same method as if
    /// the user ran `-h`
    ///
    /// **NOTE:** clap has the ability to distinguish between "short" and "long" help messages
    /// depending on if the user ran [`-h` (short)] or [`--help` (long)]
    ///
    /// **NOTE:** There is a known bug where this method does not write propagated global arguments
    /// or autogenerated arguments (i.e. the default help/version args). Prefer
    /// [`App::write_long_help`] instead if possible!
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::App;
    /// use std::io;
    /// let mut app = App::new("myprog");
    /// let mut out = io::stdout();
    /// app.write_help(&mut out).expect("failed to write to stdout");
    /// ```
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    /// [`-h` (short)]: ./struct.Arg.html#method.help
    /// [`--help` (long)]: ./struct.Arg.html#method.long_help
    pub fn write_help<W: Write>(&self, w: &mut W) -> ClapResult<()> {
        // PENDING ISSUE: 808
        //      https://github.com/clap-rs/clap/issues/808
        // If there are global arguments, or settings we need to propagate them down to subcommands
        // before parsing incase we run into a subcommand
        // self.p.propagate_globals();
        // self.p.propagate_settings();
        // self.p.derive_display_order();
        // self.p.create_help_and_version();

        Help::write_app_help(w, self, false)
    }

    /// Writes the full help message to the user to a [`io::Write`] object in the same method as if
    /// the user ran `--help`
    ///
    /// **NOTE:** clap has the ability to distinguish between "short" and "long" help messages
    /// depending on if the user ran [`-h` (short)] or [`--help` (long)]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::App;
    /// use std::io;
    /// let mut app = App::new("myprog");
    /// let mut out = io::stdout();
    /// app.write_long_help(&mut out).expect("failed to write to stdout");
    /// ```
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    /// [`-h` (short)]: ./struct.Arg.html#method.help
    /// [`--help` (long)]: ./struct.Arg.html#method.long_help
    pub fn write_long_help<W: Write>(&mut self, w: &mut W) -> ClapResult<()> {
        // If there are global arguments, or settings we need to propagate them down to subcommands
        // before parsing incase we run into a subcommand
        self.p.propagate_globals();
        self.p.propagate_settings();
        self.p.derive_display_order();
        self.p.create_help_and_version();

        Help::write_app_help(w, self, true)
    }

    /// Writes the version message to the user to a [`io::Write`] object as if the user ran `-V`.
    ///
    /// **NOTE:** clap has the ability to distinguish between "short" and "long" version messages
    /// depending on if the user ran [`-V` (short)] or [`--version` (long)]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::App;
    /// use std::io;
    /// let mut app = App::new("myprog");
    /// let mut out = io::stdout();
    /// app.write_version(&mut out).expect("failed to write to stdout");
    /// ```
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    /// [`-V` (short)]: ./struct.App.html#method.version
    /// [`--version` (long)]: ./struct.App.html#method.long_version
    pub fn write_version<W: Write>(&self, w: &mut W) -> ClapResult<()> {
        self.p.write_version(w, false).map_err(From::from)
    }

    /// Writes the version message to the user to a [`io::Write`] object
    ///
    /// **NOTE:** clap has the ability to distinguish between "short" and "long" version messages
    /// depending on if the user ran [`-V` (short)] or [`--version` (long)]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::App;
    /// use std::io;
    /// let mut app = App::new("myprog");
    /// let mut out = io::stdout();
    /// app.write_long_version(&mut out).expect("failed to write to stdout");
    /// ```
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    /// [`-V` (short)]: ./struct.App.html#method.version
    /// [`--version` (long)]: ./struct.App.html#method.long_version
    pub fn write_long_version<W: Write>(&self, w: &mut W) -> ClapResult<()> {
        self.p.write_version(w, true).map_err(From::from)
    }

    /// Generate a completions file for a specified shell at compile time.
    ///
    /// **NOTE:** to generate the file at compile time you must use a `build.rs` "Build Script"
    ///
    /// # Examples
    ///
    /// The following example generates a bash completion script via a `build.rs` script. In this
    /// simple example, we'll demo a very small application with only a single subcommand and two
    /// args. Real applications could be many multiple levels deep in subcommands, and have tens or
    /// potentially hundreds of arguments.
    ///
    /// First, it helps if we separate out our `App` definition into a separate file. Whether you
    /// do this as a function, or bare App definition is a matter of personal preference.
    ///
    /// ```
    /// // src/cli.rs
    ///
    /// use clap::{App, Arg, SubCommand};
    ///
    /// pub fn build_cli() -> App<'static, 'static> {
    ///     App::new("compl")
    ///         .about("Tests completions")
    ///         .arg(Arg::with_name("file")
    ///             .help("some input file"))
    ///         .subcommand(SubCommand::with_name("test")
    ///             .about("tests things")
    ///             .arg(Arg::with_name("case")
    ///                 .long("case")
    ///                 .takes_value(true)
    ///                 .help("the case to test")))
    /// }
    /// ```
    ///
    /// In our regular code, we can simply call this `build_cli()` function, then call
    /// `get_matches()`, or any of the other normal methods directly after. For example:
    ///
    /// ```ignore
    /// // src/main.rs
    ///
    /// mod cli;
    ///
    /// fn main() {
    ///     let m = cli::build_cli().get_matches();
    ///
    ///     // normal logic continues...
    /// }
    /// ```
    ///
    /// Next, we set up our `Cargo.toml` to use a `build.rs` build script.
    ///
    /// ```toml
    /// # Cargo.toml
    /// build = "build.rs"
    ///
    /// [build-dependencies]
    /// clap = "2.23"
    /// ```
    ///
    /// Next, we place a `build.rs` in our project root.
    ///
    /// ```ignore
    /// extern crate clap;
    ///
    /// use clap::Shell;
    ///
    /// include!("src/cli.rs");
    ///
    /// fn main() {
    ///     let outdir = match env::var_os("OUT_DIR") {
    ///         None => return,
    ///         Some(outdir) => outdir,
    ///     };
    ///     let mut app = build_cli();
    ///     app.gen_completions("myapp",      // We need to specify the bin name manually
    ///                         Shell::Bash,  // Then say which shell to build completions for
    ///                         outdir);      // Then say where write the completions to
    /// }
    /// ```
    /// Now, once we compile there will be a `{bin_name}.bash` file in the directory.
    /// Assuming we compiled with debug mode, it would be somewhere similar to
    /// `<project>/target/debug/build/myapp-<hash>/out/myapp.bash`.
    ///
    /// Fish shell completions will use the file format `{bin_name}.fish`
    pub fn gen_completions<T: Into<OsString>, S: Into<String>>(
        &mut self,
        bin_name: S,
        for_shell: Shell,
        out_dir: T,
    ) {
        self.p.meta.bin_name = Some(bin_name.into());
        self.p.gen_completions(for_shell, out_dir.into());
    }

    /// Generate a completions file for a specified shell at runtime.  Until `cargo install` can
    /// install extra files like a completion script, this may be used e.g. in a command that
    /// outputs the contents of the completion script, to be redirected into a file by the user.
    ///
    /// # Examples
    ///
    /// Assuming a separate `cli.rs` like the [example above](./struct.App.html#method.gen_completions),
    /// we can let users generate a completion script using a command:
    ///
    /// ```ignore
    /// // src/main.rs
    ///
    /// mod cli;
    /// use std::io;
    ///
    /// fn main() {
    ///     let matches = cli::build_cli().get_matches();
    ///
    ///     if matches.is_present("generate-bash-completions") {
    ///         cli::build_cli().gen_completions_to("myapp", Shell::Bash, &mut io::stdout());
    ///     }
    ///
    ///     // normal logic continues...
    /// }
    ///
    /// ```
    ///
    /// Usage:
    ///
    /// ```shell
    /// $ myapp generate-bash-completions > /usr/share/bash-completion/completions/myapp.bash
    /// ```
    pub fn gen_completions_to<W: Write, S: Into<String>>(
        &mut self,
        bin_name: S,
        for_shell: Shell,
        buf: &mut W,
    ) {
        self.p.meta.bin_name = Some(bin_name.into());
        self.p.gen_completions_to(for_shell, buf);
    }

    /// Starts the parsing process, upon a failed parse an error will be displayed to the user and
    /// the process will exit with the appropriate error code. By default this method gets all user
    /// provided arguments from [`env::args_os`] in order to allow for invalid UTF-8 code points,
    /// which are legal on many platforms.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let matches = App::new("myprog")
    ///     // Args and options go here...
    ///     .get_matches();
    /// ```
    /// [`env::args_os`]: https://doc.rust-lang.org/std/env/fn.args_os.html
    pub fn get_matches(self) -> ArgMatches<'a> {
        self.get_matches_from(&mut env::args_os())
    }

    /// Starts the parsing process. This method will return a [`clap::Result`] type instead of exiting
    /// the process on failed parse. By default this method gets matches from [`env::args_os`]
    ///
    /// **NOTE:** This method WILL NOT exit when `--help` or `--version` (or short versions) are
    /// used. It will return a [`clap::Error`], where the [`kind`] is a
    /// [`ErrorKind::HelpDisplayed`] or [`ErrorKind::VersionDisplayed`] respectively. You must call
    /// [`Error::exit`] or perform a [`std::process::exit`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let matches = App::new("myprog")
    ///     // Args and options go here...
    ///     .get_matches_safe()
    ///     .unwrap_or_else( |e| e.exit() );
    /// ```
    /// [`env::args_os`]: https://doc.rust-lang.org/std/env/fn.args_os.html
    /// [`ErrorKind::HelpDisplayed`]: ./enum.ErrorKind.html#variant.HelpDisplayed
    /// [`ErrorKind::VersionDisplayed`]: ./enum.ErrorKind.html#variant.VersionDisplayed
    /// [`Error::exit`]: ./struct.Error.html#method.exit
    /// [`std::process::exit`]: https://doc.rust-lang.org/std/process/fn.exit.html
    /// [`clap::Result`]: ./type.Result.html
    /// [`clap::Error`]: ./struct.Error.html
    /// [`kind`]: ./struct.Error.html
    pub fn get_matches_safe(self) -> ClapResult<ArgMatches<'a>> {
        // Start the parsing
        self.get_matches_from_safe(&mut env::args_os())
    }

    /// Starts the parsing process. Like [`App::get_matches`] this method does not return a [`clap::Result`]
    /// and will automatically exit with an error message. This method, however, lets you specify
    /// what iterator to use when performing matches, such as a [`Vec`] of your making.
    ///
    /// **NOTE:** The first argument will be parsed as the binary name unless
    /// [`AppSettings::NoBinaryName`] is used
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let arg_vec = vec!["my_prog", "some", "args", "to", "parse"];
    ///
    /// let matches = App::new("myprog")
    ///     // Args and options go here...
    ///     .get_matches_from(arg_vec);
    /// ```
    /// [`App::get_matches`]: ./struct.App.html#method.get_matches
    /// [`clap::Result`]: ./type.Result.html
    /// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
    /// [`AppSettings::NoBinaryName`]: ./enum.AppSettings.html#variant.NoBinaryName
    pub fn get_matches_from<I, T>(mut self, itr: I) -> ArgMatches<'a>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        self.get_matches_from_safe_borrow(itr).unwrap_or_else(|e| {
            // Otherwise, write to stderr and exit
            if e.use_stderr() {
                wlnerr!("{}", e.message);
                if self.p.is_set(AppSettings::WaitOnError) {
                    wlnerr!("\nPress [ENTER] / [RETURN] to continue...");
                    let mut s = String::new();
                    let i = io::stdin();
                    i.lock().read_line(&mut s).unwrap();
                }
                drop(self);
                drop(e);
                process::exit(1);
            }

            drop(self);
            e.exit()
        })
    }

    /// Starts the parsing process. A combination of [`App::get_matches_from`], and
    /// [`App::get_matches_safe`]
    ///
    /// **NOTE:** This method WILL NOT exit when `--help` or `--version` (or short versions) are
    /// used. It will return a [`clap::Error`], where the [`kind`] is a [`ErrorKind::HelpDisplayed`]
    /// or [`ErrorKind::VersionDisplayed`] respectively. You must call [`Error::exit`] or
    /// perform a [`std::process::exit`] yourself.
    ///
    /// **NOTE:** The first argument will be parsed as the binary name unless
    /// [`AppSettings::NoBinaryName`] is used
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let arg_vec = vec!["my_prog", "some", "args", "to", "parse"];
    ///
    /// let matches = App::new("myprog")
    ///     // Args and options go here...
    ///     .get_matches_from_safe(arg_vec)
    ///     .unwrap_or_else( |e| { panic!("An error occurs: {}", e) });
    /// ```
    /// [`App::get_matches_from`]: ./struct.App.html#method.get_matches_from
    /// [`App::get_matches_safe`]: ./struct.App.html#method.get_matches_safe
    /// [`ErrorKind::HelpDisplayed`]: ./enum.ErrorKind.html#variant.HelpDisplayed
    /// [`ErrorKind::VersionDisplayed`]: ./enum.ErrorKind.html#variant.VersionDisplayed
    /// [`Error::exit`]: ./struct.Error.html#method.exit
    /// [`std::process::exit`]: https://doc.rust-lang.org/std/process/fn.exit.html
    /// [`clap::Error`]: ./struct.Error.html
    /// [`Error::exit`]: ./struct.Error.html#method.exit
    /// [`kind`]: ./struct.Error.html
    /// [`AppSettings::NoBinaryName`]: ./enum.AppSettings.html#variant.NoBinaryName
    pub fn get_matches_from_safe<I, T>(mut self, itr: I) -> ClapResult<ArgMatches<'a>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        self.get_matches_from_safe_borrow(itr)
    }

    /// Starts the parsing process without consuming the [`App`] struct `self`. This is normally not
    /// the desired functionality, instead prefer [`App::get_matches_from_safe`] which *does*
    /// consume `self`.
    ///
    /// **NOTE:** The first argument will be parsed as the binary name unless
    /// [`AppSettings::NoBinaryName`] is used
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::{App, Arg};
    /// let arg_vec = vec!["my_prog", "some", "args", "to", "parse"];
    ///
    /// let mut app = App::new("myprog");
    ///     // Args and options go here...
    /// let matches = app.get_matches_from_safe_borrow(arg_vec)
    ///     .unwrap_or_else( |e| { panic!("An error occurs: {}", e) });
    /// ```
    /// [`App`]: ./struct.App.html
    /// [`App::get_matches_from_safe`]: ./struct.App.html#method.get_matches_from_safe
    /// [`AppSettings::NoBinaryName`]: ./enum.AppSettings.html#variant.NoBinaryName
    pub fn get_matches_from_safe_borrow<I, T>(&mut self, itr: I) -> ClapResult<ArgMatches<'a>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        // If there are global arguments, or settings we need to propagate them down to subcommands
        // before parsing incase we run into a subcommand
        if !self.p.is_set(AppSettings::Propagated) {
            self.p.propagate_globals();
            self.p.propagate_settings();
            self.p.derive_display_order();
            self.p.set(AppSettings::Propagated);
        }

        let mut matcher = ArgMatcher::new();

        let mut it = itr.into_iter();
        // Get the name of the program (argument 1 of env::args()) and determine the
        // actual file
        // that was used to execute the program. This is because a program called
        // ./target/release/my_prog -a
        // will have two arguments, './target/release/my_prog', '-a' but we don't want
        // to display
        // the full path when displaying help messages and such
        if !self.p.is_set(AppSettings::NoBinaryName) {
            if let Some(name) = it.next() {
                let bn_os = name.into();
                let p = Path::new(&*bn_os);
                if let Some(f) = p.file_name() {
                    if let Some(s) = f.to_os_string().to_str() {
                        if self.p.meta.bin_name.is_none() {
                            self.p.meta.bin_name = Some(s.to_owned());
                        }
                    }
                }
            }
        }

        // do the real parsing
        if let Err(e) = self.p.get_matches_with(&mut matcher, &mut it.peekable()) {
            return Err(e);
        }

        let global_arg_vec: Vec<&str> = self.p.global_args.iter().map(|ga| ga.b.name).collect();
        matcher.propagate_globals(&global_arg_vec);

        Ok(matcher.into())
    }
}

#[cfg(feature = "yaml")]
impl<'a> From<&'a Yaml> for App<'a, 'a> {
    fn from(mut yaml: &'a Yaml) -> Self {
        use crate::args::SubCommand;
        // We WANT this to panic on error...so expect() is good.
        let mut is_sc = None;
        let mut a = if let Some(name) = yaml["name"].as_str() {
            App::new(name)
        } else {
            let yaml_hash = yaml.as_hash().unwrap();
            let sc_key = yaml_hash.keys().nth(0).unwrap();
            is_sc = Some(yaml_hash.get(sc_key).unwrap());
            App::new(sc_key.as_str().unwrap())
        };
        yaml = if let Some(sc) = is_sc { sc } else { yaml };

        macro_rules! yaml_str {
            ($a:ident, $y:ident, $i:ident) => {
                if let Some(v) = $y[stringify!($i)].as_str() {
                    $a = $a.$i(v);
                } else if $y[stringify!($i)] != Yaml::BadValue {
                    panic!(
                        "Failed to convert YAML value {:?} to a string",
                        $y[stringify!($i)]
                    );
                }
            };
        }

        yaml_str!(a, yaml, version);
        yaml_str!(a, yaml, long_version);
        yaml_str!(a, yaml, author);
        yaml_str!(a, yaml, bin_name);
        yaml_str!(a, yaml, about);
        yaml_str!(a, yaml, long_about);
        yaml_str!(a, yaml, before_help);
        yaml_str!(a, yaml, after_help);
        yaml_str!(a, yaml, template);
        yaml_str!(a, yaml, usage);
        yaml_str!(a, yaml, help);
        yaml_str!(a, yaml, help_short);
        yaml_str!(a, yaml, version_short);
        yaml_str!(a, yaml, help_message);
        yaml_str!(a, yaml, version_message);
        yaml_str!(a, yaml, alias);
        yaml_str!(a, yaml, visible_alias);

        if let Some(v) = yaml["display_order"].as_i64() {
            a = a.display_order(v as usize);
        } else if yaml["display_order"] != Yaml::BadValue {
            panic!(
                "Failed to convert YAML value {:?} to a u64",
                yaml["display_order"]
            );
        }
        if let Some(v) = yaml["setting"].as_str() {
            a = a.setting(v.parse().expect("unknown AppSetting found in YAML file"));
        } else if yaml["setting"] != Yaml::BadValue {
            panic!(
                "Failed to convert YAML value {:?} to an AppSetting",
                yaml["setting"]
            );
        }
        if let Some(v) = yaml["settings"].as_vec() {
            for ys in v {
                if let Some(s) = ys.as_str() {
                    a = a.setting(s.parse().expect("unknown AppSetting found in YAML file"));
                }
            }
        } else if let Some(v) = yaml["settings"].as_str() {
            a = a.setting(v.parse().expect("unknown AppSetting found in YAML file"));
        } else if yaml["settings"] != Yaml::BadValue {
            panic!(
                "Failed to convert YAML value {:?} to a string",
                yaml["settings"]
            );
        }
        if let Some(v) = yaml["global_setting"].as_str() {
            a = a.setting(v.parse().expect("unknown AppSetting found in YAML file"));
        } else if yaml["global_setting"] != Yaml::BadValue {
            panic!(
                "Failed to convert YAML value {:?} to an AppSetting",
                yaml["setting"]
            );
        }
        if let Some(v) = yaml["global_settings"].as_vec() {
            for ys in v {
                if let Some(s) = ys.as_str() {
                    a = a.global_setting(s.parse().expect("unknown AppSetting found in YAML file"));
                }
            }
        } else if let Some(v) = yaml["global_settings"].as_str() {
            a = a.global_setting(v.parse().expect("unknown AppSetting found in YAML file"));
        } else if yaml["global_settings"] != Yaml::BadValue {
            panic!(
                "Failed to convert YAML value {:?} to a string",
                yaml["global_settings"]
            );
        }

        macro_rules! vec_or_str {
            ($a:ident, $y:ident, $as_vec:ident, $as_single:ident) => {{
                let maybe_vec = $y[stringify!($as_vec)].as_vec();
                if let Some(vec) = maybe_vec {
                    for ys in vec {
                        if let Some(s) = ys.as_str() {
                            $a = $a.$as_single(s);
                        } else {
                            panic!("Failed to convert YAML value {:?} to a string", ys);
                        }
                    }
                } else {
                    if let Some(s) = $y[stringify!($as_vec)].as_str() {
                        $a = $a.$as_single(s);
                    } else if $y[stringify!($as_vec)] != Yaml::BadValue {
                        panic!(
                            "Failed to convert YAML value {:?} to either a vec or string",
                            $y[stringify!($as_vec)]
                        );
                    }
                }
                $a
            }};
        }

        a = vec_or_str!(a, yaml, aliases, alias);
        a = vec_or_str!(a, yaml, visible_aliases, visible_alias);

        if let Some(v) = yaml["args"].as_vec() {
            for arg_yaml in v {
                a = a.arg(Arg::from_yaml(arg_yaml.as_hash().unwrap()));
            }
        }
        if let Some(v) = yaml["subcommands"].as_vec() {
            for sc_yaml in v {
                a = a.subcommand(SubCommand::from_yaml(sc_yaml));
            }
        }
        if let Some(v) = yaml["groups"].as_vec() {
            for ag_yaml in v {
                a = a.group(ArgGroup::from(ag_yaml.as_hash().unwrap()));
            }
        }

        a
    }
}

impl<'a, 'b> Clone for App<'a, 'b> {
    fn clone(&self) -> Self {
        App { p: self.p.clone() }
    }
}

impl<'n, 'e> AnyArg<'n, 'e> for App<'n, 'e> {
    fn name(&self) -> &'n str {
        ""
    }
    fn overrides(&self) -> Option<&[&'e str]> {
        None
    }
    fn requires(&self) -> Option<&[(Option<&'e str>, &'n str)]> {
        None
    }
    fn blacklist(&self) -> Option<&[&'e str]> {
        None
    }
    fn required_unless(&self) -> Option<&[&'e str]> {
        None
    }
    fn val_names(&self) -> Option<&VecMap<&'e str>> {
        None
    }
    fn is_set(&self, _: ArgSettings) -> bool {
        false
    }
    fn val_terminator(&self) -> Option<&'e str> {
        None
    }
    fn set(&mut self, _: ArgSettings) {
        unreachable!("App struct does not support AnyArg::set, this is a bug!")
    }
    fn has_switch(&self) -> bool {
        false
    }
    fn max_vals(&self) -> Option<u64> {
        None
    }
    fn num_vals(&self) -> Option<u64> {
        None
    }
    fn possible_vals(&self) -> Option<&[&'e str]> {
        None
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator(&self) -> Option<&Rc<Fn(String) -> StdResult<(), String>>> {
        None
    }
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::type_complexity))]
    fn validator_os(&self) -> Option<&Rc<Fn(&OsStr) -> StdResult<(), OsString>>> {
        None
    }
    fn min_vals(&self) -> Option<u64> {
        None
    }
    fn short(&self) -> Option<char> {
        None
    }
    fn long(&self) -> Option<&'e str> {
        None
    }
    fn val_delim(&self) -> Option<char> {
        None
    }
    fn takes_value(&self) -> bool {
        true
    }
    fn help(&self) -> Option<&'e str> {
        self.p.meta.about
    }
    fn long_help(&self) -> Option<&'e str> {
        self.p.meta.long_about
    }
    fn default_val(&self) -> Option<&'e OsStr> {
        None
    }
    fn default_vals_ifs(&self) -> Option<map::Values<(&'n str, Option<&'e OsStr>, &'e OsStr)>> {
        None
    }
    fn env<'s>(&'s self) -> Option<(&'n OsStr, Option<&'s OsString>)> {
        None
    }
    fn longest_filter(&self) -> bool {
        true
    }
    fn aliases(&self) -> Option<Vec<&'e str>> {
        if let Some(ref aliases) = self.p.meta.aliases {
            let vis_aliases: Vec<_> = aliases
                .iter()
                .filter_map(|&(n, v)| if v { Some(n) } else { None })
                .collect();
            if vis_aliases.is_empty() {
                None
            } else {
                Some(vis_aliases)
            }
        } else {
            None
        }
    }
}

impl<'n, 'e> fmt::Display for App<'n, 'e> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.p.meta.name)
    }
}
