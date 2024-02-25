---
title: BLS Signatures
---

Boneh–Lynn–Shacham (BLS) signatures form the foundation of [CometBLS](/docs/architecture/cometbls). They are cheaper to verify for both regular [IBC](/docs/concepts/ibc) and zero-knowledge-proof (zkp) based IBC. With BLS signatures, we can aggregate the public keys and the signatures, and verify the aggregated signature with the aggregated private key. This has a few advantages:

- Transaction size decreases compared to ECDSA verification. We do not need to transfer all signatures, just the aggregate.

- On-chain computation cost decreases. Instead of verifying each signature, we verify the aggregate.

- Zkp verification is much more efficient.

- Allows for [distributed validator tech](/docs/concepts/distributed-validator-tech)

Note that the Union validators do not produce zkps directly. This function is performed by [galois](/docs/architecture/cometbls). Relayers can produce proofs themselves, or use Union as a distributed sequencing layer through the use of [proof claims](https://github.com/unionlabs/union/discussions/41).

