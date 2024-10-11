---
sidebar_position: 1
---

# Query Services

:::note Synopsis
A Protobuf Query service processes [`queries`](./02-messages-and-queries.md#queries). Query services are specific to the module in which they are defined, and only process `queries` defined within said module. They are called from `BaseApp`'s [`Query` method](../../learn/advanced/00-baseapp.md#query).
:::

:::note Pre-requisite Readings

* [Module Manager](./01-module-manager.md)
* [Messages and Queries](./02-messages-and-queries.md)

:::

## Implementation of a module query service

### gRPC Service

When defining a Protobuf `Query` service, a `QueryServer` interface is generated for each module with all the service methods:

```go
type QueryServer interface {
	QueryBalance(context.Context, *QueryBalanceParams) (*types.Coin, error)
	QueryAllBalances(context.Context, *QueryAllBalancesParams) (*QueryAllBalancesResponse, error)
}
```

These custom queries methods should be implemented by a module's keeper, typically in `./keeper/grpc_query.go`. The first parameter of these methods is a generic `context.Context`. Therefore, the Cosmos SDK provides a function `sdk.UnwrapSDKContext` to retrieve the `context.Context` from the provided
`context.Context`.

Here's an example implementation for the bank module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/bank/keeper/grpc_query.go
```

### Calling queries from the State Machine

The Cosmos SDK v0.47 introduces a new `cosmos.query.v1.module_query_safe` Protobuf annotation which is used to state that a query that is safe to be called from within the state machine, for example:

* a Keeper's query function can be called from another module's Keeper,
* ADR-033 intermodule query calls,
* CosmWasm contracts can also directly interact with these queries.

If the `module_query_safe` annotation set to `true`, it means:

* The query is deterministic: given a block height it will return the same response upon multiple calls, and doesn't introduce any state-machine breaking changes across SDK patch versions.
* Gas consumption never fluctuates across calls and across patch versions.

If you are a module developer and want to use `module_query_safe` annotation for your own query, you have to ensure the following things:

* the query is deterministic and won't introduce state-machine-breaking changes without coordinated upgrades
* it has its gas tracked, to avoid the attack vector where no gas is accounted for
 on potentially high-computation queries.
