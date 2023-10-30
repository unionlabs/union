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

// *R* and *r* in Montgomery math refer to different things, so we always use
// `R` to refer to *R* to avoid confusion, even when that's against the normal
// naming conventions. Also the standard camelCase names are used for `KeyPair`
// components.

/// RSA signatures.
use crate::{
    arithmetic::bigint,
    bits, error,
    io::{self, der},
    limb,
};

mod padding;

// `RSA_PKCS1_SHA1` is intentionally not exposed.
pub use self::padding::{
    RsaEncoding, RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, RSA_PSS_SHA256,
    RSA_PSS_SHA384, RSA_PSS_SHA512,
};

// Maximum RSA modulus size supported for signature verification (in bytes).
const PUBLIC_KEY_PUBLIC_MODULUS_MAX_LEN: usize = bigint::MODULUS_MAX_LIMBS * limb::LIMB_BYTES;

// Keep in sync with the documentation comment for `KeyPair`.
const PRIVATE_KEY_PUBLIC_MODULUS_MAX_BITS: bits::BitLength = bits::BitLength::from_usize_bits(4096);

/// Parameters for RSA verification.
#[derive(Debug)]
pub struct RsaParameters {
    padding_alg: &'static dyn padding::Verification,
    min_bits: bits::BitLength,
}

fn parse_public_key(
    input: untrusted::Input,
) -> Result<(io::Positive, io::Positive), error::Unspecified> {
    input.read_all(error::Unspecified, |input| {
        der::nested(input, der::Tag::Sequence, error::Unspecified, |input| {
            let n = der::positive_integer(input)?;
            let e = der::positive_integer(input)?;
            Ok((n, e))
        })
    })
}

// Type-level representation of an RSA public modulus *n*. See
// `super::bigint`'s modulue-level documentation.
#[derive(Copy, Clone)]
pub enum N {}

unsafe impl bigint::PublicModulus for N {}

pub mod verification;

pub mod signing;
