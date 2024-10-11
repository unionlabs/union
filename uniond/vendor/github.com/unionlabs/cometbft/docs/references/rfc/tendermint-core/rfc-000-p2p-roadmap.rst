====================
RFC 000: P2P Roadmap
====================

Changelog
---------

- 2021-08-20: Completed initial draft and distributed via a gist
- 2021-08-25: Migrated as an RFC and changed format

Abstract
--------

This document discusses the future of peer network management in Tendermint, with
a particular focus on features, semantics, and a proposed roadmap.
Specifically, we consider libp2p as a tool kit for implementing some fundamentals.

Background
----------

For the 0.35 release cycle the switching/routing layer of Tendermint was
replaced. This work was done "in place," and produced a version of Tendermint
that was backward-compatible and interoperable with previous versions of the
software. While there are new p2p/peer management constructs in the new
version (e.g. ``PeerManager`` and ``Router``), the main effect of this change
was to simplify the ways that other components within Tendermint interacted with
the peer management layer, and to make it possible for higher-level components
(specifically the reactors), to be used and tested more independently.

This refactoring, which was a major undertaking, was entirely necessary to
enable areas for future development and iteration on this aspect of
Tendermint. There are also a number of potential user-facing features that
depend heavily on the p2p layer: additional transport protocols, transport
compression, improved resilience to network partitions. These improvements to
modularity, stability, and reliability of the p2p system will also make
ongoing maintenance and feature development easier in the rest of Tendermint.

Critique of Current Peer-to-Peer Infrastructure
---------------------------------------

The current (refactored) P2P stack is an improvement on the previous iteration
(legacy), but as of 0.35, there remains room for improvement in the design and
implementation of the P2P layer.

Some limitations of the current stack include:

- heavy reliance on buffering to avoid backups in the flow of components,
  which is fragile to maintain and can lead to unexpected memory usage
  patterns and forces the routing layer to make decisions about when messages
  should be discarded.

- the current p2p stack relies on convention (rather than the compiler) to
  enforce the API boundaries and conventions between reactors and the router,
  making it very easy to write "wrong" reactor code or introduce a bad
  dependency.

- the current stack is probably more complex and difficult to maintain because
  the legacy system must coexist with the new components in 0.35. When the
  legacy stack is removed there are some simple changes that will become
  possible and could reduce the complexity of the new system. (e.g. `#6598
  <https://github.com/tendermint/tendermint/issues/6598>`_.)

- the current stack encapsulates a lot of information about peers, and makes it
  difficult to expose that information to monitoring/observability tools. This
  general opacity also makes it difficult to interact with the peer system
  from other areas of the code base (e.g. tests, reactors).

- the legacy stack provided some control to operators to force the system to
  dial new peers or seed nodes or manipulate the topology of the system _in
  situ_. The current stack can't easily provide this, and while the new stack
  may have better behavior, it does leave operators hands tied.

Some of these issues will be resolved early in the 0.36 cycle, with the
removal of the legacy components.

The 0.36 release also provides the opportunity to make changes to the
protocol, as the release will not be compatible with previous releases.

Areas for Development
---------------------

These sections describe features that may make sense to include in a Phase 2 of
a P2P project.

Internal Message Passing
~~~~~~~~~~~~~~~~~~~~~~~~

Currently, there's no provision for intranode communication using the P2P
layer, which means when two reactors need to interact with each other they
have to have dependencies on each other's interfaces, and
initialization. Changing these interactions (e.g. transitions between
blocksync and consensus) from procedure calls to message passing.

This is a relatively simple change and could be implemented with the following
components:

- a constant to represent "local" delivery as  the ``To`` field on
  ``p2p.Envelope``.

- special path for routing local messages that doesn't require message
  serialization (protobuf marshalling/unmarshaling).

Adding these semantics, particularly if in conjunction with synchronous
semantics provides a solution to dependency graph problems currently present
in the Tendermint codebase, which will simplify development, make it possible
to isolate components for testing.

