---
title: "CometBLS"
---

# Introduction

Proving a blockchain's state and finality lies at the root of secure bridging and consensus verification. [Tendermint](https://github.com/cometbft/cometbft), although designed for fast, single-slot finality, is not suited for bridging to block space-restricted chains, such as Ethereum. It requires too much computation to verify, and even creating zero-knowledge proofs of the verification is expensive, slow, and scales poorly with the number of validators. Most attempts to bridge to Ethereum before Union are centralized and not based on consensus verification.

## CometBLS V1

_CometBLS_ is an improvement upon Tendermint which makes it suitable for zero-knowledge proving. Currently, this is achieved by two major changes to [CometBFT](https://github.com/cometbft/cometbft), with further improvement pending. These improvements will decrease proving times even further, leading to faster bridging transfers and cheaper relaying.

### BLS Signatures

Boneh–Lynn–Shacham (BLS) signatures form the foundation of CometBLS. They are cheaper to verify for both regular IBC and zero-knowledge-proof (zkp) based IBC. With BLS signatures, we can aggregate the public keys and the signatures, and verify the aggregated signature with the aggregated private key. This has a few advantages:

- Transaction size decreases compared to ECDSA verification. We do not need to transfer all signatures, just the aggregate.

- On-chain computation cost decreases. Instead of verifying each signature, we verify the aggregate.

- Zkps verification is much more efficient.

Note that the Union network does not produce zkps. This function is performed by [galois](./galois.md). Relayers can produce proofs themselves, or use Union as a distributed sequencing layer through the use of [proof claims](https://github.com/unionfi/union/discussions/41).

Under CometBLS, the Union network can scale to over a hundred validators without impacting performance or bridging latency.

#### Distributed Validators

We can scale the validator set using [distributed validator tech](https://figment.io/distributed-validator-technology-and-infrastructure-resilience/) (DVT) even more, allowing the Union network to effectively support thousands of validators. The foundation for this scaling is once again BLS signatures, which allows us to aggregate signed votes in smaller steps:

## TODO: create a diagram of a tree of validators

## Pending Improvements

### MiMC Hashing

### Verkle Trees

### Epoch-based Validator Rotation
