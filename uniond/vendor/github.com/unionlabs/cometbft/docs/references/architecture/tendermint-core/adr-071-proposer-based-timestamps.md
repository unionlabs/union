# ADR 71: Proposer-Based Timestamps

## Changelog

 - July 15 2021: Created by @williambanfield
 - Aug 4 2021: Draft completed by @williambanfield
 - Aug 5 2021: Draft updated to include data structure changes by @williambanfield
 - Aug 20 2021: Language edits completed by @williambanfield
 - Oct 25 2021: Update the ADR to match updated spec from @cason by @williambanfield
 - Nov 10 2021: Additional language updates by @williambanfield per feedback from @cason
 - Feb 2 2022: Synchronize logic for timely with latest version of the spec by @williambanfield

## Status

 **Accepted**

## Context

Tendermint currently provides a monotonically increasing source of time known as [BFTTime](https://github.com/tendermint/tendermint/blob/main/spec/consensus/bft-time.md).
This mechanism for producing a source of time is reasonably simple.
Each correct validator adds a timestamp to each `Precommit` message it sends.
The timestamp it sends is either the validator's current known Unix time or one millisecond greater than the previous block time, depending on which value is greater.
When a block is produced, the proposer chooses the block timestamp as the weighted median of the times in all of the `Precommit` messages the proposer received.
The weighting is proportional to the amount of voting power, or stake, a validator has on the network.
This mechanism for producing timestamps is both deterministic and byzantine fault tolerant.

This current mechanism for producing timestamps has a few drawbacks.
Validators do not have to agree at all on how close the selected block timestamp is to their own currently known Unix time.
Additionally, any amount of voting power `>1/3` may directly control the block timestamp.
As a result, it is quite possible that the timestamp is not particularly meaningful.

These drawbacks present issues in the Tendermint protocol.
Timestamps are used by light clients to verify blocks.
Light clients rely on correspondence between their own currently known Unix time and the block timestamp to verify blocks they see;
However, their currently known Unix time may be greatly divergent from the block timestamp as a result of the limitations of `BFTTime`.

The proposer-based timestamps specification suggests an alternative approach for producing block timestamps that remedies these issues.
Proposer-based timestamps alter the current mechanism for producing block timestamps in two main ways:

1. The block proposer is amended to offer up its currently known Unix time as the timestamp for the next block instead of the `BFTTime`.
1. Correct validators only approve the proposed block timestamp if it is close enough to their own currently known Unix time.

The result of these changes is a more meaningful timestamp that cannot be controlled by `<= 2/3` of the validator voting power.
This document outlines the necessary code changes in Tendermint to implement the corresponding [proposer-based timestamps specification](https://github.com/tendermint/tendermint/tree/main/spec/consensus/proposer-based-timestamp).

## Alternative Approaches

### Remove timestamps altogether

Computer clocks are bound to skew for a variety of reasons.
Using timestamps in our protocol means either accepting the timestamps as not reliable or impacting the protocol’s liveness guarantees.
This design requires impacting the protocol’s liveness in order to make the timestamps more reliable.
An alternate approach is to remove timestamps altogether from the block protocol.
`BFTTime` is deterministic but may be arbitrarily inaccurate.
However, having a reliable source of time is quite useful for applications and protocols built on top of a blockchain.

We therefore decided not to remove the timestamp.
Applications often wish for some transactions to occur on a certain day, on a regular period, or after some time following a different event.
All of these require some meaningful representation of agreed upon time.
The following protocols and application features require a reliable source of time:
* Tendermint Light Clients [rely on correspondence between their known time](https://github.com/tendermint/tendermint/blob/main/spec/light-client/verification/README.md#definitions-1) and the block time for block verification.
* Tendermint Evidence validity is determined [either in terms of heights or in terms of time](https://github.com/tendermint/tendermint/blob/8029cf7a0fcc89a5004e173ec065aa48ad5ba3c8/spec/consensus/evidence.md#verification).
* Unbonding of staked assets in the Cosmos Hub [occurs after a period of 21 days](https://github.com/cosmos/governance/blob/ce75de4019b0129f6efcbb0e752cd2cc9e6136d3/params-change/Staking.md#unbondingtime).
* IBC packets can use either a [timestamp or a height to timeout packet delivery](https://docs.cosmos.network/v0.45/ibc/overview.html#acknowledgements)

Finally, inflation distribution in the Cosmos Hub uses an approximation of time to calculate an annual percentage rate.
This approximation of time is calculated using [block heights with an estimated number of blocks produced in a year](https://github.com/cosmos/governance/blob/master/params-change/Mint.md#blocksperyear).
Proposer-based timestamps will allow this inflation calculation to use a more meaningful and accurate source of time.


## Decision

Implement proposer-based timestamps and remove `BFTTime`.

## Detailed Design

### Overview

Implementing proposer-based timestamps will require a few changes to Tendermint’s code.
These changes will be to the following components:
* The `internal/consensus/` package.
* The `state/` package.
* The `Vote`, `CommitSig` and `Header` types.
* The consensus parameters.

### Changes to `CommitSig`

The [CommitSig](https://github.com/tendermint/tendermint/blob/a419f4df76fe4aed668a6c74696deabb9fe73211/types/block.go#L604) struct currently contains a timestamp.
This timestamp is the current Unix time known to the validator when it issued a `Precommit` for the block.
This timestamp is no longer used and will be removed in this change.

`CommitSig` will be updated as follows:

```diff
type CommitSig struct {
	BlockIDFlag      BlockIDFlag `json:"block_id_flag"`
	ValidatorAddress Address     `json:"validator_address"`
--	Timestamp        time.Time   `json:"timestamp"`
	Signature        []byte      `json:"signature"`
}
```

### Changes to `Vote` messages

`Precommit` and `Prevote` messages use a common [Vote struct](https://github.com/tendermint/tendermint/blob/a419f4df76fe4aed668a6c74696deabb9fe73211/types/vote.go#L50).
This struct currently contains a timestamp.
This timestamp is set using the [voteTime](https://github.com/tendermint/tendermint/blob/e8013281281985e3ada7819f42502b09623d24a0/internal/consensus/state.go#L2241) function and therefore vote times correspond to the current Unix time known to the validator, provided this time is greater than the timestamp of the previous block.
For precommits, this timestamp is used to construct the [CommitSig that is included in the block in the LastCommit](https://github.com/tendermint/tendermint/blob/e8013281281985e3ada7819f42502b09623d24a0/types/block.go#L754) field.
For prevotes, this field is currently unused.
Proposer-based timestamps will use the timestamp that the proposer sets into the block and will therefore no longer require that a timestamp be included in the vote messages.
This timestamp is therefore no longer useful as part of consensus and may optionally be dropped from the message.

`Vote` will be updated as follows:

```diff
type Vote struct {
	Type             tmproto.SignedMsgType `json:"type"`
	Height           int64                 `json:"height"`
	Round            int32                 `json:"round"`
	BlockID          BlockID               `json:"block_id"` // zero if vote is nil.
--	Timestamp        time.Time             `json:"timestamp"`
	ValidatorAddress Address               `json:"validator_address"`
	ValidatorIndex   int32                 `json:"validator_index"`
	Signature        []byte                `json:"signature"`
}
```

### New consensus parameters

The proposer-based timestamp specification includes a pair of new parameters that must be the same among all validators.
These parameters are `PRECISION`, and `MSGDELAY`.

The `PRECISION` and `MSGDELAY` parameters are used to determine if the proposed timestamp is acceptable.
A validator will only Prevote a proposal if the proposal timestamp is considered `timely`.
A proposal timestamp is considered `timely` if it is within `PRECISION` and `MSGDELAY` of the Unix time known to the validator.
More specifically, a proposal timestamp is `timely` if `proposalTimestamp - PRECISION ≤ validatorLocalTime ≤ proposalTimestamp + PRECISION + MSGDELAY`.

Because the `PRECISION` and `MSGDELAY` parameters must be the same across all validators, they will be added to the [consensus parameters](https://github.com/tendermint/tendermint/blob/main/proto/tendermint/types/params.proto#L11) as [durations](https://protobuf.dev/reference/protobuf/google.protobuf/#duration).

The consensus parameters will be updated to include this `Synchrony` field as follows:

```diff
type ConsensusParams struct {
	Block     BlockParams     `json:"block"`
	Evidence  EvidenceParams  `json:"evidence"`
	Validator ValidatorParams `json:"validator"`
	Version   VersionParams   `json:"version"`
++	Synchrony SynchronyParams `json:"synchrony"`
}
```

```go
type SynchronyParams struct {
	MessageDelay time.Duration `json:"message_delay"`
	Precision    time.Duration `json:"precision"`
}
```

### Changes to the block proposal step

#### Proposer selects block timestamp

Tendermint currently uses the `BFTTime` algorithm to produce the block's `Header.Timestamp`.
The [proposal logic](https://github.com/tendermint/tendermint/blob/68ca65f5d79905abd55ea999536b1a3685f9f19d/internal/state/state.go#L269) sets the weighted median of the times in the `LastCommit.CommitSigs` as the proposed block's `Header.Timestamp`.

In proposer-based timestamps, the proposer will still set a timestamp into the `Header.Timestamp`.
The timestamp the proposer sets into the `Header` will change depending on if the block has previously received a [polka](https://github.com/tendermint/tendermint/blob/053651160f496bb44b107a434e3e6482530bb287/docs/introduction/what-is-tendermint.md#consensus-overview) or not.

#### Proposal of a block that has not previously received a polka

If a proposer is proposing a new block then it will set the Unix time currently known to the proposer into the `Header.Timestamp` field.
The proposer will also set this same timestamp into the `Timestamp` field of the `Proposal` message that it issues.

#### Re-proposal of a block that has previously received a polka

If a proposer is re-proposing a block that has previously received a polka on the network, then the proposer does not update the `Header.Timestamp` of that block.
Instead, the proposer simply re-proposes the exact same block.
This way, the proposed block has the exact same block ID as the previously proposed block and the validators that have already received that block do not need to attempt to receive it again.

The proposer will set the re-proposed block's `Header.Timestamp` as the `Proposal` message's `Timestamp`.

#### Proposer waits

Block timestamps must be monotonically increasing.
In `BFTTime`, if a validator’s clock was behind, the [validator added 1 millisecond to the previous block’s time and used that in its vote messages](https://github.com/tendermint/tendermint/blob/e8013281281985e3ada7819f42502b09623d24a0/internal/consensus/state.go#L2246).
A goal of adding proposer-based timestamps is to enforce some degree of clock synchronization, so having a mechanism that completely ignores the Unix time of the validator time no longer works.
Validator clocks will not be perfectly in sync.
Therefore, the proposer’s current known Unix time may be less than the previous block's `Header.Time`.
If the proposer’s current known Unix time is less than the previous block's `Header.Time`, the proposer will sleep until its known Unix time exceeds it.

This change will require amending the [defaultDecideProposal](https://github.com/tendermint/tendermint/blob/822893615564cb20b002dd5cf3b42b8d364cb7d9/internal/consensus/state.go#L1180) method.
This method should now schedule a timeout that fires when the proposer’s time is greater than the previous block's `Header.Time`.
When the timeout fires, the proposer will finally issue the `Proposal` message.

### Changes to proposal validation rules

The rules for validating a proposed block will be modified to implement proposer-based timestamps.
We will change the validation logic to ensure that a proposal is `timely`.

Per the proposer-based timestamps spec, `timely` only needs to be checked if a block has not received a +2/3 majority of `Prevotes` in a round.
If a block previously received a +2/3 majority of prevotes in a previous round, then +2/3 of the voting power considered the block's timestamp near enough to their own currently known Unix time in that round.

The validation logic will be updated to check `timely` for blocks that did not previously receive +2/3 prevotes in a round.
Receiving +2/3 prevotes in a round is frequently referred to as a 'polka' and we will use this term for simplicity.

#### Current timestamp validation logic

To provide a better understanding of the changes needed to timestamp validation, we will first detail how timestamp validation works currently in Tendermint.

The [validBlock function](https://github.com/tendermint/tendermint/blob/c3ae6f5b58e07b29c62bfdc5715b6bf8ae5ee951/state/validation.go#L14) currently [validates the proposed block timestamp in three ways](https://github.com/tendermint/tendermint/blob/c3ae6f5b58e07b29c62bfdc5715b6bf8ae5ee951/state/validation.go#L118).
First, the validation logic checks that this timestamp is greater than the previous block’s timestamp.

Second, it validates that the block timestamp is correctly calculated as the weighted median of the timestamps in the [block’s LastCommit](https://github.com/tendermint/tendermint/blob/e8013281281985e3ada7819f42502b09623d24a0/types/block.go#L48).

Finally, the validation logic authenticates the timestamps in the `LastCommit.CommitSig`.
The cryptographic signature in each `CommitSig` is created by signing a hash of fields in the block with the voting validator’s private key.
One of the items in this `signedBytes` hash is the timestamp in the `CommitSig`.
To authenticate the `CommitSig` timestamp, the validator authenticating votes builds a hash of fields that includes the `CommitSig` timestamp and checks this hash against the signature.
This takes place in the [VerifyCommit function](https://github.com/tendermint/tendermint/blob/e8013281281985e3ada7819f42502b09623d24a0/types/validation.go#L25).

#### Remove unused timestamp validation logic

`BFTTime` validation is no longer applicable and will be removed.
This means that validators will no longer check that the block timestamp is a weighted median of `LastCommit` timestamps.
Specifically, we will remove the call to [MedianTime in the validateBlock function](https://github.com/tendermint/tendermint/blob/4db71da68e82d5cb732b235eeb2fd69d62114b45/state/validation.go#L117).
The `MedianTime` function can be completely removed.

Since `CommitSig`s will no longer contain a timestamp, the validator authenticating a commit will no longer include the `CommitSig` timestamp in the hash of fields it builds to check against the cryptographic signature.

#### Timestamp validation when a block has not received a polka

The [POLRound](https://github.com/tendermint/tendermint/blob/68ca65f5d79905abd55ea999536b1a3685f9f19d/types/proposal.go#L29) in the `Proposal` message indicates which round the block received a polka.
A negative value in the `POLRound` field indicates that the block has not previously been proposed on the network.
Therefore the validation logic will check for timely when `POLRound < 0`.

When a validator receives a `Proposal` message, the validator will check that the `Proposal.Timestamp` is at most `PRECISION` greater than the current Unix time known to the validator, and at maximum `PRECISION + MSGDELAY` less than the current Unix time known to the validator.
If the timestamp is not within these bounds, the proposed block will not be considered `timely`.

Once a full block matching the `Proposal` message is received, the validator will also check that the timestamp in the `Header.Timestamp` of the block matches this `Proposal.Timestamp`.
Using the `Proposal.Timestamp` to check `timely` allows for the `MSGDELAY` parameter to be more finely tuned since `Proposal` messages do not change sizes and are therefore faster to gossip than full blocks across the network.

A validator will also check that the proposed timestamp is greater than the timestamp of the block for the previous height.
If the timestamp is not greater than the previous block's timestamp, the block will not be considered valid, which is the same as the current logic.

#### Timestamp validation when a block has received a polka

When a block is re-proposed that has already received a +2/3 majority of `Prevote`s on the network, the `Proposal` message for the re-proposed block is created with a `POLRound` that is `>= 0`.
A validator will not check that the `Proposal` is `timely` if the propose message has a non-negative `POLRound`.
If the `POLRound` is non-negative, each validator will simply ensure that it received the `Prevote` messages for the proposed block in the round indicated by `POLRound`.

If the validator does not receive `Prevote` messages for the proposed block before the proposal timeout, then it will prevote nil.
Validators already check that +2/3 prevotes were seen in `POLRound`, so this does not represent a change to the prevote logic.

A validator will also check that the proposed timestamp is greater than the timestamp of the block for the previous height.
If the timestamp is not greater than the previous block's timestamp, the block will not be considered valid, which is the same as the current logic.

Additionally, this validation logic can be updated to check that the `Proposal.Timestamp` matches the `Header.Timestamp` of the proposed block, but it is less relevant since checking that votes were received is sufficient to ensure the block timestamp is correct.

#### Relaxation of the 'Timely' check

The `Synchrony` parameters, `MessageDelay` and `Precision` provide a means to bound the timestamp of a proposed block.
Selecting values that are too small presents a possible liveness issue for the network.
If a Tendermint network selects a `MessageDelay` parameter that does not accurately reflect the time to broadcast a proposal message to all of the validators on the network, nodes will begin rejecting proposals from otherwise correct proposers because these proposals will appear to be too far in the past.

`MessageDelay` and `Precision` are planned to be configured as `ConsensusParams`.
A very common way to update `ConsensusParams` is by executing a transaction included in a block that specifies new values for them.
However, if the network is unable to produce blocks because of this liveness issue, no such transaction may be executed.
To prevent this dangerous condition, we will add a relaxation mechanism to the `Timely` predicate.
If consensus takes more than 10 rounds to produce a block for any reason, the `MessageDelay` will be doubled.
This doubling will continue for each subsequent 10 rounds of consensus.
This will enable chains that selected too small of a value for the `MessageDelay` parameter to eventually issue a transaction and readjust the parameters to more accurately reflect the broadcast time.

This liveness issue is not as problematic for chains with very small `Precision` values.
Operators can more easily readjust local validator clocks to be more aligned.
Additionally, chains that wish to increase a small `Precision` value can still take advantage of the `MessageDelay` relaxation, waiting for the `MessageDelay` value to grow significantly and issuing proposals with timestamps that are far in the past of their peers.

For more discussion of this, see [issue 371](https://github.com/tendermint/spec/issues/371).

### Changes to the prevote step

Currently, a validator will prevote a proposal in one of three cases:

* Case 1:  Validator has no locked block and receives a valid proposal.
* Case 2:  Validator has a locked block and receives a valid proposal matching its locked block.
* Case 3:  Validator has a locked block, sees a valid proposal not matching its locked block but sees +2/3 prevotes for the proposal’s block, either in the current round or in a round greater than or equal to the round in which it locked its locked block.

The only change we will make to the prevote step is to what a validator considers a valid proposal as detailed above.

### Changes to the precommit step

The precommit step will not require much modification.
Its proposal validation rules will change in the same ways that validation will change in the prevote step with the exception of the `timely` check: precommit validation will never check that the timestamp is `timely`.

### Remove voteTime Completely

[voteTime](https://github.com/tendermint/tendermint/blob/822893615564cb20b002dd5cf3b42b8d364cb7d9/internal/consensus/state.go#L2229) is a mechanism for calculating the next `BFTTime` given both the validator's current known Unix time and the previous block timestamp.
If the previous block timestamp is greater than the validator's current known Unix time, then voteTime returns a value one millisecond greater than the previous block timestamp.
This logic is used in multiple places and is no longer needed for proposer-based timestamps.
It should therefore be removed completely.

## Future Improvements

* Implement BLS signature aggregation.
By removing fields from the `Precommit` messages, we are able to aggregate signatures.

## Consequences

### Positive

* `<2/3` of validators can no longer influence block timestamps.
* Block timestamp will have stronger correspondence to real time.
* Improves the reliability of light client block verification.
* Enables BLS signature aggregation.
* Enables evidence handling to use time instead of height for evidence validity.

### Neutral

* Alters Tendermint’s liveness properties.
Liveness now requires that all correct validators have synchronized clocks within a bound.
Liveness will now also require that validators’ clocks move forward, which was not required under `BFTTime`.

### Negative

* May increase the length of the propose step if there is a large skew between the previous proposer and the current proposer’s local Unix time.
This skew will be bound by the `PRECISION` value, so it is unlikely to be too large.

* Current chains with block timestamps far in the future will either need to pause consensus until after the erroneous block timestamp or must maintain synchronized but very inaccurate clocks.

## References

* [PBTS Spec](https://github.com/tendermint/tendermint/tree/main/spec/consensus/proposer-based-timestamp)
* [BFTTime spec](https://github.com/tendermint/tendermint/blob/main/spec/consensus/bft-time.md)
* [Issue 371](https://github.com/tendermint/spec/issues/371)
