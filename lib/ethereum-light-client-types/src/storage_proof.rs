use unionlabs::uint::U256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageProof {
    // #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub key: U256,
    // #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub value: U256,
    // #[serde(with = "::serde_utils::hex_string_list")]
    pub proof: Vec<Vec<u8>>,
}
