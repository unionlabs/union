// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis lovecruft
// Copyright (c) 2016-2019 Henry de Valence
// Portions Copyright 2017 Brian Smith
// See LICENSE for licensing information.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>
// - Brian Smith <brian@briansmith.org>

//! Arithmetic on scalars (integers mod the group order).
//!
//! Both the Ristretto group and the Ed25519 basepoint have prime order
//! \\( \ell = 2\^{252} + 27742317777372353535851937790883648493 \\).
//!
//! This code is intended to be useful with both the Ristretto group
//! (where everything is done modulo \\( \ell \\)), and the X/Ed25519
//! setting, which mandates specific bit-twiddles that are not
//! well-defined modulo \\( \ell \\).
//!
//! All arithmetic on `Scalars` is done modulo \\( \ell \\).
//!
//! # Constructing a scalar
//!
//! To create a [`Scalar`](struct.Scalar.html) from a supposedly canonical encoding, use
//! [`Scalar::from_canonical_bytes`](struct.Scalar.html#method.from_canonical_bytes).
//!
//! This function does input validation, ensuring that the input bytes
//! are the canonical encoding of a `Scalar`.
//! If they are, we'll get
//! `Some(Scalar)` in return:
//!
//! ```
//! use curve25519_dalek::scalar::Scalar;
//!
//! let one_as_bytes: [u8; 32] = Scalar::one().to_bytes();
//! let a: Option<Scalar> = Scalar::from_canonical_bytes(one_as_bytes);
//!
//! assert!(a.is_some());
//! ```
//!
//! However, if we give it bytes representing a scalar larger than \\( \ell \\)
//! (in this case, \\( \ell + 2 \\)), we'll get `None` back:
//!
//! ```
//! use curve25519_dalek::scalar::Scalar;
//!
//! let l_plus_two_bytes: [u8; 32] = [
//!    0xef, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
//!    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
//!    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//!    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
//! ];
//! let a: Option<Scalar> = Scalar::from_canonical_bytes(l_plus_two_bytes);
//!
//! assert!(a.is_none());
//! ```
//!
//! Another way to create a `Scalar` is by reducing a \\(256\\)-bit integer mod
//! \\( \ell \\), for which one may use the
//! [`Scalar::from_bytes_mod_order`](struct.Scalar.html#method.from_bytes_mod_order)
//! method.  In the case of the second example above, this would reduce the
//! resultant scalar \\( \mod \ell \\), producing \\( 2 \\):
//!
//! ```
//! use curve25519_dalek::scalar::Scalar;
//!
//! let l_plus_two_bytes: [u8; 32] = [
//!    0xef, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
//!    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
//!    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//!    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
//! ];
//! let a: Scalar = Scalar::from_bytes_mod_order(l_plus_two_bytes);
//!
//! let two: Scalar = Scalar::one() + Scalar::one();
//!
//! assert!(a == two);
//! ```
//!
//! There is also a constructor that reduces a \\(512\\)-bit integer,
//! [`Scalar::from_bytes_mod_order_wide`](struct.Scalar.html#method.from_bytes_mod_order_wide).
//!
//! To construct a `Scalar` as the hash of some input data, use
//! [`Scalar::hash_from_bytes`](struct.Scalar.html#method.hash_from_bytes),
//! which takes a buffer, or
//! [`Scalar::from_hash`](struct.Scalar.html#method.from_hash),
//! which allows an IUF API.
//!
//! ```
//! # extern crate curve25519_dalek;
//! # extern crate sha2;
//! #
//! # fn main() {
//! use sha2::{Digest, Sha512};
//! use curve25519_dalek::scalar::Scalar;
//!
//! // Hashing a single byte slice
//! let a = Scalar::hash_from_bytes::<Sha512>(b"Abolish ICE");
//!
//! // Streaming data into a hash object
//! let mut hasher = Sha512::default();
//! hasher.update(b"Abolish ");
//! hasher.update(b"ICE");
//! let a2 = Scalar::from_hash(hasher);
//!
//! assert_eq!(a, a2);
//! # }
//! ```
//!
//! Finally, to create a `Scalar` with a specific bit-pattern
//! (e.g., for compatibility with X/Ed25519
//! ["clamping"](https://github.com/isislovecruft/ed25519-dalek/blob/f790bd2ce/src/ed25519.rs#L349)),
//! use [`Scalar::from_bits`](struct.Scalar.html#method.from_bits). This
//! constructs a scalar with exactly the bit pattern given, without any
//! assurances as to reduction modulo the group order:
//!
//! ```
//! use curve25519_dalek::scalar::Scalar;
//!
//! let l_plus_two_bytes: [u8; 32] = [
//!    0xef, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
//!    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
//!    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//!    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
//! ];
//! let a: Scalar = Scalar::from_bits(l_plus_two_bytes);
//!
//! let two: Scalar = Scalar::one() + Scalar::one();
//!
//! assert!(a != two);              // the scalar is not reduced (mod l)…
//! assert!(! a.is_canonical());    // …and therefore is not canonical.
//! assert!(a.reduce() == two);     // if we were to reduce it manually, it would be.
//! ```
//!
//! The resulting `Scalar` has exactly the specified bit pattern,
//! **except for the highest bit, which will be set to 0**.

use core::borrow::Borrow;
use core::cmp::{Eq, PartialEq};
use core::fmt::Debug;
use core::iter::{Product, Sum};
use core::ops::Index;
use core::ops::Neg;
use core::ops::{Add, AddAssign};
use core::ops::{Mul, MulAssign};
use core::ops::{Sub, SubAssign};

#[allow(unused_imports)]
use prelude::*;

use rand_core::{CryptoRng, RngCore};

use digest::generic_array::typenum::U64;
use digest::Digest;

use subtle::Choice;
use subtle::ConditionallySelectable;
use subtle::ConstantTimeEq;

use zeroize::Zeroize;

use backend;
use constants;

/// An `UnpackedScalar` represents an element of the field GF(l), optimized for speed.
///
/// This is a type alias for one of the scalar types in the `backend`
/// module.
#[cfg(feature = "fiat_u32_backend")]
type UnpackedScalar = backend::serial::fiat_u32::scalar::Scalar29;
#[cfg(feature = "fiat_u64_backend")]
type UnpackedScalar = backend::serial::fiat_u64::scalar::Scalar52;

/// An `UnpackedScalar` represents an element of the field GF(l), optimized for speed.
///
/// This is a type alias for one of the scalar types in the `backend`
/// module.
#[cfg(feature = "u64_backend")]
type UnpackedScalar = backend::serial::u64::scalar::Scalar52;

/// An `UnpackedScalar` represents an element of the field GF(l), optimized for speed.
///
/// This is a type alias for one of the scalar types in the `backend`
/// module.
#[cfg(feature = "u32_backend")]
type UnpackedScalar = backend::serial::u32::scalar::Scalar29;


/// The `Scalar` struct holds an integer \\(s < 2\^{255} \\) which
/// represents an element of \\(\mathbb Z / \ell\\).
#[derive(Copy, Clone, Hash)]
pub struct Scalar {
    /// `bytes` is a little-endian byte encoding of an integer representing a scalar modulo the
    /// group order.
    ///
    /// # Invariant
    ///
    /// The integer representing this scalar must be bounded above by \\(2\^{255}\\), or
    /// equivalently the high bit of `bytes[31]` must be zero.
    ///
    /// This ensures that there is room for a carry bit when computing a NAF representation.
    //
    // XXX This is pub(crate) so we can write literal constants.  If const fns were stable, we could
    //     make the Scalar constructors const fns and use those instead.
    pub(crate) bytes: [u8; 32],
}

impl Scalar {
    /// Construct a `Scalar` by reducing a 256-bit little-endian integer
    /// modulo the group order \\( \ell \\).
    pub fn from_bytes_mod_order(bytes: [u8; 32]) -> Scalar {
        // Temporarily allow s_unreduced.bytes > 2^255 ...
        let s_unreduced = Scalar{bytes};

        // Then reduce mod the group order and return the reduced representative.
        let s = s_unreduced.reduce();
        debug_assert_eq!(0u8, s[31] >> 7);

        s
    }

