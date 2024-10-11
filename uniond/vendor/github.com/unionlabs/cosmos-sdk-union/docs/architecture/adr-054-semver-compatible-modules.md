# ADR 054: Semver Compatible SDK Modules

## Changelog

* 2022-04-27: First draft

## Status

DRAFT

## Abstract

In order to move the Cosmos SDK to a system of decoupled semantically versioned
modules which can be composed in different combinations (ex. staking v3 with
bank v1 and distribution v2), we need to reassess how we organize the API surface
of modules to avoid problems with go semantic import versioning and
circular dependencies. This ADR explores various approaches we can take to
addressing these issues.

## Context

There has been [a fair amount of desire](https://github.com/cosmos/cosmos-sdk/discussions/10162)
in the community for semantic versioning in the SDK and there has been significant
movement to splitting SDK modules into [standalone go modules](https://github.com/cosmos/cosmos-sdk/issues/11899).
Both of these will ideally allow the ecosystem to move faster because we won't
be waiting for all dependencies to update synchronously. For instance, we could
have 3 versions of the core SDK compatible with the latest 2 releases of
CosmWasm as well as 4 different versions of staking . This sort of setup would
allow early adopters to aggressively integrate new versions, while allowing
more conservative users to be selective about which versions they're ready for.

In order to achieve this, we need to solve the following problems:

1. because of the way [go semantic import versioning](https://research.swtch.com/vgo-import) (SIV)
   works, moving to SIV naively will actually make it harder to achieve these goals
2. circular dependencies between modules need to be broken to actually release
   many modules in the SDK independently
3. pernicious minor version incompatibilities introduced through correctly
   [evolving protobuf schemas](https://protobuf.dev/programming-guides/proto3/#updating)
   without correct [unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering)

Note that all the following discussion assumes that the proto file versioning and state machine versioning of a module
are distinct in that:

* proto files are maintained in a non-breaking way (using something
  like [buf breaking](https://docs.buf.build/breaking/overview)
  to ensure all changes are backwards compatible)
* proto file versions get bumped much less frequently, i.e. we might maintain `cosmos.bank.v1` through many versions
  of the bank module state machine
* state machine breaking changes are more common and ideally this is what we'd want to semantically version with
  go modules, ex. `x/bank/v2`, `x/bank/v3`, etc.

### Problem 1: Semantic Import Versioning Compatibility

Consider we have a module `foo` which defines the following `MsgDoSomething` and that we've released its state
machine in go module `example.com/foo`:

```protobuf
package foo.v1;

message MsgDoSomething {
  string sender = 1;
  uint64 amount = 2;
}

service Msg {
  DoSomething(MsgDoSomething) returns (MsgDoSomethingResponse);
}
```

Now consider that we make a revision to this module and add a new `condition` field to `MsgDoSomething` and also
add a new validation rule on `amount` requiring it to be non-zero, and that following go semantic versioning we
release the next state machine version of `foo` as `example.com/foo/v2`.

```protobuf
// Revision 1
package foo.v1;

message MsgDoSomething {
  string sender = 1;
  
  // amount must be a non-zero integer.
  uint64 amount = 2;
  
  // condition is an optional condition on doing the thing.
  //
  // Since: Revision 1
  Condition condition = 3;
}
```

Approaching this naively, we would generate the protobuf types for the initial
version of `foo` in `example.com/foo/types` and we would generate the protobuf
types for the second version in `example.com/foo/v2/types`.

Now let's say we have a module `bar` which talks to `foo` using this keeper
interface which `foo` provides:

```go
type FooKeeper interface {
	DoSomething(MsgDoSomething) error
}
```

#### Scenario A: Backward Compatibility: Newer Foo, Older Bar

Imagine we have a chain which uses both `foo` and `bar` and wants to upgrade to
`foo/v2`, but the `bar` module has not upgraded to `foo/v2`.

In this case, the chain will not be able to upgrade to `foo/v2` until `bar`
has upgraded its references to `example.com/foo/types.MsgDoSomething` to
`example.com/foo/v2/types.MsgDoSomething`.

Even if `bar`'s usage of `MsgDoSomething` has not changed at all, the upgrade
will be impossible without this change because `example.com/foo/types.MsgDoSomething`
and `example.com/foo/v2/types.MsgDoSomething` are fundamentally different
incompatible structs in the go type system.

#### Scenario B: Forward Compatibility: Older Foo, Newer Bar

Now let's consider the reverse scenario, where `bar` upgrades to `foo/v2`
by changing the `MsgDoSomething` reference to `example.com/foo/v2/types.MsgDoSomething`
and releases that as `bar/v2` with some other changes that a chain wants.
The chain, however, has decided that it thinks the changes in `foo/v2` are too
risky and that it'd prefer to stay on the initial version of `foo`.

In this scenario, it is impossible to upgrade to `bar/v2` without upgrading
to `foo/v2` even if `bar/v2` would have worked 100% fine with `foo` other
than changing the import path to `MsgDoSomething` (meaning that `bar/v2`
doesn't actually use any new features of `foo/v2`).

Now because of the way go semantic import versioning works, we are locked
into either using `foo` and `bar` OR `foo/v2` and `bar/v2`. We cannot have
`foo` + `bar/v2` OR `foo/v2` + `bar`. The go type system doesn't allow this
even if both versions of these modules are otherwise compatible with each
other.

#### Naive Mitigation

A naive approach to fixing this would be to not regenerate the protobuf types
in `example.com/foo/v2/types` but instead just update `example.com/foo/types`
to reflect the changes needed for `v2` (adding `condition` and requiring
`amount` to be non-zero). Then we could release a patch of `example.com/foo/types`
with this update and use that for `foo/v2`. But this change is state machine
breaking for `v1`. It requires changing the `ValidateBasic` method to reject
the case where `amount` is zero, and it adds the `condition` field which
should be rejected based
on [ADR 020 unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering).
So adding these changes as a patch on `v1` is actually incorrect based on semantic
versioning. Chains that want to stay on `v1` of `foo` should not
be importing these changes because they are incorrect for `v1.`

### Problem 2: Circular dependencies

None of the above approaches allow `foo` and `bar` to be separate modules
if for some reason `foo` and `bar` depend on each other in different ways.
For instance, we can't have `foo` import `bar/types` while `bar` imports
`foo/types`.

We have several cases of circular module dependencies in the SDK
(ex. staking, distribution and slashing) that are legitimate from a state machine
perspective. Without separating the API types out somehow, there would be
no way to independently semantically version these modules without some other
mitigation.

### Problem 3: Handling Minor Version Incompatibilities

Imagine that we solve the first two problems but now have a scenario where
`bar/v2` wants the option to use `MsgDoSomething.condition` which only `foo/v2`
supports. If `bar/v2` works with `foo` `v1` and sets `condition` to some non-nil
value, then `foo` will silently ignore this field resulting in a silent logic
possibly dangerous logic error. If `bar/v2` were able to check whether `foo` was
on `v1` or `v2` and dynamically, it could choose to only use `condition` when
`foo/v2` is available. Even if `bar/v2` were able to perform this check, however,
how do we know that it is always performing the check properly. Without
some sort of
framework-level [unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering),
it is hard to know whether these pernicious hard to detect bugs are getting into
our app and a client-server layer such as [ADR 033: Inter-Module Communication](./adr-033-protobuf-inter-module-comm.md)
may be needed to do this.

## Solutions

### Approach A) Separate API and State Machine Modules

One solution (first proposed in https://github.com/cosmos/cosmos-sdk/discussions/10582) is to isolate all protobuf
generated code into a separate module
from the state machine module. This would mean that we could have state machine
go modules `foo` and `foo/v2` which could use a types or API go module say
`foo/api`. This `foo/api` go module would be perpetually on `v1.x` and only
accept non-breaking changes. This would then allow other modules to be
compatible with either `foo` or `foo/v2` as long as the inter-module API only
depends on the types in `foo/api`. It would also allow modules `foo` and `bar`
to depend on each other in that both of them could depend on `foo/api` and
`bar/api` without `foo` directly depending on `bar` and vice versa.

This is similar to the naive mitigation described above except that it separates
the types into separate go modules which in and of itself could be used to
break circular module dependencies. It has the same problems as the naive solution,
otherwise, which we could rectify by:

1. removing all state machine breaking code from the API module (ex. `ValidateBasic` and any other interface methods)
2. embedding the correct file descriptors for unknown field filtering in the binary

#### Migrate all interface methods on API types to handlers

To solve 1), we need to remove all interface implementations from generated
types and instead use a handler approach which essentially means that given
a type `X`, we have some sort of resolver which allows us to resolve interface
implementations for that type (ex. `sdk.Msg` or `authz.Authorization`). For
example:

```go
func (k Keeper) DoSomething(msg MsgDoSomething) error {
	var validateBasicHandler ValidateBasicHandler
	err := k.resolver.Resolve(&validateBasicHandler, msg)
	if err != nil {
		return err
	}   
	
	err = validateBasicHandler.ValidateBasic()
	...
}
```

In the case of some methods on `sdk.Msg`, we could replace them with declarative
annotations. For instance, `GetSigners` can already be replaced by the protobuf
annotation `cosmos.msg.v1.signer`. In the future, we may consider some sort
of protobuf validation framework (like https://github.com/bufbuild/protoc-gen-validate
but more Cosmos-specific) to replace `ValidateBasic`.

#### Pinned FileDescriptor's

To solve 2), state machine modules must be able to specify what the version of
the protobuf files was that they were built against. For instance if the API
module for `foo` upgrades to `foo/v2`, the original `foo` module still needs
a copy of the original protobuf files it was built with so that ADR 020
unknown field filtering will reject `MsgDoSomething` when `condition` is
set.

The simplest way to do this may be to embed the protobuf `FileDescriptor`s into
the module itself so that these `FileDescriptor`s are used at runtime rather
than the ones that are built into the `foo/api` which may be different. Using
[buf build](https://docs.buf.build/build/usage#output-format), [go embed](https://pkg.go.dev/embed),
and a build script we can probably come up with a solution for embedding
`FileDescriptor`s into modules that is fairly straightforward.

#### Potential limitations to generated code

One challenge with this approach is that it places heavy restrictions on what
can go in API modules and requires that most of this is state machine breaking.
All or most of the code in the API module would be generated from protobuf
files, so we can probably control this with how code generation is done, but
it is a risk to be aware of.

For instance, we do code generation for the ORM that in the future could
contain optimizations that are state machine breaking. We
would either need to ensure very carefully that the optimizations aren't
actually state machine breaking in generated code or separate this generated code
out from the API module into the state machine module. Both of these mitigations
are potentially viable but the API module approach does require an extra level
of care to avoid these sorts of issues.

#### Minor Version Incompatibilities

This approach in and of itself does little to address any potential minor
version incompatibilities and the
requisite [unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering).
Likely some sort of client-server routing layer which does this check such as
[ADR 033: Inter-Module communication](./adr-033-protobuf-inter-module-comm.md)
is required to make sure that this is done properly. We could then allow
modules to perform a runtime check given a `MsgClient`, ex:

```go
func (k Keeper) CallFoo() error {
	if k.interModuleClient.MinorRevision(k.fooMsgClient) >= 2 {
		k.fooMsgClient.DoSomething(&MsgDoSomething{Condition: ...})
    } else {
        ...
    }
}
```

To do the unknown field filtering itself, the ADR 033 router would need to use
the [protoreflect API](https://pkg.go.dev/google.golang.org/protobuf/reflect/protoreflect)
to ensure that no fields unknown to the receiving module are set. This could
result in an undesirable performance hit depending on how complex this logic is.

### Approach B) Changes to Generated Code

An alternate approach to solving the versioning problem is to change how protobuf code is generated and move modules
mostly or completely in the direction of inter-module communication as described
in [ADR 033](./adr-033-protobuf-inter-module-comm.md).
In this paradigm, a module could generate all the types it needs internally - including the API types of other modules -
and talk to other modules via a client-server boundary. For instance, if `bar` needs to talk to `foo`, it could
generate its own version of `MsgDoSomething` as `bar/internal/foo/v1.MsgDoSomething` and just pass this to the
inter-module router which would somehow convert it to the version which foo needs (ex. `foo/internal.MsgDoSomething`).

Currently, two generated structs for the same protobuf type cannot exist in the same go binary without special
build flags (see https://protobuf.dev/reference/go/faq/#fix-namespace-conflict).
A relatively simple mitigation to this issue would be to set up the protobuf code to not register protobuf types
globally if they are generated in an `internal/` package. This will require modules to register their types manually
with the app-level level protobuf registry, this is similar to what modules already do with the `InterfaceRegistry`
and amino codec.

If modules _only_ do ADR 033 message passing then a naive and non-performant solution for
converting `bar/internal/foo/v1.MsgDoSomething`
to `foo/internal.MsgDoSomething` would be marshaling and unmarshaling in the ADR 033 router. This would break down if
we needed to expose protobuf types in `Keeper` interfaces because the whole point is to try to keep these types
`internal/` so that we don't end up with all the import version incompatibilities we've described above. However,
because of the issue with minor version incompatibilities and the need
for [unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering),
sticking with the `Keeper` paradigm instead of ADR 033 may be unviable to begin with.

A more performant solution (that could maybe be adapted to work with `Keeper` interfaces) would be to only expose
getters and setters for generated types and internally store data in memory buffers which could be passed from
one implementation to another in a zero-copy way.

For example, imagine this protobuf API with only getters and setters is exposed for `MsgSend`:

```go
type MsgSend interface {
	proto.Message
	GetFromAddress() string
	GetToAddress() string
	GetAmount() []v1beta1.Coin
    SetFromAddress(string)
    SetToAddress(string)
    SetAmount([]v1beta1.Coin)
}

func NewMsgSend() MsgSend { return &msgSendImpl{memoryBuffers: ...} }
```

Under the hood, `MsgSend` could be implemented based on some raw memory buffer in the same way
that [Cap'n Proto](https://capnproto.org)
and [FlatBuffers](https://google.github.io/flatbuffers/) so that we could convert between one version of `MsgSend`
and another without serialization (i.e. zero-copy). This approach would have the added benefits of allowing zero-copy
message passing to modules written in other languages such as Rust and accessed through a VM or FFI. It could also make
unknown field filtering in inter-module communication simpler if we require that all new fields are added in sequential
order, ex. just checking that no field `> 5` is set.

Also, we wouldn't have any issues with state machine breaking code on generated types because all the generated
code used in the state machine would actually live in the state machine module itself. Depending on how interface
types and protobuf `Any`s are used in other languages, however, it may still be desirable to take the handler
approach described in approach A. Either way, types implementing interfaces would still need to be registered
with an `InterfaceRegistry` as they are now because there would be no way to retrieve them via the global registry.

In order to simplify access to other modules using ADR 033, a public API module (maybe even one
[remotely generated by Buf](https://docs.buf.build/bsr/remote-generation/go)) could be used by client modules instead
of requiring to generate all client types internally.

The big downsides of this approach are that it requires big changes to how people use protobuf types and would be a
substantial rewrite of the protobuf code generator. This new generated code, however, could still be made compatible
with
the [`google.golang.org/protobuf/reflect/protoreflect`](https://pkg.go.dev/google.golang.org/protobuf/reflect/protoreflect)
API in order to work with all standard golang protobuf tooling.

It is possible that the naive approach of marshaling/unmarshaling in the ADR 033 router is an acceptable intermediate
solution if the changes to the code generator are seen as too complex. However, since all modules would likely need
to migrate to ADR 033 anyway with this approach, it might be better to do this all at once.

### Approach C) Don't address these issues

If the above solutions are seen as too complex, we can also decide not to do anything explicit to enable better module
version compatibility, and break circular dependencies.

In this case, when developers are confronted with the issues described above they can require dependencies to update in
sync (what we do now) or attempt some ad-hoc potentially hacky solution.

One approach is to ditch go semantic import versioning (SIV) altogether. Some people have commented that go's SIV
(i.e. changing the import path to `foo/v2`, `foo/v3`, etc.) is too restrictive and that it should be optional. The
golang maintainers disagree and only officially support semantic import versioning. We could, however, take the
contrarian perspective and get more flexibility by using 0.x-based versioning basically forever.

Module version compatibility could then be achieved using go.mod replace directives to pin dependencies to specific
compatible 0.x versions. For instance if we knew `foo` 0.2 and 0.3 were both compatible with `bar` 0.3 and 0.4, we
could use replace directives in our go.mod to stick to the versions of `foo` and `bar` we want. This would work as
long as the authors of `foo` and `bar` avoid incompatible breaking changes between these modules.

Or, if developers choose to use semantic import versioning, they can attempt the naive solution described above
and would also need to use special tags and replace directives to make sure that modules are pinned to the correct
versions.

Note, however, that all of these ad-hoc approaches, would be vulnerable to the minor version compatibility issues
described above unless [unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering)
is properly addressed.

### Approach D) Avoid protobuf generated code in public APIs

An alternative approach would be to avoid protobuf generated code in public module APIs. This would help avoid the
discrepancy between state machine versions and client API versions at the module to module boundaries. It would mean
that we wouldn't do inter-module message passing based on ADR 033, but rather stick to the existing keeper approach
and take it one step further by avoiding any protobuf generated code in the keeper interface methods.

Using this approach, our `foo.Keeper.DoSomething` method wouldn't have the generated `MsgDoSomething` struct (which
comes from the protobuf API), but instead positional parameters. Then in order for `foo/v2` to support the `foo/v1`
keeper it would simply need to implement both the v1 and v2 keeper APIs. The `DoSomething` method in v2 could have the
additional `condition` parameter, but this wouldn't be present in v1 at all so there would be no danger of a client
accidentally setting this when it isn't available. 

So this approach would avoid the challenge around minor version incompatibilities because the existing module keeper
API would not get new fields when they are added to protobuf files.

Taking this approach, however, would likely require making all protobuf generated code internal in order to prevent
it from leaking into the keeper API. This means we would still need to modify the protobuf code generator to not
register `internal/` code with the global registry, and we would still need to manually register protobuf
`FileDescriptor`s (this is probably true in all scenarios). It may, however, be possible to avoid needing to refactor
interface methods on generated types to handlers.

Also, this approach doesn't address what would be done in scenarios where modules still want to use the message router.
Either way, we probably still want a way to pass messages from one module to another router safely even if it's just for
use cases like `x/gov`, `x/authz`, CosmWasm, etc. That would still require most of the things outlined in approach (B),
although we could advise modules to prefer keepers for communicating with other modules.

The biggest downside of this approach is probably that it requires a strict refactoring of keeper interfaces to avoid
generated code leaking into the API. This may result in cases where we need to duplicate types that are already defined
in proto files and then write methods for converting between the golang and protobuf version. This may end up in a lot
of unnecessary boilerplate and that may discourage modules from actually adopting it and achieving effective version
compatibility. Approaches (A) and (B), although heavy handed initially, aim to provide a system which once adopted
more or less gives the developer version compatibility for free with minimal boilerplate. Approach (D) may not be able
to provide such a straightforward system since it requires a golang API to be defined alongside a protobuf API in a
way that requires duplication and differing sets of design principles (protobuf APIs encourage additive changes
while golang APIs would forbid it).

Other downsides to this approach are:

* no clear roadmap to supporting modules in other languages like Rust
* doesn't get us any closer to proper object capability security (one of the goals of ADR 033)
* ADR 033 needs to be done properly anyway for the set of use cases which do need it

## Decision

The latest **DRAFT** proposal is:

1. we are alignment on adopting [ADR 033](./adr-033-protobuf-inter-module-comm.md) not just as an addition to the
   framework, but as a core replacement to the keeper paradigm entirely.
2. the ADR 033 inter-module router will accommodate any variation of approach (A) or (B) given the following rules:
   a. if the client type is the same as the server type then pass it directly through,
   b. if both client and server use the zero-copy generated code wrappers (which still need to be defined), then pass
   the memory buffers from one wrapper to the other, or
   c. marshal/unmarshal types between client and server.

This approach will allow for both maximal correctness and enable a clear path to enabling modules within in other
languages, possibly executed within a WASM VM.

### Minor API Revisions

To declare minor API revisions of proto files, we propose the following guidelines (which were already documented
in [cosmos.app.v1alpha module options](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/app/v1alpha1/module.proto):

* proto packages which are revised from their initial version (considered revision `0`) should include a `package`
* comment in some .proto file containing the test `Revision N` at the start of a comment line where `N` is the current
revision number.
* all fields, messages, etc. added in a version beyond the initial revision should add a comment at the start of a
comment line of the form `Since: Revision N` where `N` is the non-zero revision it was added.

It is advised that there is a 1:1 correspondence between a state machine module and versioned set of proto files
which are versioned either as a buf module a go API module or both. If the buf schema registry is used, the version of
this buf module should always be `1.N` where `N` corresponds to the package revision. Patch releases should be used when
only documentation comments are updated. It is okay to include proto packages named `v2`, `v3`, etc. in this same
`1.N` versioned buf module (ex. `cosmos.bank.v2`) as long as all these proto packages consist of a single API intended
to be served by a single SDK module.

### Introspecting Minor API Revisions

In order for modules to introspect the minor API revision of peer modules, we propose adding the following method
to `cosmossdk.io/core/intermodule.Client`:

```go
ServiceRevision(ctx context.Context, serviceName string) uint64
```

Modules could all this using the service name statically generated by the go grpc code generator:

```go
intermoduleClient.ServiceRevision(ctx, bankv1beta1.Msg_ServiceDesc.ServiceName)
```

In the future, we may decide to extend the code generator used for protobuf services to add a field
to client types which does this check more concisely, ex:

```go
package bankv1beta1

type MsgClient interface {
	Send(context.Context, MsgSend) (MsgSendResponse, error)
	ServiceRevision(context.Context) uint64
}
```

### Unknown Field Filtering

To correctly perform [unknown field filtering](./adr-020-protobuf-transaction-encoding.md#unknown-field-filtering),
the inter-module router can do one of the following:

* use the `protoreflect` API for messages which support that
* for gogo proto messages, marshal and use the existing `codec/unknownproto` code
* for zero-copy messages, do a simple check on the highest set field number (assuming we can require that fields are
  adding consecutively in increasing order)

### `FileDescriptor` Registration

Because a single go binary may contain different versions of the same generated protobuf code, we cannot rely on the
global protobuf registry to contain the correct `FileDescriptor`s. Because `appconfig` module configuration is itself
written in protobuf, we would like to load the `FileDescriptor`s for a module before loading a module itself. So we
will provide ways to register `FileDescriptor`s at module registration time before instantiation. We propose the
following `cosmossdk.io/core/appmodule.Option` constructors for the various cases of how `FileDescriptor`s may be
packaged:

```go
package appmodule

// this can be used when we are using google.golang.org/protobuf compatible generated code
// Ex:
//   ProtoFiles(bankv1beta1.File_cosmos_bank_v1beta1_module_proto)
func ProtoFiles(file []protoreflect.FileDescriptor) Option {}

// this can be used when we are using gogo proto generated code.
func GzippedProtoFiles(file [][]byte) Option {}

// this can be used when we are using buf build to generated a pinned file descriptor
func ProtoImage(protoImage []byte) Option {}
```

This approach allows us to support several ways protobuf files might be generated:

* proto files generated internally to a module (use `ProtoFiles`)
* the API module approach with pinned file descriptors (use `ProtoImage`)
* gogo proto (use `GzippedProtoFiles`)

### Module Dependency Declaration

One risk of ADR 033 is that dependencies are called at runtime which are not present in the loaded set of SDK modules.  
Also we want modules to have a way to define a minimum dependency API revision that they require. Therefore, all
modules should declare their set of dependencies upfront. These dependencies could be defined when a module is
instantiated, but ideally we know what the dependencies are before instantiation and can statically look at an app
config and determine whether the set of modules. For example, if `bar` requires `foo` revision `>= 1`, then we
should be able to know this when creating an app config with two versions of `bar` and `foo`.

We propose defining these dependencies in the proto options of the module config object itself.

### Interface Registration

We will also need to define how interface methods are defined on types that are serialized as `google.protobuf.Any`'s.
In light of the desire to support modules in other languages, we may want to think of solutions that will accommodate
other languages such as plugins described briefly in [ADR 033](./adr-033-protobuf-inter-module-comm.md#internal-methods).

### Testing

In order to ensure that modules are indeed with multiple versions of their dependencies, we plan to provide specialized
unit and integration testing infrastructure that automatically tests multiple versions of dependencies.

#### Unit Testing

Unit tests should be conducted inside SDK modules by mocking their dependencies. In a full ADR 033 scenario,
this means that all interaction with other modules is done via the inter-module router, so mocking of dependencies
means mocking their msg and query server implementations. We will provide both a test runner and fixture to make this
streamlined. The key thing that the test runner should do to test compatibility is to test all combinations of
dependency API revisions. This can be done by taking the file descriptors for the dependencies, parsing their comments
to determine the revisions various elements were added, and then created synthetic file descriptors for each revision
by subtracting elements that were added later.

Here is a proposed API for the unit test runner and fixture:

```go
package moduletesting

import (
	"context"
	"testing"

	"cosmossdk.io/core/intermodule"
	"cosmossdk.io/depinject"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
)

type TestFixture interface {
	context.Context
	intermodule.Client // for making calls to the module we're testing
	BeginBlock()
	EndBlock()
}

type UnitTestFixture interface {
	TestFixture
	grpc.ServiceRegistrar // for registering mock service implementations
}

type UnitTestConfig struct {
	ModuleConfig              proto.Message    // the module's config object
	DepinjectConfig           depinject.Config // optional additional depinject config options
	DependencyFileDescriptors []protodesc.FileDescriptorProto // optional dependency file descriptors to use instead of the global registry
}

// Run runs the test function for all combinations of dependency API revisions.
func (cfg UnitTestConfig) Run(t *testing.T, f func(t *testing.T, f UnitTestFixture)) {
	// ...
}
```

Here is an example for testing bar calling foo which takes advantage of conditional service revisions in the expected
mock arguments:

```go
func TestBar(t *testing.T) {
    UnitTestConfig{ModuleConfig: &foomodulev1.Module{}}.Run(t, func (t *testing.T, f moduletesting.UnitTestFixture) {
        ctrl := gomock.NewController(t)
        mockFooMsgServer := footestutil.NewMockMsgServer()
        foov1.RegisterMsgServer(f, mockFooMsgServer)
        barMsgClient := barv1.NewMsgClient(f)
		if f.ServiceRevision(foov1.Msg_ServiceDesc.ServiceName) >= 1 {
            mockFooMsgServer.EXPECT().DoSomething(gomock.Any(), &foov1.MsgDoSomething{
				...,
				Condition: ..., // condition is expected in revision >= 1
            }).Return(&foov1.MsgDoSomethingResponse{}, nil)
        } else {
            mockFooMsgServer.EXPECT().DoSomething(gomock.Any(), &foov1.MsgDoSomething{...}).Return(&foov1.MsgDoSomethingResponse{}, nil)
        }
        res, err := barMsgClient.CallFoo(f, &MsgCallFoo{})
        ...
    })
}
```

The unit test runner would make sure that no dependency mocks return arguments which are invalid for the service
revision being tested to ensure that modules don't incorrectly depend on functionality not present in a given revision.

#### Integration Testing

An integration test runner and fixture would also be provided which instead of using mocks would test actual module
dependencies in various combinations. Here is the proposed API:

```go
type IntegrationTestFixture interface {
    TestFixture
}

type IntegrationTestConfig struct {
    ModuleConfig     proto.Message    // the module's config object
    DependencyMatrix map[string][]proto.Message // all the dependent module configs
}

// Run runs the test function for all combinations of dependency modules.
func (cfg IntegationTestConfig) Run(t *testing.T, f func (t *testing.T, f IntegrationTestFixture)) {
    // ...
}
```

And here is an example with foo and bar:

```go
func TestBarIntegration(t *testing.T) {
    IntegrationTestConfig{
        ModuleConfig: &barmodulev1.Module{},
        DependencyMatrix: map[string][]proto.Message{
            "runtime": []proto.Message{ // test against two versions of runtime
                &runtimev1.Module{},
                &runtimev2.Module{},
            },
            "foo": []proto.Message{ // test against three versions of foo
                &foomodulev1.Module{},
                &foomodulev2.Module{},
                &foomodulev3.Module{},
            }
        }   
    }.Run(t, func (t *testing.T, f moduletesting.IntegrationTestFixture) {
        barMsgClient := barv1.NewMsgClient(f)
        res, err := barMsgClient.CallFoo(f, &MsgCallFoo{})
        ...
    })
}
```

Unlike unit tests, integration tests actually pull in other module dependencies. So that modules can be written
without direct dependencies on other modules and because golang has no concept of development dependencies, integration
tests should be written in separate go modules, ex. `example.com/bar/v2/test`. Because this paradigm uses go semantic
versioning, it is possible to build a single go module which imports 3 versions of bar and 2 versions of runtime and
can test these all together in the six various combinations of dependencies.

## Consequences

### Backwards Compatibility

Modules which migrate fully to ADR 033 will not be compatible with existing modules which use the keeper paradigm.
As a temporary workaround we may create some wrapper types that emulate the current keeper interface to minimize
the migration overhead.

### Positive

* we will be able to deliver interoperable semantically versioned modules which should dramatically increase the
  ability of the Cosmos SDK ecosystem to iterate on new features
* it will be possible to write Cosmos SDK modules in other languages in the near future

### Negative

* all modules will need to be refactored somewhat dramatically

### Neutral

* the `cosmossdk.io/core/appconfig` framework will play a more central role in terms of how modules are defined, this
  is likely generally a good thing but does mean additional changes for users wanting to stick to the pre-depinject way
  of wiring up modules
* `depinject` is somewhat less needed or maybe even obviated because of the full ADR 033 approach. If we adopt the
  core API proposed in https://github.com/cosmos/cosmos-sdk/pull/12239, then a module would probably always instantiate
  itself with a method `ProvideModule(appmodule.Service) (appmodule.AppModule, error)`. There is no complex wiring of
  keeper dependencies in this scenario and dependency injection may not have as much of (or any) use case.

## Further Discussions

The decision described above is considered in draft mode and is pending final buy-in from the team and key stakeholders.
Key outstanding discussions if we do adopt that direction are:

* how do module clients introspect dependency module API revisions
* how do modules determine a minor dependency module API revision requirement
* how do modules appropriately test compatibility with different dependency versions
* how to register and resolve interface implementations
* how do modules register their protobuf file descriptors depending on the approach they take to generated code (the
  API module approach may still be viable as a supported strategy and would need pinned file descriptors)

## References

* https://github.com/cosmos/cosmos-sdk/discussions/10162
* https://github.com/cosmos/cosmos-sdk/discussions/10582
* https://github.com/cosmos/cosmos-sdk/discussions/10368
* https://github.com/cosmos/cosmos-sdk/pull/11340
* https://github.com/cosmos/cosmos-sdk/issues/11899
* [ADR 020](./adr-020-protobuf-transaction-encoding.md)
* [ADR 033](./adr-033-protobuf-inter-module-comm.md)
