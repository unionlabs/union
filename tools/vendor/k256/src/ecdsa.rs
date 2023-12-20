//! Elliptic Curve Digital Signature Algorithm (ECDSA).
//!
//! This module contains support for computing and verifying ECDSA signatures.
//! To use it, you will need to enable one of the two following Cargo features:
//!
//! - `ecdsa-core`: provides only the [`Signature`] type (which represents an
//!   ECDSA/secp256k1 signature). Does not require the `arithmetic` feature.
//!   This is useful for 3rd-party crates which wish to use the `Signature`
//!   type for interoperability purposes (particularly in conjunction with the
//!   [`signature::Signer`] trait). Example use cases for this include other
//!   software implementations of ECDSA/secp256k1 and wrappers for cloud KMS
//!   services or hardware devices (HSM or crypto hardware wallet).
//! - `ecdsa`: provides `ecdsa-core` features plus the [`SigningKey`] and
//!   [`VerifyingKey`] types which natively implement ECDSA/secp256k1 signing and
//!   verification.
//!
//! Most users of this library who want to sign/verify signatures will want to
//! enable the `ecdsa` and `sha256` Cargo features.
//!
//! ## Signing and Verifying Signatures
//!
//! This example requires the `ecdsa` and `sha256` Cargo features are enabled:
//!
//! ```
//! # #[cfg(all(feature = "ecdsa", feature = "sha256"))]
//! # {
//! use k256::{
//!     ecdsa::{SigningKey, Signature, signature::Signer},
//!     SecretKey,
//! };
//! use rand_core::OsRng; // requires 'getrandom' feature
//!
//! // Signing
//! let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
//! let message = b"ECDSA proves knowledge of a secret number in the context of a single message";
//!
//! // Note: The signature type must be annotated or otherwise inferable as
//! // `Signer` has many impls of the `Signer` trait (for both regular and
//! // recoverable signature types).
//! let signature: Signature = signing_key.sign(message);
//!
//! // Verification
//! use k256::{EncodedPoint, ecdsa::{VerifyingKey, signature::Verifier}};
//!
//! let verifying_key = VerifyingKey::from(&signing_key); // Serialize with `::to_encoded_point()`
//! assert!(verifying_key.verify(message, &signature).is_ok());
//! # }
//! ```
//!
//! ## Recovering [`VerifyingKey`] from [`Signature`]
//!
//! ECDSA makes it possible to recover the public key used to verify a
//! signature with the assistance of 2-bits of additional information.
//!
//! This is helpful when there is already a trust relationship for a particular
//! key, and it's desirable to omit the full public key used to sign a
//! particular message.
//!
//! One common application of signature recovery with secp256k1 is Ethereum.
//!
//! ### Upgrading recoverable signature code from earlier versions of `k256`
//!
//! The v0.12 release of `k256` contains a brand new recoverable signature API
//! from previous releases. Functionality has been upstreamed from `k256` to a
//! generic implementation in the [`ecdsa`](`ecdsa_core`) crate.
//!
//! If you previously used `k256::ecdsa::recoverable::Signature`, the old
//! functionality now uses a "detached" [`Signature`] and [`RecoveryId`].
//! Here is where the various functionality went:
//!
//! - Signing now requires the use of the [`hazmat::SignPrimitive`] trait
//!   (see examples immediately below).
//! - Signature recovery is now implemented as methods of the [`VerifyingKey`]
//!   type (i.e. `::recover_from_*`).
//! - Trial recovery is now defined on the [`RecoveryId`] type
//!   (i.e. `::trial_recovery_from_*`).
//!
//! ### Computing a signature with a [`RecoveryId`].
//!
//! This example shows how to compute a signature and its associated
//! [`RecoveryId`] in a manner which is byte-for-byte compatible with
//! Ethereum libraries, leveraging the [`SigningKey::sign_digest_recoverable`]
//! API:
//!
#![cfg_attr(feature = "std", doc = "```")]
#![cfg_attr(not(feature = "std"), doc = "```ignore")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hex_literal::hex;
//! use k256::ecdsa::{hazmat::SignPrimitive, RecoveryId, Signature, SigningKey};
//! use sha2::Sha256;
//! use sha3::{Keccak256, Digest};
//!
//! let signing_key = SigningKey::from_bytes(&hex!(
//!     "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
//! ).into())?;
//!
//! let msg = hex!("e9808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080018080");
//! let digest = Keccak256::new_with_prefix(msg);
//! let (signature, recid) = signing_key.sign_digest_recoverable(digest)?;
//!
//! assert_eq!(
//!     signature.to_bytes().as_slice(),
//!     &hex!("c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa68")
//! );
//!
//! assert_eq!(recid, RecoveryId::try_from(0u8).unwrap());
//! # Ok(())
//! # }
//! ```
//!
//! ### Recovering a [`VerifyingKey`] from a signature
//!
#![cfg_attr(feature = "std", doc = "```")]
#![cfg_attr(not(feature = "std"), doc = "```ignore")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hex_literal::hex;
//! use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
//! use sha3::{Keccak256, Digest};
//! use elliptic_curve::sec1::ToEncodedPoint;
//!
//! let msg = b"example message";
//!
//! let signature = Signature::try_from(hex!(
//!     "46c05b6368a44b8810d79859441d819b8e7cdc8bfd371e35c53196f4bcacdb51
//!      35c7facce2a97b95eacba8a586d87b7958aaf8368ab29cee481f76e871dbd9cb"
//! ).as_slice())?;
//!
//! let recid = RecoveryId::try_from(1u8)?;
//!
//! let recovered_key = VerifyingKey::recover_from_digest(
//!     Keccak256::new_with_prefix(msg),
//!     &signature,
//!     recid
//! )?;
//!
//! let expected_key = VerifyingKey::from_sec1_bytes(
//!     &hex!("0200866db99873b09fc2fb1e3ba549b156e96d1a567e3284f5f0e859a83320cb8b")
//! )?;
//!
//! assert_eq!(recovered_key, expected_key);
//! # Ok(())
//! # }
//! ```

