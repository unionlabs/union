# ADR 007: Trust Metric Usage Guide

## Context

Tendermint is required to monitor peer quality in order to inform its peer dialing and peer exchange strategies.

When a node first connects to the network, it is important that it can quickly find good peers.
Thus, while a node has fewer connections, it should prioritize connecting to higher quality peers.
As the node becomes well connected to the rest of the network, it can dial lesser known or lesser
quality peers and help assess their quality. Similarly, when queried for peers, a node should make
sure they dont return low quality peers.

Peer quality can be tracked using a trust metric that flags certain behaviors as good or bad. When enough
bad behavior accumulates, we can mark the peer as bad and disconnect.
For example, when the PEXReactor makes a request for peers network addresses from an already known peer, and the returned network addresses are unreachable, this undesirable behavior should be tracked. Returning a few bad network addresses probably shouldn’t cause a peer to be dropped, while excessive amounts of this behavior does qualify the peer for removal. The originally proposed approach and design document for the trust metric can be found in the [ADR 006](adr-006-trust-metric.md) document.

The trust metric implementation allows a developer to obtain a peer's trust metric from a trust metric store, and track good and bad events relevant to a peer's behavior, and at any time, the peer's metric can be queried for a current trust value. The current trust value is calculated with a formula that utilizes current behavior, previous behavior, and change between the two. Current behavior is calculated as the percentage of good behavior within a time interval. The time interval is short; probably set between 30 seconds and 5 minutes. On the other hand, the historic data can estimate a peer's behavior over days worth of tracking. At the end of a time interval, the current behavior becomes part of the historic data, and a new time interval begins with the good and bad counters reset to zero.

These are some important things to keep in mind regarding how the trust metrics handle time intervals and scoring:

- Each new time interval begins with a perfect score
- Bad events quickly bring the score down and good events cause the score to slowly rise
- When the time interval is over, the percentage of good events becomes historic data.

Some useful information about the inner workings of the trust metric:

- When a trust metric is first instantiated, a timer (ticker) periodically fires in order to handle transitions between trust metric time intervals
- If a peer is disconnected from a node, the timer should be paused, since the node is no longer connected to that peer
- The ability to pause the metric is supported with the store **PeerDisconnected** method and the metric **Pause** method
- After a pause, if a good or bad event method is called on a metric, it automatically becomes unpaused and begins a new time interval.

## Decision

The trust metric capability is now available, yet, it still leaves the question of how should it be applied throughout Tendermint in order to properly track the quality of peers?

### Proposed Process

Peers are managed using an address book and a trust metric:

- The address book keeps a record of peers and provides selection methods
- The trust metric tracks the quality of the peers

#### Presence in Address Book

Outbound peers are added to the address book before they are dialed,
and inbound peers are added once the peer connection is set up.
Peers are also added to the address book when they are received in response to
a pexRequestMessage.

While a node has less than `needAddressThreshold`, it will periodically request more,
via pexRequestMessage, from randomly selected peers and from newly dialed outbound peers.

When a new address is added to an address book that has more than `0.5*needAddressThreshold` addresses,
then with some low probability, a randomly chosen low quality peer is removed.

#### Outbound Peers

Peers attempt to maintain a minimum number of outbound connections by
repeatedly querying the address book for peers to connect to.
While a node has few to no outbound connections, the address book is biased to return
higher quality peers. As the node increases the number of outbound connections,
the address book is biased to return less-vetted or lower-quality peers.

#### Inbound Peers

Peers also maintain a maximum number of total connections, MaxNumPeers.
If a peer has MaxNumPeers, new incoming connections will be accepted with low probability.
When such a new connection is accepted, the peer disconnects from a probabilistically chosen low ranking peer
so it does not exceed MaxNumPeers.

#### Peer Exchange

When a peer receives a pexRequestMessage, it returns a random sample of high quality peers from the address book. Peers with no score or low score should not be inclided in a response to pexRequestMessage.

#### Peer Quality

Peer quality is tracked in the connection and across the reactors by storing the TrustMetric in the peer's
thread safe Data store.

Peer behavior is then defined as one of the following:

- Fatal - something outright malicious that causes us to disconnect the peer and ban it from the address book for some amount of time
- Bad - Any kind of timeout, messages that don't unmarshal, fail other validity checks, or messages we didn't ask for or aren't expecting (usually worth one bad event)
- Neutral - Unknown channels/message types/version upgrades (no good or bad events recorded)
- Correct - Normal correct behavior (worth one good event)
- Good - some random majority of peers per reactor sending us useful messages (worth more than one good event).

Note that Fatal behavior causes us to remove the peer, and neutral behavior does not affect the score.

## Status

Proposed.

## Consequences

### Positive

- Bringing the address book and trust metric store together will cause the network to be built in a way that encourages greater security and reliability.

### Negative

- TBD

### Neutral

- Keep in mind that, good events need to be recorded just as bad events do using this implementation.
