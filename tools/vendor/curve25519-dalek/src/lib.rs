// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis lovecruft
// Copyright (c) 2016-2019 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

#![no_std]
#![cfg_attr(feature = "nightly", feature(test))]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![cfg_attr(feature = "simd_backend", feature(stdsimd))]

// Refuse to compile if documentation is missing.
#![deny(missing_docs)]

#![doc(html_logo_url = "https://doc.dalek.rs/assets/dalek-logo-clear.png")]
#![doc(html_root_url = "https://docs.rs/curve25519-dalek/3.2.0")]

//! # curve25519-dalek [![](https://img.shields.io/crates/v/curve25519-dalek.svg)](https://crates.io/crates/curve25519-dalek) [![](https://img.shields.io/badge/dynamic/json.svg?label=docs&uri=https%3A%2F%2Fcrates.io%2Fapi%2Fv1%2Fcrates%2Fcurve25519-dalek%2Fversions&query=%24.versions%5B0%5D.num&colorB=4F74A6)](https://doc.dalek.rs) [![](https://travis-ci.org/dalek-cryptography/curve25519-dalek.svg?branch=master)](https://travis-ci.org/dalek-cryptography/curve25519-dalek)
//!
//! <img
//!  width="33%"
//!  align="right"
//!  src="https://doc.dalek.rs/assets/dalek-logo-clear.png"/>
//!
//! **A pure-Rust implementation of group operations on Ristretto and Curve25519.**
//!
//! `curve25519-dalek` is a library providing group operations on the Edwards and
//! Montgomery forms of Curve25519, and on the prime-order Ristretto group.
//!
//! `curve25519-dalek` is not intended to provide implementations of any particular
//! crypto protocol.  Rather, implementations of those protocols (such as
//! [`x25519-dalek`][x25519-dalek] and [`ed25519-dalek`][ed25519-dalek]) should use
//! `curve25519-dalek` as a library.
//!
//! `curve25519-dalek` is intended to provide a clean and safe _mid-level_ API for use
//! implementing a wide range of ECC-based crypto protocols, such as key agreement,
//! signatures, anonymous credentials, rangeproofs, and zero-knowledge proof
//! systems.
//!
//! In particular, `curve25519-dalek` implements Ristretto, which constructs a
//! prime-order group from a non-prime-order Edwards curve.  This provides the
//! speed and safety benefits of Edwards curve arithmetic, without the pitfalls of
//! cofactor-related abstraction mismatches.
//!
//! # Documentation
//!
//! The semver-stable, public-facing `curve25519-dalek` API is documented
//! [here][docs-external].  In addition, the unstable internal implementation
//! details are documented [here][docs-internal].
//!
//! The `curve25519-dalek` documentation requires a custom HTML header to include
//! KaTeX for math support. Unfortunately `cargo doc` does not currently support
//! this, but docs can be built using
//! ```sh
//! make doc
//! make doc-internal
//! ```
//!
//! # Use
//!
//! To import `curve25519-dalek`, add the following to the dependencies section of
//! your project's `Cargo.toml`:
//! ```toml
//! curve25519-dalek = "3"
//! ```
//!
//! The sole breaking change in the `3.x` series was an update to the `digest`
//! version, and in terms of non-breaking changes it includes:
//!
//! * support for using `alloc` instead of `std` on stable Rust,
//! * the Elligator2 encoding for Edwards points,
//! * a fix to use `packed_simd2`,
//! * various documentation fixes and improvements,
//! * support for configurably-sized, precomputed lookup tables for basepoint scalar
//!   multiplication,
//! * two new formally-verified field arithmetic backends which use the Fiat Crypto
//!   Rust code, which is generated from proofs of functional correctness checked by
//!   the Coq theorem proving system, and
//! * support for explicitly calling the `zeroize` traits for all point types.
//!
//! The `2.x` series has API almost entirely unchanged from the `1.x` series,
//! except that:
//!
//! * an error in the data modeling for the (optional) `serde` feature was
//!   corrected, so that when the `2.x`-series `serde` implementation is used
//!   with `serde-bincode`, the derived serialization matches the usual X/Ed25519
//!   formats;
//! * the `rand` version was updated.
//!
//! See `CHANGELOG.md` for more details.
//!
//! # Backends and Features
//!
//! The `nightly` feature enables features available only when using a Rust nightly
//! compiler.  In particular, it is required for rendering documentation and for
//! the SIMD backends.
//!
//! Curve arithmetic is implemented using one of the following backends:
//!
//! * a `u32` backend using serial formulas and `u64` products;
//! * a `u64` backend using serial formulas and `u128` products;
//! * an `avx2` backend using [parallel formulas][parallel_doc] and `avx2` instructions (sets speed records);
//! * an `ifma` backend using [parallel formulas][parallel_doc] and `ifma` instructions (sets speed records);
//!
//! By default the `u64` backend is selected.  To select a specific backend, use:
//! ```sh
//! cargo build --no-default-features --features "std u32_backend"
//! cargo build --no-default-features --features "std u64_backend"
//! # Requires nightly, RUSTFLAGS="-C target_feature=+avx2" to use avx2
//! cargo build --no-default-features --features "std simd_backend"
//! # Requires nightly, RUSTFLAGS="-C target_feature=+avx512ifma" to use ifma
//! cargo build --no-default-features --features "std simd_backend"
//! ```
//! Crates using `curve25519-dalek` can either select a backend on behalf of their
//! users, or expose feature flags that control the `curve25519-dalek` backend.
//!
//! The `std` feature is enabled by default, but it can be disabled for no-`std`
//! builds using `--no-default-features`.  Note that this requires explicitly
//! selecting an arithmetic backend using one of the `_backend` features.
//! If no backend is selected, compilation will fail.
//!
//! # Safety
//!
//! The `curve25519-dalek` types are designed to make illegal states
//! unrepresentable.  For example, any instance of an `EdwardsPoint` is
//! guaranteed to hold a point on the Edwards curve, and any instance of a
//! `RistrettoPoint` is guaranteed to hold a valid point in the Ristretto
//! group.
//!
//! All operations are implemented using constant-time logic (no
//! secret-dependent branches, no secret-dependent memory accesses),
//! unless specifically marked as being variable-time code.
//! We believe that our constant-time logic is lowered to constant-time
//! assembly, at least on `x86_64` targets.
//!
//! As an additional guard against possible future compiler optimizations,
//! the `subtle` crate places an optimization barrier before every
//! conditional move or assignment.  More details can be found in [the
//! documentation for the `subtle` crate][subtle_doc].
//!
//! Some functionality (e.g., multiscalar multiplication or batch
//! inversion) requires heap allocation for temporary buffers.  All
//! heap-allocated buffers of potentially secret data are explicitly
//! zeroed before release.
//!
//! However, we do not attempt to zero stack data, for two reasons.
//! First, it's not possible to do so correctly: we don't have control
//! over stack allocations, so there's no way to know how much data to
//! wipe.  Second, because `curve25519-dalek` provides a mid-level API,
//! the correct place to start zeroing stack data is likely not at the
//! entrypoints of `curve25519-dalek` functions, but at the entrypoints of
//! functions in other crates.
//!
//! The implementation is memory-safe, and contains no significant
//! `unsafe` code.  The SIMD backend uses `unsafe` internally to call SIMD
//! intrinsics.  These are marked `unsafe` only because invoking them on an
//! inappropriate CPU would cause `SIGILL`, but the entire backend is only
//! compiled with appropriate `target_feature`s, so this cannot occur.
//!
//! # Performance
//!
//! Benchmarks are run using [`criterion.rs`][criterion]:
//!
//! ```sh
//! cargo bench --no-default-features --features "std u32_backend"
//! cargo bench --no-default-features --features "std u64_backend"
//! # Uses avx2 or ifma only if compiled for an appropriate target.
//! export RUSTFLAGS="-C target_cpu=native"
//! cargo bench --no-default-features --features "std simd_backend"
//! ```
//!
//! Performance is a secondary goal behind correctness, safety, and
//! clarity, but we aim to be competitive with other implementations.
//!
//! # FFI
//!
//! Unfortunately, we have no plans to add FFI to `curve25519-dalek` directly.  The
//! reason is that we use Rust features to provide an API that maintains safety
//! invariants, which are not possible to maintain across an FFI boundary.  For
//! instance, as described in the _Safety_ section above, invalid points are
//! impossible to construct, and this would not be the case if we exposed point
//! operations over FFI.
//!
//! However, `curve25519-dalek` is designed as a *mid-level* API, aimed at
//! implementing other, higher-level primitives.  Instead of providing FFI at the
//! mid-level, our suggestion is to implement the higher-level primitive (a
//! signature, PAKE, ZKP, etc) in Rust, using `curve25519-dalek` as a dependency,
//! and have that crate provide a minimal, byte-buffer-oriented FFI specific to
//! that primitive.
//!
//! # Contributing
//!
//! Please see [CONTRIBUTING.md][contributing].
//!
//! Patches and pull requests should be make against the `develop`
//! branch, **not** `master`.
//!
//! # About
//!
//! **SPOILER ALERT:** *The Twelfth Doctor's first encounter with the Daleks is in
//! his second full episode, "Into the Dalek". A beleaguered ship of the "Combined
//! Galactic Resistance" has discovered a broken Dalek that has turned "good",
//! desiring to kill all other Daleks. The Doctor, Clara and a team of soldiers
//! are miniaturized and enter the Dalek, which the Doctor names Rusty. They
//! repair the damage, but accidentally restore it to its original nature, causing
//! it to go on the rampage and alert the Dalek fleet to the whereabouts of the
//! rebel ship. However, the Doctor manages to return Rusty to its previous state
//! by linking his mind with the Dalek's: Rusty shares the Doctor's view of the
//! universe's beauty, but also his deep hatred of the Daleks. Rusty destroys the
//! other Daleks and departs the ship, determined to track down and bring an end
//! to the Dalek race.*
//!
//! `curve25519-dalek` is authored by Isis Agora Lovecruft and Henry de Valence.
//!
//! Portions of this library were originally a port of [Adam Langley's
//! Golang ed25519 library](https://!github.com/agl/ed25519), which was in
//! turn a port of the reference `ref10` implementation.  Most of this code,
//! including the 32-bit field arithmetic, has since been rewritten.
//!
//! The fast `u32` and `u64` scalar arithmetic was implemented by Andrew Moon, and
//! the addition chain for scalar inversion was provided by Brian Smith.  The
//! optimised batch inversion was contributed by Sean Bowe and Daira Hopwood.
//!
//! The `no_std` and `zeroize` support was contributed by Tony Arcieri.
//!
//! The formally verified backends, `fiat_u32_backend` and `fiat_u64_backend`, which
//! integrate with the Rust generated by the
//! [Fiat Crypto project](https://github.com/mit-plv/fiat-crypto) were contributed
//! by François Garillot.
//!
//! Thanks also to Ashley Hauck, Lucas Salibian, Manish Goregaokar, Jack Grigg,
//! Pratyush Mishra, Michael Rosenberg, and countless others for their
//! contributions.
//!
//! [ed25519-dalek]: https://github.com/dalek-cryptography/ed25519-dalek
//! [x25519-dalek]: https://github.com/dalek-cryptography/x25519-dalek
//! [contributing]: https://github.com/dalek-cryptography/curve25519-dalek/blob/master/CONTRIBUTING.md
//! [docs-external]: https://doc.dalek.rs/curve25519_dalek/
//! [docs-internal]: https://doc-internal.dalek.rs/curve25519_dalek/
//! [criterion]: https://github.com/japaric/criterion.rs
//! [parallel_doc]: https://doc-internal.dalek.rs/curve25519_dalek/backend/vector/avx2/index.html
//! [subtle_doc]: https://doc.dalek.rs/subtle/

