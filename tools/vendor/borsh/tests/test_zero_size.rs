use borsh::BorshDeserialize;

#[derive(BorshDeserialize, PartialEq, Debug)]
struct A;

#[test]
fn test_deserialize_vector_to_many_zero_size_struct() {
    let v = [0u8, 0u8, 0u8, 64u8];
    let a = Vec::<A>::try_from_slice(&v).unwrap();
    assert_eq!(A {}, a[usize::pow(2, 30) - 1])
}
