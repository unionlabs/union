# ADR 033: Protobuf-based Inter-Module Communication

## Changelog

* 2020-10-05: Initial Draft

## Status

Proposed

## Abstract

This ADR introduces a system for permissioned inter-module communication leveraging the protobuf `Query` and `Msg`
service definitions defined in [ADR 021](./adr-021-protobuf-query-encoding.md) and
[ADR 031](./adr-031-msg-service.md) which provides:

* stable protobuf based module interfaces to potentially later replace the keeper paradigm
* stronger inter-module object capabilities (OCAPs) guarantees
* module accounts and sub-account authorization

## Context

In the current Cosmos SDK documentation on the [Object-Capability Model](../core/10-ocap.md), it is stated that:

> We assume that a thriving ecosystem of Cosmos SDK modules that are easy to compose into a blockchain application will contain faulty or malicious modules.

There is currently not a thriving ecosystem of Cosmos SDK modules. We hypothesize that this is in part due to:

1. lack of a stable v1.0 Cosmos SDK to build modules off of. Module interfaces are changing, sometimes dramatically, from
point release to point release, often for good reasons, but this does not create a stable foundation to build on.
2. lack of a properly implemented object capability or even object-oriented encapsulation system which makes refactors
of module keeper interfaces inevitable because the current interfaces are poorly constrained.

### `x/bank` Case Study

Currently the `x/bank` keeper gives pretty much unrestricted access to any module which references it. For instance, the
`SetBalance` method allows the caller to set the balance of any account to anything, bypassing even proper tracking of supply.

There appears to have been some later attempts to implement some semblance of OCAPs using module-level minting, staking
and burning permissions. These permissions allow a module to mint, burn or delegate tokens with reference to the module’s
own account. These permissions are actually stored as a `[]string` array on the `ModuleAccount` type in state.

However, these permissions don’t really do much. They control what modules can be referenced in the `MintCoins`,
`BurnCoins` and `DelegateCoins***` methods, but for one there is no unique object capability token that controls access —
just a simple string. So the `x/upgrade` module could mint tokens for the `x/staking` module simple by calling
`MintCoins(“staking”)`. Furthermore, all modules which have access to these keeper methods, also have access to
`SetBalance` negating any other attempt at OCAPs and breaking even basic object-oriented encapsulation.

## Decision

Based on [ADR-021](./adr-021-protobuf-query-encoding.md) and [ADR-031](./adr-031-msg-service.md), we introduce the
Inter-Module Communication framework for secure module authorization and OCAPs.
When implemented, this could also serve as an alternative to the existing paradigm of passing keepers between
modules. The approach outlined here-in is intended to form the basis of a Cosmos SDK v1.0 that provides the necessary
stability and encapsulation guarantees that allow a thriving module ecosystem to emerge.

Of particular note — the decision is to _enable_ this functionality for modules to adopt at their own discretion.
Proposals to migrate existing modules to this new paradigm will have to be a separate conversation, potentially
addressed as amendments to this ADR.

### New "Keeper" Paradigm

In [ADR 021](./adr-021-protobuf-query-encoding.md), a mechanism for using protobuf service definitions to define queriers
was introduced and in [ADR 31](./adr-031-msg-service.md), a mechanism for using protobuf service to define `Msg`s was added.
Protobuf service definitions generate two golang interfaces representing the client and server sides of a service plus
some helper code. Here is a minimal example for the bank `cosmos.bank.Msg/Send` message type:

```go
package bank

type MsgClient interface {
	Send(context.Context, *MsgSend, opts ...grpc.CallOption) (*MsgSendResponse, error)
}

type MsgServer interface {
	Send(context.Context, *MsgSend) (*MsgSendResponse, error)
}
```

[ADR 021](./adr-021-protobuf-query-encoding.md) and [ADR 31](./adr-031-msg-service.md) specifies how modules can implement the generated `QueryServer`
and `MsgServer` interfaces as replacements for the legacy queriers and `Msg` handlers respectively.

In this ADR we explain how modules can make queries and send `Msg`s to other modules using the generated `QueryClient`
and `MsgClient` interfaces and propose this mechanism as a replacement for the existing `Keeper` paradigm. To be clear,
this ADR does not necessitate the creation of new protobuf definitions or services. Rather, it leverages the same proto
based service interfaces already used by clients for inter-module communication.

Using this `QueryClient`/`MsgClient` approach has the following key benefits over exposing keepers to external modules:

1. Protobuf types are checked for breaking changes using [buf](https://buf.build/docs/breaking-overview) and because of
the way protobuf is designed this will give us strong backwards compatibility guarantees while allowing for forward
evolution.
2. The separation between the client and server interfaces will allow us to insert permission checking code in between
the two which checks if one module is authorized to send the specified `Msg` to the other module providing a proper
object capability system (see below).
3. The router for inter-module communication gives us a convenient place to handle rollback of transactions,
enabling atomicy of operations ([currently a problem](https://github.com/cosmos/cosmos-sdk/issues/8030)). Any failure within a module-to-module call would result in a failure of the entire
transaction

This mechanism has the added benefits of:

* reducing boilerplate through code generation, and
* allowing for modules in other languages either via a VM like CosmWasm or sub-processes using gRPC

### Inter-module Communication

To use the `Client` generated by the protobuf compiler we need a `grpc.ClientConn` [interface](https://github.com/grpc/grpc-go/blob/v1.49.x/clientconn.go#L441-L450)
implementation. For this we introduce
a new type, `ModuleKey`, which implements the `grpc.ClientConn` interface. `ModuleKey` can be thought of as the "private
key" corresponding to a module account, where authentication is provided through use of a special `Invoker()` function,
described in more detail below.

Blockchain users (external clients) use their account's private key to sign transactions containing `Msg`s where they are listed as signers (each
message specifies required signers with `Msg.GetSigner`). The authentication checks is performed by `AnteHandler`.

Here, we extend this process, by allowing modules to be identified in `Msg.GetSigners`. When a module wants to trigger the execution a `Msg` in another module,
its `ModuleKey` acts as the sender (through the `ClientConn` interface we describe below) and is set as a sole "signer". It's worth to note
that we don't use any cryptographic signature in this case.
For example, module `A` could use its `A.ModuleKey` to create `MsgSend` object for `/cosmos.bank.Msg/Send` transaction. `MsgSend` validation
will assure that the `from` account (`A.ModuleKey` in this case) is the signer.

Here's an example of a hypothetical module `foo` interacting with `x/bank`:

```go
package foo


type FooMsgServer {
  // ...

  bankQuery bank.QueryClient
  bankMsg   bank.MsgClient
}

func NewFooMsgServer(moduleKey RootModuleKey, ...) FooMsgServer {
  // ...

  return FooMsgServer {
    // ...
    modouleKey: moduleKey,
    bankQuery: bank.NewQueryClient(moduleKey),
    bankMsg: bank.NewMsgClient(moduleKey),
  }
}

func (foo *FooMsgServer) Bar(ctx context.Context, req *MsgBarRequest) (*MsgBarResponse, error) {
  balance, err := foo.bankQuery.Balance(&bank.QueryBalanceRequest{Address: fooMsgServer.moduleKey.Address(), Denom: "foo"})

  ...

  res, err := foo.bankMsg.Send(ctx, &bank.MsgSendRequest{FromAddress: fooMsgServer.moduleKey.Address(), ...})

  ...
}
```

This design is also intended to be extensible to cover use cases of more fine grained permissioning like minting by
denom prefix being restricted to certain modules (as discussed in
[#7459](https://github.com/cosmos/cosmos-sdk/pull/7459#discussion_r529545528)).

### `ModuleKey`s and `ModuleID`s

A `ModuleKey` can be thought of as a "private key" for a module account and a `ModuleID` can be thought of as the
corresponding "public key". From the [ADR 028](./adr-028-public-key-addresses.md), modules can have both a root module account and any number of sub-accounts
or derived accounts that can be used for different pools (ex. staking pools) or managed accounts (ex. group
accounts). We can also think of module sub-accounts as similar to derived keys - there is a root key and then some
derivation path. `ModuleID` is a simple struct which contains the module name and optional "derivation" path,
and forms its address based on the `AddressHash` method from [the ADR-028](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-028-public-key-addresses.md):

```go
type ModuleID struct {
  ModuleName string
  Path []byte
}

func (key ModuleID) Address() []byte {
  return AddressHash(key.ModuleName, key.Path)
}
```

In addition to being able to generate a `ModuleID` and address, a `ModuleKey` contains a special function called
`Invoker` which is the key to safe inter-module access. The `Invoker` creates an `InvokeFn` closure which is used as an `Invoke` method in
the `grpc.ClientConn` interface and under the hood is able to route messages to the appropriate `Msg` and `Query` handlers
performing appropriate security checks on `Msg`s. This allows for even safer inter-module access than keeper's whose
private member variables could be manipulated through reflection. Golang does not support reflection on a function
closure's captured variables and direct manipulation of memory would be needed for a truly malicious module to bypass
the `ModuleKey` security.

The two `ModuleKey` types are `RootModuleKey` and `DerivedModuleKey`:

```go
type Invoker func(callInfo CallInfo) func(ctx context.Context, request, response interface{}, opts ...interface{}) error

type CallInfo {
  Method string
  Caller ModuleID
}

type RootModuleKey struct {
  moduleName string
  invoker Invoker
}

func (rm RootModuleKey) Derive(path []byte) DerivedModuleKey { /* ... */}

type DerivedModuleKey struct {
  moduleName string
  path []byte
  invoker Invoker
}
```

A module can get access to a `DerivedModuleKey`, using the `Derive(path []byte)` method on `RootModuleKey` and then
would use this key to authenticate `Msg`s from a sub-account. Ex:

```go
package foo

func (fooMsgServer *MsgServer) Bar(ctx context.Context, req *MsgBar) (*MsgBarResponse, error) {
  derivedKey := fooMsgServer.moduleKey.Derive(req.SomePath)
  bankMsgClient := bank.NewMsgClient(derivedKey)
  res, err := bankMsgClient.Balance(ctx, &bank.MsgSend{FromAddress: derivedKey.Address(), ...})
  ...
}
```

In this way, a module can gain permissioned access to a root account and any number of sub-accounts and send
authenticated `Msg`s from these accounts. The `Invoker` `callInfo.Caller` parameter is used under the hood to
distinguish between different module accounts, but either way the function returned by `Invoker` only allows `Msg`s
from either the root or a derived module account to pass through.

Note that `Invoker` itself returns a function closure based on the `CallInfo` passed in. This will allow client implementations
in the future that cache the invoke function for each method type avoiding the overhead of hash table lookup.
This would reduce the performance overhead of this inter-module communication method to the bare minimum required for
checking permissions.

To re-iterate, the closure only allows access to authorized calls. There is no access to anything else regardless of any
name impersonation.

Below is a rough sketch of the implementation of `grpc.ClientConn.Invoke` for `RootModuleKey`:

```go
func (key RootModuleKey) Invoke(ctx context.Context, method string, args, reply interface{}, opts ...grpc.CallOption) error {
  f := key.invoker(CallInfo {Method: method, Caller: ModuleID {ModuleName: key.moduleName}})
  return f(ctx, args, reply)
}
```

### `AppModule` Wiring and Requirements

In [ADR 031](./adr-031-msg-service.md), the `AppModule.RegisterService(Configurator)` method was introduced. To support
inter-module communication, we extend the `Configurator` interface to pass in the `ModuleKey` and to allow modules to
specify their dependencies on other modules using `RequireServer()`:

```go
type Configurator interface {
   MsgServer() grpc.Server
   QueryServer() grpc.Server

   ModuleKey() ModuleKey
   RequireServer(msgServer interface{})
}
```

The `ModuleKey` is passed to modules in the `RegisterService` method itself so that `RegisterServices` serves as a single
entry point for configuring module services. This is intended to also have the side-effect of greatly reducing boilerplate in
`app.go`. For now, `ModuleKey`s will be created based on `AppModuleBasic.Name()`, but a more flexible system may be
introduced in the future. The `ModuleManager` will handle creation of module accounts behind the scenes.

Because modules do not get direct access to each other anymore, modules may have unfulfilled dependencies. To make sure
that module dependencies are resolved at startup, the `Configurator.RequireServer` method should be added. The `ModuleManager`
will make sure that all dependencies declared with `RequireServer` can be resolved before the app starts. An example
module `foo` could declare it's dependency on `x/bank` like this:

```go
package foo

func (am AppModule) RegisterServices(cfg Configurator) {
  cfg.RequireServer((*bank.QueryServer)(nil))
  cfg.RequireServer((*bank.MsgServer)(nil))
}
```

### Security Considerations

In addition to checking for `ModuleKey` permissions, a few additional security precautions will need to be taken by
the underlying router infrastructure.

#### Recursion and Re-entry

Recursive or re-entrant method invocations pose a potential security threat. This can be a problem if Module A
calls Module B and Module B calls module A again in the same call.

One basic way for the router system to deal with this is to maintain a call stack which prevents a module from
being referenced more than once in the call stack so that there is no re-entry. A `map[string]interface{}` table
in the router could be used to perform this security check.

#### Queries

Queries in Cosmos SDK are generally un-permissioned so allowing one module to query another module should not pose
any major security threats assuming basic precautions are taken. The basic precaution that the router system will
need to take is making sure that the `sdk.Context` passed to query methods does not allow writing to the store. This
can be done for now with a `CacheMultiStore` as is currently done for `BaseApp` queries.

### Internal Methods

In many cases, we may wish for modules to call methods on other modules which are not exposed to clients at all. For this
purpose, we add the `InternalServer` method to `Configurator`:

```go
type Configurator interface {
   MsgServer() grpc.Server
   QueryServer() grpc.Server
   InternalServer() grpc.Server
}
```

As an example, x/slashing's Slash must call x/staking's Slash, but we don't want to expose x/staking's Slash to end users
and clients.

Internal protobuf services will be defined in a corresponding `internal.proto` file in the given module's
proto package.

Services registered against `InternalServer` will be callable from other modules but not by external clients.

An alternative solution to internal-only methods could involve hooks / plugins as discussed [here](https://github.com/cosmos/cosmos-sdk/pull/7459#issuecomment-733807753).
A more detailed evaluation of a hooks / plugin system will be addressed later in follow-ups to this ADR or as a separate
ADR.

### Authorization

By default, the inter-module router requires that messages are sent by the first signer returned by `GetSigners`. The
inter-module router should also accept authorization middleware such as that provided by [ADR 030](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-030-authz-module.md).
This middleware will allow accounts to otherwise specific module accounts to perform actions on their behalf.
Authorization middleware should take into account the need to grant certain modules effectively "admin" privileges to
other modules. This will be addressed in separate ADRs or updates to this ADR.

### Future Work

Other future improvements may include:

* custom code generation that:
    * simplifies interfaces (ex. generates code with `sdk.Context` instead of `context.Context`)
    * optimizes inter-module calls - for instance caching resolved methods after first invocation
* combining `StoreKey`s and `ModuleKey`s into a single interface so that modules have a single OCAPs handle
* code generation which makes inter-module communication more performant
* decoupling `ModuleKey` creation from `AppModuleBasic.Name()` so that app's can override root module account names
* inter-module hooks and plugins

## Alternatives

### MsgServices vs `x/capability`

The `x/capability` module does provide a proper object-capability implementation that can be used by any module in the
Cosmos SDK and could even be used for inter-module OCAPs as described in [\#5931](https://github.com/cosmos/cosmos-sdk/issues/5931).

The advantages of the approach described in this ADR are mostly around how it integrates with other parts of the Cosmos SDK,
specifically:

* protobuf so that:
    * code generation of interfaces can be leveraged for a better dev UX
    * module interfaces are versioned and checked for breakage using [buf](https://docs.buf.build/breaking-overview)
* sub-module accounts as per ADR 028
* the general `Msg` passing paradigm and the way signers are specified by `GetSigners`

Also, this is a complete replacement for keepers and could be applied to _all_ inter-module communication whereas the
`x/capability` approach in #5931 would need to be applied method by method.

## Consequences

### Backwards Compatibility

This ADR is intended to provide a pathway to a scenario where there is greater long term compatibility between modules.
In the short-term, this will likely result in breaking certain `Keeper` interfaces which are too permissive and/or
replacing `Keeper` interfaces altogether.

### Positive

* an alternative to keepers which can more easily lead to stable inter-module interfaces
* proper inter-module OCAPs
* improved module developer DevX, as commented on by several particpants on
    [Architecture Review Call, Dec 3](https://hackmd.io/E0wxxOvRQ5qVmTf6N_k84Q)
* lays the groundwork for what can be a greatly simplified `app.go`
* router can be setup to enforce atomic transactions for module-to-module calls

### Negative

* modules which adopt this will need significant refactoring

### Neutral

## Test Cases [optional]

## References

* [ADR 021](./adr-021-protobuf-query-encoding.md)
* [ADR 031](./adr-031-msg-service.md)
* [ADR 028](./adr-028-public-key-addresses.md)
* [ADR 030 draft](https://github.com/cosmos/cosmos-sdk/pull/7105)
* [Object-Capability Model](https://docs.network.com/main/core/ocap)