//------------------------------------------------------------------------
// External dependencies:
//------------------------------------------------------------------------

#[cfg(all(feature = "alloc", not(feature = "std")))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(all(feature = "nightly", feature = "packed_simd"))]
extern crate packed_simd;

extern crate byteorder;
pub extern crate digest;
extern crate rand_core;
extern crate zeroize;

#[cfg(any(feature = "fiat_u64_backend", feature = "fiat_u32_backend"))]
extern crate fiat_crypto;

// Used for traits related to constant-time code.
extern crate subtle;

#[cfg(all(test, feature = "serde"))]
extern crate bincode;
#[cfg(feature = "serde")]
extern crate serde;

// Internal macros. Must come first!
#[macro_use]
pub(crate) mod macros;

//------------------------------------------------------------------------
// curve25519-dalek public modules
//------------------------------------------------------------------------

// Scalar arithmetic mod l = 2^252 + ..., the order of the Ristretto group
pub mod scalar;

// Point operations on the Montgomery form of Curve25519
pub mod montgomery;

// Point operations on the Edwards form of Curve25519
pub mod edwards;

// Group operations on the Ristretto group
pub mod ristretto;

// Useful constants, like the Ed25519 basepoint
pub mod constants;

// External (and internal) traits.
pub mod traits;

//------------------------------------------------------------------------
// curve25519-dalek internal modules
//------------------------------------------------------------------------

// Finite field arithmetic mod p = 2^255 - 19
pub(crate) mod field;

// Arithmetic backends (using u32, u64, etc) live here
pub(crate) mod backend;

// Crate-local prelude (for alloc-dependent features like `Vec`)
pub(crate) mod prelude;

// Generic code for window lookups
pub(crate) mod window;