    /// Construct a `Scalar` by reducing a 512-bit little-endian integer
    /// modulo the group order \\( \ell \\).
    pub fn from_bytes_mod_order_wide(input: &[u8; 64]) -> Scalar {
        UnpackedScalar::from_bytes_wide(input).pack()
    }

    /// Attempt to construct a `Scalar` from a canonical byte representation.
    ///
    /// # Return
    ///
    /// - `Some(s)`, where `s` is the `Scalar` corresponding to `bytes`,
    ///   if `bytes` is a canonical byte representation;
    /// - `None` if `bytes` is not a canonical byte representation.
    pub fn from_canonical_bytes(bytes: [u8; 32]) -> Option<Scalar> {
        // Check that the high bit is not set
        if (bytes[31] >> 7) != 0u8 { return None; }
        let candidate = Scalar::from_bits(bytes);

        if candidate.is_canonical() {
            Some(candidate)
        } else {
            None
        }
    }

    /// Construct a `Scalar` from the low 255 bits of a 256-bit integer.
    ///
    /// This function is intended for applications like X25519 which
    /// require specific bit-patterns when performing scalar
    /// multiplication.
    pub const fn from_bits(bytes: [u8; 32]) -> Scalar {
        let mut s = Scalar{bytes};
        // Ensure that s < 2^255 by masking the high bit
        s.bytes[31] &= 0b0111_1111;

        s
    }
}

impl Debug for Scalar {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "Scalar{{\n\tbytes: {:?},\n}}", &self.bytes)
    }
}

impl Eq for Scalar {}
impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}

impl ConstantTimeEq for Scalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.bytes.ct_eq(&other.bytes)
    }
}

impl Index<usize> for Scalar {
    type Output = u8;

    /// Index the bytes of the representative for this `Scalar`.  Mutation is not permitted.
    fn index(&self, _index: usize) -> &u8 {
        &(self.bytes[_index])
    }
}

impl<'b> MulAssign<&'b Scalar> for Scalar {
    fn mul_assign(&mut self, _rhs: &'b Scalar) {
        *self = UnpackedScalar::mul(&self.unpack(), &_rhs.unpack()).pack();
    }
}

define_mul_assign_variants!(LHS = Scalar, RHS = Scalar);

impl<'a, 'b> Mul<&'b Scalar> for &'a Scalar {
    type Output = Scalar;
    fn mul(self, _rhs: &'b Scalar) -> Scalar {
        UnpackedScalar::mul(&self.unpack(), &_rhs.unpack()).pack()
    }
}

define_mul_variants!(LHS = Scalar, RHS = Scalar, Output = Scalar);

impl<'b> AddAssign<&'b Scalar> for Scalar {
    fn add_assign(&mut self, _rhs: &'b Scalar) {
        *self = *self + _rhs;
    }
}

define_add_assign_variants!(LHS = Scalar, RHS = Scalar);

impl<'a, 'b> Add<&'b Scalar> for &'a Scalar {
    type Output = Scalar;
    #[allow(non_snake_case)]
    fn add(self, _rhs: &'b Scalar) -> Scalar {
        // The UnpackedScalar::add function produces reduced outputs
        // if the inputs are reduced.  However, these inputs may not
        // be reduced -- they might come from Scalar::from_bits.  So
        // after computing the sum, we explicitly reduce it mod l
        // before repacking.
        let sum = UnpackedScalar::add(&self.unpack(), &_rhs.unpack());
        let sum_R = UnpackedScalar::mul_internal(&sum, &constants::R);
        let sum_mod_l = UnpackedScalar::montgomery_reduce(&sum_R);
        sum_mod_l.pack()
    }
}

define_add_variants!(LHS = Scalar, RHS = Scalar, Output = Scalar);

impl<'b> SubAssign<&'b Scalar> for Scalar {
    fn sub_assign(&mut self, _rhs: &'b Scalar) {
        *self = *self - _rhs;
    }
}

define_sub_assign_variants!(LHS = Scalar, RHS = Scalar);

impl<'a, 'b> Sub<&'b Scalar> for &'a Scalar {
    type Output = Scalar;
    #[allow(non_snake_case)]
    fn sub(self, rhs: &'b Scalar) -> Scalar {
        // The UnpackedScalar::sub function requires reduced inputs
        // and produces reduced output. However, these inputs may not
        // be reduced -- they might come from Scalar::from_bits.  So
        // we explicitly reduce the inputs.
        let self_R = UnpackedScalar::mul_internal(&self.unpack(), &constants::R);
        let self_mod_l = UnpackedScalar::montgomery_reduce(&self_R);
        let rhs_R = UnpackedScalar::mul_internal(&rhs.unpack(), &constants::R);
        let rhs_mod_l = UnpackedScalar::montgomery_reduce(&rhs_R);

        UnpackedScalar::sub(&self_mod_l, &rhs_mod_l).pack()
    }
}

define_sub_variants!(LHS = Scalar, RHS = Scalar, Output = Scalar);

impl<'a> Neg for &'a Scalar {
    type Output = Scalar;
    #[allow(non_snake_case)]
    fn neg(self) -> Scalar {
        let self_R = UnpackedScalar::mul_internal(&self.unpack(), &constants::R);
        let self_mod_l = UnpackedScalar::montgomery_reduce(&self_R);
        UnpackedScalar::sub(&UnpackedScalar::zero(), &self_mod_l).pack()
    }
}

impl<'a> Neg for Scalar {
    type Output = Scalar;
    fn neg(self) -> Scalar {
        -&self
    }
}

impl ConditionallySelectable for Scalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::conditional_select(&a.bytes[i], &b.bytes[i], choice);
        }
        Scalar { bytes }
    }
}

#[cfg(feature = "serde")]
use serde::{self, Serialize, Deserialize, Serializer, Deserializer};
#[cfg(feature = "serde")]
use serde::de::Visitor;

#[cfg(feature = "serde")]
impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(32)?;
        for byte in self.as_bytes().iter() {
            tup.serialize_element(byte)?;
        }
        tup.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ScalarVisitor;

        impl<'de> Visitor<'de> for ScalarVisitor {
            type Value = Scalar;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("a valid point in Edwards y + sign format")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Scalar, A::Error>
                where A: serde::de::SeqAccess<'de>
            {
                let mut bytes = [0u8; 32];
                for i in 0..32 {
                    bytes[i] = seq.next_element()?
                        .ok_or(serde::de::Error::invalid_length(i, &"expected 32 bytes"))?;
                }
                Scalar::from_canonical_bytes(bytes)
                    .ok_or(serde::de::Error::custom(
                        &"scalar was not canonically encoded"
                    ))
            }
        }

        deserializer.deserialize_tuple(32, ScalarVisitor)
    }
}

impl<T> Product<T> for Scalar
where
    T: Borrow<Scalar>
{
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>
    {
        iter.fold(Scalar::one(), |acc, item| acc * item.borrow())
    }
}

impl<T> Sum<T> for Scalar
where
    T: Borrow<Scalar>
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>
    {
        iter.fold(Scalar::zero(), |acc, item| acc + item.borrow())
    }
}

impl Default for Scalar {
    fn default() -> Scalar {
        Scalar::zero()
    }
}

impl From<u8> for Scalar {
    fn from(x: u8) -> Scalar {
        let mut s_bytes = [0u8; 32];
        s_bytes[0] = x;
        Scalar{ bytes: s_bytes }
    }
}

impl From<u16> for Scalar {
    fn from(x: u16) -> Scalar {
        use byteorder::{ByteOrder, LittleEndian};
        let mut s_bytes = [0u8; 32];
        LittleEndian::write_u16(&mut s_bytes, x);
        Scalar{ bytes: s_bytes }
    }
}

impl From<u32> for Scalar {
    fn from(x: u32) -> Scalar {
        use byteorder::{ByteOrder, LittleEndian};
        let mut s_bytes = [0u8; 32];
        LittleEndian::write_u32(&mut s_bytes, x);
        Scalar{ bytes: s_bytes }
    }
}

