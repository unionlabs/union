```console
$ 03_01_flag_bool_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_01_flag_bool_derive[EXE] [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    
    -V, --version    Print version information

$ 03_01_flag_bool_derive
verbose: false

$ 03_01_flag_bool_derive --verbose
verbose: true

$ 03_01_flag_bool_derive --verbose --verbose
verbose: true

```
