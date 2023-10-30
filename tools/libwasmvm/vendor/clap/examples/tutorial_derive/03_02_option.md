```console
$ 03_02_option_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_02_option_derive[EXE] [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    
    -V, --version        Print version information

$ 03_02_option_derive
name: None

$ 03_02_option_derive --name bob
name: Some("bob")

$ 03_02_option_derive --name=bob
name: Some("bob")

$ 03_02_option_derive -n bob
name: Some("bob")

$ 03_02_option_derive -n=bob
name: Some("bob")

$ 03_02_option_derive -nbob
name: Some("bob")

```
