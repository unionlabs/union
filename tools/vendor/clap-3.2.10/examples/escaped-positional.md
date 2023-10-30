*Jump to [source](escaped-positional.rs)*

**This requires enabling the `cargo` feature flag.**

You can use `--` to escape further arguments.

Let's see what this looks like in the help:
```console
$ escaped-positional --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    escaped-positional[EXE] [OPTIONS] [-- <SLOP>...]

ARGS:
    <SLOP>...    

OPTIONS:
    -f               
    -h, --help       Print help information
    -p <PEAR>        
    -V, --version    Print version information

```

Here is a baseline without any arguments:
```console
$ escaped-positional
-f used: false
-p's value: None
'slops' values: []

```

Notice that we can't pass positional arguments before `--`:
```console
$ escaped-positional foo bar
? failed
error: Found argument 'foo' which wasn't expected, or isn't valid in this context

USAGE:
    escaped-positional[EXE] [OPTIONS] [-- <SLOP>...]

For more information try --help

```

But you can after:
```console
$ escaped-positional -f -p=bob -- sloppy slop slop
-f used: true
-p's value: Some("bob")
'slops' values: ["sloppy", "slop", "slop"]

```

As mentioned, the parser will directly pass everything through:
```console
$ escaped-positional -- -f -p=bob sloppy slop slop
-f used: false
-p's value: None
'slops' values: ["-f", "-p=bob", "sloppy", "slop", "slop"]

```
