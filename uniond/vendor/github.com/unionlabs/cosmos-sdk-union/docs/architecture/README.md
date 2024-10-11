---
sidebar_position: 1
---

# Architecture Decision Records (ADR)

This is a location to record all high-level architecture decisions in the Cosmos-SDK.

An Architectural Decision (**AD**) is a software design choice that addresses a functional or non-functional requirement that is architecturally significant.
An Architecturally Significant Requirement (**ASR**) is a requirement that has a measurable effect on a software system’s architecture and quality.
An Architectural Decision Record (**ADR**) captures a single AD, such as often done when writing personal notes or meeting minutes; the collection of ADRs created and maintained in a project constitute its decision log. All these are within the topic of Architectural Knowledge Management (AKM).

You can read more about the ADR concept in this [blog post](https://product.reverb.com/documenting-architecture-decisions-the-reverb-way-a3563bb24bd0#.78xhdix6t).

## Rationale

ADRs are intended to be the primary mechanism for proposing new feature designs and new processes, for collecting community input on an issue, and for documenting the design decisions.
An ADR should provide:

* Context on the relevant goals and the current state
* Proposed changes to achieve the goals
* Summary of pros and cons
* References
* Changelog

Note the distinction between an ADR and a spec. The ADR provides the context, intuition, reasoning, and
justification for a change in architecture, or for the architecture of something
new. The spec is much more compressed and streamlined summary of everything as
it stands today.

If recorded decisions turned out to be lacking, convene a discussion, record the new decisions here, and then modify the code to match.

## Creating new ADR

Read about the [PROCESS](./PROCESS.md).

### Use RFC 2119 Keywords

When writing ADRs, follow the same best practices for writing RFCs. When writing RFCs, key words are used to signify the requirements in the specification. These words are often capitalized: "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL. They are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

## ADR Table of Contents

### Accepted

* [ADR 002: SDK Documentation Structure](./adr-002-docs-structure.md)
* [ADR 004: Split Denomination Keys](./adr-004-split-denomination-keys.md)
* [ADR 006: Secret Store Replacement](./adr-006-secret-store-replacement.md)
* [ADR 009: Evidence Module](./adr-009-evidence-module.md)
* [ADR 010: Modular AnteHandler](./adr-010-modular-antehandler.md)
* [ADR 019: Protocol Buffer State Encoding](./adr-019-protobuf-state-encoding.md)
* [ADR 020: Protocol Buffer Transaction Encoding](./adr-020-protobuf-transaction-encoding.md)
* [ADR 021: Protocol Buffer Query Encoding](./adr-021-protobuf-query-encoding.md)
* [ADR 023: Protocol Buffer Naming and Versioning](./adr-023-protobuf-naming.md)
* [ADR 024: Coin Metadata](./adr-024-coin-metadata.md)
* [ADR 029: Fee Grant Module](./adr-029-fee-grant-module.md)
* [ADR 030: Message Authorization Module](./adr-030-authz-module.md)
* [ADR 031: Protobuf Msg Services](./adr-031-msg-service.md)
* [ADR 046: Module Params](./adr-046-module-params.md)
* [ADR 055: ORM](./adr-055-orm.md)
* [ADR 058: Auto-Generated CLI](./adr-058-auto-generated-cli.md)
* [ADR 060: ABCI 1.0 (Phase I)](adr-060-abci-1.0.md)
* [ADR 061: Liquid Staking](./adr-061-liquid-staking.md)
* [ADR 070: Un-Ordered Transaction Inclusion](./adr-070-unordered-transactions.md)
* [ADR 065: Store v2](./adr-065-store-v2.md)
* [ADR 073: Built-in In-process Indexer](./adr-073-indexer.md)

### Proposed

* [ADR 003: Dynamic Capability Store](./adr-003-dynamic-capability-store.md)
* [ADR 011: Generalize Genesis Accounts](./adr-011-generalize-genesis-accounts.md)
* [ADR 012: State Accessors](./adr-012-state-accessors.md)
* [ADR 013: Metrics](./adr-013-metrics.md)
* [ADR 016: Validator Consensus Key Rotation](./adr-016-validator-consensus-key-rotation.md)
* [ADR 017: Historical Header Module](./adr-017-historical-header-module.md)
* [ADR 018: Extendable Voting Periods](./adr-018-extendable-voting-period.md)
* [ADR 022: Custom baseapp panic handling](./adr-022-custom-panic-handling.md)
* [ADR 027: Deterministic Protobuf Serialization](./adr-027-deterministic-protobuf-serialization.md)
* [ADR 028: Public Key Addresses](./adr-028-public-key-addresses.md)
* [ADR 032: Typed Events](./adr-032-typed-events.md)
* [ADR 033: Inter-module RPC](./adr-033-protobuf-inter-module-comm.md)
* [ADR 035: Rosetta API Support](./adr-035-rosetta-api-support.md)
* [ADR 037: Governance Split Votes](./adr-037-gov-split-vote.md)
* [ADR 038: State Listening](./adr-038-state-listening.md)
* [ADR 039: Epoched Staking](./adr-039-epoched-staking.md)
* [ADR 040: Storage and SMT State Commitments](./adr-040-storage-and-smt-state-commitments.md)
* [ADR 054: Semver Compatible SDK Modules](./adr-054-semver-compatible-modules.md)
* [ADR 057: App Wiring](./adr-057-app-wiring.md)
* [ADR 059: Test Scopes](./adr-059-test-scopes.md)
* [ADR 062: Collections State Layer](./adr-062-collections-state-layer.md)
* [ADR 063: Core Module API](./adr-063-core-module-api.md)
* [ADR 067: Simulator v2](./adr-067-simulator-v2.md)
* [ADR 069: `x/gov` modularity, multiple choice and optimisic proposals](./adr-069-gov-improvements.md)
* [ADR 074: Messages with implicit signers](./adr-074-implicit-msg-signers.md)

### Draft

* [ADR 044: Guidelines for Updating Protobuf Definitions](./adr-044-protobuf-updates-guidelines.md)
* [ADR 047: Extend Upgrade Plan](./adr-047-extend-upgrade-plan.md)
* [ADR 053: Go Module Refactoring](./adr-053-go-module-refactoring.md)
* [ADR 068: Preblock](./adr-068-preblock.md)
