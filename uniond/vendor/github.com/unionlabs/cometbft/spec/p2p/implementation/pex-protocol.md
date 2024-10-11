# Peer Exchange Protocol

The Peer Exchange (PEX) protocol enables nodes to exchange peer addresses, thus
implementing a peer discovery mechanism.

The PEX protocol uses two messages:

- `PexRequest`: sent by a node to [request](#requesting-addresses) peer
  addresses to a peer
- `PexAddrs`: a list of peer addresses [provided](#providing-addresses) to a
  peer as response to a `PexRequest` message

While all nodes, with few exceptions, participate on the PEX protocol,
a subset of nodes, configured as [seed nodes](#seed-nodes) have a particular
role in the protocol.
They crawl the network, connecting to random peers, in order to learn as many
peer addresses as possible to provide to other nodes.

## Requesting Addresses

A node requests peer addresses by sending a `PexRequest` message to a peer.

For regular nodes, not operating in seed mode, a PEX request is sent when
the node *needs* peers addresses, a condition checked:

1. When an *outbound* peer is added, causing the node to request addresses from
   the new peer
2. Periodically, by the `ensurePeersRoutine`, causing the node to request peer
   addresses to a randomly selected peer

A node needs more peer addresses when its addresses book has
[less than 1000 records](./addressbook.md#need-for-addresses).
It is thus reasonable to assume that the common case is that a peer needs more
peer addresses, so that PEX requests are sent whenever the above two situations happen.

A PEX request is sent when a new *outbound* peer is added.
The same does not happen with new inbound peers because the implementation
considers outbound peers, that the node has chosen for dialing, more
trustworthy than inbound peers, that the node has accepted.
Moreover, when a node is short of peer addresses, it dials the configured seed nodes;
since they are added as outbound peers, the node can immediately request peer addresses.

The `ensurePeersRoutine` periodically checks, by default every 30 seconds (`ensurePeersPeriod`),
whether the node has enough outbound peers.
If it does not have, the node tries dialing some peer addresses stored in the address book.
As part of this procedure, the node selects a peer at random,
from the set of connected peers retrieved from the switch,
and sends a PEX request to the selected peer.

Sending a PEX request to a peer is implemented by the `RequestAddrs` method of
the PEX reactor.

### Responses

After a PEX request is sent to a peer, the node expects to receive,
as a response, a `PexAddrs` message from the peer.
This message encodes a list of peer addresses that are
[added to address book](./addressbook.md#adding-addresses),
having the peer from which the PEX response was received as their source.

Received PEX responses are handled by the `ReceiveAddrs` method of the PEX reactor.
In the case of a PEX response received from a peer which is configured as
a seed node, the PEX reactor attempts immediately to dial the provided peer
addresses, as detailed [here](./peer_manager.md#fast-dialing).

### Misbehavior

Sending multiple PEX requests to a peer, before receiving a reply from it,
is considered a misbehavior.
To prevent it, the node maintains a `requestsSent` set of outstanding
requests, indexed by destination peers.
While a peer ID is present in the `requestsSent` set, the node does not send
further PEX requests to that peer.
A peer ID is removed from the `requestsSent` set when a PEX response is
received from it.

Sending a PEX response to a peer that has not requested peer addresses
is also considered a misbehavior.
So, if a PEX response is received from a peer that is not registered in
the `requestsSent` set, a `ErrUnsolicitedList` error is produced.
This leads the peer to be disconnected and [marked as a bad peer](./addressbook.md#bad-peers).

## Providing Addresses

When a node receives a `PexRequest` message from a peer,
it replies with a `PexAddrs` message.

This message encodes a [random selection of peer addresses](./addressbook.md#random-selection)
retrieved from the address book.

Sending a PEX response to a peer is implemented by the `SendAddrs` method of
the PEX reactor.

### Misbehavior

Requesting peer addresses too often is considered a misbehavior.
Since node are expected to send PEX requests every `ensurePeersPeriod`,
the minimum accepted interval between requests from the same peer is set
to `ensurePeersPeriod / 3`, 10 seconds by default.

The `receiveRequest` method is responsible for verifying this condition.
The node keeps a `lastReceivedRequests` map with the time of the last PEX
request received from every peer.
If the interval between successive requests is less than the minimum accepted
one, the peer is disconnected and [marked as a bad peer](./addressbook.md#bad-peers).
An exception is made for the first two PEX requests received from a peer.

> The probably reason is that, when a new peer is added, the two conditions for
> a node to request peer addresses can be triggered with an interval lower than
> the minimum accepted interval.
> Since this is a legit behavior, it should not be punished.

## Seed nodes

A seed node is a node configured to operate in `SeedMode`.

### Crawling peers

Seed nodes crawl the network, connecting to random peers and sending PEX
requests to them, in order to learn as many peer addresses as possible.
More specifically, a node operating in seed mode sends PEX requests in two cases:

1. When an outbound peer is added, and the seed node needs more peer addresses,
   it requests peer addresses to the new peer
2. Periodically, the `crawlPeersRoutine` sends PEX requests to a random set of
   peers, whose addresses are registered in the Address Book

The first case also applies for nodes not operating in seed mode.
The second case replaces the second for regular nodes, as seed nodes do not
run the `ensurePeersRoutine`, as regular nodes,
but run the `crawlPeersRoutine`, which is not run by regular nodes.

The `crawlPeersRoutine` periodically, every 30 seconds (`crawlPeerPeriod`),
starts a new peer discovery round.
First, the seed node retrieves a random selection of peer addresses from its
Address Book.
This selection is produced in the same way as in the random selection of peer
addresses that are [provided](#providing-addresses) to a requesting peer.
Peers that the seed node has crawled recently,
less than 2 minutes ago (`minTimeBetweenCrawls`), are removed from this selection.
The remaining peer addresses are registered in the `crawlPeerInfos` table.

The seed node is not necessarily connected to the peer whose address is
selected for each round of crawling.
So, the seed node dials the selected peer addresses.
This is performed in foreground, one peer at a time.
As a result, a round of crawling can take a substantial amount of time.
For each selected peer it succeeds dialing to, this include already connected
peers, the seed node sends a PEX request.

Dialing a selected peer address can fail for multiple reasons.
The seed node might have attempted to dial the peer too many times.
In this case, the peer address is marked as [bad in the address book](./addressbook.md#bad-peers).
The seed node might have attempted to dial the peer recently, without success,
and the exponential `backoffDuration` has not yet passed.
Or the current connection attempt might fail, which is registered in the address book.

Failures to dial to a peer address produce an information that is important for
a seed node.
They indicate that a peer is unreachable, or is not operating correctly, and
therefore its address should not be provided to other nodes.
This occurs when, due to multiple failed connection attempts or authentication
failures, the peer address ends up being removed from the address book.
As a result, the periodically crawling of selected peers not only enables the
discovery of new peers, but also allows the seed node to stop providing
addresses of bad peers.

### Offering addresses

Nodes operating in seed mode handle PEX requests differently than regular
nodes, whose operation is described [here](#providing-addresses).

This distinction exists because nodes dial a seed node with the main, if not
exclusive goal of retrieving peer addresses.
In other words, nodes do not dial a seed node because they intend to have it as
a peer in the multiple CometBFT protocols, but because they believe that a
seed node is a good source of addresses of nodes to which they can establish
connections and interact in the multiple CometBFT protocols.

So, when a seed node receives a `PexRequest` message from an inbound peer,
it sends a `PexAddrs` message, containing a selection of peer
addresses, back to the peer and *disconnects* from it.
Seed nodes therefore treat inbound connections from peers as a short-term
connections, exclusively intended to retrieve peer addresses.
Once the requested peer addresses are sent, the connection with the peer is closed.

Moreover, the selection of peer addresses provided to inbound peers by a seed
node, although still essentially random, has a [bias toward old
addresses](./addressbook.md#random-selection-with-bias).
The selection bias is defined by `biasToSelectNewPeers`, hard-coded to `30%`,
meaning that `70%` of the peer addresses provided by a seed node are expected
to be old addresses.
Although this nomenclature is not clear, *old* addresses are the addresses that
survived the most in the address book, that is, are addresses that the seed
node believes being from *good* peers (more details [here](./addressbook.md#good-peers)).

Another distinction is on the handling of potential [misbehavior](#misbehavior-1)
of peers requesting addresses.
A seed node does not enforce, a priori, a minimal interval between PEX requests
from inbound peers.
Instead, it does not reply to more than one PEX request per peer inbound
connection, and, as above mentioned, it disconnects from incoming peers after
responding to them.
If the same peer dials again to the seed node and requests peer addresses, the
seed node will reply to this peer like it was the first time it has requested
peer addresses.

> This is more an implementation restriction than a desired behavior.
> The `lastReceivedRequests` map stores the last time a PEX request was
> received from a peer, and the entry relative to a peer is removed from this
> map when the peer is disconnected.
>
> It is debatable whether this approach indeed prevents abuse against seed nodes.

### Disconnecting from peers

Seed nodes treat connections with peers as short-term connections, which are
mainly, if not exclusively, intended to exchange peer addresses.

In the case of inbound peers, that have dialed the seed node, the intent of the
connection is achieved once a PEX response is sent to the peer.
The seed node thus disconnects from an inbound peer after sending a `PexAddrs`
message to it.

In the case of outbound peers, which the seed node has dialed for crawling peer
addresses, the intent of the connection is essentially achieved when a PEX
response is received from the peer.
The seed node, however, does not disconnect from a peer after receiving a
selection of peer addresses from it.
As a result, after some rounds of crawling, a seed node will have established
connections to a substantial amount of peers.

To couple with the existence of multiple connections with peers that have no
longer purpose for the seed node, the `crawlPeersRoutine` also invokes, after
each round of crawling, the `attemptDisconnects` method.
This method retrieves the list of connected peers from the switch, and
disconnects from peers that are not persistent peers, and with which a
connection is established for more than `SeedDisconnectWaitPeriod`.
This period is a configuration parameter, set to 28 hours when the PEX reactor
is created by the default node constructor.
