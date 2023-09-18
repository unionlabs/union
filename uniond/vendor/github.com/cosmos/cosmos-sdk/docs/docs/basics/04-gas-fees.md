---
sidebar_position: 1
---

# Gas and Fees

:::note Synopsis
This document describes the default strategies to handle gas and fees within a Cosmos SDK application.
:::

:::note Pre-requisite Readings

* [Anatomy of a Cosmos SDK Application](./00-app-anatomy.md)

:::

## Introduction to `Gas` and `Fees`

In the Cosmos SDK, `gas` is a special unit that is used to track the consumption of resources during execution. `gas` is typically consumed whenever read and writes are made to the store, but it can also be consumed if expensive computation needs to be done. It serves two main purposes:

* Make sure blocks are not consuming too many resources and are finalized. This is implemented by default in the Cosmos SDK via the [block gas meter](#block-gas-meter).
* Prevent spam and abuse from end-user. To this end, `gas` consumed during [`message`](../building-modules/02-messages-and-queries.md#messages) execution is typically priced, resulting in a `fee` (`fees = gas * gas-prices`). `fees` generally have to be paid by the sender of the `message`. Note that the Cosmos SDK does not enforce `gas` pricing by default, as there may be other ways to prevent spam (e.g. bandwidth schemes). Still, most applications implement `fee` mechanisms to prevent spam by using the [`AnteHandler`](#antehandler).

## Gas Meter

In the Cosmos SDK, `gas` is a simple alias for `uint64`, and is managed by an object called a _gas meter_. Gas meters implement the `GasMeter` interface

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/store/types/gas.go#L40-L51
```

where:

* `GasConsumed()` returns the amount of gas that was consumed by the gas meter instance.
* `GasConsumedToLimit()` returns the amount of gas that was consumed by gas meter instance, or the limit if it is reached.
* `GasRemaining()` returns the gas left in the GasMeter.
* `Limit()` returns the limit of the gas meter instance. `0` if the gas meter is infinite.
* `ConsumeGas(amount Gas, descriptor string)` consumes the amount of `gas` provided. If the `gas` overflows, it panics with the `descriptor` message. If the gas meter is not infinite, it panics if `gas` consumed goes above the limit.
* `RefundGas()` deducts the given amount from the gas consumed. This functionality enables refunding gas to the transaction or block gas pools so that EVM-compatible chains can fully support the go-ethereum StateDB interface.
* `IsPastLimit()` returns `true` if the amount of gas consumed by the gas meter instance is strictly above the limit, `false` otherwise.
* `IsOutOfGas()` returns `true` if the amount of gas consumed by the gas meter instance is above or equal to the limit, `false` otherwise.

The gas meter is generally held in [`ctx`](../core/02-context.md), and consuming gas is done with the following pattern:

```go
ctx.GasMeter().ConsumeGas(amount, "description")
```

By default, the Cosmos SDK makes use of two different gas meters, the [main gas meter](#main-gas-metter) and the [block gas meter](#block-gas-meter).

### Main Gas Meter

`ctx.GasMeter()` is the main gas meter of the application. The main gas meter is initialized in `BeginBlock` via `setDeliverState`, and then tracks gas consumption during execution sequences that lead to state-transitions, i.e. those originally triggered by [`BeginBlock`](../core/00-baseapp.md#beginblock), [`DeliverTx`](../core/00-baseapp.md#delivertx) and [`EndBlock`](../core/00-baseapp.md#endblock). At the beginning of each `DeliverTx`, the main gas meter **must be set to 0** in the [`AnteHandler`](#antehandler), so that it can track gas consumption per-transaction.

Gas consumption can be done manually, generally by the module developer in the [`BeginBlocker`, `EndBlocker`](../building-modules/05-beginblock-endblock.md) or [`Msg` service](../building-modules/03-msg-services.md), but most of the time it is done automatically whenever there is a read or write to the store. This automatic gas consumption logic is implemented in a special store called [`GasKv`](../core/04-store.md#gaskv-store).

### Block Gas Meter

`ctx.BlockGasMeter()` is the gas meter used to track gas consumption per block and make sure it does not go above a certain limit. A new instance of the `BlockGasMeter` is created each time [`BeginBlock`](../core/00-baseapp.md#beginblock) is called. The `BlockGasMeter` is finite, and the limit of gas per block is defined in the application's consensus parameters. By default, Cosmos SDK applications use the default consensus parameters provided by CometBFT:

```go reference
https://github.com/cometbft/cometbft/blob/v0.37.0/types/params.go#L66-L105
```

When a new [transaction](../core/01-transactions.md) is being processed via `DeliverTx`, the current value of `BlockGasMeter` is checked to see if it is above the limit. If it is, `DeliverTx` returns immediately. This can happen even with the first transaction in a block, as `BeginBlock` itself can consume gas. If not, the transaction is processed normally. At the end of `DeliverTx`, the gas tracked by `ctx.BlockGasMeter()` is increased by the amount consumed to process the transaction:

```go
ctx.BlockGasMeter().ConsumeGas(
	ctx.GasMeter().GasConsumedToLimit(),
	"block gas meter",
)
```

## AnteHandler

The `AnteHandler` is run for every transaction during `CheckTx` and `DeliverTx`, before a Protobuf `Msg` service method for each `sdk.Msg` in the transaction. 

The anteHandler is not implemented in the core Cosmos SDK but in a module. That said, most applications today use the default implementation defined in the [`auth` module](https://github.com/cosmos/cosmos-sdk/tree/main/x/auth). Here is what the `anteHandler` is intended to do in a normal Cosmos SDK application:

* Verify that the transactions are of the correct type. Transaction types are defined in the module that implements the `anteHandler`, and they follow the transaction interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/types/tx_msg.go#L42-L50
```

  This enables developers to play with various types for the transaction of their application. In the default `auth` module, the default transaction type is `Tx`: 

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/tx/v1beta1/tx.proto#L13-L26
```

* Verify signatures for each [`message`](../building-modules/02-messages-and-queries.md#messages) contained in the transaction. Each `message` should be signed by one or multiple sender(s), and these signatures must be verified in the `anteHandler`.
* During `CheckTx`, verify that the gas prices provided with the transaction is greater than the local `min-gas-prices` (as a reminder, gas-prices can be deducted from the following equation: `fees = gas * gas-prices`). `min-gas-prices` is a parameter local to each full-node and used during `CheckTx` to discard transactions that do not provide a minimum amount of fees. This ensures that the mempool cannot be spammed with garbage transactions.
* Verify that the sender of the transaction has enough funds to cover for the `fees`. When the end-user generates a transaction, they must indicate 2 of the 3 following parameters (the third one being implicit): `fees`, `gas` and `gas-prices`. This signals how much they are willing to pay for nodes to execute their transaction. The provided `gas` value is stored in a parameter called `GasWanted` for later use.
* Set `newCtx.GasMeter` to 0, with a limit of `GasWanted`. **This step is crucial**, as it not only makes sure the transaction cannot consume infinite gas, but also that `ctx.GasMeter` is reset in-between each `DeliverTx` (`ctx` is set to `newCtx` after `anteHandler` is run, and the `anteHandler` is run each time `DeliverTx` is called).

As explained above, the `anteHandler` returns a maximum limit of `gas` the transaction can consume during execution called `GasWanted`. The actual amount consumed in the end is denominated `GasUsed`, and we must therefore have `GasUsed =< GasWanted`. Both `GasWanted` and `GasUsed` are relayed to the underlying consensus engine when [`DeliverTx`](../core/00-baseapp.md#delivertx) returns.
