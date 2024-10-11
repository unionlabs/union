---
order: 1
parent:
  title: Fork Detection
  order: 2
---

# Cosmos fork detection and IBC fork detection

## Status

This is a work in progress.
This directory captures the ongoing work and discussion on fork
detection both in the context of a Cosmos light node and in the
context of IBC. It contains the following files

### [detection.md](./detection_003_reviewed.md)

a draft of the light node fork detection including "proof of fork"
  definition, that is, the data structure to submit evidence to full
  nodes.
  
### [discussions.md](./discussions.md)

A collection of ideas and intuitions from recent discussions

- the outcome of recent discussion
- a sketch of the light client supervisor to provide the context in
  which fork detection happens
- a discussion about lightstore semantics

### [req-ibc-detection.md](./req-ibc-detection.md)

- a collection of requirements for fork detection in the IBC
  context. In particular it contains a section "Required Changes in
  ICS 007" with necessary updates to ICS 007 to support Cosmos
  fork detection

### [draft-functions.md](./draft-functions.md)

In order to address the collected requirements, we started to sketch
some functions that we will need in the future when we specify in more
detail the

- fork detections
- proof of fork generation
- proof of fork verification

on the following components.

- IBC on-chain components
- Relayer

### TODOs

We decided to merge the files while there are still open points to
address to record the current state an move forward. In particular,
the following points need to be addressed:

- <https://github.com/informalsystems/tendermint-rs/pull/479#discussion_r466504876>

- <https://github.com/informalsystems/tendermint-rs/pull/479#discussion_r466493900>
  
- <https://github.com/informalsystems/tendermint-rs/pull/479#discussion_r466489045>
  
- <https://github.com/informalsystems/tendermint-rs/pull/479#discussion_r466491471>
  
Most likely we will write a specification on the light client
supervisor along the outcomes of
  
- <https://github.com/informalsystems/tendermint-rs/pull/509>

that also addresses initialization

- <https://github.com/tendermint/spec/issues/131>
