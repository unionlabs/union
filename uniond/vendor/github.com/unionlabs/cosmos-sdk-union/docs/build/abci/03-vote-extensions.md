# Vote Extensions

:::note Synopsis
This section describes how the application can define and use vote extensions
defined in ABCI++.
:::

## Extend Vote

ABCI2.0 (colloquially called ABCI++) allows an application to extend a pre-commit vote with arbitrary data. This process does NOT have to be deterministic, and the data returned can be unique to the
validator process. The Cosmos SDK defines [`baseapp.ExtendVoteHandler`](https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/types/abci.go#L26-L27):

```go
type ExtendVoteHandler func(Context, *abci.ExtendVoteRequest) (*abci.ExtendVoteResponse, error)
```

An application can set this handler in `app.go` via the `baseapp.SetExtendVoteHandler`
`BaseApp` option function. The `sdk.ExtendVoteHandler`, if defined, is called during
the `ExtendVote` ABCI method. Note, if an application decides to implement
`baseapp.ExtendVoteHandler`, it MUST return a non-nil `VoteExtension`. However, the vote
extension can be empty. See [here](https://docs.cometbft.com/v1.0/spec/abci/abci++_methods#extendvote)
for more details.

There are many decentralized censorship-resistant use cases for vote extensions.
For example, a validator may want to submit prices for a price oracle or encryption
shares for an encrypted transaction mempool. Note, an application should be careful
to consider the size of the vote extensions as they could increase latency in block
production. See [here](https://docs.cometbft.com/v1.0/references/qa/cometbft-qa-38#vote-extensions-testbed)
for more details.

Click [here](https://docs.cosmos.network/main/build/abci/vote-extensions) if you would like a walkthrough of how to implement vote extensions.


## Verify Vote Extension

Similar to extending a vote, an application can also verify vote extensions from
other validators when validating their pre-commits. For a given vote extension,
this process MUST be deterministic. The Cosmos SDK defines [`sdk.VerifyVoteExtensionHandler`](https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/types/abci.go#L29-L31):

```go
type VerifyVoteExtensionHandler func(Context, *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error)
```

An application can set this handler in `app.go` via the `baseapp.SetVerifyVoteExtensionHandler`
`BaseApp` option function. The `sdk.VerifyVoteExtensionHandler`, if defined, is called
during the `VerifyVoteExtension` ABCI method. If an application defines a vote
extension handler, it should also define a verification handler. Note, not all
validators will share the same view of what vote extensions they verify depending
on how votes are propagated. See [here](https://docs.cometbft.com/v1.0/spec/abci/abci++_methods#verifyvoteextension)
for more details.

Additionally, please keep in mind that performance can be degraded if vote extensions are too big ([see vote extension testbed](https://docs.cometbft.com/v1.0/references/qa/cometbft-qa-38#vote-extensions-testbed)), so we highly recommend a size validation in `VerifyVoteExtensions`.


## Vote Extension Propagation

The agreed upon vote extensions at height `H` are provided to the proposing validator
at height `H+1` during `PrepareProposal`. As a result, the vote extensions are
not natively provided or exposed to the remaining validators during `ProcessProposal`.
As a result, if an application requires that the agreed upon vote extensions from
height `H` are available to all validators at `H+1`, the application must propagate
these vote extensions manually in the block proposal itself. This can be done by
"injecting" them into the block proposal, since the `Txs` field in `PrepareProposal`
is just a slice of byte slices.

`FinalizeBlock` will ignore any byte slice that doesn't implement an `sdk.Tx`, so
any injected vote extensions will safely be ignored in `FinalizeBlock`. For more
details on propagation, see the [ABCI++ 2.0 ADR](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-064-abci-2.0.md#vote-extension-propagation--verification).

### Recovery of injected Vote Extensions

As stated before, vote extensions can be injected into a block proposal (along with
other transactions in the `Txs` field). The Cosmos SDK provides a pre-FinalizeBlock
hook to allow applications to recover vote extensions, perform any necessary
computation on them, and then store the results in the cached store. These results
will be available to the application during the subsequent `FinalizeBlock` call.

An example of how a pre-FinalizeBlock hook could look like is shown below:

```go
app.SetPreBlocker(func(ctx sdk.Context, req *abci.RequestFinalizeBlock) error {
    allVEs := []VE{} // store all parsed vote extensions here
    for _, tx := range req.Txs {
        // define a custom function that tries to parse the tx as a vote extension
        ve, ok := parseVoteExtension(tx)
        if !ok {
            continue
        }

        allVEs = append(allVEs, ve)
    }

    // perform any necessary computation on the vote extensions and store the result
    // in the cached store
    result := compute(allVEs)
    err := storeVEResult(ctx, result)
    if err != nil {
        return err
    }

    return nil
})

```

Then, in an app's module, the application can retrieve the result of the computation
of vote extensions from the cached store:

```go
func (k Keeper) BeginBlocker(ctx context.Context) error {
    // retrieve the result of the computation of vote extensions from the cached store
    result, err := k.GetVEResult(ctx)
    if err != nil {
        return err
    }

    // use the result of the computation of vote extensions
    k.setSomething(result)

    return nil
}
```
