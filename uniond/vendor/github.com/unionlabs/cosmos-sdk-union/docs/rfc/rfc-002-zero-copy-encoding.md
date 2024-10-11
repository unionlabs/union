# RFC 002: Zero-Copy Encoding

## Changelog

* 2022-03-08: Initial draft

## Background

When the SDK originally migrated to [protobuf state encoding](./../architecture/adr-019-protobuf-state-encoding.md),
zero-copy encodings such as [Cap'n Proto](https://capnproto.org/)
and [FlatBuffers](https://google.github.io/flatbuffers/)
were considered. We considered how a zero-copy encoding could be beneficial for interoperability with modules
and scripts in other languages and VMs. However, protobuf was still chosen because the maturity of its ecosystem and
tooling was much higher and the client experience and performance were considered the highest priorities.

In [ADR 033: Protobuf-based Inter-Module Communication](./../architecture/adr-033-protobuf-inter-module-comm.md), the
idea of cross-language/VM inter-module
communication was considered again. And in the discussions surrounding [ADR 054: Semver Compatible SDK Modules](./../architecture/adr-054-semver-compatible-modules.md),
it was determined that multi-language/VM support in the SDK is a near term priority.

While we could do cross-language/VM inter-module communication with protobuf binary or even JSON, the performance
overhead is deemed to be too high because:
* we are proposing replacing keeper calls with inter-module message calls and the overhead of even the inter-module
routing checks has come into question by some SDK users without even considering the possible overhead of encoding.
Effectively we would be replacing function calls with encoding. One of the SDK's primary objectives currently is
improving performance, and we want to avoid inter-module calls from becoming a big step backward.
* we want Rust code to be able to operate in highly resource constrained virtual machines so whatever we can do to
reduce performance overhead as well as the size of generated code will make it easier and more feasible to deploy
first-class integrations with these virtual machines.

Thus, the agreement when the [ADR 054](./../architecture/adr-054-semver-compatible-modules.md) working group concluded
was to pursue a performant zero-copy encoding which is suitable for usage in highly resource constrained environments.

## Proposal

This RFC proposes a zero-copy encoding that is derived from the schema definitions defined in .proto files in the SDK
and all app chains. This would result in a new code generator for that supports both this zero-copy encoding as well as
the existing protobuf binary and JSON encodings as well as the google.golang.org/protobuf API. To make this zero-copy
encoding work, a number of changes are needed to how we manage the versioning of protobuf messages that should
address other concerns raised in [ADR 054](./../architecture/adr-054-semver-compatible-modules.md). The API for using
protobuf in golang would also change and this will be described in the [code generation](#generated-code) section
along with a proposed Rust code generator.

An alternative approach to building a zero-copy encoding based on protobuf schemas would be to switch to FlatBuffers
or Cap'n Proto directly. However, this would require a complete rewrite of the SDK and all app chains. Places this
burden on the ecosystem would not be a wise choice when creating a zero-copy encoding compatible with all our
existing types and schemas is feasible. In the future, we may consider a native schema language for this encoding
that is more natural and succinct for its rules, but for now we are assuming that it is best to continue supporting
the existing protobuf based workflow.

Also, we are not proposing a new encoding for transactions or gRPC query servers. From a client API perspective nothing
would change. The SDK would be capable of marshaling any message to and from protobuf binary and this zero-copy encoding
as needed.

Furthermore, migrating to the **new golang generated code would be 100% opt-in** because the inter-module router will
simply marshal existing gogo proto generated types to/from the zero-copy encoding when needed. So migrating to the new
code generator would provide a performance benefit, but would not be required.

In addition to supporting first-class Cosmos SDK modules defined in other languages and VMs, this encoding is intended
to be useful for user-defined code executing in a VM. To satisfy this, this encoding is designed to enable proper bounds
checking on all memory access at the expense of introducing some error return values in generated code.

### New Protobuf Linting and Breaking Change Rules

This zero-copy encoding places some additional requirements on the definition and maintenance of protobuf schemas.

#### No New Fields Can Be Added To Existing Messages

The biggest change is that it will be invalid to add a new field to an existing message and a breaking change detector
will need to be created which augments [buf breaking](https://docs.buf.build/breaking/overview) to detect this.

The reasons for this are two-fold:

1) from an API compatibility perspective, adding a new field to an existing message is actually a state machine breaking
   change which in [ADR 020](../architecture/adr-020-protobuf-transaction-encoding.md) required us to add an unknown
   field detector. Furthermore, in [ADR 054](../architecture/adr-054-semver-compatible-modules.md) this "feature" of protobuf
   poses one of the biggest problems for correct forward compatibility between different versions of the same module.
2) not allowing new fields in existing messages makes the generated code in languages like Rust (which is currently our
   highest priority target), much simpler and more performant because we can assume a fixed size struct gets allocated.
   If new fields can be added to existing messages, we need to encode the number of fields into the message and then
   do runtime checks. So this both increases memory layers and requires another layout of indirection. With the encoding
   proposed below, "plain old Rust structs" (used with some special field types) can be used.

Instead of adding new fields to existing messages, APIs can add new messages to existing packages or create new packages
with new versions of the messages. Also, we are not restricting the addition of cases to `oneof`s or values to `enum`s.
All of these cases are easier to detect at runtime with standard `switch` statements than the addition of new fields.

#### Additional Linting Rules

The following additional rules will be enforced by a linter that
complements [buf lint](https://docs.buf.build/lint/overview):

* all message fields must be specified in continuous ascending order starting from `1`
* all enums must be specified in continuous ascending order starting from `0` - otherwise it is too complex to check at
  runtime whether an enum value is unknown. An alternative would be to make adding new values to existing enums breaking
* all enum values must be `<= 255`. Any enum in a blockchain application which needs more than 256 values is probably
  doing something very wrong.
* all oneof's must be the *only* element in their containing message and must start at field number `1` and be added in
  continuous ascending order - this makes it possible to quickly check for unknown values
* all `oneof` field numbers must be `<= 255`. Any `oneof` which needs more field cases is probably doing something very
  wrong.

These requirements make the encoding and generated code simpler.

### Encoding

#### Buffers and Memory Management

By default, this encoding attempts to use a single fixed size encoding buffer of 64kb. This imposes a limit on the
maximum size of a message that can be encoded. In the context of a message passing protocol for blockchains, this
is generally a reasonable limit and the only known valid use case for exceeding it is to store user-uploaded byte
code for execution in VMs. To accommodate this, large `string` and `bytes` values can be encoded in additional
standalone buffers if needed. Still, the body of a message included all scalar and message fields
must fit inside the 64kb buffer.

While this design decision greatly simplifies the encoding and decoding logic, as well as the complexity of
generated code, it does mean that APIs will need to do proper bounds checking when writing data that is not fixed
size and return errors.

The term `Root` is used to refer to the main 64kb buffer plus any additional large `string`/`bytes` buffers that are
allocated.

#### Scalar Encoding

* `bool`s are encoded as 1 byte - `0` or `1`
* `uint32`, `int32`, `sint32`, `fixed32`, `sfixed32` are encoded as 4 bytes by default
* `uint64`, `int64`, `sint64`, `fixed64`, `sfixed64` are encoded as 8 bytes by default
* `enum`s are encoded as 1 byte and values *MUST* be in the range of `0` to `255`.
* all scalars declared as `optional` are prefixed with 1 additional byte whose value is `0` or `1` to indicate presence

All multibyte integers are encoded as little-endian which is by far the most common native byte order for modern
CPUs. Signed integers always use two's complement encoding.

#### Message Encoding

By default, messages field are encoded inline as structs. Meaning that if a message struct takes 8 bytes then its inline
field in another struct will add 8 bytes to that struct size.

`optional` message fields will be prefixed by 1 byte to indicate presence. (Alternatively, we could encode optional
message fields as pointers (see below) if the desire is to save memory when they are rarely used needed.)

#### Oneof’s

`oneof`s are encoded as a combination of a `uint8` discriminant field and memory that is as large as the largest member
field. `oneof` field numbers *MUST* be between `1` and `255`.

```protobuf
message Foo {
  oneof sum {
    bool x = 1;
    int32 y = 2;
  }
}
```

A discriminant of `0` indicates that the field is not set.

#### Pointer Types: Bytes and Strings and Repeated fields

A pointer is an 16-bit unsigned integer that points to an offset in the current memory buffer or to another memory
buffer.  If the bit mask `0xFF00` on the is unset, then the pointer points to an offset in the main 64kb memory buffer.
If that bit mask is set, then the pointer points to a large `string` or `bytes` buffer.  Up to 256 such buffers
can be referenced in a single `Root`. The pointer `0` indicates that a field is not defined.

`bytes`, `string`s and repeated fields are encoded as pointers to a memory location that is prefixed with the
length of the `bytes`, `string` or repeated field value. If the referenced memory location is in the main 64kb memory
buffer, then this length prefix will be a 16-bit unsigned integer. If the referenced memory location is a large
`string` or `bytes` buffer, then this length prefix will be a 32-bit unsigned integer.

#### `Any`s

`Any`s are encoded as a pointer to the type URL string and a pointer to the start of the message
specified by the type URL.

#### Maps

Maps are not supported.

#### Extended Encoding Options

We may choose to allow customizing the encoding of fields so that they take up less space.

For example, we could allow 8-bit or 16-bit integers:
`int32 x = 1 [(cosmos_proto.int16) = true]` would indicate that the field only needs 2 bytes

Or we could allow `string`, `bytes` or `repeated` fields to have a fixed size rather than being encoding as
pointers to a variable-length value:
`string y = 2 [(cosmos_proto.fixed_size) = 3]` could indicate that this is a fixed width 3 byte string

If we choose to enable these encoding options, changing these options would be a breaking change that needs to be
prevented by the breaking change detector.

### Generated Code

We will describe the generated Go and Rust code using this example protobuf file:

```protobuf
message Foo {
  int32 x = 1;
  optional uint32 y = 2;
  string z = 3;
  Bar bar = 4;
  repeated Bar bars = 5;
}


message Bar {
  ABC abc = 1;
  Baz baz = 2;
  repeated uint32 xs = 3;
}

message Baz {
  oneof sum {
    uint32 x = 1;
    string y = 2;
  }
}

enum ABC {
  A = 0;
  B = 1;
  C = 2;
  D = 3;
}
```

#### Go

In golang, the generated code would not expose any exported struct fields, but rather getters and setters as an
interface
or struct methods, ex:

```go
type Foo interface {
    X() int32
    SetX(int32) 
    Y() zpb.Option[uint32]
    SetY(zpb.Option[uint32])
    Z() (string, error)
    SetZ(string) error
    Bar() Bar
    Bars() (zpb.Array[Bar], error)
}

type Bar interface {
    Abc() ABC
    SetAbc(ABC) Bar
    Baz() Baz
    Xs() (zpb.ScalarArray[uint32], error)
}

type Baz interface {
    Case() Baz_case
    GetX() uint32
    SetX(uint32)
    GetY() (string, error)
    SetY(string)
}

type Baz_case int32
const (
    Baz_X Baz_case = 0
    Baz_Y Baz_case = 1
)

type ABC int32
const (
    ABC_A ABC = 0
    ABC_B ABC = 1
    ABC_C ABC = 2
    ABC_D ABC = 3
)
```

Special types `zpb.Option`, `zpb.Array` and `zpb.ScalarArray` are used to represent `optional` and repeated fields
respectively. These types would be included in the runtime library (called `zpb` here for zero-copy protobuf) and would
have an API like this:

```go
type Option[T] interface {
    IsSet() bool
    Value() T
}

type Array[T] interface {
    InitWithLength(int) error
    Len() int
    Get(int) T
}

type ScalarArray[T] interface {
    Array[T]
    Set(int, T)
}
```

Arrays in particular would not be resizable, but would be initialized with a fixed length. This is to ensure that arrays
can be written to the underlying buffer in a linear way.

In golang, buffers would be managed transparently under the hood by the first message initialized, and usage of this
generated code might look like this:

```go
foo := NewFoo()
foo.SetX(1)
foo.SetY(zpb.NewOption[uint32](2))
err := foo.SetZ("hello")
if err != nil {
    panic(err)
}

bar := foo.Bar()
bar.Baz().SetX(3)

xs, err = bar.Xs()
if err != nil {
    panic(err)
}
xs.InitWithLength(2)
xs.Set(0, 0)
xs.Set(1, 2)

bars, err = foo.Bars()
if err != nil {
    panic(err)
}
bars.InitWithLength(3)
bars.Get(0).Baz().SetY("hello")
bars.Get(1).SetAbc(ABC_B)
bars.Get(2).Baz().SetX(4)
```

Under the hood the generated code would manage memory buffers on its own. The usage of `oneof`s is a bit easier than
the existing go generated code (as with `bar.Baz()` above). And rather than using setters on embedded messages, we
simply get the field (already allocated) and set its fields (as in the case of `foo.Bar()` above or the repeated
field `foo.Bars()`). Whenever a field is stored with a pointer (`string`, `bytes`, and `repeated` fields), there is
always an error returned on the getter to do proper bounds checking on the buffer.

#### Rust

This encoding should allow generating native structs in Rust that are annotated with `#[repr(C, align(1))]`. It should
be fairly natural to use from Rust with a key difference that memory buffers (called `Root`s) must be manually allocated
and passed into any pointer type.

Here is some example code that uses library types `Option`, `Enum`, `String`, `OneOf` and `Repeated`
as well as little-endian integer types from [rend](https://lib.rs/crates/rend):

```rust!
#[repr(C, align(1))]
struct Foo {
    x: rend:i32_le,
    y: cosmos_proto::Option<rend:u32_le>,
    z: cosmos_proto::String, // String wraps a pointer to a string
    bar: Bar
}

#[repr(C, align(1))]
struct Bar {
    abc: cosmos_proto::Enum<ABC, 3>, // the Enum wrapper allows us to distinguish undefined and defined values of ABC at runtime. 3 is specified as the max value of ABC.
    baz: cosmos_proto::OneOf<Baz, 2>, // the OneOf wrapper allows distinguished undefined values of Baz at runtime. 2 is specified as the max field value of Baz.
    xs: cosmos_proto::Repeated<rend:u32_le> // Repeated wraps a pointer to repeated fields
}

#[repr(u8)]
enum ABC {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

#[repr(C, u8)]
enum Baz {
    Empty, // all oneof's have a case for Empty if they are unset
    X(rend::u32_le),
    Y(cosmos_proto::String)
}
```

Example usage (which does the exact same thing as the go example above) would be:

```rust!
let mut root = Root<Foo>::new();
let mut foo = root.get_mut();
foo.x = 1.into();
foo.y = Some(2.into());
foo.z.set(root.new_string("hello")?); // could return an allocation error

foo.bar.baz = Baz::X(3.into());

foo.bar.xs.init_with_size(&mut root, 2)?; // could return an allocation error
foo.bar.xs[0] = 0.into();
foo.bar.xs[1] = 2.into();

foo.bars.init_with_size(&mut root, 3)?; // could return an allocation error
foo.bars[0].baz = Baz::Y(root.new_string("hello")?); // could return an allocation error
foo.bars[1].abc = ABC::B;
foo.bars[2].baz = Baz::X(4.into());
```

## Abandoned Ideas (Optional)

## References

## Discussion
