<!-- cargo-sync-readme start -->

# Overview
- [📦 crates.io](https://crates.io/crates/enum-iterator)
- [📖 Documentation](https://docs.rs/enum-iterator)
- [⚖ 0BSD license](https://spdx.org/licenses/0BSD.html)

Tools to iterate over the variants of a field-less enum.

See the `IntoEnumIterator` trait.

# Example
```rust
use enum_iterator::IntoEnumIterator;

#[derive(Debug, IntoEnumIterator, PartialEq)]
enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }

fn main() {
    assert_eq!(Day::into_enum_iter().next(), Some(Day::Monday));
    assert_eq!(Day::into_enum_iter().last(), Some(Day::Sunday));
}
```

# Contribute
All contributions shall be licensed under the [0BSD license](https://spdx.org/licenses/0BSD.html).

<!-- cargo-sync-readme end -->
