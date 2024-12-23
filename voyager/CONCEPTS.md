# Concepts

![voyager ibc architecture](./doc/ibc-architecture.svg)

## Modules and Plugins

All functionality in voyager is provided by modules and plugins. Modules provide various forms of read-only data, such as the latest height of a chain or a state proof. Plugins, on the other hand, directly interact with the queue - every plugin has their own [topic queue](../lib/voyager-vm/README.md) with it's plugin name as the topic, along with an interest filter that can pull messages into this queue. Plugins also define their own internal message types that they can use to pass data around between calls to their internal queue (or even between other plugins).

## Types

### IBC Specification

An IBC specification defines the semantics of a light client based bridging protocol. A specification must have the following properties:

- some notion of a "light client update"
- a store specification, where client and consensus states are stored (among any other states required by the IBC specification)
- this store is required to be provable (i.e. the host environment must have some form of "proof" for it's storage, most likely merkleized)

Everything else is an implementation detail of the ibc specification.

### Chain

A chain is defined by a few basic properties:

- produces blocks with an incrementing height (sometimes also referred to as "block number" or "slot")
- a consensus with some notion of finality, where blocks older than the latest finalized height will never reorg and are considered finalized
- a storage layer with provable state
- one or more IBC interfaces

### Consensus

A chain's consensus defines the client and consensus state types stored in the clients that verify this consensus.

#### Examples

- cometbls
- tendermint
- ethereum

### IBC Interface

An IBC interface defines the entrypoints of an IBC specification implementation on a chain. A chain can potentially have many different IBC interfaces (for example, `ibc-go` native clients vs. `08-wasm` clients), and a consensus can be verified by the same client specification on different IBC interfaces.

#### Examples

- ibc-go-v8/08-wasm
- ibc-solidity
- ibc-cosmwasm

### Client

Clients are the mechanism used to verify a counterparty consensus. Clients are defined by 4 properties:

- compatible with an IBC specification
- on an IBC interface
- for a specific consensus mechanism
- which is verified via a consensus verification specification

#### Examples

| IBC interface     | consensus  | verifier         |
|-------------------|------------|------------------|
| ibc-go-v8/08-wasm | cometbls   | cometbls-groth16 |
| ibc-go-v8/08-wasm | cometbls   | 11-cometbls      |
| ibc-go-v8/native  | cometbls   | cometbls-groth16 |
| ibc-solidity      | cometbls   | cometbls-groth16 |
| ibc-go-v8/native  | tendermint | 07-tendermint    |
| ibc-go-v8/08-wasm | tendermint | 07-tendermint    |
