```console
$ 03_02_option_derive --help
A simple to use, efficient, and full-featured Command Line Argument Parser

Usage: 03_02_option_derive[EXE] [OPTIONS]

Options:
  -n, --name <NAME>  
  -h, --help         Print help
  -V, --version      Print version

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
