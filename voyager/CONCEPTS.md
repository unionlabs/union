# Concepts

![voyager ibc architecture](./doc/ibc-architecture.svg)

## Chain

A chain is defined by a few basic properties:

- blocks with an incrementing height (sometimes also referred to as block number or slot)
- a consensus with some notion of finality, where blocks older than the latest finalized height will never reorg and are considered finalized
- a storage layer with provable state
- one or more IBC interfaces

## Consensus

A chain's consensus defines the client and consensus state types stored in the clients that verify this consensus.

### Examples

- cometbls
- tendermint
- ethereum

## IBC Interface

An IBC interface defines the entrypoints and semantics of an IBC implementation on a chain. A chain can potentially have many different IBC interfaces (for example, `ibc-go` native clients vs. `08-wasm`), and a consensus can be verified by the same client specification on different IBC interfaces.

### Examples

- ibc-go-v8/08-wasm
- ibc-solidity

## Client

Clients are the mechanism used to verify a counterparty consensus. Clients are defined by 3 properties:

- on an IBC interface
- for a specific consensus mechanism
- which is verified via a consensus verification specification

### Examples

| IBC interface     | consensus  | verifier         |
|-------------------|------------|------------------|
| ibc-go-v8/08-wasm | cometbls   | cometbls-groth16 |
| ibc-go-v8/08-wasm | cometbls   | 11-cometbls      |
| ibc-go-v8/native  | cometbls   | cometbls-groth16 |
| ibc-solidity      | cometbls   | cometbls-groth16 |
| ibc-go-v8/native  | tendermint | 07-tendermint    |
| ibc-go-v8/08-wasm | tendermint | 07-tendermint    |
