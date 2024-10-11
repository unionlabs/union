---
order: 1
parent:
  order: false
---

# Requests for Comments

A Request for Comments (RFC) is a record of discussion on an open-ended topic
related to the design and implementation of CometBFT, for which no
immediate decision is required.

The purpose of an RFC is to serve as a historical record of a high-level
discussion that might otherwise only be recorded in an ad-hoc way (for example,
via gists or Google docs) that are difficult to discover for someone after the
fact. An RFC _may_ give rise to more specific architectural _decisions_ for
CometBFT, but those decisions must be recorded separately in
[Architecture Decision Records (ADR)](./../architecture/README.md).

As a rule of thumb, if you can articulate a specific question that needs to be
answered, write an ADR. If you need to explore the topic and get input from
others to know what questions need to be answered, an RFC may be appropriate.

## RFC Content

An RFC should provide:

- A **changelog**, documenting when and how the RFC has changed.
- An **abstract**, briefly summarizing the topic so the reader can quickly tell
  whether it is relevant to their interest.
- Any **background** a reader will need to understand and participate in the
  substance of the discussion (links to other documents are fine here).
- The **discussion**, the primary content of the document.

The [rfc-template.md](rfc-template.md) file includes placeholders for these
sections.

## Table of Contents

The RFCs listed below are exclusively relevant to CometBFT. For historical RFCs
relating to Tendermint Core prior to forking, please see
[this list](./tendermint-core/).

<!-- - [RFC-NNN: Title](./rfc-NNN-title.md) -->
- [RFC-100: ABCI Vote Extension Propagation](rfc-100-abci-vote-extension-propag.md)
- [RFC-101: Banning peers based on ResponseCheckTx](rfc-101-p2p-bad-peers-checktx.md)
- [RFC-102: Improve forward compatibility of proto-generated Rust code](rfc-102-rust-gen-builders.md)
- [RFC-103: Incoming transactions when node is catching up](rfc-103-incoming-txs-when-catching-up.md)
- [RFC-104: Internal messaging using the actor model](rfc-104-actor-model.md)
- [RFC-105: Allowing Non-Determinism in `ProcessProposal`](rfc-105-non-det-process-proposal.md)
- [RFC-106: Separation of non-idempotent methods in data companion API](rfc-106-separate-stateful-methods.md)
- [RFC-107: Internal signalling using event observers](rfc-107-event-observer.md)
