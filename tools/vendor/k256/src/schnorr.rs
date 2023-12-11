//! Taproot Schnorr signatures as defined in [BIP340].
//!
//! # About
//!
//! [Schnorr signatures] are a simple group-based digital signature scheme with
//! a number of desirable properties relating to security and composability:
//!
//! - Provably secure: strongly unforgable under chosen message attack (SUF-CMA).
//! - Non-malleable: signatures cannot be altered by an attacker and still verify.
//! - Linear: multiple parties can collaborate to produce a valid signature
//!   a.k.a. multisignatures.
//!
//! Originally described in the late 1980s by their eponymous creator Claus
//! Schnorr, they were patent-encumbered and thus lingered in obscurity until
//! the [relevant patents] expired in 2010.
//!
//! Since then, Schnorr signatures have seen something of a resurgence, with
//! [EdDSA] and its concrete instantiation Ed25519 over the Curve25519 elliptic
//! curve becoming the first Schnorr variant to see mainstream standardization.
//!
//! The Taproot upgrade to Bitcoin includes a variant of Schnorr which operates
//! over the secp256k1 elliptic curve, and is specified in [BIP340].
//! That is the variant which is implemented by this crate.
//!
//! Because Taproot Schnorr is intended for use in consensus-critical
//! applications (e.g. Bitcoin), it is fully specified such that no two
//! implementations should disagree on the validity of a signature.
//!
//! # Usage
//!
#![cfg_attr(feature = "std", doc = "```")]
#![cfg_attr(not(feature = "std"), doc = "```ignore")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use k256::schnorr::{
//!     signature::{Signer, Verifier},
//!     SigningKey, VerifyingKey
//! };
//! use rand_core::OsRng; // requires 'getrandom' feature
//!
//! //
//! // Signing
//! //
//! let signing_key = SigningKey::random(&mut OsRng); // serialize with `.to_bytes()`
//! let verifying_key_bytes = signing_key.verifying_key().to_bytes(); // 32-bytes
//!
//! let message = b"Schnorr signatures prove knowledge of a secret in the random oracle model";
//! let signature = signing_key.sign(message); // returns `k256::schnorr::Signature`
//!
//! //
//! // Verification
//! //
//! let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes)?;
//! verifying_key.verify(message, &signature)?;
//! # Ok(())
//! # }
//! ```
//!
//! [Schnorr signatures]: https://en.wikipedia.org/wiki/Schnorr_signature
//! [BIP340]: https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki
//! [relevant patents]: https://patents.google.com/patent/US4995082
//! [EdDSA]: https://en.wikipedia.org/wiki/EdDSA

#![allow(non_snake_case, clippy::many_single_char_names)]

mod signing;
mod verifying;

pub use self::{signing::SigningKey, verifying::VerifyingKey};
pub use signature::{self, rand_core::CryptoRngCore, Error};

use crate::{arithmetic::FieldElement, NonZeroScalar};
use core::fmt;
use elliptic_curve::subtle::ConstantTimeEq;
use sha2::{Digest, Sha256};
use signature::Result;

const AUX_TAG: &[u8] = b"BIP0340/aux";
const NONCE_TAG: &[u8] = b"BIP0340/nonce";
const CHALLENGE_TAG: &[u8] = b"BIP0340/challenge";

/// Taproot Schnorr signature serialized as bytes.
pub type SignatureBytes = [u8; Signature::BYTE_SIZE];

/// Taproot Schnorr signature as defined in [BIP340].
///
/// [BIP340]: https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki
#[derive(Copy, Clone)]
pub struct Signature {
    r: FieldElement,
    s: NonZeroScalar,
}

impl Signature {
    /// Size of a Taproot Schnorr signature in bytes.
    pub const BYTE_SIZE: usize = 64;

    /// Serialize this signature as bytes.
    pub fn to_bytes(&self) -> SignatureBytes {
        let mut ret = [0; Self::BYTE_SIZE];
        let (r_bytes, s_bytes) = ret.split_at_mut(Self::BYTE_SIZE / 2);
        r_bytes.copy_from_slice(&self.r.to_bytes());
        s_bytes.copy_from_slice(&self.s.to_bytes());
        ret
    }

    /// Get the `r` component of this signature.
    fn r(&self) -> &FieldElement {
        &self.r
    }

    /// Get the `s` component of this signature.
    fn s(&self) -> &NonZeroScalar {
        &self.s
    }

    /// Split this signature into its `r` and `s` components.
    fn split(&self) -> (&FieldElement, &NonZeroScalar) {
        (self.r(), self.s())
    }
}

