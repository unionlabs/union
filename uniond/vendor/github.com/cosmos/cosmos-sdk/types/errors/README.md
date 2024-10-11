# Errors

:::note
this package is deprecated, users should use "cosmossdk.io/errors" instead.
:::

Errors is a package that was deprecated but still contains error types. These error types are meant to be used by the Cosmos SDK to define error types. It is recommended that modules define their own error types, even if it is a duplicate of errors in this package. This will allow developers to more easily debug module errors and separate them from Cosmos SDK errors. 

For this package's documentation, please see the [go documentation](https://pkg.go.dev/github.com/cosmos/cosmos-sdk@v0.47.5/types/errors).
