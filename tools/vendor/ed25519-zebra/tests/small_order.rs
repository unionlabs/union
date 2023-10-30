use color_eyre::Report;
use curve25519_dalek::{
    constants::EIGHT_TORSION, edwards::CompressedEdwardsY, scalar::Scalar, traits::IsIdentity,
};
use once_cell::sync::Lazy;
use sha2::{Digest, Sha512};

mod util;
use util::TestCase;

#[allow(non_snake_case)]
pub static SMALL_ORDER_SIGS: Lazy<Vec<TestCase>> = Lazy::new(|| {
    let mut tests = Vec::new();
    let s = Scalar::zero();

    // Use all the canonical encodings of the 8-torsion points,
    // and the low-order non-canonical encodings.
    let encodings = EIGHT_TORSION
        .iter()
        .map(|point| point.compress().to_bytes())
        .chain(util::non_canonical_point_encodings().into_iter().take(6))
        .collect::<Vec<_>>();

    /*
    for (i, e) in encodings.iter().enumerate() {
        println!("{}: {}", i, hex::encode(e));
    }
    */

    for A_bytes in &encodings {
        let A = CompressedEdwardsY(*A_bytes).decompress().unwrap();
        for R_bytes in &encodings {
            let R = CompressedEdwardsY(*R_bytes).decompress().unwrap();
            let sig_bytes = {
                let mut bytes = [0u8; 64];
                bytes[0..32].copy_from_slice(&R_bytes[..]);
                bytes[32..64].copy_from_slice(s.as_bytes());
                bytes
            };
            let vk_bytes = *A_bytes;
            // The verification equation is [8][s]B = [8]R + [8][k]A.
            // If R, A are torsion points the LHS is 0, setting s = 0 makes RHS 0.
            let valid_zip215 = true;
            // In the legacy equation the RHS is 0 and the LHS is R + [k]A.
            // This will be valid only if:
            // * A is not all zeros.
            // * R is not an excluded point
            // * R + [k]A = 0
            // * R is canonically encoded (because the check recomputes R)
            let k = Scalar::from_hash(
                Sha512::default()
                    .chain(&sig_bytes[0..32])
                    .chain(vk_bytes)
                    .chain(b"Zcash"),
            );
            let check = R + k * A;
            let non_canonical_R = R.compress().as_bytes() != R_bytes;
            let valid_legacy = if vk_bytes == [0; 32]
                || util::EXCLUDED_POINT_ENCODINGS.contains(R.compress().as_bytes())
                || !check.is_identity()
                || non_canonical_R
            {
                false
            } else {
                true
            };

            tests.push(TestCase {
                vk_bytes,
                sig_bytes,
                valid_legacy,
                valid_zip215,
            })
        }
    }
    tests
});

#[test]
fn conformance() -> Result<(), Report> {
    for case in SMALL_ORDER_SIGS.iter() {
        case.check()?;
    }
    println!("{:#?}", *SMALL_ORDER_SIGS);
    Ok(())
}

#[test]
fn individual_matches_batch_verification() -> Result<(), Report> {
    use core::convert::TryFrom;
    use ed25519_zebra::{batch, Signature, VerificationKey, VerificationKeyBytes};
    for case in SMALL_ORDER_SIGS.iter() {
        let msg = b"Zcash";
        let sig = Signature::from(case.sig_bytes);
        let vkb = VerificationKeyBytes::from(case.vk_bytes);
        let individual_verification =
            VerificationKey::try_from(vkb).and_then(|vk| vk.verify(&sig, msg));
        let mut bv = batch::Verifier::new();
        bv.queue((vkb, sig, msg));
        let batch_verification = bv.verify(rand::thread_rng());
        assert_eq!(individual_verification.is_ok(), batch_verification.is_ok());
    }
    Ok(())
}
