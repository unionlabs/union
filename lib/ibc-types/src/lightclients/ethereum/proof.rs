#[derive(Debug, Clone, PartialEq)]
pub struct Proof {
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub key: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub value: Vec<u8>,
    // #[serde(with = "::serde_utils::inner_base64")]
    pub proof: Vec<Vec<u8>>,
}

impl From<Proof> for protos::union::ibc::lightclients::ethereum::v1::Proof {
    fn from(value: Proof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            proof: value.proof,
        }
    }
}
