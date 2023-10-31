# Tutorial

*Jump to [builder tutorial](../tutorial_builder/README.md)*

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

You can create an application declaratively with a `struct` and some
attributes.  **This requires enabling the `derive` feature flag.**

[Example:](01_quick.rs)
```console
$ 01_quick_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    01_quick_derive[EXE] [OPTIONS] [NAME] [SUBCOMMAND]

ARGS:
    <NAME>    Optional name to operate on

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
$ 01_quick_derive
Debug mode is off

```

But you can mix and match the various features
```console
$ 01_quick_derive -dd test
Debug mode is on
Not printing testing lists...

```

In addition to this tutorial, see the [derive reference](../derive_ref/README.md).

## Configuring the Parser

You use derive `Parser` to start building a parser.

[Example:](02_apps.rs)
```console
$ 02_apps_derive --help
MyApp 1.0
Kevin K. <kbknapp@gmail.com>
Does awesome things

USAGE:
    02_apps_derive[EXE] --two <TWO> --one <ONE>

OPTIONS:
    -h, --help         Print help information
        --one <ONE>    
        --two <TWO>    
    -V, --version      Print version information

$ 02_apps_derive --version
MyApp 1.0

```

You can use `#[clap(author, version, about)]` attribute defaults to fill these fields in from your `Cargo.toml` file.

[Example:](02_crate.rs)
```console
$ 02_crate_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    02_crate_derive[EXE] --two <TWO> --one <ONE>

OPTIONS:
    -h, --help         Print help information
        --one <ONE>    
        --two <TWO>    
    -V, --version      Print version information

$ 02_crate_derive --version
clap [..]

```

You can use attributes to change the application level behavior of clap.  Any `Command` builder function can be used as an attribute.

[Example:](02_app_settings.rs)
```console
$ 02_app_settings_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    02_app_settings_derive[EXE] --two <TWO> --one <ONE>

OPTIONS:
        --two <TWO>    
        --one <ONE>    
    -h, --help         Print help information
    -V, --version      Print version information

$ 02_app_settings_derive --one -1 --one -3 --two 10
two: "10"
one: "-3"

```

## Adding Arguments

### Positionals

You can have users specify values by their position on the command-line:

[Example:](03_03_positional.rs)
```console
$ 03_03_positional_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_03_positional_derive[EXE] [NAME]

ARGS:
    <NAME>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_03_positional_derive
name: None

$ 03_03_positional_derive bob
name: Some("bob")

```

### Options

You can name your arguments with a flag:
- Order doesn't matter
- They can be optional
- Intent is clearer

The `#[clap(short = 'c')]` and `#[clap(long = "name")]` attributes that define
the flags are `Arg` methods that are derived from the field name when no value
is specified (`#[clap(short)]` and `#[clap(long)]`).

[Example:](03_02_option.rs)
```console
$ 03_02_option_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_02_option_derive[EXE] [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -n, --name <NAME>    
    -V, --version        Print version information

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

### Flags

Flags can also be switches that can be on/off.  This is enabled via the
`#[clap(parse(from_flag)]` attribute though this is implied when the field is a
`bool`.

[Example:](03_01_flag_bool.rs)
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

Or counted with `#[clap(action = clap::ArgAction::Count)]`:

[Example:](03_01_flag_count.rs)
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

### Subcommands

Subcommands are derived with `#[derive(Subcommand)]` and be added via `#[clap(subcommand)]` attribute. Each
instance of a Subcommand can have its own version, author(s), Args, and even its own
subcommands.

[Example:](03_04_subcommands.rs)
```console
$ 03_04_subcommands_derive help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_04_subcommands_derive[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add     Adds files to myapp
    help    Print this message or the help of the given subcommand(s)

$ 03_04_subcommands_derive help add
03_04_subcommands_derive[EXE]-add [..]
Adds files to myapp

USAGE:
    03_04_subcommands_derive[EXE] add [NAME]

ARGS:
    <NAME>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_04_subcommands_derive add bob
'myapp add' was used, name is: Some("bob")

```

