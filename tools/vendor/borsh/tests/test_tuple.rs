use borsh::{BorshDeserialize, BorshSerialize};

#[test]
fn test_unary_tuple() {
    let expected = (true,);
    let buf = expected.try_to_vec().unwrap();
    let actual = <(bool,)>::try_from_slice(&buf).expect("failed to deserialize");
    assert_eq!(actual, expected);
}
