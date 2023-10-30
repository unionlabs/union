# Examples

- Basic demo: [derive](demo.md)
- Typed arguments: [derive](typed-derive.md)
  - Topics:
    - Custom `parse()`
- Custom cargo command: [builder](cargo-example.md), [derive](cargo-example-derive.md)
  - Topics:
    - Subcommands
    - Cargo plugins
- git-like interface: [builder](git.md), [derive](git-derive.md)
  - Topics:
    - Subcommands
    - External subcommands
    - Optional subcommands
    - Default subcommands
- pacman-like interface: [builder](pacman.md)
  - Topics:
    - Flag subcommands
    - Conflicting arguments
- Escaped positionals with `--`: [builder](escaped-positional.md), [derive](escaped-positional-derive.md)
- Multi-call
  - busybox: [builder](multicall-busybox.md)
    - Topics:
      - Subcommands
  - hostname: [builder](multicall-hostname.md)
    - Topics:
      - Subcommands
- repl: [builder](repl.rs)
  - Topics:
    - Read-Eval-Print Loops / Custom command lines

## Contributing

New examples:
- Building: They must be added to [Cargo.toml](../Cargo.toml) with the appropriate `required-features`.
- Testing: Ensure there is a markdown file with [trycmd](https://docs.rs/trycmd) syntax
- Link the `.md` file from here

See also the general [CONTRIBUTING](../CONTRIBUTING.md).
