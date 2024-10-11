# ADR 010: Modular AnteHandler

## Changelog

* 2019 Aug 31: Initial draft
* 2021 Sep 14: Superseded by ADR-045

## Status

SUPERSEDED by ADR-045

## Context

The current AnteHandler design allows users to either use the default AnteHandler provided in `x/auth` or to build their own AnteHandler from scratch. Ideally AnteHandler functionality is split into multiple, modular functions that can be chained together along with custom ante-functions so that users do not have to rewrite common antehandler logic when they want to implement custom behavior.

For example, let's say a user wants to implement some custom signature verification logic. In the current codebase, the user would have to write their own Antehandler from scratch largely reimplementing much of the same code and then set their own custom, monolithic antehandler in the baseapp. Instead, we would like to allow users to specify custom behavior when necessary and combine them with default ante-handler functionality in a way that is as modular and flexible as possible.

## Proposals

### Per-Module AnteHandler

One approach is to use the [ModuleManager](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/types/module) and have each module implement its own antehandler if it requires custom antehandler logic. The ModuleManager can then be passed in an AnteHandler order in the same way it has an order for BeginBlockers and EndBlockers. The ModuleManager returns a single AnteHandler function that will take in a tx and run each module's `AnteHandle` in the specified order. The module manager's AnteHandler is set as the baseapp's AnteHandler.

Pros:

1. Simple to implement
2. Utilizes the existing ModuleManager architecture

Cons:

1. Improves granularity but still cannot get more granular than a per-module basis. e.g. If auth's `AnteHandle` function is in charge of validating memo and signatures, users cannot swap the signature-checking functionality while keeping the rest of auth's `AnteHandle` functionality.
2. Module AnteHandler are run one after the other. There is no way for one AnteHandler to wrap or "decorate" another.

### Decorator Pattern

