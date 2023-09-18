# ADR 019: Protocol Buffer State Encoding

## Changelog

* 2020 Feb 15: Initial Draft
* 2020 Feb 24: Updates to handle messages with interface fields
* 2020 Apr 27: Convert usages of `oneof` for interfaces to `Any`
* 2020 May 15: Describe `cosmos_proto` extensions and amino compatibility
* 2020 Dec 4: Move and rename `MarshalAny` and `UnmarshalAny` into the `codec.Codec` interface.
* 2021 Feb 24: Remove mentions of `HybridCodec`, which has been abandoned in [#6843](https://github.com/cosmos/cosmos-sdk/pull/6843).

## Status

Accepted

## Context

Currently, the Cosmos SDK utilizes [go-amino](https://github.com/tendermint/go-amino/) for binary
and JSON object encoding over the wire bringing parity between logical objects and persistence objects.

From the Amino docs:

> Amino is an object encoding specification. It is a subset of Proto3 with an extension for interface
> support. See the [Proto3 spec](https://developers.google.com/protocol-buffers/docs/proto3) for more
> information on Proto3, which Amino is largely compatible with (but not with Proto2).
>
> The goal of the Amino encoding protocol is to bring parity into logic objects and persistence objects.

Amino also aims to have the following goals (not a complete list):

* Binary bytes must be decode-able with a schema.
* Schema must be upgradeable.
* The encoder and decoder logic must be reasonably simple.

However, we believe that Amino does not fulfill these goals completely and does not fully meet the
needs of a truly flexible cross-language and multi-client compatible encoding protocol in the Cosmos SDK.
Namely, Amino has proven to be a big pain-point in regards to supporting object serialization across
clients written in various languages while providing virtually little in the way of true backwards
compatibility and upgradeability. Furthermore, through profiling and various benchmarks, Amino has
been shown to be an extremely large performance bottleneck in the Cosmos SDK <sup>1</sup>. This is
largely reflected in the performance of simulations and application transaction throughput.

Thus, we need to adopt an encoding protocol that meets the following criteria for state serialization:

* Language agnostic
* Platform agnostic
* Rich client support and thriving ecosystem
* High performance
* Minimal encoded message size
* Codegen-based over reflection-based
* Supports backward and forward compatibility

Note, migrating away from Amino should be viewed as a two-pronged approach, state and client encoding.
This ADR focuses on state serialization in the Cosmos SDK state machine. A corresponding ADR will be
made to address client-side encoding.

## Decision

We will adopt [Protocol Buffers](https://developers.google.com/protocol-buffers) for serializing
persisted structured data in the Cosmos SDK while providing a clean mechanism and developer UX for
applications wishing to continue to use Amino. We will provide this mechanism by updating modules to
accept a codec interface, `Marshaler`, instead of a concrete Amino codec. Furthermore, the Cosmos SDK
will provide two concrete implementations of the `Marshaler` interface: `AminoCodec` and `ProtoCodec`.

* `AminoCodec`: Uses Amino for both binary and JSON encoding.
* `ProtoCodec`: Uses Protobuf for both binary and JSON encoding.

Modules will use whichever codec that is instantiated in the app. By default, the Cosmos SDK's `simapp`
instantiates a `ProtoCodec` as the concrete implementation of `Marshaler`, inside the `MakeTestEncodingConfig`
function. This can be easily overwritten by app developers if they so desire.

The ultimate goal will be to replace Amino JSON encoding with Protobuf encoding and thus have
modules accept and/or extend `ProtoCodec`. Until then, Amino JSON is still provided for legacy use-cases.
A handful of places in the Cosmos SDK still have Amino JSON hardcoded, such as the Legacy API REST endpoints
and the `x/params` store. They are planned to be converted to Protobuf in a gradual manner.

### Module Codecs

Modules that do not require the ability to work with and serialize interfaces, the path to Protobuf
migration is pretty straightforward. These modules are to simply migrate any existing types that
are encoded and persisted via their concrete Amino codec to Protobuf and have their keeper accept a
`Marshaler` that will be a `ProtoCodec`. This migration is simple as things will just work as-is.

Note, any business logic that needs to encode primitive types like `bool` or `int64` should use
[gogoprotobuf](https://github.com/cosmos/gogoproto) Value types.

Example:

```go
  ts, err := gogotypes.TimestampProto(completionTime)
  if err != nil {
    // ...
  }

  bz := cdc.MustMarshal(ts)
```

However, modules can vary greatly in purpose and design and so we must support the ability for modules
to be able to encode and work with interfaces (e.g. `Account` or `Content`). For these modules, they
must define their own codec interface that extends `Marshaler`. These specific interfaces are unique
to the module and will contain method contracts that know how to serialize the needed interfaces.

Example:

```go
// x/auth/types/codec.go

type Codec interface {
  codec.Codec

  MarshalAccount(acc exported.Account) ([]byte, error)
  UnmarshalAccount(bz []byte) (exported.Account, error)

  MarshalAccountJSON(acc exported.Account) ([]byte, error)
  UnmarshalAccountJSON(bz []byte) (exported.Account, error)
}
```

### Usage of `Any` to encode interfaces

In general, module-level .proto files should define messages which encode interfaces
using [`google.protobuf.Any`](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
After [extension discussion](https://github.com/cosmos/cosmos-sdk/issues/6030),
this was chosen as the preferred alternative to application-level `oneof`s
as in our original protobuf design. The arguments in favor of `Any` can be
summarized as follows:

* `Any` provides a simpler, more consistent client UX for dealing with
interfaces than app-level `oneof`s that will need to be coordinated more
carefully across applications. Creating a generic transaction
signing library using `oneof`s may be cumbersome and critical logic may need
to be reimplemented for each chain
* `Any` provides more resistance against human error than `oneof`
* `Any` is generally simpler to implement for both modules and apps

The main counter-argument to using `Any` centers around its additional space
and possibly performance overhead. The space overhead could be dealt with using
compression at the persistence layer in the future and the performance impact
is likely to be small. Thus, not using `Any` is seem as a pre-mature optimization,
with user experience as the higher order concern.

Note, that given the Cosmos SDK's decision to adopt the `Codec` interfaces described
above, apps can still choose to use `oneof` to encode state and transactions
but it is not the recommended approach. If apps do choose to use `oneof`s
instead of `Any` they will likely lose compatibility with client apps that
support multiple chains. Thus developers should think carefully about whether
they care more about what is possibly a pre-mature optimization or end-user
and client developer UX.

### Safe usage of `Any`

By default, the [gogo protobuf implementation of `Any`](https://pkg.go.dev/github.com/cosmos/gogoproto/types)
uses [global type registration]( https://github.com/cosmos/gogoproto/blob/master/proto/properties.go#L540)
to decode values packed in `Any` into concrete
go types. This introduces a vulnerability where any malicious module
in the dependency tree could register a type with the global protobuf registry
and cause it to be loaded and unmarshaled by a transaction that referenced
it in the `type_url` field.

To prevent this, we introduce a type registration mechanism for decoding `Any`
values into concrete types through the `InterfaceRegistry` interface which
bears some similarity to type registration with Amino:

```go
type InterfaceRegistry interface {
    // RegisterInterface associates protoName as the public name for the
    // interface passed in as iface
    // Ex:
    //   registry.RegisterInterface("cosmos_sdk.Msg", (*sdk.Msg)(nil))
    RegisterInterface(protoName string, iface interface{})

    // RegisterImplementations registers impls as a concrete implementations of
    // the interface iface
    // Ex:
    //  registry.RegisterImplementations((*sdk.Msg)(nil), &MsgSend{}, &MsgMultiSend{})
    RegisterImplementations(iface interface{}, impls ...proto.Message)

}
```

In addition to serving as a whitelist, `InterfaceRegistry` can also serve
to communicate the list of concrete types that satisfy an interface to clients.

In .proto files:

* fields which accept interfaces should be annotated with `cosmos_proto.accepts_interface`
using the same full-qualified name passed as `protoName` to `InterfaceRegistry.RegisterInterface`
* interface implementations should be annotated with `cosmos_proto.implements_interface`
using the same full-qualified name passed as `protoName` to `InterfaceRegistry.RegisterInterface`

In the future, `protoName`, `cosmos_proto.accepts_interface`, `cosmos_proto.implements_interface`
may be used via code generation, reflection &/or static linting.

The same struct that implements `InterfaceRegistry` will also implement an
interface `InterfaceUnpacker` to be used for unpacking `Any`s:

```go
type InterfaceUnpacker interface {
    // UnpackAny unpacks the value in any to the interface pointer passed in as
    // iface. Note that the type in any must have been registered with
    // RegisterImplementations as a concrete type for that interface
    // Ex:
    //    var msg sdk.Msg
    //    err := ctx.UnpackAny(any, &msg)
    //    ...
    UnpackAny(any *Any, iface interface{}) error
}
```

Note that `InterfaceRegistry` usage does not deviate from standard protobuf
usage of `Any`, it just introduces a security and introspection layer for
golang usage.

`InterfaceRegistry` will be a member of `ProtoCodec`
described above. In order for modules to register interface types, app modules
can optionally implement the following interface:

```go
type InterfaceModule interface {
    RegisterInterfaceTypes(InterfaceRegistry)
}
```

The module manager will include a method to call `RegisterInterfaceTypes` on
every module that implements it in order to populate the `InterfaceRegistry`.

### Using `Any` to encode state

The Cosmos SDK will provide support methods `MarshalInterface` and `UnmarshalInterface` to hide a complexity of wrapping interface types into `Any` and allow easy serialization.

```go
import "github.com/cosmos/cosmos-sdk/codec"

// note: eviexported.Evidence is an interface type
func MarshalEvidence(cdc codec.BinaryCodec, e eviexported.Evidence) ([]byte, error) {
	return cdc.MarshalInterface(e)
}

func UnmarshalEvidence(cdc codec.BinaryCodec, bz []byte) (eviexported.Evidence, error) {
	var evi eviexported.Evidence
	err := cdc.UnmarshalInterface(&evi, bz)
    return err, nil
}
```

### Using `Any` in `sdk.Msg`s

A similar concept is to be applied for messages that contain interfaces fields.
For example, we can define `MsgSubmitEvidence` as follows where `Evidence` is
an interface:

```protobuf
// x/evidence/types/types.proto

message MsgSubmitEvidence {
  bytes submitter = 1
    [
      (gogoproto.casttype) = "github.com/cosmos/cosmos-sdk/types.AccAddress"
    ];
  google.protobuf.Any evidence = 2;
}
```

Note that in order to unpack the evidence from `Any` we do need a reference to
`InterfaceRegistry`. In order to reference evidence in methods like
`ValidateBasic` which shouldn't have to know about the `InterfaceRegistry`, we
introduce an `UnpackInterfaces` phase to deserialization which unpacks
interfaces before they're needed.

### Unpacking Interfaces

To implement the `UnpackInterfaces` phase of deserialization which unpacks
interfaces wrapped in `Any` before they're needed, we create an interface
that `sdk.Msg`s and other types can implement:

```go
type UnpackInterfacesMessage interface {
  UnpackInterfaces(InterfaceUnpacker) error
}
```

We also introduce a private `cachedValue interface{}` field onto the `Any`
struct itself with a public getter `GetCachedValue() interface{}`.

The `UnpackInterfaces` method is to be invoked during message deserialization right
after `Unmarshal` and any interface values packed in `Any`s will be decoded
and stored in `cachedValue` for reference later.

Then unpacked interface values can safely be used in any code afterwards
without knowledge of the `InterfaceRegistry`
and messages can introduce a simple getter to cast the cached value to the
correct interface type.

This has the added benefit that unmarshaling of `Any` values only happens once
during initial deserialization rather than every time the value is read. Also,
when `Any` values are first packed (for instance in a call to
`NewMsgSubmitEvidence`), the original interface value is cached so that
unmarshaling isn't needed to read it again.

`MsgSubmitEvidence` could implement `UnpackInterfaces`, plus a convenience getter
`GetEvidence` as follows:

```go
func (msg MsgSubmitEvidence) UnpackInterfaces(ctx sdk.InterfaceRegistry) error {
  var evi eviexported.Evidence
  return ctx.UnpackAny(msg.Evidence, *evi)
}

func (msg MsgSubmitEvidence) GetEvidence() eviexported.Evidence {
  return msg.Evidence.GetCachedValue().(eviexported.Evidence)
}
```

### Amino Compatibility

Our custom implementation of `Any` can be used transparently with Amino if used
with the proper codec instance. What this means is that interfaces packed within
`Any`s will be amino marshaled like regular Amino interfaces (assuming they
have been registered properly with Amino).

In order for this functionality to work:

* **all legacy code must use `*codec.LegacyAmino` instead of `*amino.Codec` which is
  now a wrapper which properly handles `Any`**
* **all new code should use `Marshaler` which is compatible with both amino and
  protobuf**
* Also, before v0.39, `codec.LegacyAmino` will be renamed to `codec.LegacyAmino`.

### Why Wasn't X Chosen Instead

For a more complete comparison to alternative protocols, see [here](https://codeburst.io/json-vs-protocol-buffers-vs-flatbuffers-a4247f8bda6f).

### Cap'n Proto

While [Cap’n Proto](https://capnproto.org/) does seem like an advantageous alternative to Protobuf
due to it's native support for interfaces/generics and built in canonicalization, it does lack the
rich client ecosystem compared to Protobuf and is a bit less mature.

### FlatBuffers

[FlatBuffers](https://google.github.io/flatbuffers/) is also a potentially viable alternative, with the
primary difference being that FlatBuffers does not need a parsing/unpacking step to a secondary
representation before you can access data, often coupled with per-object memory allocation.

However, it would require great efforts into research and full understanding the scope of the migration
and path forward -- which isn't immediately clear. In addition, FlatBuffers aren't designed for
untrusted inputs.

## Future Improvements & Roadmap

In the future we may consider a compression layer right above the persistence
layer which doesn't change tx or merkle tree hashes, but reduces the storage
overhead of `Any`. In addition, we may adopt protobuf naming conventions which
make type URLs a bit more concise while remaining descriptive.

Additional code generation support around the usage of `Any` is something that
could also be explored in the future to make the UX for go developers more
seamless.

## Consequences

### Positive

* Significant performance gains.
* Supports backward and forward type compatibility.
* Better support for cross-language clients.

### Negative

* Learning curve required to understand and implement Protobuf messages.
* Slightly larger message size due to use of `Any`, although this could be offset
  by a compression layer in the future

### Neutral

## References

1. https://github.com/cosmos/cosmos-sdk/issues/4977
2. https://github.com/cosmos/cosmos-sdk/issues/5444
