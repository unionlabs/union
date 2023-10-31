use digest::dev::{digest_test, one_million_a};
use digest::new_test;

new_test!(sha224_main, "sha224", sha2::Sha224, digest_test);
new_test!(sha256_main, "sha256", sha2::Sha256, digest_test);
new_test!(sha384_main, "sha384", sha2::Sha384, digest_test);
new_test!(sha512_main, "sha512", sha2::Sha512, digest_test);
new_test!(
    sha512_224_main,
    "sha512_224",
    sha2::Sha512Trunc224,
    digest_test
);
new_test!(
    sha512_256_main,
    "sha512_256",
    sha2::Sha512Trunc256,
    digest_test
);

#[test]
fn sha256_1million_a() {
    let output = include_bytes!("data/sha256_one_million_a.bin");
    one_million_a::<sha2::Sha256>(output);
}

#[test]
#[rustfmt::skip]
fn sha512_avx2_bug() {
    use sha2::Digest;
    use hex_literal::hex;

    let mut msg = [0u8; 256];
    msg[0] = 42;
    let expected = hex!("
        2a3e943072f30afa45f2bf57ccd386f29b76dbcdb3a861224ca0b77bc3f55c7a
        d3880a49c0c9c166eedf7f209c41b380896886155acb8f6c7c07044343a3e692
    ");
    let res = sha2::Sha512::digest(&msg);
    assert_eq!(res[..], expected[..]);
}

#[test]
fn sha512_1million_a() {
    let output = include_bytes!("data/sha512_one_million_a.bin");
    one_million_a::<sha2::Sha512>(output);
}
