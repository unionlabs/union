```console
$ 03_04_subcommands help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_04_subcommands[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add     Adds files to myapp
    help    Print this message or the help of the given subcommand(s)

$ 03_04_subcommands help add
03_04_subcommands[EXE]-add [..]
Adds files to myapp

USAGE:
    03_04_subcommands[EXE] add [NAME]

ARGS:
    <NAME>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_04_subcommands add bob
'myapp add' was used, name is: Some("bob")

```

Because we set [`Command::arg_required_else_help`][crate::Command::arg_required_else_help]:
```console
$ 03_04_subcommands
? failed
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_04_subcommands[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add     Adds files to myapp
    help    Print this message or the help of the given subcommand(s)

```

Because we set [`Command::propagate_version`][crate::Command::propagate_version]:
```console
$ 03_04_subcommands --version
clap [..]

$ 03_04_subcommands add --version
03_04_subcommands[EXE]-add [..]

```
