# ADR 069: `x/gov` modularity, multiple choice and optimistic proposals

## Changelog

* 2023-11-17: Initial draft (@julienrbrt, @tac0turtle)

## Status

ACCEPTED - Implemented

## Abstract

Governance is an important aspect of Cosmos SDK chains.

This ADR aimed to extend the `x/gov` module functionalities by adding two different kinds of proposals, as well as making `x/gov` more composable and extendable.

Those two types are, namely: multiple choice proposals and optimistic proposals.

## Context

`x/gov` is the center of Cosmos governance, and has already been improved from its first version `v1beta1`, with a second version [`v1`][5].
This second iteration of gov unlocked many possibilities by letting governance proposals contain any number of proposals.
The last addition of gov has been expedited proposals (proposals that have a shorter voting period and a higher quorum, approval threshold).

The community requested ([1], [4]) two additional proposals for improving governance choices. Those proposals would be useful when having protocol decisions made on specific choices or simplifying regular proposals that do not require high community involvement.

Additionally, the SDK should allow chains to customize the tallying method of proposals (if they want to count the votes in another way). Currently, the Cosmos SDK counts votes proportionally to the voting power/stake. However, custom tallying could allow counting votes with a quadratic function instead.

## Decision

`x/gov` will integrate these functions and extract helpers and interfaces for extending the `x/gov` module capabilities.

### Proposals

Currently, all proposals are [`v1.Proposal`][5]. Optimistic and multiple choice proposals require a different tally logic, but the rest of the proposal stays the same to not create other proposal types, `v1.Proposal` will have an extra field:

```protobuf
// ProposalType enumerates the valid proposal types.
// All proposal types are v1.Proposal which have different voting periods or tallying logic.
enum ProposalType {
  // PROPOSAL_TYPE_UNSPECIFIED defines no proposal type, which fallback to PROPOSAL_TYPE_STANDARD.
  PROPOSAL_TYPE_UNSPECIFIED = 0;
  // PROPOSAL_TYPE_STANDARD defines the type for a standard proposal.
  PROPOSAL_TYPE_STANDARD = 1;
  // PROPOSAL_TYPE_MULTIPLE_CHOICE defines the type for a multiple choice proposal.
  PROPOSAL_TYPE_MULTIPLE_CHOICE = 2;
  // PROPOSAL_TYPE_OPTIMISTIC defines the type for an optimistic proposal.
  PROPOSAL_TYPE_OPTIMISTIC = 3;
  // PROPOSAL_TYPE_EXPEDITED defines the type for an expedited proposal.
  PROPOSAL_TYPE_EXPEDITED = 4;
}
```

Note, that expedited becomes a proposal type itself instead of a boolean on the `v1.Proposal` struct.

> An expedited proposal is by design a standard proposal with a quicker voting period and higher threshold. When an expedited proposal fails, it gets converted to a standard proposal.   

An expedited optimistic proposal and an expedited multiple choice proposal do not make sense based on the definition above and is a proposal type instead of a proposal characteristic.

#### Optimistic Proposal

An optimistic proposal is a proposal that passes unless a threshold a NO votes is reached.

Voter can only vote NO on the proposal. If the NO threshold is reached, the optimistic proposal is converted to a standard proposal.

Two governance parameters will be in added [`v1.Params`][5] to support optimistic proposals:

```protobuf
// optimistic_authorized_addreses is an optional governance parameter that limits the authorized accounts that can submit optimistic proposals
repeated string optimistic_authorized_addreses = 17 [(cosmos_proto.scalar) = "cosmos.AddressString"];

// Optimistic rejected threshold defines at which percentage of NO votes, the optimistic proposal should fail and be converted to a standard proposal.
string optimistic_rejected_threshold = 18 [(cosmos_proto.scalar) = "cosmos.Dec"];
```

#### Multiple Choice Proposal

A multiple choice proposal is a proposal where the voting options can be defined by the proposer.

The number of voting options will be limited to a maximum of 4.
A new vote option `SPAM` will be added and distinguished from those voting options. `SPAM` will be used to mark a proposal as spam and is explained further below.

Multiple choice proposals, contrary to any other proposal type, cannot have messages to execute. They are only text proposals.

Submitting a new multiple choice proposal will use a different message than the [`v1.MsgSubmitProposal`][5]. This is done in order to simplify the proposal submission and allow defining the voting options directly.


```protobuf
message MsgSubmitMultipleChoiceProposal {
  repeated cosmos.base.v1beta1.Coin initial_deposit = 1;
  string proposer = 2 [(cosmos_proto.scalar) = "cosmos.AddressString"];
  string metadata = 3;
  string title = 4;
  string summary = 5;
  string option_one = 6;
  string option_two = 7;
  string option_three = 8;
  string option_four = 9;
}
```

Voters can only vote on the defined options in the proposal.

To maintain compatibility with the existing endpoints, the voting options will not be stored in the proposal itself and each option will be mapped to [`v1.VoteOption`][5]. A multiple choice proposal will be stored as a [`v1.Proposal`][5]. A query will be available for multiple choice proposal types to get the voting options.

### Votes

