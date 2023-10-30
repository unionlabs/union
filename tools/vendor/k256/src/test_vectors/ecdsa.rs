//! ECDSA/secp256k1 test vectors

use ecdsa_core::dev::TestVector;
use hex_literal::hex;

/// ECDSA/secp256k1 test vectors
pub const ECDSA_TEST_VECTORS: &[TestVector] = &[TestVector {
    d: &hex!("ebb2c082fd7727890a28ac82f6bdf97bad8de9f5d7c9028692de1a255cad3e0f"),
    q_x: &hex!("779dd197a5df977ed2cf6cb31d82d43328b790dc6b3b7d4437a427bd5847dfcd"),
    q_y: &hex!("e94b724a555b6d017bb7607c3e3281daf5b1699d6ef4124975c9237b917d426f"),
    k: &hex!("49a0d7b786ec9cde0d0721d72804befd06571c974b191efb42ecf322ba9ddd9a"),
    m: &hex!("4b688df40bcedbe641ddb16ff0a1842d9c67ea1c3bf63f3e0471baa664531d1a"),
    r: &hex!("241097efbf8b63bf145c8961dbdf10c310efbb3b2676bbc0f8b08505c9e2f795"),
    s: &hex!("021006b7838609339e8b415a7f9acb1b661828131aef1ecbc7955dfb01f3ca0e"),
}];