Eventually, this will also make it possible to have a logical Tendermint node
running in multiple processes or in a collection of containers, although the
usecase of this may be debatable.

Synchronous Semantics (Paired Request/Response)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In the current system, all messages are sent with fire-and-forget semantics,
and there's no coupling between a request sent via the p2p layer, and a
response. These kinds of semantics would simplify the implementation of
state and block sync reactors, and make intra-node message passing more
powerful.

For some interactions, like gossiping transactions between the mempools of
different nodes, fire-and-forget semantics make sense, but for other
operations the missing link between requests/responses leads to either
inefficiency when a node fails to respond or becomes unavailable, or code that
is just difficult to follow.

To support this kind of work, the protocol would need to accommodate some kind
of request/response ID to allow identifying out-of-order responses over a
single connection. Additionally, expanded the programming model of the
``p2p.Channel`` to accommodate some kind of _future_ or similar paradigm to
make it viable to write reactor code without needing for the reactor developer
to wrestle with lower level concurrency constructs.


Timeout Handling (QoS)
~~~~~~~~~~~~~~~~~~~~~~

Currently, all timeouts, buffering, and QoS features are handled at the router
layer, and the reactors are implemented in ways that assume/require
asynchronous operation. This both increases the required complexity at the
routing layer, and means that misbehavior at the reactor level is difficult to
detect or attribute. Additionally, the current system provides three main
parameters to control quality of service:

- buffer sizes for channels and queues.

- priorities for channels

- queue implementation details for shedding load.

These end up being quite coarse controls, and changing the settings are
difficult because as the queues and channels are able to buffer large numbers
of messages it can be hard to see the impact of a given change, particularly
in our extant test environment. In general, we should endeavor to:

- set real timeouts, via contexts, on most message send operations, so that
  senders rather than queues can be responsible for timeout
  logic. Additionally, this will make it possible to avoid sending messages
  during shutdown.

- reduce (to the greatest extent possible) the amount of buffering in
  channels and the queues, to more readily surface backpressure and reduce the
  potential for buildup of stale messages.

Stream Based Connection Handling
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Currently the transport layer is message based, which makes sense from a
mental model of how the protocol works, but makes it more difficult to
implement transports and connection types, as it forces a higher level view of
the connection and interaction which makes it harder to implement for novel
transport types and makes it more likely that message-based caching and rate
limiting will be implemented at the transport layer rather than at a more
appropriate level.

The transport then, would be responsible for negotiating the connection and the
handshake and otherwise behave like a socket/file descriptor with ``Read`` and
``Write`` methods.

While this was included in the initial design for the new P2P layer, it may be
obviated entirely if the transport and peer layer is replaced with libp2p,
which is primarily stream based.

Service Discovery
~~~~~~~~~~~~~~~~~

In the current system, Tendermint assumes that all nodes in a network are
largely equivalent, and nodes tend to be "chatty" making many requests of
large numbers of peers and waiting for peers to (hopefully) respond. While
this works and has allowed Tendermint to get to a certain point, this both
produces a theoretical scaling bottle neck and makes it harder to test and
verify components of the system.

In addition to peer's identity and connection information, peers should be
able to advertise a number of services or capabilities, and node operators or
developers should be able to specify peer capability requirements (e.g. target
at least <x>-percent of peers with <y> capability.)

These capabilities may be useful in selecting peers to send messages to, it
may make sense to extend Tendermint's message addressing capability to allow
reactors to send messages to groups of peers based on role rather than only
allowing addressing to one or all peers.

Having a good service discovery mechanism may pair well with the synchronous
semantics (request/response) work, as it allows reactors to "make a request of
a peer with <x> capability and wait for the response," rather force the
reactors to need to track the capabilities or state of specific peers.

Solutions
---------

Continued Homegrown Implementation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The current peer system is homegrown and is conceptually compatible with the
needs of the project, and while there are limitations to the system, the p2p
layer is not (currently as of 0.35) a major source of bugs or friction during
development.

