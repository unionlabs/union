# RFC 002: Interprocess Communication (IPC) in Tendermint

## Changelog

- 08-Sep-2021: Initial draft (@creachadair).


## Abstract

Communication in Tendermint among consensus nodes, applications, and operator
tools all use different message formats and transport mechanisms.  In some
cases there are multiple options. Having all these options complicates both the
code and the developer experience, and hides bugs. To support a more robust,
trustworthy, and usable system, we should document which communication paths
are essential, which could be removed or reduced in scope, and what we can
improve for the most important use cases.

This document proposes a variety of possible improvements of varying size and
scope. Specific design proposals should get their own documentation.


## Background

The Tendermint state replication engine has a complex IPC footprint.

1. Consensus nodes communicate with each other using a networked peer-to-peer
   message-passing protocol.

2. Consensus nodes communicate with the application whose state is being
   replicated via the [Application BlockChain Interface (ABCI)][abci].

3. Consensus nodes export a network-accessible [RPC service][rpc-service] to
   support operations (bootstrapping, debugging) and synchronization of [light clients][light-client].
   This interface is also used by the [`tendermint` CLI][tm-cli].

4. Consensus nodes export a gRPC service exposing a subset of the methods of
   the RPC service described by (3). This was intended to simplify the
   implementation of tools that already use gRPC to communicate with an
   application (via the Cosmos SDK), and wanted to also talk to the consensus
   node without implementing yet another RPC protocol.

   The gRPC interface to the consensus node has been deprecated and is slated
   for removal in the forthcoming Tendermint v0.36 release.

5. Consensus nodes may optionally communicate with a "remote signer" that holds
   a validator key and can provide public keys and signatures to the consensus
   node. One of the stated goals of this configuration is to allow the signer
   to be run on a private network, separate from the consensus node, so that a
   compromise of the consensus node from the public network would be less
   likely to expose validator keys.

## Discussion: Transport Mechanisms

### Remote Signer Transport

A remote signer communicates with the consensus node in one of two ways:

1. "Raw": Using a TCP or Unix-domain socket which carries varint-prefixed
   protocol buffer messages. In this mode, the consensus node is the server,
   and the remote signer is the client.

   This mode has been deprecated, and is intended to be removed.

2. gRPC: This mode uses the same protobuf messages as "Raw" node, but uses a
   standard encrypted gRPC HTTP/2 stub as the transport. In this mode, the
   remote signer is the server and the consensus node is the client.


### ABCI Transport

In ABCI, the _application_ is the server, and the Tendermint consensus engine
is the client.  Most applications implement the server using the [Cosmos SDK][cosmos-sdk],
which handles low-level details of the ABCI interaction and provides a
higher-level interface to the rest of the application. The SDK is written in Go.

Beneath the SDK, the application communicates with Tendermint core in one of
two ways:

- In-process direct calls (for applications written in Go and compiled against
  the Tendermint code).  This is an optimization for the common case where an
  application is written in Go, to save on the overhead of marshaling and
  unmarshaling requests and responses within the same process:
  [`abci/client/local_client.go`][local-client]

- A custom remote procedure protocol built on wire-format protobuf messages
  using a socket (the "socket protocol"): [`abci/server/socket_server.go`][socket-server]

The SDK also provides a [gRPC service][sdk-grpc] accessible from outside the
application, allowing transactions to be broadcast to the network, look up
transactions, and simulate transaction costs.


### RPC Transport

The consensus node RPC service allows callers to query consensus parameters
(genesis data, transactions, commits), node status (network info, health
checks), application state (abci_query, abci_info), mempool state, and other
attributes of the node and its application. The service also provides methods
allowing transactions and evidence to be injected ("broadcast") into the
blockchain.

The RPC service is exposed in several ways:

- HTTP GET: Queries may be sent as URI parameters, with method names in the path.

- HTTP POST: Queries may be sent as JSON-RPC request messages in the body of an
  HTTP POST request.  The server uses a custom implementation of JSON-RPC that
  is not fully compatible with the [JSON-RPC 2.0 spec][json-rpc], but handles
  the common cases.

- Websocket: Queries may be sent as JSON-RPC request messages via a websocket.
  This transport uses more or less the same JSON-RPC plumbing as the HTTP POST
  handler.

  The websocket endpoint also includes three methods that are _only_ exported
  via websocket, which appear to support event subscription.

