# ADR 012: State Accessors

## Changelog

* 2019 Sep 04: Initial draft

## Context

Cosmos SDK modules currently use the `KVStore` interface and `Codec` to access their respective state. While
this provides a large degree of freedom to module developers, it is hard to modularize and the UX is
mediocre.

First, each time a module tries to access the state, it has to marshal the value and set or get the
value and finally unmarshal. Usually this is done by declaring `Keeper.GetXXX` and `Keeper.SetXXX` functions,
which are repetitive and hard to maintain.

Second, this makes it harder to align with the object capability theorem: the right to access the
state is defined as a `StoreKey`, which gives full access on the entire Merkle tree, so a module cannot
send the access right to a specific key-value pair (or a set of key-value pairs) to another module safely.

Finally, because the getter/setter functions are defined as methods of a module's `Keeper`, the reviewers
have to consider the whole Merkle tree space when they reviewing a function accessing any part of the state.
There is no static way to know which part of the state that the function is accessing (and which is not).

## Decision

We will define a type named `Value`:

```go
type Value struct {
  m   Mapping
  key []byte
}
```

The `Value` works as a reference for a key-value pair in the state, where `Value.m` defines the key-value
space it will access and `Value.key` defines the exact key for the reference.

We will define a type named `Mapping`:

```go
type Mapping struct {
  storeKey sdk.StoreKey
  cdc      *codec.LegacyAmino
  prefix   []byte
}
```

The `Mapping` works as a reference for a key-value space in the state, where `Mapping.storeKey` defines
the IAVL (sub-)tree and `Mapping.prefix` defines the optional subspace prefix.

We will define the following core methods for the `Value` type:

```go
// Get and unmarshal stored data, noop if not exists, panic if cannot unmarshal
func (Value) Get(ctx Context, ptr interface{}) {}

// Get and unmarshal stored data, return error if not exists or cannot unmarshal
func (Value) GetSafe(ctx Context, ptr interface{}) {}

// Get stored data as raw byte slice
func (Value) GetRaw(ctx Context) []byte {}

// Marshal and set a raw value
func (Value) Set(ctx Context, o interface{}) {}

// Check if a raw value exists
func (Value) Exists(ctx Context) bool {}

// Delete a raw value value
func (Value) Delete(ctx Context) {}
```

We will define the following core methods for the `Mapping` type:

```go
// Constructs key-value pair reference corresponding to the key argument in the Mapping space
func (Mapping) Value(key []byte) Value {}

// Get and unmarshal stored data, noop if not exists, panic if cannot unmarshal
func (Mapping) Get(ctx Context, key []byte, ptr interface{}) {}

// Get and unmarshal stored data, return error if not exists or cannot unmarshal
func (Mapping) GetSafe(ctx Context, key []byte, ptr interface{})

// Get stored data as raw byte slice
func (Mapping) GetRaw(ctx Context, key []byte) []byte {}

// Marshal and set a raw value
func (Mapping) Set(ctx Context, key []byte, o interface{}) {}

// Check if a raw value exists
func (Mapping) Has(ctx Context, key []byte) bool {}

// Delete a raw value value
func (Mapping) Delete(ctx Context, key []byte) {}
```

Each method of the `Mapping` type that is passed the arguments `ctx`, `key`, and `args...` will proxy
the call to `Mapping.Value(key)` with arguments `ctx` and `args...`.

In addition, we will define and provide a common set of types derived from the `Value` type:

```go
type Boolean struct { Value }
type Enum struct { Value }
type Integer struct { Value; enc IntEncoding }
type String struct { Value }
// ...
```

Where the encoding schemes can be different, `o` arguments in core methods are typed, and `ptr` arguments
in core methods are replaced by explicit return types.

Finally, we will define a family of types derived from the `Mapping` type:

```go
type Indexer struct {
  m   Mapping
  enc IntEncoding
}
```

Where the `key` argument in core method is typed.

Some of the properties of the accessor types are:

* State access happens only when a function which takes a `Context` as an argument is invoked
* Accessor type structs give rights to access the state only that the struct is referring, no other
* Marshalling/Unmarshalling happens implicitly within the core methods

## Status

Proposed

## Consequences

### Positive

* Serialization will be done automatically
* Shorter code size, less boilerplate, better UX
* References to the state can be transferred safely
* Explicit scope of accessing

### Negative

* Serialization format will be hidden
* Different architecture from the current, but the use of accessor types can be opt-in
* Type-specific types (e.g. `Boolean` and `Integer`) have to be defined manually

### Neutral

## References

* [#4554](https://github.com/cosmos/cosmos-sdk/issues/4554)
