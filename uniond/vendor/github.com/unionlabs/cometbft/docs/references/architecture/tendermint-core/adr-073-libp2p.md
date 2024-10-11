# ADR 073: Adopt LibP2P

## Changelog

- 2021-11-02: Initial Draft (@tychoish)

## Status

Proposed.

## Context


As part of the 0.35 development cycle, the Tendermint team completed
the first phase of the work described in ADRs 61 and 62, which included a
large scale refactoring of the reactors and the p2p message
routing. This replaced the switch and many of the other legacy
components without breaking protocol or network-level
interoperability and left the legacy connection/socket handling code.

Following the release, the team has reexamined the state of the code
and the design, as well as Tendermint's requirements. The notes
from that process are available in the [P2P Roadmap
RFC][rfc].

This ADR supersedes the decisions made in ADRs 60 and 61, but
builds on the completed portions of this work. Previously, the
boundaries of peer management, message handling, and the higher level
business logic (e.g., "the reactors") were intermingled, and core
elements of the p2p system were responsible for the orchestration of
higher-level business logic. Refactoring the legacy components
made it more obvious that this entanglement of responsibilities
had outsized influence on the entire implementation, making
it difficult to iterate within the current abstractions.
It would not be viable to maintain interoperability with legacy
systems while also achieving many of our broader objectives.

LibP2P is a thoroughly-specified implementation of a peer-to-peer
networking stack, designed specifically for systems such as
ours. Adopting LibP2P as the basis of Tendermint will allow the
Tendermint team to focus more of their time on other differentiating
aspects of the system, and make it possible for the ecosystem as a
whole to take advantage of tooling and efforts of the LibP2P
platform.

## Alternative Approaches

As discussed in the [P2P Roadmap RFC][rfc], the primary alternative would be to
continue development of Tendermint's home-grown peer-to-peer
layer. While that would give the Tendermint team maximal control
over the peer system, the current design is unexceptional on its
own merits, and the prospective maintenance burden for this system
exceeds our tolerances for the medium term.

Tendermint can and should differentiate itself not on the basis of
its networking implementation or peer management tools, but providing
a consistent operator experience, a battle-tested consensus algorithm,
and an ergonomic user experience.

## Decision

Tendermint will adopt libp2p during the 0.37 development cycle,
replacing the bespoke Tendermint P2P stack. This will remove the
`Endpoint`, `Transport`, `Connection`, and `PeerManager` abstractions
and leave the reactors, `p2p.Router` and `p2p.Channel`
abstractions.

LibP2P may obviate the need for a dedicated peer exchange (PEX)
reactor, which would also in turn obviate the need for a dedicated
seed mode. If this is the case, then all of this functionality would
be removed.

If it turns out (based on the advice of Protocol Labs) that it makes
sense to maintain separate pubsub or gossipsub topics
per-message-type, then the `Router` abstraction could also
be entirely subsumed.

## Detailed Design

### Implementation Changes

The seams in the P2P implementation between the higher level
constructs (reactors), the routing layer (`Router`) and the lower
level connection and peer management code make this operation
relatively straightforward to implement. A key
goal in this design is to minimize the impact on the reactors
(potentially entirely,) and completely remove the lower level
components (e.g., `Transport`, `Connection` and `PeerManager`) using the
separation afforded by the `Router` layer. The current state of the
code makes these changes relatively surgical, and limited to a small
number of methods:

- `p2p.Router.OpenChannel` will still return a `Channel` structure
  which will continue to serve as a pipe between the reactors and the
  `Router`. The implementation will no longer need the queue
  implementation, and will instead start goroutines that
  are responsible for routing the messages from the channel to libp2p
  fundamentals, replacing the current `p2p.Router.routeChannel`.

- The current `p2p.Router.dialPeers` and `p2p.Router.acceptPeers`,
  are responsible for establishing outbound and inbound connections,
  respectively. These methods will be removed, along with
  `p2p.Router.openConnection`, and the libp2p connection manager will
  be responsible for maintaining network connectivity.

- The `p2p.Channel` interface will change to replace Go
  channels with a more functional interface for sending messages.
  New methods on this object will take contexts to support safe
  cancellation, and return errors, and will block rather than
  running asynchronously. The `Out` channel through which
  reactors send messages to Peers, will be replaced by a `Send`
  method, and the Error channel will be replaced by an `Error`
  method.

