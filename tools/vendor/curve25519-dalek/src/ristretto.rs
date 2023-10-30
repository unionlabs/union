// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis lovecruft
// Copyright (c) 2016-2020 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

// We allow non snake_case names because coordinates in projective space are
// traditionally denoted by the capitalisation of their respective
// counterparts in affine space.  Yeah, you heard me, rustc, I'm gonna have my
// affine and projective cakes and eat both of them too.
#![allow(non_snake_case)]

//! An implementation of [Ristretto][ristretto_main], which provides a
//! prime-order group.
//!
//! # The Ristretto Group
//!
//! Ristretto is a modification of Mike Hamburg's Decaf scheme to work
//! with cofactor-\\(8\\) curves, such as Curve25519.
//!
//! The introduction of the Decaf paper, [_Decaf:
//! Eliminating cofactors through point
//! compression_](https://eprint.iacr.org/2015/673.pdf), notes that while
//! most cryptographic systems require a group of prime order, most
//! concrete implementations using elliptic curve groups fall short –
//! they either provide a group of prime order, but with incomplete or
//! variable-time addition formulae (for instance, most Weierstrass
//! models), or else they provide a fast and safe implementation of a
//! group whose order is not quite a prime \\(q\\), but \\(hq\\) for a
//! small cofactor \\(h\\) (for instance, Edwards curves, which have
//! cofactor at least \\(4\\)).
//!
//! This abstraction mismatch is commonly “handled” by pushing the
//! complexity upwards, adding ad-hoc protocol modifications.  But
//! these modifications require careful analysis and are a recurring
//! source of [vulnerabilities][cryptonote] and [design
//! complications][ed25519_hkd].
//!
//! Instead, Decaf (and Ristretto) use a quotient group to implement a
//! prime-order group using a non-prime-order curve.  This provides
//! the correct abstraction for cryptographic systems, while retaining
//! the speed and safety benefits of an Edwards curve.
//!
//! Decaf is named “after the procedure which divides the effect of
//! coffee by \\(4\\)”.  However, Curve25519 has a cofactor of
//! \\(8\\).  To eliminate its cofactor, Ristretto restricts further;
//! this [additional restriction][ristretto_coffee] gives the
//! _Ristretto_ encoding.
//!
//! More details on why Ristretto is necessary can be found in the
//! [Why Ristretto?][why_ristretto] section of the Ristretto website.
//!
//! Ristretto
//! points are provided in `curve25519-dalek` by the `RistrettoPoint`
//! struct.
//!
//! ## Encoding and Decoding
//!
//! Encoding is done by converting to and from a `CompressedRistretto`
//! struct, which is a typed wrapper around `[u8; 32]`.
//!
//! The encoding is not batchable, but it is possible to
//! double-and-encode in a batch using
//! `RistrettoPoint::double_and_compress_batch`.
//!
//! ## Equality Testing
//!
//! Testing equality of points on an Edwards curve in projective
//! coordinates requires an expensive inversion.  By contrast, equality
//! checking in the Ristretto group can be done in projective
//! coordinates without requiring an inversion, so it is much faster.
//!
//! The `RistrettoPoint` struct implements the
//! `subtle::ConstantTimeEq` trait for constant-time equality
//! checking, and the Rust `Eq` trait for variable-time equality
//! checking.
//!
//! ## Scalars
//!
//! Scalars are represented by the `Scalar` struct.  Each scalar has a
//! canonical representative mod the group order.  To attempt to load
//! a supposedly-canonical scalar, use
//! `Scalar::from_canonical_bytes()`. To check whether a
//! representative is canonical, use `Scalar::is_canonical()`.
//!
//! ## Scalar Multiplication
//!
//! Scalar multiplication on Ristretto points is provided by:
//!
//! * the `*` operator between a `Scalar` and a `RistrettoPoint`, which
//! performs constant-time variable-base scalar multiplication;
//!
//! * the `*` operator between a `Scalar` and a
//! `RistrettoBasepointTable`, which performs constant-time fixed-base
//! scalar multiplication;
//!
//! * an implementation of the
//! [`MultiscalarMul`](../traits/trait.MultiscalarMul.html) trait for
//! constant-time variable-base multiscalar multiplication;
//!
//! * an implementation of the
//! [`VartimeMultiscalarMul`](../traits/trait.VartimeMultiscalarMul.html)
//! trait for variable-time variable-base multiscalar multiplication;
//!
//! ## Random Points and Hashing to Ristretto
//!
//! The Ristretto group comes equipped with an Elligator map.  This is
//! used to implement
//!
//! * `RistrettoPoint::random()`, which generates random points from an
//! RNG;
//!
//! * `RistrettoPoint::from_hash()` and
//! `RistrettoPoint::hash_from_bytes()`, which perform hashing to the
//! group.
//!
//! The Elligator map itself is not currently exposed.
//!
//! ## Implementation
//!
//! The Decaf suggestion is to use a quotient group, such as \\(\mathcal
//! E / \mathcal E[4]\\) or \\(2 \mathcal E / \mathcal E[2] \\), to
//! implement a prime-order group using a non-prime-order curve.
//!
//! This requires only changing
//!
//! 1. the function for equality checking (so that two representatives
//!    of the same coset are considered equal);
//! 2. the function for encoding (so that two representatives of the
//!    same coset are encoded as identical bitstrings);
//! 3. the function for decoding (so that only the canonical encoding of
//!    a coset is accepted).
//!
//! Internally, each coset is represented by a curve point; two points
//! \\( P, Q \\) may represent the same coset in the same way that two
//! points with different \\(X,Y,Z\\) coordinates may represent the
//! same point.  The group operations are carried out with no overhead
//! using Edwards formulas.
//!
//! Notes on the details of the encoding can be found in the
//! [Details][ristretto_notes] section of the Ristretto website.
//!
//! [cryptonote]:
//! https://moderncrypto.org/mail-archive/curves/2017/000898.html
//! [ed25519_hkd]:
//! https://moderncrypto.org/mail-archive/curves/2017/000858.html
//! [ristretto_coffee]:
//! https://en.wikipedia.org/wiki/Ristretto
//! [ristretto_notes]:
//! https://ristretto.group/details/index.html
//! [why_ristretto]:
//! https://ristretto.group/why_ristretto.html
//! [ristretto_main]:
//! https://ristretto.group/

use core::borrow::Borrow;
use core::fmt::Debug;
use core::iter::Sum;
use core::ops::{Add, Neg, Sub};
use core::ops::{AddAssign, SubAssign};
use core::ops::{Mul, MulAssign};

use rand_core::{CryptoRng, RngCore};

use digest::generic_array::typenum::U64;
use digest::Digest;

use constants;
use field::FieldElement;

use subtle::Choice;
use subtle::ConditionallySelectable;
use subtle::ConditionallyNegatable;
use subtle::ConstantTimeEq;

use zeroize::Zeroize;

use edwards::EdwardsBasepointTable;
use edwards::EdwardsPoint;

#[allow(unused_imports)]
use prelude::*;

use scalar::Scalar;

use traits::Identity;
#[cfg(any(feature = "alloc", feature = "std"))]
use traits::{MultiscalarMul, VartimeMultiscalarMul, VartimePrecomputedMultiscalarMul};