impl From<u64> for Scalar {
    /// Construct a scalar from the given `u64`.
    ///
    /// # Inputs
    ///
    /// An `u64` to convert to a `Scalar`.
    ///
    /// # Returns
    ///
    /// A `Scalar` corresponding to the input `u64`.
    ///
    /// # Example
    ///
    /// ```
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// let fourtytwo = Scalar::from(42u64);
    /// let six = Scalar::from(6u64);
    /// let seven = Scalar::from(7u64);
    ///
    /// assert!(fourtytwo == six * seven);
    /// ```
    fn from(x: u64) -> Scalar {
        use byteorder::{ByteOrder, LittleEndian};
        let mut s_bytes = [0u8; 32];
        LittleEndian::write_u64(&mut s_bytes, x);
        Scalar{ bytes: s_bytes }
    }
}

impl From<u128> for Scalar {
    fn from(x: u128) -> Scalar {
        use byteorder::{ByteOrder, LittleEndian};
        let mut s_bytes = [0u8; 32];
        LittleEndian::write_u128(&mut s_bytes, x);
        Scalar{ bytes: s_bytes }
    }
}

impl Zeroize for Scalar {
    fn zeroize(&mut self) {
        self.bytes.zeroize();
    }
}

impl Scalar {
    /// Return a `Scalar` chosen uniformly at random using a user-provided RNG.
    ///
    /// # Inputs
    ///
    /// * `rng`: any RNG which implements the `RngCore + CryptoRng` interface.
    ///
    /// # Returns
    ///
    /// A random scalar within ℤ/lℤ.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate rand_core;
    /// # extern crate curve25519_dalek;
    /// #
    /// # fn main() {
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// use rand_core::OsRng;
    ///
    /// let mut csprng = OsRng;
    /// let a: Scalar = Scalar::random(&mut csprng);
    /// # }
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut scalar_bytes = [0u8; 64];
        rng.fill_bytes(&mut scalar_bytes);
        Scalar::from_bytes_mod_order_wide(&scalar_bytes)
    }

    /// Hash a slice of bytes into a scalar.
    ///
    /// Takes a type parameter `D`, which is any `Digest` producing 64
    /// bytes (512 bits) of output.
    ///
    /// Convenience wrapper around `from_hash`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate curve25519_dalek;
    /// # use curve25519_dalek::scalar::Scalar;
    /// extern crate sha2;
    ///
    /// use sha2::Sha512;
    ///
    /// # // Need fn main() here in comment so the doctest compiles
    /// # // See https://doc.rust-lang.org/book/documentation.html#documentation-as-tests
    /// # fn main() {
    /// let msg = "To really appreciate architecture, you may even need to commit a murder";
    /// let s = Scalar::hash_from_bytes::<Sha512>(msg.as_bytes());
    /// # }
    /// ```
    pub fn hash_from_bytes<D>(input: &[u8]) -> Scalar
        where D: Digest<OutputSize = U64> + Default
    {
        let mut hash = D::default();
        hash.update(input);
        Scalar::from_hash(hash)
    }

    /// Construct a scalar from an existing `Digest` instance.
    ///
    /// Use this instead of `hash_from_bytes` if it is more convenient
    /// to stream data into the `Digest` than to pass a single byte
    /// slice.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate curve25519_dalek;
    /// # use curve25519_dalek::scalar::Scalar;
    /// extern crate sha2;
    ///
    /// use sha2::Digest;
    /// use sha2::Sha512;
    ///
    /// # fn main() {
    /// let mut h = Sha512::new()
    ///     .chain("To really appreciate architecture, you may even need to commit a murder.")
    ///     .chain("While the programs used for The Manhattan Transcripts are of the most extreme")
    ///     .chain("nature, they also parallel the most common formula plot: the archetype of")
    ///     .chain("murder. Other phantasms were occasionally used to underline the fact that")
    ///     .chain("perhaps all architecture, rather than being about functional standards, is")
    ///     .chain("about love and death.");
    ///
    /// let s = Scalar::from_hash(h);
    ///
    /// println!("{:?}", s.to_bytes());
    /// assert!(s == Scalar::from_bits([ 21,  88, 208, 252,  63, 122, 210, 152,
    ///                                 154,  38,  15,  23,  16, 167,  80, 150,
    ///                                 192, 221,  77, 226,  62,  25, 224, 148,
    ///                                 239,  48, 176,  10, 185,  69, 168,  11, ]));
    /// # }
    /// ```
    pub fn from_hash<D>(hash: D) -> Scalar
        where D: Digest<OutputSize = U64>
    {
        let mut output = [0u8; 64];
        output.copy_from_slice(hash.finalize().as_slice());
        Scalar::from_bytes_mod_order_wide(&output)
    }

    /// Convert this `Scalar` to its underlying sequence of bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// let s: Scalar = Scalar::zero();
    ///
    /// assert!(s.to_bytes() == [0u8; 32]);
    /// ```
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }

    /// View the little-endian byte encoding of the integer representing this Scalar.
    ///
    /// # Example
    ///
    /// ```
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// let s: Scalar = Scalar::zero();
    ///
    /// assert!(s.as_bytes() == &[0u8; 32]);
    /// ```
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Construct the scalar \\( 0 \\).
    pub fn zero() -> Self {
        Scalar { bytes: [0u8; 32]}
    }

    /// Construct the scalar \\( 1 \\).
    pub fn one() -> Self {
        Scalar {
            bytes: [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        }
    }

    /// Given a nonzero `Scalar`, compute its multiplicative inverse.
    ///
    /// # Warning
    ///
    /// `self` **MUST** be nonzero.  If you cannot
    /// *prove* that this is the case, you **SHOULD NOT USE THIS
    /// FUNCTION**.
    ///
    /// # Returns
    ///
    /// The multiplicative inverse of the this `Scalar`.
    ///
    /// # Example
    ///
    /// ```
    /// use curve25519_dalek::scalar::Scalar;
    ///
    /// // x = 2238329342913194256032495932344128051776374960164957527413114840482143558222
    /// let X: Scalar = Scalar::from_bytes_mod_order([
    ///         0x4e, 0x5a, 0xb4, 0x34, 0x5d, 0x47, 0x08, 0x84,
    ///         0x59, 0x13, 0xb4, 0x64, 0x1b, 0xc2, 0x7d, 0x52,
    ///         0x52, 0xa5, 0x85, 0x10, 0x1b, 0xcc, 0x42, 0x44,
    ///         0xd4, 0x49, 0xf4, 0xa8, 0x79, 0xd9, 0xf2, 0x04,
    ///     ]);
    /// // 1/x = 6859937278830797291664592131120606308688036382723378951768035303146619657244
    /// let XINV: Scalar = Scalar::from_bytes_mod_order([
    ///         0x1c, 0xdc, 0x17, 0xfc, 0xe0, 0xe9, 0xa5, 0xbb,
    ///         0xd9, 0x24, 0x7e, 0x56, 0xbb, 0x01, 0x63, 0x47,
    ///         0xbb, 0xba, 0x31, 0xed, 0xd5, 0xa9, 0xbb, 0x96,
    ///         0xd5, 0x0b, 0xcd, 0x7a, 0x3f, 0x96, 0x2a, 0x0f,
    ///     ]);
    ///
    /// let inv_X: Scalar = X.invert();
    /// assert!(XINV == inv_X);
    /// let should_be_one: Scalar = &inv_X * &X;
    /// assert!(should_be_one == Scalar::one());
    /// ```
    pub fn invert(&self) -> Scalar {
        self.unpack().invert().pack()
    }

    /// Given a slice of nonzero (possibly secret) `Scalar`s,
    /// compute their inverses in a batch.
    ///
    /// # Return
    ///
    /// Each element of `inputs` is replaced by its inverse.
    ///
    /// The product of all inverses is returned.
    ///
    /// # Warning
    ///
    /// All input `Scalars` **MUST** be nonzero.  If you cannot
    /// *prove* that this is the case, you **SHOULD NOT USE THIS
    /// FUNCTION**.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate curve25519_dalek;
    /// # use curve25519_dalek::scalar::Scalar;
    /// # fn main() {
    /// let mut scalars = [
    ///     Scalar::from(3u64),
    ///     Scalar::from(5u64),
    ///     Scalar::from(7u64),
    ///     Scalar::from(11u64),
    /// ];
    ///
    /// let allinv = Scalar::batch_invert(&mut scalars);
    ///
    /// assert_eq!(allinv, Scalar::from(3*5*7*11u64).invert());
    /// assert_eq!(scalars[0], Scalar::from(3u64).invert());
    /// assert_eq!(scalars[1], Scalar::from(5u64).invert());
    /// assert_eq!(scalars[2], Scalar::from(7u64).invert());
    /// assert_eq!(scalars[3], Scalar::from(11u64).invert());
    /// # }
    /// ```
    #[cfg(feature = "alloc")]
    pub fn batch_invert(inputs: &mut [Scalar]) -> Scalar {
        // This code is essentially identical to the FieldElement
        // implementation, and is documented there.  Unfortunately,
        // it's not easy to write it generically, since here we want
        // to use `UnpackedScalar`s internally, and `Scalar`s
        // externally, but there's no corresponding distinction for
        // field elements.

        use zeroize::Zeroizing;

        let n = inputs.len();
        let one: UnpackedScalar = Scalar::one().unpack().to_montgomery();

        // Place scratch storage in a Zeroizing wrapper to wipe it when
        // we pass out of scope.
        let scratch_vec = vec![one; n];
        let mut scratch = Zeroizing::new(scratch_vec);

        // Keep an accumulator of all of the previous products
        let mut acc = Scalar::one().unpack().to_montgomery();

        // Pass through the input vector, recording the previous
        // products in the scratch space
        for (input, scratch) in inputs.iter_mut().zip(scratch.iter_mut()) {
            *scratch = acc;

            // Avoid unnecessary Montgomery multiplication in second pass by
            // keeping inputs in Montgomery form
            let tmp = input.unpack().to_montgomery();
            *input = tmp.pack();
            acc = UnpackedScalar::montgomery_mul(&acc, &tmp);
        }

        // acc is nonzero iff all inputs are nonzero
        debug_assert!(acc.pack() != Scalar::zero());

        // Compute the inverse of all products
        acc = acc.montgomery_invert().from_montgomery();

        // We need to return the product of all inverses later
        let ret = acc.pack();

        // Pass through the vector backwards to compute the inverses
        // in place
        for (input, scratch) in inputs.iter_mut().rev().zip(scratch.iter().rev()) {
            let tmp = UnpackedScalar::montgomery_mul(&acc, &input.unpack());
            *input = UnpackedScalar::montgomery_mul(&acc, &scratch).pack();
            acc = tmp;
        }

        ret
    }

    /// Get the bits of the scalar.
    pub(crate) fn bits(&self) -> [i8; 256] {
        let mut bits = [0i8; 256];
        for i in 0..256 {
            // As i runs from 0..256, the bottom 3 bits index the bit,
            // while the upper bits index the byte.
            bits[i] = ((self.bytes[i>>3] >> (i&7)) & 1u8) as i8;
        }
        bits
    }

    /// Compute a width-\\(w\\) "Non-Adjacent Form" of this scalar.
    ///
    /// A width-\\(w\\) NAF of a positive integer \\(k\\) is an expression
    /// $$
    /// k = \sum_{i=0}\^m n\_i 2\^i,
    /// $$
    /// where each nonzero
    /// coefficient \\(n\_i\\) is odd and bounded by \\(|n\_i| < 2\^{w-1}\\),
    /// \\(n\_{m-1}\\) is nonzero, and at most one of any \\(w\\) consecutive
    /// coefficients is nonzero.  (Hankerson, Menezes, Vanstone; def 3.32).
    ///
    /// The length of the NAF is at most one more than the length of
    /// the binary representation of \\(k\\).  This is why the
    /// `Scalar` type maintains an invariant that the top bit is
    /// \\(0\\), so that the NAF of a scalar has at most 256 digits.
    ///
    /// Intuitively, this is like a binary expansion, except that we
    /// allow some coefficients to grow in magnitude up to
    /// \\(2\^{w-1}\\) so that the nonzero coefficients are as sparse
    /// as possible.
    ///
    /// When doing scalar multiplication, we can then use a lookup
    /// table of precomputed multiples of a point to add the nonzero
    /// terms \\( k_i P \\).  Using signed digits cuts the table size
    /// in half, and using odd digits cuts the table size in half
    /// again.
    ///
    /// To compute a \\(w\\)-NAF, we use a modification of Algorithm 3.35 of HMV:
    ///
    /// 1. \\( i \gets 0 \\)
    /// 2. While \\( k \ge 1 \\):
    ///     1. If \\(k\\) is odd, \\( n_i \gets k \operatorname{mods} 2^w \\), \\( k \gets k - n_i \\).
    ///     2. If \\(k\\) is even, \\( n_i \gets 0 \\).
    ///     3. \\( k \gets k / 2 \\), \\( i \gets i + 1 \\).
    /// 3. Return \\( n_0, n_1, ... , \\)
    ///
    /// Here \\( \bar x = x \operatorname{mods} 2^w \\) means the
    /// \\( \bar x \\) with \\( \bar x \equiv x \pmod{2^w} \\) and
    /// \\( -2^{w-1} \leq \bar x < 2^w \\).
    ///
    /// We implement this by scanning across the bits of \\(k\\) from
    /// least-significant bit to most-significant-bit.
    /// Write the bits of \\(k\\) as
    /// $$
    /// k = \sum\_{i=0}\^m k\_i 2^i,
    /// $$
    /// and split the sum as
    /// $$
    /// k = \sum\_{i=0}^{w-1} k\_i 2^i + 2^w \sum\_{i=0} k\_{i+w} 2^i
    /// $$
    /// where the first part is \\( k \mod 2^w \\).
    ///
    /// If \\( k \mod 2^w\\) is odd, and \\( k \mod 2^w < 2^{w-1} \\), then we emit
    /// \\( n_0 = k \mod 2^w \\).  Instead of computing
    /// \\( k - n_0 \\), we just advance \\(w\\) bits and reindex.
    ///
    /// If \\( k \mod 2^w\\) is odd, and \\( k \mod 2^w \ge 2^{w-1} \\), then
    /// \\( n_0 = k \operatorname{mods} 2^w = k \mod 2^w - 2^w \\).
    /// The quantity \\( k - n_0 \\) is
    /// $$
    /// \begin{aligned}
    /// k - n_0 &= \sum\_{i=0}^{w-1} k\_i 2^i + 2^w \sum\_{i=0} k\_{i+w} 2^i
    ///          - \sum\_{i=0}^{w-1} k\_i 2^i + 2^w \\\\
    /// &= 2^w + 2^w \sum\_{i=0} k\_{i+w} 2^i
    /// \end{aligned}
    /// $$
    /// so instead of computing the subtraction, we can set a carry
    /// bit, advance \\(w\\) bits, and reindex.
    ///
    /// If \\( k \mod 2^w\\) is even, we emit \\(0\\), advance 1 bit
    /// and reindex.  In fact, by setting all digits to \\(0\\)
    /// initially, we don't need to emit anything.
    pub(crate) fn non_adjacent_form(&self, w: usize) -> [i8; 256] {
        // required by the NAF definition
        debug_assert!( w >= 2 );
        // required so that the NAF digits fit in i8
        debug_assert!( w <= 8 );

        use byteorder::{ByteOrder, LittleEndian};

        let mut naf = [0i8; 256];

        let mut x_u64 = [0u64; 5];
        LittleEndian::read_u64_into(&self.bytes, &mut x_u64[0..4]);

        let width = 1 << w;
        let window_mask = width - 1;

        let mut pos = 0;
        let mut carry = 0;
        while pos < 256 {
            // Construct a buffer of bits of the scalar, starting at bit `pos`
            let u64_idx = pos / 64;
            let bit_idx = pos % 64;
            let bit_buf: u64;
            if bit_idx < 64 - w {
                // This window's bits are contained in a single u64
                bit_buf = x_u64[u64_idx] >> bit_idx;
            } else {
                // Combine the current u64's bits with the bits from the next u64
                bit_buf = (x_u64[u64_idx] >> bit_idx) | (x_u64[1+u64_idx] << (64 - bit_idx));
            }

            // Add the carry into the current window
            let window = carry + (bit_buf & window_mask);

            if window & 1 == 0 {
                // If the window value is even, preserve the carry and continue.
                // Why is the carry preserved?
                // If carry == 0 and window & 1 == 0, then the next carry should be 0
                // If carry == 1 and window & 1 == 0, then bit_buf & 1 == 1 so the next carry should be 1
                pos += 1;
                continue;
            }

            if window < width/2 {
                carry = 0;
                naf[pos] = window as i8;
            } else {
                carry = 1;
                naf[pos] = (window as i8).wrapping_sub(width as i8);
            }

            pos += w;
        }

        naf
    }

    /// Write this scalar in radix 16, with coefficients in \\([-8,8)\\),
    /// i.e., compute \\(a\_i\\) such that
    /// $$
    ///    a = a\_0 + a\_1 16\^1 + \cdots + a_{63} 16\^{63},
    /// $$
    /// with \\(-8 \leq a_i < 8\\) for \\(0 \leq i < 63\\) and \\(-8 \leq a_{63} \leq 8\\).
    pub(crate) fn to_radix_16(&self) -> [i8; 64] {
        debug_assert!(self[31] <= 127);
        let mut output = [0i8; 64];

        // Step 1: change radix.
        // Convert from radix 256 (bytes) to radix 16 (nibbles)
        #[inline(always)]
        fn bot_half(x: u8) -> u8 { (x >> 0) & 15 }
        #[inline(always)]
        fn top_half(x: u8) -> u8 { (x >> 4) & 15 }

        for i in 0..32 {
            output[2*i  ] = bot_half(self[i]) as i8;
            output[2*i+1] = top_half(self[i]) as i8;
        }
        // Precondition note: since self[31] <= 127, output[63] <= 7

        // Step 2: recenter coefficients from [0,16) to [-8,8)
        for i in 0..63 {
            let carry    = (output[i] + 8) >> 4;
            output[i  ] -= carry << 4;
            output[i+1] += carry;
        }
        // Precondition note: output[63] is not recentered.  It
        // increases by carry <= 1.  Thus output[63] <= 8.

        output
    }

    /// Returns a size hint indicating how many entries of the return
    /// value of `to_radix_2w` are nonzero.
    pub(crate) fn to_radix_2w_size_hint(w: usize) -> usize {
        debug_assert!(w >= 4);
        debug_assert!(w <= 8);

        let digits_count = match w {
            4 => (256 + w - 1)/w as usize,
            5 => (256 + w - 1)/w as usize,
            6 => (256 + w - 1)/w as usize,
            7 => (256 + w - 1)/w as usize,
            // See comment in to_radix_2w on handling the terminal carry.
            8 => (256 + w - 1)/w + 1 as usize,
            _ => panic!("invalid radix parameter"),
        };

        debug_assert!(digits_count <= 64);
        digits_count
    }

    /// Creates a representation of a Scalar in radix 32, 64, 128 or 256 for use with the Pippenger algorithm.
    /// For lower radix, use `to_radix_16`, which is used by the Straus multi-scalar multiplication.
    /// Higher radixes are not supported to save cache space. Radix 256 is near-optimal even for very
    /// large inputs.
    ///
    /// Radix below 32 or above 256 is prohibited.
    /// This method returns digits in a fixed-sized array, excess digits are zeroes.
    ///
    /// ## Scalar representation
    ///
    /// Radix \\(2\^w\\), with \\(n = ceil(256/w)\\) coefficients in \\([-(2\^w)/2,(2\^w)/2)\\),
    /// i.e., scalar is represented using digits \\(a\_i\\) such that
    /// $$
    ///    a = a\_0 + a\_1 2\^1w + \cdots + a_{n-1} 2\^{w*(n-1)},
    /// $$
    /// with \\(-2\^w/2 \leq a_i < 2\^w/2\\) for \\(0 \leq i < (n-1)\\) and \\(-2\^w/2 \leq a_{n-1} \leq 2\^w/2\\).
    ///
    pub(crate) fn to_radix_2w(&self, w: usize) -> [i8; 64] {
        debug_assert!(w >= 4);
        debug_assert!(w <= 8);

        if w == 4 {
            return self.to_radix_16();
        }

        use byteorder::{ByteOrder, LittleEndian};

        // Scalar formatted as four `u64`s with carry bit packed into the highest bit.
        let mut scalar64x4 = [0u64; 4];
        LittleEndian::read_u64_into(&self.bytes, &mut scalar64x4[0..4]);

        let radix: u64 = 1 << w;
        let window_mask: u64 = radix - 1;

        let mut carry = 0u64;
        let mut digits = [0i8; 64];
        let digits_count = (256 + w - 1)/w as usize;
        for i in 0..digits_count {
            // Construct a buffer of bits of the scalar, starting at `bit_offset`.
            let bit_offset = i*w;
            let u64_idx = bit_offset / 64;
            let bit_idx = bit_offset % 64;

            // Read the bits from the scalar
            let bit_buf: u64;
            if bit_idx < 64 - w  || u64_idx == 3 {
                // This window's bits are contained in a single u64,
                // or it's the last u64 anyway.
                bit_buf = scalar64x4[u64_idx] >> bit_idx;
            } else {
                // Combine the current u64's bits with the bits from the next u64
                bit_buf = (scalar64x4[u64_idx] >> bit_idx) | (scalar64x4[1+u64_idx] << (64 - bit_idx));
            }

            // Read the actual coefficient value from the window
            let coef = carry + (bit_buf & window_mask); // coef = [0, 2^r)

             // Recenter coefficients from [0,2^w) to [-2^w/2, 2^w/2)
            carry = (coef + (radix/2) as u64) >> w;
            digits[i] = ((coef as i64) - (carry << w) as i64) as i8;
        }

        // When w < 8, we can fold the final carry onto the last digit d,
        // because d < 2^w/2 so d + carry*2^w = d + 1*2^w < 2^(w+1) < 2^8.
        //
        // When w = 8, we can't fit carry*2^w into an i8.  This should
        // not happen anyways, because the final carry will be 0 for
        // reduced scalars, but the Scalar invariant allows 255-bit scalars.
        // To handle this, we expand the size_hint by 1 when w=8,
        // and accumulate the final carry onto another digit.
        match w {
            8 => digits[digits_count] += carry as i8,
            _ => digits[digits_count-1] += (carry << w) as i8,
        }

        digits
    }

    /// Unpack this `Scalar` to an `UnpackedScalar` for faster arithmetic.
    pub(crate) fn unpack(&self) -> UnpackedScalar {
        UnpackedScalar::from_bytes(&self.bytes)
    }

    /// Reduce this `Scalar` modulo \\(\ell\\).
    #[allow(non_snake_case)]
    pub fn reduce(&self) -> Scalar {
        let x = self.unpack();
        let xR = UnpackedScalar::mul_internal(&x, &constants::R);
        let x_mod_l = UnpackedScalar::montgomery_reduce(&xR);
        x_mod_l.pack()
    }

    /// Check whether this `Scalar` is the canonical representative mod \\(\ell\\).
    ///
    /// This is intended for uses like input validation, where variable-time code is acceptable.
    ///
    /// ```
    /// # extern crate curve25519_dalek;
    /// # extern crate subtle;
    /// # use curve25519_dalek::scalar::Scalar;
    /// # use subtle::ConditionallySelectable;
    /// # fn main() {
    /// // 2^255 - 1, since `from_bits` clears the high bit
    /// let _2_255_minus_1 = Scalar::from_bits([0xff;32]);
    /// assert!(!_2_255_minus_1.is_canonical());
    ///
    /// let reduced = _2_255_minus_1.reduce();
    /// assert!(reduced.is_canonical());
    /// # }
    /// ```
    pub fn is_canonical(&self) -> bool {
        *self == self.reduce()
    }
}

