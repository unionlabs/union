*Jump to [source](custom-bool.rs)*

Example of overriding the magic `bool` behavior

```console
$ custom-bool --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    custom-bool[EXE] [OPTIONS] --foo <FOO> <BOOM>

ARGS:
    <BOOM>    [possible values: true, false]

OPTIONS:
        --bar <BAR>    [default: false]
        --foo <FOO>    [possible values: true, false]
    -h, --help         Print help information
    -V, --version      Print version information

$ custom-bool
? failed
error: The following required arguments were not provided:
    --foo <FOO>
    <BOOM>

USAGE:
    custom-bool[EXE] [OPTIONS] --foo <FOO> <BOOM>

For more information try --help

$ custom-bool --foo true false
[examples/derive_ref/custom-bool.rs:31] opt = Opt {
    foo: true,
    bar: false,
    boom: false,
}

$ custom-bool --foo true --bar true false
[examples/derive_ref/custom-bool.rs:31] opt = Opt {
    foo: true,
    bar: true,
    boom: false,
}

```
