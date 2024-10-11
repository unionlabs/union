# ADR 18: Extendable Voting Periods

## Changelog

* 1 January 2020: Start of first version

## Context

Currently the voting period for all governance proposals is the same.  However, this is suboptimal as all governance proposals do not require the same time period.  For more non-contentious proposals, they can be dealt with more efficiently with a faster period, while more contentious or complex proposals may need a longer period for extended discussion/consideration.

## Decision

We would like to design a mechanism for making the voting period of a governance proposal variable based on the demand of voters.  We would like it to be based on the view of the governance participants, rather than just the proposer of a governance proposal (thus, allowing the proposer to select the voting period length is not sufficient).

However, we would like to avoid the creation of an entire second voting process to determine the length of the voting period, as it just pushed the problem to determining the length of that first voting period.

Thus, we propose the following mechanism:

### Params

* The current gov param `VotingPeriod` is to be replaced by a `MinVotingPeriod` param.  This is the default voting period that all governance proposal voting periods start with.
* There is a new gov param called `MaxVotingPeriodExtension`.

### Mechanism

There is a new `Msg` type called `MsgExtendVotingPeriod`, which can be sent by any staked account during a proposal's voting period.  It allows the sender to unilaterally extend the length of the voting period by `MaxVotingPeriodExtension * sender's share of voting power`.  Every address can only call `MsgExtendVotingPeriod` once per proposal.

So for example, if the `MaxVotingPeriodExtension` is set to 100 Days, then anyone with 1% of voting power can extend the voting power by 1 day.  If 33% of voting power has sent the message, the voting period will be extended by 33 days.  Thus, if absolutely everyone chooses to extend the voting period, the absolute maximum voting period will be `MinVotingPeriod + MaxVotingPeriodExtension`.

This system acts as a sort of distributed coordination, where individual stakers choosing to extend or not, allows the system the gauge the conentiousness/complexity of the proposal.  It is extremely unlikely that many stakers will choose to extend at the exact same time, it allows stakers to view how long others have already extended thus far, to decide whether or not to extend further.

### Dealing with Unbonding/Redelegation

There is one thing that needs to be addressed.  How to deal with redelegation/unbonding during the voting period.  If a staker of 5% calls `MsgExtendVotingPeriod` and then unbonds, does the voting period then decrease by 5 days again?  This is not good as it can give people a false sense of how long they have to make their decision.  For this reason, we want to design it such that the voting period length can only be extended, not shortened.  To do this, the current extension amount is based on the highest percent that voted extension at any time.  This is best explained by example:

1. Let's say 2 stakers of voting power 4% and 3% respectively vote to extend.  The voting period will be extended by 7 days.
2. Now the staker of 3% decides to unbond before the end of the voting period.  The voting period extension remains 7 days.
3. Now, let's say another staker of 2% voting power decides to extend voting period.  There is now 6% of active voting power choosing the extend.  The voting power remains 7 days.
4. If a fourth staker of 10% chooses to extend now, there is a total of 16% of active voting power wishing to extend.  The voting period will be extended to 16 days.

### Delegators

Just like votes in the actual voting period, delegators automatically inherit the extension of their validators.  If their validator chooses to extend, their voting power will be used in the validator's extension.  However, the delegator is unable to override their validator and "unextend" as that would contradict the "voting power length can only be ratcheted up" principle described in the previous section.  However, a delegator may choose the extend using their personal voting power, if their validator has not done so.

## Status

Proposed

## Consequences

### Positive

* More complex/contentious governance proposals will have more time to properly digest and deliberate

### Negative

* Governance process becomes more complex and requires more understanding to interact with effectively
* Can no longer predict when a governance proposal will end. Can't assume order in which governance proposals will end.

### Neutral

* The minimum voting period can be made shorter

## References

* [Cosmos Forum post where idea first originated](https://forum.cosmos.network/t/proposal-draft-reduce-governance-voting-period-to-7-days/3032/9)
