# ADR 061: P2P Refactor Scope

## Changelog

- 2020-10-30: Initial version (@erikgrinaker)

## Context

The `p2p` package responsible for peer-to-peer networking is rather old and has a number of weaknesses, including tight coupling, leaky abstractions, lack of tests, DoS vulnerabilities, poor performance, custom protocols, and incorrect behavior. A refactor has been discussed for several years ([#2067](https://github.com/tendermint/tendermint/issues/2067)).

Informal Systems are also building a Rust implementation of Tendermint, [Tendermint-rs](https://github.com/informalsystems/tendermint-rs), and plan to implement P2P networking support over the next year. As part of this work, they have requested adopting e.g. [QUIC](https://datatracker.ietf.org/doc/draft-ietf-quic-transport/) as a transport protocol instead of implementing the custom application-level `MConnection` stream multiplexing protocol that Tendermint currently uses.

This ADR summarizes recent discussion with stakeholders on the scope of a P2P refactor. Specific designs and implementations will be submitted as separate ADRs.

## Alternative Approaches

There have been recurring proposals to adopt [LibP2P](https://libp2p.io) instead of maintaining our own P2P networking stack (see [#3696](https://github.com/tendermint/tendermint/issues/3696)). While this appears to be a good idea in principle, it would be a highly breaking protocol change, there are indications that we might have to fork and modify LibP2P, and there are concerns about the abstractions used.

In discussions with Informal Systems we decided to begin with incremental improvements to the current P2P stack, add support for pluggable transports, and then gradually start experimenting with LibP2P as a transport layer. If this proves successful, we can consider adopting it for higher-level components at a later time.

## Decision

The P2P stack will be refactored and improved iteratively, in several phases:

* **Phase 1:** code and API refactoring, maintaining protocol compatibility as far as possible.

* **Phase 2:** additional transports and incremental protocol improvements.

* **Phase 3:** disruptive protocol changes.

The scope of phases 2 and 3 is still uncertain, and will be revisited once the preceding phases have been completed as we'll have a better sense of requirements and challenges.

## Detailed Design

Separate ADRs will be submitted for specific designs and changes in each phase, following research and prototyping. Below are objectives in order of priority.

### Phase 1: Code and API Refactoring

This phase will focus on improving the internal abstractions and implementations in the `p2p` package. As far as possible, it should not change the P2P protocol in a backwards-incompatible way.

* Cleaner, decoupled abstractions for e.g. `Reactor`, `Switch`, and `Peer`. [#2067](https://github.com/tendermint/tendermint/issues/2067) [#5287](https://github.com/tendermint/tendermint/issues/5287) [#3833](https://github.com/tendermint/tendermint/issues/3833)
    * Reactors should receive messages in separate goroutines or via buffered channels. [#2888](https://github.com/tendermint/tendermint/issues/2888)
* Improved peer lifecycle management. [#3679](https://github.com/tendermint/tendermint/issues/3679) [#3719](https://github.com/tendermint/tendermint/issues/3719) [#3653](https://github.com/tendermint/tendermint/issues/3653) [#3540](https://github.com/tendermint/tendermint/issues/3540) [#3183](https://github.com/tendermint/tendermint/issues/3183) [#3081](https://github.com/tendermint/tendermint/issues/3081) [#1356](https://github.com/tendermint/tendermint/issues/1356)
    * Peer prioritization. [#2860](https://github.com/tendermint/tendermint/issues/2860) [#2041](https://github.com/tendermint/tendermint/issues/2041)
* Pluggable transports, with `MConnection` as one implementation. [#5587](https://github.com/tendermint/tendermint/issues/5587) [#2430](https://github.com/tendermint/tendermint/issues/2430) [#805](https://github.com/tendermint/tendermint/issues/805)
* Improved peer address handling.
    * Address book refactor. [#4848](https://github.com/tendermint/tendermint/issues/4848) [#2661](https://github.com/tendermint/tendermint/issues/2661)
    * Transport-agnostic peer addressing. [#5587](https://github.com/tendermint/tendermint/issues/5587) [#3782](https://github.com/tendermint/tendermint/issues/3782) [#3692](https://github.com/tendermint/tendermint/issues/3692)
    * Improved detection and advertisement of own address. [#5588](https://github.com/tendermint/tendermint/issues/5588) [#4260](https://github.com/tendermint/tendermint/issues/4260) [#3716](https://github.com/tendermint/tendermint/issues/3716) [#1727](https://github.com/tendermint/tendermint/issues/1727)
    * Support multiple IPs per peer. [#1521](https://github.com/tendermint/tendermint/issues/1521) [#2317](https://github.com/tendermint/tendermint/issues/2317)

The refactor should attempt to address the following secondary objectives: testability, observability, performance, security, quality-of-service, backpressure, and DoS resilience. Much of this will be revisited as explicit objectives in phase 2.

Ideally, the refactor should happen incrementally, with regular merges to `master` every few weeks. This will take more time overall, and cause frequent breaking changes to internal Go APIs, but it reduces the branch drift and gets the code tested sooner and more broadly.

### Phase 2: Additional Transports and Protocol Improvements

This phase will focus on protocol improvements and other breaking changes. The following are considered proposals that will need to be evaluated separately once the refactor is done. Additional proposals are likely to be added during phase 1.

* QUIC transport. [#198](https://github.com/tendermint/spec/issues/198)
* Noise protocol for secret connection handshake. [#5589](https://github.com/tendermint/tendermint/issues/5589) [#3340](https://github.com/tendermint/tendermint/issues/3340)
* Peer ID in connection handshake. [#5590](https://github.com/tendermint/tendermint/issues/5590)
* Peer and service discovery (e.g. RPC nodes, state sync snapshots). [#5481](https://github.com/tendermint/tendermint/issues/5481) [#4583](https://github.com/tendermint/tendermint/issues/4583)
* Rate-limiting, backpressure, and QoS scheduling. [#4753](https://github.com/tendermint/tendermint/issues/4753) [#2338](https://github.com/tendermint/tendermint/issues/2338)
* Compression. [#2375](https://github.com/tendermint/tendermint/issues/2375)
* Improved metrics and tracing. [#3849](https://github.com/tendermint/tendermint/issues/3849) [#2600](https://github.com/tendermint/tendermint/issues/2600)
* Simplified P2P configuration options.

### Phase 3: Disruptive Protocol Changes

This phase covers speculative, wide-reaching proposals that are poorly defined and highly uncertain. They will be evaluated once the previous phases are done.

* Adopt LibP2P. [#3696](https://github.com/tendermint/tendermint/issues/3696)
* Allow cross-reactor communication, possibly without channels.
* Dynamic channel advertisement, as reactors are enabled/disabled. [#4394](https://github.com/tendermint/tendermint/issues/4394) [#1148](https://github.com/tendermint/tendermint/issues/1148)
* Pubsub-style networking topology and pattern.
* Support multiple chain IDs in the same network.

## Status

Proposed

## Consequences

### Positive

* Cleaner, simpler architecture that's easier to reason about and test, and thus hopefully less buggy.

* Improved performance and robustness.

* Reduced maintenance burden and increased interoperability by the possible adoption of standardized protocols such as QUIC and Noise.

* Improved usability, with better observability, simpler configuration, and more automation (e.g. peer/service/address discovery, rate-limiting, and backpressure).

### Negative

* Maintaining our own P2P networking stack is resource-intensive.

* Abstracting away the underlying transport may prevent usage of advanced transport features.

* Breaking changes to APIs and protocols are disruptive to users.

## References

See issue links above.

- [#2067: P2P Refactor](https://github.com/tendermint/tendermint/issues/2067)

- [P2P refactor brainstorm document](https://docs.google.com/document/d/1FUTADZyLnwA9z7ndayuhAdAFRKujhh_y73D0ZFdKiOQ/edit?pli=1#)