impl UnpackedScalar {
    /// Pack the limbs of this `UnpackedScalar` into a `Scalar`.
    fn pack(&self) -> Scalar {
        Scalar{ bytes: self.to_bytes() }
    }

    /// Inverts an UnpackedScalar in Montgomery form.
    pub fn montgomery_invert(&self) -> UnpackedScalar {
        // Uses the addition chain from
        // https://briansmith.org/ecc-inversion-addition-chains-01#curve25519_scalar_inversion
        let    _1 = self;
        let   _10 = _1.montgomery_square();
        let  _100 = _10.montgomery_square();
        let   _11 = UnpackedScalar::montgomery_mul(&_10,     &_1);
        let  _101 = UnpackedScalar::montgomery_mul(&_10,    &_11);
        let  _111 = UnpackedScalar::montgomery_mul(&_10,   &_101);
        let _1001 = UnpackedScalar::montgomery_mul(&_10,   &_111);
        let _1011 = UnpackedScalar::montgomery_mul(&_10,  &_1001);
        let _1111 = UnpackedScalar::montgomery_mul(&_100, &_1011);

        // _10000
        let mut y = UnpackedScalar::montgomery_mul(&_1111, &_1);

        #[inline]
        fn square_multiply(y: &mut UnpackedScalar, squarings: usize, x: &UnpackedScalar) {
            for _ in 0..squarings {
                *y = y.montgomery_square();
            }
            *y = UnpackedScalar::montgomery_mul(y, x);
        }

        square_multiply(&mut y, 123 + 3, &_101);
        square_multiply(&mut y,   2 + 2, &_11);
        square_multiply(&mut y,   1 + 4, &_1111);
        square_multiply(&mut y,   1 + 4, &_1111);
        square_multiply(&mut y,       4, &_1001);
        square_multiply(&mut y,       2, &_11);
        square_multiply(&mut y,   1 + 4, &_1111);
        square_multiply(&mut y,   1 + 3, &_101);
        square_multiply(&mut y,   3 + 3, &_101);
        square_multiply(&mut y,       3, &_111);
        square_multiply(&mut y,   1 + 4, &_1111);
        square_multiply(&mut y,   2 + 3, &_111);
        square_multiply(&mut y,   2 + 2, &_11);
        square_multiply(&mut y,   1 + 4, &_1011);
        square_multiply(&mut y,   2 + 4, &_1011);
        square_multiply(&mut y,   6 + 4, &_1001);
        square_multiply(&mut y,   2 + 2, &_11);
        square_multiply(&mut y,   3 + 2, &_11);
        square_multiply(&mut y,   3 + 2, &_11);
        square_multiply(&mut y,   1 + 4, &_1001);
        square_multiply(&mut y,   1 + 3, &_111);
        square_multiply(&mut y,   2 + 4, &_1111);
        square_multiply(&mut y,   1 + 4, &_1011);
        square_multiply(&mut y,       3, &_101);
        square_multiply(&mut y,   2 + 4, &_1111);
        square_multiply(&mut y,       3, &_101);
        square_multiply(&mut y,   1 + 2, &_11);

        y
    }

