use crate::{Fr, Hash};

fn gen_key(n: char) -> Vec<u8> {
    let mut tmp = vec![n as u8; 32];
    tmp[0] = 0;
    tmp[1] = 1;
    tmp
}

#[test]
pub fn test_hash_to_fr() {
    {
        let k1 = gen_key('a');
        let h1 = Hash::from_bytes(&k1);
        let f1 = h1.fr().unwrap();
        let nh1 = f1;
        assert_eq!(f1, nh1);
    }

    {
        let fr: Fr = 12345_u64.into();
        let hash: Hash = fr.into();
        assert_eq!(
            hash.raw_bytes(),
            &[
                57, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
    }
}
