# Golden Generated Files

This directory is a repository for the generated files of `flatc`.

We check in the generated code so we can see, during a PR review, how the
changes affect the generated output. Its also useful as a reference to point too
as how things work across various languages.

These files are **NOT** intended to be depended on by any code, such as tests or
or compiled examples.

## Languages Specifics

Each language should keep their generated code in their respective directories.
However, the parent schemas can, and should, be shared so we have a consistent
view of things across languages. These are kept in the `schema/` directory.

Some languages may not support every generation feature, so each language is
required to specify the `flatc` arguments individually.

* Try to avoid includes and nested directories, preferring it as flat as 
possible.

## Updating

Just run the `generate_goldens.py` script and it should generate them all.
