# Union

[![Docs](https://img.shields.io/badge/docs-main-blue)][docs]


Most bridging protocols use a form of multi-signature verification across validators (Wormhole, Axelar) or even a fully trusted third party like Circle's cross-chain transfer protocol (CCTP). These methods are less secure and more prone to hacks and censorship. Union uses consensus verification based on zero-knowledge proofs, which provides the highest level of security currently known.


Union is designed to be completely decentralized. The upgradability of contracts on other chains, connections, token configurations, and evolution of the protocol will all be controlled by decentralized governance, aligning the priorities of Union with its users, validators, and operators.


This repository hosts a few core components of Union.

- [`uniond`](./uniond): The node implementation, using [`CometBLS`](https://github.com/unionlabs/cometbls).
- [`galoisd`](./galoisd/): The zero-knowledge prover implementation.
- [`unionvisor`](./unionvisor): A node supervisor intended for production usage.

You can find these components in the [releases](https://github.com/unionlabs/union/releases).

## Documentation

The official docs are hosted [here][docs]. Each individual component also has accompanying developer documentation for contributors, which you can find in each `README.md`.


The [contributing](./CONTRIBUTING.md) guide explains how to get started working on Union and its components.
Please make sure to [set up your GitHub PAT](<https://github.com/unionlabs/union/wiki/Personal-Access-Token-(PAT)-Setup>), otherwise Nix builds will fail.

[docs]: https://docs.union.build "Official Union Docs"
[IBC]: https://github.com/cosmos/ibc "cosmos/ibc"
