# ADR 080: ReverseSync - fetching historical data

## Changelog

- 2021-02-11: Migrate to tendermint repo (Originally [RFC 005](https://github.com/tendermint/spec/pull/224))
- 2021-04-19: Use P2P to gossip necessary data for reverse sync.
- 2021-03-03: Simplify proposal to the state sync case.
- 2021-02-17: Add notes on asynchronicity of processes.
- 2020-12-10: Rename backfill blocks to reverse sync.
- 2020-11-25: Initial draft.

## Author(s)

- Callum Waters (@cmwaters)

## Context

Two new features: [Block pruning](https://github.com/tendermint/tendermint/issues/3652)
and [State sync](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-042-state-sync.md)
meant nodes no longer needed a complete history of the blockchain. This
introduced some challenges of its own which were covered and subsequently
tackled with [RFC-001](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-077-block-retention.md).
The RFC allowed applications to set a block retention height; an upper bound on
what blocks would be pruned. However nodes who state sync past this upper bound
(which is necessary as snapshots must be saved within the trusting period for
the assisting light client to verify) have no means of backfilling the blocks
to meet the retention limit. This could be a problem as nodes who state sync and
then eventually switch to consensus (or fast sync) may not have the block and
validator history to verify evidence causing them to panic if they see 2/3
commit on what the node believes to be an invalid block.

Thus, this RFC sets out to instil a minimum block history invariant amongst
honest nodes.

## Proposal

A backfill mechanism can simply be defined as an algorithm for fetching,
verifying and storing, headers and validator sets of a height prior to the
current base of the node's blockchain. In matching the terminology used for
other data retrieving protocols (i.e. fast sync and state sync), we
call this method **ReverseSync**.

We will define the mechanism in four sections:

- Usage
- Design
- Verification
- Termination

### Usage

For now, we focus purely on the case of a state syncing node, whom after
syncing to a height will need to verify historical data in order to be capable
of processing new blocks. We can denote the earliest height that the node will
need to verify and store in order to be able to verify any evidence that might
arise as the `max_historical_height`/`time`. Both height and time are necessary
as this maps to the BFT time used for evidence expiration. After acquiring
`State`, we calculate these parameters as:

```go
max_historical_height = max(state.InitialHeight, state.LastBlockHeight - state.ConsensusParams.EvidenceAgeHeight)
max_historical_time = max(GenesisTime, state.LastBlockTime.Sub(state.ConsensusParams.EvidenceAgeTime))
```

Before starting either fast sync or consensus, we then run the following
synchronous process:

```go
func ReverseSync(max_historical_height int64, max_historical_time time.Time) error
```

Where we fetch and verify blocks until a block `A` where
`A.Height <= max_historical_height` and `A.Time <= max_historical_time`.

Upon successfully reverse syncing, a node can now safely continue. As this
feature is only used as part of state sync, one can think of this as merely an
extension to it.

In the future we may want to extend this functionality to allow nodes to fetch
historical blocks for reasons of accountability or data accessibility.

### Design

This section will provide a high level overview of some of the more important
characteristics of the design, saving the more tedious details as an ADR.

#### P2P

Implementation of this RFC will require the addition of a new channel and two
new messages.

```proto
message LightBlockRequest {
  uint64 height = 1;
}
```

```proto
message LightBlockResponse {
  Header header = 1;
  Commit commit = 2;
  ValidatorSet validator_set = 3;
}
```

The P2P path may also enable P2P networked light clients and a state sync that
also doesn't need to rely on RPC.

### Verification

ReverseSync is used to fetch the following data structures:

- `Header`
- `Commit`
- `ValidatorSet`

Nodes will also need to be able to verify these. This can be achieved by first
retrieving the header at the base height from the block store. From this trusted
header, the node hashes each of the three data structures and checks that they are correct.

1. The trusted header's last block ID matches the hash of the new header

   ```go
   header[height].LastBlockID == hash(header[height-1])
   ```

2. The trusted header's last commit hash matches the hash of the new commit

	```go
	header[height].LastCommitHash == hash(commit[height-1])
	```

3. Given that the node now trusts the new header, check that the header's validator set
   hash matches the hash of the validator set

	```go
	header[height-1].ValidatorsHash == hash(validatorSet[height-1])
	```

### Termination

ReverseSync draws a lot of parallels with fast sync. An important consideration
for fast sync that also extends to ReverseSync is termination. ReverseSync will
finish it's task when one of the following conditions have been met:

1. It reaches a block `A` where `A.Height <= max_historical_height` and
`A.Time <= max_historical_time`.
2. None of it's peers reports to have the block at the height below the
processes current block.
3. A global timeout.

This implies that we can't guarantee adequate history and thus the term
"invariant" can't be used in the strictest sense. In the case that the first
condition isn't met, the node will log an error and optimistically attempt
to continue with either fast sync or consensus.

## Alternative Solutions

The need for a minimum block history invariant stems purely from the need to
validate evidence (although there may be some application relevant needs as
well). Because of this, an alternative, could be to simply trust whatever the
2/3+ majority has agreed upon and in the case where a node is at the head of the
blockchain, you simply abstain from voting.

As it stands, if 2/3+ vote on evidence you can't verify, in the same manner if
2/3+ vote on a header that a node sees as invalid (perhaps due to a different
app hash), the node will halt.

Another alternative is the method with which the relevant data is retrieved.
Instead of introducing new messages to the P2P layer, RPC could have been used
instead.

The aforementioned data is already available via the following RPC endpoints:
`/commit` for `Header`'s' and `/validators` for `ValidatorSet`'s'. It was
decided predominantly due to the instability of the current RPC infrastructure
that P2P be used instead.

## Status

Proposed

## Consequences

### Positive

- Ensures a minimum block history invariant for honest nodes. This will allow
  nodes to verify evidence.

### Negative

- Statesync will be slower as more processing is required.

### Neutral

- By having validator sets served through p2p, this would make it easier to
extend p2p support to light clients and state sync.
- In the future, it may also be possible to extend this feature to allow for
nodes to freely fetch and verify prior blocks

## References

- [RFC-001: Block retention](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-077-block-retention.md)
- [Original issue](https://github.com/tendermint/tendermint/issues/4629)
