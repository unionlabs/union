# ADR 025 Commit

## Context

Currently the `Commit` structure contains a lot of potentially redundant or unnecessary data.
It contains a list of precommits from every validator, where the precommit
includes the whole `Vote` structure. Thus each of the commit height, round,
type, and blockID are repeated for every validator, and could be deduplicated,
leading to very significant savings in block size.

```
type Commit struct {
    BlockID    BlockID `json:"block_id"`
    Precommits []*Vote `json:"precommits"`
}

type Vote struct {
    ValidatorAddress Address   `json:"validator_address"`
    ValidatorIndex   int       `json:"validator_index"`
    Height           int64     `json:"height"`
    Round            int       `json:"round"`
    Timestamp        time.Time `json:"timestamp"`
    Type             byte      `json:"type"`
    BlockID          BlockID   `json:"block_id"`
    Signature        []byte    `json:"signature"`
}
```

The original tracking issue for this is [#1648](https://github.com/tendermint/tendermint/issues/1648).
We have discussed replacing the `Vote` type in `Commit` with a new `CommitSig`
type, which includes at minimum the vote signature. The `Vote` type will
continue to be used in the consensus reactor and elsewhere.

A primary question is what should be included in the `CommitSig` beyond the
signature. One current constraint is that we must include a timestamp, since
this is how we calculate BFT time, though we may be able to change this [in the
future](https://github.com/tendermint/tendermint/issues/2840).

Other concerns here include:

- Validator Address [#3596](https://github.com/tendermint/tendermint/issues/3596) -
    Should the CommitSig include the validator address? It is very convenient to
    do so, but likely not necessary. This was also discussed in [#2226](https://github.com/tendermint/tendermint/issues/2226).
- Absent Votes [#3591](https://github.com/tendermint/tendermint/issues/3591) -
    How to represent absent votes? Currently they are just present as `nil` in the
    Precommits list, which is actually problematic for serialization
- Other BlockIDs [#3485](https://github.com/tendermint/tendermint/issues/3485) -
    How to represent votes for nil and for other block IDs? We currently allow
    votes for nil and votes for alternative block ids, but just ignore them


## Decision

Deduplicate the fields and introduce `CommitSig`:

```
type Commit struct {
    Height  int64
    Round   int
    BlockID    BlockID      `json:"block_id"`
    Precommits []CommitSig `json:"precommits"`
}

type CommitSig struct {
    BlockID  BlockIDFlag
    ValidatorAddress Address
    Timestamp time.Time
    Signature []byte
}


// indicate which BlockID the signature is for
type BlockIDFlag int

const (
	BlockIDFlagAbsent BlockIDFlag = iota // vote is not included in the Commit.Precommits
	BlockIDFlagCommit                    // voted for the Commit.BlockID
	BlockIDFlagNil                       // voted for nil
)

```

Re the concerns outlined in the context:

**Timestamp**: Leave the timestamp for now. Removing it and switching to
proposer based time will take more analysis and work, and will be left for a
future breaking change. In the meantime, the concerns with the current approach to
BFT time [can be
mitigated](https://github.com/tendermint/tendermint/issues/2840#issuecomment-529122431).

**ValidatorAddress**: we include it in the `CommitSig` for now. While this
does increase the block size unnecessarily (20-bytes per validator), it has some ergonomic and debugging advantages:

- `Commit` contains everything necessary to reconstruct `[]Vote`, and doesn't depend on additional access to a `ValidatorSet`
- Lite clients can check if they know the validators in a commit without
  re-downloading the validator set
- Easy to see directly in a commit which validators signed what without having
  to fetch the validator set

If and when we change the `CommitSig` again, for instance to remove the timestamp,
we can reconsider whether the ValidatorAddress should be removed.

**Absent Votes**: we include absent votes explicitly with no Signature or
Timestamp but with the ValidatorAddress. This should resolve the serialization
issues and make it easy to see which validator's votes failed to be included.

**Other BlockIDs**: We use a single byte to indicate which blockID a `CommitSig`
is for. The only options are:
    - `Absent` - no vote received from the this validator, so no signature
    - `Nil` - validator voted Nil - meaning they did not see a polka in time
    - `Commit` - validator voted for this block

Note this means we don't allow votes for any other blockIDs. If a signature is
included in a commit, it is either for nil or the correct blockID. According to
the Tendermint protocol and assumptions, there is no way for a correct validator to
precommit for a conflicting blockID in the same round an actual commit was
created. This was the consensus from
[#3485](https://github.com/tendermint/tendermint/issues/3485)

We may want to consider supporting other blockIDs later, as a way to capture
evidence that might be helpful. We should clarify if/when/how doing so would
actually help first. To implement it, we could change the `Commit.BlockID`
field to a slice, where the first entry is the correct block ID and the other
entries are other BlockIDs that validators precommited before. The BlockIDFlag
enum can be extended to represent these additional block IDs on a per block
basis.

## Status

Implemented

## Consequences

### Positive

Removing the Type/Height/Round/Index and the BlockID saves roughly 80 bytes per precommit.
It varies because some integers are varint. The BlockID contains two 32-byte hashes an integer,
and the Height is 8-bytes.

For a chain with 100 validators, that's up to 8kB in savings per block!


### Negative

- Large breaking change to the block and commit structure
- Requires differentiating in code between the Vote and CommitSig objects, which may add some complexity (votes need to be reconstructed to be verified and gossiped)

### Neutral

- Commit.Precommits no longer contains nil values
