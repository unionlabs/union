// Third Party
#[cfg(feature = "yaml")]
use yaml_rust::Yaml;

// Internal
use crate::{App, ArgMatches};

/// The abstract representation of a command line subcommand.
///
/// This struct describes all the valid options of the subcommand for the program. Subcommands are
/// essentially "sub-[`App`]s" and contain all the same possibilities (such as their own
/// [arguments], subcommands, and settings).
///
/// # Examples
///
/// ```rust
/// # use clap::{App, Arg, SubCommand};
/// App::new("myprog")
///     .subcommand(
///         SubCommand::with_name("config")
///             .about("Used for configuration")
///             .arg(Arg::with_name("config_file")
///                 .help("The configuration file to use")
///                 .index(1)))
/// # ;
/// ```
/// [`App`]: ./struct.App.html
/// [arguments]: ./struct.Arg.html
#[derive(Debug, Clone)]
pub struct SubCommand<'a> {
    #[doc(hidden)]
    pub name: String,
    #[doc(hidden)]
    pub matches: ArgMatches<'a>,
}

impl<'a> SubCommand<'a> {
    /// Creates a new instance of a subcommand requiring a name. The name will be displayed
    /// to the user when they print version or help and usage information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use clap::{App, Arg, SubCommand};
    /// App::new("myprog")
    ///     .subcommand(
    ///         SubCommand::with_name("config"))
    /// # ;
    /// ```
    pub fn with_name<'b>(name: &str) -> App<'a, 'b> {
        App::new(name)
    }

    /// Creates a new instance of a subcommand from a YAML (.yml) document
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #[macro_use]
    /// # extern crate clap;
    /// # use clap::Subcommand;
    /// # fn main() {
    /// let sc_yaml = load_yaml!("test_subcommand.yml");
    /// let sc = SubCommand::from_yaml(sc_yaml);
    /// # }
    /// ```
    #[cfg(feature = "yaml")]
    pub fn from_yaml(yaml: &Yaml) -> App {
        App::from_yaml(yaml)
    }
}
