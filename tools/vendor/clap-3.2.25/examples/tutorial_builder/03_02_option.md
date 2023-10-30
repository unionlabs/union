```console
$ 03_02_option --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_02_option[EXE] [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    
    -V, --version        Print version information

$ 03_02_option
name: None

$ 03_02_option --name bob
name: Some("bob")

$ 03_02_option --name=bob
name: Some("bob")

$ 03_02_option -n bob
name: Some("bob")

$ 03_02_option -n=bob
name: Some("bob")

$ 03_02_option -nbob
name: Some("bob")

```
