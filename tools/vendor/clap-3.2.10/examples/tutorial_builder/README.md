# Tutorial

*Jump to [derive tutorial](../tutorial_derive/README.md)*

1. [Quick Start](#quick-start)
2. [Configuring the Parser](#configuring-the-parser)
3. [Adding Arguments](#adding-arguments)
    1. [Positionals](#positionals)
    2. [Options](#options)
    3. [Flags](#flags)
    4. [Subcommands](#subcommands)
    5. [Defaults](#defaults)
4. Validation
    1. [Enumerated values](#enumerated-values)
    2. [Validated values](#validated-values)
    3. [Argument Relations](#argument-relations)
    4. [Custom Validation](#custom-validation)
5. [Tips](#tips)
6. [Contributing](#contributing)

## Quick Start

You can create an application with several arguments using usage strings.

[Example:](01_quick.rs)
```console
$ 01_quick --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    01_quick[EXE] [OPTIONS] [name] [SUBCOMMAND]

ARGS:
    <name>    Optional name to operate on

OPTIONS:
    -c, --config <FILE>    Sets a custom config file
    -d, --debug            Turn debugging information on
    -h, --help             Print help information
    -V, --version          Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    test    does testing things

```

By default, the program does nothing:
```console
$ 01_quick
Debug mode is off

```

But you can mix and match the various features
```console
$ 01_quick -dd test
Debug mode is on
Not printing testing lists...

```

## Configuring the Parser

You use the `Command` to start building a parser.

[Example:](02_apps.rs)
```console
$ 02_apps --help
MyApp 1.0
Kevin K. <kbknapp@gmail.com>
Does awesome things

USAGE:
    02_apps[EXE] --two <VALUE> --one <VALUE>

OPTIONS:
    -h, --help           Print help information
        --one <VALUE>    
        --two <VALUE>    
    -V, --version        Print version information

$ 02_apps --version
MyApp 1.0

```

You can use `command!()` to fill these fields in from your `Cargo.toml`
file.  **This requires the `cargo` feature flag.**

[Example:](02_crate.rs)
```console
$ 02_crate --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    02_crate[EXE] --two <VALUE> --one <VALUE>

OPTIONS:
    -h, --help           Print help information
        --one <VALUE>    
        --two <VALUE>    
    -V, --version        Print version information

$ 02_crate --version
clap [..]

```

You can use `Command` methods to change the application level behavior of clap.

[Example:](02_app_settings.rs)
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

## Adding Arguments

### Positionals

You can have users specify values by their position on the command-line:

[Example:](03_03_positional.rs)
```console
$ 03_03_positional --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_03_positional[EXE] [NAME]

ARGS:
    <NAME>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_03_positional
NAME: None

$ 03_03_positional bob
NAME: Some("bob")

```

### Options

You can name your arguments with a flag:
- Order doesn't matter
- They can be optional
- Intent is clearer

[Example:](03_02_option.rs)
```console
$ 03_02_option --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_02_option[EXE] [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    
    -V, --version        Print version information

$ 03_02_option
name: None

$ 03_02_option --name bob
name: Some("bob")

$ 03_02_option --name=bob
name: Some("bob")

$ 03_02_option -n bob
name: Some("bob")

$ 03_02_option -n=bob
name: Some("bob")

$ 03_02_option -nbob
name: Some("bob")

```

### Flags

Flags can also be switches that can be on/off:

[Example:](03_01_flag_bool.rs)
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

Or counted.

[Example:](03_01_flag_count.rs)
```console
$ 03_01_flag_count --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_01_flag_count[EXE] [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    
    -V, --version    Print version information

$ 03_01_flag_count
verbose: 0

$ 03_01_flag_count --verbose
verbose: 1

$ 03_01_flag_count --verbose --verbose
verbose: 2

```

### Subcommands

Subcommands are defined as `Command`s that get added via `Command::subcommand`. Each
instance of a Subcommand can have its own version, author(s), Args, and even its own
subcommands.

[Example:](03_04_subcommands.rs)
```console
$ 03_04_subcommands help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_04_subcommands[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add     Adds files to myapp
    help    Print this message or the help of the given subcommand(s)

$ 03_04_subcommands help add
03_04_subcommands[EXE]-add [..]
Adds files to myapp

USAGE:
    03_04_subcommands[EXE] add [NAME]

ARGS:
    <NAME>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_04_subcommands add bob
'myapp add' was used, name is: Some("bob")

```

Because we set `Command::arg_required_else_help`:
```console
$ 03_04_subcommands
? failed
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_04_subcommands[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add     Adds files to myapp
    help    Print this message or the help of the given subcommand(s)

```

Because we set `Command::propagate_version`:
```console
$ 03_04_subcommands --version
clap [..]

$ 03_04_subcommands add --version
03_04_subcommands[EXE]-add [..]

```

### Defaults

We've previously showed that arguments can be `required` or optional.  When
optional, you work with a `Option` and can `unwrap_or`.  Alternatively, you can
set `Arg::default_value`.

[Example:](03_05_default_values.rs)
```console
$ 03_05_default_values --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_05_default_values[EXE] [NAME]

ARGS:
    <NAME>    [default: alice]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_05_default_values
NAME: "alice"

$ 03_05_default_values bob
NAME: "bob"

```

## Validation

### Enumerated values

If you have arguments of specific values you want to test for, you can use the
`PossibleValuesParser` or `Arg::value_parser(["val1", ...])` for short.

This allows you specify the valid values for that argument. If the user does not use one of
those specific values, they will receive a graceful exit with error message informing them
of the mistake, and what the possible valid values are

[Example:](04_01_possible.rs)
```console
$ 04_01_possible --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_01_possible[EXE] <MODE>

ARGS:
    <MODE>    What mode to run the program in [possible values: fast, slow]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_01_possible fast
Hare

$ 04_01_possible slow
Tortoise

$ 04_01_possible medium
? failed
error: "medium" isn't a valid value for '<MODE>'
	[possible values: fast, slow]

For more information try --help

```

When enabling the `derive` feature, you can use `ValueEnum` to take care of the boiler plate for you, giving the same results.

[Example:](04_01_enum.rs)
```console
$ 04_01_enum --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_01_enum[EXE] <MODE>

ARGS:
    <MODE>    What mode to run the program in [possible values: fast, slow]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_01_enum fast
Hare

$ 04_01_enum slow
Tortoise

$ 04_01_enum medium
? failed
error: "medium" isn't a valid value for '<MODE>'
	[possible values: fast, slow]

For more information try --help

```

### Validated values

More generally, you can validate and parse into any data type.

[Example:](04_02_parse.rs)
```console
$ 04_02_parse --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_02_parse[EXE] <PORT>

ARGS:
    <PORT>    Network port to use

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_02_parse 22
PORT = 22

$ 04_02_parse foobar
? failed
error: Invalid value "foobar" for '<PORT>': invalid digit found in string

For more information try --help

$ 04_02_parse_derive 0
? failed
error: Invalid value "0" for '<PORT>': 0 is not in 1..=65535

For more information try --help

```

A custom parser can be used to improve the error messages or provide additional validation:

[Example:](04_02_validate.rs)
```console
$ 04_02_validate --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_02_validate[EXE] <PORT>

ARGS:
    <PORT>    Network port to use

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_02_validate 22
PORT = 22

$ 04_02_validate foobar
? failed
error: Invalid value "foobar" for '<PORT>': `foobar` isn't a port number

For more information try --help

$ 04_02_validate 0
? failed
error: Invalid value "0" for '<PORT>': Port not in range 1-65535

For more information try --help

```

### Argument Relations

You can declare dependencies or conflicts between `Arg`s or even `ArgGroup`s.  

`ArgGroup`s  make it easier to declare relations instead of having to list each
individually, or when you want a rule to apply "any but not all" arguments.

Perhaps the most common use of `ArgGroup`s is to require one and *only* one argument to be
present out of a given set. Imagine that you had multiple arguments, and you want one of them to
be required, but making all of them required isn't feasible because perhaps they conflict with
each other.

[Example:](04_03_relations.rs)
```console
$ 04_03_relations --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_03_relations[EXE] [OPTIONS] <--set-ver <VER>|--major|--minor|--patch> [INPUT_FILE]

ARGS:
    <INPUT_FILE>    some regular input

OPTIONS:
    -c <CONFIG>                
    -h, --help                 Print help information
        --major                auto inc major
        --minor                auto inc minor
        --patch                auto inc patch
        --set-ver <VER>        set version manually
        --spec-in <SPEC_IN>    some special input argument
    -V, --version              Print version information

$ 04_03_relations
? failed
error: The following required arguments were not provided:
    <--set-ver <VER>|--major|--minor|--patch>

USAGE:
    04_03_relations[EXE] [OPTIONS] <--set-ver <VER>|--major|--minor|--patch> [INPUT_FILE]

For more information try --help

$ 04_03_relations --major
Version: 2.2.3

$ 04_03_relations --major --minor
? failed
error: The argument '--major' cannot be used with '--minor'

USAGE:
    04_03_relations[EXE] <--set-ver <VER>|--major|--minor|--patch>

For more information try --help

$ 04_03_relations --major -c config.toml
? failed
error: The following required arguments were not provided:
    <INPUT_FILE|--spec-in <SPEC_IN>>

USAGE:
    04_03_relations[EXE] -c <CONFIG> <--set-ver <VER>|--major|--minor|--patch> <INPUT_FILE|--spec-in <SPEC_IN>>

For more information try --help

$ 04_03_relations --major -c config.toml --spec-in input.txt
Version: 2.2.3
Doing work using input input.txt and config config.toml

```

### Custom Validation

As a last resort, you can create custom errors with the basics of clap's formatting.

[Example:](04_04_custom.rs)
```console
$ 04_04_custom --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

ARGS:
    <INPUT_FILE>    some regular input

OPTIONS:
    -c <CONFIG>                
    -h, --help                 Print help information
        --major                auto inc major
        --minor                auto inc minor
        --patch                auto inc patch
        --set-ver <VER>        set version manually
        --spec-in <SPEC_IN>    some special input argument
    -V, --version              Print version information

$ 04_04_custom
? failed
error: Can only modify one version field

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom --major
Version: 2.2.3

$ 04_04_custom --major --minor
? failed
error: Can only modify one version field

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom --major -c config.toml
? failed
Version: 2.2.3
error: INPUT_FILE or --spec-in is required when using --config

USAGE:
    04_04_custom[EXE] [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom --major -c config.toml --spec-in input.txt
Version: 2.2.3
Doing work using input input.txt and config config.toml

```

## Tips

- For more complex demonstration of features, see our [examples](../README.md).
- Proactively check for bad `Command` configurations by calling `Command::debug_assert` in a test ([example](05_01_assert.rs))

## Contributing

New example code:
- Please update the corresponding section in the [derive tutorial](../tutorial_derive/README.md)
- Building: They must be added to [Cargo.toml](../../Cargo.toml) with the appropriate `required-features`.
- Testing: Ensure there is a markdown file with [trycmd](https://docs.rs/trycmd) syntax (generally they'll go in here).

See also the general [CONTRIBUTING](../../CONTRIBUTING.md).
