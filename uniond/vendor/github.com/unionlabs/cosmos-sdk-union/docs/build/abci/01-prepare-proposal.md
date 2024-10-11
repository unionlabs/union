# Prepare Proposal

`PrepareProposal` handles construction of the block, meaning that when a proposer
is preparing to propose a block, it requests the application to evaluate a
`RequestPrepareProposal`, which contains a series of transactions from CometBFT's
mempool. At this point, the application has complete control over the proposal.
It can modify, delete, and inject transactions from its own app-side mempool into
the proposal or even ignore all the transactions altogether. What the application
does with the transactions provided to it by `RequestPrepareProposal` has no
effect on CometBFT's mempool.

Note, that the application defines the semantics of the `PrepareProposal` and it
MAY be non-deterministic and is only executed by the current block proposer.

Now, reading mempool twice in the previous sentence is confusing, lets break it down.
CometBFT has a mempool that handles gossiping transactions to other nodes
in the network. The order of these transactions is determined by CometBFT's mempool,
using FIFO as the sole ordering mechanism. It's worth noting that the priority mempool
in Comet was removed or deprecated.
However, since the application is able to fully inspect
all transactions, it can provide greater control over transaction ordering.
Allowing the application to handle ordering enables the application to define how
it would like the block constructed.

The Cosmos SDK defines the `DefaultProposalHandler` type, which provides applications with
`PrepareProposal` and `ProcessProposal` handlers. If you decide to implement your
own `PrepareProposal` handler, you must ensure that the transactions
selected DO NOT exceed the maximum block gas (if set) and the maximum bytes provided
by `req.MaxBytes`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/baseapp/abci_utils.go
```

This default implementation can be overridden by the application developer in
favor of a custom implementation in [`app_di.go`](https://docs.cosmos.network/main/build/building-apps/app-go-di):

```go
prepareOpt := func(app *baseapp.BaseApp) {
    abciPropHandler := baseapp.NewDefaultProposalHandler(mempool, app)
    app.SetPrepareProposal(abciPropHandler.PrepareProposalHandler())
}

baseAppOptions = append(baseAppOptions, prepareOpt)
```
