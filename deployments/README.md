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

## evm-deployer.json

Deployed instances of our deterministic EVM contract deployer. The file is an object mapping [UCS04] chain id to deployed address. See [../tools/build-evm-deployer-tx/README.md](../tools/build-evm-deployer-tx/README.md) for more information.

## clients.json

Well-known and maintained IBC light clients.

The file maps [UCS04] chain ids to an object containing clients on that chain. The format of the client info objects is as follows:

- `counterparty`: The [UCS04] universal chain id of the chain being tracked by this client.

- `refresh_rate`: The amount of blocks that this client can lag behind the finalized head before it should be updated. This is the refresh rate we use to guarantee liveliness of all configured clients. Note that this will be changed to a duration value in the future, since the block time of some chains can vary quite a lot (notably arbitrum chains, that only produce blocks when there are transactions).

## channels.json

Well-known [UCS03] channels.

The file maps [UCS04] chain ids to an object containing channels on that chain. The format of the channel info objects is as follows:

- `tags`: Arbitrary tags describing the channel.
  Currently supported and well-known tags:
  `canonical`: A canonical channel built over one of the clients in `clients.json`. This channel can safely be used by third party applications, and is exposed through our [GraphQL] API.

- `sla`: Maximum time (in ISO 8601) between a packet-send and a packet-ack.

  - `forward`: Maximum duration when sending _from_ the specified chain (ie. from *init* side).
  - `reverse`: Maximum duration when sending _to_ the specified chain (ie. from *try* side).

- `comments`: Arbitrary text describing the channel. This text is not parsed, but new entries should follow a consistent prose as existing entries.

Note that to prevent redundancy and reduce the potential for copy-paste errors, only the *init* side of the channel is stored in this file. Reference our [channels docs] for a full overview of all channels with all relevant information.

Both the universal chain id and the channel id sub-keys should be sorted (the former alphabetically, the latter numerically). This can be achieved by running the following command:

```sh
jq . channels.json -S | sponge channels.json
```

[channels docs]: https://docs.union.build/protocol/channels/overview/
[graphql]: https://docs.union.build/integrations/api/graphql/
[ibc interface]: ../voyager/CONCEPTS.md#ibc-interface
[ucs03]: https://docs.union.build/ucs/03/
[ucs04]: https://docs.union.build/ucs/04/