#[cfg(not(all(
    feature = "simd_backend",
    any(target_feature = "avx2", target_feature = "avx512ifma")
)))]
use backend::serial::scalar_mul;
#[cfg(all(
    feature = "simd_backend",
    any(target_feature = "avx2", target_feature = "avx512ifma")
))]
use backend::vector::scalar_mul;

// ------------------------------------------------------------------------
// Compressed points
// ------------------------------------------------------------------------

/// A Ristretto point, in compressed wire format.
///
/// The Ristretto encoding is canonical, so two points are equal if and
/// only if their encodings are equal.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct CompressedRistretto(pub [u8; 32]);

impl ConstantTimeEq for CompressedRistretto {
    fn ct_eq(&self, other: &CompressedRistretto) -> Choice {
        self.as_bytes().ct_eq(other.as_bytes())
    }
}

impl CompressedRistretto {
    /// Copy the bytes of this `CompressedRistretto`.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// View this `CompressedRistretto` as an array of bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Construct a `CompressedRistretto` from a slice of bytes.
    ///
    /// # Panics
    ///
    /// If the input `bytes` slice does not have a length of 32.
    pub fn from_slice(bytes: &[u8]) -> CompressedRistretto {
        let mut tmp = [0u8; 32];

        tmp.copy_from_slice(bytes);

        CompressedRistretto(tmp)
    }

    /// Attempt to decompress to an `RistrettoPoint`.
    ///
    /// # Return
    ///
    /// - `Some(RistrettoPoint)` if `self` was the canonical encoding of a point;
    ///
    /// - `None` if `self` was not the canonical encoding of a point.
    pub fn decompress(&self) -> Option<RistrettoPoint> {
        // Step 1. Check s for validity:
        // 1.a) s must be 32 bytes (we get this from the type system)
        // 1.b) s < p
        // 1.c) s is nonnegative
        //
        // Our decoding routine ignores the high bit, so the only
        // possible failure for 1.b) is if someone encodes s in 0..18
        // as s+p in 2^255-19..2^255-1.  We can check this by
        // converting back to bytes, and checking that we get the
        // original input, since our encoding routine is canonical.

        let s = FieldElement::from_bytes(self.as_bytes());
        let s_bytes_check = s.to_bytes();
        let s_encoding_is_canonical =
            &s_bytes_check[..].ct_eq(self.as_bytes());
        let s_is_negative = s.is_negative();

        if s_encoding_is_canonical.unwrap_u8() == 0u8 || s_is_negative.unwrap_u8() == 1u8 {
            return None;
        }

        // Step 2.  Compute (X:Y:Z:T).
        let one = FieldElement::one();
        let ss = s.square();
        let u1 = &one - &ss;      //  1 + as²
        let u2 = &one + &ss;      //  1 - as²    where a=-1
        let u2_sqr = u2.square(); // (1 - as²)²

        // v == ad(1+as²)² - (1-as²)²            where d=-121665/121666
        let v = &(&(-&constants::EDWARDS_D) * &u1.square()) - &u2_sqr;

        let (ok, I) = (&v * &u2_sqr).invsqrt(); // 1/sqrt(v*u_2²)

        let Dx = &I * &u2;         // 1/sqrt(v)
        let Dy = &I * &(&Dx * &v); // 1/u2

        // x == | 2s/sqrt(v) | == + sqrt(4s²/(ad(1+as²)² - (1-as²)²))
        let mut x = &(&s + &s) * &Dx;
        let x_neg = x.is_negative();
        x.conditional_negate(x_neg);

        // y == (1-as²)/(1+as²)
        let y = &u1 * &Dy;

        // t == ((1+as²) sqrt(4s²/(ad(1+as²)² - (1-as²)²)))/(1-as²)
        let t = &x * &y;

        if ok.unwrap_u8() == 0u8 || t.is_negative().unwrap_u8() == 1u8 || y.is_zero().unwrap_u8() == 1u8 {
            None
        } else {
            Some(RistrettoPoint(EdwardsPoint{X: x, Y: y, Z: one, T: t}))
        }
    }
}

impl Identity for CompressedRistretto {
    fn identity() -> CompressedRistretto {
        CompressedRistretto([0u8; 32])
    }
}

impl Default for CompressedRistretto {
    fn default() -> CompressedRistretto {
        CompressedRistretto::identity()
    }
}

// ------------------------------------------------------------------------
// Serde support
// ------------------------------------------------------------------------
// Serializes to and from `RistrettoPoint` directly, doing compression
// and decompression internally.  This means that users can create
// structs containing `RistrettoPoint`s and use Serde's derived
// serializers to serialize those structures.

#[cfg(feature = "serde")]
use serde::{self, Serialize, Deserialize, Serializer, Deserializer};
#[cfg(feature = "serde")]
use serde::de::Visitor;

#[cfg(feature = "serde")]
impl Serialize for RistrettoPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(32)?;
        for byte in self.compress().as_bytes().iter() {
            tup.serialize_element(byte)?;
        }
        tup.end()
    }
}

#[cfg(feature = "serde")]
impl Serialize for CompressedRistretto {
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
impl<'de> Deserialize<'de> for RistrettoPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct RistrettoPointVisitor;

        impl<'de> Visitor<'de> for RistrettoPointVisitor {
            type Value = RistrettoPoint;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("a valid point in Ristretto format")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<RistrettoPoint, A::Error>
                where A: serde::de::SeqAccess<'de>
            {
                let mut bytes = [0u8; 32];
                for i in 0..32 {
                    bytes[i] = seq.next_element()?
                        .ok_or(serde::de::Error::invalid_length(i, &"expected 32 bytes"))?;
                }
                CompressedRistretto(bytes)
                    .decompress()
                    .ok_or(serde::de::Error::custom("decompression failed"))
            }
        }

        deserializer.deserialize_tuple(32, RistrettoPointVisitor)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CompressedRistretto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct CompressedRistrettoVisitor;

        impl<'de> Visitor<'de> for CompressedRistrettoVisitor {
            type Value = CompressedRistretto;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("32 bytes of data")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<CompressedRistretto, A::Error>
                where A: serde::de::SeqAccess<'de>
            {
                let mut bytes = [0u8; 32];
                for i in 0..32 {
                    bytes[i] = seq.next_element()?
                        .ok_or(serde::de::Error::invalid_length(i, &"expected 32 bytes"))?;
                }
                Ok(CompressedRistretto(bytes))
            }
        }

        deserializer.deserialize_tuple(32, CompressedRistrettoVisitor)
    }
}

// ------------------------------------------------------------------------
// Internal point representations
// ------------------------------------------------------------------------

/// A `RistrettoPoint` represents a point in the Ristretto group for
/// Curve25519.  Ristretto, a variant of Decaf, constructs a
/// prime-order group as a quotient group of a subgroup of (the
/// Edwards form of) Curve25519.
///
/// Internally, a `RistrettoPoint` is implemented as a wrapper type
/// around `EdwardsPoint`, with custom equality, compression, and
/// decompression routines to account for the quotient.  This means that
/// operations on `RistrettoPoint`s are exactly as fast as operations on
/// `EdwardsPoint`s.
///
#[derive(Copy, Clone)]
pub struct RistrettoPoint(pub(crate) EdwardsPoint);