pub use ecdsa_core::{
    signature::{self, Error},
    RecoveryId,
};

#[cfg(any(feature = "ecdsa", feature = "sha256"))]
pub use ecdsa_core::hazmat;

use crate::Secp256k1;

#[cfg(feature = "ecdsa")]
use {
    crate::{AffinePoint, FieldBytes, ProjectivePoint, Scalar},
    ecdsa_core::hazmat::{SignPrimitive, VerifyPrimitive},
    elliptic_curve::{
        bigint::U256,
        ops::{Invert, MulByGenerator, Reduce},
        scalar::IsHigh,
        subtle::CtOption,
    },
};

/// ECDSA/secp256k1 signature (fixed-size)
pub type Signature = ecdsa_core::Signature<Secp256k1>;

/// ECDSA/secp256k1 signature (ASN.1 DER encoded)
pub type DerSignature = ecdsa_core::der::Signature<Secp256k1>;

/// ECDSA/secp256k1 signing key
#[cfg(feature = "ecdsa")]
pub type SigningKey = ecdsa_core::SigningKey<Secp256k1>;

/// ECDSA/secp256k1 verification key (i.e. public key)
#[cfg(feature = "ecdsa")]
pub type VerifyingKey = ecdsa_core::VerifyingKey<Secp256k1>;

#[cfg(feature = "sha256")]
impl hazmat::DigestPrimitive for Secp256k1 {
    type Digest = sha2::Sha256;
}

#[cfg(feature = "ecdsa")]
impl SignPrimitive<Secp256k1> for Scalar {
    #[allow(non_snake_case, clippy::many_single_char_names)]
    fn try_sign_prehashed<K>(
        &self,
        k: K,
        z: &FieldBytes,
    ) -> Result<(Signature, Option<RecoveryId>), Error>
    where
        K: AsRef<Self> + Invert<Output = CtOption<Self>>,
    {
        if k.as_ref().is_zero().into() {
            return Err(Error::new());
        }

        let z = <Self as Reduce<U256>>::reduce_bytes(z);

        // Compute scalar inversion of 𝑘
        let k_inv = Option::<Scalar>::from(k.invert()).ok_or_else(Error::new)?;

        // Compute 𝑹 = 𝑘×𝑮
        let R = ProjectivePoint::mul_by_generator(k.as_ref()).to_affine();

        // Lift x-coordinate of 𝑹 (element of base field) into a serialized big
        // integer, then reduce it into an element of the scalar field
        let r = <Self as Reduce<U256>>::reduce_bytes(&R.x.to_bytes());

        // Compute 𝒔 as a signature over 𝒓 and 𝒛.
        let s = k_inv * (z + (r * self));

        if s.is_zero().into() {
            return Err(Error::new());
        }

        let signature = Signature::from_scalars(r, s)?;
        let is_r_odd = R.y.normalize().is_odd();
        let is_s_high = s.is_high();
        let is_y_odd = is_r_odd ^ is_s_high;
        let signature_low = signature.normalize_s().unwrap_or(signature);
        let recovery_id = RecoveryId::new(is_y_odd.into(), false);

        Ok((signature_low, Some(recovery_id)))
    }
}