impl Eq for Signature {}

impl From<Signature> for SignatureBytes {
    fn from(signature: Signature) -> SignatureBytes {
        signature.to_bytes()
    }
}

impl From<&Signature> for SignatureBytes {
    fn from(signature: &Signature) -> SignatureBytes {
        signature.to_bytes()
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        (self.r == other.r) && (self.s.ct_eq(&other.s).into())
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Signature> {
        let (r_bytes, s_bytes) = bytes.split_at(Self::BYTE_SIZE / 2);

        let r: FieldElement =
            Option::from(FieldElement::from_bytes(r_bytes.into())).ok_or_else(Error::new)?;

        // one of the rules for valid signatures: !is_infinite(R);
        if r.is_zero().into() {
            return Err(Error::new());
        }

        let s = NonZeroScalar::try_from(s_bytes).map_err(|_| Error::new())?;

        Ok(Self { r, s })
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_bytes())
    }
}

impl signature::SignatureEncoding for Signature {
    type Repr = SignatureBytes;

    fn to_bytes(&self) -> Self::Repr {
        self.into()
    }
}

impl signature::PrehashSignature for Signature {
    type Digest = Sha256;
}

fn tagged_hash(tag: &[u8]) -> Sha256 {
    let tag_hash = Sha256::digest(tag);
    let mut digest = Sha256::new();
    digest.update(tag_hash);
    digest.update(tag_hash);
    digest
}

// Test vectors from:
// https://github.com/bitcoin/bips/blob/master/bip-0340/test-vectors.csv
#[cfg(test)]
mod tests {
    use super::{Signature, SigningKey, VerifyingKey};
    use hex_literal::hex;
    use signature::hazmat::PrehashVerifier;

    /// Signing test vector
    struct SignVector {
        /// Index of test case
        index: u8,

        /// Signing key
        secret_key: [u8; 32],

        /// Verifying key
        public_key: [u8; 32],

        /// Auxiliary randomness value
        aux_rand: [u8; 32],

        /// Message digest
        message: [u8; 32],

        /// Expected signature
        signature: [u8; 64],
    }

    /// BIP340 signing test vectors: index 0-3
    const BIP340_SIGN_VECTORS: &[SignVector] = &[
        SignVector {
            index: 0,
            secret_key: hex!("0000000000000000000000000000000000000000000000000000000000000003"),
            public_key: hex!("F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9"),
            aux_rand: hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            message: hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            signature: hex!(
                "E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA8215
                 25F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0"
            ),
        },
        SignVector {
            index: 1,
            secret_key: hex!("B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF"),
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            aux_rand: hex!("0000000000000000000000000000000000000000000000000000000000000001"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "6896BD60EEAE296DB48A229FF71DFE071BDE413E6D43F917DC8DCF8C78DE3341
                 8906D11AC976ABCCB20B091292BFF4EA897EFCB639EA871CFA95F6DE339E4B0A"
            ),
        },
        SignVector {
            index: 2,
            secret_key: hex!("C90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B14E5C9"),
            public_key: hex!("DD308AFEC5777E13121FA72B9CC1B7CC0139715309B086C960E18FD969774EB8"),
            aux_rand: hex!("C87AA53824B4D7AE2EB035A2B5BBBCCC080E76CDC6D1692C4B0B62D798E6D906"),
            message: hex!("7E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C"),
            signature: hex!(
                "5831AAEED7B44BB74E5EAB94BA9D4294C49BCF2A60728D8B4C200F50DD313C1B
                 AB745879A5AD954A72C45A91C3A51D3C7ADEA98D82F8481E0E1E03674A6F3FB7"
            ),
        },
        // test fails if msg is reduced modulo p or n
        SignVector {
            index: 3,
            secret_key: hex!("0B432B2677937381AEF05BB02A66ECD012773062CF3FA2549E44F58ED2401710"),
            public_key: hex!("25D1DFF95105F5253C4022F628A996AD3A0D95FBF21D468A1B33F8C160D8F517"),
            aux_rand: hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
            message: hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
            signature: hex!(
                "7EB0509757E246F19449885651611CB965ECC1A187DD51B64FDA1EDC9637D5EC
                 97582B9CB13DB3933705B32BA982AF5AF25FD78881EBB32771FC5922EFC66EA3"
            ),
        },
    ];

