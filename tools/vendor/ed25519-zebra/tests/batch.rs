use rand::thread_rng;

use ed25519_zebra::*;

#[test]
fn batch_verify() {
    let mut batch = batch::Verifier::new();
    for _ in 0..32 {
        let sk = SigningKey::new(thread_rng());
        let pk_bytes = VerificationKeyBytes::from(&sk);
        let msg = b"BatchVerifyTest";
        let sig = sk.sign(&msg[..]);
        batch.queue((pk_bytes, sig, msg));
    }
    assert!(batch.verify(thread_rng()).is_ok());
}

#[test]
fn batch_verify_with_one_bad_sig() {
    let bad_index = 10;
    let mut batch = batch::Verifier::new();
    let mut items = Vec::new();
    for i in 0..32 {
        let sk = SigningKey::new(thread_rng());
        let pk_bytes = VerificationKeyBytes::from(&sk);
        let msg = b"BatchVerifyTest";
        let sig = if i != bad_index {
            sk.sign(&msg[..])
        } else {
            sk.sign(b"badmsg")
        };
        let item: batch::Item = (pk_bytes, sig, msg).into();
        items.push(item.clone());
        batch.queue(item);
    }
    assert!(batch.verify(thread_rng()).is_err());
    for (i, item) in items.drain(..).enumerate() {
        if i != bad_index {
            assert!(item.verify_single().is_ok());
        } else {
            assert!(item.verify_single().is_err());
        }
    }
}
