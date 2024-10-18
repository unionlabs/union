use macros::model;

#[model]
pub struct GenesisMetadata {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug("{}", ::serde_utils::to_hex(&key))]
    pub key: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug("{}", ::serde_utils::to_hex(&value))]
    pub value: Vec<u8>,
}
