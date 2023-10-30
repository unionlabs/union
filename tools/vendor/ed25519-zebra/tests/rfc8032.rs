//! RFC 8032 test vectors.
//!
//! Note that RFC 8032 does not actually specify validation criteria for Ed25519,
//! so these are basic sanity checks, rather than the more detailed test vectors
//! in consensus.rs.

use bincode;
use ed25519_zebra::*;
use hex;

fn rfc8032_test_case(sk_bytes: Vec<u8>, pk_bytes: Vec<u8>, sig_bytes: Vec<u8>, msg: Vec<u8>) {
    let sk: SigningKey = bincode::deserialize(&sk_bytes).expect("sk should deserialize");
    let pk: VerificationKey = bincode::deserialize(&pk_bytes).expect("pk should deserialize");
    let sig: Signature = bincode::deserialize(&sig_bytes).expect("sig should deserialize");

    assert!(pk.verify(&sig, &msg).is_ok(), "verification failed");

    let pk_from_sk = VerificationKey::from(&sk);
    assert_eq!(
        VerificationKeyBytes::from(pk),
        VerificationKeyBytes::from(pk_from_sk),
        "regenerated pubkey did not match test vector pubkey"
    );

    let sig_from_sk = sk.sign(&msg);
    assert_eq!(
        sig, sig_from_sk,
        "regenerated signature did not match test vector"
    );
}

#[test]
fn rfc8032_test_1() {
    rfc8032_test_case(
        hex::decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
            .expect("hex should decode"),
        hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
            .expect("hex should decode"),
        hex::decode("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b")
            .expect("hex should decode"),
        hex::decode("")
            .expect("hex should decode"),
    );
}

#[test]
fn rfc8032_test_2() {
    rfc8032_test_case(
        hex::decode("4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb")
            .expect("hex should decode"),
        hex::decode("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c")
            .expect("hex should decode"),
        hex::decode("92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00")
            .expect("hex should decode"),
        hex::decode("72")
            .expect("hex should decode"),
    );
}

#[test]
fn rfc8032_test_3() {
    rfc8032_test_case(
        hex::decode("c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7")
            .expect("hex should decode"),
        hex::decode("fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025")
            .expect("hex should decode"),
        hex::decode("6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a")
            .expect("hex should decode"),
        hex::decode("af82")
            .expect("hex should decode"),
    );
}
