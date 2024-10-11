---
order: 1
parent:
  title: Consensus
  order: 4
---

# Consensus

Specification of the consensus protocol implemented in CometBFT.

## Contents

- [Consensus Paper](./consensus-paper) - Latex paper on
  [arxiv](https://arxiv.org/abs/1807.04938) describing the
  Tendermint consensus algorithm, adopted in CometBFT, with proofs of safety and termination.
- [Time](./time.md) - How the timestamp in a CometBFT
  block header is produced in a Byzantine Fault Tolerant manner
- [Creating Proposal](./creating-proposal.md) - How a proposer
  creates a block proposal for consensus
- [Light Client Protocol](./light-client) - A protocol for light weight consensus
  verification and syncing to the latest state
- [Signing](./signing.md) - Rules for cryptographic signatures
  produced by validators.
- [Write Ahead Log](./wal.md) - Write ahead log used by the
  consensus state machine to recover from crashes.

There is also a [stale markdown description](consensus.md) of the consensus state machine
(TODO update this).
