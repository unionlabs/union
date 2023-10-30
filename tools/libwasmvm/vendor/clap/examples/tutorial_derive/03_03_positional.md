```console
$ 03_03_positional_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_03_positional_derive[EXE] [NAME]

ARGS:
    <NAME>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_03_positional_derive
name: None

$ 03_03_positional_derive bob
name: Some("bob")

```
