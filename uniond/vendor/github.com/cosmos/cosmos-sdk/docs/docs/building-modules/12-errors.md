---
sidebar_position: 1
---

# Errors

:::note Synopsis
This document outlines the recommended usage and APIs for error handling in Cosmos SDK modules.
:::

Modules are encouraged to define and register their own errors to provide better
context on failed message or handler execution. Typically, these errors should be
common or general errors which can be further wrapped to provide additional specific
execution context.

## Registration

Modules should define and register their custom errors in `x/{module}/errors.go`.
Registration of errors is handled via the [`errors` package](https://github.com/cosmos/cosmos-sdk/blob/main/errors/errors.go).

Example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/distribution/types/errors.go#L1-L21
```

Each custom module error must provide the codespace, which is typically the module name
(e.g. "distribution") and is unique per module, and a uint32 code. Together, the codespace and code
provide a globally unique Cosmos SDK error. Typically, the code is monotonically increasing but does not
necessarily have to be. The only restrictions on error codes are the following:

* Must be greater than one, as a code value of one is reserved for internal errors.
* Must be unique within the module.

Note, the Cosmos SDK provides a core set of *common* errors. These errors are defined in [`types/errors/errors.go`](https://github.com/cosmos/cosmos-sdk/blob/main/types/errors/errors.go).

## Wrapping

The custom module errors can be returned as their concrete type as they already fulfill the `error`
interface. However, module errors can be wrapped to provide further context and meaning to failed
execution.

Example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/bank/keeper/keeper.go#L141-L182
```

Regardless if an error is wrapped or not, the Cosmos SDK's `errors` package provides a function to determine if
an error is of a particular kind via `Is`.

## ABCI

If a module error is registered, the Cosmos SDK `errors` package allows ABCI information to be extracted
through the `ABCIInfo` function. The package also provides `ResponseCheckTx` and `ResponseDeliverTx` as
auxiliary functions to automatically get `CheckTx` and `DeliverTx` responses from an error.
