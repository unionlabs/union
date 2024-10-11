# Types adopted in the p2p implementation

This document lists the packages and source files, excluding test units, that
implement the p2p layer, and summarizes the main types they implement.
Types play the role of classes in Go.

The reference version for this documentation is the branch
[`v0.34.x`](https://github.com/cometbft/cometbft/tree/v0.34.x/p2p).

State of August 2022.

## Package `p2p`

Implementation of the p2p layer of CometBFT.

### `base_reactor.go`

`Reactor` interface.

`BaseReactor` implements `Reactor`.

**Not documented yet**.

### `conn_set.go`

`ConnSet` interface, a "lookup table for connections and their ips".

Internal type `connSet` implements the `ConnSet` interface.

Used by the [transport](#transportgo) to store connected peers.

### `errors.go`

Defines several error types.

`ErrRejected` enumerates a number of reason for which a peer was rejected.
Mainly produced by the [transport](#transportgo),
but also by the [switch](#switchgo).

`ErrSwitchDuplicatePeerID` is produced by the `PeerSet` used by the [switch](#switchgo).

`ErrSwitchConnectToSelf` is handled by the [switch](#switchgo),
but currently is not produced outside tests.

`ErrSwitchAuthenticationFailure` is handled by the [PEX reactor](#pex_reactorgo),
but currently is not produced outside tests.

`ErrTransportClosed` is produced by the [transport](#transportgo)
and handled by the [switch](#switchgo).

`ErrNetAddressNoID`, `ErrNetAddressInvalid`, and `ErrNetAddressLookup`
are parsing a string to create an instance of `NetAddress`.
It can be returned in the setup of the [switch](#switchgo)
and of the [PEX reactor](#pex_reactorgo),
as well when the [transport](#transportgo) validates a `NodeInfo`, as part of
the connection handshake.

`ErrCurrentlyDialingOrExistingAddress` is produced by the [switch](#switchgo),
and handled by the switch and the [PEX reactor](#pex_reactorgo).

### `fuzz.go`

For testing purposes.

`FuzzedConnection` wraps a `net.Conn` and injects random delays.

### `key.go`

`NodeKey` is the persistent key of a node, namely its private key.

The `ID` of a node is a string representing the node's public key.

### `metrics.go`

Prometheus `Metrics` exposed by the p2p layer.

### `netaddress.go`

Type `NetAddress` contains the `ID` and the network address (IP and port) of a node.

The API of the [address book](#addrbookgo) receives and returns `NetAddress` instances.

This source file was adapted from [`btcd`](https://github.com/btcsuite/btcd),
a Go implementation of Bitcoin.

### `node_info.go`

Interface `NodeInfo` stores the basic information about a node exchanged with a
peer during the handshake.

It is implemented by `DefaultNodeInfo` type.

The [switch](#switchgo) stores the local `NodeInfo`.

The `NodeInfo` of connected peers is produced by the
[transport](#transportgo) during the handshake, and stored in [`Peer`](#peergo) instances.

### `peer.go`

Interface `Peer` represents a connected peer.

It is implemented by the internal `peer` type.

The [transport](#transportgo) API methods return `Peer` instances,
wrapping established secure connection with peers.

The [switch](#switchgo) API methods receive `Peer` instances.
The switch stores connected peers in a `PeerSet`.

The [`Reactor`](#base_reactorgo) methods, invoked by the switch, receive `Peer` instances.

### `peer_set.go`

Interface `IPeerSet` offers methods to access a table of [`Peer`](#peergo) instances.

Type `PeerSet` implements a thread-safe table of [`Peer`](#peergo) instances,
used by the [switch](#switchgo).

The switch provides limited access to this table by returning a `IPeerSet`
instance, used by the [PEX reactor](#pex_reactorgo).

### `switch.go`

Documented in [switch](./switch.md).

The `Switch` implements the [peer manager](./peer_manager.md) role for inbound peers.

[`Reactor`](#base_reactorgo)s have access to the `Switch` and may invoke its methods.
This includes the [PEX reactor](#pex_reactorgo).

### `transport.go`

Documented in [transport](./transport.md).

The `Transport` interface is implemented by `MultiplexTransport`.

The [switch](#switchgo) contains a `Transport` and uses it to establish
connections with peers.

### `types.go`

Aliases for p2p's `conn` package types.

## Package `p2p.conn`

Implements the connection between CometBFT nodes,
which is encrypted, authenticated, and multiplexed.

### `connection.go`

Implements the `MConnection` type and the `Channel` abstraction.

A `MConnection` multiplexes a generic network connection (`net.Conn`) into
multiple independent `Channel`s, used by different [`Reactor`](#base_reactorgo)s.

A [`Peer`](#peergo) stores the `MConnection` instance used to interact with a
peer, which multiplex a [`SecretConnection`](#secret_connectiongo).

### `conn_go110.go`

Support for go 1.10.

### `secret_connection.go`

Implements the `SecretConnection` type, which is an encrypted authenticated
connection built atop a raw network (TCP) connection.

A [`Peer`](#peergo) stores the `SecretConnection` established by the transport,
which is the underlying connection multiplexed by [`MConnection`](#connectiongo).

As briefly documented in the [transport](./transport.md#Connection-Upgrade),
a `SecretConnection` implements the Station-To-Station (STS) protocol.

The `SecretConnection` type implements the `net.Conn` interface,
which is a generic network connection.

## Package `p2p.mock`

Mock implementations of [`Peer`](#peergo) and [`Reactor`](#base_reactorgo) interfaces.

## Package `p2p.mocks`

Code generated by `mockery`.

## Package `p2p.pex`

Implementation of the [PEX reactor](./pex.md).

### `addrbook.go`

Documented in [address book](./addressbook.md).

This source file was adapted from [`btcd`](https://github.com/btcsuite/btcd),
a Go implementation of Bitcoin.

### `errors.go`

A number of errors produced and handled by the [address book](#addrbookgo).

`ErrAddrBookNilAddr` is produced by the address book, but handled (logged) by
the [PEX reactor](#pex_reactorgo).

`ErrUnsolicitedList` is produced and handled by the [PEX protocol](#pex_reactorgo).

### `file.go`

Implements the [address book](#addrbookgo) persistence.

### `known_address.go`

Type `knownAddress` represents an address stored in the [address book](#addrbookgo).

### `params.go`

Constants used by the [address book](#addrbookgo).

### `pex_reactor.go`

Implementation of the [PEX reactor](./pex.md), which is a [`Reactor`](#base_reactorgo).

This includes the implementation of the [PEX protocol](./pex-protocol.md)
and of the [peer manager](./peer_manager.md) role for outbound peers.

The PEX reactor also manages an [address book](#addrbookgo) instance.

## Package `p2p.trust`

Go documentation of `Metric` type:

> // Metric - keeps track of peer reliability
> // See cometbft/docs/architecture/adr-006-trust-metric.md for details

Not imported by any other CometBFT source file.
