// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Verification of RSA signatures.

use super::{parse_public_key, RsaParameters, N, PUBLIC_KEY_PUBLIC_MODULUS_MAX_LEN};
use crate::{
    arithmetic::{bigint, montgomery::Unencoded},
    bits, cpu, digest, error,
    limb::LIMB_BYTES,
    sealed, signature,
};

#[derive(Debug)]
pub struct Key {
    pub n: bigint::Modulus<N>,
    pub e: bigint::PublicExponent,
    pub n_bits: bits::BitLength,
}

impl Key {
    pub fn from_modulus_and_exponent(
        n: untrusted::Input,
        e: untrusted::Input,
        n_min_bits: bits::BitLength,
        n_max_bits: bits::BitLength,
        e_min_value: u64,
    ) -> Result<Self, error::KeyRejected> {
        // This is an incomplete implementation of NIST SP800-56Br1 Section
        // 6.4.2.2, "Partial Public-Key Validation for RSA." That spec defers
        // to NIST SP800-89 Section 5.3.3, "(Explicit) Partial Public Key
        // Validation for RSA," "with the caveat that the length of the modulus
        // shall be a length that is specified in this Recommendation." In
        // SP800-89, two different sets of steps are given, one set numbered,
        // and one set lettered. TODO: Document this in the end-user
        // documentation for RSA keys.

        // Step 3 / Step c for `n` (out of order).
        let (n, n_bits) = bigint::Modulus::from_be_bytes_with_bit_length(n)?;

        // `pkcs1_encode` depends on this not being small. Otherwise,
        // `pkcs1_encode` would generate padding that is invalid (too few 0xFF
        // bytes) for very small keys.
        const N_MIN_BITS: bits::BitLength = bits::BitLength::from_usize_bits(1024);

        // Step 1 / Step a. XXX: SP800-56Br1 and SP800-89 require the length of
        // the public modulus to be exactly 2048 or 3072 bits, but we are more
        // flexible to be compatible with other commonly-used crypto libraries.
        assert!(n_min_bits >= N_MIN_BITS);
        let n_bits_rounded_up =
            bits::BitLength::from_usize_bytes(n_bits.as_usize_bytes_rounded_up())
                .map_err(|error::Unspecified| error::KeyRejected::unexpected_error())?;
        if n_bits_rounded_up < n_min_bits {
            return Err(error::KeyRejected::too_small());
        }
        if n_bits > n_max_bits {
            return Err(error::KeyRejected::too_large());
        }

        // Step 2 / Step b.
        // Step 3 / Step c for `e`.
        let e = bigint::PublicExponent::from_be_bytes(e, e_min_value)?;

        // If `n` is less than `e` then somebody has probably accidentally swapped
        // them. The largest acceptable `e` is smaller than the smallest acceptable
        // `n`, so no additional checks need to be done.

        // XXX: Steps 4 & 5 / Steps d, e, & f are not implemented. This is also the
        // case in most other commonly-used crypto libraries.

        Ok(Self { n, e, n_bits })
    }
}

impl signature::VerificationAlgorithm for RsaParameters {
    fn verify(
        &self,
        public_key: untrusted::Input,
        msg: untrusted::Input,
        signature: untrusted::Input,
    ) -> Result<(), error::Unspecified> {
        let (n, e) = parse_public_key(public_key)?;
        verify_rsa_(
            self,
            (
                n.big_endian_without_leading_zero_as_input(),
                e.big_endian_without_leading_zero_as_input(),
            ),
            msg,
            signature,
        )
    }
}

impl sealed::Sealed for RsaParameters {}

macro_rules! rsa_params {
    ( $VERIFY_ALGORITHM:ident, $min_bits:expr, $PADDING_ALGORITHM:expr,
      $doc_str:expr ) => {
        #[doc=$doc_str]
        ///
        /// Only available in `alloc` mode.
        pub static $VERIFY_ALGORITHM: RsaParameters = RsaParameters {
            padding_alg: $PADDING_ALGORITHM,
            min_bits: bits::BitLength::from_usize_bits($min_bits),
        };
    };
}

