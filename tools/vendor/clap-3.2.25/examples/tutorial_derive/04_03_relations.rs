use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
            ArgGroup::new("vers")
                .required(true)
                .args(&["set-ver", "major", "minor", "patch"]),
        ))]
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
    #[clap(group = "input", value_parser)]
    input_file: Option<String>,

    /// some special input argument
    #[clap(long, group = "input", value_parser)]
    spec_in: Option<String>,

    #[clap(short, requires = "input", value_parser)]
    config: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Let's assume the old version 1.2.3
    let mut major = 1;
    let mut minor = 2;
    let mut patch = 3;

    // See if --set_ver was used to set the version manually
    let version = if let Some(ver) = cli.set_ver.as_deref() {
        ver.to_string()
    } else {
        // Increment the one requested (in a real program, we'd reset the lower numbers)
        let (maj, min, pat) = (cli.major, cli.minor, cli.patch);
        match (maj, min, pat) {
            (true, _, _) => major += 1,
            (_, true, _) => minor += 1,
            (_, _, true) => patch += 1,
            _ => unreachable!(),
        };
        format!("{}.{}.{}", major, minor, patch)
    };

    println!("Version: {}", version);

    // Check for usage of -c
    if let Some(config) = cli.config.as_deref() {
        let input = cli
            .input_file
            .as_deref()
            .unwrap_or_else(|| cli.spec_in.as_deref().unwrap());
        println!("Doing work using input {} and config {}", input, config);
    }
}
