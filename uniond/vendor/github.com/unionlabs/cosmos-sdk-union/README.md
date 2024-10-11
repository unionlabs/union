<div align="center">
  <h1> Cosmos SDK </h1>
</div>

![banner](https://github.com/cosmos/cosmos-sdk-docs/blob/main/static/img/banner.jpg)

<div align="center">
  <a href="https://github.com/cosmos/cosmos-sdk/blob/main/LICENSE">
    <img alt="License: Apache-2.0" src="https://img.shields.io/github/license/cosmos/cosmos-sdk.svg" />
  </a>
  <a href="https://pkg.go.dev/github.com/cosmos/cosmos-sdk">
    <img src="https://pkg.go.dev/badge/github.com/cosmos/cosmos-sdk.svg" alt="Go Reference">
  </a>
  <a href="https://goreportcard.com/report/github.com/cosmos/cosmos-sdk">
    <img alt="Go report card" src="https://goreportcard.com/badge/github.com/cosmos/cosmos-sdk" />
  </a>
  <a href="https://sonarcloud.io/summary/overall?id=cosmos_cosmos-sdk">
    <img alt="Code Coverage" src="https://sonarcloud.io/api/project_badges/measure?project=cosmos_cosmos-sdk&metric=coverage" />
  </a>
  <a href="https://sonarcloud.io/summary/overall?id=cosmos_cosmos-sdk">
    <img alt="SonarCloud Analysis" src="https://sonarcloud.io/api/project_badges/measure?project=cosmos_cosmos-sdk&metric=alert_status">
  </a>
</div>
<div align="center">
  <a href="https://discord.gg/interchain">
    <img alt="Discord" src="https://img.shields.io/discord/669268347736686612.svg" />
  </a>
  <a href="https://sourcegraph.com/github.com/cosmos/cosmos-sdk?badge">
    <img alt="Imported by" src="https://sourcegraph.com/github.com/cosmos/cosmos-sdk/-/badge.svg" />
  </a>
    <img alt="Sims" src="https://github.com/cosmos/cosmos-sdk/workflows/Sims/badge.svg" />
    <img alt="Lint Status" src="https://github.com/cosmos/cosmos-sdk/workflows/Lint/badge.svg" />
</div>

The Cosmos SDK is a framework for building blockchain applications. [CometBFT (BFT Consensus)](https://github.com/cometbft/cometbft) and the Cosmos SDK are written in the Go programming language. Cosmos SDK is used to build [Gaia](https://github.com/cosmos/gaia), the implementation of the Cosmos Hub.

**WARNING**: The Cosmos SDK has mostly stabilized, but we are still making some breaking changes.

**Note**: Always use the latest maintained [Go](https://go.dev/dl) version for building Cosmos SDK applications.

## Quick Start

To learn how the Cosmos SDK works from a high-level perspective, see the Cosmos SDK [High-Level Intro](https://docs.cosmos.network/v0.50/learn/intro/overview).

If you want to get started quickly and learn how to build on top of Cosmos SDK, visit [Cosmos SDK Tutorials](https://tutorials.cosmos.network). You can also fork the tutorial's repository to get started building your own Cosmos SDK application.

For more information, see the [Cosmos SDK Documentation](https://docs.cosmos.network).

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details on how to contribute and participate in our [dev calls](./CONTRIBUTING.md#teams-dev-calls).
If you want to follow the updates or learn more about the latest design then join our [Discord](https://discord.gg/interchain).

## Tools and Frameworks

The Cosmos ecosystem is vast.
[Awesome Cosmos](https://github.com/cosmos/awesome-cosmos) is a community-curated list of notable frameworks, modules and tools.

### Inter-Blockchain Communication (IBC)

The IBC module for the Cosmos SDK has its own [cosmos/ibc-go repository](https://github.com/cosmos/ibc-go). Go there to build and integrate with the IBC module.

### Version Matrix

The version matrix below shows which versions of the Cosmos SDK, modules and libraries are compatible with each other.

#### Core Dependencies

Core dependencies are the core libraries that an application may depend on.
Core dependencies not mentioned here as compatible across all maintained SDK versions.

| Cosmos SDK | cosmossdk.io/core | cosmossdk.io/api | cosmossdk.io/x/tx |
| ---------- | ----------------- | ---------------- | ----------------- |
| 0.52.z     | 1.y.z             | 0.8.z            | 0.14.z            |
| 0.50.z     | 0.11.z            | 0.7.z            | 0.13.z            |
| 0.47.z     | 0.5.z             | 0.3.z            | N/A               |

#### Module Dependencies

Module Dependencies are the modules that an application may depend on and which version of the Cosmos SDK they are compatible with.

> Note: The version table only goes back to 0.50.x, as modules started to become modular with 0.50.z.
> X signals that the module was not spun out into its own go.mod file.
> N/A signals that the module was not available in the Cosmos SDK at that time.

| Cosmos SDK                  | 0.50.z | 0.52.z |
| --------------------------- | ------ | ------ |
| cosmossdk.io/x/accounts     | N/A    | 0.2.z  |
| cosmossdk.io/x/bank         | X      | 0.2.z  |
| cosmossdk.io/x/circuit      | 0.1.z  | 0.2.z  |
| cosmossdk.io/x/consensus    | X      | 0.2.z  |
| cosmossdk.io/x/distribution | X      | 0.2.z  |
| cosmossdk.io/x/epochs       | N/A    | 0.2.z  |
| cosmossdk.io/x/evidence     | 0.1.z  | 0.2.z  |
| cosmossdk.io/x/feegrant     | 0.1.z  | 0.2.z  |
| cosmossdk.io/x/gov          | X      | 0.2.z  |
| cosmossdk.io/x/group        | X      | 0.2.z  |
| cosmossdk.io/x/mint         | X      | 0.2.z  |
| cosmossdk.io/x/nft          | 0.1.z  | 0.2.z  |
| cosmossdk.io/x/protocolpool | N/A    | 0.2.z  |
| cosmossdk.io/x/slashing     | X      | 0.2.z  |
| cosmossdk.io/x/staking      | X      | 0.2.z  |
| cosmossdk.io/x/upgrade      | 0.1.z  | 0.2.z  |

## Disambiguation

This Cosmos SDK project is not related to the [React-Cosmos](https://github.com/react-cosmos/react-cosmos) project (yet). Many thanks to Evan Coury and Ovidiu (@skidding) for this Github organization name. As per our agreement, this disambiguation notice will stay here.
