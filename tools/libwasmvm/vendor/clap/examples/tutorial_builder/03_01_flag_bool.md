```console
$ 03_01_flag_bool --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_01_flag_bool[EXE] [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    
    -V, --version    Print version information

$ 03_01_flag_bool
verbose: false

$ 03_01_flag_bool --verbose
verbose: true

$ 03_01_flag_bool --verbose --verbose
verbose: true

```
