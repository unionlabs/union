<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./.github/images/union-logo-white.svg">
    <source media="(prefers-color-scheme: light)" srcset="./.github/images/union-logo-black.svg">
    <img alt="Union"
         src="./.github/images/union-logo-black.svg"
         width="50%">
  </picture>

  [![Build](https://github.com/unionlabs/union/actions/workflows/build.yml/badge.svg)](https://github.com/unionlabs/union/actions/workflows/build.yml)
  [![Docs](https://img.shields.io/badge/docs-main-blue)][docs]
  [![Discord badge][]](https://discord.union.build)
  [![Twitter handle][]][Twitter badge]
</div>


Union is a trust-minimized, decentralized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security, and usage in decentralized finance. It implements [IBC] for compatibility with Cosmos chains and connects to EVM chains like Ethereum, Berachain, Arbitrum, and more.

The upgradability of contracts on other chains, connections, token configurations, and evolution of the protocol will all be controlled by decentralized governance, aligning the priorities of Union with its users, validators, and operators.

## Components

This repository hosts a all core components of Union.

| Component                    | Description                                        | Language(s) |
| ---------------------------- | -------------------------------------------------- | ----------- |
| [`uniond`](./uniond)         | The Union node implementation, using [`CometBLS`]. | [Go]        |
| [`galoisd`](./galoisd)       | The zero-knowledge prover implementation.          | [Go]        |
| [`voyager`](./voyager)       | Modualar hyper-performant cross-ecosystem relayer. | [Rust]      |
| [`unionvisor`](./unionvisor) | Node supervisor intended for production usage      | [Rust]      |
| [`cosmwasm`](./cosmwasm)     | [CosmWasm] smart contract stack.                   | [Rust]      |
| [`evm`](./evm)               | [EVM] smart contract stack.                        | [Solidity]  |

You can find these components in the [releases](https://github.com/unionlabs/union/releases).

## Documentation

The official docs are hosted [here][docs]. Each individual component also has accompanying developer documentation for contributors, which you can find in each `README.md`.

## Contributing

The [contributing](./CONTRIBUTING.md) guide explains how to get started working on Union and its components.
Please make sure to [set up your GitHub PAT](<https://github.com/unionlabs/union/wiki/Personal-Access-Token-(PAT)-Setup>), otherwise Nix builds will fail.

[docs]: https://docs.union.build "Official Union Docs"
[IBC]: https://github.com/cosmos/ibc "cosmos/ibc"
[Discord badge]: https://img.shields.io/discord/1158939416870522930?logo=discord
[Twitter handle]: https://img.shields.io/twitter/follow/union_build.svg?style=social&label=Follow
[Twitter badge]: https://twitter.com/intent/follow?screen_name=union_build
[CosmWasm]: https://cosmwasm.com/
[EVM]: https://ethereum.org/en/developers/docs/evm/
[Rust]: https://www.rust-lang.org/
[Solidity]: https://soliditylang.org/
[Go]: https://go.dev/
[`CometBLS`]: https://github.com/unionlabs/cometbls