- Reactors will be passed an interface that will allow them to
  access Peer information from libp2p. This will supplant the
  `p2p.PeerUpdates` subscription.

- Add some kind of heartbeat message at the application level
  (e.g. with a reactor,) potentially connected to libp2p's DHT to be
  used by reactors for service discovery, message targeting, or other
  features.

- Replace the existing/legacy handshake protocol with [Noise](http://www.noiseprotocol.org/noise.html).

This project will initially use the TCP-based transport protocols within
libp2p. QUIC is also available as an option that we may implement later.
We will not support mixed networks in the initial release, but will
revisit that possibility later if there is a demonstrated need.

### Upgrade and Compatibility

Because the routers and all current P2P libraries are `internal`
packages and not part of the public API, the only changes to the public
API surface area of Tendermint will be different configuration
file options, replacing the current P2P options with options relevant
to libp2p.

However, it will not be possible to run a network with both networking
stacks active at once, so the upgrade to the version of Tendermint
will need to be coordinated between all nodes of the network. This is
consistent with the expectations around upgrades for Tendermint moving
forward, and will help manage both the complexity of the project and
the implementation timeline.

## Open Questions

- What is the role of Protocol Labs in the implementation of libp2p in
  tendermint, both during the initial implementation and on an ongoing
  basis thereafter?

- Should all P2P traffic for a given node be pushed to a single topic,
  so that a topic maps to a specific ChainID, or should
  each reactor (or type of message) have its own topic? How many
  topics can a libp2p network support? Is there testing that validates
  the capabilities?

- Tendermint presently provides a very coarse QoS-like functionality
  using priorities based on message-type.
  This intuitively/theoretically ensures that evidence and consensus
  messages don't get starved by blocksync/statesync messages. It's
  unclear if we can or should attempt to replicate this with libp2p.

- What kind of QoS functionality does libp2p provide and what kind of
  metrics does libp2p provide about it's QoS functionality?

- Is it possible to store additional (and potentially arbitrary)
  information into the DHT as part of the heartbeats between nodes,
  such as the latest height, and then access that in the
  reactors. How frequently can the DHT be updated?

- Does it make sense to have reactors continue to consume inbound
  messages from a Channel (`In`) or is there another interface or
  pattern that we should consider?

  - We should avoid exposing Go channels when possible, and likely
	some kind of alternate iterator likely makes sense for processing
	messages within the reactors.

- What are the security and protocol implications of tracking
  information from peer heartbeats and exposing that to reactors?

- How much (or how little) configuration can Tendermint provide for
  libp2p, particularly on the first release?

  - In general, we should not support byo-functionality for libp2p
	components within Tendermint, and reduce the configuration surface
	area, as much as possible.

- What are the best ways to provide request/response semantics for
  reactors on top of libp2p? Will it be possible to add
  request/response semantics in a future release or is there
  anticipatory work that needs to be done as part of the initial
  release?

## Consequences

### Positive

- Reduce the maintenance burden for the Tendermint Core team by
  removing a large swath of legacy code that has proven to be
  difficult to modify safely.

- Remove the responsibility for maintaining and developing the entire
  peer management system (p2p) and stack.

- Provide users with a more stable peer and networking system,
  Tendermint can improve operator experience and network stability.

### Negative

- By deferring to library implementations for peer management and
  networking, Tendermint loses some flexibility for innovating at the
  peer and networking level. However, Tendermint should be innovating
  primarily at the consensus layer, and libp2p does not preclude
  optimization or development in the peer layer.

- Libp2p is a large dependency and Tendermint would become dependent
  upon Protocol Labs' release cycle and prioritization for bug
  fixes. If this proves onerous, it's possible to maintain a vendor
  fork of relevant components as needed.

### Neutral

- N/A

## References

- [ADR 61: P2P Refactor Scope][adr61]
- [ADR 62: P2P Architecture][adr62]
- [P2P Roadmap RFC][rfc]

[adr61]: adr-061-p2p-refactor-scope.md
[adr62]: adr-062-p2p-architecture.md
[rfc]: ../rfc/rfc-000-p2p-roadmap.rst
