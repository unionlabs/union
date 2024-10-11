---
sidebar_position: 0
---
<!-- markdown-link-check-disable -->
# List of Modules

Here are some production-grade modules that can be used in Cosmos SDK applications, along with their respective documentation:

* [Auth](./auth/README.md) - Authentication of accounts and transactions for Cosmos SDK applications.
* [Authz](./authz/README.md) - Authorization for accounts to perform actions on behalf of other accounts.
* [Bank](./bank/README.md) - Token transfer functionalities.
* [Circuit](./circuit/README.md) - Circuit breaker module for pausing messages.
* [Consensus](./consensus/README.md) - Consensus module for modifying CometBFT's ABCI consensus params.
* [Distribution](./distribution/README.md) - Fee distribution, and staking token provision distribution.
* [Epochs](./epochs/README.md) - Allow other modules to set that they would like to be signaled once every period
* [Evidence](./evidence/README.md) - Evidence handling for double signing, misbehaviour, etc.
* [Feegrant](./feegrant/README.md) - Grant fee allowances for executing transactions.
* [Genutil](./genutil/README.md) - Genesis utilities for the Cosmos SDK.
* [Governance](./gov/README.md) - On-chain proposals and voting.
* [Mint](./mint/README.md) - Creation of new units of staking token.
* [NFT](./nft/README.md) - NFT module implemented based on [ADR43](https://docs.cosmos.network/main/build/architecture/adr-043-nft-module).
* [Params](./params/README.md) - Globally available parameter store.
* [Protocolpool](./protocolpool/README.md) - Functionalities handling community pool funds.
* [Slashing](./slashing/README.md) - Validator punishment mechanisms.
* [Staking](./staking/README.md) - Proof-of-Stake layer for public blockchains.
* [tx](./tx/README.md) - Tx utilities for the Cosmos SDK.
* [Upgrade](./upgrade/README.md) - Software upgrades handling and coordination.
* [Validate](./validate/README.md) - Global ante/post handlers and tx validator setup.

To learn more about the process of building modules, visit the [building modules reference documentation](https://docs.cosmos.network/main/building-modules/intro).

## IBC

The IBC module for the SDK is maintained by the IBC Go team in its [own repository](https://github.com/cosmos/ibc-go).

Additionally, the [capability module](https://github.com/cosmos/ibc-go/tree/fdd664698d79864f1e00e147f9879e58497b5ef1/modules/capability) is from v0.50+ maintained by the IBC Go team in its [own repository](https://github.com/cosmos/ibc-go/tree/fdd664698d79864f1e00e147f9879e58497b5ef1/modules/capability).

## CosmWasm

The CosmWasm module enables smart contracts, learn more by going to their [documentation site](https://book.cosmwasm.com/), or visit [the repository](https://github.com/CosmWasm/cosmwasm).

## EVM

Read more about writing smart contracts with solidity at the official [`evm` documentation page](https://docs.evmos.org/).
