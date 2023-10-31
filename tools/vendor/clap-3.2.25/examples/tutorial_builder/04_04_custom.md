```console
$ 04_04_custom --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

ARGS:
    <INPUT_FILE>    some regular input

OPTIONS:
    -c <CONFIG>                
    -h, --help                 Print help information
        --major                auto inc major
        --minor                auto inc minor
        --patch                auto inc patch
        --set-ver <VER>        set version manually
        --spec-in <SPEC_IN>    some special input argument
    -V, --version              Print version information

$ 04_04_custom
? failed
error: Can only modify one version field

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom --major
Version: 2.2.3

$ 04_04_custom --major --minor
? failed
error: Can only modify one version field

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom --major -c config.toml
? failed
Version: 2.2.3
error: INPUT_FILE or --spec-in is required when using --config

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom --major -c config.toml --spec-in input.txt
Version: 2.2.3
Doing work using input input.txt and config config.toml

```
