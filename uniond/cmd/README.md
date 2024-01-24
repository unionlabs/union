# CMD

The `cmd` module is responsible for initializing the root command used in the CLI interface for `uniond`.

## Sub-Commands

In addition to initializing the `uniond` command, we use this module to add a few of our own sub-commands.

### `genbn`

Used to generate a bn254 key-pair for use as a consensus key.

### `genstateproof`

Generates a state proof for the current node.