    /// Inverts an UnpackedScalar not in Montgomery form.
    pub fn invert(&self) -> UnpackedScalar {
        self.to_montgomery().montgomery_invert().from_montgomery()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use constants;

    /// x = 2238329342913194256032495932344128051776374960164957527413114840482143558222
    pub static X: Scalar = Scalar{
        bytes: [
            0x4e, 0x5a, 0xb4, 0x34, 0x5d, 0x47, 0x08, 0x84,
            0x59, 0x13, 0xb4, 0x64, 0x1b, 0xc2, 0x7d, 0x52,
            0x52, 0xa5, 0x85, 0x10, 0x1b, 0xcc, 0x42, 0x44,
            0xd4, 0x49, 0xf4, 0xa8, 0x79, 0xd9, 0xf2, 0x04,
        ],
    };
    /// 1/x = 6859937278830797291664592131120606308688036382723378951768035303146619657244
    pub static XINV: Scalar = Scalar{
        bytes: [
            0x1c, 0xdc, 0x17, 0xfc, 0xe0, 0xe9, 0xa5, 0xbb,
            0xd9, 0x24, 0x7e, 0x56, 0xbb, 0x01, 0x63, 0x47,
            0xbb, 0xba, 0x31, 0xed, 0xd5, 0xa9, 0xbb, 0x96,
            0xd5, 0x0b, 0xcd, 0x7a, 0x3f, 0x96, 0x2a, 0x0f,
        ],
    };
    /// y = 2592331292931086675770238855846338635550719849568364935475441891787804997264
    pub static Y: Scalar = Scalar{
        bytes: [
            0x90, 0x76, 0x33, 0xfe, 0x1c, 0x4b, 0x66, 0xa4,
            0xa2, 0x8d, 0x2d, 0xd7, 0x67, 0x83, 0x86, 0xc3,
            0x53, 0xd0, 0xde, 0x54, 0x55, 0xd4, 0xfc, 0x9d,
            0xe8, 0xef, 0x7a, 0xc3, 0x1f, 0x35, 0xbb, 0x05,
        ],
    };

    /// x*y = 5690045403673944803228348699031245560686958845067437804563560795922180092780
    static X_TIMES_Y: Scalar = Scalar{
        bytes: [
            0x6c, 0x33, 0x74, 0xa1, 0x89, 0x4f, 0x62, 0x21,
            0x0a, 0xaa, 0x2f, 0xe1, 0x86, 0xa6, 0xf9, 0x2c,
            0xe0, 0xaa, 0x75, 0xc2, 0x77, 0x95, 0x81, 0xc2,
            0x95, 0xfc, 0x08, 0x17, 0x9a, 0x73, 0x94, 0x0c,
        ],
    };

    /// sage: l = 2^252 + 27742317777372353535851937790883648493
    /// sage: big = 2^256 - 1
    /// sage: repr((big % l).digits(256))
    static CANONICAL_2_256_MINUS_1: Scalar = Scalar{
        bytes: [
              28, 149, 152, 141, 116,  49, 236, 214,
             112, 207, 125, 115, 244,  91, 239, 198,
             254, 255, 255, 255, 255, 255, 255, 255,
             255, 255, 255, 255, 255, 255, 255,  15,
        ],
    };

    static A_SCALAR: Scalar = Scalar{
        bytes: [
            0x1a, 0x0e, 0x97, 0x8a, 0x90, 0xf6, 0x62, 0x2d,
            0x37, 0x47, 0x02, 0x3f, 0x8a, 0xd8, 0x26, 0x4d,
            0xa7, 0x58, 0xaa, 0x1b, 0x88, 0xe0, 0x40, 0xd1,
            0x58, 0x9e, 0x7b, 0x7f, 0x23, 0x76, 0xef, 0x09,
        ],
    };

    static A_NAF: [i8; 256] =
        [0,13,0,0,0,0,0,0,0,7,0,0,0,0,0,0,-9,0,0,0,0,-11,0,0,0,0,3,0,0,0,0,1,
         0,0,0,0,9,0,0,0,0,-5,0,0,0,0,0,0,3,0,0,0,0,11,0,0,0,0,11,0,0,0,0,0,
         -9,0,0,0,0,0,-3,0,0,0,0,9,0,0,0,0,0,1,0,0,0,0,0,0,-1,0,0,0,0,0,9,0,
         0,0,0,-15,0,0,0,0,-7,0,0,0,0,-9,0,0,0,0,0,5,0,0,0,0,13,0,0,0,0,0,-3,0,
         0,0,0,-11,0,0,0,0,-7,0,0,0,0,-13,0,0,0,0,11,0,0,0,0,-9,0,0,0,0,0,1,0,0,
         0,0,0,-15,0,0,0,0,1,0,0,0,0,7,0,0,0,0,0,0,0,0,5,0,0,0,0,0,13,0,0,0,
         0,0,0,11,0,0,0,0,0,15,0,0,0,0,0,-9,0,0,0,0,0,0,0,-1,0,0,0,0,0,0,0,7,
         0,0,0,0,0,-15,0,0,0,0,0,15,0,0,0,0,15,0,0,0,0,15,0,0,0,0,0,1,0,0,0,0];

    static LARGEST_ED25519_S: Scalar = Scalar {
        bytes: [
            0xf8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
        ],
    };

    static CANONICAL_LARGEST_ED25519_S_PLUS_ONE: Scalar = Scalar {
        bytes: [
            0x7e, 0x34, 0x47, 0x75, 0x47, 0x4a, 0x7f, 0x97,
            0x23, 0xb6, 0x3a, 0x8b, 0xe9, 0x2a, 0xe7, 0x6d,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
        ],
    };

    static CANONICAL_LARGEST_ED25519_S_MINUS_ONE: Scalar = Scalar {
        bytes: [
            0x7c, 0x34, 0x47, 0x75, 0x47, 0x4a, 0x7f, 0x97,
            0x23, 0xb6, 0x3a, 0x8b, 0xe9, 0x2a, 0xe7, 0x6d,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
        ],
    };

    #[test]
    fn fuzzer_testcase_reduction() {
        // LE bytes of 24519928653854221733733552434404946937899825954937634815
        let a_bytes = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        // LE bytes of 4975441334397345751130612518500927154628011511324180036903450236863266160640
        let b_bytes = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 210, 210, 210, 255, 255, 255, 255, 10];
        // LE bytes of 6432735165214683820902750800207468552549813371247423777071615116673864412038
        let c_bytes = [134, 171, 119, 216, 180, 128, 178, 62, 171, 132, 32, 62, 34, 119, 104, 193, 47, 215, 181, 250, 14, 207, 172, 93, 75, 207, 211, 103, 144, 204, 56, 14];

        let a = Scalar::from_bytes_mod_order(a_bytes);
        let b = Scalar::from_bytes_mod_order(b_bytes);
        let c = Scalar::from_bytes_mod_order(c_bytes);

        let mut tmp = [0u8; 64];

        // also_a = (a mod l)
        tmp[0..32].copy_from_slice(&a_bytes[..]);
        let also_a = Scalar::from_bytes_mod_order_wide(&tmp);

        // also_b = (b mod l)
        tmp[0..32].copy_from_slice(&b_bytes[..]);
        let also_b = Scalar::from_bytes_mod_order_wide(&tmp);

        let expected_c = &a * &b;
        let also_expected_c = &also_a * &also_b;

        assert_eq!(c, expected_c);
        assert_eq!(c, also_expected_c);
    }

