# CometBFT

[Byzantine-Fault Tolerant][bft] [State Machine Replication][smr]. Or
[Blockchain], for short.

[![Version][version-badge]][version-url]
[![API Reference][api-badge]][api-url]
[![Go version][go-badge]][go-url]
[![Discord chat][discord-badge]][discord-url]
[![License][license-badge]][license-url]
[![Sourcegraph][sg-badge]][sg-url]

| Branch  | Tests                                          | Linting                                     |
|---------|------------------------------------------------|---------------------------------------------|
| main    | [![Tests][tests-badge]][tests-url]             | [![Lint][lint-badge]][lint-url]             |
| v1.x    | [![Tests][tests-badge-v1x]][tests-url-v1x]     | [![Lint][lint-badge-v1x]][lint-url-v1x]     |
| v0.38.x | [![Tests][tests-badge-v038x]][tests-url-v038x] | [![Lint][lint-badge-v038x]][lint-url-v038x] |
| v0.37.x | [![Tests][tests-badge-v037x]][tests-url-v037x] | [![Lint][lint-badge-v037x]][lint-url-v037x] |
| v0.34.x | [![Tests][tests-badge-v034x]][tests-url-v034x] | [![Lint][lint-badge-v034x]][lint-url-v034x] |

CometBFT is a Byzantine Fault Tolerant (BFT) middleware that takes a
state transition machine - written in any programming language - and securely
replicates it on many machines.

It is a fork of [Tendermint Core][tm-core] and implements the Tendermint
consensus algorithm.

For protocol details, refer to the [CometBFT Specification](./spec/README.md).

