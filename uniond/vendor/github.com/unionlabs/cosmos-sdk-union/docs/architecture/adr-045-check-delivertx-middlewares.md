# ADR 045: BaseApp `{Check,Deliver}Tx` as Middlewares

## Changelog

* 20.08.2021: Initial draft.
* 07.12.2021: Update `tx.Handler` interface ([\#10693](https://github.com/cosmos/cosmos-sdk/pull/10693)).
* 17.05.2022: ADR is abandoned, as middlewares are deemed too hard to reason about.

## Status

ABANDONED. Replacement is being discussed in [#11955](https://github.com/cosmos/cosmos-sdk/issues/11955).

## Abstract

This ADR replaces the current BaseApp `runTx` and antehandlers design with a middleware-based design.

## Context

BaseApp's implementation of ABCI `{Check,Deliver}Tx()` and its own `Simulate()` method call the `runTx` method under the hood, which first runs antehandlers, then executes `Msg`s. However, the [transaction Tips](https://github.com/cosmos/cosmos-sdk/issues/9406) and [refunding unused gas](https://github.com/cosmos/cosmos-sdk/issues/2150) use cases require custom logic to be run after the `Msg`s execution. There is currently no way to achieve this.

An naive solution would be to add post-`Msg` hooks to BaseApp. However, the Cosmos SDK team thinks in parallel about the bigger picture of making app wiring simpler ([#9181](https://github.com/cosmos/cosmos-sdk/discussions/9182)), which includes making BaseApp more lightweight and modular.

## Decision

We decide to transform Baseapp's implementation of ABCI `{Check,Deliver}Tx` and its own `Simulate` methods to use a middleware-based design.

The two following interfaces are the base of the middleware design, and are defined in `types/tx`:

```go
type Handler interface {
    CheckTx(ctx context.Context, req Request, checkReq RequestCheckTx) (Response, ResponseCheckTx, error)
    DeliverTx(ctx context.Context, req Request) (Response, error)
    SimulateTx(ctx context.Context, req Request (Response, error)
}

type Middleware func(Handler) Handler
```

where we define the following arguments and return types:

```go
type Request struct {
	Tx      sdk.Tx
	TxBytes []byte
}

type Response struct {
	GasWanted uint64
	GasUsed   uint64
	// MsgResponses is an array containing each Msg service handler's response
	// type, packed in an Any. This will get proto-serialized into the `Data` field
	// in the ABCI Check/DeliverTx responses.
	MsgResponses []*codectypes.Any
	Log          string
	Events       []abci.Event
}

type RequestCheckTx struct {
	Type abci.CheckTxType
}

type ResponseCheckTx struct {
	Priority int64
}
```

Please note that because CheckTx handles separate logic related to mempool priotization, its signature is different than DeliverTx and SimulateTx.

BaseApp holds a reference to a `tx.Handler`:

```go
type BaseApp  struct {
    // other fields
    txHandler tx.Handler
}
```

Baseapp's ABCI `{Check,Deliver}Tx()` and `Simulate()` methods simply call `app.txHandler.{Check,Deliver,Simulate}Tx()` with the relevant arguments. For example, for `DeliverTx`:

```go
func (app *BaseApp) DeliverTx(req abci.RequestDeliverTx) abci.ResponseDeliverTx {
    var abciRes abci.ResponseDeliverTx
	ctx := app.getContextForTx(runTxModeDeliver, req.Tx)
	res, err := app.txHandler.DeliverTx(ctx, tx.Request{TxBytes: req.Tx})
	if err != nil {
		abciRes = sdkerrors.ResponseDeliverTx(err, uint64(res.GasUsed), uint64(res.GasWanted), app.trace)
		return abciRes
	}

	abciRes, err = convertTxResponseToDeliverTx(res)
	if err != nil {
		return sdkerrors.ResponseDeliverTx(err, uint64(res.GasUsed), uint64(res.GasWanted), app.trace)
	}

	return abciRes
}

// convertTxResponseToDeliverTx converts a tx.Response into a abci.ResponseDeliverTx.
func convertTxResponseToDeliverTx(txRes tx.Response) (abci.ResponseDeliverTx, error) {
	data, err := makeABCIData(txRes)
	if err != nil {
		return abci.ResponseDeliverTx{}, nil
	}

	return abci.ResponseDeliverTx{
		Data:   data,
		Log:    txRes.Log,
		Events: txRes.Events,
	}, nil
}

// makeABCIData generates the Data field to be sent to ABCI Check/DeliverTx.
func makeABCIData(txRes tx.Response) ([]byte, error) {
	return proto.Marshal(&sdk.TxMsgData{MsgResponses: txRes.MsgResponses})
}
```

The implementations are similar for `BaseApp.CheckTx` and `BaseApp.Simulate`.

`baseapp.txHandler`'s three methods' implementations can obviously be monolithic functions, but for modularity we propose a middleware composition design, where a middleware is simply a function that takes a `tx.Handler`, and returns another `tx.Handler` wrapped around the previous one.

### Implementing a Middleware

In practice, middlewares are created by Go function that takes as arguments some parameters needed for the middleware, and returns a `tx.Middleware`.

For example, for creating an arbitrary `MyMiddleware`, we can implement:

```go
// myTxHandler is the tx.Handler of this middleware. Note that it holds a
// reference to the next tx.Handler in the stack.
type myTxHandler struct {
    // next is the next tx.Handler in the middleware stack.
    next tx.Handler
    // some other fields that are relevant to the middleware can be added here
}

// NewMyMiddleware returns a middleware that does this and that.
func NewMyMiddleware(arg1, arg2) tx.Middleware {
    return func (txh tx.Handler) tx.Handler {
        return myTxHandler{
            next: txh,
            // optionally, set arg1, arg2... if they are needed in the middleware
        }
    }
}

// Assert myTxHandler is a tx.Handler.
var _ tx.Handler = myTxHandler{}

func (h myTxHandler) CheckTx(ctx context.Context, req Request, checkReq RequestcheckTx) (Response, ResponseCheckTx, error) {
    // CheckTx specific pre-processing logic

    // run the next middleware
    res, checkRes, err := txh.next.CheckTx(ctx, req, checkReq)

    // CheckTx specific post-processing logic

    return res, checkRes, err
}

func (h myTxHandler) DeliverTx(ctx context.Context, req Request) (Response, error) {
    // DeliverTx specific pre-processing logic

    // run the next middleware
    res, err := txh.next.DeliverTx(ctx, tx, req)

    // DeliverTx specific post-processing logic

    return res, err
}

func (h myTxHandler) SimulateTx(ctx context.Context, req Request) (Response, error) {
    // SimulateTx specific pre-processing logic

    // run the next middleware
    res, err := txh.next.SimulateTx(ctx, tx, req)

    // SimulateTx specific post-processing logic

    return res, err
}
```

### Composing Middlewares

While BaseApp simply holds a reference to a `tx.Handler`, this `tx.Handler` itself is defined using a middleware stack. The Cosmos SDK exposes a base (i.e. innermost) `tx.Handler` called `RunMsgsTxHandler`, which executes messages.

Then, the app developer can compose multiple middlewares on top on the base `tx.Handler`. Each middleware can run pre-and-post-processing logic around its next middleware, as described in the section above. Conceptually, as an example, given the middlewares `A`, `B`, and `C` and the base `tx.Handler` `H` the stack looks like:

```text
A.pre
    B.pre
        C.pre
            H # The base tx.handler, for example `RunMsgsTxHandler`
        C.post
    B.post
A.post
```

We define a `ComposeMiddlewares` function for composing middlewares. It takes the base handler as first argument, and middlewares in the "outer to inner" order. For the above stack, the final `tx.Handler` is:

```go
txHandler := middleware.ComposeMiddlewares(H, A, B, C)
```

The middleware is set in BaseApp via its `SetTxHandler` setter:

```go
// simapp/app.go

txHandler := middleware.ComposeMiddlewares(...)
app.SetTxHandler(txHandler)
```

The app developer can define their own middlewares, or use the Cosmos SDK's pre-defined middlewares from `middleware.NewDefaultTxHandler()`.

### Middlewares Maintained by the Cosmos SDK

While the app developer can define and compose the middlewares of their choice, the Cosmos SDK provides a set of middlewares that caters for the ecosystem's most common use cases. These middlewares are:

| Middleware              | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| RunMsgsTxHandler        | This is the base `tx.Handler`. It replaces the old baseapp's `runMsgs`, and executes a transaction's `Msg`s.                                                                                                                                                                                                                                                                                                                                                                             |
| TxDecoderMiddleware     | This middleware takes in transaction raw bytes, and decodes them into a `sdk.Tx`. It replaces the `baseapp.txDecoder` field, so that BaseApp stays as thin as possible. Since most middlewares read the contents of the `sdk.Tx`, the TxDecoderMiddleware should be run first in the middleware stack.                                                                                                                                                                                   |
| {Antehandlers}          | Each antehandler is converted to its own middleware. These middlewares perform signature verification, fee deductions and other validations on the incoming transaction.                                                                                                                                                                                                                                                                                                                 |
| IndexEventsTxMiddleware | This is a simple middleware that chooses which events to index in Tendermint. Replaces `baseapp.indexEvents` (which unfortunately still exists in baseapp too, because it's used to index Begin/EndBlock events)                                                                                                                                                                                                                                                                         |
| RecoveryTxMiddleware    | This index recovers from panics. It replaces baseapp.runTx's panic recovery described in [ADR-022](./adr-022-custom-panic-handling.md).                                                                                                                                                                                                                                                                                                                                                  |
| GasTxMiddleware         | This replaces the [`Setup`](https://github.com/cosmos/cosmos-sdk/blob/v0.43.0/x/auth/ante/setup.go) Antehandler. It sets a GasMeter on sdk.Context. Note that before, GasMeter was set on sdk.Context inside the antehandlers, and there was some mess around the fact that antehandlers had their own panic recovery system so that the GasMeter could be read by baseapp's recovery system. Now, this mess is all removed: one middleware sets GasMeter, another one handles recovery. |

### Similarities and Differences between Antehandlers and Middlewares

The middleware-based design builds upon the existing antehandlers design described in [ADR-010](./adr-010-modular-antehandler.md). Even though the final decision of ADR-010 was to go with the "Simple Decorators" approach, the middleware design is actually very similar to the other [Decorator Pattern](./adr-010-modular-antehandler.md#decorator-pattern) proposal, also used in [weave](https://github.com/iov-one/weave).

#### Similarities with Antehandlers

* Designed as chaining/composing small modular pieces.
* Allow code reuse for `{Check,Deliver}Tx` and for `Simulate`.
* Set up in `app.go`, and easily customizable by app developers.
* Order is important.

#### Differences with Antehandlers

* The Antehandlers are run before `Msg` execution, whereas middlewares can run before and after.
* The middleware approach uses separate methods for `{Check,Deliver,Simulate}Tx`, whereas the antehandlers pass a `simulate bool` flag and uses the `sdkCtx.Is{Check,Recheck}Tx()` flags to determine in which transaction mode we are.
* The middleware design lets each middleware hold a reference to the next middleware, whereas the antehandlers pass a `next` argument in the `AnteHandle` method.
* The middleware design use Go's standard `context.Context`, whereas the antehandlers use `sdk.Context`.

## Consequences

### Backwards Compatibility

Since this refactor removes some logic away from BaseApp and into middlewares, it introduces API-breaking changes for app developers. Most notably, instead of creating an antehandler chain in `app.go`, app developers need to create a middleware stack:

```diff
- anteHandler, err := ante.NewAnteHandler(
-    ante.HandlerOptions{
-        AccountKeeper:   app.AccountKeeper,
-        BankKeeper:      app.BankKeeper,
-        SignModeHandler: encodingConfig.TxConfig.SignModeHandler(),
-        FeegrantKeeper:  app.FeeGrantKeeper,
-        SigGasConsumer:  ante.DefaultSigVerificationGasConsumer,
-    },
-)
+txHandler, err := authmiddleware.NewDefaultTxHandler(authmiddleware.TxHandlerOptions{
+    Debug:             app.Trace(),
+    IndexEvents:       indexEvents,
+    LegacyRouter:      app.legacyRouter,
+    MsgServiceRouter:  app.msgSvcRouter,
+    LegacyAnteHandler: anteHandler,
+    TxDecoder:         encodingConfig.TxConfig.TxDecoder,
+})
if err != nil {
    panic(err)
}
- app.SetAnteHandler(anteHandler)
+ app.SetTxHandler(txHandler)
```

Other more minor API breaking changes will also be provided in the CHANGELOG. As usual, the Cosmos SDK will provide a release migration document for app developers.

This ADR does not introduce any state-machine-, client- or CLI-breaking changes.

### Positive

* Allow custom logic to be run before an after `Msg` execution. This enables the [tips](https://github.com/cosmos/cosmos-sdk/issues/9406) and [gas refund](https://github.com/cosmos/cosmos-sdk/issues/2150) uses cases, and possibly other ones.
* Make BaseApp more lightweight, and defer complex logic to small modular components.
* Separate paths for `{Check,Deliver,Simulate}Tx` with different returns types. This allows for improved readability (replace `if sdkCtx.IsRecheckTx() && !simulate {...}` with separate methods) and more flexibility (e.g. returning a `priority` in `ResponseCheckTx`).

### Negative

* It is hard to understand at first glance the state updates that would occur after a middleware runs given the `sdk.Context` and `tx`. A middleware can have an arbitrary number of nested middleware being called within its function body, each possibly doing some pre- and post-processing before calling the next middleware on the chain. Thus to understand what a middleware is doing, one must also understand what every other middleware further along the chain is also doing, and the order of middlewares matters. This can get quite complicated to understand.
* API-breaking changes for app developers.

### Neutral

No neutral consequences.

## Further Discussions

* [#9934](https://github.com/cosmos/cosmos-sdk/discussions/9934) Decomposing BaseApp's other ABCI methods into middlewares.
* Replace `sdk.Tx` interface with the concrete protobuf Tx type in the `tx.Handler` methods signature.

## Test Cases

We update the existing baseapp and antehandlers tests to use the new middleware API, but keep the same test cases and logic, to avoid introducing regressions. Existing CLI tests will also be left untouched.

For new middlewares, we introduce unit tests. Since middlewares are purposefully small, unit tests suit well.

## References

* Initial discussion: https://github.com/cosmos/cosmos-sdk/issues/9585
* Implementation: [#9920 BaseApp refactor](https://github.com/cosmos/cosmos-sdk/pull/9920) and [#10028 Antehandlers migration](https://github.com/cosmos/cosmos-sdk/pull/10028)
