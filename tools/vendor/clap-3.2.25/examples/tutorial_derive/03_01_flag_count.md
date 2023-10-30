```console
$ 03_01_flag_count_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_01_flag_count_derive[EXE] [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    
    -V, --version    Print version information

$ 03_01_flag_count_derive
verbose: 0

$ 03_01_flag_count_derive --verbose
verbose: 1

$ 03_01_flag_count_derive --verbose --verbose
verbose: 2

```
