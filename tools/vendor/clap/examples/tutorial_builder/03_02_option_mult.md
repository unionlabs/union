```console
$ 03_02_option_mult --help
A simple to use, efficient, and full-featured Command Line Argument Parser

Usage: 03_02_option_mult[EXE] [OPTIONS]

Options:
  -n, --name <name>  
  -h, --help         Print help
  -V, --version      Print version

$ 03_02_option_mult
name: None

$ 03_02_option_mult --name bob
name: Some("bob")

$ 03_02_option_mult --name=bob
name: Some("bob")

$ 03_02_option_mult -n bob
name: Some("bob")

$ 03_02_option_mult -n=bob
name: Some("bob")

$ 03_02_option_mult -nbob
name: Some("bob")

```