    #[test]
    fn bip340_sign_vectors() {
        for vector in BIP340_SIGN_VECTORS {
            let sk = SigningKey::from_bytes(&vector.secret_key).unwrap();
            assert_eq!(sk.verifying_key().to_bytes().as_slice(), &vector.public_key);

            let sig = sk
                .sign_prehash_with_aux_rand(&vector.message, &vector.aux_rand)
                .unwrap_or_else(|_| {
                    panic!(
                        "low-level Schnorr signing failure for index {}",
                        vector.index
                    )
                });

            assert_eq!(
                vector.signature,
                sig.to_bytes(),
                "wrong signature for index {}",
                vector.index
            );
        }
    }

    /// Verification test vector
    struct VerifyVector {
        /// Index of test case
        index: u8,

        /// Verifying key
        public_key: [u8; 32],

        /// Message digest
        message: [u8; 32],

        /// Claimed signature
        signature: [u8; 64],

        /// Is signature valid
        valid: bool,
    }

    /// BIP340 verification test vectors: index 4-14
    const BIP340_VERIFY_VECTORS: &[VerifyVector] = &[
        VerifyVector {
            index: 4,
            public_key: hex!("D69C3509BB99E412E68B0FE8544E72837DFA30746D8BE2AA65975F29D22DC7B9"),
            message: hex!("4DF3C3F68FCC83B27E9D42C90431A72499F17875C81A599B566C9889B9696703"),
            signature: hex!(
                "00000000000000000000003B78CE563F89A0ED9414F5AA28AD0D96D6795F9C63
                 76AFB1548AF603B3EB45C9F8207DEE1060CB71C04E80F593060B07D28308D7F4"
            ),
            valid: true,
        },
        // public key not on curve
        VerifyVector {
            index: 5,
            public_key: hex!("EEFDEA4CDB677750A420FEE807EACF21EB9898AE79B9768766E4FAA04A2D4A34"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E177769
                 69E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B"
            ),
            valid: false,
        },
        // has_even_y(R) is false
        VerifyVector {
            index: 6,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "FFF97BD5755EEEA420453A14355235D382F6472F8568A18B2F057A1460297556
                 3CC27944640AC607CD107AE10923D9EF7A73C643E166BE5EBEAFA34B1AC553E2"
            ),
            valid: false,
        },
        // negated message
        VerifyVector {
            index: 7,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "1FA62E331EDBC21C394792D2AB1100A7B432B013DF3F6FF4F99FCB33E0E1515F
                 28890B3EDB6E7189B630448B515CE4F8622A954CFE545735AAEA5134FCCDB2BD"
            ),
            valid: false,
        },
        // negated s value
        VerifyVector {
            index: 8,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E177769
                 961764B3AA9B2FFCB6EF947B6887A226E8D7C93E00C5ED0C1834FF0D0C2E6DA6"
            ),
            valid: false,
        },
        // sG - eP is infinite. Test fails in single verification if has_even_y(inf) is defined as true and x(inf) as 0
        VerifyVector {
            index: 9,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "0000000000000000000000000000000000000000000000000000000000000000
                 123DDA8328AF9C23A94C1FEECFD123BA4FB73476F0D594DCB65C6425BD186051"
            ),
            valid: false,
        },
        // sG - eP is infinite. Test fails in single verification if has_even_y(inf) is defined as true and x(inf) as 1
        VerifyVector {
            index: 10,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "0000000000000000000000000000000000000000000000000000000000000001
                 7615FBAF5AE28864013C099742DEADB4DBA87F11AC6754F93780D5A1837CF197"
            ),
            valid: false,
        },
        // sig[0:32] is not an X coordinate on the curve
        VerifyVector {
            index: 11,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "4A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1D
                 69E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B"
            ),
            valid: false,
        },
        // sig[0:32] is equal to field size
        VerifyVector {
            index: 12,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
                 69E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B"
            ),
            valid: false,
        },
        // sig[32:64] is equal to curve order
        VerifyVector {
            index: 13,
            public_key: hex!("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E177769
                 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"
            ),
            valid: false,
        },
        // public key is not a valid X coordinate because it exceeds the field size
        VerifyVector {
            index: 14,
            public_key: hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC30"),
            message: hex!("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89"),
            signature: hex!(
                "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E177769
                 69E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B"
            ),
            valid: false,
        },
    ];

    #[test]
    fn bip340_verify_vectors() {
        for vector in BIP340_VERIFY_VECTORS {
            let valid = match (
                VerifyingKey::from_bytes(&vector.public_key),
                Signature::try_from(vector.signature.as_slice()),
            ) {
                (Ok(pk), Ok(sig)) => pk.verify_prehash(&vector.message, &sig).is_ok(),
                _ => false,
            };

            assert_eq!(
                vector.valid, valid,
                "incorrect validation for index {}",
                vector.index
            );
        }
    }
}
