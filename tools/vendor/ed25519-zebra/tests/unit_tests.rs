use core::convert::TryFrom;

use rand::thread_rng;

use ed25519_zebra::{Signature, SigningKey, VerificationKey, VerificationKeyBytes};

#[test]
fn parsing() {
    let sk = SigningKey::new(thread_rng());
    let pk = VerificationKey::from(&sk);
    let pkb = VerificationKeyBytes::from(&sk);
    let sig = sk.sign(b"test");

    // Most of these types don't implement Eq, so we check a round trip
    // conversion to bytes, using these as the reference points:

    let sk_array: [u8; 32] = sk.into();
    let pk_array: [u8; 32] = pk.into();
    let pkb_array: [u8; 32] = pkb.into();
    let sig_array: [u8; 64] = sig.into();

    let sk2 = SigningKey::try_from(sk.as_ref()).unwrap();
    let pk2 = VerificationKey::try_from(pk.as_ref()).unwrap();
    let pkb2 = VerificationKeyBytes::try_from(pkb.as_ref()).unwrap();
    let sig2 = Signature::try_from(<[u8; 64]>::from(sig).as_ref()).unwrap();

    assert_eq!(&sk_array[..], sk2.as_ref());
    assert_eq!(&pk_array[..], pk2.as_ref());
    assert_eq!(&pkb_array[..], pkb2.as_ref());
    assert_eq!(&sig_array[..], <[u8; 64]>::from(sig2).as_ref());

    let sk3: SigningKey = bincode::deserialize(sk.as_ref()).unwrap();
    let pk3: VerificationKey = bincode::deserialize(pk.as_ref()).unwrap();
    let pkb3: VerificationKeyBytes = bincode::deserialize(pkb.as_ref()).unwrap();
    let sig3: Signature = bincode::deserialize(<[u8; 64]>::from(sig).as_ref()).unwrap();

    assert_eq!(&sk_array[..], sk3.as_ref());
    assert_eq!(&pk_array[..], pk3.as_ref());
    assert_eq!(&pkb_array[..], pkb3.as_ref());
    assert_eq!(&sig_array[..], <[u8; 64]>::from(sig3).as_ref());
}

#[test]
fn sign_and_verify() {
    let sk = SigningKey::new(thread_rng());
    let pk = VerificationKey::from(&sk);

    let msg = b"ed25519-zebra test message";

    let sig = sk.sign(&msg[..]);

    assert_eq!(pk.verify(&sig, &msg[..]), Ok(()))
}
