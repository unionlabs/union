use crate::{decode_config, encode::encoded_size, encode_config_buf, CharacterSet, Config};

use std::str;

use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
    FromEntropy, Rng,
};

#[test]
fn roundtrip_random_config_short() {
    // exercise the slower encode/decode routines that operate on shorter buffers more vigorously
    roundtrip_random_config(Uniform::new(0, 50), 10_000);
}

#[test]
fn roundtrip_random_config_long() {
    roundtrip_random_config(Uniform::new(0, 1000), 10_000);
}

pub fn assert_encode_sanity(encoded: &str, config: Config, input_len: usize) {
    let input_rem = input_len % 3;
    let expected_padding_len = if input_rem > 0 {
        if config.pad {
            3 - input_rem
        } else {
            0
        }
    } else {
        0
    };

    let expected_encoded_len = encoded_size(input_len, config).unwrap();

    assert_eq!(expected_encoded_len, encoded.len());

    let padding_len = encoded.chars().filter(|&c| c == '=').count();

    assert_eq!(expected_padding_len, padding_len);

    let _ = str::from_utf8(encoded.as_bytes()).expect("Base64 should be valid utf8");
}

fn roundtrip_random_config(input_len_range: Uniform<usize>, iterations: u32) {
    let mut input_buf: Vec<u8> = Vec::new();
    let mut encoded_buf = String::new();
    let mut rng = rand::rngs::SmallRng::from_entropy();

    for _ in 0..iterations {
        input_buf.clear();
        encoded_buf.clear();

        let input_len = input_len_range.sample(&mut rng);

        let config = random_config(&mut rng);

        for _ in 0..input_len {
            input_buf.push(rng.gen());
        }

        encode_config_buf(&input_buf, config, &mut encoded_buf);

        assert_encode_sanity(&encoded_buf, config, input_len);

        assert_eq!(input_buf, decode_config(&encoded_buf, config).unwrap());
    }
}

pub fn random_config<R: Rng>(rng: &mut R) -> Config {
    const CHARSETS: &[CharacterSet] = &[
        CharacterSet::UrlSafe,
        CharacterSet::Standard,
        CharacterSet::Crypt,
        CharacterSet::ImapMutf7,
        CharacterSet::BinHex,
    ];
    let charset = *CHARSETS.choose(rng).unwrap();

    Config::new(charset, rng.gen())
}
