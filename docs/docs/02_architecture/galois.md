---
title: "Galois"
---

# Galois

Galois is the umbrella term of Union's ZK efforts. It currently has one purpose: [consensus verification](../concepts/consensus-verification). We've built Galois around three fundamental principles:

1. Fast execution: fast proving times equate to fast bridging and good user experience.
2. Low-cost: zero-knowledge provers are computationally expensive to operate, and often require large machine clusters. [Galoisd](https://github.com/unionlabs/union/blob/main/uniond/README.md) can run on a single machine and still outperform other zk bridges.
3. Decentralized: infrastructure operators do not need complex cloud architecture to operate a relayer and prover. Anyone can participate in the system.

## Architecture

Transactions through Union to other layers are composed of three steps:

1. Emit a send-packet event
2. Generate a ZKP of the Union state
3. Update Counterparty with Union state

```mermaid
sequenceDiagram
    Union->>+Voyager: Emits IBC datagram at block N
    Voyager->>+Galois: Proof Request for block M..N+1
    Galois-->>-Voyager: Sends Generated Proofs for blocks
    Voyager->>-Counterparty: Updates counterparty with Union state
```

:::note

Depending on validator set drift, Galois may need to generate multiple proofs. This results in $M$ being the last trusted height and $N+1$ being the height to update to.

:::

### Technologies

Galois is built using

- [Go](https://go.dev/): simple, secure, scalable.
- [Gnark](https://github.com/ConsenSys/gnark): fast zk-SNARK library.
- [gRPC](https://grpc.io/): high performance, open source universal RPC framework

## Future Roadmap

The current focus for Galois is production-readiness. We're mainly maintaining the current implementation and making performance improvements. After mainnet, the major roadmap items include:

- Proof caching
- Verkle tree support (CometBLS v2).
- Formal verification with [Lean](https://leanprover.github.io/).
- Groth16 proof aggregation.

### Proof Caching

[Proof caching](https://github.com/unionlabs/union/discussions/41) is currently in the pre-RFC stage. It ensures that the network does not perform redundant work and incentivizes decentralized proving, effectively using Union as a decentralized sequencers orchestration layer.