    #[test]
    fn non_adjacent_form_test_vector() {
        let naf = A_SCALAR.non_adjacent_form(5);
        for i in 0..256 {
            assert_eq!(naf[i], A_NAF[i]);
        }
    }

    fn non_adjacent_form_iter(w: usize, x: &Scalar) {
        let naf = x.non_adjacent_form(w);

        // Reconstruct the scalar from the computed NAF
        let mut y = Scalar::zero();
        for i in (0..256).rev() {
            y += y;
            let digit = if naf[i] < 0 {
                -Scalar::from((-naf[i]) as u64)
            } else {
                Scalar::from(naf[i] as u64)
            };
            y += digit;
        }

        assert_eq!(*x, y);
    }

    #[test]
    fn non_adjacent_form_random() {
        let mut rng = rand::thread_rng();
        for _ in 0..1_000 {
            let x = Scalar::random(&mut rng);
            for w in &[5, 6, 7, 8] {
                non_adjacent_form_iter(*w, &x);
            }
        }
    }

    #[test]
    fn from_u64() {
        let val: u64 = 0xdeadbeefdeadbeef;
        let s = Scalar::from(val);
        assert_eq!(s[7], 0xde);
        assert_eq!(s[6], 0xad);
        assert_eq!(s[5], 0xbe);
        assert_eq!(s[4], 0xef);
        assert_eq!(s[3], 0xde);
        assert_eq!(s[2], 0xad);
        assert_eq!(s[1], 0xbe);
        assert_eq!(s[0], 0xef);
    }

