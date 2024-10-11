# ADR 058: Event hashing

## Changelog

- 2020-07-17: initial version
- 2020-07-27: fixes after Ismail and Ethan's comments
- 2020-07-27: declined

## Context

Before [PR#4845](https://github.com/tendermint/tendermint/pull/4845),
`Header#LastResultsHash` was a root of the Merkle tree built from `DeliverTx`
results. Only `Code`, `Data` fields were included because `Info` and `Log`
fields are non-deterministic.

At some point, we've added events to `ResponseBeginBlock`, `ResponseEndBlock`,
and `ResponseDeliverTx` to give applications a way to attach some additional
information to blocks / transactions.

Many applications seem to have started using them since.

However, before [PR#4845](https://github.com/tendermint/tendermint/pull/4845)
there was no way to prove that certain events were a part of the result
(_unless the application developer includes them into the state tree_).

Hence, [PR#4845](https://github.com/tendermint/tendermint/pull/4845) was
opened. In it, `GasWanted` along with `GasUsed` are included when hashing
`DeliverTx` results. Also, events from `BeginBlock`, `EndBlock` and `DeliverTx`
results are hashed into the `LastResultsHash` as follows:

- Since we do not expect `BeginBlock` and `EndBlock` to contain many events,
  these will be Protobuf encoded and included in the Merkle tree as leaves.
- `LastResultsHash` therefore is the root hash of a Merkle tree w/ 3 leafs:
  proto-encoded `ResponseBeginBlock#Events`, root hash of a Merkle tree build
  from `ResponseDeliverTx` responses (Log, Info and Codespace fields are
  ignored), and proto-encoded `ResponseEndBlock#Events`.
- Order of events is unchanged - same as received from the ABCI application.

[Spec PR](https://github.com/tendermint/spec/pull/97/files)

While it's certainly good to be able to prove something, introducing new events
or removing such becomes difficult because it breaks the `LastResultsHash`. It
means that every time you add, remove or update an event, you'll need a
hard-fork. And that is undoubtedly bad for applications, which are evolving and
don't have a stable events set.

## Decision

As a middle ground approach, the proposal is to add the
`Block#LastResultsEvents` consensus parameter that is a list of all events that
are to be hashed in the header.

```
@ proto/tendermint/abci/types.proto:295 @ message BlockParams {
  int64 max_bytes = 1;
  // Note: must be greater or equal to -1
  int64 max_gas = 2;
  // List of events, which will be hashed into the LastResultsHash
  repeated string last_results_events = 3;
}
```

Initially the list is empty. The ABCI application can change it via `InitChain`
or `EndBlock`.

Example:

```go
func (app *MyApp) DeliverTx(req types.RequestDeliverTx) types.ResponseDeliverTx {
    //...
    events := []abci.Event{
        {
            Type: "transfer",
            Attributes: []abci.EventAttribute{
                {Key: []byte("sender"), Value: []byte("Bob"), Index: true},
            },
        },
    }
    return types.ResponseDeliverTx{Code: code.CodeTypeOK, Events: events}
}
```

For "transfer" event to be hashed, the `LastResultsEvents` must contain a
string "transfer".

## Status

Declined

**Until there's more stability/motivation/use-cases/demand, the decision is to
push this entirely application side and just have apps which want events to be
provable to insert them into their application-side merkle trees. Of course
this puts more pressure on their application state and makes event proving
application specific, but it might help built up a better sense of use-cases
and how this ought to ultimately be done by Tendermint.**

## Consequences

### Positive

1. networks can perform parameter change proposals to update this list as new events are added
2. allows networks to avoid having to do hard-forks
3. events can still be added at-will to the application w/o breaking anything

### Negative

1. yet another consensus parameter
2. more things to track in the tendermint state

## References

- [ADR 021](adr-021-abci-events.md)
- [Indexing transactions](../app-dev/indexing-transactions.md)

## Appendix A. Alternative proposals

The other proposal was to add `Hash bool` flag to the `Event`, similarly to
`Index bool` EventAttribute's field. When `true`, Tendermint would hash it into
the `LastResultsEvents`. The downside is that the logic is implicit and depends
largely on the node's operator, who decides what application code to run. The
above proposal makes it (the logic) explicit and easy to upgrade via
governance.