impl RistrettoPoint {
    /// Compress this point using the Ristretto encoding.
    pub fn compress(&self) -> CompressedRistretto {
        let mut X = self.0.X;
        let mut Y = self.0.Y;
        let Z = &self.0.Z;
        let T = &self.0.T;

        let u1 = &(Z + &Y) * &(Z - &Y);
        let u2 = &X * &Y;
        // Ignore return value since this is always square
        let (_, invsqrt) = (&u1 * &u2.square()).invsqrt();
        let i1 = &invsqrt * &u1;
        let i2 = &invsqrt * &u2;
        let z_inv = &i1 * &(&i2 * T);
        let mut den_inv = i2;

        let iX = &X * &constants::SQRT_M1;
        let iY = &Y * &constants::SQRT_M1;
        let ristretto_magic = &constants::INVSQRT_A_MINUS_D;
        let enchanted_denominator = &i1 * ristretto_magic;

        let rotate = (T * &z_inv).is_negative();

        X.conditional_assign(&iY, rotate);
        Y.conditional_assign(&iX, rotate);
        den_inv.conditional_assign(&enchanted_denominator, rotate);

        Y.conditional_negate((&X * &z_inv).is_negative());

        let mut s = &den_inv * &(Z - &Y);
        let s_is_negative = s.is_negative();
        s.conditional_negate(s_is_negative);

        CompressedRistretto(s.to_bytes())
    }

    /// Double-and-compress a batch of points.  The Ristretto encoding
    /// is not batchable, since it requires an inverse square root.
    ///
    /// However, given input points \\( P\_1, \ldots, P\_n, \\)
    /// it is possible to compute the encodings of their doubles \\(
    /// \mathrm{enc}( [2]P\_1), \ldots, \mathrm{enc}( [2]P\_n ) \\)
    /// in a batch.
    ///
    /// ```
    /// # extern crate curve25519_dalek;
    /// # use curve25519_dalek::ristretto::RistrettoPoint;
    /// extern crate rand_core;
    /// use rand_core::OsRng;
    ///
    /// # // Need fn main() here in comment so the doctest compiles
    /// # // See https://doc.rust-lang.org/book/documentation.html#documentation-as-tests
    /// # fn main() {
    /// let mut rng = OsRng;
    /// let points: Vec<RistrettoPoint> =
    ///     (0..32).map(|_| RistrettoPoint::random(&mut rng)).collect();
    ///
    /// let compressed = RistrettoPoint::double_and_compress_batch(&points);
    ///
    /// for (P, P2_compressed) in points.iter().zip(compressed.iter()) {
    ///     assert_eq!(*P2_compressed, (P + P).compress());
    /// }
    /// # }
    /// ```
    #[cfg(feature = "alloc")]
    pub fn double_and_compress_batch<'a, I>(points: I) -> Vec<CompressedRistretto>
        where I: IntoIterator<Item = &'a RistrettoPoint>
    {
        #[derive(Copy, Clone, Debug)]
        struct BatchCompressState {
            e: FieldElement,
            f: FieldElement,
            g: FieldElement,
            h: FieldElement,
            eg: FieldElement,
            fh: FieldElement,
        }

        impl BatchCompressState {
            fn efgh(&self) -> FieldElement {
                &self.eg * &self.fh
            }
        }

        impl<'a> From<&'a RistrettoPoint> for BatchCompressState {
            fn from(P: &'a RistrettoPoint) -> BatchCompressState {
                let XX = P.0.X.square();
                let YY = P.0.Y.square();
                let ZZ = P.0.Z.square();
                let dTT = &P.0.T.square() * &constants::EDWARDS_D;

                let e = &P.0.X * &(&P.0.Y + &P.0.Y); // = 2*X*Y
                let f = &ZZ + &dTT;                  // = Z^2 + d*T^2
                let g = &YY + &XX;                   // = Y^2 - a*X^2
                let h = &ZZ - &dTT;                  // = Z^2 - d*T^2

                let eg = &e * &g;
                let fh = &f * &h;

                BatchCompressState{ e, f, g, h, eg, fh }
            }
        }

        let states: Vec<BatchCompressState> = points.into_iter().map(BatchCompressState::from).collect();

        let mut invs: Vec<FieldElement> = states.iter().map(|state| state.efgh()).collect();

        FieldElement::batch_invert(&mut invs[..]);

        states.iter().zip(invs.iter()).map(|(state, inv): (&BatchCompressState, &FieldElement)| {
            let Zinv = &state.eg * &inv;
            let Tinv = &state.fh * &inv;

            let mut magic = constants::INVSQRT_A_MINUS_D;

            let negcheck1 = (&state.eg * &Zinv).is_negative();

            let mut e = state.e;
            let mut g = state.g;
            let mut h = state.h;

            let minus_e = -&e;
            let f_times_sqrta = &state.f * &constants::SQRT_M1;

            e.conditional_assign(&state.g,       negcheck1);
            g.conditional_assign(&minus_e,       negcheck1);
            h.conditional_assign(&f_times_sqrta, negcheck1);

            magic.conditional_assign(&constants::SQRT_M1, negcheck1);

            let negcheck2 = (&(&h * &e) * &Zinv).is_negative();

            g.conditional_negate(negcheck2);

            let mut s = &(&h - &g) * &(&magic * &(&g * &Tinv));

            let s_is_negative = s.is_negative();
            s.conditional_negate(s_is_negative);

            CompressedRistretto(s.to_bytes())
        }).collect()
    }


    /// Return the coset self + E[4], for debugging.
    fn coset4(&self) -> [EdwardsPoint; 4] {
        [  self.0
        , &self.0 + &constants::EIGHT_TORSION[2]
        , &self.0 + &constants::EIGHT_TORSION[4]
        , &self.0 + &constants::EIGHT_TORSION[6]
        ]
    }

    /// Computes the Ristretto Elligator map.
    ///
    /// # Note
    ///
    /// This method is not public because it's just used for hashing
    /// to a point -- proper elligator support is deferred for now.
    pub(crate) fn elligator_ristretto_flavor(r_0: &FieldElement) -> RistrettoPoint {
        let i = &constants::SQRT_M1;
        let d = &constants::EDWARDS_D;
        let one_minus_d_sq = &constants::ONE_MINUS_EDWARDS_D_SQUARED;
        let d_minus_one_sq = &constants::EDWARDS_D_MINUS_ONE_SQUARED;
        let mut c = constants::MINUS_ONE;

        let one = FieldElement::one();

        let r = i * &r_0.square();
        let N_s = &(&r + &one) * &one_minus_d_sq;
        let D = &(&c - &(d * &r)) * &(&r + d);

        let (Ns_D_is_sq, mut s) = FieldElement::sqrt_ratio_i(&N_s, &D);
        let mut s_prime = &s * r_0;
        let s_prime_is_pos = !s_prime.is_negative();
        s_prime.conditional_negate(s_prime_is_pos);

        s.conditional_assign(&s_prime, !Ns_D_is_sq);
        c.conditional_assign(&r, !Ns_D_is_sq);

        let N_t = &(&(&c * &(&r - &one)) * &d_minus_one_sq) - &D;
        let s_sq = s.square();

        use backend::serial::curve_models::CompletedPoint;

        // The conversion from W_i is exactly the conversion from P1xP1.
        RistrettoPoint(CompletedPoint{
            X: &(&s + &s) * &D,
            Z: &N_t * &constants::SQRT_AD_MINUS_ONE,
            Y: &FieldElement::one() - &s_sq,
            T: &FieldElement::one() + &s_sq,
        }.to_extended())
    }

