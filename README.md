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

Union is a hyper-efficient zero-knowledge infrastructure layer for general message passing, asset transfers, NFTs, and DeFi. It’s based on [Consensus Verification] and has no dependencies on trusted third parties, oracles, multi-signatures, or MPC. It implements [IBC] for compatibility with Cosmos chains and connects to EVM chains like [Ethereum], [Berachain (beacon-kit)](https://github.com/berachain/beacon-kit), [Arbitrum], and more.

The upgradability of contracts on other chains, connections, token configurations, and evolution of the protocol will all be controlled by decentralized governance, aligning the priorities of Union with its users, validators, and operators.

## Components

| Component                          | Description                                        | Language(s)           |
| ---------------------------------- | -------------------------------------------------- | --------------------- |
| [`uniond`](./uniond)               | The Union node implementation, using [`CometBLS`]  | [Go]                  |
| [`galoisd`](./galoisd)             | The zero-knowledge prover implementation           | [Go] [Gnark]          |
| [`voyager`](./voyager)             | Modular hyper-performant cross-ecosystem relayer   | [Rust]                |
| [`unionvisor`](./unionvisor)       | Node supervisor intended for production usage      | [Rust]                |
| [`cosmwasm`](./cosmwasm)           | [CosmWasm] smart contract stack                    | [Rust]                |
| [`light-clients`](./light-clients) | [Light Clients] for various ecosystems             | [Rust]                |
| [`evm`](./evm)                     | [EVM] smart contract stack                         | [Solidity]            |
| [`app`](./app)                     | [app.union.build](https://app.union.build)         | [TypeScript] [Svelte] |
| [`site`](./site)                   | [union.build](https://union.build)                 | [TypeScript] [Astro]  |

## Quickstart

Install [Nix] to _[reproducibly build](https://en.wikipedia.org/wiki/Reproducible_builds) any component_, and to enter a dev shell with _all dependencies_:
```
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```
_(Note that some components can only be built on Linux. If you are using macOS, we recommend using [OrbStack] to easily set up a [NixOS] VM within two minutes. Most Union developers use macOS with [OrbStack], and there is no need to install Nix inside of the [NixOS] VM.)_

You can now _reproducibly_ build any of Union's components from source:

```sh
nix build .#uniond -L
nix build .#voyager -L
nix build .#app -L

# to see all packages, run:
nix flake show
```

The result of whatever you build will be in `result/`

You can now also enter our dev shell, which has all of the dependencies (`cargo`, `rustc`, `node`, `go`, etc.) you need to work on any component: 
_(Don't worry, this will not affect your system outside of this repo)_

```sh
nix develop
```

Check the `#developers` channel on [Union's discord](https://discord.union.build) if you need any help with this.

## Docs

The official docs are hosted [here][docs]. Each individual component also has accompanying developer documentation for contributors, which you can find in each `README.md`.

[docs]: https://docs.union.build "Official Union Docs"
[IBC]: https://github.com/cosmos/ibc "cosmos/ibc"
[Discord badge]: https://img.shields.io/discord/1158939416870522930?logo=discord
[Twitter handle]: https://img.shields.io/twitter/follow/union_build.svg?style=social&label=Follow
[Twitter badge]: https://twitter.com/intent/follow?screen_name=union_build
[CosmWasm]: https://cosmwasm.com/
[Arbitrum]: https://github.com/OffchainLabs/arbitrum
[Ethereum]: https://ethereum.org
[EVM]: https://ethereum.org/en/developers/docs/evm/
[Rust]: https://www.rust-lang.org/
[Solidity]: https://soliditylang.org/
[Go]: https://go.dev/
[TypeScript]: https://www.typescriptlang.org/
[Svelte]: https://svelte.dev
[Astro]: https://astro.build
[`CometBLS`]: https://github.com/unionlabs/cometbls
[Light Clients]: https://a16zcrypto.com/posts/article/an-introduction-to-light-clients/
[Gnark]: https://github.com/ConsenSys/gnark
[Nix]: https://zero-to-nix.com/
[NixOS]: https://nixos.org
[OrbStack]: https://orbstack.dev/
[Consensus Verification]: https://union.build/docs/concepts/consensus-verification/
