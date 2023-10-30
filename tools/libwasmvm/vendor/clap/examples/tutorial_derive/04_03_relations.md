```console
$ 04_03_relations_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_03_relations_derive[EXE] [OPTIONS] <--set-ver <VER>|--major|--minor|--patch> [INPUT_FILE]

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

$ 04_03_relations_derive
? failed
error: The following required arguments were not provided:
    <--set-ver <VER>|--major|--minor|--patch>

USAGE:
    04_03_relations_derive[EXE] [OPTIONS] <--set-ver <VER>|--major|--minor|--patch> [INPUT_FILE]

For more information try --help

$ 04_03_relations_derive --major
Version: 2.2.3

$ 04_03_relations_derive --major --minor
? failed
error: The argument '--major' cannot be used with '--minor'

USAGE:
    04_03_relations_derive[EXE] <--set-ver <VER>|--major|--minor|--patch>

For more information try --help

$ 04_03_relations_derive --major -c config.toml
? failed
error: The following required arguments were not provided:
    <INPUT_FILE|--spec-in <SPEC_IN>>

USAGE:
    04_03_relations_derive[EXE] -c <CONFIG> <--set-ver <VER>|--major|--minor|--patch> <INPUT_FILE|--spec-in <SPEC_IN>>

For more information try --help

$ 04_03_relations_derive --major -c config.toml --spec-in input.txt
Version: 2.2.3
Doing work using input input.txt and config config.toml

```
