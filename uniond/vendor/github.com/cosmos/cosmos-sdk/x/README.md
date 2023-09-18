---
sidebar_position: 0
slug : /modules
---

# List of Modules

Here are some production-grade modules that can be used in Cosmos SDK applications, along with their respective documentation:

* [Auth](./auth/README.md) - Authentication of accounts and transactions for Cosmos SDK applications.
* [Authz](./authz/README.md) - Authorization for accounts to perform actions on behalf of other accounts.
* [Bank](./bank/README.md) - Token transfer functionalities.
* [Capability](./capability/README.md) - Object capability implementation.
* [Crisis](./crisis/README.md) - Halting the blockchain under certain circumstances (e.g. if an invariant is broken).
* [Distribution](./distribution/README.md) - Fee distribution, and staking token provision distribution.
* [Evidence](./evidence/README.md) - Evidence handling for double signing, misbehaviour, etc.
* [Feegrant](./feegrant/README.md) - Grant fee allowances for executing transactions.
* [Governance](./gov/README.md) - On-chain proposals and voting.
* [Mint](./mint/README.md) - Creation of new units of staking token.
* [Params](./params/README.md) - Globally available parameter store.
* [Slashing](./slashing/README.md) - Validator punishment mechanisms.
* [Staking](./staking/README.md) - Proof-of-Stake layer for public blockchains.
* [Upgrade](./upgrade/README.md) - Software upgrades handling and coordination.
* [NFT](./nft/README.md) - NFT module implemented based on [ADR43](https://docs.cosmos.network/main/architecture/adr-043-nft-module.html).
* [Consensus](./consensus/README.md) - Consensus module for modifying CometBFT's ABCI consensus params.

To learn more about the process of building modules, visit the [building modules reference documentation](https://docs.cosmos.network/main/building-modules/intro).

## IBC

The IBC module for the SDK has moved to its [own repository](https://github.com/cosmos/ibc-go).

## CosmWasm

The CosmWasm module enables smart contracts, and has its [own repository](https://github.com/CosmWasm/cosmwasm) and [documentation site](https://docs.cosmwasm.com/docs).
