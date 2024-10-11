# ADR 041: Application should be in charge of validator set

## Changelog


## Context

Currently Tendermint is in charge of validator set and proposer selection. Application can only update the validator set changes at EndBlock time.
To support Light Client, application should make sure at least 2/3 of validator are same at each round.

Application should have full control on validator set changes and proposer selection. In each round Application can provide the list of validators for next rounds in order with their power. The proposer is the first in the list, in case the proposer is offline, the next one can propose the proposal and so on.

## Decision

## Status

## Consequences

Tendermint is no more in charge of validator set and its changes. The Application should provide the correct information.
However Tendermint can provide psedo-randomness algorithm to help application for selecting proposer in each round.

### Positive

### Negative

### Neutral

## References

