*Jump to [source](cargo-example.rs)*

For more on creating a custom subcommand, see [the cargo
book](https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands).
The crate [`clap-cargo`](https://github.com/crate-ci/clap-cargo) can help in
mimicking cargo's interface.

The help looks like:
```console
$ cargo-example --help
cargo 

USAGE:
    cargo <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    example    A simple to use, efficient, and full-featured Command Line Argument Parser
    help       Print this message or the help of the given subcommand(s)

$ cargo-example example --help
cargo-example [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    cargo example [OPTIONS]

OPTIONS:
    -h, --help                    Print help information
        --manifest-path <PATH>    
    -V, --version                 Print version information

```

Then to directly invoke the command, run:
```console
$ cargo-example example
None

$ cargo-example example --manifest-path Cargo.toml
Some("Cargo.toml")

```