As mentioned above [multiple choice proposal](#multiple-choice-proposal) will introduce an additional vote option: `SPAM`.

This vote option will be supported by all proposal types.
At the end of the voting period, if a proposal is voted as `SPAM`, it fails and its deposit is burned.

`SPAM` differs from the `No with Veto` vote as its threshold is dynamic.
A proposal is marked as `SPAM` when the total of weighted votes for all options is lower than the amount of weighted vote on `SPAM`
(`spam` > `option_one + option_two + option_three + option_four` = proposal marked as spam).
This allows clear spam proposals to be marked as spam easily, even with low participation from validators.

To avoid voters wrongfully voting down a proposal as `SPAM`, voters will be slashed `x`% (default 0%) of their voting stake if they voted `SPAM` on a proposal that wasn't a spam proposal. The parameter allows to incentivise voters to only vote `SPAM` on actual spam proposals and not use `SPAM` as a way to vote `No with Veto` with a different threshold.

This leads to the addition of the following governance parameter in [`v1.Params`][5]:

```protobuf
// burn_spam_amount defines the percentage of the voting stake that will be burned if a voter votes SPAM on a proposal that is not marked as SPAM.
string burn_spam_amount = 8 [(cosmos_proto.scalar) = "cosmos.Dec"];
```

Additionally, the current vote options will be aliased to better accommodate the multiple choice proposal:

```protobuf
// VoteOption enumerates the valid vote options for a given governance proposal.
enum VoteOption {
  option allow_alias = true;

  // VOTE_OPTION_UNSPECIFIED defines a no-op vote option.
  VOTE_OPTION_UNSPECIFIED = 0;
  // VOTE_OPTION_ONE defines the first proposal vote option.
  VOTE_OPTION_ONE = 1;
  // VOTE_OPTION_YES defines the yes proposal vote option.
  VOTE_OPTION_YES = 1;
  // VOTE_OPTION_TWO defines the second proposal vote option.
  VOTE_OPTION_TWO = 2;
  // VOTE_OPTION_ABSTAIN defines the abstain proposal vote option.
  VOTE_OPTION_ABSTAIN = 2;
  // VOTE_OPTION_THREE defines the third proposal vote option.
  VOTE_OPTION_THREE = 3;
  // VOTE_OPTION_NO defines the no proposal vote option.
  VOTE_OPTION_NO = 3;
  // VOTE_OPTION_FOUR defines the fourth proposal vote option.
  VOTE_OPTION_FOUR = 4;
  // VOTE_OPTION_NO_WITH_VETO defines the no with veto proposal vote option.
  VOTE_OPTION_NO_WITH_VETO = 4;
  // VOTE_OPTION_SPAM defines the spam proposal vote option.
  VOTE_OPTION_SPAM = 5;
}
```

The order does not change for a standard proposal (1 = yes, 2 = abstain, 3 = no, 4 = no with veto as it was) and the aliased enum can be used interchangeably.

Updating vote options means updating [`v1.TallyResult`][5] as well.

#### Tally

Due to the vote option change, each proposal can have the same tallying method.

However, chains may want to change the tallying function (weighted vote per voting power) of `x/gov` for a different algorithm (using a quadratic function on the voter stake, for instance).

The custom tallying function can be passed to the `x/gov` keeper config:

```go
type CalculateVoteResultsAndVotingPowerFn func(
	ctx context.Context,
	keeper Keeper,
	proposalID uint64,
	validators map[string]v1.ValidatorGovInfo,
) (totalVoterPower math.LegacyDec, results map[v1.VoteOption]math.LegacyDec, err error)
```

## Consequences

Changing voting possibilities has a direct consequence for the clients. Clients, like Keplr or Mintscan, need to implement logic for multiple choice proposals.

That logic consists of querying multiple choice proposals vote mapping their vote options.

### Backwards Compatibility

Legacy proposals (`v1beta1`) endpoints will not be supporting the new proposal types.

Voting on a gov v1 proposal having a different type than [`standard` or `expedited`](#proposals) via the `v1beta1` will not be supported.
This is already the case for the expedited proposals.

### Positive

* Extended governance features
* Extended governance customization

### Negative

* Increase gov wiring complexity

### Neutral

* Increases the number of parameters available

## Further Discussions

This ADR starts the `x/gov` overhaul for the `cosmossdk.io/x/gov` v1.0.0 release.
Further internal improvements of `x/gov` will happen soon after, in order to simplify its state management and making gov calculation in a more "lazy"-fashion.

Those improvements may change the tallying api.

* https://github.com/cosmos/cosmos-sdk/issues/16270

## References

* [https://github.com/cosmos/cosmos-sdk/issues/16270][1]
* [https://github.com/cosmos/cosmos-sdk/issues/17781][2]
* [https://github.com/cosmos/cosmos-sdk/issues/14403][3]
* [https://github.com/decentralists/DAO/issues/28][4]

[1]: https://grants.osmosis.zone/blog/rfp-cosmos-sdk-governance-module-improvements
[2]: https://github.com/cosmos/cosmos-sdk/issues/17781
[3]: https://github.com/cosmos/cosmos-sdk/issues/14403
[4]: https://github.com/decentralists/DAO/issues/28
[5]: https://buf.build/cosmos/cosmos-sdk/docs/main:cosmos.gov.v1