For detailed analysis of the consensus protocol, including safety and liveness
proofs, read our paper, "[The latest gossip on BFT
consensus](https://arxiv.org/abs/1807.04938)".

## Documentation

Complete documentation can be found on the
[website](https://docs.cometbft.com/).

## Releases

Please do not depend on `main` as your production branch, as it may receive
significant breaking changes at any time. Use
[releases](https://github.com/cometbft/cometbft/releases) instead.

If you intend to run CometBFT in production, we're happy to help. To contact us,
in order of preference:

- [Create a new discussion on
  GitHub](https://github.com/cometbft/cometbft/discussions)
- Reach out to us via [Telegram](https://t.me/CometBFT)
- [Join the Cosmos Network Discord](https://discord.gg/interchain) and
  discuss in
  [`#cometbft`](https://discord.com/channels/669268347736686612/1069933855307472906)

More on how releases are conducted can be found [here](./RELEASES.md).

## Security

Please see [SECURITY.md](./SECURITY.md).

## Minimum requirements

| CometBFT version | Requirement | Version        | Tested with  |
|------------------|-------------|----------------|--------------|
| main             | Go version  | 1.22 or higher | up to 1.22.5 |
| v1.x             | Go version  | 1.22 or higher | up to 1.22   |
| v0.38.x          | Go version  | 1.21 or higher | up to 1.22   |
| v0.37.x          | Go version  | 1.20 or higher | up to 1.22   |
| v0.34.x          | Go version  | 1.20 or higher | up to 1.20   |

### Install

See the [install guide](docs/tutorials/install.md).

### Quick Start

- [Single node](docs/tutorials/quick-start.md)
- [Local cluster using docker-compose](./docs/guides/networks/docker-compose.md)

## Contributing

Please abide by the [Code of Conduct](CODE_OF_CONDUCT.md) in all interactions.

Before contributing to the project, please take a look at the [contributing
guidelines](CONTRIBUTING.md) and the [style guide](STYLE_GUIDE.md). You may also
find it helpful to read the [specifications](./spec/README.md), and familiarize
yourself with our [Architectural Decision Records
(ADRs)](docs/references/architecture/README.md) and [Request For Comments
(RFCs)](docs/references/rfc/README.md).

## Versioning

As of v1, CometBFT uses the following approach to versioning:

- **Major version** bumps, such as v1.0.0 to v2.0.0, would generally involve
  changes that _force_ users to perform a coordinated upgrade in order to use
  the new version, such as protocol-breaking changes (e.g. changes to how block
  hashes are computed and thus what the network considers to be "valid blocks",
  or how the consensus protocol works, or changes that affect network-level
  compatibility between nodes, etc.).
- **Minor version** bumps, such as v1.1.0 to v1.2.0, are reserved for rolling
  out new features or substantial changes that do not force a coordinated
  upgrade (i.e. not protocol-breaking), but could potentially break Go APIs.
- **Patch version** bumps, such as v1.0.0 to v1.0.1, are reserved for
  bug/security fixes that are not protocol- or Go API-breaking.

### Upgrades

We do not guarantee compatibility between major releases of CometBFT. Minor
releases of the same major release series (v1.1, v1.2, etc.) should, unless
otherwise specified, be compatible with each other. Patch releases of the same
minor release series (v1.0.1, v1.0.2, etc.) are guaranteed to be compatible with
each other.

For more detailed information on upgrading from one version to another, see
[UPGRADING.md](./UPGRADING.md).

### Supported Versions

Because we are a small core team, we have limited capacity to ship patch
updates, including security updates. Consequently, we strongly recommend keeping
CometBFT up-to-date. Upgrading instructions can be found in
[UPGRADING.md](./UPGRADING.md).

Currently supported versions include:

- v1.x: Currently in pre-release with no guarantees as to API stability until a
  release candidate is cut. See [RELEASES.md](./RELEASES.md) for details on our
  process as to API stability guarantees that can be expected of CometBFT
  pre-releases.
- v0.38.x: CometBFT v0.38 introduces ABCI 2.0, which implements the entirety of
  ABCI++
- v0.37.x: CometBFT v0.37 introduces ABCI 1.0, which is the first major step
  towards the full ABCI++ implementation in ABCI 2.0
- v0.34.x: The CometBFT v0.34 series is compatible with the Tendermint Core
  v0.34 series

## Resources

### Libraries

- [Cosmos SDK](http://github.com/cosmos/cosmos-sdk): A framework for building
  high-value public blockchain applications in Go
- [Tendermint in Rust](https://github.com/informalsystems/tendermint-rs)
- [ABCI Tower](https://github.com/penumbra-zone/tower-abci)

### Applications

- [Cosmos Hub](https://hub.cosmos.network/)
- [Celestia](https://celestia.org/)
- [Anoma](https://anoma.network/)
- [Vocdoni](https://developer.vocdoni.io/)

### Research

Below are links to the original Tendermint consensus algorithm and relevant
whitepapers, which CometBFT will continue to build on.

- [The latest gossip on BFT consensus](https://arxiv.org/abs/1807.04938)
- [Master's Thesis on Tendermint](https://atrium.lib.uoguelph.ca/xmlui/handle/10214/9769)
- [Original Whitepaper: "Tendermint: Consensus Without Mining"](https://tendermint.com/static/docs/tendermint.pdf)

## Join us

CometBFT is currently maintained by [Informal
Systems](https://informal.systems). If you'd like to work full-time on CometBFT,
[we're hiring](https://informal.systems/careers)!

Funding for CometBFT development comes primarily from the [Interchain
Foundation](https://interchain.io), a Swiss non-profit. Informal Systems also
maintains [cometbft.com](https://cometbft.com).

[bft]: https://en.wikipedia.org/wiki/Byzantine_fault_tolerance
[smr]: https://en.wikipedia.org/wiki/State_machine_replication
[Blockchain]: https://en.wikipedia.org/wiki/Blockchain
[version-badge]: https://img.shields.io/github/v/release/cometbft/cometbft.svg
[version-url]: https://github.com/cometbft/cometbft/releases/latest
[api-badge]: https://camo.githubusercontent.com/915b7be44ada53c290eb157634330494ebe3e30a/68747470733a2f2f676f646f632e6f72672f6769746875622e636f6d2f676f6c616e672f6764646f3f7374617475732e737667
[api-url]: https://pkg.go.dev/github.com/cometbft/cometbft
[go-badge]: https://img.shields.io/badge/go-1.21-blue.svg
[go-url]: https://github.com/moovweb/gvm
[discord-badge]: https://img.shields.io/discord/669268347736686612.svg
[discord-url]: https://discord.gg/interchain
[license-badge]: https://img.shields.io/github/license/cometbft/cometbft.svg
[license-url]: https://github.com/cometbft/cometbft/blob/main/LICENSE
[sg-badge]: https://sourcegraph.com/github.com/cometbft/cometbft/-/badge.svg
[sg-url]: https://sourcegraph.com/github.com/cometbft/cometbft?badge
[tests-url]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml
[tests-url-v1x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml?query=branch%3Av1.x
[tests-url-v038x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml?query=branch%3Av0.38.x
[tests-url-v037x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml?query=branch%3Av0.37.x
[tests-url-v034x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml?query=branch%3Av0.34.x
[tests-badge]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml/badge.svg?branch=main
[tests-badge-v1x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml/badge.svg?branch=v1.x
[tests-badge-v038x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml/badge.svg?branch=v0.38.x
[tests-badge-v037x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml/badge.svg?branch=v0.37.x
[tests-badge-v034x]: https://github.com/cometbft/cometbft/actions/workflows/tests.yml/badge.svg?branch=v0.34.x
[lint-badge]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml/badge.svg?branch=main
[lint-badge-v034x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml/badge.svg?branch=v0.34.x
[lint-badge-v037x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml/badge.svg?branch=v0.37.x
[lint-badge-v038x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml/badge.svg?branch=v0.38.x
[lint-badge-v1x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml/badge.svg?branch=v1.x
[lint-url]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml
[lint-url-v034x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml?query=branch%3Av0.34.x
[lint-url-v037x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml?query=branch%3Av0.37.x
[lint-url-v038x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml?query=branch%3Av0.38.x
[lint-url-v1x]: https://github.com/cometbft/cometbft/actions/workflows/lint.yml?query=branch%3Av1.x
[tm-core]: https://github.com/tendermint/tendermint
