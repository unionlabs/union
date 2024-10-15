use macros::model;

use crate::ibc::{
    core::client::height::Height, lightclients::cometbls::signed_header::SignedHeader,
};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::Header),
        into,
        from
    ),
    ethabi(raw(ibc_solidity::cometbls::Header), into, from)
)]
pub struct Header {
    pub signed_header: SignedHeader,
    pub trusted_height: Height,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug("{}", ::serde_utils::to_hex(&zero_knowledge_proof))]
    pub zero_knowledge_proof: Vec<u8>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField},
        ibc::lightclients::cometbls::{
            header::Header, signed_header::proto::TryFromSignedHeaderError,
        },
    };

    impl From<Header> for protos::union::ibc::lightclients::cometbls::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                signed_header: Some(value.signed_header.into()),
                trusted_height: Some(value.trusted_height.into()),
                zero_knowledge_proof: value.zero_knowledge_proof,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromHeaderError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid signed header")]
        SignedHeader(#[from] TryFromSignedHeaderError),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Header> for Header {
        type Error = TryFromHeaderError;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: required!(value.signed_header)?.try_into()?,
                trusted_height: required!(value.trusted_height)?.into(),
                zero_knowledge_proof: value.zero_knowledge_proof,
            })
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use crate::ibc::{
        core::client::height::Height,
        lightclients::cometbls::{
            header::Header, signed_header::ethabi::TryFromEthAbiSignedHeaderError,
        },
    };

    impl From<Header> for ibc_solidity::cometbls::Header {
        fn from(value: Header) -> Self {
            Self {
                signedHeader: value.signed_header.into(),
                trustedHeight: value.trusted_height.revision(),
                zeroKnowledgeProof: value.zero_knowledge_proof.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TryFromEthAbiHeaderError {
        SignedHeader(TryFromEthAbiSignedHeaderError),
    }

    impl TryFrom<ibc_solidity::cometbls::Header> for Header {
        type Error = TryFromEthAbiHeaderError;

        fn try_from(value: ibc_solidity::cometbls::Header) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: value
                    .signedHeader
                    .try_into()
                    .map_err(TryFromEthAbiHeaderError::SignedHeader)?,
                trusted_height: Height::new(value.trustedHeight),
                zero_knowledge_proof: value.zeroKnowledgeProof.to_vec(),
            })
        }
    }
}

#[cfg(feature = "bcs")]
#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use serde_utils::Hex;

    use super::*;
    use crate::{
        encoding::{Bcs, DecodeAs, EncodeAs},
        hash::H256,
    };

    #[test]
    fn bcs() {
        let bz = hex!("630e0000000000002239df6600000000406dcb0b2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500dfc45a4f41582fbfc1e3b7f79b0fd39d5f738133e68fdd47468fb037b0a44c9da01000000000000001d0900000000000080031c9bc15a0c4541aff1d12780d6cf4ae2bdc6e3afafceae9d4fa36209fa323b68002e9c77c223d830e5df6a80cdd683f0986353933ee3179970fccc5d893219d30726f3b8c0dbe630b815b01b5557228a0dfeb0e0435bb0d15d1ccff7f6133fc110937d9fceee2f9052468c198fafeca89d524142a0efa9dc4df445853ce617302059018fef03dc34456ad201d2a5420a7d1c8fac57cb48cbe6709ac4da27d1eb250f73eab007d26cbff41ceb4564ab1cdfa83e9ee88be4f816dc841bbf2e90c80186ad9437fce7655c71b54addae1ccea429da3edba3232d073cb7e89ff2d27218556f1af0c446962ace932f637279dd0ad3ef1501fb6da39d5f68282f54bcf6094999672f3d8cbbf0409aef1048175ffff50b03a5154016d307a2ef425ffee509cd447b22ce6331c7a3473b2c6da1f9d550e8c3ab19bde65e699e07f4f2886c03ec4ff2faa0e342de7ac5daf32025acd6070c19ed8b007c121db0d955472c7d2e38d5a943d15bc902613029e4baa8c26034ff280e3a4d5468fcd6745afe53b5");

        let header = Header {
            signed_header: SignedHeader {
                height: 3683.try_into().unwrap(),
                time: "2024-09-09T18:06:26.197881152Z".parse().unwrap(),
                validators_hash: H256::new(hex!(
                    "2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d"
                )),
                next_validators_hash: H256::new(hex!(
                    "2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d"
                )),
                app_hash: H256::new(hex!(
                    "fc45a4f41582fbfc1e3b7f79b0fd39d5f738133e68fdd47468fb037b0a44c9da"
                )),
            },
            trusted_height: Height::new_with_revision(1, 2333),
            zero_knowledge_proof: hex!("1c9bc15a0c4541aff1d12780d6cf4ae2bdc6e3afafceae9d4fa36209fa323b68002e9c77c223d830e5df6a80cdd683f0986353933ee3179970fccc5d893219d30726f3b8c0dbe630b815b01b5557228a0dfeb0e0435bb0d15d1ccff7f6133fc110937d9fceee2f9052468c198fafeca89d524142a0efa9dc4df445853ce617302059018fef03dc34456ad201d2a5420a7d1c8fac57cb48cbe6709ac4da27d1eb250f73eab007d26cbff41ceb4564ab1cdfa83e9ee88be4f816dc841bbf2e90c80186ad9437fce7655c71b54addae1ccea429da3edba3232d073cb7e89ff2d27218556f1af0c446962ace932f637279dd0ad3ef1501fb6da39d5f68282f54bcf6094999672f3d8cbbf0409aef1048175ffff50b03a5154016d307a2ef425ffee509cd447b22ce6331c7a3473b2c6da1f9d550e8c3ab19bde65e699e07f4f2886c03ec4ff2faa0e342de7ac5daf32025acd6070c19ed8b007c121db0d955472c7d2e38d5a943d15bc902613029e4baa8c26034ff280e3a4d5468fcd6745afe53b5").to_vec(),
        };

        dbg!(&header);

        let header_bz = header.encode_as::<Bcs>();

        dbg!(Hex(&header_bz));

        let header = Header::decode_as::<Bcs>(&bz).unwrap();

        dbg!(header);
    }
}
