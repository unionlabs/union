This library contains types for the [Ethereum Consensus Specs][consensus-specs] specification, and implementations of their canonical serializations (both [ssz](/lib/ssz/README.md) and json).

Every type has it's own module for organization, and then re-exported at the crate root. For types that are "bounded" in their ssz spec, a separate `TypeSsz` type with the bounds is provided (as well as conversions between the bounded and non-bounded type). This type is also re-exported at the crate root.

[consensus-specs]: https://github.com/ethereum/consensus-specs/tree/dev/specs
