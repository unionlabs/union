---
sidebar_position: 1
---

# Requests for Comments

A Request for Comments (RFC) is a record of discussion on an open-ended topic
related to the design and implementation of the Cosmos SDK, for which no
immediate decision is required.

The purpose of an RFC is to serve as a historical record of a high-level
discussion that might otherwise only be recorded in an ad-hoc way (for example,
via gists or Google docs) that are difficult to discover for someone after the
fact. An RFC _may_ give rise to more specific architectural _decisions_ for
the Cosmos SDK, but those decisions must be recorded separately in
[Architecture Decision Records (ADR)](../architecture).

As a rule of thumb, if you can articulate a specific question that needs to be
answered, write an ADR. If you need to explore the topic and get input from
others to know what questions need to be answered, an RFC may be appropriate.

## RFC Content

An RFC should provide:

* A **changelog**, documenting when and how the RFC has changed.
* An **abstract**, briefly summarizing the topic so the reader can quickly tell
  whether it is relevant to their interest.
* Any **background** a reader will need to understand and participate in the
  substance of the discussion (links to other documents are fine here).
* The **discussion**, the primary content of the document.

The [rfc-template.md](./rfc-template.md) file includes placeholders for these
sections.

## Table of Contents


* [RFC-001: Tx Validation](./rfc-001-tx-validation.md)
* [RFC-002: Zero Copy Encoding](./rfc-002-zero-copy-encoding.md)
* [RFC-004: Accounts](./rfc-004-accounts.md)
* [RFC-005: Optimistic Execution](./rfc-005-optimistic-execution.md)
