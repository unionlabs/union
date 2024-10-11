==================================
RFC 010: Peer to Peer Light Client
==================================

Changelog
---------

- 2022-01-21: Initial draft (@tychoish)

Abstract
--------

The dependency on access to the RPC system makes running or using the light
client more complicated than it should be, because in practice node operators
choose to restrict access to these end points (often correctly.) There is no
deep dependency for the light client on the RPC system, and there is a
persistent notion that "make a p2p light client" is a solution to this
operational limitation. This document explores the implications and
requirements of implementing a p2p-based light client, as well as the
possibilities afforded by this implementation.

Background
----------

High Level Design
~~~~~~~~~~~~~~~~~

From a high level, the light client P2P implementation, is relatively straight
forward, but is orthogonal to the P2P-backed statesync implementation that
took place during the 0.35 cycle. The light client only really needs to be
able to request (and receive) a `LightBlock` at a given height. To support
this, a new Reactor would run on every full node and validator which would be
able to service these requests. The workload would be entirely
request-response, and the implementation of the reactor would likely be very
straight forward, and the implementation of the provider is similarly
relatively simple.

The complexity of the project focuses around peer discovery, handling when
peers disconnect from the light clients, and how to change the current P2P
code to appropriately handle specialized nodes.

I believe it's safe to assume that much of the current functionality of the
current ``light`` mode would *not* need to be maintained: there is no need to
proxy the RPC endpoints over the P2P layer and there may be no need to run a
node/process for the p2p light client (e.g. all use of this will be as a
client.)

The ability to run light clients using the RPC system will continue to be
maintained.

LibP2P
~~~~~~

While some aspects of the P2P light client implementation are orthogonal to
LibP2P project, it's useful to think about the ways that these efforts may
combine or interact.

We expect to be able to leverage libp2p tools to provide some kind of service
discovery for tendermint-based networks. This means that it will be possible
for the p2p stack to easily identify specialized nodes, (e.g. light clients)
thus obviating many of the design challenges with providing this feature in
the context of the current stack.

Similarly, libp2p makes it possible for a project to be able back their non-Go
light clients, without the major task of first implementing Tendermint's p2p
connection handling. We should identify if there exist users (e.g. the go IBC
relayer, it's maintainers, and operators) who would be able to take advantage
of p2p light client, before switching to libp2p. To our knowledge there are
limited implementations of this p2p protocol (a simple implementation without
secret connection support exists in rust but it has not been used in
production), and it seems unlikely that a team would implement this directly
ahead of its impending removal.

User Cases
~~~~~~~~~~

This RFC makes a few assumptions about the use cases and users of light
clients in tendermint.

The most active and delicate use cases for light clients is in the
implementation of the IBC relayer. Thus, we expect that providing P2P light
clients might increase the reliability of relayers and reduce the cost of
running a relayer, because relayer operators won't have to decide between rely
on public RPC endpoints (unreliable) or running their own full nodes
(expensive.) This also assumes that there are *no* other uses of the RPC in
the relayer, and unless the relayers have the option of dropping all RPC use,
it's unclear if a P2P light client will actually be able to successfully
remove the dependency on the RPC system.

Given that the primary relayer implementation is Hermes (rust,) it might be
safe to deliver a version of Tendermint that adds a light client rector in
the full nodes, but that does not provide an implementation of a Go light
client. This either means that the rust implementation would need support for
the legacy P2P connection protocol or wait for the libp2p implementation.

Client side light client (e.g. wallets, etc.) users may always want to use (a
subset) of the RPC rather than connect to the P2P network for an ephemeral
use.

Discussion
----------

Implementation Questions
~~~~~~~~~~~~~~~~~~~~~~~~

Most of the complication in the is how to have a long lived light client node
that *only* runs the light client reactor, as this raises a few questions:

- would users specify a single P2P node to connect to when creating a light
  client or would they also need/want to discover peers?

  - **answer**: most light client use cases won't care much about selecting
    peers (and those that do can either disable PEX and specify persistent
    peers, *or* use the RPC light client.)

- how do we prevent full nodes and validators from allowing their peer slots,
  which are typically limited, from filling with light clients? If
  light-clients aren't limited, how do we prevent light clients from consuming
  resources on consensus nodes?

  - **answer**: I think we can institute an internal cap on number of light
    client connections to accept and also elide light client nodes from PEX
    (pre-libp2p, if we implement this.) I believe that libp2p should provide
    us with the kind of service discovery semantics for network connectivity
    that would obviate this issue.

- when a light client disconnects from its peers will it need to reset its
  internal state (cache)? does this change if it connects to the same peers?

  - **answer**: no, the internal state only needs to be reset if the light
    client detects an invalid block or other divergence, and changing
    witnesses--which will be more common with a p2p light client--need not
    invalidate the cache.

These issues are primarily present given that the current peer management later
does not have a particularly good service discovery mechanism nor does it have
a very sophisticated way of identifying nodes of different types or modes.

Report Evidence
~~~~~~~~~~~~~~~

The current light client implementation currently has the ability to report
observed evidence. Either the notional light client reactor needs to be able
to handle these kinds of requests *or* all light client nodes need to also run
the evidence reactor. This could be configured at runtime.