Above, we used a struct-variant to define the `add` subcommand.  Alternatively,
you can
[use a struct for your subcommand's arguments](03_04_subcommands_alt.rs).

Because we used `command: Commands` instead of `command: Option<Commands>`:
```console
$ 03_04_subcommands_derive
? failed
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_04_subcommands_derive[EXE] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add     Adds files to myapp
    help    Print this message or the help of the given subcommand(s)

```

Because we added `#[clap(propagate_version = true)]`:
```console
$ 03_04_subcommands_derive --version
clap [..]

$ 03_04_subcommands_derive add --version
03_04_subcommands_derive[EXE]-add [..]

```

### Defaults

We've previously showed that arguments can be `required` or optional.  When
optional, you work with an `Option` and can `unwrap_or`.  Alternatively, you can
set `#[clap(default_value_t)]`.

[Example:](03_05_default_values.rs)
```console
$ 03_05_default_values_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    03_05_default_values_derive[EXE] [NAME]

ARGS:
    <NAME>    [default: alice]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 03_05_default_values_derive
name: "alice"

$ 03_05_default_values_derive bob
name: "bob"

```

## Validation

### Enumerated values

If you have arguments of specific values you want to test for, you can derive
`ValueEnum`.

This allows you specify the valid values for that argument. If the user does not use one of
those specific values, they will receive a graceful exit with error message informing them
of the mistake, and what the possible valid values are

[Example:](04_01_enum.rs)
```console
$ 04_01_enum_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_01_enum_derive[EXE] <MODE>

ARGS:
    <MODE>    What mode to run the program in [possible values: fast, slow]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_01_enum_derive fast
Hare

$ 04_01_enum_derive slow
Tortoise

$ 04_01_enum_derive medium
? failed
error: "medium" isn't a valid value for '<MODE>'
	[possible values: fast, slow]

For more information try --help

```

### Validated values

More generally, you can validate and parse into any data type.

[Example:](04_02_parse.rs)
```console
$ 04_02_parse_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_02_parse_derive[EXE] <PORT>

ARGS:
    <PORT>    Network port to use

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

$ 04_02_parse_derive 22
PORT = 22

$ 04_02_parse_derive foobar
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
$ 04_03_relations_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_03_relations_derive[EXE] [OPTIONS] <--set-ver <VER>|--major|--minor|--patch> [INPUT_FILE]

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

$ 04_03_relations_derive
? failed
error: The following required arguments were not provided:
    <--set-ver <VER>|--major|--minor|--patch>

USAGE:
    04_03_relations_derive[EXE] [OPTIONS] <--set-ver <VER>|--major|--minor|--patch> [INPUT_FILE]

For more information try --help

$ 04_03_relations_derive --major
Version: 2.2.3

$ 04_03_relations_derive --major --minor
? failed
error: The argument '--major' cannot be used with '--minor'

USAGE:
    04_03_relations_derive[EXE] <--set-ver <VER>|--major|--minor|--patch>

For more information try --help

$ 04_03_relations_derive --major -c config.toml
? failed
error: The following required arguments were not provided:
    <INPUT_FILE|--spec-in <SPEC_IN>>

USAGE:
    04_03_relations_derive[EXE] -c <CONFIG> <--set-ver <VER>|--major|--minor|--patch> <INPUT_FILE|--spec-in <SPEC_IN>>

For more information try --help

$ 04_03_relations_derive --major -c config.toml --spec-in input.txt
Version: 2.2.3
Doing work using input input.txt and config config.toml

```

### Custom Validation

As a last resort, you can create custom errors with the basics of clap's formatting.

[Example:](04_04_custom.rs)
```console
$ 04_04_custom_derive --help
clap [..]
A simple to use, efficient, and full-featured Command Line Argument Parser

USAGE:
    04_04_custom_derive[EXE] [OPTIONS] [INPUT_FILE]

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

$ 04_04_custom_derive
? failed
error: Can only modify one version field

USAGE:
    clap [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom_derive --major
Version: 2.2.3

$ 04_04_custom_derive --major --minor
? failed
error: Can only modify one version field

USAGE:
    clap [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom_derive --major -c config.toml
? failed
Version: 2.2.3
error: INPUT_FILE or --spec-in is required when using --config

USAGE:
    clap [OPTIONS] [INPUT_FILE]

For more information try --help

$ 04_04_custom_derive --major -c config.toml --spec-in input.txt
Version: 2.2.3
Doing work using input input.txt and config config.toml

```

## Tips

- For more complex demonstration of features, see our [examples](../README.md).
- See the [derive reference](../derive_ref/README.md) to understand how to use
  anything in the [builder API](https://docs.rs/clap/) in the derive API.
- Proactively check for bad `Command` configurations by calling `Command::debug_assert` in a test ([example](05_01_assert.rs))

## Contributing

New example code:
- Please update the corresponding section in the [builder tutorial](../tutorial_builder/README.md)
- Building: They must be added to [Cargo.toml](../../Cargo.toml) with the appropriate `required-features`.
- Testing: Ensure there is a markdown file with [trycmd](https://docs.rs/trycmd) syntax (generally they'll go in here).

See also the general [CONTRIBUTING](../../CONTRIBUTING.md).
