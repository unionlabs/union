use criterion::{criterion_group, criterion_main, Criterion, PlottingBackend};
use std::time::Duration;

use english_numbers::convert_no_fmt;
use hex_literal::hex;
use serde::Deserialize;

// Crypto stuff
use digest::Digest;
use k256::ecdsa::SigningKey; // type alias
use k256::elliptic_curve::sec1::ToEncodedPoint;
use sha2::Sha256;

use cosmwasm_crypto::{
    ed25519_batch_verify, ed25519_verify, secp256k1_recover_pubkey, secp256k1_verify,
};
use std::cmp::min;

const COSMOS_SECP256K1_MSG_HEX: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712650a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a02080112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
const COSMOS_SECP256K1_SIGNATURE_HEX: &str = "c9dd20e07464d3a688ff4b710b1fbc027e495e797cfa0b4804da2ed117959227772de059808f765aa29b8f92edf30f4c2c5a438e30d3fe6897daa7141e3ce6f9";
const COSMOS_SECP256K1_PUBKEY_BASE64: &str = "A08EGB7ro1ORuFhjOnZcSgwYlpe0DSFjVNUIkNNQxwKQ";

// TEST 3 test vector from https://tools.ietf.org/html/rfc8032#section-7.1
const COSMOS_ED25519_MSG_HEX: &str = "af82";
const COSMOS_ED25519_SIGNATURE_HEX: &str = "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a";
const COSMOS_ED25519_PUBLIC_KEY_HEX: &str =
    "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";

// Test data from https://tools.ietf.org/html/rfc8032#section-7.1
const COSMOS_ED25519_TESTS_JSON: &str = "./testdata/ed25519_tests.json";

#[derive(Deserialize, Debug)]
struct Encoded {
    #[serde(rename = "privkey")]
    #[allow(dead_code)]
    private_key: String,
    #[serde(rename = "pubkey")]
    public_key: String,
    message: String,
    signature: String,
}

fn read_cosmos_sigs() -> Vec<Encoded> {
    use std::fs::File;
    use std::io::BufReader;

    // Open the file in read-only mode with buffer.
    let file = File::open(COSMOS_ED25519_TESTS_JSON).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}

#[allow(clippy::type_complexity)]
fn read_decode_cosmos_sigs() -> (Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let codes = read_cosmos_sigs();

    let mut messages: Vec<Vec<u8>> = vec![];
    let mut signatures: Vec<Vec<u8>> = vec![];
    let mut public_keys: Vec<Vec<u8>> = vec![];

    for encoded in codes {
        let message = hex::decode(&encoded.message).unwrap();
        messages.push(message);

        let signature = hex::decode(&encoded.signature).unwrap();
        signatures.push(signature);

        let public_key = hex::decode(&encoded.public_key).unwrap();
        public_keys.push(public_key);
    }

    (messages, signatures, public_keys)
}

fn bench_crypto(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crypto");

    group.bench_function("secp256k1_verify", |b| {
        let message = hex::decode(COSMOS_SECP256K1_MSG_HEX).unwrap();
        let message_hash = Sha256::digest(message);
        let signature = hex::decode(COSMOS_SECP256K1_SIGNATURE_HEX).unwrap();
        let public_key = base64::decode(COSMOS_SECP256K1_PUBKEY_BASE64).unwrap();
        b.iter(|| {
            assert!(secp256k1_verify(&message_hash, &signature, &public_key).unwrap());
        });
    });

    group.bench_function("secp256k1_recover_pubkey", |b| {
        let message_hash =
            hex!("82ff40c0a986c6a5cfad4ddf4c3aa6996f1a7837f9c398e17e5de5cbd5a12b28");
        let private_key =
            hex!("3c9229289a6125f7fdf1885a77bb12c37a8d3b4962d936f7e3084dece32a3ca1");
        let r_s = hex!("99e71a99cb2270b8cac5254f9e99b6210c6c10224a1579cf389ef88b20a1abe9129ff05af364204442bdb53ab6f18a99ab48acc9326fa689f228040429e3ca66");
        let recovery_param: u8 = 0;

        let expected = SigningKey::from_bytes(&private_key)
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();

        b.iter(|| {
            let pubkey = secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap();
            assert_eq!(pubkey, expected);
        });
    });

    group.bench_function("ed25519_verify", |b| {
        let message = hex::decode(COSMOS_ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(COSMOS_ED25519_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(COSMOS_ED25519_PUBLIC_KEY_HEX).unwrap();
        b.iter(|| {
            assert!(ed25519_verify(&message, &signature, &public_key).unwrap());
        });
    });

    // Ed25519 batch verification of different batch lengths
    {
        let (messages, signatures, public_keys) = read_decode_cosmos_sigs();
        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        for n in (1..=min(messages.len(), 10)).step_by(2) {
            group.bench_function(
                format!("ed25519_batch_verify_{}", convert_no_fmt(n as i64)),
                |b| {
                    b.iter(|| {
                        assert!(ed25519_batch_verify(
                            &messages[..n],
                            &signatures[..n],
                            &public_keys[..n]
                        )
                        .unwrap());
                    });
                },
            );
        }
    }

    // Ed25519 batch verification of different batch lengths, with the same pubkey
    {
        //FIXME: Use different messages / signatures
        let messages = [hex::decode(COSMOS_ED25519_MSG_HEX).unwrap()];
        let signatures = [hex::decode(COSMOS_ED25519_SIGNATURE_HEX).unwrap()];
        let public_keys = [hex::decode(COSMOS_ED25519_PUBLIC_KEY_HEX).unwrap()];

        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        for n in (1..10).step_by(2) {
            group.bench_function(
                format!(
                    "ed25519_batch_verify_one_pubkey_{}",
                    convert_no_fmt(n as i64)
                ),
                |b| {
                    b.iter(|| {
                        assert!(ed25519_batch_verify(
                            &messages.repeat(n),
                            &signatures.repeat(n),
                            &public_keys
                        )
                        .unwrap());
                    });
                },
            );
        }
    }

    group.finish();
}

fn make_config() -> Criterion {
    Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
}

criterion_group!(
    name = crypto;
    config = make_config();
    targets = bench_crypto
);
criterion_main!(crypto);