    #[test]
    fn scalar_mul_by_one() {
        let test_scalar = &X * &Scalar::one();
        for i in 0..32 {
            assert!(test_scalar[i] == X[i]);
        }
    }

    #[test]
    fn add_reduces() {
        // Check that the addition works
        assert_eq!(
            (LARGEST_ED25519_S + Scalar::one()).reduce(),
            CANONICAL_LARGEST_ED25519_S_PLUS_ONE
        );
        // Check that the addition reduces
        assert_eq!(
            LARGEST_ED25519_S + Scalar::one(),
            CANONICAL_LARGEST_ED25519_S_PLUS_ONE
        );
    }

    #[test]
    fn sub_reduces() {
        // Check that the subtraction works
        assert_eq!(
            (LARGEST_ED25519_S - Scalar::one()).reduce(),
            CANONICAL_LARGEST_ED25519_S_MINUS_ONE
        );
        // Check that the subtraction reduces
        assert_eq!(
            LARGEST_ED25519_S - Scalar::one(),
            CANONICAL_LARGEST_ED25519_S_MINUS_ONE
        );
    }

    #[test]
    fn quarkslab_scalar_overflow_does_not_occur() {
        // Check that manually-constructing large Scalars with
        // from_bits cannot produce incorrect results.
        //
        // The from_bits function is required to implement X/Ed25519,
        // while all other methods of constructing a Scalar produce
        // reduced Scalars.  However, this "invariant loophole" allows
        // constructing large scalars which are not reduced mod l.
        //
        // This issue was discovered independently by both Jack
        // "str4d" Grigg (issue #238), who noted that reduction was
        // not performed on addition, and Laurent Grémy & Nicolas
        // Surbayrole of Quarkslab, who noted that it was possible to
        // cause an overflow and compute incorrect results.
        //
        // This test is adapted from the one suggested by Quarkslab.

        let large_bytes = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
        ];

        let a = Scalar::from_bytes_mod_order(large_bytes);
        let b = Scalar::from_bits(large_bytes);

        assert_eq!(a, b.reduce());

        let a_3 = a + a + a;
        let b_3 = b + b + b;

        assert_eq!(a_3, b_3);

        let neg_a = -a;
        let neg_b = -b;

        assert_eq!(neg_a, neg_b);

        let minus_a_3 = Scalar::zero() - a - a - a;
        let minus_b_3 = Scalar::zero() - b - b - b;

