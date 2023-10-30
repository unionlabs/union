```console
$ 04_01_possible --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_01_possible[EXE] <MODE>

ARGS:
    <MODE>    What mode to run the program in [possible values: fast, slow]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_01_possible fast
Hare

$ 04_01_possible slow
Tortoise

$ 04_01_possible medium
? failed
error: "medium" isn't a valid value for '<MODE>'
	[possible values: fast, slow]

For more information try --help

```
