# ADR 021: Protocol Buffer Query Encoding

## Changelog

* 2020 March 27: Initial Draft

## Status

Accepted

## Context

This ADR is a continuation of the motivation, design, and context established in
[ADR 019](./adr-019-protobuf-state-encoding.md) and
[ADR 020](./adr-020-protobuf-transaction-encoding.md), namely, we aim to design the
Protocol Buffer migration path for the client-side of the Cosmos SDK.

This ADR continues from [ADD 020](./adr-020-protobuf-transaction-encoding.md)
to specify the encoding of queries.

## Decision

### Custom Query Definition

Modules define custom queries through a protocol buffers `service` definition.
These `service` definitions are generally associated with and used by the
GRPC protocol. However, the protocol buffers specification indicates that
they can be used more generically by any request/response protocol that uses
protocol buffer encoding. Thus, we can use `service` definitions for specifying
custom ABCI queries and even reuse a substantial amount of the GRPC infrastructure.

Each module with custom queries should define a service canonically named `Query`:

```protobuf
// x/bank/types/types.proto

service Query {
  rpc QueryBalance(QueryBalanceParams) returns (cosmos_sdk.v1.Coin) { }
  rpc QueryAllBalances(QueryAllBalancesParams) returns (QueryAllBalancesResponse) { }
}
```

#### Handling of Interface Types

Modules that use interface types and need true polymorphism generally force a
`oneof` up to the app-level that provides the set of concrete implementations of
that interface that the app supports. While app's are welcome to do the same for
queries and implement an app-level query service, it is recommended that modules
provide query methods that expose these interfaces via `google.protobuf.Any`.
There is a concern on the transaction level that the overhead of `Any` is too
high to justify its usage. However for queries this is not a concern, and
providing generic module-level queries that use `Any` does not preclude apps
from also providing app-level queries that return use the app-level `oneof`s.

A hypothetical example for the `gov` module would look something like:

```protobuf
// x/gov/types/types.proto

import "google/protobuf/any.proto";

service Query {
  rpc GetProposal(GetProposalParams) returns (AnyProposal) { }
}

message AnyProposal {
  ProposalBase base = 1;
  google.protobuf.Any content = 2;
}
```

### Custom Query Implementation