    /// Return a `RistrettoPoint` chosen uniformly at random using a user-provided RNG.
    ///
    /// # Inputs
    ///
    /// * `rng`: any RNG which implements the `RngCore + CryptoRng` interface.
    ///
    /// # Returns
    ///
    /// A random element of the Ristretto group.
    ///
    /// # Implementation
    ///
    /// Uses the Ristretto-flavoured Elligator 2 map, so that the
    /// discrete log of the output point with respect to any other
    /// point should be unknown.  The map is applied twice and the
    /// results are added, to ensure a uniform distribution.
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut uniform_bytes = [0u8; 64];
        rng.fill_bytes(&mut uniform_bytes);

        RistrettoPoint::from_uniform_bytes(&uniform_bytes)
    }

    /// Hash a slice of bytes into a `RistrettoPoint`.
    ///
    /// Takes a type parameter `D`, which is any `Digest` producing 64
    /// bytes of output.
    ///
    /// Convenience wrapper around `from_hash`.
    ///
    /// # Implementation
    ///
    /// Uses the Ristretto-flavoured Elligator 2 map, so that the
    /// discrete log of the output point with respect to any other
    /// point should be unknown.  The map is applied twice and the
    /// results are added, to ensure a uniform distribution.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate curve25519_dalek;
    /// # use curve25519_dalek::ristretto::RistrettoPoint;
    /// extern crate sha2;
    /// use sha2::Sha512;
    ///
    /// # // Need fn main() here in comment so the doctest compiles
    /// # // See https://doc.rust-lang.org/book/documentation.html#documentation-as-tests
    /// # fn main() {
    /// let msg = "To really appreciate architecture, you may even need to commit a murder";
    /// let P = RistrettoPoint::hash_from_bytes::<Sha512>(msg.as_bytes());
    /// # }
    /// ```
    ///
    pub fn hash_from_bytes<D>(input: &[u8]) -> RistrettoPoint
        where D: Digest<OutputSize = U64> + Default
    {
        let mut hash = D::default();
        hash.update(input);
        RistrettoPoint::from_hash(hash)
    }

    /// Construct a `RistrettoPoint` from an existing `Digest` instance.
    ///
    /// Use this instead of `hash_from_bytes` if it is more convenient
    /// to stream data into the `Digest` than to pass a single byte
    /// slice.
    pub fn from_hash<D>(hash: D) -> RistrettoPoint
        where D: Digest<OutputSize = U64> + Default
    {
        // dealing with generic arrays is clumsy, until const generics land
        let output = hash.finalize();
        let mut output_bytes = [0u8; 64];
        output_bytes.copy_from_slice(&output.as_slice());

        RistrettoPoint::from_uniform_bytes(&output_bytes)
    }

    /// Construct a `RistrettoPoint` from 64 bytes of data.
    ///
    /// If the input bytes are uniformly distributed, the resulting
    /// point will be uniformly distributed over the group, and its
    /// discrete log with respect to other points should be unknown.
    ///
    /// # Implementation
    ///
    /// This function splits the input array into two 32-byte halves,
    /// takes the low 255 bits of each half mod p, applies the
    /// Ristretto-flavored Elligator map to each, and adds the results.
    pub fn from_uniform_bytes(bytes: &[u8; 64]) -> RistrettoPoint {
        let mut r_1_bytes = [0u8; 32];
        r_1_bytes.copy_from_slice(&bytes[0..32]);
        let r_1 = FieldElement::from_bytes(&r_1_bytes);
        let R_1 = RistrettoPoint::elligator_ristretto_flavor(&r_1);

        let mut r_2_bytes = [0u8; 32];
        r_2_bytes.copy_from_slice(&bytes[32..64]);
        let r_2 = FieldElement::from_bytes(&r_2_bytes);
        let R_2 = RistrettoPoint::elligator_ristretto_flavor(&r_2);

        // Applying Elligator twice and adding the results ensures a
        // uniform distribution.
        &R_1 + &R_2
    }
}

impl Identity for RistrettoPoint {
    fn identity() -> RistrettoPoint {
        RistrettoPoint(EdwardsPoint::identity())
    }
}

impl Default for RistrettoPoint {
    fn default() -> RistrettoPoint {
        RistrettoPoint::identity()
    }
}

// ------------------------------------------------------------------------
// Equality
// ------------------------------------------------------------------------

impl PartialEq for RistrettoPoint {
    fn eq(&self, other: &RistrettoPoint) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}

impl ConstantTimeEq for RistrettoPoint {
    /// Test equality between two `RistrettoPoint`s.
    ///
    /// # Returns
    ///
    /// * `Choice(1)` if the two `RistrettoPoint`s are equal;
    /// * `Choice(0)` otherwise.
    fn ct_eq(&self, other: &RistrettoPoint) -> Choice {
        let X1Y2 = &self.0.X * &other.0.Y;
        let Y1X2 = &self.0.Y * &other.0.X;
        let X1X2 = &self.0.X * &other.0.X;
        let Y1Y2 = &self.0.Y * &other.0.Y;

        X1Y2.ct_eq(&Y1X2) | X1X2.ct_eq(&Y1Y2)
    }
}

impl Eq for RistrettoPoint {}

// ------------------------------------------------------------------------
// Arithmetic
// ------------------------------------------------------------------------

impl<'a, 'b> Add<&'b RistrettoPoint> for &'a RistrettoPoint {
    type Output = RistrettoPoint;

    fn add(self, other: &'b RistrettoPoint) -> RistrettoPoint {
        RistrettoPoint(&self.0 + &other.0)
    }
}

define_add_variants!(LHS = RistrettoPoint, RHS = RistrettoPoint, Output = RistrettoPoint);

impl<'b> AddAssign<&'b RistrettoPoint> for RistrettoPoint {
    fn add_assign(&mut self, _rhs: &RistrettoPoint) {
        *self = (self as &RistrettoPoint) + _rhs;
    }
}

define_add_assign_variants!(LHS = RistrettoPoint, RHS = RistrettoPoint);

impl<'a, 'b> Sub<&'b RistrettoPoint> for &'a RistrettoPoint {
    type Output = RistrettoPoint;

    fn sub(self, other: &'b RistrettoPoint) -> RistrettoPoint {
        RistrettoPoint(&self.0 - &other.0)
    }
}

define_sub_variants!(LHS = RistrettoPoint, RHS = RistrettoPoint, Output = RistrettoPoint);

impl<'b> SubAssign<&'b RistrettoPoint> for RistrettoPoint {
    fn sub_assign(&mut self, _rhs: &RistrettoPoint) {
        *self = (self as &RistrettoPoint) - _rhs;
    }
}

define_sub_assign_variants!(LHS = RistrettoPoint, RHS = RistrettoPoint);

