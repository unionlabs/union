use borsh::BorshDeserialize;

#[derive(BorshDeserialize, Debug)]
enum A {
    X,
    Y,
}

#[derive(BorshDeserialize, Debug)]
struct B {
    #[allow(unused)]
    x: u64,
    #[allow(unused)]
    y: u32,
}

const ERROR_UNEXPECTED_LENGTH_OF_INPUT: &str = "Unexpected length of input";
const ERROR_INVALID_ZERO_VALUE: &str = "Expected a non-zero value";

#[test]
fn test_missing_bytes() {
    let bytes = vec![1, 0];
    assert_eq!(
        B::try_from_slice(&bytes).unwrap_err().to_string(),
        ERROR_UNEXPECTED_LENGTH_OF_INPUT
    );
}

#[test]
fn test_invalid_enum_variant() {
    let bytes = vec![123];
    assert_eq!(
        A::try_from_slice(&bytes).unwrap_err().to_string(),
        "Unexpected variant index: 123"
    );
}

#[test]
fn test_extra_bytes() {
    let bytes = vec![1, 0, 0, 0, 32, 32];
    assert_eq!(
        <Vec<u8>>::try_from_slice(&bytes).unwrap_err().to_string(),
        "Not all bytes read"
    );
}

#[test]
fn test_invalid_bool() {
    for i in 2u8..=255 {
        let bytes = [i];
        assert_eq!(
            <bool>::try_from_slice(&bytes).unwrap_err().to_string(),
            format!("Invalid bool representation: {}", i)
        );
    }
}

#[test]
fn test_invalid_option() {
    for i in 2u8..=255 {
        let bytes = [i, 32];
        assert_eq!(
            <Option<u8>>::try_from_slice(&bytes)
                .unwrap_err()
                .to_string(),
            format!(
                "Invalid Option representation: {}. The first byte must be 0 or 1",
                i
            )
        );
    }
}

#[test]
fn test_invalid_result() {
    for i in 2u8..=255 {
        let bytes = [i, 0];
        assert_eq!(
            <Result<u64, String>>::try_from_slice(&bytes)
                .unwrap_err()
                .to_string(),
            format!(
                "Invalid Result representation: {}. The first byte must be 0 or 1",
                i
            )
        );
    }
}

#[test]
fn test_invalid_length() {
    let bytes = vec![255u8; 4];
    assert_eq!(
        <Vec<u64>>::try_from_slice(&bytes).unwrap_err().to_string(),
        ERROR_UNEXPECTED_LENGTH_OF_INPUT
    );
}

#[test]
fn test_invalid_length_string() {
    let bytes = vec![255u8; 4];
    assert_eq!(
        String::try_from_slice(&bytes).unwrap_err().to_string(),
        ERROR_UNEXPECTED_LENGTH_OF_INPUT
    );
}

#[test]
fn test_non_utf_string() {
    let bytes = vec![1, 0, 0, 0, 0xC0];
    assert_eq!(
        String::try_from_slice(&bytes).unwrap_err().to_string(),
        "invalid utf-8 sequence of 1 bytes from index 0"
    );
}

#[test]
fn test_nan_float() {
    let bytes = vec![0, 0, 192, 127];
    assert_eq!(
        f32::try_from_slice(&bytes).unwrap_err().to_string(),
        "For portability reasons we do not allow to deserialize NaNs."
    );
}

#[test]
fn test_evil_bytes_vec_with_extra() {
    // Should fail to allocate given length
    // test takes a really long time if read() is used instead of read_exact()
    let bytes = vec![255, 255, 255, 255, 32, 32];
    assert_eq!(
        <Vec<[u8; 32]>>::try_from_slice(&bytes)
            .unwrap_err()
            .to_string(),
        ERROR_UNEXPECTED_LENGTH_OF_INPUT
    );
}

#[test]
fn test_evil_bytes_string_extra() {
    // Might fail if reading too much
    let bytes = vec![255, 255, 255, 255, 32, 32];
    assert_eq!(
        String::try_from_slice(&bytes).unwrap_err().to_string(),
        ERROR_UNEXPECTED_LENGTH_OF_INPUT
    );
}

#[test]
fn test_zero_on_nonzero_integer_u8() {
    let bytes = &[0];
    assert_eq!(
        std::num::NonZeroU8::try_from_slice(bytes)
            .unwrap_err()
            .to_string(),
        ERROR_INVALID_ZERO_VALUE
    );
}

#[test]
fn test_zero_on_nonzero_integer_u32() {
    let bytes = &[0; 4];
    assert_eq!(
        std::num::NonZeroU32::try_from_slice(bytes)
            .unwrap_err()
            .to_string(),
        ERROR_INVALID_ZERO_VALUE
    );
}

#[test]
fn test_zero_on_nonzero_integer_i64() {
    let bytes = &[0; 8];
    assert_eq!(
        std::num::NonZeroI64::try_from_slice(bytes)
            .unwrap_err()
            .to_string(),
        ERROR_INVALID_ZERO_VALUE
    );
}

#[test]
fn test_zero_on_nonzero_integer_usize() {
    let bytes = &[0; 8];
    assert_eq!(
        std::num::NonZeroUsize::try_from_slice(bytes)
            .unwrap_err()
            .to_string(),
        ERROR_INVALID_ZERO_VALUE
    );
}

#[test]
fn test_zero_on_nonzero_integer_missing_byte() {
    let bytes = &[0; 7];
    assert_eq!(
        std::num::NonZeroUsize::try_from_slice(bytes)
            .unwrap_err()
            .to_string(),
        ERROR_UNEXPECTED_LENGTH_OF_INPUT
    );
}
