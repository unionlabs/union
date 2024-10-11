# Transport

The transport establishes secure and authenticated connections with peers.

The transport [`Dial`](#dial)s peer addresses to establish outbound connections,
and [`Listen`](#listen)s in a configured network address
to [`Accept`](#accept) inbound connections from peers.

The transport establishes raw TCP connections with peers
and [upgrade](#connection-upgrade) them into authenticated secret connections.
The established secret connection is then wrapped into `Peer` instance, which
is returned to the caller, typically the [switch](./switch.md).

## Dial

The `Dial` method is used by the switch to establish an outbound connection with a peer.
It is a synchronous method, which blocks until a connection is established or an error occurs.
The method returns an outbound `Peer` instance wrapping the established connection.

The transport first dials the provided peer's address to establish a raw TCP connection.
The dialing maximum duration is determined by `dialTimeout`, hard-coded to 1 second.
The established raw connection is then submitted to a set of [filters](#connection-filtering),
which can reject it.
If the connection is not rejected, it is recorded in the table of established connections.

The established raw TCP connection is then [upgraded](#connection-upgrade) into
an authenticated secret connection.
This procedure should ensure, in particular, that the public key of the remote peer
matches the ID of the dialed peer, which is part of peer address provided to this method.
In the absence of errors,
the established secret connection (`conn.SecretConnection` type)
and the information about the peer (`NodeInfo` record) retrieved and verified
during the version handshake,
are wrapped into an outbound `Peer` instance and returned to the switch.

## Listen

The `Listen` method produces a TCP listener instance for the provided network
address, and spawns an `acceptPeers` routine to handle the raw connections
accepted by the listener.
The `NetAddress` method exports the listen address configured for the transport.

The maximum number of simultaneous incoming connections accepted by the listener
is bound to `MaxNumInboundPeer` plus the configured number of unconditional peers,
using the `MultiplexTransportMaxIncomingConnections` option,
in the node [initialization](https://github.com/cometbft/cometbft/blob/v0.34.x/node/node.go#L563).

This method is called when a node is [started](https://github.com/cometbft/cometbft/blob/v0.34.x/node/node.go#L974).
In case of errors, the `acceptPeers` routine is not started and the error is returned.

## Accept

The `Accept` method returns to the switch inbound connections established with a peer.
It is a synchronous method, which blocks until a connection is accepted or an error occurs.
The method returns an inbound `Peer` instance wrapping the established connection.

The transport handles incoming connections in the `acceptPeers` persistent routine.
This routine is started by the [`Listen`](#listen) method
and accepts raw connections from a TCP listener.
A new routine is spawned for each accepted connection.
The raw connection is submitted to a set of [filters](#connection-filtering),
which can reject it.
If the connection is not rejected, it is recorded in the table of established connections.

The established raw TCP connection is then [upgraded](#connection-upgrade) into
an authenticated secret connection.
The established secret connection (`conn.SecretConnection` type),
the information about the peer (`NodeInfo` record) retrieved and verified
during the version handshake,
as well any error returned in this process are added to a queue of accepted connections.
This queue is consumed by the `Accept` method.

> Handling accepted connection asynchronously was introduced due to this issue:
> <https://github.com/tendermint/tendermint/issues/2047>

## Connection Filtering

The `filterConn` method is invoked for every new raw connection established by the transport.
Its main goal is avoid the transport to maintain duplicated connections with the same peer.
It also runs a set of configured connection filters.

The transports keeps a table `conns` of established connections.
The table maps the remote address returned by a generic connection to a list of
IP addresses, to which the connection remote address is resolved.
If the remote address of the new connection is already present in the table,
the connection is rejected.
Otherwise, the connection's remote address is resolved into a list of IPs,
which are recorded in the established connections table.

The connection and the resolved IPs are then passed through a set of connection filters,
configured via the `MultiplexTransportConnFilters` transport option.
The maximum duration for the filters execution, which is performed in parallel,
is determined by `filterTimeout`.
Its default value is 5 seconds,
which can be changed using the `MultiplexTransportFilterTimeout` transport option.

If the connection and the resolved remote addresses are not filtered out,
the transport registers them into the `conns` table and returns.

In case of errors, the connection is removed from the table of established
connections and closed.

### Errors

If the address of the new connection is already present in the `conns` table,
an `ErrRejected` error with the `isDuplicate` reason is returned.

If the IP resolution of the connection's remote address fails,
an `AddrError` or `DNSError` error is returned.

If any of the filters reject the connection,
an `ErrRejected` error with the `isRejected` reason is returned.

If the filters execution times out,
an `ErrFilterTimeout` error is returned.

## Connection Upgrade

The `upgrade` method is invoked for every new raw connection established by the
transport that was not [filtered out](#connection-filtering).
It upgrades an established raw TCP connection into a secret authenticated
connection, and validates the information provided by the peer.

This is a complex procedure, that can be summarized by the following three
message exchanges between the node and the new peer:

1. Encryption: the nodes produce ephemeral key pairs and exchange ephemeral
   public keys, from which are derived: (i) a pair of secret keys used to
   encrypt the data exchanged between the nodes, and (ii) a challenge message.
1. Authentication: the nodes exchange their persistent public keys and a
   signature of the challenge message produced with the their persistent
   private keys. This allows validating the peer's persistent public key,
   which plays the role of node ID.
1. Version handshake: nodes exchange and validate each other `NodeInfo` records.
   This records contain, among other fields, their node IDs, the network/chain
   ID they are part of, and the list of supported channel IDs.

Steps (1) and (2) are implemented in the `conn` package.
In case of success, they produce the secret connection that is actually used by
the node to communicate with the peer.
An overview of this procedure, which implements the station-to-station (STS)
[protocol][sts-paper] ([PDF][sts-paper-pdf]), can be found [here][peer-sts].
The maximum duration for establishing a secret connection with the peer is
defined by `handshakeTimeout`, hard-coded to 3 seconds.

The established secret connection stores the persistent public key of the peer,
which has been validated via the challenge authentication of step (2).
If the connection being upgraded is an outbound connection, i.e., if the node has
dialed the peer, the dialed peer's ID is compared to the peer's persistent public key:
if they do not match, the connection is rejected.
This verification is not performed in the case of inbound (accepted) connections,
as the node does not know a priori the remote node's ID.

Step (3), the version handshake, is performed by the transport.
Its maximum duration is also defined by `handshakeTimeout`, hard-coded to 3 seconds.
The version handshake retrieves the `NodeInfo` record of the new peer,
which can be rejected for multiple reasons, listed [here][peer-handshake].

If the connection upgrade succeeds, the method returns the established secret
connection, an instance of `conn.SecretConnection` type,
and the `NodeInfo` record of the peer.

In case of errors, the connection is removed from the table of established
connections and closed.

### Errors

The timeouts for steps (1) and (2), and for step (3), are configured as the
deadline for operations on the TCP connection that is being upgraded.
If this deadline it is reached, the connection produces an
`os.ErrDeadlineExceeded` error, returned by the corresponding step.

Any error produced when establishing a secret connection with the peer (steps 1 and 2) or
during the version handshake (step 3), including timeouts,
is encapsulated into an `ErrRejected` error with reason `isAuthFailure` and returned.

If the upgraded connection is an outbound connection, and the peer ID learned in step (2)
does not match the dialed peer's ID,
an `ErrRejected` error with reason `isAuthFailure` is returned.

If the peer's `NodeInfo` record, retrieved in step (3), is invalid,
or if reports a node ID that does not match peer ID learned in step (2),
an `ErrRejected` error with reason `isAuthFailure` is returned.
If it reports a node ID equals to the local node ID,
an `ErrRejected` error with reason `isSelf` is returned.
If it is not compatible with the local `NodeInfo`,
an `ErrRejected` error with reason `isIncompatible` is returned.

## Close

The `Close` method closes the TCP listener created by the `Listen` method,
and sends a signal for interrupting the `acceptPeers` routine.

This method is called when a node is [stopped](https://github.com/cometbft/cometbft/blob/v0.34.x/node/node.go#L1023).

## Cleanup

The `Cleanup` method receives a `Peer` instance,
and removes the connection established with a peer from the table of established connections.
It also invokes the `Peer` interface method to close the connection associated with a peer.

It is invoked when the connection with a peer is closed.

## Supported channels

The `AddChannel` method registers a channel in the transport.

The channel ID is added to the list of supported channel IDs,
stored in the local `NodeInfo` record.

The `NodeInfo` record is exchanged with peers in the version handshake.
For this reason, this method is not invoked with a started transport.

> The only call to this method is performed in the `CustomReactors` constructor
> option of a node, i.e., before the node is started.
> Note that the default list of supported channel IDs, including the default reactors,
> is provided to the transport as its original `NodeInfo` record.

[peer-sts]: ../legacy-docs/peer.md#authenticated-encryption-handshake
[peer-handshake]: ../legacy-docs/peer.md#cometbft-version-handshake
[sts-paper]: https://link.springer.com/article/10.1007/BF00124891
[sts-paper-pdf]: https://github.com/tendermint/tendermint/blob/0.1/docs/sts-final.pdf
