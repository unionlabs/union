use blake2::{digest::FixedOutput, Blake2bMac512, Blake2sMac256};
use hex_literal::hex;

#[test]
#[rustfmt::skip]
fn blake2s_persona() {
    let key= hex!("
        000102030405060708090a0b0c0d0e0f
        101112131415161718191a1b1c1d1e1f
    ");
    let persona = b"personal";
    let ctx = Blake2sMac256::new_with_salt_and_personal(&key, &[], persona).unwrap();
    assert_eq!(
        ctx.finalize_fixed()[..],
        hex!("
            25a4ee63b594aed3f88a971e1877ef70
            99534f9097291f88fb86c79b5e70d022
        ")[..],
    );
}

#[test]
#[rustfmt::skip]
fn blake2b_persona() {
    let key = hex!("
        000102030405060708090a0b0c0d0e0f
        101112131415161718191a1b1c1d1e1f
    ");
    let persona = b"personal";
    let ctx = Blake2bMac512::new_with_salt_and_personal(&key, &[], persona).unwrap();
    assert_eq!(
        ctx.finalize_fixed()[..],
        hex!("
            03de3b295dcfc3b25b05abb09bc95fe3
            e9ff3073638badc68101d1e42019d077
            1dd07525a3aae8318e92c5e5d967ba92
            e4810d0021d7bf3b49da0b4b4a8a4e1f
        ")[..],
    );
}
