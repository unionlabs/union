---
sidebar_position: 1
---

# BeginBlocker and EndBlocker

:::note Synopsis
`BeginBlocker` and `EndBlocker` are optional methods module developers can implement in their module. They will be triggered at the beginning and at the end of each block respectively, when the [`BeginBlock`](../core/00-baseapp.md#beginblock) and [`EndBlock`](../core/00-baseapp.md#endblock) ABCI messages are received from the underlying consensus engine.
:::

:::note Pre-requisite Readings

* [Module Manager](./01-module-manager.md)

:::

## BeginBlocker and EndBlocker

`BeginBlocker` and `EndBlocker` are a way for module developers to add automatic execution of logic to their module. This is a powerful tool that should be used carefully, as complex automatic functions can slow down or even halt the chain.

When needed, `BeginBlocker` and `EndBlocker` are implemented as part of the [`BeginBlockAppModule` and `BeginBlockAppModule` interfaces](./01-module-manager.md#appmodule). This means either can be left-out if not required. The `BeginBlock` and `EndBlock` methods of the interface implemented in `module.go` generally defer to `BeginBlocker` and `EndBlocker` methods respectively, which are usually implemented in `abci.go`.

The actual implementation of `BeginBlocker` and `EndBlocker` in `abci.go` are very similar to that of a [`Msg` service](./03-msg-services.md):

* They generally use the [`keeper`](./06-keeper.md) and [`ctx`](../core/02-context.md) to retrieve information about the latest state.
* If needed, they use the `keeper` and `ctx` to trigger state-transitions.
* If needed, they can emit [`events`](../core/08-events.md) via the `ctx`'s `EventManager`.

A specificity of the `EndBlocker` is that it can return validator updates to the underlying consensus engine in the form of an [`[]abci.ValidatorUpdates`](https://docs.cometbft.com/v0.37/spec/abci/abci++_methods#endblock). This is the preferred way to implement custom validator changes.

It is possible for developers to define the order of execution between the `BeginBlocker`/`EndBlocker` functions of each of their application's modules via the module's manager `SetOrderBeginBlocker`/`SetOrderEndBlocker` methods. For more on the module manager, click [here](./01-module-manager.md#manager).

See an example implementation of `BeginBlocker` from the `distribution` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/distribution/abci.go#L14-L38
```

and an example implementation of `EndBlocker` from the `staking` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/staking/abci.go#L22-L27
```
