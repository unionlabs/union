use serde::{Deserialize, Serialize};

use crate::{
    ethereum::H256, ibc::core::client::height::Height,
    tendermint::types::signed_header::SignedHeader, Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub signed_header: SignedHeader,
    pub untrusted_validator_set_root: H256,
    pub trusted_height: Height,
    #[serde(with = "::serde_utils::hex_string")]
    pub zero_knowledge_proof: Vec<u8>,
}

impl Proto for Header {
    type Proto = protos::union::ibc::lightclients::cometbls::v1::Header;
}

impl TypeUrl for protos::union::ibc::lightclients::cometbls::v1::Header {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.cometbls.v1.Header";
}

// #[derive(Debug)]
// pub enum TryFromHeaderError {}

// impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Header> for Header {
//     type Error = TryFromHeaderError;

//     fn try_from(
//         value: protos::union::ibc::lightclients::cometbls::v1::Header,
//     ) -> Result<Self, Self::Error> {
//         Ok(Self {
//             signed_header: required!(value.signed_header)?.into(),
//             untrusted_validator_set_root: todo!(),
//             trusted_height: todo!(),
//             zero_knowledge_proof: todo!(),
//         })
//     }
// }

impl From<Header> for protos::union::ibc::lightclients::cometbls::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            signed_header: Some(value.signed_header.into()),
            untrusted_validator_set_root: value.untrusted_validator_set_root.into(),
            trusted_height: Some(value.trusted_height.into()),
            zero_knowledge_proof: value.zero_knowledge_proof,
        }
    }
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

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Header {
    type EthAbi = contracts::glue::UnionIbcLightclientsCometblsV1HeaderData;
}
