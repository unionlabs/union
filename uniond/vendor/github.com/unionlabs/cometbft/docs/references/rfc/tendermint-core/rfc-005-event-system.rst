=====================
RFC 005: Event System
=====================

Changelog
---------

- 2021-09-17: Initial Draft (@tychoish)

Abstract
--------

The event system within Tendermint, which supports a lot of core
functionality, also represents a major infrastructural liability. As part of
our upcoming review of the RPC interfaces and our ongoing thoughts about
stability and performance, as well as the preparation for Tendermint 1.0, we
should revisit the design and implementation of the event system. This
document discusses both the current state of the system and potential
directions for future improvement.

Background
----------

Current State of Events
~~~~~~~~~~~~~~~~~~~~~~~

The event system makes it possible for clients, both internal and external,
to receive notifications of state replication events, such as new blocks,
new transactions, validator set changes, as well as intermediate events during
consensus. Because the event system is very cross cutting, the behavior and
performance of the event publication and subscription system has huge impacts
for all of Tendermint.

The subscription service is exposed over the RPC interface, but also powers
the indexing (e.g. to an external database,) and is the mechanism by which
`BroadcastTxCommit` is able to wait for transactions to land in a block.

The current pubsub mechanism relies on a couple of buffered channels,
primarily between all event creators and subscribers, but also for each
subscription. The result of this design is that, in some situations with the
right collection of slow subscription consumers the event system can put
backpressure on the consensus state machine and message gossiping in the
network, thereby causing nodes to lag.

Improvements
~~~~~~~~~~~~

The current system relies on implicit, bounded queues built by the buffered channels,
and though threadsafe, can force all activity within Tendermint to serialize,
which does not need to happen. Additionally, timeouts for subscription
consumers related to the implementation of the RPC layer, may complicate the
use of the system.

References
~~~~~~~~~~

- Legacy Implementation
  - `publication of events <https://github.com/tendermint/tendermint/blob/main/libs/pubsub/pubsub.go#L333-L345>`_ 
  - `send operation <https://github.com/tendermint/tendermint/blob/main/libs/pubsub/pubsub.go#L489-L527>`_ 
  - `send loop <https://github.com/tendermint/tendermint/blob/main/libs/pubsub/pubsub.go#L381-L402>`_
- Related RFCs 
  - `RFC 002: IPC Ecosystem <./rfc-002-ipc-ecosystem.md>`_ 
  - `RFC 003: Performance Questions <./rfc-003-performance-questions.md>`_ 

Discussion
----------

Changes to Published Events
~~~~~~~~~~~~~~~~~~~~~~~~~~~

As part of this process, the Tendermint team should do a study of the existing
event types and ensure that there are viable production use cases for
subscriptions to all event types. Instinctively it seems plausible that some
of the events may not be usable outside of tendermint, (e.g. ``TimeoutWait``
or ``NewRoundStep``) and it might make sense to remove them. Certainly, it
would be good to make sure that we don't maintain infrastructure for unused or
un-useful message indefinitely.

Blocking Subscription
~~~~~~~~~~~~~~~~~~~~~

The blocking subscription mechanism makes it possible to have *send*
operations into the subscription channel be un-buffered (the event processing
channel is still buffered.) In the blocking case, events from one subscription
can block processing that event for other non-blocking subscriptions. The main
case, it seems for blocking subscriptions is ensuring that a transaction has
been committed to a block for ``BroadcastTxCommit``. Removing blocking
subscriptions entirely, and potentially finding another way to implement
``BroadcastTxCommit``, could lead to important simplifications and
improvements to throughput without requiring large changes.

Subscription Identification
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Before `#6386 <https://github.com/tendermint/tendermint/pull/6386>`_, all
subscriptions were identified by the combination of a client ID and a query,
and with that change, it became possible to identify all subscription given
only an ID, but compatibility with the legacy identification means that there's a
good deal of legacy code as well as client side efficiency that could be
improved. 

Pubsub Changes
~~~~~~~~~~~~~~

The pubsub core should be implemented in a way that removes the possibility of
backpressure from the event system to impact the core system *or* for one
subscription to impact the behavior of another area of the
system. Additionally, because the current system is implemented entirely in
terms of a collection of buffered channels, the event system (and large
numbers of subscriptions) can be a source of memory pressure. 

These changes could include: 

- explicit cancellation and timeouts promulgated from callers (e.g. RPC end
  points, etc,) this should be done using contexts.

- subscription system should be able to spill to disk to avoid putting memory
  pressure on the core behavior of the node (consensus, gossip).
  
- subscriptions implemented as cursors rather than channels, with either
  condition variables to simulate the existing "push" API or a client side
  iterator API with some kind of long polling-type interface. 
