```console
$ 04_02_validate_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_02_validate_derive[EXE] <PORT>

ARGS:
    <PORT>    Network port to use

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_02_validate_derive 22
PORT = 22

$ 04_02_validate_derive foobar
? failed
error: Invalid value "foobar" for '<PORT>': `foobar` isn't a port number

For more information try --help

$ 04_02_validate_derive 0
? failed
error: Invalid value "0" for '<PORT>': Port not in range 1-65535

For more information try --help

```
