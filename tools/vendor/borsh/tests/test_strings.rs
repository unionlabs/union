use borsh::{BorshDeserialize, BorshSerialize};

macro_rules! test_string {
    ($test_name: ident, $str: expr) => {
        #[test]
        fn $test_name() {
            let s = $str.to_string();
            let buf = s.try_to_vec().unwrap();
            let actual_s = <String>::try_from_slice(&buf).expect("failed to deserialize a string");
            assert_eq!(actual_s, s);
        }
    };
}

test_string!(test_empty_string, "");
test_string!(test_a, "a");
test_string!(test_hello_world, "hello world");
test_string!(test_x_1024, "x".repeat(1024));
test_string!(test_x_4096, "x".repeat(4096));
test_string!(test_x_65535, "x".repeat(65535));
test_string!(test_hello_1000, "hello world!".repeat(1000));
test_string!(test_non_ascii, "💩");
