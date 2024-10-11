# PEX Reactor

The PEX reactor is one of the reactors running in a CometBFT node.

Its implementation is located in the `p2p/pex` package, and it is considered
part of the implementation of the p2p layer.

This document overviews the implementation of the PEX reactor, describing how
the methods from the `Reactor` interface are implemented.

The actual operation of the PEX reactor is presented in documents describing
the roles played by the PEX reactor in the p2p layer:

- [Address Book](./addressbook.md): stores known peer addresses and information
  about peers to which the node is connected or has attempted to connect
- [Peer Manager](./peer_manager.md): manages connections established with peers,
  defining when a node should dial peers and which peers it should dial
- [Peer Exchange protocol](./pex-protocol.md): enables nodes to exchange peer
  addresses, thus implementing a peer discovery service

## OnStart

The `OnStart` method implements `BaseService` and starts the PEX reactor.

The [address book](./addressbook.md), which is a `Service` is started.
This loads the address book content from disk,
and starts a routine that periodically persists the address book content to disk.

The PEX reactor is configured with the addresses of a number of seed nodes,
the `Seeds` parameter of the `ReactorConfig`.
The addresses of seed nodes are parsed into `NetAddress` instances and resolved
into IP addresses, which is implemented by the `checkSeeds` method.
Valid seed node addresses are stored in the `seedAddrs` field,
and are used by the `dialSeeds` method to contact the configured seed nodes.

The last action is to start one of the following persistent routines, based on
the `SeedMode` configuration parameter:

- Regular nodes run the `ensurePeersRoutine` to check whether the node has
  enough outbound peers, dialing peers when necessary
- Seed nodes run the `crawlPeersRoutine` to periodically start a new round
  of [crawling](./pex-protocol.md#Crawling-peers) to discover as many peer
  addresses as possible

### Errors

Errors encountered when loading the address book from disk are returned,
and prevent the reactor from being started.
An exception is made for the `service.ErrAlreadyStarted` error, which is ignored.

Errors encountered when parsing the configured addresses of seed nodes
are returned and cause the reactor startup to fail.
An exception is made for DNS resolution `ErrNetAddressLookup` errors,
which are not deemed fatal and are only logged as invalid addresses.

If none of the configured seed node addresses is valid, and the loaded address
book is empty, the reactor is not started and an error is returned.

## OnStop

The `OnStop` method implements `BaseService` and stops the PEX reactor.

The address book routine that periodically saves its content to disk is stopped.

## GetChannels

The `GetChannels` method, from the `Reactor` interface, returns the descriptor
of the channel used by the PEX protocol.

The channel ID is `PexChannel` (0), with priority `1`, send queue capacity of
`10`, and maximum message size of `64000` bytes.

## AddPeer

The `AddPeer` method, from the `Reactor` interface,
adds a new peer to the PEX protocol.

If the new peer is an **inbound peer**, i.e., if the peer has dialed the node,
the peer's address is [added to the address book](./addressbook.md#adding-addresses).
Since the peer was authenticated when establishing a secret connection with it,
the source of the peer address is trusted, and its source is set by the peer itself.
In the case of an outbound peer, the node should already have its address in
the address book, as the switch has dialed the peer.

If the peer is an **outbound peer**, i.e., if the node has dialed the peer,
and the PEX protocol needs more addresses,
the node [sends a PEX request](./pex-protocol.md#Requesting-Addresses) to the peer.
The same is not done when inbound peers are added because they are deemed least
trustworthy than outbound peers.

## RemovePeer

The `RemovePeer` method, from the `Reactor` interface,
removes a peer from the PEX protocol.

The peer's ID is removed from the tables tracking PEX requests
[sent](./pex-protocol.md#misbehavior) but not yet replied
and PEX requests [received](./pex-protocol.md#misbehavior-1).

## Receive

The `Receive` method, from the `Reactor` interface,
handles a message received by the PEX protocol.

A node receives two type of messages as part of the PEX protocol:

- `PexRequest`: a request for addresses received from a peer, handled as
  described [here](./pex-protocol.md#providing-addresses)
- `PexAddrs`: a list of addresses received from a peer, as a response to a PEX
  request sent by the node, as described [here](./pex-protocol.md#responses)

