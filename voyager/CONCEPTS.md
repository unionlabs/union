---
title: Concepts
---

# Concepts

![voyager ibc architecture](./doc/ibc-architecture.svg)

## Modules and Plugins

All functionality in voyager is provided by modules and plugins. Modules provide various forms of read-only data, such as the latest height of a chain or a state proof. Plugins, on the other hand, directly interact with the queue (see [*Plugins and the queue*](#plugins-and-the-queue) for more information).

## Types

### IBC Specification

An IBC specification defines the semantics of a light client based bridging protocol. A specification must have the following properties:

- Some notion of a "light client update"
- A store specification, where client and consensus states are stored (among any other states required by the IBC specification)
- This store is required to be provable, i.e. the host environment must have some form of "proof" for it's storage. This is most likely achieved via a merkleized state trie, although this is not strictly required.

Everything else is an implementation detail of the IBC specification. This flexibility allows voyager to trivially support other IBC-like protocols, such as traditional IBC (referred to as `ibc-classic` throughout these docs) alongside `ibc-union`.

### Chain

A chain is defined by a few basic properties:

- Produces blocks with an incrementing height (sometimes also referred to as "block number" or "slot")
- A consensus with some notion of finality, where blocks older than the latest finalized height will never reorg and are considered finalized
- A storage layer with provable state
- One or more IBC interfaces

### Consensus

A chain's consensus defines the client and consensus state types stored in the clients that verify said consensus mechanism.

#### Examples

- `cometbls`
- `tendermint`
- `ethereum`

### IBC Interface

An IBC interface defines the entrypoints of an IBC specification implementation on a chain. A chain can potentially have many different IBC interfaces (for example, `ibc-go` native clients vs. `08-wasm` clients), and a consensus can be verified by the same client specification on different IBC interfaces.

#### Examples

- `ibc-go-v8/08-wasm`
- `ibc-solidity`
- `ibc-cosmwasm`

### Client Type

Clients are the mechanism used to verify a counterparty consensus. Clients are defined by 4 properties:

- Compatible with an IBC specification
- On an IBC interface
- Verifies a specific consensus mechanism
- For a specific IBC specification

#### Examples

| IBC interface       | consensus                | verifier                 | IBC Specification  |
|---------------------|--------------------------|--------------------------|--------------------|
| `ibc-cosmwasm`      | `cometbls`               | `cometbls-groth16`       | `ibc-union`        |
| `ibc-cosmwasm`      | `tendermint`             | `tendermint`             | `ibc-union`        |
| `ibc-cosmwasm`      | `ethereum`               | `ethereum-sync-protocol` | `ibc-union`        |
| `ibc-solidity`      | `state-lens/ics23/ics23` | `state-lens/ics23/ics23` | `ibc-union`        |
| `ibc-solidity`      | `cometbls`               | `cometbls-groth16`       | `ibc-union`        |
| `ibc-go-v8/08-wasm` | `tendermint`             | `07-tendermint`          | `ibc-union`        |
| `ibc-go-v8/08-wasm` | `cometbls`               | `cometbls-groth16`       | `ibc-classic`      |
| `ibc-go-v8/08-wasm` | `cometbls`               | `11-cometbls`            | `ibc-classic`      |
| `ibc-go-v8/native`  | `cometbls`               | `cometbls-groth16`       | `ibc-classic`      |
| `ibc-go-v8/native`  | `tendermint`             | `07-tendermint`          | `ibc-classic`      |

## Features

The voyager binary exposes a JSON-RPC interface to allow for querying any configured chain. For example, you can query the state of any client on any chain, as long as the state module for the host chain is configured (using the voyager binary's cli):

```sh
voyager rpc -r voy.run client-state union-1 1 --height 2194359 | jq
```

```json
{
  "height": "2194359",
  "state": "0x000000000100000000000000010000004b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe955730c65f000000001c6e5f0100000000000000000000000000ee4ea8d358473f0fcebf0329feed95d56e8c04d700"
}
```

If there is a finality module configured for the host chain as well, then `--height` can be omitted (as it will default to `latest`):

```sh
voyager rpc -r voy.run client-state union-1 1 | jq
```

```json
{
  "height": "1-2194387",
  "state": "0x000000000100000000000000010000004b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe955730c65f000000001c6e5f0100000000000000000000000000ee4ea8d358473f0fcebf0329feed95d56e8c04d700"
}
```

And finally, if the client module is configured for whatever type of client this is (in this case, it happens to be `ethereum` on `ibc-cosmwasm`), `--decode` can be passed as well to receive the client state as a JSON value instead of the raw bytes:

```sh
voyager rpc -r voy.run client-state union-1 1 --decode | jq
```

```json
{
  "height": "1-2194412",
  "state": {
    "data": {
      "chain_id": 1,
      "chain_spec": "mainnet",
      "frozen_height": "0",
      "genesis_time": 1606824023,
      "genesis_validators_root": "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95",
      "ibc_contract_address": "0xee4ea8d358473f0fcebf0329feed95d56e8c04d7",
      "latest_height": 23031324
    },
    "version": "v1"
  }
}
```

This general concept of modularity is present in all areas of voyager. As another example, many EVM chains (various EVM L2s, custom geth fork L1s such as BSC, or fully custom EVM-compatible chains such as SEI), many of the interfaces are the exact same as ethereum mainnet. In these cases, the ethereum state module can be completely reused for these chains, just configured with a different chain ID and RPC url. The same applies to all modules, meaning that when adding support to voyager for a new chain, often times a vast majority of the work required can be fully reused from existing plugins and modules.

## Plugins and the queue

Plugins are a special type of module that also have access to the message queue. Every plugin has their own [topic queue](../lib/voyager-vm/README.md) with it's plugin name as the topic, along with an interest filter that can pull messages into this queue. Plugins also define their own internal message types that they can use to pass data around between calls to their internal queue (or even between other plugins).

For more information about plugin lifecycle and management, see the [`voyager-plugin-protocol`](../lib/voyager-plugin-protocol) crate.

## Putting it all together

The ability to query any chain in an abstract manner also drastically improves the DX and reliability of writing new plugins and modules. One area in particular where this architecture shines is when dealing with [recursive clients] (sometimes also referred to as conditional clients). Recursive clients inherently rely on state from other chains, such as L2 settlement in relation to L1 finality for the L2 finality, or requiring potentially multiple clients to be updated before the recursive client itself can be updated.

A good example of this is our [state lens client architecture][state-lens], where many modules are fully reused from existing modules. The finality of a state lens client is the finality of the "L2" client being tracked through the hop chain - this means that no additional module is required for finality, as the target chain's finality module will be used directly. Additionally, no *new* state or proof modules are required to be loaded when dealing with state lens clients, since these modules will need to be loaded for the host chain where the state lens client is on anyways. There are, however, several new plugins and modules that are required for this architecture to work:

- **Client Module**: This is standard for all new client types. The client module provides the coded for encoding and decoding various states for this client.

- **Client Bootstrap Module**: Similar to the client module, this is also standard for all new client types, however this is only required for creating new clients.

- **Client Update Plugin**: This is the most complex part of the state lens architecture. Up to two individual client updates are required to update a state lens client: the L2 client on the L1 and the L1 client on the L0 (the host chain).

  This is trivially achieved by leveraging the voyager-vm messages:

  ```rs
  // do all contained operations concurrently
  conc([
      // update the l2 client on the l1
      promise(
          [
              // fetch the update headers of the l2 client
              call(FetchUpdateHeaders { /* snip */ })
          ],
          // this is the data queue of the promise callback, this allows for configuring data on creation of the promise
          // in this case, there is no extra data, so it can be left empty
          [],
          // this is the callback that will process the data once all messages in the internal queue are processed
          AggregateSubmitTxFromOrderedHeaders { /* snip */ },
      ),
      // do all contained operations in sequence, waiting until the head message fully resolves (i.e. returns no additional non-data messages) before processing the next message
      seq([
          // wait for the trusted height of the client we just updated to be finalized on the hop chain
          // without this, weird things can happen with transaction ordering and reorgs
          call(WaitForTrustedHeight { /* snip */ }),
          // call back into this plugin to update the other clients
          call(PluginMessage::new(
              self.plugin_name(),
              ModuleCall::from(FetchUpdateAfterL1Update { /* snip */ }),
          ))
      ]),
  ])
  ```

  The handling of `FetchUpdateAfterL1Update` is as follows:

  ```rs
  conc([
      // this promimse is the same as the one above, except this time we're updating the L1 client on the L0
      promise(
          [call(FetchUpdateHeaders { /* snip */ })],
          [],
          AggregateSubmitTxFromOrderedHeaders { /* snip */ },
      ),
      seq([
          call(WaitForTrustedHeight { /* snip */ }),
          // wait for 1 extra block to ensure that the L1 update is in state, and this update will not end up in the same block (and potentially get reordered)
          call(WaitForHeightRelative { /* snip */ }),
          // this contains the actual headers for *this* client update.
          data(OrderedHeaders { /* snip */ }),
      ]),
  ])
  ```

  In building these messages, several additional modules and plugins are also needed. To update the L2 on the L1, the L2 client update plugin is required (as well as all of it's transitive requirements), and the same goes for the L1 on the L0. Additionally, in order to actually *submit* these intermediate client updates on chain, transactiopn plugins for both the L1 and L0 are required to be loaded. All of state, proof, and finality modules are also required to be loaded for the L1 as well (recall that the client update of the state lens client contains a state proof of the L2 state in the L1).

  This may seem like a lot of requirements, however remember that all of the dependencies listed above were in this case already written - all that needed to be done was to configure them for the chains we need to use here, and to build the state lens client logic only 1 plugin (client update) and 2 modules (client and client bootstrap) needed to be written from scratch. The same concepts, with differing degrees of reusability, apply to L2s (arbitrum, optimism, various types of rollups), customized execution environments (SEI/ethermint), novel consensus mechanisms (beacon-kit), and even entirely new chains (sui, aptos).

  The full non-abridged implementation of the state lens client update plugin can be found [here](../voyager/plugins/client-update/state-lens).

[recursive clients]: https://docs.union.build/protocol/connections/recursive
[state-lens]: https://research.union.build/State-Lenses-9e3d6578ec0e48fca8e502a0d28f485c
