# ADR 037: Governance split votes

## Changelog

* 2020/10/28: Initial draft

## Status

Accepted

## Abstract

This ADR defines a modification to the governance module that would allow a staker to split their votes into several voting options. For example, it could use 70% of its voting power to vote Yes and 30% of its voting power to vote No.

## Context

Currently, an address can cast a vote with only one options (Yes/No/Abstain/NoWithVeto) and use their full voting power behind that choice.

However, often times the entity owning that address might not be a single individual.  For example, a company might have different stakeholders who want to vote differently, and so it makes sense to allow them to split their voting power.  Another example use case is exchanges.  Many centralized exchanges often stake a portion of their users' tokens in their custody.  Currently, it is not possible for them to do "passthrough voting" and giving their users voting rights over their tokens.  However, with this system, exchanges can poll their users for voting preferences, and then vote on-chain proportionally to the results of the poll.

## Decision

We modify the vote structs to be

```go
type WeightedVoteOption struct {
  Option string
  Weight sdk.Dec
}

type Vote struct {
  ProposalID int64
  Voter      sdk.Address
  Options    []WeightedVoteOption
}
```

And for backwards compatibility, we introduce `MsgVoteWeighted` while keeping `MsgVote`.

```go
type MsgVote struct {
  ProposalID int64
  Voter      sdk.Address
  Option     Option
}

type MsgVoteWeighted struct {
  ProposalID int64
  Voter      sdk.Address
  Options    []WeightedVoteOption
}
```

The `ValidateBasic` of a `MsgVoteWeighted` struct would require that

1. The sum of all the Rates is equal to 1.0
2. No Option is repeated

The governance tally function will iterate over all the options in a vote and add to the tally the result of the voter's voting power * the rate for that option.

```go
tally() {
    results := map[types.VoteOption]sdk.Dec

    for _, vote := range votes {
        for i, weightedOption := range vote.Options {
            results[weightedOption.Option] += getVotingPower(vote.voter) * weightedOption.Weight
        }
    }
}
```

The CLI command for creating a multi-option vote would be as such:

```shell
simd tx gov vote 1 "yes=0.6,no=0.3,abstain=0.05,no_with_veto=0.05" --from mykey
```

To create a single-option vote a user can do either

```shell
simd tx gov vote 1 "yes=1" --from mykey
```

or

```shell
simd tx gov vote 1 yes --from mykey
```

to maintain backwards compatibility.

## Consequences

### Backwards Compatibility

* Previous VoteMsg types will remain the same and so clients will not have to update their procedure unless they want to support the WeightedVoteMsg feature.
* When querying a Vote struct from state, its structure will be different, and so clients wanting to display all voters and their respective votes will have to handle the new format and the fact that a single voter can have split votes.
* The result of querying the tally function should have the same API for clients.

### Positive

* Can make the voting process more accurate for addresses representing multiple stakeholders, often some of the largest addresses.

### Negative

* Is more complex than simple voting, and so may be harder to explain to users.  However, this is mostly mitigated because the feature is opt-in.

### Neutral

* Relatively minor change to governance tally function.
