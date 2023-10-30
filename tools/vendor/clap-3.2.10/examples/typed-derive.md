*Jump to [source](typed-derive.rs)*

**This requires enabling the `derive` feature flag.**

Help:
```console
$ typed-derive --help
clap 

USAGE:
    typed-derive[EXE] [OPTIONS]

OPTIONS:
        --bind <BIND>        Handle IP addresses
    -D <DEFINES>             Hand-written parser for tuples
    -h, --help               Print help information
    -I <DIR>                 Allow invalid UTF-8 paths
    -O <OPTIMIZATION>        Implicitly using `std::str::FromStr`
        --sleep <SLEEP>      Allow human-readable durations

```

Optimization-level (number)
```console
$ typed-derive -O 1
Args { optimization: Some(1), include: None, bind: None, sleep: None, defines: [] }

$ typed-derive -O plaid
? failed
error: Invalid value "plaid" for '-O <OPTIMIZATION>': invalid digit found in string

For more information try --help

```

Include (path)
```console
$ typed-derive -I../hello
Args { optimization: None, include: Some("../hello"), bind: None, sleep: None, defines: [] }

```

IP Address
```console
$ typed-derive --bind 192.0.0.1
Args { optimization: None, include: None, bind: Some(192.0.0.1), sleep: None, defines: [] }

$ typed-derive --bind localhost
? failed
error: Invalid value "localhost" for '--bind <BIND>': invalid IP address syntax

For more information try --help

```

Time
```console
$ typed-derive --sleep 10s
Args { optimization: None, include: None, bind: None, sleep: Some(Duration(10s)), defines: [] }

$ typed-derive --sleep forever
? failed
error: Invalid value "forever" for '--sleep <SLEEP>': expected number at 0

For more information try --help

```

Defines (key-value pairs)
```console
$ typed-derive -D Foo=10 -D Alice=30
Args { optimization: None, include: None, bind: None, sleep: None, defines: [("Foo", 10), ("Alice", 30)] }

$ typed-derive -D Foo
? failed
error: Invalid value "Foo" for '-D <DEFINES>': invalid KEY=value: no `=` found in `Foo`

For more information try --help

$ typed-derive -D Foo=Bar
? failed
error: Invalid value "Foo=Bar" for '-D <DEFINES>': invalid digit found in string

For more information try --help

```
