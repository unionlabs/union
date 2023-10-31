Following are tests for the interop examples in this directory.

## Augment Args

```console
$ interop_augment_args
Value of built: false
Value of derived via ArgMatches: false
Value of derived: DerivedArgs {
    derived: false,
}

```

```console
$ interop_augment_args -b --derived
Value of built: true
Value of derived via ArgMatches: true
Value of derived: DerivedArgs {
    derived: true,
}

```

```console
$ interop_augment_args -d --built
Value of built: true
Value of derived via ArgMatches: true
Value of derived: DerivedArgs {
    derived: true,
}

```

```console
$ interop_augment_args --unknown
? failed
error: Found argument '--unknown' which wasn't expected, or isn't valid in this context

	If you tried to supply `--unknown` as a value rather than a flag, use `-- --unknown`

USAGE:
    interop_augment_args[EXE] [OPTIONS]

For more information try --help

```

## Augment Subcommands

```console
$ interop_augment_subcommands
? failed
error: A subcommand is required but one was not provided.
```

```console
$ interop_augment_subcommands derived
Derived subcommands: Derived {
    derived_flag: false,
}

```

```console
$ interop_augment_subcommands derived --derived-flag
Derived subcommands: Derived {
    derived_flag: true,
}

```

```console
$ interop_augment_subcommands derived --unknown
? failed
error: Found argument '--unknown' which wasn't expected, or isn't valid in this context

	If you tried to supply `--unknown` as a value rather than a flag, use `-- --unknown`

USAGE:
    interop_augment_subcommands[EXE] derived [OPTIONS]

For more information try --help

```

```console
$ interop_augment_subcommands unknown
? failed
error: Found argument 'unknown' which wasn't expected, or isn't valid in this context

USAGE:
    interop_augment_subcommands[EXE] [SUBCOMMAND]

For more information try --help

```

## Hand-Implemented Subcommand

```console
$ interop_hand_subcommand
? failed
error: 'interop_hand_subcommand[EXE]' requires a subcommand but one was not provided

USAGE:
    interop_hand_subcommand[EXE] [OPTIONS] <SUBCOMMAND>

For more information try --help

```

```console
$ interop_hand_subcommand add
Cli {
    top_level: false,
    subcommand: Add(
        AddArgs {
            name: [],
        },
    ),
}

```

```console
$ interop_hand_subcommand add a b c
Cli {
    top_level: false,
    subcommand: Add(
        AddArgs {
            name: [
                "a",
                "b",
                "c",
            ],
        },
    ),
}

```

```console
$ interop_hand_subcommand add --unknown
? failed
error: Found argument '--unknown' which wasn't expected, or isn't valid in this context

	If you tried to supply `--unknown` as a value rather than a flag, use `-- --unknown`

USAGE:
    interop_hand_subcommand[EXE] add [NAME]...

For more information try --help

```

```console
$ interop_hand_subcommand remove
Cli {
    top_level: false,
    subcommand: Remove(
        RemoveArgs {
            force: false,
            name: [],
        },
    ),
}

```

```console
$ interop_hand_subcommand remove --force a b c
Cli {
    top_level: false,
    subcommand: Remove(
        RemoveArgs {
            force: true,
            name: [
                "a",
                "b",
                "c",
            ],
        },
    ),
}

```

```console
$ interop_hand_subcommand unknown
? failed
error: Found argument 'unknown' which wasn't expected, or isn't valid in this context

USAGE:
    interop_hand_subcommand[EXE] [OPTIONS] <SUBCOMMAND>

For more information try --help

```

## Flatten Hand-Implemented Args

```console
$ interop_flatten_hand_args
Cli {
    top_level: false,
    more_args: CliArgs {
        foo: false,
        bar: false,
        quuz: None,
    },
}

```

```console
$ interop_flatten_hand_args -f --bar
Cli {
    top_level: false,
    more_args: CliArgs {
        foo: true,
        bar: true,
        quuz: None,
    },
}

```

```console
$ interop_flatten_hand_args --quuz abc
Cli {
    top_level: false,
    more_args: CliArgs {
        foo: false,
        bar: false,
        quuz: Some(
            "abc",
        ),
    },
}

```

```console
$ interop_flatten_hand_args --unknown
? failed
error: Found argument '--unknown' which wasn't expected, or isn't valid in this context

	If you tried to supply `--unknown` as a value rather than a flag, use `-- --unknown`

USAGE:
    interop_flatten_hand_args[EXE] [OPTIONS]

For more information try --help

```