The [weave project](https://github.com/iov-one/weave) achieves AnteHandler modularity through the use of a decorator pattern. The interface is designed as follows:

```go
// Decorator wraps a Handler to provide common functionality
// like authentication, or fee-handling, to many Handlers
type Decorator interface {
	Check(ctx Context, store KVStore, tx Tx, next Checker) (*CheckResult, error)
	Deliver(ctx Context, store KVStore, tx Tx, next Deliverer) (*DeliverResult, error)
}
```

Each decorator works like a modularized Cosmos SDK antehandler function, but it can take in a `next` argument that may be another decorator or a Handler (which does not take in a next argument). These decorators can be chained together, one decorator being passed in as the `next` argument of the previous decorator in the chain. The chain ends in a Router which can take a tx and route to the appropriate msg handler.

A key benefit of this approach is that one Decorator can wrap its internal logic around the next Checker/Deliverer. A weave Decorator may do the following:

```go
// Example Decorator's Deliver function
func (example Decorator) Deliver(ctx Context, store KVStore, tx Tx, next Deliverer) {
    // Do some pre-processing logic

    res, err := next.Deliver(ctx, store, tx)

    // Do some post-processing logic given the result and error
}
```

Pros:

1. Weave Decorators can wrap over the next decorator/handler in the chain. The ability to both pre-process and post-process may be useful in certain settings.
2. Provides a nested modular structure that isn't possible in the solution above, while also allowing for a linear one-after-the-other structure like the solution above.

Cons:

1. It is hard to understand at first glance the state updates that would occur after a Decorator runs given the `ctx`, `store`, and `tx`. A Decorator can have an arbitrary number of nested Decorators being called within its function body, each possibly doing some pre- and post-processing before calling the next decorator on the chain. Thus to understand what a Decorator is doing, one must also understand what every other decorator further along the chain is also doing. This can get quite complicated to understand. A linear, one-after-the-other approach while less powerful, may be much easier to reason about.

### Chained Micro-Functions

The benefit of Weave's approach is that the Decorators can be very concise, which when chained together allows for maximum customizability. However, the nested structure can get quite complex and thus hard to reason about.

Another approach is to split the AnteHandler functionality into tightly scoped "micro-functions", while preserving the one-after-the-other ordering that would come from the ModuleManager approach.

We can then have a way to chain these micro-functions so that they run one after the other. Modules may define multiple ante micro-functions and then also provide a default per-module AnteHandler that implements a default, suggested order for these micro-functions.

Users can order the AnteHandlers easily by simply using the ModuleManager. The ModuleManager will take in a list of AnteHandlers and return a single AnteHandler that runs each AnteHandler in the order of the list provided. If the user is comfortable with the default ordering of each module, this is as simple as providing a list with each module's antehandler (exactly the same as BeginBlocker and EndBlocker).

If however, users wish to change the order or add, modify, or delete ante micro-functions in anyway; they can always define their own ante micro-functions and add them explicitly to the list that gets passed into module manager.

#### Default Workflow

This is an example of a user's AnteHandler if they choose not to make any custom micro-functions.

##### Cosmos SDK code

```go
// Chains together a list of AnteHandler micro-functions that get run one after the other.
// Returned AnteHandler will abort on first error.
func Chainer(order []AnteHandler) AnteHandler {
    return func(ctx Context, tx Tx, simulate bool) (newCtx Context, err error) {
        for _, ante := range order {
            ctx, err := ante(ctx, tx, simulate)
            if err != nil {
                return ctx, err
            }
        }
        return ctx, err
    }
}
```

```go
// AnteHandler micro-function to verify signatures
func VerifySignatures(ctx Context, tx Tx, simulate bool) (newCtx Context, err error) {
    // verify signatures
    // Returns InvalidSignature Result and abort=true if sigs invalid
    // Return OK result and abort=false if sigs are valid
}

// AnteHandler micro-function to validate memo
func ValidateMemo(ctx Context, tx Tx, simulate bool) (newCtx Context, err error) {
    // validate memo
}

// Auth defines its own default ante-handler by chaining its micro-functions in a recommended order
AuthModuleAnteHandler := Chainer([]AnteHandler{VerifySignatures, ValidateMemo})
```

```go
// Distribution micro-function to deduct fees from tx
func DeductFees(ctx Context, tx Tx, simulate bool) (newCtx Context, err error) {
    // Deduct fees from tx
    // Abort if insufficient funds in account to pay for fees
}

// Distribution micro-function to check if fees > mempool parameter
func CheckMempoolFees(ctx Context, tx Tx, simulate bool) (newCtx Context, err error) {
    // If CheckTx: Abort if the fees are less than the mempool's minFee parameter
}

// Distribution defines its own default ante-handler by chaining its micro-functions in a recommended order
DistrModuleAnteHandler := Chainer([]AnteHandler{CheckMempoolFees, DeductFees})
```

```go
type ModuleManager struct {
    // other fields
    AnteHandlerOrder []AnteHandler
}

func (mm ModuleManager) GetAnteHandler() AnteHandler {
    return Chainer(mm.AnteHandlerOrder)
}
```

##### User Code

```go
// Note: Since user is not making any custom modifications, we can just SetAnteHandlerOrder with the default AnteHandlers provided by each module in our preferred order
moduleManager.SetAnteHandlerOrder([]AnteHandler(AuthModuleAnteHandler, DistrModuleAnteHandler))

app.SetAnteHandler(mm.GetAnteHandler())
```

#### Custom Workflow

This is an example workflow for a user that wants to implement custom antehandler logic. In this example, the user wants to implement custom signature verification and change the order of antehandler so that validate memo runs before signature verification.

##### User Code

```go
// User can implement their own custom signature verification antehandler micro-function
func CustomSigVerify(ctx Context, tx Tx, simulate bool) (newCtx Context, err error) {
    // do some custom signature verification logic
}
```

```go
// Micro-functions allow users to change order of when they get executed, and swap out default ante-functionality with their own custom logic.
// Note that users can still chain the default distribution module handler, and auth micro-function along with their custom ante function
moduleManager.SetAnteHandlerOrder([]AnteHandler(ValidateMemo, CustomSigVerify, DistrModuleAnteHandler))
```

Pros:

1. Allows for ante functionality to be as modular as possible.
2. For users that do not need custom ante-functionality, there is little difference between how antehandlers work and how BeginBlock and EndBlock work in ModuleManager.
3. Still easy to understand

Cons:

1. Cannot wrap antehandlers with decorators like you can with Weave.

### Simple Decorators

This approach takes inspiration from Weave's decorator design while trying to minimize the number of breaking changes to the Cosmos SDK and maximizing simplicity. Like Weave decorators, this approach allows one `AnteDecorator` to wrap the next AnteHandler to do pre- and post-processing on the result. This is useful since decorators can do defer/cleanups after an AnteHandler returns as well as perform some setup beforehand. Unlike Weave decorators, these `AnteDecorator` functions can only wrap over the AnteHandler rather than the entire handler execution path. This is deliberate as we want decorators from different modules to perform authentication/validation on a `tx`. However, we do not want decorators being capable of wrapping and modifying the results of a `MsgHandler`.

In addition, this approach will not break any core Cosmos SDK API's. Since we preserve the notion of an AnteHandler and still set a single AnteHandler in baseapp, the decorator is simply an additional approach available for users that desire more customization. The API of modules (namely `x/auth`) may break with this approach, but the core API remains untouched.

Allow Decorator interface that can be chained together to create a Cosmos SDK AnteHandler.

This allows users to choose between implementing an AnteHandler by themselves and setting it in the baseapp, or use the decorator pattern to chain their custom decorators with the Cosmos SDK provided decorators in the order they wish.

```go
// An AnteDecorator wraps an AnteHandler, and can do pre- and post-processing on the next AnteHandler
type AnteDecorator interface {
    AnteHandle(ctx Context, tx Tx, simulate bool, next AnteHandler) (newCtx Context, err error)
}
```

```go
// ChainAnteDecorators will recursively link all of the AnteDecorators in the chain and return a final AnteHandler function
// This is done to preserve the ability to set a single AnteHandler function in the baseapp.
func ChainAnteDecorators(chain ...AnteDecorator) AnteHandler {
    if len(chain) == 1 {
        return func(ctx Context, tx Tx, simulate bool) {
            chain[0].AnteHandle(ctx, tx, simulate, nil)
        }
    }
    return func(ctx Context, tx Tx, simulate bool) {
        chain[0].AnteHandle(ctx, tx, simulate, ChainAnteDecorators(chain[1:]))
    }
}
```

#### Example Code

Define AnteDecorator functions

```go
// Setup GasMeter, catch OutOfGasPanic and handle appropriately
type SetUpContextDecorator struct{}

func (sud SetUpContextDecorator) AnteHandle(ctx Context, tx Tx, simulate bool, next AnteHandler) (newCtx Context, err error) {
    ctx.GasMeter = NewGasMeter(tx.Gas)

    defer func() {
        // recover from OutOfGas panic and handle appropriately
    }

    return next(ctx, tx, simulate)
}

// Signature Verification decorator. Verify Signatures and move on
type SigVerifyDecorator struct{}

func (svd SigVerifyDecorator) AnteHandle(ctx Context, tx Tx, simulate bool, next AnteHandler) (newCtx Context, err error) {
    // verify sigs. Return error if invalid

    // call next antehandler if sigs ok
    return next(ctx, tx, simulate)
}

// User-defined Decorator. Can choose to pre- and post-process on AnteHandler
type UserDefinedDecorator struct{
    // custom fields
}

func (udd UserDefinedDecorator) AnteHandle(ctx Context, tx Tx, simulate bool, next AnteHandler) (newCtx Context, err error) {
    // pre-processing logic

    ctx, err = next(ctx, tx, simulate)

    // post-processing logic
}
```

Link AnteDecorators to create a final AnteHandler. Set this AnteHandler in baseapp.

```go
// Create final antehandler by chaining the decorators together
antehandler := ChainAnteDecorators(NewSetUpContextDecorator(), NewSigVerifyDecorator(), NewUserDefinedDecorator())

// Set chained Antehandler in the baseapp
bapp.SetAnteHandler(antehandler)
```

Pros:

1. Allows one decorator to pre- and post-process the next AnteHandler, similar to the Weave design.
2. Do not need to break baseapp API. Users can still set a single AnteHandler if they choose.

Cons:

1. Decorator pattern may have a deeply nested structure that is hard to understand, this is mitigated by having the decorator order explicitly listed in the `ChainAnteDecorators` function.
2. Does not make use of the ModuleManager design. Since this is already being used for BeginBlocker/EndBlocker, this proposal seems unaligned with that design pattern.

## Consequences

Since pros and cons are written for each approach, it is omitted from this section

## References

* [#4572](https://github.com/cosmos/cosmos-sdk/issues/4572):  Modular AnteHandler Issue
* [#4582](https://github.com/cosmos/cosmos-sdk/pull/4583): Initial Implementation of Per-Module AnteHandler Approach
* [Weave Decorator Code](https://github.com/iov-one/weave/blob/master/handler.go#L35)
* [Weave Design Videos](https://vimeo.com/showcase/6189877)
