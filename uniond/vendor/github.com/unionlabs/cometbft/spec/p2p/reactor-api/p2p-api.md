# API for Reactors

This document describes the API provided by the p2p layer to the protocol
layer, namely to the registered reactors.

This API consists of two interfaces: the one provided by the `Switch` instance,
and the ones provided by multiple `Peer` instances, one per connected peer.
The `Switch` instance is provided to every reactor as part of the reactor's
[registration procedure][reactor-registration].
The multiple `Peer` instances are provided to every registered reactor whenever
a [new connection with a peer][reactor-addpeer] is established.

> **Note**
>
> The practical reasons that lead to the interface to be provided in two parts,
> `Switch` and `Peer` instances are discussed in more datail in the
> [knowledge-base repository](https://github.com/cometbft/knowledge-base/blob/main/p2p/reactors/switch-peer.md).

## `Switch` API

The [`Switch`][switch-type] is the central component of the p2p layer
implementation.  It manages all the reactors running in a node and keeps track
of the connections with peers.
The table below summarizes the interaction of the standard reactors with the `Switch`:

| `Switch` API method                                     | consensus | block sync | state sync | mempool | evidence  | PEX   |
|--------------------------------------------|-----------|------------|------------|---------|-----------|-------|
| `Peers() IPeerSet`                         | x         | x          |            |         |           | x     |
| `NumPeers() (int, int, int)`               |           | x          |            |         |           | x     |
| `Broadcast(Envelope) chan bool`            | x         | x          | x          |         |           |       |
| `MarkPeerAsGood(Peer)`                     | x         |            |            |         |           |       |
| `StopPeerForError(Peer, interface{})`      | x         | x          | x          | x       | x         | x     |
| `StopPeerGracefully(Peer)`                 |           |            |            |         |           | x     |
| `Reactor(string) Reactor`                  |           | x          |            |         |           |       |

The above list is not exhaustive as it does not include all the `Switch` methods
invoked by the PEX reactor, a special component that should be considered part
of the p2p layer. This document does not cover the operation of the PEX reactor
as a connection manager.

### Peers State

The first two methods in the switch API allow reactors to query the state of
the p2p layer: the set of connected peers.

    func (sw *Switch) Peers() IPeerSet

The `Peers()` method returns the current set of connected peers. The returned
`IPeerSet` is concurrency-safe. Observe that the `Peer` handlers returned by
this method were previously [added to the reactor][reactor-addpeer] via the
`InitPeer(Peer)` method, but not yet removed via the `RemovePeer(Peer)` method.
Thus, a priori, reactors should already have this information.

    func (sw *Switch) NumPeers() (outbound, inbound, dialing int)

The `NumPeers()` method returns the current number of connected peers,
distinguished between `outbound` and `inbound` peers. An `outbound` peer is a
peer the node has dialed to, while an `inbound` peer is a peer the node has
accepted a connection from. Note that `unconditional` peers are not being
counted here.

The third field `dialing` reports the number of peers to which the node is
currently attempting to connect, so not (yet) connected peers.

> **Note**
>
> The third field returned by `NumPeers()`, the number of peers in `dialing`
> state, is not an information that should regard the protocol layer.
> In fact, with the exception of the PEX reactor, which can be considered part
> of the p2p layer implementation, no standard reactor actually uses this
> information, that could be removed when this interface is refactored.

### Broadcast

The switch provides, mostly for historical or retro-compatibility reasons,
a method for sending a message to all connected peers:

    func (sw *Switch) Broadcast(e Envelope) chan bool

The `Broadcast()` method is not blocking and returns a channel of booleans.
For every connected `Peer`, it starts a background thread for sending the
message to that peer, using the `Peer.Send()` method
(which is blocking, as detailed in [Send Methods](#send-methods)).
The result of each unicast send operation (success or failure) is added to the
returned channel, which is closed when all operations are completed.

> **Note**
>
> - The current _implementation_ of the `Switch.Broadcast(Envelope)` method is
>   not efficient, as the marshalling of the provided message is performed as
>   part of the `Peer.Send(Envelope)` helper method, that is, once per
>   connected peer.
> - The return value of the broadcast method is not considered by any of the
>   standard reactors that employ the method. One of the reasons is that is is
>   not possible to associate each of the boolean outputs added to the
>   returned channel to a peer.

### Vetting Peers

The p2p layer relies on the registered reactors to gauge the _quality_ of peers.
The following method can be invoked by a reactor to inform the p2p layer that a
peer has presented a "good" behaviour.
This information is registered in the node's address book and influences the
operation of the Peer Exchange (PEX) protocol, as node discovery adopts a bias
towards "good" peers:

    func (sw *Switch) MarkPeerAsGood(peer Peer)

At the moment, it is up to the consensus reactor to vet a peer.
In the current logic, a peer is marked as good whenever the consensus protocol
collects a multiple of `votesToContributeToBecomeGoodPeer = 10000` useful votes
or `blocksToContributeToBecomeGoodPeer = 10000` useful block parts from that peer.
By "useful", the consensus implementation considers messages that are valid and
that are received by the node when the node is expected for such information,
which excludes duplicated or late received messages.

> **Note**
>
> The switch doesn't currently provide a method to mark a peer as a bad peer.
> In fact, the peer quality management is really implemented in the current
> version of the p2p layer.
> This topic is being discussed in the [knowledge-base repository](https://github.com/cometbft/knowledge-base/blob/main/p2p/reactors/peer-quality.md).

### Stopping Peers

Reactors can instruct the p2p layer to disconnect from a peer.
Using the p2p layer's nomenclature, the reactor requests a peer to be stopped.
The peer's send and receive routines are in fact stopped, interrupting the
communication with the peer.
The `Peer` is then [removed from every registered reactor][reactor-removepeer],
using the `RemovePeer(Peer)` method, and from the set of connected peers.

    func (sw *Switch) StopPeerForError(peer Peer, reason interface{})

All the standard reactors employ the above method for disconnecting from a peer
in case of errors.
These are errors that occur when processing a message received from a `Peer`.
The produced `error` is provided to the method as the `reason`.

The `StopPeerForError()` method has an important *caveat*: if the peer to be
stopped is configured as a _persistent peer_, the switch will attempt
reconnecting to that same peer.
While this behaviour makes sense when the method is invoked by other components
of the p2p layer (e.g., in the case of communication errors), it does not make
sense when it is invoked by a reactor.

> **Note**
>
> A more comprehensive discussion regarding this topic can be found on the
> [knowledge-base repository](https://github.com/cometbft/knowledge-base/blob/main/p2p/reactors/stop-peer.md).

    func (sw *Switch) StopPeerGracefully(peer Peer)

The second method instructs the switch to disconnect from a peer for no
particular reason.
This method is only adopted by the PEX reactor of a node operating in _seed mode_,
as seed nodes disconnect from a peer after exchanging peer addresses with it.

### Reactors Table

The switch keeps track of all registered reactors, indexed by unique reactor names.
A reactor can therefore use the switch to access another `Reactor` from its `name`:

    func (sw *Switch) Reactor(name string) Reactor

This method is currently only used by the Block Sync reactor to access the
Consensus reactor implementation, from which it uses the exported
`SwitchToConsensus()` method.
While available, this inter-reactor interaction approach is discouraged and
should be avoided, as it violates the assumption that reactors are independent.


## `Peer` API

The [`Peer`][peer-interface] interface represents a connected peer.
A `Peer` instance encapsulates a multiplex connection that implements the
actual communication (sending and receiving messages) with a peer.
When a connection is established with a peer, the `Switch` provides the
corresponding `Peer` instance to all registered reactors.
From this point, reactors can use the methods of the new `Peer` instance.

The table below summarizes the interaction of the standard reactors with
connected peers, with the `Peer` methods used by them:

| `Peer` API method                                     | consensus | block sync | state sync | mempool | evidence  | PEX   |
|--------------------------------------------|-----------|------------|------------|---------|-----------|-------|
| `ID() ID`                                  | x         | x          | x          | x       | x         | x     |
| `IsRunning() bool`                         | x         |            |            | x       | x         |       |
| `Quit() <-chan struct{}`                   |           |            |            | x       | x         |       |
| `Get(string) interface{}`                  | x         |            |            | x       | x         |       |
| `Set(string, interface{})`                 | x         |            |            |         |           |       |
| `Send(Envelope) bool`                      | x         | x          | x          | x       | x         | x     |
| `TrySend(Envelope) bool`                   | x         | x          |            |         |           |       |

The above list is not exhaustive as it does not include all the `Peer` methods
invoked by the PEX reactor, a special component that should be considered part
of the p2p layer. This document does not cover the operation of the PEX reactor
as a connection manager.

### Identification

Nodes in the p2p network are configured with a unique cryptographic key pair.
The public part of this key pair is verified when establishing a connection
with the peer, as part of the authentication handshake, and constitutes the
peer's `ID`:

    func (p Peer) ID() p2p.ID

Observe that each time the node connects to a peer (e.g., after disconnecting
from it), a new (distinct) `Peer` handler is provided to the reactors via
`InitPeer(Peer)` method.
In fact, the `Peer` handler is associated to a _connection_ with a peer, not to
the actual _node_ in the network.
To keep track of actual peers, the unique peer `p2p.ID` provided by the above
method should be employed.

### Peer state

The switch starts the peer's send and receive routines before adding the peer
to every registered reactor using the `AddPeer(Peer)` method.
The reactors then usually start routines to interact with the new connected
peer using the received `Peer` handler.
For these routines it is useful to check whether the peer is still connected
and its send and receive routines are still running:

    func (p Peer) IsRunning() bool
    func (p Peer) Quit() <-chan struct{}

The above two methods provide the same information about the state of a `Peer`
instance in two different ways.
Both of them are defined in the  [`Service`][service-interface] interface.
The `IsRunning()` method is synchronous and returns whether the peer has been
started and has not been stopped.
The `Quit()` method returns a channel that is closed when the peer is stopped;
it is an asynchronous state query.

### Key-value store

Each `Peer` instance provides a synchronized key-value store that allows
sharing peer-specific state between reactors:


    func (p Peer) Get(key string) interface{}
    func (p Peer) Set(key string, data interface{})

This key-value store can be seen as an asynchronous mechanism to exchange the
state of a peer between reactors.
In the current use-case of this mechanism, the Consensus reactor populates the
key-value store with a `PeerState` instance for each connected peer.
The Consensus reactor routines interacting with a peer read and update the
shared peer state.
The Evidence and Mempool reactors, in their turn, periodically query the
key-value store of each peer for retrieving, in particular, the last height
reported by the peer.
This information, produced by the Consensus reactor, influences the interaction
of these two reactors with their peers.

> **Note**
>
> More details of how this key-value store is used to share state between reactors can be found on the
> [knowledge-base repository](https://github.com/cometbft/knowledge-base/blob/main/p2p/reactors/peer-kvstore.md).

### Send methods

Finally, a `Peer` instance allows a reactor to send messages to companion
reactors running at that peer.
This is ultimately the goal of the switch when it provides `Peer` instances to
the registered reactors.
There are two methods for sending messages:

    func (p Peer) Send(e Envelope) bool
    func (p Peer) TrySend(e Envelope) bool

The two message-sending methods receive an `Envelope`, whose content should be
set as follows:

- `ChannelID`: the channel the message should be sent through, which defines
  the reactor that will process the message;
- `Src`: this field represents the source of an incoming message, which is
  irrelevant for outgoing messages;
- `Message`: the actual message's payload, which is marshalled using protocol buffers.

The two message-sending methods attempt to add the message (`e.Payload`) to the
send queue of the peer's destination channel (`e.ChannelID`).
There is a send queue for each registered channel supported by the peer, and
each send queue has a capacity.
The capacity of the send queues for each channel are [configured][reactor-channels]
by reactors via the corresponding `ChannelDescriptor`.

The two message-sending methods return whether it was possible to enqueue
the marshalled message to the channel's send queue.
The most common reason for these methods to return `false` is the channel's
send queue being full.
Further reasons for returning `false` are: the peer being stopped, providing a
non-registered channel ID, or errors when marshalling the message's payload.

The difference between the two message-sending methods is _when_ they return `false`.
The `Send()` method is a _blocking_ method, it returns `false` if the message
could not be enqueued, because the channel's send queue is still full, after a
10-second _timeout_.
The `TrySend()` method is a _non-blocking_ method, it _immediately_ returns
`false` when the channel's send queue is full.

[peer-interface]: ../../../p2p/peer.go
[service-interface]: ../../../libs/service/service.go
[switch-type]: ../../../p2p/switch.go

[reactor-interface]: ../../../p2p/base_reactor.go
[reactor-registration]: ./reactor.md#registration
[reactor-channels]: ./reactor.md#registration
[reactor-addpeer]: ./reactor.md#peer-management
[reactor-removepeer]: ./reactor.md#stop-peer