impl<T> Sum<T> for RistrettoPoint
where
    T: Borrow<RistrettoPoint>
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>
    {
        iter.fold(RistrettoPoint::identity(), |acc, item| acc + item.borrow())
    }
}

impl<'a> Neg for &'a RistrettoPoint {
    type Output = RistrettoPoint;

    fn neg(self) -> RistrettoPoint {
        RistrettoPoint(-&self.0)
    }
}

impl Neg for RistrettoPoint {
    type Output = RistrettoPoint;

    fn neg(self) -> RistrettoPoint {
        -&self
    }
}

impl<'b> MulAssign<&'b Scalar> for RistrettoPoint {
    fn mul_assign(&mut self, scalar: &'b Scalar) {
        let result = (self as &RistrettoPoint) * scalar;
        *self = result;
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a RistrettoPoint {
    type Output = RistrettoPoint;
    /// Scalar multiplication: compute `scalar * self`.
    fn mul(self, scalar: &'b Scalar) -> RistrettoPoint {
        RistrettoPoint(self.0 * scalar)
    }
}

impl<'a, 'b> Mul<&'b RistrettoPoint> for &'a Scalar {
    type Output = RistrettoPoint;

    /// Scalar multiplication: compute `self * scalar`.
    fn mul(self, point: &'b RistrettoPoint) -> RistrettoPoint {
        RistrettoPoint(self * point.0)
    }
}

define_mul_assign_variants!(LHS = RistrettoPoint, RHS = Scalar);

define_mul_variants!(LHS = RistrettoPoint, RHS = Scalar, Output = RistrettoPoint);
define_mul_variants!(LHS = Scalar, RHS = RistrettoPoint, Output = RistrettoPoint);

// ------------------------------------------------------------------------
// Multiscalar Multiplication impls
// ------------------------------------------------------------------------

// These use iterator combinators to unwrap the underlying points and
// forward to the EdwardsPoint implementations.

#[cfg(feature = "alloc")]
impl MultiscalarMul for RistrettoPoint {
    type Point = RistrettoPoint;

    fn multiscalar_mul<I, J>(scalars: I, points: J) -> RistrettoPoint
    where
        I: IntoIterator,
        I::Item: Borrow<Scalar>,
        J: IntoIterator,
        J::Item: Borrow<RistrettoPoint>,
    {
        let extended_points = points.into_iter().map(|P| P.borrow().0);
        RistrettoPoint(
            EdwardsPoint::multiscalar_mul(scalars, extended_points)
        )
    }
}

#[cfg(feature = "alloc")]
impl VartimeMultiscalarMul for RistrettoPoint {
    type Point = RistrettoPoint;

    fn optional_multiscalar_mul<I, J>(scalars: I, points: J) -> Option<RistrettoPoint>
    where
        I: IntoIterator,
        I::Item: Borrow<Scalar>,
        J: IntoIterator<Item = Option<RistrettoPoint>>,
    {
        let extended_points = points.into_iter().map(|opt_P| opt_P.map(|P| P.borrow().0));

        EdwardsPoint::optional_multiscalar_mul(scalars, extended_points).map(RistrettoPoint)
    }
}

/// Precomputation for variable-time multiscalar multiplication with `RistrettoPoint`s.
// This wraps the inner implementation in a facade type so that we can
// decouple stability of the inner type from the stability of the
// outer type.
#[cfg(feature = "alloc")]
pub struct VartimeRistrettoPrecomputation(scalar_mul::precomputed_straus::VartimePrecomputedStraus);

#[cfg(feature = "alloc")]
impl VartimePrecomputedMultiscalarMul for VartimeRistrettoPrecomputation {
    type Point = RistrettoPoint;

    fn new<I>(static_points: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<Self::Point>,
    {
        Self(
            scalar_mul::precomputed_straus::VartimePrecomputedStraus::new(
                static_points.into_iter().map(|P| P.borrow().0),
            ),
        )
    }

    fn optional_mixed_multiscalar_mul<I, J, K>(
        &self,
        static_scalars: I,
        dynamic_scalars: J,
        dynamic_points: K,
    ) -> Option<Self::Point>
    where
        I: IntoIterator,
        I::Item: Borrow<Scalar>,
        J: IntoIterator,
        J::Item: Borrow<Scalar>,
        K: IntoIterator<Item = Option<Self::Point>>,
    {
        self.0
            .optional_mixed_multiscalar_mul(
                static_scalars,
                dynamic_scalars,
                dynamic_points.into_iter().map(|P_opt| P_opt.map(|P| P.0)),
            )
            .map(RistrettoPoint)
    }
}

impl RistrettoPoint {
    /// Compute \\(aA + bB\\) in variable time, where \\(B\\) is the
    /// Ristretto basepoint.
    pub fn vartime_double_scalar_mul_basepoint(
        a: &Scalar,
        A: &RistrettoPoint,
        b: &Scalar,
    ) -> RistrettoPoint {
        RistrettoPoint(
            EdwardsPoint::vartime_double_scalar_mul_basepoint(a, &A.0, b)
        )
    }
}

/// A precomputed table of multiples of a basepoint, used to accelerate
/// scalar multiplication.
///
/// A precomputed table of multiples of the Ristretto basepoint is
/// available in the `constants` module:
/// ```
/// use curve25519_dalek::constants;
/// use curve25519_dalek::scalar::Scalar;
///
/// let a = Scalar::from(87329482u64);
/// let P = &a * &constants::RISTRETTO_BASEPOINT_TABLE;
/// ```
#[derive(Clone)]
pub struct RistrettoBasepointTable(pub(crate) EdwardsBasepointTable);

impl<'a, 'b> Mul<&'b Scalar> for &'a RistrettoBasepointTable {
    type Output = RistrettoPoint;

    fn mul(self, scalar: &'b Scalar) -> RistrettoPoint {
        RistrettoPoint(&self.0 * scalar)
    }
}

impl<'a, 'b> Mul<&'a RistrettoBasepointTable> for &'b Scalar {
    type Output = RistrettoPoint;

    fn mul(self, basepoint_table: &'a RistrettoBasepointTable) -> RistrettoPoint {
        RistrettoPoint(self * &basepoint_table.0)
    }
}

impl RistrettoBasepointTable {
    /// Create a precomputed table of multiples of the given `basepoint`.
    pub fn create(basepoint: &RistrettoPoint) -> RistrettoBasepointTable {
        RistrettoBasepointTable(EdwardsBasepointTable::create(&basepoint.0))
    }

    /// Get the basepoint for this table as a `RistrettoPoint`.
    pub fn basepoint(&self) -> RistrettoPoint {
        RistrettoPoint(self.0.basepoint())
    }
}

// ------------------------------------------------------------------------
// Constant-time conditional selection
// ------------------------------------------------------------------------

impl ConditionallySelectable for RistrettoPoint {
    /// Conditionally select between `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate subtle;
    /// # extern crate curve25519_dalek;
    /// #
    /// use subtle::ConditionallySelectable;
    /// use subtle::Choice;
    /// #
    /// # use curve25519_dalek::traits::Identity;
    /// # use curve25519_dalek::ristretto::RistrettoPoint;
    /// # use curve25519_dalek::constants;
    /// # fn main() {
    ///
    /// let A = RistrettoPoint::identity();
    /// let B = constants::RISTRETTO_BASEPOINT_POINT;
    ///
    /// let mut P = A;
    ///
    /// P = RistrettoPoint::conditional_select(&A, &B, Choice::from(0));
    /// assert_eq!(P, A);
    /// P = RistrettoPoint::conditional_select(&A, &B, Choice::from(1));
    /// assert_eq!(P, B);
    /// # }
    /// ```
    fn conditional_select(
        a: &RistrettoPoint,
        b: &RistrettoPoint,
        choice: Choice,
    ) -> RistrettoPoint {
        RistrettoPoint(EdwardsPoint::conditional_select(&a.0, &b.0, choice))
    }
}

// ------------------------------------------------------------------------
// Debug traits
// ------------------------------------------------------------------------

impl Debug for CompressedRistretto {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "CompressedRistretto: {:?}", self.as_bytes())
    }
}

impl Debug for RistrettoPoint {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let coset = self.coset4();
        write!(f, "RistrettoPoint: coset \n{:?}\n{:?}\n{:?}\n{:?}",
               coset[0], coset[1], coset[2], coset[3])
    }
}

