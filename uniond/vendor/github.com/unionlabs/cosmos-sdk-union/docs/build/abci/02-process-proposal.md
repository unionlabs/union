# Process Proposal

`ProcessProposal` handles the validation of a proposal from `PrepareProposal`,
which also includes a block header. Meaning, that after a block has been proposed
the other validators have the right to accept or reject that block. The validator in the
default implementation of `PrepareProposal` runs basic validity checks on each
transaction.

Note, `ProcessProposal` MAY NOT be non-deterministic, i.e. it must be deterministic.
This means if `ProcessProposal` panics or fails and we reject, all honest validator
processes should reject (i.e., prevote nil). If so, then CometBFT will start a new round with a new block proposal, and the same cycle will happen with `PrepareProposal`
and `ProcessProposal` for the new proposal.

Here is the implementation of the default implementation:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/baseapp/abci_utils.go#L224-L231
```

Like `PrepareProposal` this implementation is the default and can be modified by
the application developer in [`app_di.go`](https://docs.cosmos.network/main/build/building-apps/app-go-di). If you decide to implement
your own `ProcessProposal` handler, you must ensure that the transactions
provided in the proposal DO NOT exceed the maximum block gas and `maxtxbytes` (if set).

```go
processOpt := func(app *baseapp.BaseApp) {
    abciPropHandler := baseapp.NewDefaultProposalHandler(mempool, app)
    app.SetProcessProposal(abciPropHandler.ProcessProposalHandler())
}

baseAppOptions = append(baseAppOptions, processOpt)
```
