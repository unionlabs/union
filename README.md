[![Build](https://github.com/UnionFi/union/actions/workflows/build.yml/badge.svg)](https://github.com/UnionFi/union/actions/workflows/build.yml)

Union is a trust-minimized, zero-knowledge bridging protocol, designed for censorship resistance, extremely high security, and usage in decentralized finance. It implements [IBC](https://github.com/cosmos/ibc) for compatibility with Cosmos chains and connects to Ethereum. 

Most bridging protocols use a form of multi-signature verification across validators (Wormhole, Axelar) or even a fully trusted third party like Circle's cross-chain transfer protocol (CCTP). These methods are less secure and more prone to hacks and censorship. Union uses consensus verification based on zero-knowledge proofs, which provide the highest level of security currently known.

Although IBC originated from the Cosmos ecosystem, we have extended it to Ethereum and will be connecting to other ecosystems like Bitcoin.

Union is designed to be completely decentralized. The upgradability of contracts on other chains, connections, token configurations, and evolution of the protocol will all be controlled by decentralized governance, aligning the priorities of Union with its users, validators, and operators.

## Components

This repository hosts a few core components of Union.

- [`uniond`](./uniond): The node implementation, using [`CometBLS`](https://github.com/unionfi/cometbls).
- [`unionpd`](./unionpd/): The zero-knowledge prover implementation.
- [`unionvisor`](./unionvisor): A node supervisor intended for production usage.

You can find these components in the [releases](https://github.com/unionfi/union/releases).

## Documentation

The documentation is not publicly hosted yet. Check out the docs [here](./docs)

## Contributing

The [contributing](./CONTRIBUTING.md) guide explains how to get started working on Union and its components.