// ------------------------------------------------------------------------
// Zeroize traits
// ------------------------------------------------------------------------

impl Zeroize for CompressedRistretto {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Zeroize for RistrettoPoint {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

// ------------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use rand_core::OsRng;

    use scalar::Scalar;
    use constants;
    use edwards::CompressedEdwardsY;
    use traits::{Identity};
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn serde_bincode_basepoint_roundtrip() {
        use bincode;

        let encoded = bincode::serialize(&constants::RISTRETTO_BASEPOINT_POINT).unwrap();
        let enc_compressed = bincode::serialize(&constants::RISTRETTO_BASEPOINT_COMPRESSED).unwrap();
        assert_eq!(encoded, enc_compressed);

        // Check that the encoding is 32 bytes exactly
        assert_eq!(encoded.len(), 32);

        let dec_uncompressed: RistrettoPoint = bincode::deserialize(&encoded).unwrap();
        let dec_compressed: CompressedRistretto = bincode::deserialize(&encoded).unwrap();

        assert_eq!(dec_uncompressed, constants::RISTRETTO_BASEPOINT_POINT);
        assert_eq!(dec_compressed, constants::RISTRETTO_BASEPOINT_COMPRESSED);

        // Check that the encoding itself matches the usual one
        let raw_bytes = constants::RISTRETTO_BASEPOINT_COMPRESSED.as_bytes();
        let bp: RistrettoPoint = bincode::deserialize(raw_bytes).unwrap();
        assert_eq!(bp, constants::RISTRETTO_BASEPOINT_POINT);
    }

    #[test]
    fn scalarmult_ristrettopoint_works_both_ways() {
        let P = constants::RISTRETTO_BASEPOINT_POINT;
        let s = Scalar::from(999u64);

        let P1 = &P * &s;
        let P2 = &s * &P;

        assert!(P1.compress().as_bytes() == P2.compress().as_bytes());
    }

    #[test]
    fn impl_sum() {

        // Test that sum works for non-empty iterators
        let BASE = constants::RISTRETTO_BASEPOINT_POINT;

        let s1 = Scalar::from(999u64);
        let P1 = &BASE * &s1;

        let s2 = Scalar::from(333u64);
        let P2 = &BASE * &s2;

        let vec = vec![P1.clone(), P2.clone()];
        let sum: RistrettoPoint = vec.iter().sum();

        assert_eq!(sum, P1 + P2);

        // Test that sum works for the empty iterator
        let empty_vector: Vec<RistrettoPoint> = vec![];
        let sum: RistrettoPoint = empty_vector.iter().sum();

        assert_eq!(sum, RistrettoPoint::identity());

        // Test that sum works on owning iterators
        let s = Scalar::from(2u64);
        let mapped = vec.iter().map(|x| x * s);
        let sum: RistrettoPoint = mapped.sum();

        assert_eq!(sum, &P1 * &s + &P2 * &s);
    }

    #[test]
    fn decompress_negative_s_fails() {
        // constants::d is neg, so decompression should fail as |d| != d.
        let bad_compressed = CompressedRistretto(constants::EDWARDS_D.to_bytes());
        assert!(bad_compressed.decompress().is_none());
    }

    #[test]
    fn decompress_id() {
        let compressed_id = CompressedRistretto::identity();
        let id = compressed_id.decompress().unwrap();
        let mut identity_in_coset = false;
        for P in &id.coset4() {
            if P.compress() == CompressedEdwardsY::identity() {
                identity_in_coset = true;
            }
        }
        assert!(identity_in_coset);
    }

    #[test]
    fn compress_id() {
        let id = RistrettoPoint::identity();
        assert_eq!(id.compress(), CompressedRistretto::identity());
    }

    #[test]
    fn basepoint_roundtrip() {
        let bp_compressed_ristretto = constants::RISTRETTO_BASEPOINT_POINT.compress();
        let bp_recaf = bp_compressed_ristretto.decompress().unwrap().0;
        // Check that bp_recaf differs from bp by a point of order 4
        let diff = &constants::RISTRETTO_BASEPOINT_POINT.0 - &bp_recaf;
        let diff4 = diff.mul_by_pow_2(2);
        assert_eq!(diff4.compress(), CompressedEdwardsY::identity());
    }

