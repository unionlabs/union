---
order: 1
parent:
  order: false
---

# Tendermint Core Architecture Decision Records (ADR)

Here we record all high-level architecture decisions in the Tendermint Core
project. All implemented ADRs in this list naturally affect CometBFT, since
CometBFT is a fork of Tendermint Core as of December 2022.

This list is currently frozen and kept for reference purposes. To add new ADRs,
please do so for CometBFT [here](../).

## Table of Contents

### Implemented

- [ADR-001: Logging](adr-001-logging.md)
- [ADR-002: Event-Subscription](adr-002-event-subscription.md)
- [ADR-003: ABCI-APP-RPC](adr-003-abci-app-rpc.md)
- [ADR-004: Historical-Validators](adr-004-historical-validators.md)
- [ADR-005: Consensus-Params](adr-005-consensus-params.md)
- [ADR-008: Priv-Validator](adr-008-priv-validator.md)
- [ADR-009: ABCI-Design](adr-009-ABCI-design.md)
- [ADR-010: Crypto-Changes](adr-010-crypto-changes.md)
- [ADR-011: Monitoring](adr-011-monitoring.md)
- [ADR-014: Secp-Malleability](adr-014-secp-malleability.md)
- [ADR-015: Crypto-Encoding](adr-015-crypto-encoding.md)
- [ADR-016: Protocol-Versions](adr-016-protocol-versions.md)
- [ADR-017: Chain-Versions](adr-017-chain-versions.md)
- [ADR-018: ABCI-Validators](adr-018-ABCI-Validators.md)
- [ADR-019: Multisigs](adr-019-multisigs.md)
- [ADR-020: Block-Size](adr-020-block-size.md)
- [ADR-021: ABCI-Events](adr-021-abci-events.md)
- [ADR-025: Commit](adr-025-commit.md)
- [ADR-026: General-Merkle-Proof](adr-026-general-merkle-proof.md)
- [ADR-033: Pubsub](adr-033-pubsub.md)
- [ADR-034: Priv-Validator-File-Structure](adr-034-priv-validator-file-structure.md)
- [ADR-043: Blockchain-RiRi-Org](adr-043-blockchain-riri-org.md)
- [ADR-044: Lite-Client-With-Weak-Subjectivity](adr-044-lite-client-with-weak-subjectivity.md)
- [ADR-046: Light-Client-Implementation](adr-046-light-client-implementation.md)
- [ADR-047: Handling-Evidence-From-Light-Client](adr-047-handling-evidence-from-light-client.md)
- [ADR-051: Double-Signing-Risk-Reduction](adr-051-double-signing-risk-reduction.md)
- [ADR-052: Tendermint-Mode](adr-052-tendermint-mode.md)
- [ADR-053: State-Sync-Prototype](adr-053-state-sync-prototype.md)
- [ADR-054: Crypto-Encoding-2](adr-054-crypto-encoding-2.md)
- [ADR-055: Protobuf-Design](adr-055-protobuf-design.md)
- [ADR-056: Light-Client-Amnesia-Attacks](adr-056-light-client-amnesia-attacks.md)
- [ADR-059: Evidence-Composition-and-Lifecycle](adr-059-evidence-composition-and-lifecycle.md)
- [ADR-065: Custom Event Indexing](adr-065-custom-event-indexing.md)
- [ADR-066: E2E-Testing](adr-066-e2e-testing.md)
- [ADR-072: Restore Requests for Comments](adr-072-request-for-comments.md)
- [ADR-076: Combine Spec and Tendermint Repositories](adr-076-combine-spec-repo.md)
- [ADR-077: Configurable Block Retention](adr-077-block-retention.md)
- [ADR-078: Non-zero Genesis](adr-078-nonzero-genesis.md)

### Accepted

- [ADR-006: Trust-Metric](adr-006-trust-metric.md)
- [ADR-024: Sign-Bytes](adr-024-sign-bytes.md)
- [ADR-039: Peer-Behaviour](adr-039-peer-behaviour.md)
- [ADR-063: Privval-gRPC](adr-063-privval-grpc.md)
- [ADR-067: Mempool Refactor](adr-067-mempool-refactor.md)
- [ADR-071: Proposer-Based Timestamps](adr-071-proposer-based-timestamps.md)
- [ADR-075: RPC Event Subscription Interface](adr-075-rpc-subscription.md)
- [ADR-079: Ed25519 Verification](adr-079-ed25519-verification.md)
- [ADR-081: Protocol Buffers Management](adr-081-protobuf-mgmt.md)

### Deprecated

- [ADR-035: Documentation](adr-035-documentation.md)

### Rejected

- [ADR-023: ABCI-Propose-tx](adr-023-ABCI-propose-tx.md)
- [ADR-029: Check-Tx-Consensus](adr-029-check-tx-consensus.md)
- [ADR-058: Event-Hashing](adr-058-event-hashing.md)

### Proposed

- [ADR-007: Trust-Metric-Usage](adr-007-trust-metric-usage.md)
- [ADR-012: Peer-Transport](adr-012-peer-transport.md)
- [ADR-013: Symmetric-Crypto](adr-013-symmetric-crypto.md)
- [ADR-022: ABCI-Errors](adr-022-abci-errors.md)
- [ADR-030: Consensus-Refactor](adr-030-consensus-refactor.md)
- [ADR-036: Empty Blocks via ABCI](adr-036-empty-blocks-abci.md)
- [ADR-037: Deliver-Block](adr-037-deliver-block.md)
- [ADR-038: Non-Zero-Start-Height](adr-038-non-zero-start-height.md)
- [ADR-040: Blockchain Reactor Refactor](adr-040-blockchain-reactor-refactor.md)
- [ADR-041: Proposer-Selection-via-ABCI](adr-041-proposer-selection-via-abci.md)
- [ADR-042: State Sync Design](adr-042-state-sync.md)
- [ADR-045: ABCI-Evidence](adr-045-abci-evidence.md)
- [ADR-050: Improved Trusted Peering](adr-050-improved-trusted-peering.md)
- [ADR-057: RPC](adr-057-RPC.md)
- [ADR-060: Go-API-Stability](adr-060-go-api-stability.md)
- [ADR-061: P2P-Refactor-Scope](adr-061-p2p-refactor-scope.md)
- [ADR-062: P2P-Architecture](adr-062-p2p-architecture.md)
- [ADR-064: Batch Verification](adr-064-batch-verification.md)
- [ADR-068: Reverse-Sync](adr-068-reverse-sync.md)
- [ADR-069: Node Initialization](adr-069-flexible-node-initialization.md)
- [ADR-073: Adopt LibP2P](adr-073-libp2p.md)
- [ADR-074: Migrate Timeout Parameters to Consensus Parameters](adr-074-timeout-params.md)
- [ADR-080: Reverse Sync](adr-080-reverse-sync.md)