rsa_params!(
    RSA_PKCS1_1024_8192_SHA1_FOR_LEGACY_USE_ONLY,
    1024,
    &super::padding::RSA_PKCS1_SHA1_FOR_LEGACY_USE_ONLY,
    "Verification of signatures using RSA keys of 1024-8192 bits,
             PKCS#1.5 padding, and SHA-1.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_2048_8192_SHA1_FOR_LEGACY_USE_ONLY,
    2048,
    &super::padding::RSA_PKCS1_SHA1_FOR_LEGACY_USE_ONLY,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PKCS#1.5 padding, and SHA-1.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY,
    1024,
    &super::RSA_PKCS1_SHA256,
    "Verification of signatures using RSA keys of 1024-8192 bits,
             PKCS#1.5 padding, and SHA-256.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_2048_8192_SHA256,
    2048,
    &super::RSA_PKCS1_SHA256,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PKCS#1.5 padding, and SHA-256.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_2048_8192_SHA384,
    2048,
    &super::RSA_PKCS1_SHA384,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PKCS#1.5 padding, and SHA-384.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_2048_8192_SHA512,
    2048,
    &super::RSA_PKCS1_SHA512,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PKCS#1.5 padding, and SHA-512.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_1024_8192_SHA512_FOR_LEGACY_USE_ONLY,
    1024,
    &super::RSA_PKCS1_SHA512,
    "Verification of signatures using RSA keys of 1024-8192 bits,
             PKCS#1.5 padding, and SHA-512.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PKCS1_3072_8192_SHA384,
    3072,
    &super::RSA_PKCS1_SHA384,
    "Verification of signatures using RSA keys of 3072-8192 bits,
             PKCS#1.5 padding, and SHA-384.\n\nSee \"`RSA_PKCS1_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);

rsa_params!(
    RSA_PSS_2048_8192_SHA256,
    2048,
    &super::RSA_PSS_SHA256,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PSS padding, and SHA-256.\n\nSee \"`RSA_PSS_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PSS_2048_8192_SHA384,
    2048,
    &super::RSA_PSS_SHA384,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PSS padding, and SHA-384.\n\nSee \"`RSA_PSS_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);
rsa_params!(
    RSA_PSS_2048_8192_SHA512,
    2048,
    &super::RSA_PSS_SHA512,
    "Verification of signatures using RSA keys of 2048-8192 bits,
             PSS padding, and SHA-512.\n\nSee \"`RSA_PSS_*` Details\" in
             `ring::signature`'s module-level documentation for more details."
);

/// Low-level API for the verification of RSA signatures.
///
/// When the public key is in DER-encoded PKCS#1 ASN.1 format, it is
/// recommended to use `ring::signature::verify()` with
/// `ring::signature::RSA_PKCS1_*`, because `ring::signature::verify()`
/// will handle the parsing in that case. Otherwise, this function can be used
/// to pass in the raw bytes for the public key components as
/// `untrusted::Input` arguments.
//
// There are a small number of tests that test this directly, but the
// test coverage for this function mostly depends on the test coverage for the
// `signature::VerificationAlgorithm` implementation for `RsaParameters`. If we
// change that, test coverage for `verify_rsa()` will need to be reconsidered.
// (The NIST test vectors were originally in a form that was optimized for
// testing `verify_rsa` directly, but the testing work for RSA PKCS#1
// verification was done during the implementation of
// `signature::VerificationAlgorithm`, before `verify_rsa` was factored out).
#[derive(Debug)]
pub struct RsaPublicKeyComponents<B: AsRef<[u8]> + core::fmt::Debug> {
    /// The public modulus, encoded in big-endian bytes without leading zeros.
    pub n: B,

    /// The public exponent, encoded in big-endian bytes without leading zeros.
    pub e: B,
}

impl<B: Copy> Copy for RsaPublicKeyComponents<B> where B: AsRef<[u8]> + core::fmt::Debug {}

impl<B: Clone> Clone for RsaPublicKeyComponents<B>
where
    B: AsRef<[u8]> + core::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            n: self.n.clone(),
            e: self.e.clone(),
        }
    }
}

