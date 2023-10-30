# Changelog

All notable changes to similar are documented here.

## 2.0.0

* `Fragile` no longer boxes internally.
* `Sticky` and `SemiSticky` now require the use of stack tokens.
  For more information see [#26](https://github.com/mitsuhiko/fragile/issues/26)
* `Sticky` now tries to drop entries from the thread local registry eagerly
  if it's dropped on the right thread.

## 1.2.1

* Fixed non slab versions only allowing a single sticky.

## 1.2.0

Note on safety: the `Sticky` and `SemiSticky` types allow data to live
longer than the wrapper type which is why they are now requiring a `'static`
bound.  Previously it was possible to create a sticky containing a bare
reference which permitted unsafe access.

* `Sticky` now requires `'static`.
* Added the `slab` feature for an internal optimization for `Sticky` to use
  a slab instead of a `HashMap`.

## Older Releases

Older releases were yanked due to the insufficient trait bound on `Sticky`.
