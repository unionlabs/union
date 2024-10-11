# ADR 022: Custom BaseApp panic handling

## Changelog

* 2020 Apr 24: Initial Draft
* 2021 Sep 14: Superseded by ADR-045

## Status

SUPERSEDED by ADR-045

## Context

The current implementation of BaseApp does not allow developers to write custom error handlers during panic recovery
[runTx()](https://github.com/cosmos/cosmos-sdk/blob/bad4ca75f58b182f600396ca350ad844c18fc80b/baseapp/baseapp.go#L539)
method. We think that this method can be more flexible and can give Cosmos SDK users more options for customizations without
the need to rewrite whole BaseApp. Also there's one special case for `sdk.ErrorOutOfGas` error handling, that case
might be handled in a "standard" way (middleware) alongside the others.

We propose middleware-solution, which could help developers implement the following cases:

* add external logging (let's say sending reports to external services like [Sentry](https://sentry.io));
* call panic for specific error cases;

It will also make `OutOfGas` case and `default` case one of the middlewares.
`Default` case wraps recovery object to an error and logs it ([example middleware implementation](#recovery-middleware)).

Our project has a sidecar service running alongside the blockchain node (smart contracts virtual machine). It is
essential that node <-> sidecar connectivity stays stable for TXs processing. So when the communication breaks we need
to crash the node and reboot it once the problem is solved. That behaviour makes node's state machine execution
deterministic. As all keeper panics are caught by runTx's `defer()` handler, we have to adjust the BaseApp code
in order to customize it.

## Decision

### Design

#### Overview

Instead of hardcoding custom error handling into BaseApp we suggest using set of middlewares which can be customized
externally and will allow developers use as many custom error handlers as they want. Implementation with tests
can be found [here](https://github.com/cosmos/cosmos-sdk/pull/6053).

#### Implementation details

##### Recovery handler

New `RecoveryHandler` type added. `recoveryObj` input argument is an object returned by the standard Go function
`recover()` from the `builtin` package.

```go
type RecoveryHandler func(recoveryObj interface{}) error
```

Handler should type assert (or other methods) an object to define if object should be handled.
`nil` should be returned if input object can't be handled by that `RecoveryHandler` (not a handler's target type).
Not `nil` error should be returned if input object was handled and middleware chain execution should be stopped.

An example:

```go
func exampleErrHandler(recoveryObj interface{}) error {
    err, ok := recoveryObj.(error)
    if !ok { return nil }

    if someSpecificError.Is(err) {
        panic(customPanicMsg)
    } else {
        return nil
    }
}
```

This example breaks the application execution, but it also might enrich the error's context like the `OutOfGas` handler.

##### Recovery middleware

We also add a middleware type (decorator). That function type wraps `RecoveryHandler` and returns the next middleware in
execution chain and handler's `error`. Type is used to separate actual `recovery()` object handling from middleware
chain processing.

```go
type recoveryMiddleware func(recoveryObj interface{}) (recoveryMiddleware, error)

func newRecoveryMiddleware(handler RecoveryHandler, next recoveryMiddleware) recoveryMiddleware {
    return func(recoveryObj interface{}) (recoveryMiddleware, error) {
        if err := handler(recoveryObj); err != nil {
            return nil, err
        }
        return next, nil
    }
}
```

Function receives a `recoveryObj` object and returns:

* (next `recoveryMiddleware`, `nil`) if object wasn't handled (not a target type) by `RecoveryHandler`;
* (`nil`, not nil `error`) if input object was handled and other middlewares in the chain should not be executed;
* (`nil`, `nil`) in case of invalid behavior. Panic recovery might not have been properly handled;
this can be avoided by always using a `default` as a rightmost middleware in the chain (always returns an `error`');

`OutOfGas` middleware example:

```go
func newOutOfGasRecoveryMiddleware(gasWanted uint64, ctx sdk.Context, next recoveryMiddleware) recoveryMiddleware {
    handler := func(recoveryObj interface{}) error {
        err, ok := recoveryObj.(sdk.ErrorOutOfGas)
        if !ok { return nil }

        return errorsmod.Wrap(
            sdkerrors.ErrOutOfGas, fmt.Sprintf(
                "out of gas in location: %v; gasWanted: %d, gasUsed: %d", err.Descriptor, gasWanted, ctx.GasMeter().GasConsumed(),
            ),
        )
    }

    return newRecoveryMiddleware(handler, next)
}
```

`Default` middleware example:

```go
func newDefaultRecoveryMiddleware() recoveryMiddleware {
    handler := func(recoveryObj interface{}) error {
        return errorsmod.Wrap(
            sdkerrors.ErrPanic, fmt.Sprintf("recovered: %v\nstack:\n%v", recoveryObj, string(debug.Stack())),
        )
    }

    return newRecoveryMiddleware(handler, nil)
}
```

##### Recovery processing

Basic chain of middlewares processing would look like:

```go
func processRecovery(recoveryObj interface{}, middleware recoveryMiddleware) error {
	if middleware == nil { return nil }

	next, err := middleware(recoveryObj)
	if err != nil { return err }
	if next == nil { return nil }

	return processRecovery(recoveryObj, next)
}
```

That way we can create a middleware chain which is executed from left to right, the rightmost middleware is a
`default` handler which must return an `error`.

##### BaseApp changes

The `default` middleware chain must exist in a `BaseApp` object. `Baseapp` modifications:

```go
type BaseApp struct {
    // ...
    runTxRecoveryMiddleware recoveryMiddleware
}

func NewBaseApp(...) {
    // ...
    app.runTxRecoveryMiddleware = newDefaultRecoveryMiddleware()
}

func (app *BaseApp) runTx(...) {
    // ...
    defer func() {
        if r := recover(); r != nil {
            recoveryMW := newOutOfGasRecoveryMiddleware(gasWanted, ctx, app.runTxRecoveryMiddleware)
            err, result = processRecovery(r, recoveryMW), nil
        }

        gInfo = sdk.GasInfo{GasWanted: gasWanted, GasUsed: ctx.GasMeter().GasConsumed()}
    }()
    // ...
}
```

Developers can add their custom `RecoveryHandler`s by providing `AddRunTxRecoveryHandler` as a BaseApp option parameter to the `NewBaseapp` constructor:

```go
func (app *BaseApp) AddRunTxRecoveryHandler(handlers ...RecoveryHandler) {
    for _, h := range handlers {
        app.runTxRecoveryMiddleware = newRecoveryMiddleware(h, app.runTxRecoveryMiddleware)
    }
}
```

This method would prepend handlers to an existing chain.

## Consequences

### Positive

* Developers of Cosmos SDK based projects can add custom panic handlers to:
    * add error context for custom panic sources (panic inside of custom keepers);
    * emit `panic()`: passthrough recovery object to the Tendermint core;
    * other necessary handling;
* Developers can use standard Cosmos SDK `BaseApp` implementation, rather that rewriting it in their projects;
* Proposed solution doesn't break the current "standard" `runTx()` flow;

### Negative

* Introduces changes to the execution model design.

### Neutral

* `OutOfGas` error handler becomes one of the middlewares;
* Default panic handler becomes one of the middlewares;

## References

* [PR-6053 with proposed solution](https://github.com/cosmos/cosmos-sdk/pull/6053)
* [Similar solution. ADR-010 Modular AnteHandler](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-010-modular-antehandler.md)
