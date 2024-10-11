# Address Book

The address book tracks information about peers, i.e., about other nodes in the network.

The primary information stored in the address book are peer addresses.
A peer address is composed by a node ID and a network address; a network
address is composed by an IP address or a DNS name plus a port number.
The same node ID can be associated to multiple network addresses.

There are two sources for the addresses stored in the address book.
The [Peer Exchange protocol](./pex-protocol.md) stores in the address book
the peer addresses it discovers, i.e., it learns from connected peers.
And the [Switch](./switch.md) registers the addresses of peers with which it
has interacted: to which it has dialed or from which it has accepted a
connection.

The address book also records additional information about peers with which the
node has interacted, from which is possible to rank peers.
The Switch reports [connection attempts](#dial-attempts) to a peer address; too
much failed attempts indicate that a peer address is invalid.
Reactors, in they turn, report a peer as [good](#good-peers) when it behaves as
expected, or as a [bad peer](#bad-peers), when it misbehaves.

There are two entities that retrieve peer addresses from the address book.
The [Peer Manager](./peer_manager.md) retrieves peer addresses to dial, so to
establish outbound connections.
This selection is random, but has a configurable bias towards peers that have
been marked as good peers.
The [Peer Exchange protocol](./pex-protocol.md) retrieves random samples of
addresses to offer (send) to peers.
This selection is also random but it includes, in particular for nodes that
operate in seed mode, some bias toward peers marked as good ones.

## Buckets

Peer addresses are stored in buckets.
There are buckets for new addresses and buckets for old addresses.
The buckets for new addresses store addresses of peers about which the node
does not have much information; the first address registered for a peer ID is
always stored in a bucket for new addresses.
The buckets for old addresses store addresses of peers with which the node has
interacted and that were reported as [good peers](#good-peers) by a reactor.
An old address therefore can be seen as an alias for a good address.

> Note that new addresses does not mean bad addresses.
> The addresses of peers marked as [bad peers](#bad-peers) are removed from the
> buckets where they are stored, and temporarily kept in a table of banned peers.

The number of buckets is fixed and there are more buckets for new addresses
(`256`) than buckets for old addresses (`64`), a ratio of 4:1.
Each bucket can store up to `64` addresses.
When a bucket becomes full, the peer address with the lowest ranking is removed
from the bucket.
The first choice is to remove bad addresses, with multiple failed attempts
associated.
In the absence of those, the *oldest* address in the bucket is removed, i.e.,
the address with the oldest last attempt to dial.

When a bucket for old addresses becomes full, the lowest-ranked peer address in
the bucket is moved to a bucket of new addresses.
When a bucket for new addresses becomes full, the lowest-ranked peer address in
the bucket is removed from the address book.
In other words, exceeding old or good addresses are downgraded to new
addresses, while exceeding new addresses are dropped.

The bucket that stores an `address` is defined by the following two methods,
for new and old addresses:

- `calcNewBucket(address, source) = hash(key + groupKey(source) + hash(key + groupKey(address) + groupKey(source)) % newBucketsPerGroup) % newBucketCount`
- `calcOldBucket(address) = hash(key + groupKey(address) + hash(key + address) % oldBucketsPerGroup) % oldBucketCount`

The `key` is a fixed random 96-bit (8-byte) string.
The `groupKey` for an address is a string representing its network group.
The `source` of an address is the address of the peer from which we learn the
address..
The first (internal) hash is reduced to an integer up to `newBucketsPerGroup =
32`, for new addresses, and `oldBucketsPerGroup = 4`, for old addresses.
The second (external) hash is reduced to bucket indexes, in the interval from 0
to the number of new (`newBucketCount = 256`) or old (`oldBucketCount = 64`) buckets.

Notice that new addresses with sources from the same network group are more
likely to end up in the same bucket, therefore to competing for it.
For old address, instead, two addresses are more likely to end up in the same
bucket when they belong to the same network group.

## Adding addresses

The `AddAddress` method adds the address of a peer to the address book.

The added address is associated to a *source* address, which identifies the
node from which the peer address was learned.

Addresses are added to the address book in the following situations:

1. When a peer address is learned via PEX protocol, having the sender
   of the PEX message as its source
2. When an inbound peer is added, in this case the peer itself is set as the
   source of its own address
3. When the switch is instructed to dial addresses via the `DialPeersAsync`
   method, in this case the node itself is set as the source

If the added address contains a node ID that is not registered in the address
book, the address is added to a [bucket](#buckets) of new addresses.
Otherwise, the additional address for an existing node ID is **not added** to
the address book when:

- The last address added with the same node ID is stored in an old bucket, so
  it is considered a "good" address
- There are addresses associated to the same node ID stored in
  `maxNewBucketsPerAddress = 4` distinct buckets
- Randomly, with a probability that increases exponentially with the number of
  buckets in which there is an address with the same node ID.
  So, a new address for a node ID which is already present in one bucket is
  added with 50% of probability; if the node ID is present in two buckets, the
  probability decreases to 25%; and if it is present in three buckets, the
  probability is 12.5%.

The new address is also added to the `addrLookup` table, which stores
`knownAddress` entries indexed by their node IDs.
If the new address is from an unknown peer, a new entry is added to the
`addrLookup` table; otherwise, the existing entry is updated with the new
address.
Entries of this table contain, among other fields, the list of buckets where
addresses of a peer are stored.
The `addrLookup` table is used by most of the address book methods (e.g.,
`HasAddress`, `IsGood`, `MarkGood`, `MarkAttempt`), as it provides fast access
to addresses.

### Errors

- if the added address or the associated source address are nil
- if the added address is invalid
- if the added address is the local node's address
- if the added address ID is of a [banned](#bad-peers) peer
- if either the added address or the associated source address IDs are configured as private IDs
- if `routabilityStrict` is set and the address is not routable
- in case of failures computing the bucket for the new address (`calcNewBucket` method)
- if the added address instance, which is a new address, is configured as an
  old address (sanity check of `addToNewBucket` method)

## Need for Addresses

The `NeedMoreAddrs` method verifies whether the address book needs more addresses.

It is invoked by the PEX reactor to define whether to request peer addresses
to a new outbound peer or to a randomly selected connected peer.

The address book needs more addresses when it has less than `1000` addresses
registered, counting all buckets for new and old addresses.

## Pick address

The `PickAddress` method returns an address stored in the address book, chosen
at random with a configurable bias towards new addresses.

It is invoked by the Peer Manager to obtain a peer address to dial, as part of
its `ensurePeers` routine.
The bias starts from 10%, when the peer has no outbound peers, increasing by
10% for each outbound peer the node has, up to 90%, when the node has at least
8 outbound peers.

The configured bias is a parameter that influences the probability of choosing
an address from a bucket of new addresses or from a bucket of old addresses.
A second parameter influencing this choice is the number of new and old
addresses stored in the address book.
In the absence of bias (i.e., if the configured bias is 50%), the probability
of picking a new address is given by the square root of the number of new
addresses divided by the sum of the square roots of the numbers of new and old
addresses.
By adding a bias toward new addresses (i.e., configured bias larger than 50%),
the portion on the sample occupied by the square root of the number of new
addresses increases, while the corresponding portion for old addresses decreases.
As a result, it becomes more likely to pick a new address at random from this sample.

> The use of the square roots softens the impact of disproportional numbers of
> new and old addresses in the address book. This is actually the expected
> scenario, as there are 4 times more buckets for new addresses than buckets
> for old addresses.

Once the type of address, new or old, is defined, a non-empty bucket of this
type is selected at random.
From the selected bucket, an address is chosen at random and returned.
If all buckets of the selected type are empty, no address is returned.

## Random selection

The `GetSelection` method returns a selection of addresses stored in the
address book, with no bias toward new or old addresses.

It is invoked by the PEX protocol to obtain a list of peer addresses with two
purposes:

- To send to a peer in a PEX response, in the case of outbound peers or of
  nodes not operating in seed mode
- To crawl, in the case of nodes operating in seed mode, as part of every
  interaction of the `crawlPeersRoutine`

The selection is a random subset of the peer addresses stored in the
`addrLookup` table, which stores the last address added for each peer ID.
The target size of the selection is `23%` (`getSelectionPercent`) of the
number of addresses stored in the address book, but it should not be lower than
`32` (`minGetSelection`) --- if it is, all addresses in the book are returned
--- nor greater than `250` (`maxGetSelection`).

> The random selection is produced by:
>
> - Retrieving all entries of the `addrLookup` map, which by definition are
>   returned in random order.
> - Randomly shuffling the retrieved list, using the Fisher-Yates algorithm

## Random selection with bias

The `GetSelectionWithBias` method returns a selection of addresses stored in
the address book, with bias toward new addresses.

It is invoked by the PEX protocol to obtain a list of peer addresses to be sent
to a peer in a PEX response.
This method is only invoked by seed nodes, when replying to a PEX request
received from an inbound peer (i.e., a peer that dialed the seed node).
The bias used in this scenario is hard-coded to 30%, meaning that 70% of
the returned addresses are expected to be old addresses.

The number of addresses that compose the selection is computed in the same way
as for the non-biased random selection.
The bias toward new addresses is implemented by requiring that the configured
bias, interpreted as a percentage, of the select addresses come from buckets of
new addresses, while the remaining come from buckets of old addresses.
Since the number of old addresses is typically lower than the number of new
addresses, it is possible that the address book does not have enough old
addresses to include in the selection.
In this case, additional new addresses are included in the selection.
Thus, the configured bias, in practice, is towards old addresses, not towards
new addresses.

To randomly select addresses of a type, the address book considers all
addresses present in every bucket of that type.
This list of all addresses of a type is randomly shuffled, and the requested
number of addresses are retrieved from the tail of this list.
The returned selection contains, at its beginning, a random selection of new
addresses in random order, followed by a random selection of old addresses, in
random order.

## Dial Attempts

The `MarkAttempt` method records a failed attempt to connect to an address.

It is invoked by the Peer Manager when it fails dialing a peer, but the failure
is not in the authentication step (`ErrSwitchAuthenticationFailure` error).
In case of authentication errors, the peer is instead marked as a [bad peer](#bad-peers).

The failed connection attempt is recorded in the address registered for the
peer's ID in the `addrLookup` table, which is the last address added with that ID.
The known address' counter of failed `Attempts` is increased and the failure
time is registered in `LastAttempt`.

The possible effect of recording multiple failed connect attempts to a peer is
to turn its address into a *bad* address (do not confuse with banned addresses).
A known address becomes bad if it is stored in buckets of new addresses, and
when connection attempts:

- Have not been made over a week, i.e., `LastAttempt` is older than a week
- Have failed 3 times and never succeeded, i.e., `LastSucess` field is unset
- Have failed 10 times in the last week, i.e., `LastSucess` is older than a week

Addresses marked as *bad* are the first candidates to be removed from a bucket of
new addresses when the bucket becomes full.

> Note that failed connection attempts are reported for a peer address, but in
> fact the address book records them for a peer.
>
> More precisely, failed connection attempts are recorded in the entry of  the
> `addrLookup` table with reported peer ID, which contains the last address
> added for that node ID, which is not necessarily the reported peer address.

## Good peers

The `MarkGood` method marks a peer ID as good.

It is invoked by the consensus reactor, via switch, when the number of useful
messages received from a peer is a multiple of `10000`.
Vote and block part messages are considered for this number, they must be valid
and not be duplicated messages to be considered useful.

> The `SwitchReporter` type of `behaviour` package also invokes the `MarkGood`
> method when a "reason" associated with consensus votes and block parts is
> reported.
> No reactor, however, currently provides these "reasons" to the `SwitchReporter`.

The effect of this action is that the address registered for the peer's ID in the
`addrLookup` table, which is the last address added with that ID, is marked as
good and moved to a bucket of old addresses.
An address marked as good has its failed to connect counter and timestamp reset.
If the destination bucket of old addresses is full, the oldest address in the
bucket is moved (downgraded) to a bucket of new addresses.

Moving the peer address to a bucket of old addresses has the effect of
upgrading, or increasing the ranking of a peer in the address book.

## Bad peers

The `MarkBad` method marks a peer as bad and bans it for a period of time.

This method is only invoked within the PEX reactor, with a banning time of 24
hours, for the following reasons:

- A peer misbehaves in the [PEX protocol](./pex-protocol.md#misbehavior)
- When the `maxAttemptsToDial` limit (`16`) is reached for a peer
- If an `ErrSwitchAuthenticationFailure` error is returned when dialing a peer

The effect of this action is that the address registered for the peer's ID in the
`addrLookup` table, which is the last address added with that ID, is banned for
a period of time.
The banned peer is removed from the `addrLookup` table and from all buckets
where its addresses are stored.

The information about banned peers, however, is not discarded.
It is maintained in the `badPeers` map, indexed by peer ID.
This allows, in particular, addresses of banned peers to be
[reinstated](#reinstating-addresses), i.e., to be added
back to the address book, when their ban period expires.

## Reinstating addresses

The `ReinstateBadPeers` method attempts to re-add banned addresses to the address book.

It is invoked by the PEX reactor when dialing new peers.
This action is taken before requesting additional addresses to peers,
in the case that the node needs more peer addresses.

The set of banned peer addresses is retrieved from the `badPeers` map.
Addresses that are not any longer banned, i.e., whose banned period has expired,
are added back to the address book as new addresses, while the corresponding
node IDs are removed from the `badPeers` map.

## Removing addresses

The `RemoveAddress` method removes an address from the address book.

It is invoked by the switch when it dials a peer or accepts a connection from a
peer that ends up being the node itself (`IsSelf` error).
In both cases, the address dialed or accepted is also added to the address book
as a local address, via the `AddOurAddress` method.

The same logic is also internally used by the address book for removing
addresses of a peer that is [marked as a bad peer](#bad-peers).

The entry registered with the peer ID of the address in the `addrLookup` table,
which is the last address added with that ID, is removed from all buckets where
it is stored and from the `addrLookup` table.

> FIXME: is it possible that addresses with the same ID as the removed address,
> but with distinct network addresses, are kept in buckets of the address book?
> While they will not be accessible anymore, as there is no reference to them
> in the `addrLookup`, they will still be there.

## Persistence

The `loadFromFile` method, called when the address book is started, reads
address book entries from a file, passed to the address book constructor.
The file, at this point, does not need to exist.

The `saveRoutine` is started when the address book is started.
It saves the address book to the configured file every `dumpAddressInterval`,
hard-coded to 2 minutes.
It is also possible to save the content of the address book using the `Save`
method.
Saving the address book content to a file acquires the address book lock, also
employed by all other public methods.
