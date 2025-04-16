# Union IBC Deployments

This directory contains information about our Union IBC deployments.

## deployments.json

Deployed contract addresses of the Union IBC stack.

The format of the file is as follows:

- `universal_chain_id`: The [UCS04] universal chain id for this chain.

- `chain_id`: The (potentially non-unique) chain id for this chain.

- `ibc_interface`: The [IBC interface] this deployment is on.

- `deployments`: The deployed contract addresses. it is expected to have the following fields:

  - `core`: The core IBC handler stack.
  - `lightclient`: An object containing all lightclients registered to the stack, keyed by the client type.
  - `app`: An object containing all apps deployed on this chain, keyed by the UCS identifier.

  All leaf objects must have the following fields:

  - `address`: The contract address.
  - `height`: The height that the contract was first uploaded at.
  - `commit`: The commit of <https://github.com/unionlabs/union> of the latest deployment of this contract.

  Any other fields may also be included as necessary at the root of the deployments object (i.e. alongside `core`, `lightclient`, and `app`).

Deployments can be updated by running the following command from the root of the repo:

```sh
nix run .#update-deployments-json
```

Note that it is also possible to update either EVM or CosmWasm deployments individually:

```sh
# for cosmwasm
nix run .#cosmwasm-scripts.update-deployments-json

# for evm
nix run .#evm-scripts.update-deployments-json
```

[ibc interface]: ../voyager/CONCEPTS.md#ibc-interface
[ucs04]: https://docs.union.build/ucs/04/
