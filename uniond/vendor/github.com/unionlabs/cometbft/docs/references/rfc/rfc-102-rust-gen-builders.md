# RFC 102: Improve forward compatibility of proto-generated Rust code

## Changelog

- 17-Apr-2023: Initial draft

## Abstract

In protobuf, adding a field to a message or a oneof field is considered
a backward-compatible change and language bindings should avoid
breakages on source level for such changes in the proto definitions.
This is not currently the case, in general, for Rust bindings as implemented
by [prost-build].

We propose to augment the prost-build code generator with an add-on providing
a forward-compatible builder API, and use the `#[non_exhaustive]` attribute on
the generated data types to forbid their use with syntax that prevents future
member additions. This will allow us to evolve CometBFT protobuf APIs without
versioning churn that's not necessary for the Go bindings.

[prost-build]: https://crates.io/crates/prost-build

## Background

As we are renaming protobuf packages for CometBFT and introducing versioning
practices recommended by [buf.build][buf-versioning], it's important to lay
down the basis for future development that does not perpetuate workarounds for
limitations of a particular language binding.

[buf-versioning]: https://buf.build/docs/best-practices/module-development/#package-versions

### References

* Issue [#399](https://github.com/tokio-rs/prost/issues/399) in the prost
  repository captures the general problem and discussion.
* CometBFT PR [#495](https://github.com/cometbft/cometbft/pull/495) introduces
  versioned protobuf definitions, currently with extra versioning applied
  to accommodate API breakage caused in tendermint-rs by code generated with
  prost-build.
* [Notes](https://docs.google.com/document/d/1DoxKiYtUx44xZv5my-bkfWZKY6TklvxpSUrdX9yOpNw/edit?usp=sharing) on the discussion during a 13 Apr 2023 meeting, detailing the
  considerations specific to CometBFT versioning.
* [ADR 103](https://github.com/cometbft/cometbft/blob/main/docs/architecture/adr-103-proto-versioning.md) details the versioning approach as currently
  accepted.

## Discussion

The approach taken in prost-build to represent protobuf messages is
to generate corresponding structs with all fields declared public. This is
generally preferable to a more encapsulated Rust API with member accessors,
because domain-appropriate data access and enforcement of invariants often
cannot be adequately expressed by means of protobuf and is better realized via
hand-crafted domain types. Proto3 also enforces optionality on all fields,
which (in absence of customizations) makes the generated type ugly and
sub-optimal to work with if some of the fields shall always be set to a
non-degenerate value. So use of the proto-generated types should be
dedicated to decoding and encoding protobuf, and possibly for deriving some
utility trait impls like serde that can reuse the simple structures. 

However, this allows Rust code consuming a message-derived struct type to use
struct initializer or matching syntax where all defined struct fields must be
present. If more fields are later added to the message definition without
changing its package name, and the generated struct type is updated, such
usages will fail to compile. This is not the case in Go, where field-keyed
struct initializers are allowed to omit fields, which then get initialized to
the zero value (which conveniently corresponds to the protobuf specification
for optional fields).

To work around this, the generated struct types can be annotated with a
`#[non_exhaustive]` attribute, which forbids struct initializer syntax or
exhaustive field matching in foreign crates, making all usages of these struct
types compatible with future field additions. This alone, however, leaves only
a cludgy way to initialize messages that relies on a derived `Default`
implementation and individual field assignments. To alleviate the pain, it is
recommended to add a builder pattern API allowing ergonomic initialization
syntax. To do this manually for each generated struct, however, would be very
tedious and time-consuming.

### Builder API generator/plugin

To plug this gap, we propose to create a code generator in Rust to augment
the output of prost-build with a builder API for each generated struct type.
This generator can be invoked
either from `build.rs` or an in-project generator tool, or as a `buf` plugin.

As an example, using this proto definition:

```proto
message HelloRequest {
    int version = 1;
    repeated string flags = 2;
}
```

The generator will provide a builder API along these lines:

```rust
impl HelloRequest {
    pub fn builder() -> self::prost_builders::HelloRequestBuilder {
        todo!()
    }
}

pub mod prost_builders {
    pub struct HelloRequestBuilder {
        inner: super::HelloRequest,
    }

    impl HelloRequestBuilder {
        pub fn version(mut self, version: i32) -> Self {
            self.inner.version = version;
            self
        }

        pub fn flags<T>(mut self, flags: impl IntoIterator<Item = T>) -> Self
        where T: Into<String>,
        {
            self.inner.flags = flags.into_iter().map(Into::into).collect();
            self
        }

        pub fn build(self) -> super::HelloRequest {
            self.inner
        }
    }
}
```

Note how the initializer methods of a builder can be equipped with convenient
generics, utilizing knowledge of the protobuf type system.

### Open issues

Do we still want to bump the version package for field additions
between major CometBFT proto releases, especially when adding important semantics?