    #[test]
    fn encodings_of_small_multiples_of_basepoint() {
        // Table of encodings of i*basepoint
        // Generated using ristretto.sage
        let compressed = [
            CompressedRistretto([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            CompressedRistretto([226, 242, 174, 10, 106, 188, 78, 113, 168, 132, 169, 97, 197, 0, 81, 95, 88, 227, 11, 106, 165, 130, 221, 141, 182, 166, 89, 69, 224, 141, 45, 118]),
            CompressedRistretto([106, 73, 50, 16, 247, 73, 156, 209, 127, 236, 181, 16, 174, 12, 234, 35, 161, 16, 232, 213, 185, 1, 248, 172, 173, 211, 9, 92, 115, 163, 185, 25]),
            CompressedRistretto([148, 116, 31, 93, 93, 82, 117, 94, 206, 79, 35, 240, 68, 238, 39, 213, 209, 234, 30, 43, 209, 150, 180, 98, 22, 107, 22, 21, 42, 157, 2, 89]),
            CompressedRistretto([218, 128, 134, 39, 115, 53, 139, 70, 111, 250, 223, 224, 179, 41, 58, 179, 217, 253, 83, 197, 234, 108, 149, 83, 88, 245, 104, 50, 45, 175, 106, 87]),
            CompressedRistretto([232, 130, 177, 49, 1, 107, 82, 193, 211, 51, 112, 128, 24, 124, 247, 104, 66, 62, 252, 203, 181, 23, 187, 73, 90, 184, 18, 196, 22, 15, 244, 78]),
            CompressedRistretto([246, 71, 70, 211, 201, 43, 19, 5, 14, 216, 216, 2, 54, 167, 240, 0, 124, 59, 63, 150, 47, 91, 167, 147, 209, 154, 96, 30, 187, 29, 244, 3]),
            CompressedRistretto([68, 245, 53, 32, 146, 110, 200, 31, 189, 90, 56, 120, 69, 190, 183, 223, 133, 169, 106, 36, 236, 225, 135, 56, 189, 207, 166, 167, 130, 42, 23, 109]),
            CompressedRistretto([144, 50, 147, 216, 242, 40, 126, 190, 16, 226, 55, 77, 193, 165, 62, 11, 200, 135, 229, 146, 105, 159, 2, 208, 119, 213, 38, 60, 221, 85, 96, 28]),
            CompressedRistretto([2, 98, 42, 206, 143, 115, 3, 163, 28, 175, 198, 63, 143, 196, 143, 220, 22, 225, 200, 200, 210, 52, 178, 240, 214, 104, 82, 130, 169, 7, 96, 49]),
            CompressedRistretto([32, 112, 111, 215, 136, 178, 114, 10, 30, 210, 165, 218, 212, 149, 43, 1, 244, 19, 188, 240, 231, 86, 77, 232, 205, 200, 22, 104, 158, 45, 185, 95]),
            CompressedRistretto([188, 232, 63, 139, 165, 221, 47, 165, 114, 134, 76, 36, 186, 24, 16, 249, 82, 43, 198, 0, 74, 254, 149, 135, 122, 199, 50, 65, 202, 253, 171, 66]),
            CompressedRistretto([228, 84, 158, 225, 107, 154, 160, 48, 153, 202, 32, 140, 103, 173, 175, 202, 250, 76, 63, 62, 78, 83, 3, 222, 96, 38, 227, 202, 143, 248, 68, 96]),
            CompressedRistretto([170, 82, 224, 0, 223, 46, 22, 245, 95, 177, 3, 47, 195, 59, 196, 39, 66, 218, 214, 189, 90, 143, 192, 190, 1, 103, 67, 108, 89, 72, 80, 31]),
            CompressedRistretto([70, 55, 107, 128, 244, 9, 178, 157, 194, 181, 246, 240, 197, 37, 145, 153, 8, 150, 229, 113, 111, 65, 71, 124, 211, 0, 133, 171, 127, 16, 48, 30]),
            CompressedRistretto([224, 196, 24, 247, 200, 217, 196, 205, 215, 57, 91, 147, 234, 18, 79, 58, 217, 144, 33, 187, 104, 29, 252, 51, 2, 169, 217, 154, 46, 83, 230, 78]),
        ];
        let mut bp = RistrettoPoint::identity();
        for i in 0..16 {
            assert_eq!(bp.compress(), compressed[i]);
            bp = &bp + &constants::RISTRETTO_BASEPOINT_POINT;
        }
    }

    #[test]
    fn four_torsion_basepoint() {
        let bp = constants::RISTRETTO_BASEPOINT_POINT;
        let bp_coset = bp.coset4();
        for i in 0..4 {
            assert_eq!(bp, RistrettoPoint(bp_coset[i]));
        }
    }

    #[test]
    fn four_torsion_random() {
        let mut rng = OsRng;
        let B = &constants::RISTRETTO_BASEPOINT_TABLE;
        let P = B * &Scalar::random(&mut rng);
        let P_coset = P.coset4();
        for i in 0..4 {
            assert_eq!(P, RistrettoPoint(P_coset[i]));
        }
    }

    #[test]
    fn elligator_vs_ristretto_sage() {
        // Test vectors extracted from ristretto.sage.
        //
        // Notice that all of the byte sequences have bit 255 set to 0; this is because
        // ristretto.sage does not mask the high bit of a field element.  When the high bit is set,
        // the ristretto.sage elligator implementation gives different results, since it takes a
        // different field element as input.
        let bytes: [[u8;32]; 16] = [
            [184, 249, 135, 49, 253, 123, 89, 113, 67, 160, 6, 239, 7, 105, 211, 41, 192, 249, 185, 57, 9, 102, 70, 198, 15, 127, 7, 26, 160, 102, 134, 71],
            [229, 14, 241, 227, 75, 9, 118, 60, 128, 153, 226, 21, 183, 217, 91, 136, 98, 0, 231, 156, 124, 77, 82, 139, 142, 134, 164, 169, 169, 62, 250, 52],
            [115, 109, 36, 220, 180, 223, 99, 6, 204, 169, 19, 29, 169, 68, 84, 23, 21, 109, 189, 149, 127, 205, 91, 102, 172, 35, 112, 35, 134, 69, 186, 34],
            [16, 49, 96, 107, 171, 199, 164, 9, 129, 16, 64, 62, 241, 63, 132, 173, 209, 160, 112, 215, 105, 50, 157, 81, 253, 105, 1, 154, 229, 25, 120, 83],
            [156, 131, 161, 162, 236, 251, 5, 187, 167, 171, 17, 178, 148, 210, 90, 207, 86, 21, 79, 161, 167, 215, 234, 1, 136, 242, 182, 248, 38, 85, 79, 86],
            [251, 177, 124, 54, 18, 101, 75, 235, 245, 186, 19, 46, 133, 157, 229, 64, 10, 136, 181, 185, 78, 144, 254, 167, 137, 49, 107, 10, 61, 10, 21, 25],
            [232, 193, 20, 68, 240, 77, 186, 77, 183, 40, 44, 86, 150, 31, 198, 212, 76, 81, 3, 217, 197, 8, 126, 128, 126, 152, 164, 208, 153, 44, 189, 77],
            [173, 229, 149, 177, 37, 230, 30, 69, 61, 56, 172, 190, 219, 115, 167, 194, 71, 134, 59, 75, 28, 244, 118, 26, 162, 97, 64, 16, 15, 189, 30, 64],
            [106, 71, 61, 107, 250, 117, 42, 151, 91, 202, 212, 100, 52, 188, 190, 21, 125, 218, 31, 18, 253, 241, 160, 133, 57, 242, 3, 164, 189, 68, 111, 75],
            [112, 204, 182, 90, 220, 198, 120, 73, 173, 107, 193, 17, 227, 40, 162, 36, 150, 141, 235, 55, 172, 183, 12, 39, 194, 136, 43, 153, 244, 118, 91, 89],
            [111, 24, 203, 123, 254, 189, 11, 162, 51, 196, 163, 136, 204, 143, 10, 222, 33, 112, 81, 205, 34, 35, 8, 66, 90, 6, 164, 58, 170, 177, 34, 25],
            [225, 183, 30, 52, 236, 82, 6, 183, 109, 25, 227, 181, 25, 82, 41, 193, 80, 77, 161, 80, 242, 203, 79, 204, 136, 245, 131, 110, 237, 106, 3, 58],
            [207, 246, 38, 56, 30, 86, 176, 90, 27, 200, 61, 42, 221, 27, 56, 210, 79, 178, 189, 120, 68, 193, 120, 167, 77, 185, 53, 197, 124, 128, 191, 126],
            [1, 136, 215, 80, 240, 46, 63, 147, 16, 244, 230, 207, 82, 189, 74, 50, 106, 169, 138, 86, 30, 131, 214, 202, 166, 125, 251, 228, 98, 24, 36, 21],
            [210, 207, 228, 56, 155, 116, 207, 54, 84, 195, 251, 215, 249, 199, 116, 75, 109, 239, 196, 251, 194, 246, 252, 228, 70, 146, 156, 35, 25, 39, 241, 4],
            [34, 116, 123, 9, 8, 40, 93, 189, 9, 103, 57, 103, 66, 227, 3, 2, 157, 107, 134, 219, 202, 74, 230, 154, 78, 107, 219, 195, 214, 14, 84, 80],
        ];
        let encoded_images: [CompressedRistretto; 16] = [
            CompressedRistretto([176, 157, 237, 97, 66, 29, 140, 166, 168, 94, 26, 157, 212, 216, 229, 160, 195, 246, 232, 239, 169, 112, 63, 193, 64, 32, 152, 69, 11, 190, 246, 86]),
            CompressedRistretto([234, 141, 77, 203, 181, 225, 250, 74, 171, 62, 15, 118, 78, 212, 150, 19, 131, 14, 188, 238, 194, 244, 141, 138, 166, 162, 83, 122, 228, 201, 19, 26]),
            CompressedRistretto([232, 231, 51, 92, 5, 168, 80, 36, 173, 179, 104, 68, 186, 149, 68, 40, 140, 170, 27, 103, 99, 140, 21, 242, 43, 62, 250, 134, 208, 255, 61, 89]),
            CompressedRistretto([208, 120, 140, 129, 177, 179, 237, 159, 252, 160, 28, 13, 206, 5, 211, 241, 192, 218, 1, 97, 130, 241, 20, 169, 119, 46, 246, 29, 79, 80, 77, 84]),
            CompressedRistretto([202, 11, 236, 145, 58, 12, 181, 157, 209, 6, 213, 88, 75, 147, 11, 119, 191, 139, 47, 142, 33, 36, 153, 193, 223, 183, 178, 8, 205, 120, 248, 110]),
            CompressedRistretto([26, 66, 231, 67, 203, 175, 116, 130, 32, 136, 62, 253, 215, 46, 5, 214, 166, 248, 108, 237, 216, 71, 244, 173, 72, 133, 82, 6, 143, 240, 104, 41]),
            CompressedRistretto([40, 157, 102, 96, 201, 223, 200, 197, 150, 181, 106, 83, 103, 126, 143, 33, 145, 230, 78, 6, 171, 146, 210, 143, 112, 5, 245, 23, 183, 138, 18, 120]),
            CompressedRistretto([220, 37, 27, 203, 239, 196, 176, 131, 37, 66, 188, 243, 185, 250, 113, 23, 167, 211, 154, 243, 168, 215, 54, 171, 159, 36, 195, 81, 13, 150, 43, 43]),
            CompressedRistretto([232, 121, 176, 222, 183, 196, 159, 90, 238, 193, 105, 52, 101, 167, 244, 170, 121, 114, 196, 6, 67, 152, 80, 185, 221, 7, 83, 105, 176, 208, 224, 121]),
            CompressedRistretto([226, 181, 183, 52, 241, 163, 61, 179, 221, 207, 220, 73, 245, 242, 25, 236, 67, 84, 179, 222, 167, 62, 167, 182, 32, 9, 92, 30, 165, 127, 204, 68]),
            CompressedRistretto([226, 119, 16, 242, 200, 139, 240, 87, 11, 222, 92, 146, 156, 243, 46, 119, 65, 59, 1, 248, 92, 183, 50, 175, 87, 40, 206, 53, 208, 220, 148, 13]),
            CompressedRistretto([70, 240, 79, 112, 54, 157, 228, 146, 74, 122, 216, 88, 232, 62, 158, 13, 14, 146, 115, 117, 176, 222, 90, 225, 244, 23, 94, 190, 150, 7, 136, 96]),
            CompressedRistretto([22, 71, 241, 103, 45, 193, 195, 144, 183, 101, 154, 50, 39, 68, 49, 110, 51, 44, 62, 0, 229, 113, 72, 81, 168, 29, 73, 106, 102, 40, 132, 24]),
            CompressedRistretto([196, 133, 107, 11, 130, 105, 74, 33, 204, 171, 133, 221, 174, 193, 241, 36, 38, 179, 196, 107, 219, 185, 181, 253, 228, 47, 155, 42, 231, 73, 41, 78]),
            CompressedRistretto([58, 255, 225, 197, 115, 208, 160, 143, 39, 197, 82, 69, 143, 235, 92, 170, 74, 40, 57, 11, 171, 227, 26, 185, 217, 207, 90, 185, 197, 190, 35, 60]),
            CompressedRistretto([88, 43, 92, 118, 223, 136, 105, 145, 238, 186, 115, 8, 214, 112, 153, 253, 38, 108, 205, 230, 157, 130, 11, 66, 101, 85, 253, 110, 110, 14, 148, 112]),
        ];
        for i in 0..16 {
            let r_0 = FieldElement::from_bytes(&bytes[i]);
            let Q = RistrettoPoint::elligator_ristretto_flavor(&r_0);
            assert_eq!(Q.compress(), encoded_images[i]);
        }
    }

    #[test]
    fn random_roundtrip() {
        let mut rng = OsRng;
        let B = &constants::RISTRETTO_BASEPOINT_TABLE;
        for _ in 0..100 {
            let P = B * &Scalar::random(&mut rng);
            let compressed_P = P.compress();
            let Q = compressed_P.decompress().unwrap();
            assert_eq!(P, Q);
        }
    }

    #[test]
    fn double_and_compress_1024_random_points() {
        let mut rng = OsRng;

        let points: Vec<RistrettoPoint> =
            (0..1024).map(|_| RistrettoPoint::random(&mut rng)).collect();

        let compressed = RistrettoPoint::double_and_compress_batch(&points);

        for (P, P2_compressed) in points.iter().zip(compressed.iter()) {
            assert_eq!(*P2_compressed, (P + P).compress());
        }
    }

    #[test]
    fn vartime_precomputed_vs_nonprecomputed_multiscalar() {
        let mut rng = rand::thread_rng();

        let B = &::constants::RISTRETTO_BASEPOINT_TABLE;

        let static_scalars = (0..128)
            .map(|_| Scalar::random(&mut rng))
            .collect::<Vec<_>>();

        let dynamic_scalars = (0..128)
            .map(|_| Scalar::random(&mut rng))
            .collect::<Vec<_>>();

        let check_scalar: Scalar = static_scalars
            .iter()
            .chain(dynamic_scalars.iter())
            .map(|s| s * s)
            .sum();

        let static_points = static_scalars.iter().map(|s| s * B).collect::<Vec<_>>();
        let dynamic_points = dynamic_scalars.iter().map(|s| s * B).collect::<Vec<_>>();

        let precomputation = VartimeRistrettoPrecomputation::new(static_points.iter());

        let P = precomputation.vartime_mixed_multiscalar_mul(
            &static_scalars,
            &dynamic_scalars,
            &dynamic_points,
        );

        use traits::VartimeMultiscalarMul;
        let Q = RistrettoPoint::vartime_multiscalar_mul(
            static_scalars.iter().chain(dynamic_scalars.iter()),
            static_points.iter().chain(dynamic_points.iter()),
        );

        let R = &check_scalar * B;

        assert_eq!(P.compress(), R.compress());
        assert_eq!(Q.compress(), R.compress());
    }
}
