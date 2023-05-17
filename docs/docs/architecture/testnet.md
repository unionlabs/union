---
title: "Testnet Overview"
---

# Testnet

The union.fi testnet acts as a production-like staging environment, tracking the release candidates of [uniond](https://github.com/unionfi/union).

## Genesis

The network was bootstrapped with 6 validators from the founding team. The genesis config may be found [here](https://github.com/UnionFi/genesis).

## Architecture

### Topology

The private testnet has 6 validators. It is not configured to be proof of authority (POA), but since tokens are not available until the public testnet, it is effectively a POA chain. The genesis validators are used as boot and RPC nodes.

<img src="/img/testnet-overview.drawio.svg"  width="100%" height="30%"/>

### Prover

The prover connects over the RPC interface to construct CometBLS-Groth16 proofs. It submits these to the [Goerli Testnet](https://goerli.net/).

In production configurations, a relayer should connect to its validator to avoid data withholding attacks and for increased reliability.
