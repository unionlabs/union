# ADR 012: ABCI Events

## Changelog

- *2018-09-02* Remove ABCI errors component. Update description for events
- *2018-07-12* Initial version

## Context

ABCI tags were first described in [ADR 002](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-002-event-subscription.md).
They are key-value pairs that can be used to index transactions.

Currently, ABCI messages return a list of tags to describe an
"event" that took place during the Check/DeliverTx/Begin/EndBlock,
where each tag refers to a different property of the event, like the sending and receiving account addresses.

Since there is only one list of tags, recording data for multiple such events in
a single Check/DeliverTx/Begin/EndBlock must be done using prefixes in the key
space.

Alternatively, groups of tags that constitute an event can be separated by a
special tag that denotes a break between the events. This would allow
straightforward encoding of multiple events into a single list of tags without
prefixing, at the cost of these "special" tags to separate the different events.

TODO: brief description of how the indexing works

## Decision

Instead of returning a list of tags, return a list of events, where
each event is a list of tags. This way we naturally capture the concept of
multiple events happening during a single ABCI message.

TODO: describe impact on indexing and querying

## Status

Implemented

## Consequences

### Positive

- Ability to track distinct events separate from ABCI calls (DeliverTx/BeginBlock/EndBlock)
- More powerful query abilities

### Negative

- More complex query syntax
- More complex search implementation

### Neutral