- gRPC: A subset of queries may be issued in protocol buffer format to the gRPC
  interface described above under (4). As noted, this endpoint is deprecated
  and will be removed in v0.36.

### Opportunities for Simplification

**Claim:** There are too many IPC mechanisms.

The preponderance of ABCI usage is via the Cosmos SDK, which means the
application and the consensus node are compiled together into a single binary,
and the consensus node calls the ABCI methods of the application directly as Go
functions.

We also need a true IPC transport to support ABCI applications _not_ written in
Go.  There are also several known applications written in Rust, for example
(including [Anoma](https://github.com/anoma/anoma), Penumbra,
[Oasis](https://github.com/oasisprotocol/oasis-core), Twilight, and
[Nomic](https://github.com/nomic-io/nomic)). Ideally we will have at most one
such transport "built-in": More esoteric cases can be handled by a custom proxy.
Pragmatically, gRPC is probably the right choice here.

The primary consumers of the multi-headed "RPC service" today are the light
client and the `tendermint` command-line client. There is probably some local
use via curl, but I expect that is mostly ad hoc. Ethan reports that nodes are
often configured with the ports to the RPC service blocked, which is good for
security but complicates use by the light client.

### Context: Remote Signer Issues

Since the remote signer needs a secure communication channel to exchange keys
and signatures, and is expected to run truly remotely from the node (i.e., on a
separate physical server), there is not a whole lot we can do here. We should
finish the deprecation and removal of the "raw" socket protocol between the
consensus node and remote signers, but the use of gRPC is appropriate.

The main improvement we can make is to simplify the implementation quite a bit,
once we no longer need to support both "raw" and gRPC transports.

### Context: ABCI Issues

In the original design of ABCI, the presumption was that all access to the
application should be mediated by the consensus node. The idea is that outside
access could change application state and corrupt the consensus process, which
relies on the application to be deterministic. Of course, even without outside
access an application could behave nondeterministically, but allowing other
programs to send it requests was seen as courting trouble.

Conversely, users noted that most of the time, tools written for a particular
application don't want to talk to the consensus module directly. The
application "owns" the state machine the consensus engine is replicating, so
tools that care about application state should talk to the application.
Otherwise, they would have to bake in knowledge about Tendermint (e.g., its
interfaces and data structures) just because of the mediation.

For clients to talk directly to the application, however, there is another
concern: The consensus node is the ABCI _client_, so it is inconvenient for the
application to "push" work into the consensus module via ABCI itself.  The
current implementation works around this by calling the consensus node's RPC
service, which exposes an `ABCIQuery` kitchen-sink method that allows the
application a way to poke ABCI messages in the other direction.

Without this RPC method, you could work around this (at least in principle) by
having the consensus module "poll" the application for work that needs done,
but that has unsatisfactory implications for performance and robustness, as
well as being harder to understand.

There has apparently been discussion about trying to make a more bidirectional
communication between the consensus node and the application, but this issue
seems to still be unresolved.

Another complication of ABCI is that it requires the application (server) to
maintain [four separate connections][abci-conn]: One for "consensus" operations
(BeginBlock, EndBlock, DeliverTx, Commit), one for "mempool" operations, one
for "query" operations, and one for "snapshot" (state synchronization) operations.
The rationale seems to have been that these groups of operations should be able
to proceed concurrently with each other. In practice, it results in a very complex
state management problem to coordinate state updates between the separate streams.
While application authors in Go are mostly insulated from that complexity by the
Cosmos SDK, the plumbing to maintain those separate streams is complicated, hard
to understand, and we suspect it contains concurrency bugs and/or lock contention
issues affecting performance that are subtle and difficult to pin down.

Even without changing the semantics of any ABCI operations, this code could be
made smaller and easier to debug by separating the management of concurrency
and locking from the IPC transport: If all requests and responses are routed
through one connection, the server can explicitly maintain priority queues for
requests and responses, and make less-conservative decisions about when locks
are (or aren't) required to synchronize state access. With independent queues,
the server must lock conservatively, and no optimistic scheduling is practical.

This would be a tedious implementation change, but should be achievable without
breaking any of the existing interfaces. More importantly, it could potentially
address a lot of difficult concurrency and performance problems we currently
see anecdotally but have difficultly isolating because of how intertwined these
separate message streams are at runtime.

TODO: Impact of ABCI++ for this topic?

### Context: RPC Issues

The RPC system serves several masters, and has a complex surface area. I
believe there are some improvements that can be exposed by separating some of
these concerns.

The Tendermint light client currently uses the RPC service to look up blocks
and transactions, and to forward ABCI queries to the application.  The light
client proxy uses the RPC service via a websocket. The Cosmos IBC relayer also
uses the RPC service via websocket to watch for transaction events, and uses
the `ABCIQuery` method to fetch information and proofs for posted transactions.

Some work is already underway toward using P2P message passing rather than RPC
to synchronize light client state with the rest of the network.  IBC relaying,
however, requires access to the event system, which is currently not accessible
except via the RPC interface. Event subscription _could_ be exposed via P2P,
but that is a larger project since it adds P2P communication load, and might
thus have an impact on the performance of consensus.

If event subscription can be moved into the P2P network, we could entirely
remove the websocket transport, even for clients that still need access to the
RPC service. Until then, we may still be able to reduce the scope of the
websocket endpoint to _only_ event subscription, by moving uses of the RPC
server as a proxy to ABCI over to the gRPC interface.

Having the RPC server still makes sense for local bootstrapping and operations,
but can be further simplified. Here are some specific proposals:

- Remove the HTTP GET interface entirely.

- Simplify JSON-RPC plumbing to remove unnecessary reflection and wrapping.

- Remove the gRPC interface (this is already planned for v0.36).

- Separate the websocket interface from the rest of the RPC service, and
  restrict it to only event subscription.

  Eventually we should try to emove the websocket interface entirely, but we
  will need to revisit that (probably in a new RFC) once we've done some of the
  easier things.

These changes would preserve the ability of operators to issue queries with
curl (but would require using JSON-RPC instead of URI parameters). That would
be a little less user-friendly, but for a use case that should not be that
prevalent.

These changes would also preserve compatibility with existing JSON-RPC based
code paths like the `tendermint` CLI and the light client (even ahead of
further work to remove that dependency).

**Design goal:** An operator should be able to disable non-local access to the
RPC server on any node in the network without impairing the ability of the
network to function for service of state replication, including light clients.

**Design principle:** All communication required to implement and monitor the
consensus network should use P2P, including the various synchronizations.

### Options for ABCI Transport

The majority of current usage is in Go, and the majority of that is mediated by
the Cosmos SDK, which uses the "direct call" interface. There is probably some
opportunity to clean up the implementation of that code, notably by inverting
which interface is at the "top" of the abstraction stack (currently it acts
like an RPC interface, and escape-hatches into the direct call). However, this
general approach works fine and doesn't need to be fundamentally changed.

For applications _not_ written in Go, the two remaining options are the
"socket" protocol (another variation on varint-prefixed protobuf messages over
an unstructured stream) and gRPC. It would be nice if we could get rid of one
of these to reduce (unneeded?) optionality.

Since both the socket protocol and gRPC depend on protocol buffers, the
"socket" protocol is the most obvious choice to remove. While gRPC is more
complex, the set of languages that _have_ protobuf support but _lack_ gRPC
support is small. Moreover, gRPC is already widely used in the rest of the
ecosystem (including the Cosmos SDK).

If some use case did arise later that can't work with gRPC, it would not be too
difficult for that application author to write a little proxy (in Go) that
bridges the convenient SDK APIs into a simpler protocol than gRPC.

**Design principle:** It is better for an uncommon special case to carry the
burdens of its specialness, than to bake an escape hatch into the infrastructure.

**Recommendation:** We should deprecate and remove the socket protocol.

### Options for RPC Transport

[ADR 057][adr-57] proposes using gRPC for the Tendermint RPC implementation.
This is still possible, but if we are able to simplify and decouple the
concerns as described above, I do not think it should be necessary.

While JSON-RPC is not the best possible RPC protocol for all situations, it has
some advantages over gRPC for our domain. Specifically:

- It is easy to call JSON-RPC manually from the command-line, which helps with
  a common concern for the RPC service, local debugging and operations.

  Relatedly: JSON is relatively easy for humans to read and write, and it can
  be easily copied and pasted to share sample queries and debugging results in
  chat, issue comments, and so on. Ideally, the RPC service will not be used
  for activities where the costs of a text protocol are important compared to
  its legibility and manual usability benefits.

- gRPC has an enormous dependency footprint for both clients and servers, and
  many of the features it provides to support security and performance
  (encryption, compression, streaming, etc.) are mostly irrelevant to local
  use. Tendermint already needs to include a gRPC client for the remote signer,
  but if we can avoid the need for a _client_ to depend on gRPC, that is a win
  for usability.

- If we intend to migrate light clients off RPC to use P2P entirely, there is
  no advantage to forcing a temporary migration to gRPC along the way; and once
  the light client is not dependent on the RPC service, the efficiency of the
  protocol is much less important.

- We can still get the benefits of generated data types using protocol buffers, even
  without using gRPC:

    - Protobuf defines a standard JSON encoding for all message types so
    languages with protobuf support do not need to worry about type mapping
    oddities.

    - Using JSON means that even languages _without_ good protobuf support can
    implement the protocol with a bit more work, and I expect this situation to
    be rare.

Even if a language lacks a good standard JSON-RPC mechanism, the protocol is
lightweight and can be implemented by simple send/receive over TCP or
Unix-domain sockets with no need for code generation, encryption, etc. gRPC
uses a complex HTTP/2 based transport that is not easily replicated.

### Future Work

The background and proposals sketched above focus on the existing structure of
Tendermint and improvements we can make in the short term. It is worthwhile to
also consider options for longer-term broader changes to the IPC ecosystem.
The following outlines some ideas at a high level:

- **Consensus service:** Today, the application and the consensus node are
  nominally connected only via ABCI. Tendermint was originally designed with
  the assumption that all communication with the application should be mediated
  by the consensus node.  Based on further experience, however, the design goal
  is now that the _application_ should be the mediator of application state.

  As noted above, however, ABCI is a client/server protocol, with the
  application as the server. For outside clients that turns out to have been a
  good choice, but it complicates the relationship between the application and
  the consensus node: Previously transactions were entered via the node, now
  they are entered via the app.

  We have worked around this by using the Tendermint RPC service to give the
  application a "back channel" to the consensus node, so that it can push
  transactions back into the consensus network. But the RPC service exposes a
  lot of other functionality, too, including event subscription, block and
  transaction queries, and a lot of node status information.

  Even if we can't easily "fix" the orientation of the ABCI relationship, we
  could improve isolation by splitting out the parts of the RPC service that
  the application needs as a back-channel, and sharing those _only_ with the
  application. By defining a "consensus service", we could give the application
  a way to talk back limited to only the capabilities it needs. This approach
  has the benefit that we could do it without breaking existing use, and if we
  later did "fix" the ABCI directionality, we could drop the special case
  without disrupting the rest of the RPC interface.

- **Event service:** Right now, the IBC relayer relies on the Tendermint RPC
  service to provide a stream of block and transaction events, which it uses to
  discover which transactions need relaying to other chains.  While I think
  that event subscription should eventually be handled via P2P, we could gain
  some immediate benefit by splitting out event subscription from the rest of
  the RPC service.

  In this model, an event subscription service would be exposed on the public
  network, but on a different endpoint. This would remove the need for the RPC
  service to support the websocket protocol, and would allow operators to
  isolate potentially sensitive status query results from the public network.

  At the moment the relayers also use the RPC service to get block data for
  synchronization, but work is already in progress to handle that concern via
  the P2P layer. Once that's done, event subscription could be separated.

Separating parts of the existing RPC service is not without cost: It might
require additional connection endpoints, for example, though it is also not too
difficult for multiple otherwise-independent services to share a connection.

In return, though, it would become easier to reduce transport options and for
operators to independently control access to sensitive data. Considering the
viability and implications of these ideas is beyond the scope of this RFC, but
they are documented here since they follow from the background we have already
discussed.

## References

[abci]: https://github.com/tendermint/tendermint/tree/main/spec/abci
[rpc-service]: https://docs.tendermint.com/v0.34/rpc/
[light-client]: https://docs.tendermint.com/v0.34/tendermint-core/light-client.html
[tm-cli]: https://github.com/tendermint/tendermint/tree/main/cmd/tendermint
[cosmos-sdk]: https://github.com/cosmos/cosmos-sdk/
[local-client]: https://github.com/tendermint/tendermint/blob/main/abci/client/local_client.go
[socket-server]: https://github.com/tendermint/tendermint/blob/main/abci/server/socket_server.go
[sdk-grpc]: https://pkg.go.dev/github.com/cosmos/cosmos-sdk/types/tx#ServiceServer
[json-rpc]: https://www.jsonrpc.org/specification
[abci-conn]: https://github.com/tendermint/tendermint/blob/main/spec/abci/abci++_basic_concepts.md
[adr-57]: https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-057-RPC.md
