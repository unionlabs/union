---
sidebar_position: 1
---

# Specifications

This directory contains specifications for the modules of the Cosmos SDK as well as Interchain Standards (ICS) and other specifications.

Cosmos SDK applications hold this state in a Merkle store. Updates to
the store may be made during transactions and at the beginning and end of every
block.

## Cosmos SDK specifications

* [Store](./store) - The core Merkle store that holds the state.
* [Bech32](./addresses/bech32.md) - Address format for Cosmos SDK applications.

## Modules specifications

Go the [module directory](https://github.com/cosmos/cosmos-sdk/blob/main/x/README.md)

## CometBFT

For details on the underlying blockchain and p2p protocols, see
the [CometBFT specification](https://github.com/cometbft/cometbft/tree/main/spec).