#[cfg(feature = "ecdsa")]
impl VerifyPrimitive<Secp256k1> for AffinePoint {}

#[cfg(all(test, feature = "ecdsa", feature = "arithmetic"))]
mod tests {
    mod normalize {
        use crate::ecdsa::Signature;

        // Test vectors generated using rust-secp256k1
        #[test]
        #[rustfmt::skip]
        fn s_high() {
            let sig_hi = Signature::try_from([
                0x20, 0xc0, 0x1a, 0x91, 0x0e, 0xbb, 0x26, 0x10,
                0xaf, 0x2d, 0x76, 0x3f, 0xa0, 0x9b, 0x3b, 0x30,
                0x92, 0x3c, 0x8e, 0x40, 0x8b, 0x11, 0xdf, 0x2c,
                0x61, 0xad, 0x76, 0xd9, 0x70, 0xa2, 0xf1, 0xbc,
                0xee, 0x2f, 0x11, 0xef, 0x8c, 0xb0, 0x0a, 0x49,
                0x61, 0x7d, 0x13, 0x57, 0xf4, 0xd5, 0x56, 0x41,
                0x09, 0x0a, 0x48, 0xf2, 0x01, 0xe9, 0xb9, 0x59,
                0xc4, 0x8f, 0x6f, 0x6b, 0xec, 0x6f, 0x93, 0x8f,
            ].as_slice()).unwrap();

            let sig_lo = Signature::try_from([
                0x20, 0xc0, 0x1a, 0x91, 0x0e, 0xbb, 0x26, 0x10,
                0xaf, 0x2d, 0x76, 0x3f, 0xa0, 0x9b, 0x3b, 0x30,
                0x92, 0x3c, 0x8e, 0x40, 0x8b, 0x11, 0xdf, 0x2c,
                0x61, 0xad, 0x76, 0xd9, 0x70, 0xa2, 0xf1, 0xbc,
                0x11, 0xd0, 0xee, 0x10, 0x73, 0x4f, 0xf5, 0xb6,
                0x9e, 0x82, 0xec, 0xa8, 0x0b, 0x2a, 0xa9, 0xbd,
                0xb1, 0xa4, 0x93, 0xf4, 0xad, 0x5e, 0xe6, 0xe1,
                0xfb, 0x42, 0xef, 0x20, 0xe3, 0xc6, 0xad, 0xb2,
            ].as_slice()).unwrap();

            let sig_normalized = sig_hi.normalize_s().unwrap();
            assert_eq!(sig_lo, sig_normalized);
        }

        #[test]
        fn s_low() {
            #[rustfmt::skip]
            let sig = Signature::try_from([
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ].as_slice()).unwrap();

            assert_eq!(sig.normalize_s(), None);
        }
    }

    #[cfg(feature = "sha256")]
    mod recovery {
        use crate::{
            ecdsa::{signature::DigestVerifier, RecoveryId, Signature, SigningKey, VerifyingKey},
            EncodedPoint,
        };
        use hex_literal::hex;
        use sha2::{Digest, Sha256};
        use sha3::Keccak256;

        /// Signature recovery test vectors
        struct RecoveryTestVector {
            pk: [u8; 33],
            msg: &'static [u8],
            sig: [u8; 64],
            recid: RecoveryId,
        }

        const RECOVERY_TEST_VECTORS: &[RecoveryTestVector] = &[
            // Recovery ID 0
            RecoveryTestVector {
                pk: hex!("021a7a569e91dbf60581509c7fc946d1003b60c7dee85299538db6353538d59574"),
                msg: b"example message",
                sig: hex!(
                    "ce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa032
                     3971edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d52"
                ),
                recid: RecoveryId::new(false, false),
            },
            // Recovery ID 1
            RecoveryTestVector {
                pk: hex!("036d6caac248af96f6afa7f904f550253a0f3ef3f5aa2fe6838a95b216691468e2"),
                msg: b"example message",
                sig: hex!(
                    "46c05b6368a44b8810d79859441d819b8e7cdc8bfd371e35c53196f4bcacdb51
                     35c7facce2a97b95eacba8a586d87b7958aaf8368ab29cee481f76e871dbd9cb"
                ),
                recid: RecoveryId::new(true, false),
            },
        ];

