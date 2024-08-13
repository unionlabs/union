<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/unionlabs/union/6f464a971875744b7ae149183f13c0e4f86bd5a3/.github/images/union-logo-white.svg">
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/unionlabs/union/6f464a971875744b7ae149183f13c0e4f86bd5a3/.github/images/union-logo-black.svg">
    <img alt="Union"
         src="https://raw.githubusercontent.com/unionlabs/union/6f464a971875744b7ae149183f13c0e4f86bd5a3/.github/images/union-logo-black.svg"
         width="50%">
  </picture>
</div>

# Union

[![Build](https://github.com/unionlabs/union/actions/workflows/build.yml/badge.svg)](https://github.com/unionlabs/union/actions/workflows/build.yml)
[![Docs](https://img.shields.io/badge/docs-main-blue)][docs]
[![Discord badge][]](https://discord.union.build)
[![Twitter handle][]][Twitter badge]

Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security, and usage in decentralized finance. It implements [IBC] for compatibility with Cosmos chains and connects to Ethereum.

Most bridging protocols use a form of multi-signature verification across validators (Wormhole, Axelar) or even a fully trusted third party like Circle's cross-chain transfer protocol (CCTP). These methods are less secure and more prone to hacks and censorship. Union uses consensus verification based on zero-knowledge proofs, which provides the highest level of security currently known.

Although IBC originated from the Cosmos ecosystem, we have extended it to Ethereum and will be connecting to other ecosystems like Bitcoin.

Union is designed to be completely decentralized. The upgradability of contracts on other chains, connections, token configurations, and evolution of the protocol will all be controlled by decentralized governance, aligning the priorities of Union with its users, validators, and operators.

## Components

This repository hosts a few core components of Union.

- [`uniond`](./uniond): The node implementation, using [`CometBLS`](https://github.com/unionlabs/cometbls).
- [`galoisd`](./galoisd/): The zero-knowledge prover implementation.
- [`unionvisor`](./unionvisor): A node supervisor intended for production usage.

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
