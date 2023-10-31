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

//! ECDSA Signatures using the P-256 and P-384 curves.

use crate::{
    digest,
    ec::suite_b::ops::*,
    limb::{self, LIMB_BYTES},
};

/// Calculate the digest of `msg` using the digest algorithm `digest_alg`. Then
/// convert the digest to a scalar in the range [0, n) as described in
/// NIST's FIPS 186-4 Section 4.2. Note that this is one of the few cases where
/// a `Scalar` is allowed to have the value zero.
///
/// NIST's FIPS 186-4 4.2 says "When the length of the output of the hash
/// function is greater than N (i.e., the bit length of q), then the leftmost N
/// bits of the hash function output block shall be used in any calculation
/// using the hash function output during the generation or verification of a
/// digital signature."
///
/// "Leftmost N bits" means "N most significant bits" because we interpret the
/// digest as a bit-endian encoded integer.
///
/// The NSA guide instead vaguely suggests that we should convert the digest
/// value to an integer and then reduce it mod `n`. However, real-world
/// implementations (e.g. `digest_to_bn` in OpenSSL and `hashToInt` in Go) do
/// what FIPS 186-4 says to do, not what the NSA guide suggests.
///
/// Why shifting the value right by at most one bit is sufficient: P-256's `n`
/// has its 256th bit set; i.e. 2**255 < n < 2**256. Once we've truncated the
/// digest to 256 bits and converted it to an integer, it will have a value
/// less than 2**256. If the value is larger than `n` then shifting it one bit
/// right will give a value less than 2**255, which is less than `n`. The
/// analogous argument applies for P-384. However, it does *not* apply in
/// general; for example, it doesn't apply to P-521.
pub fn digest_scalar(ops: &ScalarOps, msg: digest::Digest) -> Scalar {
    digest_scalar_(ops, msg.as_ref())
}

#[cfg(test)]
pub(crate) fn digest_bytes_scalar(ops: &ScalarOps, digest: &[u8]) -> Scalar {
    digest_scalar_(ops, digest)
}

// This is a separate function solely so that we can test specific digest
// values like all-zero values and values larger than `n`.
fn digest_scalar_(ops: &ScalarOps, digest: &[u8]) -> Scalar {
    let cops = ops.common;
    let num_limbs = cops.num_limbs;
    let digest = if digest.len() > num_limbs * LIMB_BYTES {
        &digest[..(num_limbs * LIMB_BYTES)]
    } else {
        digest
    };

    scalar_parse_big_endian_partially_reduced_variable_consttime(
        cops,
        limb::AllowZero::Yes,
        untrusted::Input::from(digest),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::digest_bytes_scalar;
    use crate::{
        digest,
        ec::suite_b::ops::*,
        limb::{self, LIMB_BYTES},
        test,
    };

    #[test]
    fn test() {
        test::run(
            test_file!("ecdsa_digest_scalar_tests.txt"),
            |section, test_case| {
                assert_eq!(section, "");

                let curve_name = test_case.consume_string("Curve");
                let digest_name = test_case.consume_string("Digest");
                let input = test_case.consume_bytes("Input");
                let output = test_case.consume_bytes("Output");

                let (ops, digest_alg) = match (curve_name.as_str(), digest_name.as_str()) {
                    ("P-256", "SHA256") => (&p256::PUBLIC_SCALAR_OPS, &digest::SHA256),
                    ("P-256", "SHA384") => (&p256::PUBLIC_SCALAR_OPS, &digest::SHA384),
                    ("P-384", "SHA256") => (&p384::PUBLIC_SCALAR_OPS, &digest::SHA256),
                    ("P-384", "SHA384") => (&p384::PUBLIC_SCALAR_OPS, &digest::SHA384),
                    _ => {
                        panic!("Unsupported curve+digest: {}+{}", curve_name, digest_name);
                    }
                };

                let num_limbs = ops.public_key_ops.common.num_limbs;
                assert_eq!(input.len(), digest_alg.output_len);
                assert_eq!(
                    output.len(),
                    ops.public_key_ops.common.num_limbs * LIMB_BYTES
                );

                let expected = scalar_parse_big_endian_variable(
                    ops.public_key_ops.common,
                    limb::AllowZero::Yes,
                    untrusted::Input::from(&output),
                )
                .unwrap();

                let actual = digest_bytes_scalar(ops.scalar_ops, &input);

                assert_eq!(actual.limbs[..num_limbs], expected.limbs[..num_limbs]);

                Ok(())
            },
        );
    }
}
