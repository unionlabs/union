//! How to use environment variable fallback an how it
//! interacts with `default_value`.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! env 0.3.25
//! Example for allowing to specify options via environment variables
//!
//! USAGE:
//!     env [OPTIONS] --api-url <api-url>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!         --api-url <api-url>    URL for the API server [env: API_URL=]
//!         --retries <retries>    Number of retries [env: RETRIES=]  [default: 5]
//! -----------------------------------------------------

use structopt::StructOpt;

/// Example for allowing to specify options via environment variables.
#[derive(StructOpt, Debug)]
#[structopt(name = "env")]
struct Opt {
    // Use `env` to enable specifying the option with an environment
    // variable. Command line arguments take precedence over env.
    /// URL for the API server
    #[structopt(long, env = "API_URL")]
    api_url: String,

    // The default value is used if neither argument nor environment
    // variable is specified.
    /// Number of retries
    #[structopt(long, env = "RETRIES", default_value = "5")]
    retries: u32,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
}
