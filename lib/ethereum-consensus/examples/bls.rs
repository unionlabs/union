use ethereum_consensus::crypto::{verify_signature, SecretKey};
use rand::prelude::*;

/// A simple example that adopts the original BLST's example:
/// https://github.com/supranational/blst/tree/master/bindings/rust
fn main() {
    let mut rng = thread_rng();
    let sk = SecretKey::random(&mut rng).expect("can randomly generate a secret key");
    let pk = sk.public_key();

    let msg = b"blst is such a blast";
    let sig = sk.sign(msg);

    let valid = verify_signature(&pk, msg, &sig).is_ok();
    println!("Signature is valid: {valid:?}");
}
