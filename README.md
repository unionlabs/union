<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./.github/images/union-logo-white.svg">
    <source media="(prefers-color-scheme: light)" srcset="./.github/images/union-logo-black.svg">
    <img alt="Union"
         src="./.github/images/union-logo-black.svg"
         width="100%">
  </picture>
</div>

<br/>

<div align="center">

[![built with garnix](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fgarnix.io%2Fapi%2Fbadges%2Funionlabs%2Funion%3Fbranch%3Dmain)](https://garnix.io)
[![Docs](https://img.shields.io/badge/docs-main-blue)][docs]
[![Discord badge]](https://discord.union.build)
[![Twitter handle]][twitter badge]

</div>

Union is the hyper-efficient zero-knowledge infrastructure layer for general message passing, asset transfers, NFTs, and DeFi. Its based on [Consensus Verification] and has no dependencies on trusted third parties, oracles, multi-signatures or MPC. It implements [IBC] for compatibility with [Cosmos] chains and connects to EVM chains like [Ethereum], [Berachain (beacon-kit)](https://github.com/berachain/beacon-kit), [Arbitrum], and more.

The upgradability of contracts on other chains, connections, token configurations, and evolution of the protocol will all be controlled by decentralized governance, aligning the priorities of Union with its users, validators, and operators.

## Components

| Component                                           | Description                                          | Language(s)           |
| --------------------------------------------------- | ---------------------------------------------------- | --------------------- |
| [`uniond`](./uniond/README.md)                      | The Union node implementation, using [`CometBLS`]    | [Go]                  |
| [`galoisd`](./galoisd)                              | The zero-knowledge prover implementation             | [Go] [Gnark]          |
| [`voyager`](./voyager)                              | Modular hyper-performant cross-ecosystem relayer     | [Rust]                |
| [`cosmwasm`](./cosmwasm)                            | [CosmWasm] smart contract stack                      | [Rust]                |
| [`light-clients`](./cosmwasm/lightclient)           | [Light Clients] for various ecosystems               | [Rust]                |
| [`unionvisor`](./unionvisor/README.md)              | Node supervisor intended for production usage        | [Rust]                |
| [`drip`](./drip)                                    | Faucet for [Cosmos] chains: [app.union.build/faucet] | [Rust]                |
| [`evm`](./evm)                                      | [EVM] smart contract stack                           | [Solidity]            |
| [`app`](./app2)                                     | [app.union.build]                                    | [TypeScript] [Svelte] |
| [`site`](./site)                                    | [union.build]                                        | [TypeScript] [Astro]  |
| [`TypeScript SDK`](./ts-sdk)                        | TypeScript SDK for interacting with Union            | [TypeScript]          |

## Quickstart

Install [Nix] to _[reproducibly build](https://en.wikipedia.org/wiki/Reproducible_builds) any component_, and to enter a dev shell with _all dependencies_:

```sh
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

Run the following to format the entire repo and check your spelling before each PR:

```sh
nix run .#pre-commit -L
```

Check the `#developers` channel on [Union's discord](https://discord.union.build) if you need any help with this.

## Supported Chains

| Network       | Mainnets                    | Testnets                               |
| ------------- | --------------------------- | -------------------------------------- |
| **Arbitrum**  | `arbitrum.42161`            | `arbitrum.421614`                      |
| **Babylon**   | `babylon.bbn-1`             | `babylon.bbn-test-6`                   |
| **Base**      | `base.8453`                 | `base.84532`                           |
| **Berachain** | `berachain.80094`           | `berachain.80069`                      |
| **Bob**       | `bob.60808`                 | `bob.808813`                           |
| **BSC**       | `bsc.56`                    | `bsc.97`                               |
| **Corn**      | `corn.21000000`             | `corn.21000001`                        |
| **Ethereum**  | `ethereum.1`                | `ethereum.11155111`, `ethereum.560048` |
| **Osmosis**   | `osmosis.osmosis-1`         | `osmosis.osmo-test-5`                  |
| **Sei**       | `sei.pacific-1`, `sei.1329` | `sei.atlantic-2`, `sei.1328`           |
| **Sui**       | -                           | `sui.4c78adac`                         |
| **Union**     | `union.union-1`             | `union.union-testnet-10`               |
| **Xion**      | `xion.xion-mainnet-1`       | `xion.xion-testnet-2`                  |

*For the full list see https://docs.union.build/ucs/04/*

## Docs

The official docs are hosted [here][docs]. Each individual component also has accompanying developer documentation for contributors, which you can find in each `README.md`.

[app.union.build]: https://app.union.build
[app.union.build/faucet]: https://app.union.build/faucet
[arbitrum]: https://github.com/OffchainLabs/arbitrum
[astro]: https://astro.build
[consensus verification]: https://union.build/docs/concepts/consensus-verification/
[cosmos]: https://cosmos.network
[cosmwasm]: https://cosmwasm.com/
[discord badge]: https://img.shields.io/discord/1158939416870522930?logo=discord
[docs]: https://docs.union.build "Official Union Docs"
[ethereum]: https://ethereum.org
[evm]: https://ethereum.org/en/developers/docs/evm/
[gnark]: https://github.com/ConsenSys/gnark
[go]: https://go.dev/
[ibc]: https://github.com/cosmos/ibc "cosmos/ibc"
[light clients]: https://a16zcrypto.com/posts/article/an-introduction-to-light-clients/
[nix]: https://zero-to-nix.com/
[nixos]: https://nixos.org
[orbstack]: https://orbstack.dev/
[rust]: https://www.rust-lang.org/
[solidity]: https://soliditylang.org/
[svelte]: https://svelte.dev
[twitter badge]: https://twitter.com/intent/follow?screen_name=union_build
[twitter handle]: https://img.shields.io/twitter/follow/union_build.svg?style=social&label=Follow
[typescript]: https://www.typescriptlang.org/
[union.build]: https://union.build
[`cometbls`]: https://github.com/unionlabs/cometbls
