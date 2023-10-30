# Derive Reference

1. [Overview](#overview)
2. [Attributes](#attributes)
    1. [Terminology](#terminology)
    2. [Command Attributes](#command-attributes)
    3. [Arg Attributes](#arg-attributes)
    4. [Arg Enum Attributes](#arg-enum-attributes)
    5. [Possible Value Attributes](#possible-value-attributes)
3. [Arg Types](#arg-types)
4. [Doc Comments](#doc-comments)
5. [Tips](#tips)
6. [Mixing Builder and Derive APIs](#mixing-builder-and-derive-apis)

## Overview

To derive `clap` types, you need to enable the `derive` feature flag.

See [demo.rs](../demo.rs) and [demo.md](../demo.md) for a brief example.

Let's start by breaking down the anatomy of the derive attributes:
```rust
use clap::{Parser, Args, Subcommand, ValueEnum};

/// Doc comment
#[derive(Parser)]
#[clap(APP ATTRIBUTE)]
struct Cli {
    /// Doc comment
    #[clap(ARG ATTRIBUTE)]
    field: UserType,

    #[clap(value_enum, ARG ATTRIBUTE...)]
    field: EnumValues,

    #[clap(flatten)]
    delegate: Struct,

    #[clap(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Args)]
#[clap(PARENT APP ATTRIBUTE)]
struct Struct { 
    /// Doc comment
    #[clap(ARG ATTRIBUTE)]
    field: UserType,
}

/// Doc comment
#[derive(Subcommand)]
#[clap(PARENT APP ATTRIBUTE)]
enum Command {
    /// Doc comment
    #[clap(APP ATTRIBUTE)]
    Variant1(Struct),

    /// Doc comment
    #[clap(APP ATTRIBUTE)]
    Variant2 {
        /// Doc comment
        #[clap(ARG ATTRIBUTE)]
        field: UserType,
    }
}

/// Doc comment
#[derive(ValueEnum)]
#[clap(ARG ENUM ATTRIBUTE)]
enum EnumValues {
    /// Doc comment
    #[clap(POSSIBLE VALUE ATTRIBUTE)]
    Variant1,
}

fn main() {
    let cli = Cli::parse();
}
```

- `Parser` parses arguments into a `struct` (arguments) or `enum` (subcommands).
- `Args` allows defining a set of re-usable arguments that get merged into their parent container.
- `Subcommand` defines available subcommands.
  - Subcommand arguments can be defined in a struct-variant or automatically flattened with a tuple-variant.
- `ValueEnum` allows parsing a value directly into an `enum`, erroring on unsupported values.
  - The derive doesn't work on enums that contain non-unit variants, unless they are skipped

See also the [tutorial](../tutorial_derive/README.md) and [examples](../README.md).

## Attributes

### Terminology

**Raw attributes** are forwarded directly to the underlying `clap` builder.  Any
`Command`, `Arg`, or `PossibleValue` method can be used as an attribute.

Raw attributes come in two different syntaxes:
```rust
#[clap(
    global = true, // name = arg form, neat for one-arg methods
    required_if_eq("out", "file") // name(arg1, arg2, ...) form.
)]
```

- `method = arg` can only be used for methods which take only one argument.
- `method(arg1, arg2)` can be used with any method.

As long as `method_name` is not one of the magical methods - it will be
translated into a mere method call.

**Magic attributes** have post-processing done to them, whether that is
- Providing of defaults
- Special behavior is triggered off of it

Magic attributes are more constrained in the syntax they support, usually just
`<attr> = <value>` though some use `<attr>(<value>)` instead.  See the specific
magic attributes documentation for details.  This allows users to access the
raw behavior of an attribute via `<attr>(<value>)` syntax.

**NOTE:** Some attributes are inferred from [Arg Types](#arg-types) and [Doc
Comments](#doc-comments).  Explicit attributes take precedence over inferred
attributes.

### Command Attributes

These correspond to a `clap::Command` which is used for both top-level parsers and
when defining subcommands.

**Raw attributes:**  Any [`Command` method](https://docs.rs/clap/latest/clap/type.Command.html) can also be used as an attribute, see [Terminology](#terminology) for syntax.
- e.g. `#[clap(arg_required_else_help(true))]` would translate to `cmd.arg_required_else_help(true)`

**Magic attributes:**
- `name  = <expr>`: `clap::Command::name`
  - When not present: [crate `name`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field) (`Parser` container), variant name (`Subcommand` variant)
- `version [= <expr>]`: `clap::Command::version`
  - When not present: no version set
  - Without `<expr>`: defaults to [crate `version`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-version-field)
- `author [= <expr>]`: `clap::Command::author`
  - When not present: no author set
  - Without `<expr>`: defaults to [crate `authors`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-authors-field)
- `about [= <expr>]`: `clap::Command::about`
  - When not present: [Doc comment summary](#doc-comments)
  - Without `<expr>`: [crate `description`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-description-field) (`Parser` container)
    - **TIP:** When a doc comment is also present, you most likely want to add
      `#[clap(long_about = None)]` to clear the doc comment so only `about`
      gets shown with both `-h` and `--help`.
- `long_about = <expr>`: `clap::Command::long_about`
  - When not present: [Doc comment](#doc-comments) if there is a blank line, else nothing
- `verbatim_doc_comment`: Minimizes pre-processing when converting doc comments to `about` / `long_about`
- `next_display_order`: `clap::Command::next_display_order`
- `next_help_heading`: `clap::Command::next_help_heading`
  - When `flatten`ing `Args`, this is scoped to just the args in this struct and any struct `flatten`ed into it
- `rename_all = <string_literal>`: Override default field / variant name case conversion for `Command::name` / `Arg::name`
  - When not present: `"kebab-case"`
  - Available values: `"camelCase"`, `"kebab-case"`, `"PascalCase"`, `"SCREAMING_SNAKE_CASE"`, `"snake_case"`, `"lower"`, `"UPPER"`, `"verbatim"`
- `rename_all_env = <string_literal>`: Override default field name case conversion for env variables for  `clap::Arg::env`
  - When not present: `"SCREAMING_SNAKE_CASE"`
  - Available values: `"camelCase"`, `"kebab-case"`, `"PascalCase"`, `"SCREAMING_SNAKE_CASE"`, `"snake_case"`, `"lower"`, `"UPPER"`, `"verbatim"`

And for `Subcommand` variants:
- `skip`: Ignore this variant
- `flatten`: Delegates to the variant for more subcommands (must implement `Subcommand`)
- `subcommand`: Nest subcommands under the current set of subcommands (must implement `Subcommand`)
- `external_subcommand`: `clap::Command::allow_external_subcommand(true)`
  - Variant must be either `Variant(Vec<String>)` or `Variant(Vec<OsString>)`

### Arg Attributes

These correspond to a `clap::Arg`.

**Raw attributes:**  Any [`Arg` method](https://docs.rs/clap/latest/clap/struct.Arg.html) can also be used as an attribute, see [Terminology](#terminology) for syntax.
- e.g. `#[clap(max_values(3))]` would translate to `arg.max_values(3)`

**Magic attributes**:
- `name = <expr>`: `clap::Arg::new`
  - When not present: case-converted field name is used
- `value_parser [= <expr>]`: `clap::Arg::value_parser`
  - When not present: will auto-select an implementation based on the field type
  - To register a custom type's `ValueParser`, implement `ValueParserFactory`
  - When present, implies `#[clap(action)]`
- `action [= <expr>]`: `clap::Arg::action`
  - When not present: will auto-select an action based on the field type
  - When present, implies `#[clap(value_parser)]`
- `help = <expr>`: `clap::Arg::help`
  - When not present: [Doc comment summary](#doc-comments)
- `long_help = <expr>`: `clap::Arg::long_help`
  - When not present: [Doc comment](#doc-comments) if there is a blank line, else nothing
- `verbatim_doc_comment`: Minimizes pre-processing when converting doc comments to `help` / `long_help`
- `short [= <char>]`: `clap::Arg::short`
  - When not present: no short set
  - Without `<char>`: defaults to first character in the case-converted field name
- `long [= <str>]`: `clap::Arg::long`
  - When not present: no long set
  - Without `<str>`: defaults to the case-converted field name
- `env [= <str>]`: `clap::Arg::env` (needs `env` feature enabled)
  - When not present: no env set
  - Without `<str>`: defaults to the case-converted field name
- `flatten`: Delegates to the field for more arguments (must implement `Args`)
  - Only `help_heading` can be used with `flatten`.  See
    [clap-rs/clap#3269](https://github.com/clap-rs/clap/issues/3269) for why
    arg attributes are not generally supported.
  - **Tip:** Though we do apply a flattened `Args`'s Parent Command Attributes, this
    makes reuse harder. Generally prefer putting the cmd attributes on the `Parser`
    or on the flattened field.
- `subcommand`: Delegates definition of subcommands to the field (must implement `Subcommand`)
  - When `Option<T>`, the subcommand becomes optional
- `from_global`: Read a `clap::Arg::global` argument (raw attribute), regardless of what subcommand you are in
- `parse(<kind> [= <function>])`: `clap::Arg::validator` and `clap::ArgMatches::values_of_t`
  - **Deprecated:**
    - Use `value_parser(...)` for `from_str`, `try_from_str`, `from_os_str`, and `try_from_os_str`
    - Use `action(ArgAction::Count` for `from_occurrences`
    - Use `action(ArgAction::SetTrue` for `from_flag`
  - Default: `try_from_str`
  - Warning: for `Path` / `OsString`, be sure to use `try_from_os_str`
  - See [Arg Types](#arg-types) for more details
- `value_enum`: Parse the value using the `ValueEnum` trait
- `skip [= <expr>]`: Ignore this field, filling in with `<expr>`
  - Without `<expr>`: fills the field with `Default::default()`
- `default_value = <str>`: `clap::Arg::default_value` and `clap::Arg::required(false)`
- `default_value_t [= <expr>]`: `clap::Arg::default_value` and `clap::Arg::required(false)`
  - Requires `std::fmt::Display` or `#[clap(value_enum)]`
  - Without `<expr>`, relies on `Default::default()`
- `default_value_os_t [= <expr>]`: `clap::Arg::default_value_os` and `clap::Arg::required(false)`
  - Requires `std::convert::Into<OsString>` or `#[clap(value_enum)]`
  - Without `<expr>`, relies on `Default::default()`

### Arg Enum Attributes

- `rename_all = <string_literal>`: Override default field / variant name case conversion for `PossibleValue::new`
  - When not present: `"kebab-case"`
  - Available values: `"camelCase"`, `"kebab-case"`, `"PascalCase"`, `"SCREAMING_SNAKE_CASE"`, `"snake_case"`, `"lower"`, `"UPPER"`, `"verbatim"`

### Possible Value Attributes

These correspond to a `clap::PossibleValue`.

**Raw attributes:**  Any [`PossibleValue` method](https://docs.rs/clap/latest/clap/struct.PossibleValue.html) can also be used as an attribute, see [Terminology](#terminology) for syntax.
- e.g. `#[clap(alias("foo"))]` would translate to `pv.alias("foo")`

**Magic attributes**:
- `name = <expr>`: `clap::PossibleValue::new`
  - When not present: case-converted field name is used
- `help = <expr>`: `clap::PossibleValue::help`
  - When not present: [Doc comment summary](#doc-comments)

## Arg Types

`clap` assumes some intent based on the type used:

| Type                | Effect                               | Implies                                                          |
|---------------------|--------------------------------------|------------------------------------------------------------------|
| `bool`              | flag                                 | `#[clap(parse(from_flag))]`                                     |
| `Option<T>`         | optional argument                    | `.takes_value(true).required(false)`                             |
| `Option<Option<T>>` | optional value for optional argument | `.takes_value(true).required(false).min_values(0).max_values(1)` |
| `T`                 | required argument                    | `.takes_value(true).required(!has_default)`                      |
| `Vec<T>`            | `0..` occurrences of argument        | `.takes_value(true).required(false).multiple_occurrences(true)`  |
| `Option<Vec<T>>`    | `0..` occurrences of argument        | `.takes_value(true).required(false).multiple_occurrences(true)`  |

Notes:
- For custom type behavior, you can override the implied attributes/settings and/or set additional ones
  - For example, see [custom-bool](./custom-bool.md)
- `Option<Vec<T>>` will be `None` instead of `vec![]` if no arguments are provided.
  - This gives the user some flexibility in designing their argument, like with `min_values(0)`

You can then support your custom type with `#[clap(parse(<kind> [= <function>]))]`:

| `<kind>`                 | Signature                             | Default `<function>`            |
|--------------------------|---------------------------------------|---------------------------------|
| `from_str`               | `fn(&str) -> T`                       | `::std::convert::From::from`    |
| `try_from_str` (default) | `fn(&str) -> Result<T, E>`            | `::std::str::FromStr::from_str` |
| `from_os_str`            | `fn(&OsStr) -> T`                     | `::std::convert::From::from`    |
| `try_from_os_str`        | `fn(&OsStr) -> Result<T, E>`          | (no default function)           |
| `from_occurrences`       | `fn(u64) -> T`                        | `value as T`                    |
| `from_flag`              | `fn(bool) -> T`                       | `::std::convert::From::from`    |

Notes:
- `from_os_str`:
  - Implies `arg.takes_value(true).allow_invalid_utf8(true)`
- `try_from_os_str`:
  - Implies `arg.takes_value(true).allow_invalid_utf8(true)`
- `from_occurrences`:
  - Implies `arg.takes_value(false).multiple_occurrences(true)`
  - Reads from `clap::ArgMatches::occurrences_of` rather than a `get_one` function
    - Note: operations on values, like `default_value`, are unlikely to do what you want
- `from_flag`
  - Implies `arg.takes_value(false)`
  - Reads from `clap::ArgMatches::is_present` rather than a `get_one` function
    - Note: operations on values, like `default_value`, are unlikely to do what you want

**Warning:**
- To support non-UTF8 paths, you should use `#[clap(value_parser)]` otherwise
  `clap` will parse it as a `String` which will fail on some paths.

## Doc Comments

In clap, help messages for the whole binary can be specified
via [`Command::about`] and [`Command::long_about`] while help messages
for individual arguments can be specified via [`Arg::help`] and [`Arg::long_help`]".

`long_*` variants are used when user calls the program with
`--help` and "short" variants are used with `-h` flag.

```rust
# use clap::Parser;

#[derive(Parser)]
#[clap(about = "I am a program and I work, just pass `-h`", long_about = None)]
struct Foo {
    #[clap(short, help = "Pass `-h` and you'll see me!")]
    bar: String,
}
```

For convenience, doc comments can be used instead of raw methods
(this example works exactly like the one above):

```rust
# use clap::Parser;

#[derive(Parser)]
/// I am a program and I work, just pass `-h`
struct Foo {
    /// Pass `-h` and you'll see me!
    bar: String,
}
```

**NOTE:** Attributes have priority over doc comments!

**Top level doc comments always generate `Command::about/long_about` calls!**
If you really want to use the `Command::about/long_about` methods (you likely don't),
use the `about` / `long_about` attributes to override the calls generated from
the doc comment.  To clear `long_about`, you can use
`#[clap(long_about = None)]`.

**TIP:** Set `#![deny(missing_docs)]` to catch missing `--help` documentation at compile time.

### Pre-processing

```rust
# use clap::Parser;
#[derive(Parser)]
/// Hi there, I'm Robo!
///
/// I like beeping, stumbling, eating your electricity,
/// and making records of you singing in a shower.
/// Pay up, or I'll upload it to youtube!
struct Robo {
    /// Call my brother SkyNet.
    ///
    /// I am artificial superintelligence. I won't rest
    /// until I'll have destroyed humanity. Enjoy your
    /// pathetic existence, you mere mortals.
    #[clap(long, action)]
    kill_all_humans: bool,
}
```

A doc comment consists of three parts:
- Short summary
- A blank line (whitespace only)
- Detailed description, all the rest

The summary corresponds with `Command::about` / `Arg::help`.  When a blank line is
present, the whole doc comment will be passed to `Command::long_about` /
`Arg::long_help`.  Or in other words, a doc may result in just a `Command::about` /
`Arg::help` or `Command::about` / `Arg::help` and `Command::long_about` /
`Arg::long_help`

In addition, when `verbatim_doc_comment` is not present, `clap` applies some preprocessing, including:

- Strip leading and trailing whitespace from every line, if present.

- Strip leading and trailing blank lines, if present.

- Interpret each group of non-empty lines as a word-wrapped paragraph.

  We replace newlines within paragraphs with spaces to allow the output
  to be re-wrapped to the terminal width.

- Strip any excess blank lines so that there is exactly one per paragraph break.

- If the first paragraph ends in exactly one period,
  remove the trailing period (i.e. strip trailing periods but not trailing ellipses).

Sometimes you don't want this preprocessing to apply, for example the comment contains
some ASCII art or markdown tables, you would need to preserve LFs along with
blank lines and the leading/trailing whitespace. When you pass use the
`verbatim_doc_comment` magic attribute, you  preserve
them.

**Note:** Keep in mind that `verbatim_doc_comment` will *still*
- Remove one leading space from each line, even if this attribute is present,
  to allow for a space between `///` and the content.
- Remove leading and trailing blank lines

## Tips

- To get access to a `Command` call `CommandFactory::command` (implemented when deriving `Parser`)
- Proactively check for bad `Command` configurations by calling `Command::debug_assert` in a test ([example](../tutorial_derive/05_01_assert.rs))

## Mixing Builder and Derive APIs

The builder and derive APIs do not live in isolation. They can work together, which is especially helpful if some arguments can be specified at compile-time while others must be specified at runtime.

### Using derived arguments in a builder application

*[Jump to source](augment_args.rs)*

When using the derive API, you can `#[clap(flatten)]` a struct deriving `Args` into a struct deriving `Args` or `Parser`. This example shows how you can augment a `Command` instance created using the builder API with `Args` created using the derive API.

It uses the `Args::augment_args` method to add the arguments to the `Command` instance.

Crates such as [clap-verbosity-flag](https://github.com/rust-cli/clap-verbosity-flag) provide structs that implement `Args` or `Parser`. Without the technique shown in this example, it would not be possible to use such crates with the builder API. `augment_args` to the rescue!

### Using derived subcommands in a builder application

*[Jump to source](augment_subcommands.rs)*

When using the derive API, you can use `#[clap(subcommand)]` inside the struct to add subcommands. The type of the field is usually an enum that derived `Parser`. However, you can also add the subcommands in that enum to a `Command` instance created with the builder API.

It uses the `Subcommand::augment_subcommands` method to add the subcommands to the `Command` instance.

### Adding hand-implemented subcommands to a derived application

*[Jump to source](hand_subcommand.rs)*

When using the derive API, you can use `#[clap(subcommand)]` inside the struct to add subcommands. The type of the field is usually an enum that derived `Parser`. However, you can also implement the `Subcommand` trait manually on this enum (or any other type) and it can still be used inside the struct created with the derive API. The implementation of the `Subcommand` trait will use the builder API to add the subcommands to the `Command` instance created behind the scenes for you by the derive API.

Notice how in the previous example we used `augment_subcommands` on an enum that derived `Parser`, whereas now we implement `augment_subcommands` ourselves, but the derive API calls it automatically since we used the `#[clap(subcommand)]` attribute.

### Flattening hand-implemented args into a derived application

*[Jump to source](flatten_hand_args.rs)*

When using the derive API, you can use `#[clap(flatten)]` inside the struct to add arguments as if they were added directly to the containing struct. The type of the field is usually an struct that derived `Args`. However, you can also implement the `Args` trait manually on this struct (or any other type) and it can still be used inside the struct created with the derive API. The implementation of the `Args` trait will use the builder API to add the arguments to the `Command` instance created behind the scenes for you by the derive API.

Notice how in the example 1 we used `augment_args` on the struct that derived `Parser`, whereas now we implement `augment_args` ourselves, but the derive API calls it automatically since we used the `#[clap(flatten)]` attribute.
