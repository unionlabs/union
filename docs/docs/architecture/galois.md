---
title: "Galois"
---

# Galois

Galois is the umbrella term of Union's ZK efforts. It currently has one purpose: [consensus verification](../research/consensus-verification). We've built Galois around three fundamental principles:

1. Fast execution: fast proving times equate to fast bridging and good user experience.
2. Low-cost: zero-knowledge provers are computationally expensive to operate, and often require large machine clusters. [Galoisd](https://github.com/unionfi/union/blob/main/uniond/README.md) can run on a single machine and still outperform other zk bridges.
3. Decentralized: infrastructure operators do not need complex cloud architecture to operate a relayer and prover. Anyone can participate in the system.

## Architecture

Transactions through Union to other layers are composed of three steps:

```mermaid
sequenceDiagram
    Union->>Galois: Generate a zkp of Union consensus
    Galois->>Relayer: Forward zkp for to HA relaying service
    Galois->>Union: Submit zkp for proof caching
    Relayer->>Counterparty: Submit zkp for packet processing
```

[Proof caching](https://github.com/unionfi/union/discussions/41) is currently in the pre-RFC stage. It ensures that the network does not perform redundant work and incentivizes decentralized proving, effectively using Union as a decentralized sequencers orchestration layer.

### Technologies

Galois is built using

- [Go](https://go.dev/): simple, secure, scalable.
- [Gnark](https://github.com/ConsenSys/gnark): fast zk-SNARK library.
- [gRPC](https://grpc.io/): high performance, open source universal RPC framework

## Future Roadmap

The current focus for Galois is production-readiness. We're mainly maintaining the current implementation and making performance improvements. After mainnet, the major roadmap items include:

- Verkle tree support (CometBLS v2).
- Formal verification with [Lean](https://leanprover.github.io/).
- Groth16 proof aggregation.
