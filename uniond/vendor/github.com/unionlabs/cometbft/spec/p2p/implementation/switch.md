# Switch

The switch is a core component of the p2p layer.
It manages the procedures for [dialing peers](#dialing-peers) and
[accepting](#accepting-peers) connections from peers, which are actually
implemented by the [transport](./transport.md).
It also manages the reactors, i.e., protocols implemented by the node that
interact with its peers.
Once a connection with a peer is established, the peer is [added](#add-peer) to
the switch and all registered reactors.
Reactors may also instruct the switch to [stop a peer](#stop-peer), namely
disconnect from it.
The switch, in this case, makes sure that the peer is removed from all
registered reactors.

## Dialing peers

Dialing a peer is implemented by the `DialPeerWithAddress` method.

This method is invoked by the [peer manager](./peer_manager.md#ensure-peers)
to dial a peer address and establish a connection with an outbound peer.

The switch keeps a single dialing routine per peer ID.
This is ensured by keeping a synchronized map `dialing` with the IDs of peers
to which the peer is dialing.
A peer ID is added to `dialing` when the `DialPeerWithAddress` method is called
for that peer, and it is removed when the method returns for whatever reason.
The method returns immediately when invoked for a peer which ID is already in
the `dialing` structure.

The actual dialing is implemented by the [`Dial`](./transport.md#dial) method
of the transport configured for the switch, in the `addOutboundPeerWithConfig`
method.
If the transport succeeds establishing a connection, the returned `Peer` is
added to the switch using the [`addPeer`](#add-peer) method.
This operation can fail, returning an error. In this case, the switch invokes
the transport's [`Cleanup`](./transport.md#cleanup) method to clean any resources
associated with the peer.

If the transport fails to establish a connection with the peer that is configured
as a persistent peer, the switch spawns a routine to [reconnect to the peer](#reconnect-to-peer).
If the peer is already in the `reconnecting` state, the spawned routine has no
effect and returns immediately.
This is in fact a likely scenario, as the `reconnectToPeer` routine relies on
this same `DialPeerWithAddress` method for dialing peers.

### Manual operation

The `DialPeersAsync` method receives a list of peer addresses (strings)
and dials all of them in parallel.
It is invoked in two situations:

- In the [setup](https://github.com/cometbft/cometbft/blob/v0.34.x/node/node.go#L987)
of a node, to establish connections with every configured persistent peer
- In the RPC package, to implement two unsafe RPC commands, not used in production:
  [`DialSeeds`](https://github.com/cometbft/cometbft/blob/v0.34.x/rpc/core/net.go#L47) and
  [`DialPeers`](https://github.com/cometbft/cometbft/blob/v0.34.x/rpc/core/net.go#L87)

The received list of peer addresses to dial is parsed into `NetAddress` instances.
In case of parsing errors, the method returns. An exception is made for
DNS resolution `ErrNetAddressLookup` errors, which do not interrupt the procedure.

As the peer addresses provided to this method are typically not known by the node,
contrarily to the addressed dialed using the `DialPeerWithAddress` method,
they are added to the node's address book, which is persisted to disk.

The switch dials the provided peers in parallel.
The list of peer addresses is randomly shuffled, and for each peer a routine is
spawned.
Each routine sleeps for a random interval, up to 3 seconds, then invokes the
`DialPeerWithAddress` method that actually dials the peer.

### Reconnect to peer

The `reconnectToPeer` method is invoked when a connection attempt to a peer fails,
and the peer is configured as a persistent peer.

The `reconnecting` synchronized map keeps the peer's in this state, identified
by their IDs (string).
This should ensure that a single instance of this method is running at any time.
The peer is kept in this map while this method is running for it: it is set on
the beginning, and removed when the method returns for whatever reason.
If the peer is already in the `reconnecting` state, nothing is done.

The remaining of the method performs multiple connection attempts to the peer,
via `DialPeerWithAddress` method.
If a connection attempt succeeds, the methods returns and the routine finishes.
The same applies when an `ErrCurrentlyDialingOrExistingAddress` error is
returned by the dialing method, as it indicates that peer is already connected
or that another routine is attempting to (re)connect to it.

A first set of connection attempts is done at (about) regular intervals.
More precisely, between two attempts, the switch waits for a interval of
`reconnectInterval`, hard-coded to 5 seconds, plus a random jitter up to
`dialRandomizerIntervalMilliseconds`, hard-coded to 3 seconds.
At most `reconnectAttempts`, hard-coded to 20, are made using this
regular-interval approach.

A second set of connection attempts is done with exponentially increasing
intervals.
The base interval `reconnectBackOffBaseSeconds` is hard-coded to 3 seconds,
which is also the increasing factor.
The exponentially increasing dialing interval is adjusted as well by a random
jitter up to `dialRandomizerIntervalMilliseconds`.
At most `reconnectBackOffAttempts`, hard-coded to 10, are made using this  approach.

> Note: the first sleep interval, to which a random jitter is applied, is 1,
> not `reconnectBackOffBaseSeconds`, as the first exponent is `0`...

## Accepting peers

The `acceptRoutine` method is a persistent routine that handles connections
accepted by the transport configured for the switch.

The [`Accept`](./transport.md#accept) method of the configured transport
returns a `Peer` with which an inbound connection was established.
The switch accepts a new peer if the maximum number of inbound peers was not
reached, or if the peer was configured as an _unconditional peer_.
The maximum number of inbound peers is determined by the `MaxNumInboundPeers`
configuration parameter, whose default value is `40`.

If accepted, the peer is added to the switch using the [`addPeer`](#add-peer) method.
If the switch does not accept the established incoming connection, or if the
`addPeer` method returns an error, the switch invokes the transport's
[`Cleanup`](./transport.md#cleanup) method to clean any resources associated
with the peer.

The transport's `Accept` method can also return a number of errors.
Errors of `ErrRejected` or `ErrFilterTimeout` types are ignored,
an `ErrTransportClosed` causes the accepted routine to be interrupted,
while other errors cause the routine to panic.

> TODO: which errors can cause the routine to panic?

## Add peer

The `addPeer` method adds a peer to the switch,
either after dialing (by `addOutboundPeerWithConfig`, called by `DialPeerWithAddress`)
a peer and establishing an outbound connection,
or after accepting (`acceptRoutine`) a peer and establishing an inbound connection.

The first step is to invoke the `filterPeer` method.
It checks whether the peer is already in the set of connected peers,
and whether any of the configured `peerFilter` methods reject the peer.
If the peer is already present or it is rejected by any filter, the `addPeer`
method fails and returns an error.

Then, the new peer is started, added to the set of connected peers, and added
to all reactors.
More precisely, first the new peer's information is first provided to every
reactor (`InitPeer` method).
Next, the peer's sending and receiving routines are started, and the peer is
added to set of connected peers.
These two operations can fail, causing `addPeer` to return an error.
Then, in the absence of previous errors, the peer is added to every reactor (`AddPeer` method).

> Adding the peer to the peer set returns a `ErrSwitchDuplicatePeerID` error
> when a peer with the same ID is already presented.
>
> TODO: Starting a peer could be reduced as starting the MConn with that peer?

## Stop peer

There are two methods for stopping a peer, namely disconnecting from it, and
removing it from the table of connected peers.

The `StopPeerForError` method is invoked to stop a peer due to an external
error, which is provided to method as a generic "reason".

The `StopPeerGracefully` method stops a peer in the absence of errors or, more
precisely, not providing to the switch any "reason" for that.

In both cases the `Peer` instance is stopped, the peer is removed from all
registered reactors, and finally from the list of connected peers.

> Issue <https://github.com/tendermint/tendermint/issues/3338> is mentioned in
> the internal `stopAndRemovePeer` method explaining why removing the peer from
> the list of connected peers is the last action taken.

When there is a "reason" for stopping the peer (`StopPeerForError` method)
and the peer is a persistent peer, the method creates a routine to attempt
reconnecting to the peer address, using the `reconnectToPeer` method.
If the peer is an outbound peer, the peer's address is know, since the switch
has dialed the peer.
Otherwise, the peer address is retrieved from the `NodeInfo` instance from the
connection handshake.

## Add reactor

The `AddReactor` method registers a `Reactor` to the switch.

The reactor is associated to the set of channel ids it employs.
Two reactors (in the same node) cannot share the same channel id.

There is a call back to the reactor, in which the switch passes itself to the
reactor.

## Remove reactor

The `RemoveReactor` method unregisters a `Reactor` from the switch.

The reactor is disassociated from the set of channel ids it employs.

There is a call back to the reactor, in which the switch passes `nil` to the
reactor.

## OnStart

This is a `BaseService` method.

All registered reactors are started.

The switch's `acceptRoutine` is started.

## OnStop

This is a `BaseService` method.

All (connected) peers are stopped and removed from the peer's list using the
`stopAndRemovePeer` method.

All registered reactors are stopped.

## Broadcast

This method broadcasts a message on a channel, by sending the message in
parallel to all connected peers.

The method spawns a thread for each connected peer, invoking the `Send` method
provided by each `Peer` instance with the provided message and channel ID.
The return value (a boolean) of these calls are redirected to a channel that is
returned by the method.

> TODO: detail where this method is invoked:
>
> - By the consensus protocol, in `broadcastNewRoundStepMessage`,
>   `broadcastNewValidBlockMessage`, and `broadcastHasVoteMessage`
> - By the state sync protocol