In order to implement the query service, we can reuse the existing [gogo protobuf](https://github.com/cosmos/gogoproto)
grpc plugin, which for a service named `Query` generates an interface named
`QueryServer` as below:

```go
type QueryServer interface {
	QueryBalance(context.Context, *QueryBalanceParams) (*types.Coin, error)
	QueryAllBalances(context.Context, *QueryAllBalancesParams) (*QueryAllBalancesResponse, error)
}
```

The custom queries for our module are implemented by implementing this interface.

The first parameter in this generated interface is a generic `context.Context`,
whereas querier methods generally need an instance of `sdk.Context` to read
from the store. Since arbitrary values can be attached to `context.Context`
using the `WithValue` and `Value` methods, the Cosmos SDK should provide a function
`sdk.UnwrapSDKContext` to retrieve the `sdk.Context` from the provided
`context.Context`.

An example implementation of `QueryBalance` for the bank module as above would
look something like:

```go
type Querier struct {
	Keeper
}

func (q Querier) QueryBalance(ctx context.Context, params *types.QueryBalanceParams) (*sdk.Coin, error) {
	balance := q.GetBalance(sdk.UnwrapSDKContext(ctx), params.Address, params.Denom)
	return &balance, nil
}
```

### Custom Query Registration and Routing

Query server implementations as above would be registered with `AppModule`s using
a new method `RegisterQueryService(grpc.Server)` which could be implemented simply
as below:

```go
// x/bank/module.go
func (am AppModule) RegisterQueryService(server grpc.Server) {
	types.RegisterQueryServer(server, keeper.Querier{am.keeper})
}
```

Underneath the hood, a new method `RegisterService(sd *grpc.ServiceDesc, handler interface{})`
will be added to the existing `baseapp.QueryRouter` to add the queries to the custom
query routing table (with the routing method being described below).
The signature for this method matches the existing
`RegisterServer` method on the GRPC `Server` type where `handler` is the custom
query server implementation described above.

GRPC-like requests are routed by the service name (ex. `cosmos_sdk.x.bank.v1.Query`)
and method name (ex. `QueryBalance`) combined with `/`s to form a full
method name (ex. `/cosmos_sdk.x.bank.v1.Query/QueryBalance`). This gets translated
into an ABCI query as `custom/cosmos_sdk.x.bank.v1.Query/QueryBalance`. Service handlers
registered with `QueryRouter.RegisterService` will be routed this way.

Beyond the method name, GRPC requests carry a protobuf encoded payload, which maps naturally
to `RequestQuery.Data`, and receive a protobuf encoded response or error. Thus
there is a quite natural mapping of GRPC-like rpc methods to the existing
`sdk.Query` and `QueryRouter` infrastructure.

This basic specification allows us to reuse protocol buffer `service` definitions
for ABCI custom queries substantially reducing the need for manual decoding and
encoding in query methods.

### GRPC Protocol Support

In addition to providing an ABCI query pathway, we can easily provide a GRPC
proxy server that routes requests in the GRPC protocol to ABCI query requests
under the hood. In this way, clients could use their host languages' existing
GRPC implementations to make direct queries against Cosmos SDK app's using
these `service` definitions. In order for this server to work, the `QueryRouter`
on `BaseApp` will need to expose the service handlers registered with
`QueryRouter.RegisterService` to the proxy server implementation. Nodes could
launch the proxy server on a separate port in the same process as the ABCI app
with a command-line flag.

### REST Queries and Swagger Generation

[grpc-gateway](https://github.com/grpc-ecosystem/grpc-gateway) is a project that
translates REST calls into GRPC calls using special annotations on service
methods. Modules that want to expose REST queries should add `google.api.http`
annotations to their `rpc` methods as in this example below.

```protobuf
// x/bank/types/types.proto

service Query {
  rpc QueryBalance(QueryBalanceParams) returns (cosmos_sdk.v1.Coin) {
    option (google.api.http) = {
      get: "/x/bank/v1/balance/{address}/{denom}"
    };
  }
  rpc QueryAllBalances(QueryAllBalancesParams) returns (QueryAllBalancesResponse) {
    option (google.api.http) = {
      get: "/x/bank/v1/balances/{address}"
    };
  }
}
```

grpc-gateway will work directly against the GRPC proxy described above which will
translate requests to ABCI queries under the hood. grpc-gateway can also
generate Swagger definitions automatically.

In the current implementation of REST queries, each module needs to implement
REST queries manually in addition to ABCI querier methods. Using the grpc-gateway
approach, there will be no need to generate separate REST query handlers, just
query servers as described above as grpc-gateway handles the translation of protobuf
to REST as well as Swagger definitions.

The Cosmos SDK should provide CLI commands for apps to start GRPC gateway either in
a separate process or the same process as the ABCI app, as well as provide a
command for generating grpc-gateway proxy `.proto` files and the `swagger.json`
file.

### Client Usage

The gogo protobuf grpc plugin generates client interfaces in addition to server
interfaces. For the `Query` service defined above we would get a `QueryClient`
interface like:

```go
type QueryClient interface {
	QueryBalance(ctx context.Context, in *QueryBalanceParams, opts ...grpc.CallOption) (*types.Coin, error)
	QueryAllBalances(ctx context.Context, in *QueryAllBalancesParams, opts ...grpc.CallOption) (*QueryAllBalancesResponse, error)
}
```

Via a small patch to gogo protobuf ([gogo/protobuf#675](https://github.com/gogo/protobuf/pull/675))
we have tweaked the grpc codegen to use an interface rather than concrete type
for the generated client struct. This allows us to also reuse the GRPC infrastructure
for ABCI client queries.

1Context`will receive a new method`QueryConn`that returns a`ClientConn`
that routes calls to ABCI queries

Clients (such as CLI methods) will then be able to call query methods like this:

```go
clientCtx := client.NewContext()
queryClient := types.NewQueryClient(clientCtx.QueryConn())
params := &types.QueryBalanceParams{addr, denom}
result, err := queryClient.QueryBalance(gocontext.Background(), params)
```

### Testing

Tests would be able to create a query client directly from keeper and `sdk.Context`
references using a `QueryServerTestHelper` as below:

```go
queryHelper := baseapp.NewQueryServerTestHelper(ctx)
types.RegisterQueryServer(queryHelper, keeper.Querier{app.BankKeeper})
queryClient := types.NewQueryClient(queryHelper)
```

## Future Improvements

## Consequences

### Positive

* greatly simplified querier implementation (no manual encoding/decoding)
* easy query client generation (can use existing grpc and swagger tools)
* no need for REST query implementations
* type safe query methods (generated via grpc plugin)
* going forward, there will be less breakage of query methods because of the
backwards compatibility guarantees provided by buf

### Negative

* all clients using the existing ABCI/REST queries will need to be refactored
for both the new GRPC/REST query paths as well as protobuf/proto-json encoded
data, but this is more or less unavoidable in the protobuf refactoring

### Neutral

## References
