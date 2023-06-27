# uniond

**uniond** is the canonical implementation of a full node for the union network. Validators, RPC, and archive operators can run it to participate in the network.

## Getting started

The easiest way to obtain the `uniond` binary is to check the [releases](https://github.com/unionlabs/union/releases). Alternatively, they can be built by running:

```sh
nix build .\#uniond
```

Which will build the `uniond` binary for the current architecture.

### Usage

For an overview of the commands and usage, run:

```sh
/path/to/uniond --help
```

The commands are self-explanatory and can be used to both run a node and interact with the network over a command-line interface.

### Production Usage

When running `uniond` in production, we recommend using [`unionvisor`](../unionvisor/README.md).
