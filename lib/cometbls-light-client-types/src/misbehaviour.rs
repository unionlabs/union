use crate::header::Header;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Misbehaviour {
    pub header_a: Header,
    pub header_b: Header,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{header, misbehaviour::Misbehaviour};

    impl_proto_via_try_from_into!(Misbehaviour => protos::union::ibc::lightclients::cometbls::v1::Misbehaviour);

    impl From<Misbehaviour> for protos::union::ibc::lightclients::cometbls::v1::Misbehaviour {
        fn from(value: Misbehaviour) -> Self {
            Self {
                header_a: Some(value.header_a.into()),
                header_b: Some(value.header_b.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        Header(#[from] header::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Misbehaviour> for Misbehaviour {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::Misbehaviour,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                header_a: required!(value.header_a)?.try_into()?,
                header_b: required!(value.header_b)?.try_into()?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bcs, Bincode, Json, Proto},
        google::protobuf::timestamp::Timestamp,
        ibc::core::client::height::Height,
        primitives::H256,
        test_utils::assert_codec_iso,
    };

    use super::*;
    use crate::LightHeader;

    fn mk_misbehaviour() -> Misbehaviour {
        Misbehaviour {
            header_a: Header {
                signed_header: LightHeader {
                    height: 0xDEADC0DE.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 1.try_into().unwrap(),
                        nanos: 1.try_into().unwrap(),
                    },
                    validators_hash: H256::from([0xAA; 32]),
                    next_validators_hash: H256::from([0xAA; 32]),
                    app_hash: H256::from([0xAA; 32]),
                },
                trusted_height: Height::new(100),
                zero_knowledge_proof: b"zkp".into(),
            },
            header_b: Header {
                signed_header: LightHeader {
                    height: 0xDEADC0DE.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 1.try_into().unwrap(),
                        nanos: 1.try_into().unwrap(),
                    },
                    validators_hash: H256::from([0xAA; 32]),
                    next_validators_hash: H256::from([0xAA; 32]),
                    app_hash: H256::from([0xAA; 32]),
                },
                trusted_height: Height::new(100),
                zero_knowledge_proof: b"zkp".into(),
            },
        }
    }

    // TODO: Implement misbehaviour on the solidity stack
    // #[test]
    // fn ethabi_iso() {
    //     assert_codec_iso::<_, EthAbi>(&mk_misbehaviour());
    // }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_misbehaviour());
    }

    #[test]
    fn bcs_iso() {
        assert_codec_iso::<_, Bcs>(&mk_misbehaviour());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_misbehaviour());
    }

    #[test]
    fn proto_iso() {
        assert_codec_iso::<_, Proto>(&mk_misbehaviour());
    }
}
