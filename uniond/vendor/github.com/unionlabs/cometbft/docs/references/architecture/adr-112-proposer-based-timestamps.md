# ADR 112: Proposer-Based Timestamps

## Changelog

 - July 15 2021: Created by @williambanfield
 - Aug 4 2021: Draft completed by @williambanfield
 - Aug 5 2021: Draft updated to include data structure changes by @williambanfield
 - Aug 20 2021: Language edits completed by @williambanfield
 - Oct 25 2021: Update the ADR to match updated spec from @cason by @williambanfield
 - Nov 10 2021: Additional language updates by @williambanfield per feedback from @cason
 - Feb 2 2022: Synchronize logic for timely with latest version of the spec by @williambanfield
 - Feb 1 2024: Renamed to ADR 112 as basis for its adoption ([#1731](https://github.com/cometbft/cometbft/issues/1731)) in CometBFT v1.0 by @cason
 - Feb 7 2024: Multiple revisions, fixes, and backwards compatibility discussion by @cason
 - Feb 12 2024: More detailed backwards compatibility discussion by @cason
 - Feb 22 2024: Consensus parameters for backwards compatibility by @cason

## Status

**Accepted**

## Context

CometBFT currently provides a monotonically increasing source of time known as [BFT Time][bfttime].
This mechanism for producing a source of time is reasonably simple.
Each validator adds a timestamp to each `Precommit` message it sends.
The timestamp a correct validator sends is either the validator's current known Unix time or one millisecond greater than the previous block time, depending on which value is greater.
When a block is produced, the proposer chooses the block timestamp as the weighted median of the times in all of the `Precommit` messages the proposer received.
The weighting is defined by the amount of voting power, or stake, each validator has on the network.
This mechanism for producing timestamps is both deterministic and Byzantine fault tolerant.

This current mechanism for producing timestamps has a few drawbacks.
Validators do not have to agree at all on how close the selected block timestamp is to their own currently known Unix time.
Additionally, any amount of voting power `>1/3` may control the block timestamp.
As a result, it is quite possible that the timestamp is not particularly meaningful.

These drawbacks present issues in CometBFT.
Timestamps are used by light clients to verify blocks.
Light clients rely on correspondence between their own currently known Unix time and the block timestamp to verify blocks they see.
However, their currently known Unix time may be greatly divergent from the block timestamp as a result of the limitations of `BFT Time`.

The [Proposer-Based Timestamps specification (PBTS)][pbts-spec] suggests an alternative approach for producing block timestamps that remedies these issues.
Proposer-based timestamps alter the current mechanism for producing block timestamps in two main ways:

1. The block proposer is amended to offer up its currently known Unix time as the timestamp for the next block instead of the `BFT Time`.
1. Correct validators are assumed to be equipped with synchronized clocks and only approve the proposed block timestamp if it is close enough to their own currently known Unix time.

The result of these changes is a more meaningful timestamp that cannot be controlled by `<= 2/3` of the validator voting power.
This document outlines the necessary code changes in CometBFT to implement the corresponding [specification][pbts-spec].

## Alternative Approaches

### Remove timestamps altogether

Computer clocks are bound to skew for a variety of reasons.
Using timestamps in our protocol means either accepting the timestamps as not reliable or impacting the protocol’s liveness guarantees.
This design requires impacting the protocol’s liveness in order to make the timestamps more reliable.
An alternate approach is to remove timestamps altogether from the block protocol.
`BFT Time` is deterministic but may be arbitrarily inaccurate.
However, having a reliable source of time is quite useful for applications and protocols built on top of a blockchain.

We therefore decided not to remove the timestamp.
Applications often wish for some transactions to occur on a certain day, on a regular period, or after some time following a different event.
All of these require some meaningful representation of agreed upon time.
The following protocols and application features require a reliable source of time:

* Light Clients [rely on correspondence between their known time](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/README.md#failure-model) and the block time for block verification.
* Evidence validity is determined [either in terms of heights or in terms of time](https://github.com/cometbft/cometbft/blob/main/spec/consensus/evidence.md#verification).
* Unbonding of staked assets in the Cosmos Hub [occurs after a period of 21 days](https://github.com/cosmos/governance/blob/ce75de4019b0129f6efcbb0e752cd2cc9e6136d3/params-change/Staking.md#unbondingtime).
* IBC packets can use either a [timestamp or a height to timeout packet delivery](https://docs.cosmos.network/v0.45/ibc/overview.html#acknowledgements)

Finally, inflation distribution in the Cosmos Hub uses an approximation of time to calculate an annual percentage rate.
This approximation of time is calculated using [block heights with an estimated number of blocks produced in a year](https://github.com/cosmos/governance/blob/master/params-change/Mint.md#blocksperyear).
Proposer-based timestamps will allow this inflation calculation to use a more meaningful and accurate source of time.

## Decision

Implement Proposer-Based Timestamps while maintaining backwards compatibility with `BFT Time`.

## Detailed Design

### Overview

Implementing Proposer-Based Timestamps (PBTS) will require a few changes to CometBFT’s code.
These changes will be to the following components:

* The consensus parameters.
* The `internal/consensus/` package.
* The `internal/state/` package.

The original version of this document ([ADR 071][original-adr]) dir not
consider that the introduced `PBTS` and the previous method `BFT Time` could 
be adopted in the same chain/network.
The [backwards compatibility](#backwards-compatibility) section below was thus
added to address topic.

<!---
### Changes to `CommitSig`

The [CommitSig](https://github.com/cometbft/cometbft/blob/a419f4df76fe4aed668a6c74696deabb9fe73211/types/block.go#L604) struct currently contains a timestamp.
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

`Precommit` and `Prevote` messages use a common [Vote struct](https://github.com/cometbft/cometbft/blob/a419f4df76fe4aed668a6c74696deabb9fe73211/types/vote.go#L50).
This struct currently contains a timestamp.
This timestamp is set using the [voteTime](https://github.com/cometbft/cometbft/blob/e8013281281985e3ada7819f42502b09623d24a0/internal/consensus/state.go#L2241) function and therefore vote times correspond to the current Unix time known to the validator, provided this time is greater than the timestamp of the previous block.
For precommits, this timestamp is used to construct the [CommitSig that is included in the block in the LastCommit](https://github.com/cometbft/cometbft/blob/e8013281281985e3ada7819f42502b09623d24a0/types/block.go#L754) field.
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
--->

### Backwards compatibility

In order to ensure backwards compatibility, PBTS should be enabled using a [consensus parameter](#compatibility-parameters).
The proposed approach is similar to the one adopted to enable vote extensions via
[`VoteExtensionsEnableHeight`](https://github.com/cometbft/cometbft/blob/main/spec/abci/abci++_app_requirements.md#featureparamsvoteextensionsenableheight).

In summary, the network will migrate from the `BFT Time` method for assigning
and validating timestamps to the new method for assigning and validating
timestamps adopted by `PBTS` from a given, configurable height.
Once `PBTS` is activated, there are no provisions for the network to revert
back to `BFT Time` (see [issue 2063][issue2063]).

Moreover, when compared to the original ([ADR 071][original-adr]), we will **NOT**:

- Update `CommitSigs` and `Vote` types, removing the `Timestamp` field
- Remove the `MedianTime` method used by `BFT Time` to produce and validate the block's time
- Remove the `voteTime` method used by `BFT Time` to set timestamps to precommits
- Remove the [validation logic](#current-block-time-validation-logic) used by `BFT Time`

### New consensus parameters

The PBTS specification includes some new parameters that must be the same among across all validators.
The set of [consensus parameters](https://github.com/cometbft/cometbft/blob/main/proto/cometbft/types/v1/params.proto#L13)
will be updated to include new fields as follows:

```diff
type ConsensusParams struct {
        Block     BlockParams     `json:"block"`
        Evidence  EvidenceParams  `json:"evidence"`
        Validator ValidatorParams `json:"validator"`
        Version   VersionParams   `json:"version"`
        ABCI      ABCIParams      `json:"abci"`
++      Synchrony SynchronyParams `json:"synchrony"`
++      Feature   FeatureParams   `json:"feature"`
}
```

#### Synchrony parameters

The `PRECISION` and `MSGDELAY` parameters are used to determine if the proposed timestamp is acceptable.
A validator will only Prevote a proposal if the proposal timestamp is considered `timely`.
A proposal timestamp is considered `timely` if it is within `PRECISION` and `MSGDELAY` of the Unix time known to the validator.
More specifically, the timestamp of a proposal received at `proposalReceiveTime` is `timely` if

    proposalTimestamp - PRECISION ≤ proposalReceiveTime ≤ proposalTimestamp + PRECISION + MSGDELAY

`PRECISION` and `MSGDELAY` will be added to the consensus synchrony parameters as [durations](https://protobuf.dev/reference/protobuf/google.protobuf/#duration):

```go
type SynchronyParams struct {
        Precision    time.Duration `json:"precision,string"`
        MessageDelay time.Duration `json:"message_delay,string"`
}
```

#### Compatibility parameters

In order to ensure backwards compatibility, PBTS should be enabled using a consensus parameter:

```go
type FeatureParams struct {
        PbtsEnableHeight int64 `json:"pbts_enable_height"`
        ...
}
```

The semantics are similar to the ones adopted to enable vote extensions via
[`VoteExtensionsEnableHeight`](https://github.com/cometbft/cometbft/blob/main/spec/abci/abci++_app_requirements.md#abciparamsvoteextensionsenableheight).
The PBTS algorithm is enabled from `FeatureParams.PbtsEnableHeight`, when this
parameter is set to a value greater than zero, and greater to the height at
which it was set.
Until that height, the BFT Time algorithm is used.

For more discussion of this, see [issue 2197][issue2197].


### Changes to the block proposal step

#### Proposer selects block timestamp

CometBFT currently uses the `BFT Time` algorithm to produce the block's `Header.Timestamp`.
The [block production logic](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/internal/state/state.go#L248)
sets the weighted median of the times in the `LastCommit.CommitSigs` as the proposed block's `Header.Timestamp`.
This method will be preserved, but it is only used while operating in `BFT Time` mode.

In PBTS, the proposer will still set a timestamp into the `Header.Timestamp`.
The timestamp the proposer sets into the `Header` will change depending on whether the block has previously received `2/3+` prevotes in a previous round.
Receiving +2/3 prevotes in a round is frequently referred to as a 'Polka' and we will use this term for simplicity.

#### Proposal of a block that has not previously received a Polka

If a proposer is proposing a new block then it will set the Unix time currently known to the proposer into the `Header.Timestamp` field.
The proposer will also set this same timestamp into the `Timestamp` field of the `Proposal` message that it issues.

#### Re-proposal of a block that has previously received a Polka

If a proposer is re-proposing a block that has previously received a Polka on the network, then the proposer does not update the `Header.Timestamp` of that block.
Instead, the proposer simply re-proposes the exact same block.
This way, the proposed block has the exact same block ID as the previously proposed block and the nodes that have already received that block do not need to attempt to receive it again.

The proposer will set the re-proposed block's `Header.Timestamp` as the `Proposal` message's `Timestamp`.

#### Proposer waits

Block timestamps must be monotonically increasing.
In `BFT Time`, if a validator’s clock was behind, the [validator added 1 millisecond to the previous block’s time and used that in its vote messages](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/internal/consensus/state.go#L2460).
A goal of adding PBTS is to enforce some degree of clock synchronization, so having a mechanism that completely ignores the Unix time of the validator time no longer works.
Validator clocks will not be perfectly in sync.
Therefore, the proposer’s current known Unix time may be less than the previous block's `Header.Time`.
If the proposer’s current known Unix time is less than the previous block's `Header.Time`, the proposer will sleep until its known Unix time exceeds it.

This change will require amending the [`defaultDecideProposal`](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/internal/consensus/state.go#L1195) method.
This method should now schedule a timeout that fires when the proposer’s time is greater than the previous block's `Header.Time`.
When the timeout fires, the proposer will finally issue the `Proposal` message.

### Changes to proposal validation rules

The rules for validating a proposed block will be modified to implement PBTS.
We will change the validation logic to ensure that a proposal is `timely`.
The `timely` verification is adopted once the node enabled PBTS.

Per the PBTS spec, `timely` only needs to be checked if a block has not received a Polka in a previous round.
If a block previously received a +2/3 majority of prevotes in a round, then +2/3 of the voting power considered the block's timestamp near enough to their own currently known Unix time in that round.

The validation logic will be updated to check `timely` for blocks that did not previously receive a Polka in a round.

#### Timestamp validation when a block has not received a Polka

The [`POLRound`](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/types/proposal.go#L29) in the `Proposal` message indicates which round the block received a Polka.
A negative value in the `POLRound` field indicates that the block has not previously been proposed on the network.
Therefore the validation logic will check for timely when `POLRound == -1`.

When a node receives a `Proposal` message, it records it `proposalReceiveTime` as the current Unix time known to the node.
The node will check that the `Proposal.Timestamp` is at most `PRECISION` greater than `proposalReceiveTime`, and at maximum `PRECISION + MSGDELAY` less than `proposalReceiveTime`.
If the timestamp is not within these bounds, the proposed block will not be considered `timely`.
A validator prevotes nil when the proposed block is not considered `timely`.

Once a full block matching the `Proposal` message is received, the node will also check that the timestamp in the `Header.Timestamp` of the block matches this `Proposal.Timestamp`.
Using the `Proposal.Timestamp` to check `timely` allows for the `MSGDELAY` parameter to be more finely tuned since `Proposal` messages do not change sizes and are therefore faster to gossip than full blocks across the network.

A node will also check that the proposed timestamp is greater than the timestamp of the block for the previous height.
If the timestamp is not greater than the previous block's timestamp, the block will not be considered valid, which is the same as the current logic.

#### Timestamp validation when a block has received a Polka

When a block is re-proposed that has already received a +2/3 majority of `Prevote`s (i.e., a Polka) on the network, the `Proposal` message for the re-proposed block is created with a `POLRound` that is `>= 0`.
A node will not check that the `Proposal` is `timely` if the proposal message has a non-negative `POLRound`.
If the `POLRound` is non-negative, each node (although this is only relevant for validators) will simply ensure that it received the `Prevote` messages for the proposed block in the round indicated by `POLRound`.

If the node is a validator and it does not receive `Prevote` messages for the proposed block before the proposal timeout, then it will prevote nil.
Validators already check that +2/3 prevotes were seen in `POLRound`, so this does not represent a change to the prevote logic.

A node will also check that the proposed timestamp is greater than the timestamp of the block for the previous height.
If the timestamp is not greater than the previous block's timestamp, the block will not be considered valid, which is the same as the current logic.

Additionally, this validation logic can be updated to check that the `Proposal.Timestamp` matches the `Header.Timestamp` of the proposed block, but it is less relevant since checking that votes were received is sufficient to ensure the block timestamp is correct.

#### Relaxation of the 'Timely' check

The `Synchrony` parameters, `MessageDelay` and `Precision` provide a means to bound the timestamp of a proposed block.
Selecting values that are too small presents a possible liveness issue for the network.
If a CometBFT network selects a `MessageDelay` parameter that does not accurately reflect the time to broadcast a proposal message to all of the validators on the network, validators will begin rejecting proposals from otherwise correct proposers because these proposals will appear to be too far in the past.

`MessageDelay` and `Precision` are planned to be configured as `ConsensusParams`.
A very common way to update `ConsensusParams` is by executing a transaction included in a block that specifies new values for them.
However, if the network is unable to produce blocks because of this liveness issue, no such transaction may be executed.
To prevent this dangerous condition, we will add a relaxation mechanism to the `Timely` predicate.

The chosen solution for this issue is to adopt the configured `MessageDelay`
for the first round (0) of consensus.
Then, as more rounds are needed to commit a value, we increase the
adopted value for `MessageDelay`, at a rate of 10% per additional round.
More precisely, the `MessageDelay(r)` adopted for round `r` of consensus is
given by `MessageDelay(r) = MessageDelay * (1.1)^r`.
Of course, `MessageDelay(0) = MessageDelay`.

This liveness issue is not as problematic for chains with very small `Precision` values.
Operators can more easily readjust local validator clocks to be more aligned.
Additionally, chains that wish to increase a small `Precision` value can still take advantage of the `MessageDelay` relaxation, waiting for the `MessageDelay` value to grow significantly and issuing proposals with timestamps that are far in the past of their peers.

For more discussion of this, see [issue 2184][issue2184].

### Changes to the prevote step

Currently, a validator will prevote a proposal in one of three cases:

* Case 1:  Validator has no locked block and receives a valid proposal.
* Case 2:  Validator has a locked block and receives a valid proposal matching its locked block.
* Case 3:  Validator has a locked block, sees a valid proposal not matching its locked block but sees +2/3 prevotes for the proposal’s block, either in the current round or in a round greater than or equal to the round in which it locked its locked block.

The only change we will make to the prevote step is to what a validator considers a valid proposal as detailed above.

### Changes to the precommit step

The precommit step will not require much modification.
Its proposal validation rules will change in the same ways that validation will change in the prevote step with the exception of the `timely` check: precommit validation will never check that the timestamp is `timely`.

<!---
### Remove voteTime Completely

[voteTime](https://github.com/cometbft/cometbft/blob/822893615564cb20b002dd5cf3b42b8d364cb7d9/internal/consensus/state.go#L2229) is a mechanism for calculating the next `BFT Time` given both the validator's current known Unix time and the previous block timestamp.
If the previous block timestamp is greater than the validator's current known Unix time, then voteTime returns a value one millisecond greater than the previous block timestamp.
This logic is used in multiple places and is no longer needed for PBTS.
It should therefore be removed completely.
--->

### Changes to the block validation

To provide a better understanding of the changes needed for timestamp validation, we first detail how timestamp validation works currently with BFT Time,
then presents how it will work with PBTS.

#### Current block time validation logic

The [`validateBlock` function](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/internal/state/validation.go#L15) currently [validates the proposed block timestamp in three ways](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/internal/state/validation.go#L116).
First, the validation logic checks that this timestamp is greater than the previous block’s timestamp.

Second, it validates that the block timestamp is correctly calculated as the weighted median of the timestamps in the [block’s `LastCommit`](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/types/block.go#L49).

Finally, the validation logic authenticates the timestamps in the `LastCommit.CommitSig`.
The cryptographic signature in each `CommitSig` is created by signing a hash of fields in the block with the voting validator’s private key.
One of the items in this `signedBytes` hash is the timestamp in the `CommitSig`.
To authenticate the `CommitSig` timestamp, the node authenticating votes builds a hash of fields that includes the `CommitSig` timestamp and checks this hash against the signature.
This takes place in the [`VerifyCommit` function](https://github.com/cometbft/cometbft/blob/1f430f51f0e390cd7c789ba9b1e9b35846e34642/types/validation.go#L26).

<!---
#### Remove unused timestamp validation logic

`BFT Time` validation is no longer applicable and will be removed.
This means that validators will no longer check that the block timestamp is a weighted median of `LastCommit` timestamps.
Specifically, we will remove the call to [MedianTime in the validateBlock function](https://github.com/cometbft/cometbft/blob/4db71da68e82d5cb732b235eeb2fd69d62114b45/state/validation.go#L117).
The `MedianTime` function can be completely removed.

Since `CommitSig`s will no longer contain a timestamp, the validator authenticating a commit will no longer include the `CommitSig` timestamp in the hash of fields it builds to check against the cryptographic signature.
--->

#### PBTS block time validation

PBTS does not perform a validation of the timestamp of a block, as part of the `validateBlock` method.
This means that nodes will no longer check that the block time is a weighted median of `LastCommit` timestamps.

Instead of validating the timestamp of proposed blocks,
PBTS validates the timestamp of the `Proposal` message for a block, as detailed [here](#changes-to-proposal-validation-rules).
Notice that the `Proposal` timestamp must match the proposed block's `Time` field.

This also means that committed blocks, retrieved from peers via consensus catch-up mechanisms or via block sync,
will not have their timestamps validated, since the timestamp validation is now part of the consensus logic.


## Future Improvements

* Implement BLS signature aggregation.
If we remove the `Timestamp` field from the `Precommit` messages, we are able to aggregate signatures,
as votes for the same block, height and round become identical.

We have left the removal of the `Timestamp` field of vote messages out for the time being, as it would break the block format and validation
rules (signature verification) and thus may force a hard-fork on chains upgrading to the latest version of CometBFT.
We will remove the timestamps in votes when changing the block format is supported in CometBFT without
requiring a hard-fork (this feature is called [Soft Upgrades](https://github.com/cometbft/cometbft/issues/122)).

## Consequences

### Positive

* `<2/3` of validators can no longer arbitrarily influence block timestamps.
* Block timestamps will have stronger correspondence to real time.
* Improves the reliability of [components](#remove-timestamps-altogether) that rely on block timestamps:
  Light Client verification, Evidence validity, Unbonding of staked assets, IBC packet timeouts, inflation distribution, etc.
* It is a step towards enabling BLS signature aggregation.

### Neutral

* Alters the liveness requirements for the consensus algorithm.
Liveness now requires that all correct validators have synchronized clocks, with inaccuracy bound by `PRECISION`,
and that end-to-end delays of `PROPOSAL` messages are bound by `MSGDELAY`.

### Negative

* May increase the duration of the propose step if there is a large skew between the clocks of the previous proposer and the current proposer.
The clock skew between correct validators is supposed to be bound by `PRECISION`, so this impact is relevant when block times are shorter than `PRECISION`.
* Existing chains that adopt PBTS may have block times far in the future, which may cause the transition height to have a very long duration (to preserve time monotonicity).
The workaround in this case is, first, to synchronize the validators' clocks, then to maintain the legacy operation (using BFT Time), until block times align with real time.
At this point, the transition from BFT Time to PBTS should be smooth.

## References

* [PBTS Spec][pbts-spec]
* [BFT Time spec][bfttime]
* [PBTS: support both PBTS and legacy BFT Time #2063][issue2063]
* [PBTS: should synchrony parameters be adaptive? #2184][issue2184]

[issue2184]: https://github.com/cometbft/cometbft/issues/2184
[issue2197]: https://github.com/cometbft/cometbft/issues/2197
[issue2063]: https://github.com/cometbft/cometbft/issues/2063
[bfttime]: https://github.com/cometbft/cometbft/blob/main/spec/consensus/bft-time.md
[pbts-spec]: https://github.com/cometbft/cometbft/tree/main/spec/consensus/proposer-based-timestamp/README.md
[original-adr]: https://github.com/cometbft/cometbft/blob/main/docs/references/architecture/tendermint-core/adr-071-proposer-based-timestamps.md
