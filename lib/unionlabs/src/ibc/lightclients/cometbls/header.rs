use serde::{Deserialize, Serialize};

use crate::{
    ethereum::H256, ibc::core::client::height::Height,
    tendermint::types::signed_header::SignedHeader,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub signed_header: SignedHeader,
    pub untrusted_validator_set_root: H256,
    pub trusted_height: Height,
    #[serde(with = "::serde_utils::hex_string")]
    pub zero_knowledge_proof: Vec<u8>,
}

#[cfg(feature = "ethabi")]
impl From<Header> for contracts::glue::UnionIbcLightclientsCometblsV1HeaderData {
    fn from(value: Header) -> Self {
        Self {
            signed_header: value.signed_header.into(),
            untrusted_validator_set_root: value.untrusted_validator_set_root.into(),
            trusted_height: value.trusted_height.into(),
            zero_knowledge_proof: value.zero_knowledge_proof.into(),
        }
    }
}

// #[cfg(feature = "ethabi")]
// impl From<contracts::glue::UnionIbcLightclientsCometblsV1HeaderData> for Header {
//     fn from(value: contracts::glue::UnionIbcLightclientsCometblsV1HeaderData) -> Self {
//         Self {
//             signed_header: value.signed_header.into(),
//             untrusted_validator_set_root: value.untrusted_validator_set_root.into(),
//             trusted_height: value.trusted_height.into(),
//             zero_knowledge_proof: value.zero_knowledge_proof.into(),
//         }
//     }
// }

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Header {
    type EthAbi = contracts::glue::UnionIbcLightclientsCometblsV1HeaderData;
}
