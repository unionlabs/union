# Collection of examples "how to use `structopt`"

### [Help on the bottom](after_help.rs)

How to append a postscript to the help message generated.

### [At least N](at_least_two.rs)

How to require presence of at least N values, like `val1 val2 ... valN ... valM`.

### [Basic](basic.rs)

A basic example how to use `structopt`.

### [Deny missing docs](deny_missing_docs.rs)

**This is not an example but a test**, it should be moved to `tests` folder
as soon as [this](https://github.com/rust-lang/rust/issues/24584) is fixed (if ever).

### [Doc comments](doc_comments.rs)

How to use doc comments in place of `help/long_help`.

### [Enums as arguments](enum_in_args.rs)

How to use `arg_enum!` with `StructOpt`.

### [Arguments of subcommands in separate `struct`](enum_tuple.rs)

How to extract subcommands' args into external structs.

### [Environment variables](env.rs)

How to use environment variable fallback and how it interacts with `default_value`.

### [Advanced](example.rs)

Somewhat complex example of usage of `structopt`.

### [Flatten](flatten.rs)

How to use `#[structopt(flatten)]`

### [`bash` completions](gen_completions.rs)

Generating `bash` completions with `structopt`.

### [Git](git.rs)

Pseudo-`git` example, shows how to use subcommands and how to document them.

### [Groups](group.rs)

Using `clap::Arg::group` with `structopt`.

### [`key=value` pairs](keyvalue.rs)

How to parse `key=value` pairs.

### [`--no-*` flags](negative_flag.rs)

How to add `no-thing` flag which is `true` by default and `false` if passed.

### [No version](no_version.rs)

How to completely remove version.

### [Rename all](rename_all.rs)

How `#[structopt(rename_all)]` works.

### [Required If](required_if.rs)

How to use `#[structopt(required_if)]`.

### [Skip](skip.rs)

How to use `#[structopt(skip)]`.

### [Aliases](subcommand_aliases.rs)

How to use aliases

### [`true` or `false`](true_or_false.rs)

How to express "`"true"` or `"false"` argument.
