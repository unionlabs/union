```console
$ 03_05_default_values_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_05_default_values_derive[EXE] [NAME]

ARGS:
    <NAME>    [default: alice]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_05_default_values_derive
name: "alice"

$ 03_05_default_values_derive bob
name: "bob"

```