However, the current implementation makes a number of allowances for
interoperability, and there are a collection of iterative improvements that
should be considered in the next couple of releases. To maintain the current
implementation, upcoming work would include:

- change the ``Transport`` mechanism to facilitate easier implementations.

- implement different ``Transport`` handlers to be able to manage peer
  connections using different protocols (e.g. QUIC, etc.)

- entirely remove the constructs and implementations of the legacy peer
  implementation.

- establish and enforce clearer chains of responsibility for connection
  establishment (e.g. handshaking, setup,) which is currently shared between
  three components.

- report better metrics regarding the into the state of peers and network
  connectivity, which are opaque outside of the system. This is constrained at
  the moment as a side effect of the split responsibility for connection
  establishment.

- extend the PEX system to include service information so that nodes in the
  network weren't necessarily homogeneous.

While maintaining a bespoke peer management layer would seem to distract from
development of core functionality, the truth is that (once the legacy code is
removed,) the scope of the peer layer is relatively small from a maintenance
perspective, and having control at this layer might actually afford the
project with the ability to more rapidly iterate on some features.

LibP2P
~~~~~~

LibP2P provides components that, approximately, account for the
``PeerManager`` and ``Transport`` components of the current (new) P2P
stack. The Go APIs seem reasonable, and being able to externalize the
implementation details of peer and connection management seems like it could
provide a lot of benefits, particularly in supporting a more active ecosystem.

In general the API provides the kind of stream-based, multi-protocol
supporting, and idiomatic baseline for implementing a peer layer. Additionally
because it handles peer exchange and connection management at a lower
level, by using libp2p it'd be possible to remove a good deal of code in favor
of just using libp2p. Having said that, Tendermint's P2P layer covers a
greater scope (e.g. message routing to different peers) and that layer is
something that Tendermint might want to retain.

The are a number of unknowns that require more research including how much of
a peer database the Tendermint engine itself needs to maintain, in order to
support higher level operations (consensus, statesync), but it might be the
case that our internal systems need to know much less about peers than
otherwise specified. Similarly, the current system has a notion of peer
scoring that cannot be communicated to libp2p, which may be fine as this is
only used to support peer exchange (PEX,) which would become a property libp2p
and not expressed in it's current higher-level form.

In general, the effort to switch to libp2p would involve:

- timing it during an appropriate protocol-breaking window, as it doesn't seem
  viable to support both libp2p *and* the current p2p protocol.

- providing some in-memory testing network to support the use case that the
  current ``p2p.MemoryNetwork`` provides.

- re-homing the ``p2p.Router`` implementation on top of libp2p components to
  be able to maintain the current reactor implementations.

Open question include:

- how much local buffering should we be doing? It sort of seems like we should
  figure out what the expected behavior is for libp2p for QoS-type
  functionality, and if our requirements mean that we should be implementing
  this on top of things ourselves?

- if Tendermint was going to use libp2p, how would libp2p's stability
  guarantees (protocol, etc.) impact/constrain Tendermint's stability
  guarantees?

- what kind of introspection does libp2p provide, and to what extend would
  this change or constrain the kind of observability that Tendermint is able
  to provide?

- how do efforts to select "the best" (healthy, close, well-behaving, etc.)
  peers work out if Tendermint is not maintaining a local peer database?

- would adding additional higher level semantics (internal message passing,
  request/response pairs, service discovery, etc.) facilitate removing some of
  the direct linkages between constructs/components in the system and reduce
  the need for Tendermint nodes to maintain state about its peers?

References
----------

- `Tracking Ticket for P2P Refactor Project <https://github.com/tendermint/tendermint/issues/5670>`_
- `ADR 61: P2P Refactor Scope <../architecture/adr-061-p2p-refactor-scope.md>`_
- `ADR 62: P2P Architecture and Abstraction <../architecture/adr-062-p2p-architecture.md>`_
