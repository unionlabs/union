#[cfg(not(feature = "reset"))]
use digest::new_mac_test as new_test;
#[cfg(feature = "reset")]
use digest::new_resettable_mac_test as new_test;

new_test!(blake2b_mac, "blake2b/mac", blake2::Blake2bMac512);
new_test!(blake2s_mac, "blake2s/mac", blake2::Blake2sMac256);

#[test]
fn blake2b_new_test() {
    use blake2::digest::{generic_array::GenericArray, KeyInit, Mac};

    fn run<T: Mac + KeyInit>(key: &[u8]) {
        const DATA: &[u8] = &[42; 300];
        let res1 = <T as Mac>::new(GenericArray::from_slice(key))
            .chain_update(DATA)
            .finalize()
            .into_bytes();
        let res2 = <T as Mac>::new_from_slice(&key)
            .unwrap()
            .chain_update(DATA)
            .finalize()
            .into_bytes();
        assert_eq!(res1, res2);
    }

    run::<blake2::Blake2sMac256>(&[0x42; 32]);
    run::<blake2::Blake2bMac512>(&[0x42; 64]);
}
