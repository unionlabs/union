---
title: CometBFT Documentation
description: CometBFT is a blockchain application platform.
footer:
  newsletter: false
---

# CometBFT

Welcome to the CometBFT documentation!

CometBFT is a blockchain application platform; it provides the equivalent
of a web-server, database, and supporting libraries for blockchain applications
written in any programming language. Like a web-server serving web applications,
CometBFT serves blockchain applications.

More formally, CometBFT performs Byzantine Fault Tolerant (BFT)
State Machine Replication (SMR) for arbitrary deterministic, finite state machines.
For more background, see [What is CometBFT?](./explanation/introduction/README.md).

To get started quickly with an example application, see the [quick start guide](tutorials/quick-start.md).

To learn about application development on CometBFT, see the [Application Blockchain Interface](https://github.com/cometbft/cometbft/tree/main/spec/abci).

For more details on using CometBFT, see the respective documentation for
[CometBFT internals](explanation/core/), [benchmarking and monitoring](guides/tools/), and [network deployments](guides/networks/).

## Contribute

To recommend a change to the documentation, please submit a PR. Each major
release's documentation is housed on the corresponding release branch, e.g. for
the v0.34 release series, the documentation is housed on the `v0.34.x` branch.

When submitting changes that affect all releases, please start by submitting a
PR to the docs on `main` - this will be backported to the relevant release
branches. If a change is exclusively relevant to a specific release, please
target that release branch with your PR.

Changes to the documentation will be reviewed by the team and, if accepted and
merged, published to <https://docs.cometbft.com> for the respective version(s).

The build process for the documentation is housed in the [CometBFT documentation
repository](https://github.com/cometbft/cometbft-docs).
