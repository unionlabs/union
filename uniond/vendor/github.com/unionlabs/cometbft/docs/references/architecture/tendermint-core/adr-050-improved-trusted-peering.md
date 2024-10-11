# ADR 50: Improved Trusted Peering

## Changelog
* 22-10-2019: Initial draft
* 05-11-2019: Modify `maximum-dial-period` to `persistent-peers-max-dial-period`

## Context

When `max-num-inbound-peers` or `max-num-outbound-peers` of a node is reached, the node cannot spare more slots to any peer 
by inbound or outbound. Therefore, after a certain period of disconnection, any important peering can be lost indefinitely 
because all slots are consumed by other peers, and the node stops trying to dial the peer anymore.

This is happening because of two reasons, exponential backoff and absence of unconditional peering feature for trusted peers.


## Decision

We would like to suggest solving this problem by introducing two parameters in `config.toml`, `unconditional-peer-ids` and 
`persistent-peers-max-dial-period`. 

1) `unconditional-peer-ids`

A node operator inputs list of ids of peers which are allowed to be connected by both inbound or outbound regardless of 
`max-num-inbound-peers` or `max-num-outbound-peers` of user's node reached or not.

2) `persistent-peers-max-dial-period`

Terms between each dial to each persistent peer will not exceed `persistent-peers-max-dial-period` during exponential backoff. 
Therefore, `dial-period` = min(`persistent-peers-max-dial-period`, `exponential-backoff-dial-period`)

Alternative approach

Persistent-peers is only for outbound, therefore it is not enough to cover the full utility of `unconditional-peer-ids`. 
@creamers158(https://github.com/Creamers158) suggested putting id-only items into persistent-peers to be handled as 
`unconditional-peer-ids`, but it needs very complicated struct exception for different structure of items in persistent-peers.
Therefore we decided to have `unconditional-peer-ids` to independently cover this use-case.

## Status

Proposed

## Consequences

### Positive

A node operator can configure two new parameters in `config.toml` so that he/she can assure that tendermint will allow connections
from/to peers in `unconditional-peer-ids`. Also he/she can assure that every persistent peer will be dialed at least once in every 
`persistent-peers-max-dial-period` term. It achieves more stable and persistent peering for trusted peers.

### Negative

The new feature introduces two new parameters in `config.toml` which needs explanation for node operators.

### Neutral

## References

* two p2p feature enhancement proposal(https://github.com/tendermint/tendermint/issues/4053)
