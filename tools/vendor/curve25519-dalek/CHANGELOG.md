# Changelog

Entries are listed in reverse chronological order per undeprecated
major series.

## 3.x series

### 3.2.0

* Add support for getting the identity element for the Montgomery
  form of curve25519, which is useful in certain protocols for
  checking contributory behaviour in derivation of shared secrets.

### 3.1.2

* Revert a commit which mistakenly removed support for `zeroize` traits
  for some point types, as well as elligator2 support for Edwards points.

### 3.1.1

* Fix documentation builds on nightly due to syntax changes to
  `#![cfg_attr(feature = "nightly", doc = include_str!("../README.md"))]`.

### 3.1.0

* Add support for the Elligator2 encoding for Edwards points.
* Add two optional formally-verified field arithmetic backends which
  use the Fiat Crypto project's Rust code, which is generated from
  proofs of functional correctness checked by the Coq theorem proving
  system.
* Add support for additional sizes of precomputed tables for basepoint
  scalar multiplication.
* Fix an unused import.
* Add support for using the `zeroize` traits with all point types.
  Note that points are not automatically zeroized on Drop, but that
  consumers of `curve25519-dalek` should call these methods manually
  when needed.

### 3.0.3

* Fix documentation builds on nightly due to syntax changes to
  `#![cfg_attr(feature = "nightly", doc = include_str!("../README.md"))]`.

### 3.0.2

* Multiple documentation typo fixes.
* Fixes to make using `alloc`+`no_std` possible for stable Rust.

### 3.0.1

* Update the optional `packed-simd` dependency to rely on a newer,
  maintained version of the `packed-simd-2` crate.

### 3.0.0

* Update the `digest` dependency to `0.9`.  This requires a major version
  because the `digest` traits are part of the public API, but there are
  otherwise no changes to the API.

## 2.x series

### 2.1.3

* Fix documentation builds on nightly due to syntax changes to
  `#![fg_attr(feature = "nightly", doc = include_str!("../README.md"))]`.

### 2.1.2

* Multiple documenation typo fixes.
* Fix `alloc` feature working with stable rust.

### 2.1.1

* Update the optional `packed-simd` dependency to rely on a newer,
  maintained version of the `packed-simd-2` crate.

### 2.1.0

* Make `Scalar::from_bits` a `const fn`, allowing its use in `const` contexts.

### 2.0.0

* Fix a data modeling error in the `serde` feature pointed out by Trevor Perrin
  which caused points and scalars to be serialized with length fields rather
  than as fixed-size 32-byte arrays.  This is a breaking change, but it fixes
  compatibility with `serde-json` and ensures that the `serde-bincode` encoding
  matches the conventional encoding for X/Ed25519.
* Update `rand_core` to `0.5`, allowing use with new `rand` versions.
* Switch from `clear_on_drop` to `zeroize` (by Tony Arcieri).
* Require `subtle = ^2.2.1` and remove the note advising nightly Rust, which is
  no longer required as of that version of `subtle`.  See the `subtle`
  changelog for more details.
* Update `README.md` for `2.x` series.
* Remove the `build.rs` hack which loaded the entire crate into its own
  `build.rs` to generate constants, and keep the constants in the source code.

The only significant change is the data model change to the `serde` feature;
besides the `rand_core` version bump, there are no other user-visible changes.

## 1.x series

### 1.2.6

* Fixes to make using alloc+no_std possible for stable Rust.

### 1.2.5

* Update the optional `packed-simd` dependency to rely on a newer,
  maintained version of the `packed-simd-2` crate.

### 1.2.4

* Specify a semver bound for `clear_on_drop` rather than an exact version,
  addressing an issue where changes to inline assembly in rustc prevented
  `clear_on_drop` from working without an update.

### 1.2.3

* Fix an issue identified by a Quarkslab audit (and Jack Grigg), where manually
  constructing unreduced `Scalar` values, as needed for X/Ed25519, and then
  performing scalar/scalar arithmetic could compute incorrect results.
* Switch to upstream Rust intrinsics for the IFMA backend now that they exist in
  Rust and don't need to be defined locally.
* Ensure that the NAF computation works correctly, even for parameters never
  used elsewhere in the codebase.
* Minor refactoring to EdwardsPoint decompression.
* Fix broken links in documentation.
* Fix compilation on nightly broken due to changes to the `#[doc(include)]` path
  root (not quite correctly done in 1.2.2).

### 1.2.2

* Fix a typo in an internal doc-comment.
* Add the "crypto" tag to crate metadata.
* Fix compilation on nightly broken due to changes to the `#[doc(include)]` path
  root.

### 1.2.1

* Fix a bug in bucket index calculations in the Pippenger multiscalar algorithm
  for very large input sizes.
* Add a more extensive randomized multiscalar multiplication consistency check
  to the test suite to prevent regressions.
* Ensure that that multiscalar and NAF computations work correctly on extremal
  `Scalar` values constructed via `from_bits`.

### 1.2.0

* New multiscalar multiplication algorithm with better performance for
  large problem sizes.  The backend algorithm is selected
  transparently using the size hints of the input iterators, so no
  changes are required for client crates to start using it.
* Equality of Edwards points is now checked in projective coordinates.
* Serde can now be used with `no_std`.

### 1.1.4

* Fix typos in documentation comments.
* Remove unnecessary `Default` bound on `Scalar::from_hash`.

### 1.1.3

* Reverts the change in 1.1.0 to allow owned and borrowed RNGs, which caused a breakage due to a subtle interaction with ownership rules.  (The `RngCore` change is retained).

### 1.1.2

* Disabled KaTeX on `docs.rs` pending proper [support upstream](https://github.com/rust-lang/docs.rs/issues/302).

## 1.1.1

* Fixed an issue related to `#[cfg(rustdoc)]` which prevented documenting multiple backends.

### 1.1.0

* Adds support for precomputation for multiscalar multiplication.
* Restructures the internal source tree into `serial` and `vector` backends (no change to external API).
* Adds a new IFMA backend which sets speed records.
* The `avx2_backend` feature is now an alias for the `simd_backend` feature, which autoselects an appropriate vector backend (currently AVX2 or IFMA).
* Replaces the `rand` dependency with `rand_core`.
* Generalizes trait bounds on `RistrettoPoint::random()` and `Scalar::random()` to allow owned and borrowed RNGs and to allow `RngCore` instead of `Rng`.

### 1.0.3

* Adds `ConstantTimeEq` implementation for compressed points.

### 1.0.2

* Fixes a typo in the naming of variables in Ristretto formulas (no change to functionality).

### 1.0.1

* Depends on the stable `2.0` version of `subtle` instead of `2.0.0-pre.0`.

### 1.0.0

Initial stable release.  Yanked due to a dependency mistake (see above).

