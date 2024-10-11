# ADR 17: Historical Header Module

## Changelog

* 26 November 2019: Start of first version
* 2 December 2019: Final draft of first version
* 7 September 2023: Reduce HistoricalInfo type

## Context

In order for the Cosmos SDK to implement the [IBC specification](https://github.com/cosmos/ics), modules within the Cosmos SDK must have the ability to introspect recent consensus states (validator sets & commitment roots) as proofs of these values on other chains must be checked during the handshakes.

## Decision

The application MUST store the most recent `n` headers in a persistent store. At first, this store MAY be the current Merklised store. A non-Merklised store MAY be used later as no proofs are necessary.

The application MUST store this information by storing new headers immediately when handling `abci.RequestBeginBlock`:

```go
func BeginBlock(ctx sdk.Context, keeper HistoricalHeaderKeeper) error {
  info := HistoricalInfo{
    apphash: ctx.HeaderInfo().AppHash,
    Time: ctx.HeaderInfo().Time,
    NextValidatorsHash: ctx.CometInfo().NextValidatorsHash,
  }
  keeper.SetHistoricalInfo(ctx, ctx.BlockHeight(), info)
  n := keeper.GetParamRecentHeadersToStore()
  keeper.PruneHistoricalInfo(ctx, ctx.BlockHeight() - n)
  // continue handling request
}
```

Alternatively, the application MAY store only the hash of the validator set.

The application MUST make these past `n` committed headers available for querying by Cosmos SDK modules through the `Keeper`'s `GetHistoricalInfo` function. This MAY be implemented in a new module, or it MAY also be integrated into an existing one (likely `x/staking` or `x/ibc`).

`n` MAY be configured as a parameter store parameter, in which case it could be changed by `ParameterChangeProposal`s, although it will take some blocks for the stored information to catch up if `n` is increased.

## Status

Proposed.

## Consequences

Implementation of this ADR will require changes to the Cosmos SDK. It will not require changes to Tendermint.

### Positive

* Easy retrieval of headers & state roots for recent past heights by modules anywhere in the Cosmos SDK.
* No RPC calls to Tendermint required.
* No ABCI alterations required.

### Negative

* Duplicates `n` headers data in Tendermint & the application (additional disk usage) - in the long term, an approach such as [this](https://github.com/tendermint/tendermint/issues/4210) might be preferable.

### Neutral

(none known)

## References

* [ICS 2: "Consensus state introspection"](https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics#consensus-state-introspection)
