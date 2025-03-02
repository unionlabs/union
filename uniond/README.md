# uniond

**uniond** is the canonical implementation of a full node for the union network. Validators, RPC, and archive operators can run it to participate in the network.

## Getting started

The easiest way to obtain the `uniond` binary is to check the [releases](https://github.com/unionlabs/union/releases). Alternatively, they can be built from source by running:

```sh
nix build .#uniond
```

### Usage

For an overview of the commands and usage, run:

```sh
/path/to/uniond --help
```

The commands are self-explanatory and can be used to both run a node and interact with the network over a command-line interface.

### Production Usage

When running `uniond` in production, we recommend using [`unionvisor`](../unionvisor/README.md).

## Architecture

Uniond is a [Cosmos SDK](https://github.com/cosmos/cosmos-sdk) based blockchain. Outside of the core Cosmos SDK modules, uniond also utilizes [`Wasmd`](https://github.com/cosmwasm/wasmd) to host our virtualized IBC-Union stack. We also support the [`ibc-go`](https://github.com/cosmos/ibc-go) module for native connections.

### Wasmd

To enable the deployment of CosmWasm contracts, including the virtualized IBC-Union stack, the Union network includes `Wasmd`. Through `Wasmd`, union has access to the `Wasmvm`. Union's mainnet is shipped with `Wasmvm` 2.1.2.

### PoA

Union will temporarily utilize [PoA](https://github.com/strangelove-ventures/poa/) during the incubation stage of mainnet. Once the Union mainnet is ready for the public, Union will [migrate to Proof of Stake from Proof of Authority](https://github.com/strangelove-ventures/poa/blob/34aee49474018a4035fecbe676b765c2717d78aa/INTEGRATION.md#migrating-to-pos-from-poa).
