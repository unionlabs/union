use clap::{CommandFactory, ErrorKind, Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// set version manually
    #[clap(long, value_name = "VER", value_parser)]
    set_ver: Option<String>,

    /// auto inc major
    #[clap(long, action)]
    major: bool,

    /// auto inc minor
    #[clap(long, action)]
    minor: bool,

    /// auto inc patch
    #[clap(long, action)]
    patch: bool,

    /// some regular input
    #[clap(value_parser)]
    input_file: Option<String>,

    /// some special input argument
    #[clap(long, value_parser)]
    spec_in: Option<String>,

    #[clap(short, value_parser)]
    config: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Let's assume the old version 1.2.3
    let mut major = 1;
    let mut minor = 2;
    let mut patch = 3;

    // See if --set-ver was used to set the version manually
    let version = if let Some(ver) = cli.set_ver.as_deref() {
        if cli.major || cli.minor || cli.patch {
            let mut cmd = Cli::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "Can't do relative and absolute version change",
            )
            .exit();
        }
        ver.to_string()
    } else {
        // Increment the one requested (in a real program, we'd reset the lower numbers)
        let (maj, min, pat) = (cli.major, cli.minor, cli.patch);
        match (maj, min, pat) {
            (true, false, false) => major += 1,
            (false, true, false) => minor += 1,
            (false, false, true) => patch += 1,
            _ => {
                let mut cmd = Cli::command();
                cmd.error(
                    ErrorKind::ArgumentConflict,
                    "Can only modify one version field",
                )
                .exit();
            }
        };
        format!("{}.{}.{}", major, minor, patch)
    };

    println!("Version: {}", version);

    // Check for usage of -c
    if let Some(config) = cli.config.as_deref() {
        // todo: remove `#[allow(clippy::or_fun_call)]` lint when MSRV is bumped.
        #[allow(clippy::or_fun_call)]
        let input = cli
            .input_file
            .as_deref()
            // 'or' is preferred to 'or_else' here since `Option::as_deref` is 'const'
            .or(cli.spec_in.as_deref())
            .unwrap_or_else(|| {
                let mut cmd = Cli::command();
                cmd.error(
                    ErrorKind::MissingRequiredArgument,
                    "INPUT_FILE or --spec-in is required when using --config",
                )
                .exit()
            });
        println!("Doing work using input {} and config {}", input, config);
    }
}