        assert_eq!(minus_a_3, minus_b_3);
        assert_eq!(minus_a_3, -a_3);
        assert_eq!(minus_b_3, -b_3);
    }

    #[test]
    fn impl_add() {
        let two = Scalar::from(2u64);
        let one = Scalar::one();
        let should_be_two = &one + &one;
        assert_eq!(should_be_two, two);
    }

    #[allow(non_snake_case)]
    #[test]
    fn impl_mul() {
        let should_be_X_times_Y = &X * &Y;
        assert_eq!(should_be_X_times_Y, X_TIMES_Y);
    }

    #[allow(non_snake_case)]
    #[test]
    fn impl_product() {
        // Test that product works for non-empty iterators
        let X_Y_vector = vec![X, Y];
        let should_be_X_times_Y: Scalar = X_Y_vector.iter().product();
        assert_eq!(should_be_X_times_Y, X_TIMES_Y);

        // Test that product works for the empty iterator
        let one = Scalar::one();
        let empty_vector = vec![];
        let should_be_one: Scalar = empty_vector.iter().product();
        assert_eq!(should_be_one, one);

        // Test that product works for iterators where Item = Scalar
        let xs = [Scalar::from(2u64); 10];
        let ys = [Scalar::from(3u64); 10];
        // now zs is an iterator with Item = Scalar
        let zs = xs.iter().zip(ys.iter()).map(|(x,y)| x * y);

        let x_prod: Scalar = xs.iter().product();
        let y_prod: Scalar = ys.iter().product();
        let z_prod: Scalar = zs.product();

        assert_eq!(x_prod, Scalar::from(1024u64));
        assert_eq!(y_prod, Scalar::from(59049u64));
        assert_eq!(z_prod, Scalar::from(60466176u64));
        assert_eq!(x_prod * y_prod, z_prod);

    }

    #[test]
    fn impl_sum() {

        // Test that sum works for non-empty iterators
        let two = Scalar::from(2u64);
        let one_vector = vec![Scalar::one(), Scalar::one()];
        let should_be_two: Scalar = one_vector.iter().sum();
        assert_eq!(should_be_two, two);

        // Test that sum works for the empty iterator
        let zero = Scalar::zero();
        let empty_vector = vec![];
        let should_be_zero: Scalar = empty_vector.iter().sum();
        assert_eq!(should_be_zero, zero);

        // Test that sum works for owned types
        let xs = [Scalar::from(1u64); 10];
        let ys = [Scalar::from(2u64); 10];
        // now zs is an iterator with Item = Scalar
        let zs = xs.iter().zip(ys.iter()).map(|(x,y)| x + y);

        let x_sum: Scalar = xs.iter().sum();
        let y_sum: Scalar = ys.iter().sum();
        let z_sum: Scalar = zs.sum();

        assert_eq!(x_sum, Scalar::from(10u64));
        assert_eq!(y_sum, Scalar::from(20u64));
        assert_eq!(z_sum, Scalar::from(30u64));
        assert_eq!(x_sum + y_sum, z_sum);
    }

    #[test]
    fn square() {
        let expected = &X * &X;
        let actual = X.unpack().square().pack();
        for i in 0..32 {
            assert!(expected[i] == actual[i]);
        }
    }

    #[test]
    fn reduce() {
        let biggest = Scalar::from_bytes_mod_order([0xff; 32]);
        assert_eq!(biggest, CANONICAL_2_256_MINUS_1);
    }

    #[test]
    fn from_bytes_mod_order_wide() {
        let mut bignum = [0u8; 64];
        // set bignum = x + 2^256x
        for i in 0..32 {
            bignum[   i] = X[i];
            bignum[32+i] = X[i];
        }
        // 3958878930004874126169954872055634648693766179881526445624823978500314864344
        // = x + 2^256x (mod l)
        let reduced = Scalar{
            bytes: [
                216, 154, 179, 139, 210, 121,   2,  71,
                 69,  99, 158, 216,  23, 173,  63, 100,
                204,   0,  91,  50, 219, 153,  57, 249,
                 28,  82,  31, 197, 100, 165, 192,   8,
            ],
        };
        let test_red = Scalar::from_bytes_mod_order_wide(&bignum);
        for i in 0..32 {
            assert!(test_red[i] == reduced[i]);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn invert() {
        let inv_X = X.invert();
        assert_eq!(inv_X, XINV);
        let should_be_one = &inv_X * &X;
        assert_eq!(should_be_one, Scalar::one());
    }

    // Negating a scalar twice should result in the original scalar.
    #[allow(non_snake_case)]
    #[test]
    fn neg_twice_is_identity() {
        let negative_X = -&X;
        let should_be_X = -&negative_X;

        assert_eq!(should_be_X, X);
    }

    #[test]
    fn to_bytes_from_bytes_roundtrips() {
        let unpacked = X.unpack();
        let bytes = unpacked.to_bytes();
        let should_be_unpacked = UnpackedScalar::from_bytes(&bytes);

        assert_eq!(should_be_unpacked.0, unpacked.0);
    }

    #[test]
    fn montgomery_reduce_matches_from_bytes_mod_order_wide() {
        let mut bignum = [0u8; 64];

        // set bignum = x + 2^256x
        for i in 0..32 {
            bignum[   i] = X[i];
            bignum[32+i] = X[i];
        }
        // x + 2^256x (mod l)
        //         = 3958878930004874126169954872055634648693766179881526445624823978500314864344
        let expected = Scalar{
            bytes: [
                216, 154, 179, 139, 210, 121,   2,  71,
                 69,  99, 158, 216,  23, 173,  63, 100,
                204,   0,  91,  50, 219, 153,  57, 249,
                 28,  82,  31, 197, 100, 165, 192,   8
            ],
        };
        let reduced = Scalar::from_bytes_mod_order_wide(&bignum);

        // The reduced scalar should match the expected
        assert_eq!(reduced.bytes, expected.bytes);

        //  (x + 2^256x) * R
        let interim = UnpackedScalar::mul_internal(&UnpackedScalar::from_bytes_wide(&bignum),
                                                   &constants::R);
        // ((x + 2^256x) * R) / R  (mod l)
        let montgomery_reduced = UnpackedScalar::montgomery_reduce(&interim);

        // The Montgomery reduced scalar should match the reduced one, as well as the expected
        assert_eq!(montgomery_reduced.0, reduced.unpack().0);
        assert_eq!(montgomery_reduced.0, expected.unpack().0)
    }

    #[test]
    fn canonical_decoding() {
        // canonical encoding of 1667457891
        let canonical_bytes = [99, 99, 99, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,];

        // encoding of
        //   7265385991361016183439748078976496179028704920197054998554201349516117938192
        // = 28380414028753969466561515933501938171588560817147392552250411230663687203 (mod l)
        // non_canonical because unreduced mod l
        let non_canonical_bytes_because_unreduced = [16; 32];

        // encoding with high bit set, to check that the parser isn't pre-masking the high bit
        let non_canonical_bytes_because_highbit = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128];

        assert!( Scalar::from_canonical_bytes(canonical_bytes).is_some() );
        assert!( Scalar::from_canonical_bytes(non_canonical_bytes_because_unreduced).is_none() );
        assert!( Scalar::from_canonical_bytes(non_canonical_bytes_because_highbit).is_none() );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_bincode_scalar_roundtrip() {
        use bincode;
        let encoded = bincode::serialize(&X).unwrap();
        let parsed: Scalar = bincode::deserialize(&encoded).unwrap();
        assert_eq!(parsed, X);

        // Check that the encoding is 32 bytes exactly
        assert_eq!(encoded.len(), 32);

        // Check that the encoding itself matches the usual one
        assert_eq!(
            X,
            bincode::deserialize(X.as_bytes()).unwrap(),
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn batch_invert_with_a_zero_input_panics() {
        let mut xs = vec![Scalar::one(); 16];
        xs[3] = Scalar::zero();
        // This should panic in debug mode.
        Scalar::batch_invert(&mut xs);
    }

    #[test]
    fn batch_invert_empty() {
        assert_eq!(Scalar::one(), Scalar::batch_invert(&mut []));
    }

    #[test]
    fn batch_invert_consistency() {
        let mut x = Scalar::from(1u64);
        let mut v1: Vec<_> = (0..16).map(|_| {let tmp = x; x = x + x; tmp}).collect();
        let v2 = v1.clone();

        let expected: Scalar = v1.iter().product();
        let expected = expected.invert();
        let ret = Scalar::batch_invert(&mut v1);
        assert_eq!(ret, expected);

        for (a, b) in v1.iter().zip(v2.iter()) {
            assert_eq!(a * b, Scalar::one());
        }
    }

    fn test_pippenger_radix_iter(scalar: Scalar, w: usize) {
        let digits_count = Scalar::to_radix_2w_size_hint(w);
        let digits = scalar.to_radix_2w(w);

        let radix = Scalar::from((1<<w) as u64);
        let mut term = Scalar::one();
        let mut recovered_scalar = Scalar::zero();
        for digit in &digits[0..digits_count] {
            let digit = *digit;
            if digit != 0 {
                let sdigit = if digit < 0 {
                    -Scalar::from((-(digit as i64)) as u64)
                } else {
                    Scalar::from(digit as u64)
                };
                recovered_scalar += term * sdigit;
            }
            term *= radix;
        }
        // When the input is unreduced, we may only recover the scalar mod l.
        assert_eq!(recovered_scalar, scalar.reduce());
    }

    #[test]
    fn test_pippenger_radix() {
        use core::iter;
        // For each valid radix it tests that 1000 random-ish scalars can be restored
        // from the produced representation precisely.
        let cases = (2..100)
            .map(|s| Scalar::from(s as u64).invert())
            // The largest unreduced scalar, s = 2^255-1
            .chain(iter::once(Scalar::from_bits([0xff; 32])));

        for scalar in cases {
            test_pippenger_radix_iter(scalar, 6);
            test_pippenger_radix_iter(scalar, 7);
            test_pippenger_radix_iter(scalar, 8);
        }
    }
}
