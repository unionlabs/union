# ADR 14: Proportional Slashing

## Changelog

* 2019-10-15: Initial draft
* 2020-05-25: Removed correlation root slashing
* 2020-07-01: Updated to include S-curve function instead of linear

## Context

In Proof of Stake-based chains, centralization of consensus power amongst a small set of validators can cause harm to the network due to increased risk of censorship, liveness failure, fork attacks, etc.  However, while this centralization causes a negative externality to the network, it is not directly felt by the delegators contributing towards delegating towards already large validators.  We would like a way to pass on the negative externality cost of centralization onto those large validators and their delegators.

## Decision

### Design

To solve this problem, we will implement a procedure called Proportional Slashing.  The desire is that the larger a validator is, the more they should be slashed.  The first naive attempt is to make a validator's slash percent proportional to their share of consensus voting power.

```text
slash_amount = k * power // power is the faulting validator's voting power and k is some on-chain constant
```

However, this will incentivize validators with large amounts of stake to split up their voting power amongst accounts (sybil attack), so that if they fault, they all get slashed at a lower percent.  The solution to this is to take into account not just a validator's own voting percentage, but also the voting percentage of all the other validators who get slashed in a specified time frame.

```text
slash_amount = k * (power_1 + power_2 + ... + power_n) // where power_i is the voting power of the ith validator faulting in the specified time frame and k is some on-chain constant
```

Now, if someone splits a validator of 10% into two validators of 5% each which both fault, then they both fault in the same time frame, they both will get slashed at the sum 10% amount.

However in practice, we likely don't want a linear relation between amount of stake at fault, and the percentage of stake to slash. In particular, solely 5% of stake double signing effectively did nothing to majorly threaten security, whereas 30% of stake being at fault clearly merits a large slashing factor, due to being very close to the point at which Tendermint security is threatened. A linear relation would require a factor of 6 gap between these two, whereas the difference in risk posed to the network is much larger. We propose using S-curves (formally [logistic functions](https://en.wikipedia.org/wiki/Logistic_function) to solve this). S-Curves capture the desired criterion quite well. They allow the slashing factor to be minimal for small values, and then grow very rapidly near some threshold point where the risk posed becomes notable.

#### Parameterization

This requires parameterizing a logistic function. It is very well understood how to parameterize this. It has four parameters:

1) A minimum slashing factor
2) A maximum slashing factor
3) The inflection point of the S-curve (essentially where do you want to center the S)
4) The rate of growth of the S-curve (How elongated is the S)

#### Correlation across non-sybil validators

One will note, that this model doesn't differentiate between multiple validators run by the same operators vs validators run by different operators.  This can be seen as an additional benefit in fact.  It incentivizes validators to differentiate their setups from other validators, to avoid having correlated faults with them or else they risk a higher slash.  So for example, operators should avoid using the same popular cloud hosting platforms or using the same Staking as a Service providers.  This will lead to a more resilient and decentralized network.

#### Griefing

Griefing, the act of intentionally getting oneself slashed in order to make another's slash worse, could be a concern here.  However, using the protocol described here, the attacker also gets equally impacted by the grief as the victim, so it would not provide much benefit to the griefer.

### Implementation

In the slashing module, we will add two queues that will track all of the recent slash events.  For double sign faults, we will define "recent slashes" as ones that have occurred within the last `unbonding period`.  For liveness faults, we will define "recent slashes" as ones that have occurred within the last `jail period`.

```go
type SlashEvent struct {
    Address                     sdk.ValAddress
    ValidatorVotingPercent      sdk.Dec
    SlashedSoFar                sdk.Dec
}
```

These slash events will be pruned from the queue once they are older than their respective "recent slash period".

Whenever a new slash occurs, a `SlashEvent` struct is created with the faulting validator's voting percent and a `SlashedSoFar` of 0.  Because recent slash events are pruned before the unbonding period and unjail period expires, it should not be possible for the same validator to have multiple SlashEvents in the same Queue at the same time.

We then will iterate over all the SlashEvents in the queue, adding their `ValidatorVotingPercent` to calculate the new percent to slash all the validators in the queue at, using the "Square of Sum of Roots" formula introduced above.

Once we have the `NewSlashPercent`, we then iterate over all the `SlashEvent`s in the queue once again, and if `NewSlashPercent > SlashedSoFar` for that SlashEvent, we call the `staking.Slash(slashEvent.Address, slashEvent.Power, Math.Min(Math.Max(minSlashPercent, NewSlashPercent - SlashedSoFar), maxSlashPercent))` (we pass in the power of the validator before any slashes occurred, so that we slash the right amount of tokens).  We then set `SlashEvent.SlashedSoFar` amount to `NewSlashPercent`.

## Status

Proposed

## Consequences

### Positive

* Increases decentralization by disincentivizing delegating to large validators
* Incentivizes Decorrelation of Validators
* More severely punishes attacks than accidental faults
* More flexibility in slashing rates parameterization

### Negative

* More computationally expensive than current implementation.  Will require more data about "recent slashing events" to be stored on chain.
