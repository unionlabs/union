# Simsx

This package introduces some new helper types to simplify message construction for simulations (sims).  The focus is on better dev UX for new message factories.
Technically, they are adapters that build upon the existing sims framework.

## [Message factory](https://github.com/cosmos/cosmos-sdk/blob/main/simsx/msg_factory.go)

Simple functions as factories for dedicated sdk.Msgs. They have access to the context, reporter and test data environment. For example:

```go
func MsgSendFactory() simsx.SimMsgFactoryFn[*types.MsgSend] {
    return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgSend) {
        from := testData.AnyAccount(reporter, simsx.WithSpendableBalance())
        to := testData.AnyAccount(reporter, simsx.ExcludeAccounts(from))
        coins := from.LiquidBalance().RandSubsetCoins(reporter, simsx.WithSendEnabledCoins())
        return []simsx.SimAccount{from}, types.NewMsgSend(from.AddressBech32, to.AddressBech32, coins)
    }
}
```

## [Sims registry](https://github.com/cosmos/cosmos-sdk/blob/main/simsx/registry.go)

A new helper to register message factories with a default weight value. They can be overwritten by a parameters file as before. The registry is passed to the AppModule type. For example:

```go
func (am AppModule) WeightedOperationsX(weights simsx.WeightSource, reg simsx.Registry) {
    reg.Add(weights.Get("msg_send", 100), simulation.MsgSendFactory())
    reg.Add(weights.Get("msg_multisend", 10), simulation.MsgMultiSendFactory())
}
```

## [Reporter](https://github.com/cosmos/cosmos-sdk/blob/main/simsx/reporter.go)

The reporter is a flow control structure that can be used in message factories to skip execution at any point. The idea is similar to the testing.T Skip in Go stdlib. Internally, it converts skip, success and failure events to legacy sim messages.
The reporter also provides some capability to print an execution summary.
It is also used to interact with the test data environment to not have errors checked all the time.
Message factories may want to abort early via

```go
if reporter.IsSkipped() {
    return nil, nil
}
```

## [Test data environment](https://github.com/cosmos/cosmos-sdk/blob/main/simsx/environment.go)

The test data environment provides simple access to accounts and other test data used in most message factories.  It also encapsulates some app internals like bank keeper or address codec.
