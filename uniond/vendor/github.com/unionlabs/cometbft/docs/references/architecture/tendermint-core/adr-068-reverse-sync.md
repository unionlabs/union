# ADR 068: Reverse Sync

## Changelog

- 20 April 2021: Initial Draft (@cmwaters)

## Status

Proposed

## Context

The advent of state sync and block pruning gave rise to the opportunity for full nodes to participate in consensus without needing complete block history. This also introduced a problem with respect to evidence handling. Nodes that didn't have all the blocks within the evidence age were incapable of validating evidence, thus halting if that evidence was committed on chain.

[ADR 068](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-068-reverse-sync.md) was published in response to this problem and modified the spec to add a minimum block history invariant. This predominantly sought to extend state sync so that it was capable of fetching and storing the `Header`, `Commit` and `ValidatorSet` (essentially a `LightBlock`) of the last `n` heights, where `n` was calculated based from the evidence age.

This ADR sets out to describe the design of this state sync extension as well as modifications to the light client provider and the merging of tm store.

## Decision

The state sync reactor will be extended by introducing 2 new P2P messages (and a new channel).

```protobuf
message LightBlockRequest {
  uint64 height = 1;
}

message LightBlockResponse {
  tendermint.types.LightBlock light_block = 1;
}
```

This will be used by the "reverse sync" protocol that will fetch, verify and store prior light blocks such that the node can safely participate in consensus.

Furthermore this allows for a new light client provider which offers the ability for the `StateProvider` to use the underlying P2P stack instead of RPC.

## Detailed Design

This section will focus first on the reverse sync (here we call it `backfill`) mechanism as a standalone protocol and then look to describe how it integrates within the state sync reactor and how we define the new p2p light client provider.

```go
// Backfill fetches, verifies, and stores necessary history
// to participate in consensus and validate evidence.
func (r *Reactor) backfill(state State) error {}
```

`State` is used to work out how far to go back, namely we need all light blocks that have:
- a height: `h >= state.LastBlockHeight - state.ConsensusParams.Evidence.MaxAgeNumBlocks`
- a time: `t >= state.LastBlockTime - state.ConsensusParams.Evidence.MaxAgeDuration`

Reverse Sync relies on two components: A `Dispatcher` and a `BlockQueue`. The `Dispatcher` is a pattern taken from a similar [PR](https://github.com/tendermint/tendermint/pull/4508). It is wired to the `LightBlockChannel` and allows for concurrent light block requests by shifting through a linked list of peers. This abstraction has the nice quality that it can also be used as an array of light providers for a P2P based light client.

The `BlockQueue` is a data structure that allows for multiple workers to fetch light blocks, serializing them for the main thread which picks them off the end of the queue, verifies the hashes and persists them.

### Integration with State Sync

Reverse sync is a blocking process that runs directly after syncing state and before transitioning into either fast sync or consensus.

Prior, the state sync service was not connected to any db, instead it passed the state and commit back to the node. For reverse sync, state sync will be given access to both the `StateStore` and `BlockStore` to be able to write `Header`'s, `Commit`'s and `ValidatorSet`'s and read them so as to serve other state syncing peers.

This also means adding new methods to these respective stores in order to persist them

### P2P Light Client Provider

As mentioned previously, the `Dispatcher` is capable of handling requests to multiple peers. We can therefore simply peel off a `blockProvider` instance which is assigned to each peer. By giving it the chain ID, the `blockProvider` is capable of doing a basic validation of the light block before returning it to the client.

It's important to note that because state sync doesn't have access to the evidence channel it is incapable of allowing the light client to report evidence thus `ReportEvidence` is a no op. This is not too much of a concern for reverse sync but will need to be addressed for pure p2p light clients.

### Pruning

A final small note is with pruning. This ADR will introduce changes that will not allow an application to prune blocks that are within the evidence age.

## Future Work

This ADR tries to remain within the scope of extending state sync, however the changes made opens the door for several areas to be followed up:
- Properly integrate p2p messaging in the light client package. This will require adding the evidence channel so the light client is capable of reporting evidence. We may also need to rethink the providers model (i.e. currently providers are only added on start up)
- Merge and clean up the tendermint stores (state, block and evidence). This ADR adds new methods to both the state and block store for saving headers, commits and validator sets. This doesn't quite fit with the current struct (i.e. only `BlockMeta`s instead of `Header`s are saved). We should explore consolidating this for the sake of atomicity and the opportunity for batching. There are also other areas for changes such as the way we store block parts. See [here](https://github.com/tendermint/tendermint/issues/5383) and [here](https://github.com/tendermint/tendermint/issues/4630) for more context.
- Explore opportunistic reverse sync. Technically we don't need to reverse sync if no evidence is observed. I've tried to design the protocol such that it could be possible to move it across to the evidence package if we see fit. Thus only when evidence is seen where we don't have the necessary data, do we perform a reverse sync. The problem with this is that imagine we are in consensus and some evidence pops up requiring us to first fetch and verify the last 10,000 blocks. There's no way a node could do this (sequentially) and vote before the round finishes. Also as we don't punish invalid evidence, a malicious node could easily spam the chain just to get a bunch of "stateless" nodes to perform a bunch of useless work.
- Explore full reverse sync. Currently we only fetch light blocks. There might be benefits in the future to fetch and persist entire blocks especially if we give control to the application to do this.

## Consequences

### Positive

- All nodes should have sufficient history to validate all types of evidence
- State syncing nodes can use the p2p layer for light client verification of state. This has better UX and could be faster but I haven't benchmarked.

### Negative

- Introduces more code = more maintenance

### Neutral

## References

- [Reverse Sync RFC](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-068-reverse-sync.md)
- [Original Issue](https://github.com/tendermint/tendermint/issues/5617)
