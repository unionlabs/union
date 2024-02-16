use crate::{Byte32, Field, Fr, HashScheme, PrimeField};

#[derive(Debug, PartialEq, Clone)]
pub struct TestHash;
impl HashScheme for TestHash {
    fn hash_scheme(arr: &[Fr], domain: &Fr) -> Fr {
        let lc_eff = Fr::from_u64(65536);
        let mut sum = *domain;
        for bi in arr {
            let mut nbi = *bi;
            nbi.mul_assign(bi);
            sum.square();
            sum.mul_assign(&lc_eff);
            sum.add_assign(&nbi);
        }
        sum
    }
}

#[test]
fn test_new_byte32() {
    struct Testcase {
        input: Vec<u8>,
        expected: &'static [u8],
        expected_padding_zero: &'static [u8],
        expected_hash: &'static str,
        expected_hash_padding: &'static str,
    }
    let tests = &[
        Testcase {
            input: vec![1, 1, 1, 1],
            expected: &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 1, 1,
            ],
            expected_padding_zero: &[
                1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            expected_hash: "19342813114117753747472897",
            expected_hash_padding:
                "4198633341355723145865718849633731687852896197776343461751712629107518959468",
        },
        Testcase {
            input: vec![1_u8; 34],
            expected: &[
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ],
            expected_padding_zero: &[
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ],
            expected_hash:
                "19162873132136764367682277409313605623778997630491468285254908822491098844002",
            expected_hash_padding:
                "19162873132136764367682277409313605623778997630491468285254908822491098844002",
        },
    ];

    for tt in tests {
        let result = Byte32::from_bytes(&tt.input[..]);
        let padding_result = Byte32::from_bytes_padding(&tt.input[..]);
        assert_eq!(tt.expected, result.bytes());
        assert_eq!(tt.expected_padding_zero, padding_result.bytes());
        let hash = result.hash::<TestHash>().unwrap();
        let padding_hash = padding_result.hash::<TestHash>().unwrap();
        let expected_hash = Fr::from_str(tt.expected_hash).unwrap();
        let expected_hash_padding = Fr::from_str(tt.expected_hash_padding).unwrap();
        assert_eq!(hash, expected_hash);
        assert_eq!(padding_hash, expected_hash_padding);
    }
}