        #[test]
        fn public_key_recovery() {
            for vector in RECOVERY_TEST_VECTORS {
                let digest = Sha256::new_with_prefix(vector.msg);
                let sig = Signature::try_from(vector.sig.as_slice()).unwrap();
                let recid = vector.recid;
                let pk = VerifyingKey::recover_from_digest(digest, &sig, recid).unwrap();
                assert_eq!(&vector.pk[..], EncodedPoint::from(&pk).as_bytes());
            }
        }

        /// End-to-end example which ensures RFC6979 is implemented in the same
        /// way as other Ethereum libraries, using HMAC-DRBG-SHA-256 for RFC6979,
        /// and Keccak256 for hashing the message.
        ///
        /// Test vectors adapted from:
        /// <https://github.com/gakonst/ethers-rs/blob/ba00f549/ethers-signers/src/wallet/private_key.rs#L197>
        #[test]
        fn ethereum_end_to_end_example() {
            let signing_key = SigningKey::from_bytes(
                &hex!("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318").into(),
            )
            .unwrap();

            let msg = hex!(
                "e9808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080018080"
            );
            let digest = Keccak256::new_with_prefix(msg);

            let (sig, recid) = signing_key.sign_digest_recoverable(digest.clone()).unwrap();
            assert_eq!(
                sig.to_bytes().as_slice(),
                &hex!("c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa68")
            );
            assert_eq!(recid, RecoveryId::from_byte(0).unwrap());

            let verifying_key =
                VerifyingKey::recover_from_digest(digest.clone(), &sig, recid).unwrap();

            assert_eq!(signing_key.verifying_key(), &verifying_key);
            assert!(verifying_key.verify_digest(digest, &sig).is_ok());
        }
    }

    mod wycheproof {
        use crate::{EncodedPoint, Secp256k1};
        use ecdsa_core::{signature::Verifier, Signature};
        use elliptic_curve::generic_array::typenum::Unsigned;

        #[test]
        fn wycheproof() {
            use blobby::Blob5Iterator;

            // Build a field element but allow for too-short input (left pad with zeros)
            // or too-long input (check excess leftmost bytes are zeros).
            fn element_from_padded_slice<C: elliptic_curve::Curve>(
                data: &[u8],
            ) -> elliptic_curve::FieldBytes<C> {
                let point_len = C::FieldBytesSize::USIZE;
                if data.len() >= point_len {
                    let offset = data.len() - point_len;
                    for v in data.iter().take(offset) {
                        assert_eq!(*v, 0, "EcdsaVerifier: point too large");
                    }
                    elliptic_curve::FieldBytes::<C>::clone_from_slice(&data[offset..])
                } else {
                    let iter = core::iter::repeat(0)
                        .take(point_len - data.len())
                        .chain(data.iter().cloned());
                    elliptic_curve::FieldBytes::<C>::from_exact_iter(iter).unwrap()
                }
            }

            fn run_test(
                wx: &[u8],
                wy: &[u8],
                msg: &[u8],
                sig: &[u8],
                pass: bool,
            ) -> Option<&'static str> {
                let x = element_from_padded_slice::<Secp256k1>(wx);
                let y = element_from_padded_slice::<Secp256k1>(wy);
                let q_encoded =
                    EncodedPoint::from_affine_coordinates(&x, &y, /* compress= */ false);
                let verifying_key =
                    ecdsa_core::VerifyingKey::from_encoded_point(&q_encoded).unwrap();

                let sig = match Signature::<Secp256k1>::from_der(sig) {
                    Ok(s) => s.normalize_s().unwrap_or(s),
                    Err(_) if !pass => return None,
                    Err(_) => return Some("failed to parse signature ASN.1"),
                };

                match verifying_key.verify(msg, &sig) {
                    Ok(_) if pass => None,
                    Ok(_) => Some("signature verify unexpectedly succeeded"),
                    Err(_) if !pass => None,
                    Err(_) => Some("signature verify failed"),
                }
            }

            let data = include_bytes!(concat!("test_vectors/data/", "wycheproof", ".blb"));

            for (i, row) in Blob5Iterator::new(data).unwrap().enumerate() {
                let [wx, wy, msg, sig, status] = row.unwrap();
                let pass = match status[0] {
                    0 => false,
                    1 => true,
                    _ => panic!("invalid value for pass flag"),
                };
                if let Some(desc) = run_test(wx, wy, msg, sig, pass) {
                    panic!(
                        "\n\
                                 Failed test №{}: {}\n\
                                 wx:\t{:?}\n\
                                 wy:\t{:?}\n\
                                 msg:\t{:?}\n\
                                 sig:\t{:?}\n\
                                 pass:\t{}\n",
                        i, desc, wx, wy, msg, sig, pass,
                    );
                }
            }
        }
    }
}
