# slice-group-by

[![slice-group-by crate](https://img.shields.io/crates/v/slice-group-by.svg)](https://crates.io/crates/slice-group-by)
[![slice-group-by documentation](https://docs.rs/slice-group-by/badge.svg)](https://docs.rs/slice-group-by)
[![dependency status](https://deps.rs/repo/github/Kerollmops/slice-group-by/status.svg)](https://deps.rs/repo/github/Kerollmops/slice-group-by)
[![build & tests worflow](https://github.com/Kerollmops/slice-group-by/actions/workflows/ci.yml/badge.svg)](https://github.com/Kerollmops/slice-group-by/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/Kerollmops/slice-group-by.svg)](https://github.com/Kerollmops/slice-group-by)

An implementation of the [`groupBy` Haskell function], providing tools for efficiently iterating over `slice` and `str` by groups defined by a function that specifies if two elements are in the same group.

### Differences with `Itertools::group_by`

The [`Itertools::group_by`] method use a key to compare elements, this library works like, say, [`slice::sort_by`], it uses a comparison function. It works on every `Iterator` type, `slice-group-by` work only with `slice` and `str`, which is the power of this library, it is fast thanks to [data locality].

Also `slice-group-by` support multiple search algorithms (i.e. [linear], [binary] and [exponential search]) and can return groups starting from the end.

[`groupBy` Haskell function]: http://hackage.haskell.org/package/base-4.12.0.0/docs/Data-List.html#v:groupBy
[`Itertools::group_by`]: https://docs.rs/itertools/0.8.0/itertools/trait.Itertools.html#method.group_by
[`slice::sort_by`]: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_by
[data locality]: https://en.wikipedia.org/wiki/Locality_of_reference
[linear]: https://en.wikipedia.org/wiki/Linear_search
[binary]: https://en.wikipedia.org/wiki/Binary_search_algorithm
[exponential search]: https://en.wikipedia.org/wiki/Exponential_search

## Examples

### Linear Searched Immutable Groups

You will only need to define a function that returns `true` if two elements are in the same group.

The `LinearGroupBy` iterator will always gives contiguous elements to the predicate function.

```rust
use slice_group_by::GroupBy;

let slice = &[1, 1, 1, 3, 3, 2, 2, 2];

let mut iter = slice.linear_group_by(|a, b| a == b);

assert_eq!(iter.next(), Some(&[1, 1, 1][..]));
assert_eq!(iter.next(), Some(&[3, 3][..]));
assert_eq!(iter.next(), Some(&[2, 2, 2][..]));
assert_eq!(iter.next(), None);
```

### Linear Searched Immutable Str Groups

You will only need to define a function that returns `true` if two `char` are in the same group.

The `LinearStrGroupBy` iterator will always gives contiguous `char` to the predicate function.

```rust
use slice_group_by::StrGroupBy;

let string = "aaaabbbbb饰饰cccc";

let mut iter = string.linear_group_by(|a, b| a == b);

assert_eq!(iter.next(), Some("aaaa"));
assert_eq!(iter.next(), Some("bbbbb"));
assert_eq!(iter.next(), Some("饰饰"));
assert_eq!(iter.next(), Some("cccc"));
assert_eq!(iter.next(), None);
```

### Binary Searched Mutable Groups

It is also possible to get mutable non overlapping groups of a slice.

The `BinaryGroupBy/Mut` and `ExponentialGroupBy/Mut` iterators will not necessarily
gives contiguous elements to the predicate function. The predicate function should implement
an order consistent with the sort order of the slice.

```rust
use slice_group_by::GroupByMut;

let slice = &mut [1, 1, 1, 2, 2, 2, 3, 3];

let mut iter = slice.binary_group_by_mut(|a, b| a == b);

assert_eq!(iter.next(), Some(&mut [1, 1, 1][..]));
assert_eq!(iter.next(), Some(&mut [2, 2, 2][..]));
assert_eq!(iter.next(), Some(&mut [3, 3][..]));
assert_eq!(iter.next(), None);
```

### Exponential Searched Mutable Groups starting from the End

It is also possible to get mutable non overlapping groups of a slice even starting from end of it.

```rust
use slice_group_by::GroupByMut;

let slice = &mut [1, 1, 1, 2, 2, 2, 3, 3];

let mut iter = slice.exponential_group_by_mut(|a, b| a == b).rev();

assert_eq!(iter.next(), Some(&mut [3, 3][..]));
assert_eq!(iter.next(), Some(&mut [2, 2, 2][..]));
assert_eq!(iter.next(), Some(&mut [1, 1, 1][..]));
assert_eq!(iter.next(), None);
```
