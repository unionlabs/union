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

  Additional fields may also be included as necessary at both the root of the deployments object (i.e. alongside `core`, `lightclient`, and `app`) and in the leaf objects (i.e. alongside `address`, `height`, and `commit`).

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

  - `forward`: Maximum duration when sending _from_ the specified chain (ie. send packet was emitted on this chain).
  - `reverse`: Maximum duration when sending _to_ the specified chain (ie. send packet was emitted on the counterparty chain).

- `comments`: Arbitrary text describing the channel. This text is not parsed, but new entries should follow a consistent prose as existing entries.

Note that to prevent redundancy and reduce the potential for copy-paste errors, only the *init* side of the channel is stored in this file. Reference our [channels docs] for a full overview of all channels with all relevant information.

Both the universal chain id and the channel id sub-keys should be sorted (the former alphabetically, the latter numerically). This is handled by running `nix fmt` in the root of the project.

## editions.json

Union supports multiple editions of the app, each tailored to a specific target audience. Each edition maps to a subdomain and defines a set of chains that are active within that edition.

The structure is as follows:

- `<edition>`: name of the edition, corresponding to the subdomain `<edition>.union.build`.
  - `chains`: mapping of universal chain identifiers to their configuration.
    - `<universal_chain_id>`: [UCS04] chain id of the chain that is in scope for this edition.
      - `environment`: Specifies the visibility of the chain within the given edition (see [environments](#environments) below).

### environments

The environment determines where the chain is available. Scope increases with each level: `production` includes everything in `staging`, which includes everything in `development`.

Allowed values are:

- `development` – Available only in the development environment.
- `staging` – Available in both development and staging environments.
- `production` – Available in all environments.

Both the editions and the chain sub-keys should be sorted. This is handled by running `nix fmt` in the root of the project.

## universal-chain-ids.json

As described in [UCS04], Union uses a two-part identifier — chain family name and chain ID — to uniquely identify a network. This helps prevent ambiguity across environments, testnets, and mainnets.

This file defines all known universal chain IDs used in the Union ecosystem. It provides the canonical mapping from chain family to their supported chain IDs and serves as a reference for disambiguating chain identifiers across environments.

The structure is as follows:

- `<chain family name>`: The name of the chain family (e.g. `ethereum`, `babylon`).
  - A list of supported chain IDs for that chain. These IDs are used to form **universal chain IDs** by combining the chain name and ID, e.g. `ethereum.1`, `babylon.bbn-test-5`.

## token-whitelist.json

This file defines whitelisted tokens for each chain. Only transfers of tokens listed in this file are permitted within the Union ecosystem. It acts as an access control mechanism, ensuring whitelisted assets are recognized and processed.

The structure is as follows:

- `<universal_chain_id>`: A fully qualified chain identifier (see [UCS04]), such as `ethereum.1` or `babylon.bbn-1`.
  - `<token address>`: The **canonical** address of the token on that chain, as defined in [UCS05]. All addresses are hex-encoded.
    - `comments`: Arbitrary text describing the token. This text is not and should not be parsed; it is meant for human readers only.

[channels docs]: https://docs.union.build/protocol/channels/overview/
[graphql]: https://docs.union.build/integrations/api/graphql/
[ibc interface]: ../voyager/CONCEPTS.md#ibc-interface
[ucs03]: https://docs.union.build/ucs/03/
[ucs04]: https://docs.union.build/ucs/04/
[ucs05]: https://docs.union.build/ucs/05/
