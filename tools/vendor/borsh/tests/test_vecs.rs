use borsh::{BorshDeserialize, BorshSerialize};

macro_rules! test_vec {
    ($v: expr, $t: ty) => {
        let buf = $v.try_to_vec().unwrap();
        let actual_v: Vec<$t> =
            BorshDeserialize::try_from_slice(&buf).expect("failed to deserialize");
        assert_eq!(actual_v, $v);
    };
}

macro_rules! test_vecs {
    ($test_name: ident, $el: expr, $t: ty) => {
        #[test]
        fn $test_name() {
            test_vec!(Vec::<$t>::new(), $t);
            test_vec!(vec![$el], $t);
            test_vec!(vec![$el; 10], $t);
            test_vec!(vec![$el; 100], $t);
            test_vec!(vec![$el; 1000], $t);
            test_vec!(vec![$el; 10000], $t);
        }
    };
}

test_vecs!(test_vec_u8, 100u8, u8);
test_vecs!(test_vec_i8, 100i8, i8);
test_vecs!(test_vec_u32, 1000000000u32, u32);
test_vecs!(test_vec_f32, 1000000000.0f32, f32);
test_vecs!(test_vec_string, "a".to_string(), String);
test_vecs!(test_vec_vec_u8, vec![100u8; 10], Vec<u8>);
test_vecs!(test_vec_vec_u32, vec![100u32; 10], Vec<u32>);
