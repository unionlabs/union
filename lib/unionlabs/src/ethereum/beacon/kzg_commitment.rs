use crate::macros::hex_string_array_wrapper;

hex_string_array_wrapper! {
    pub struct KZGCommitment(pub [u8; 48]);
}
