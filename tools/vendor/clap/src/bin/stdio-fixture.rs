fn main() {
    let cmd = clap::Command::new("stdio-fixture")
        .version("1.0")
        .long_version("1.0 - a2132c")
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("more"))
        .arg(
            clap::Arg::new("verbose")
                .long("verbose")
                .help("log")
                .action(clap::ArgAction::SetTrue)
                .long_help("more log"),
        );
    cmd.get_matches();
}
