---
sidebar_position: 1
---

# Errors

:::note Synopsis
This document outlines the recommended usage and APIs for error handling in Cosmos SDK modules.
:::

Modules are encouraged to define and register their own errors to provide better
context on failed message or handler execution. Typically, these errors should be
common or general errors which can be further wrapped to provide additional specific execution context.

There are two ways to return errors. You can register custom errors with a codespace that is meant to provide more information to clients and normal go errors. The Cosmos SDK uses a mixture of both. 

:::Note
Errors v2 has been created as a zero dependency errors package. GRPC errors and tracing support is removed natively from the errors package. Users are required to wrap stack traces and add tracing information to their errors.
:::

:::Warning
If errors are registered they are part of consensus and cannot be changed in a minor release
:::

## Registration

Modules should define and register their custom errors in `x/{module}/errors.go`.
Registration of errors is handled via the [`errors` package](https://github.com/cosmos/cosmos-sdk/blob/main/errors/errors.go).

Example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/distribution/types/errors.go
```

Each custom module error must provide the codespace, which is typically the module name
(e.g. "distribution") and is unique per module, and a uint32 code. Together, the codespace and code
provide a globally unique Cosmos SDK error. Typically, the code is monotonically increasing but does not
necessarily have to be. The only restrictions on error codes are the following:

* Must be greater than one, as a code value of one is reserved for internal errors.
* Must be unique within the module.

## Wrapping

The custom module errors can be returned as their concrete type as they already fulfill the `error`
interface. However, module errors can be wrapped to provide further context and meaning to failed execution.

Example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/bank/keeper/keeper.go#L141-L182
```

## ABCI

If a module error is registered, the Cosmos SDK `errors` package allows ABCI information to be extracted
through the `ABCIInfo` function. The package also provides `ResponseCheckTx` and `ResponseDeliverTx` as
auxiliary functions to automatically get `CheckTx` and `DeliverTx` responses from an error.
