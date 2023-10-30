```console
$ 02_app_settings --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    02_app_settings[EXE] --two <VALUE> --one <VALUE>

OPTIONS:
        --two <VALUE>    
        --one <VALUE>    
    -h, --help           Print help information
    -V, --version        Print version information

$ 02_app_settings --one -1 --one -3 --two 10
two: "10"
one: "-3"

```
