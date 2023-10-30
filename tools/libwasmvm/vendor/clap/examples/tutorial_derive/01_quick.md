```console
$ 01_quick_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    01_quick_derive[EXE] [OPTIONS] [NAME] [SUBCOMMAND]

ARGS:
    <NAME>    Optional name to operate on

OPTIONS:
    -c, --config <FILE>    Sets a custom config file
    -d, --debug            Turn debugging information on
    -h, --help             Print help information
    -V, --version          Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    test    does testing things

```

By default, the program does nothing:
```console
$ 01_quick_derive
Debug mode is off

```

But you can mix and match the various features
```console
$ 01_quick_derive -dd test
Debug mode is on
Not printing testing lists...

```