impl<B> RsaPublicKeyComponents<B>
where
    B: AsRef<[u8]> + core::fmt::Debug,
{
    /// Verifies that `signature` is a valid signature of `message` using `self`
    /// as the public key. `params` determine what algorithm parameters
    /// (padding, digest algorithm, key length range, etc.) are used in the
    /// verification.
    pub fn verify(
        &self,
        params: &RsaParameters,
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), error::Unspecified> {
        let _ = cpu::features();
        verify_rsa_(
            params,
            (
                untrusted::Input::from(self.n.as_ref()),
                untrusted::Input::from(self.e.as_ref()),
            ),
            untrusted::Input::from(message),
            untrusted::Input::from(signature),
        )
    }
}

pub(crate) fn verify_rsa_(
    params: &RsaParameters,
    (n, e): (untrusted::Input, untrusted::Input),
    msg: untrusted::Input,
    signature: untrusted::Input,
) -> Result<(), error::Unspecified> {
    let max_bits = bits::BitLength::from_usize_bytes(PUBLIC_KEY_PUBLIC_MODULUS_MAX_LEN)?;

    // XXX: FIPS 186-4 seems to indicate that the minimum
    // exponent value is 2**16 + 1, but it isn't clear if this is just for
    // signing or also for verification. We support exponents of 3 and larger
    // for compatibility with other commonly-used crypto libraries.
    let Key { n, e, n_bits } = Key::from_modulus_and_exponent(n, e, params.min_bits, max_bits, 3)?;

    // The signature must be the same length as the modulus, in bytes.
    if signature.len() != n_bits.as_usize_bytes_rounded_up() {
        return Err(error::Unspecified);
    }

    // RFC 8017 Section 5.2.2: RSAVP1.

    // Step 1.
    let s = bigint::Elem::from_be_bytes_padded(signature, &n)?;
    if s.is_zero() {
        return Err(error::Unspecified);
    }

    // Step 2.
    let m = bigint::elem_exp_vartime(s, e, &n);
    let m = m.into_unencoded(&n);

    // Step 3.
    let mut decoded = [0u8; PUBLIC_KEY_PUBLIC_MODULUS_MAX_LEN];
    let decoded = fill_be_bytes_n(m, n_bits, &mut decoded);

    // Verify the padded message is correct.
    let m_hash = digest::digest(params.padding_alg.digest_alg(), msg.as_slice_less_safe());
    untrusted::Input::from(decoded).read_all(error::Unspecified, |m| {
        params.padding_alg.verify(&m_hash, m, n_bits)
    })
}

/// Returns the big-endian representation of `elem` that is
/// the same length as the minimal-length big-endian representation of
/// the modulus `n`.
///
/// `n_bits` must be the bit length of the public modulus `n`.
fn fill_be_bytes_n(
    elem: bigint::Elem<N, Unencoded>,
    n_bits: bits::BitLength,
    out: &mut [u8; PUBLIC_KEY_PUBLIC_MODULUS_MAX_LEN],
) -> &[u8] {
    let n_bytes = n_bits.as_usize_bytes_rounded_up();
    let n_bytes_padded = ((n_bytes + (LIMB_BYTES - 1)) / LIMB_BYTES) * LIMB_BYTES;
    let out = &mut out[..n_bytes_padded];
    elem.fill_be_bytes(out);
    let (padding, out) = out.split_at(n_bytes_padded - n_bytes);
    assert!(padding.iter().all(|&b| b == 0));
    out
}
