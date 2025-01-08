use unionlabs::{ibc::core::client::height::Height, primitives::Bytes};

use crate::light_header::LightHeader;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub signed_header: LightHeader,
    pub trusted_height: Height,
    pub zero_knowledge_proof: Bytes,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{header::Header, light_header};

    impl_proto_via_try_from_into!(Header => protos::union::ibc::lightclients::cometbls::v1::Header);

    impl From<Header> for protos::union::ibc::lightclients::cometbls::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                signed_header: Some(value.signed_header.into()),
                trusted_height: Some(value.trusted_height.into()),
                zero_knowledge_proof: value.zero_knowledge_proof.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        SignedHeader(#[from] light_header::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: required!(value.signed_header)?.try_into()?,
                trusted_height: required!(value.trusted_height)?.into(),
                zero_knowledge_proof: value.zero_knowledge_proof.into(),
            })
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::{
        bounded::{BoundedI32, BoundedI64, BoundedIntError},
        google::protobuf::timestamp::Timestamp,
        ibc::core::client::height::Height,
        impl_ethabi_via_try_from_into, TryFromEthAbiBytesErrorAlloy,
    };

    use crate::{Header, LightHeader};

    impl_ethabi_via_try_from_into!(Header => SolHeader);

    alloy::sol! {
        struct SolSignedHeader {
            uint64 height;
            uint64 secs;
            uint64 nanos;
            bytes32 validatorsHash;
            bytes32 nextValidatorsHash;
            bytes32 appHash;
        }

        struct SolHeader {
            SolSignedHeader signedHeader;
            uint64 trustedHeight;
            bytes zeroKnowledgeProof;
        }
    }

    impl From<Header> for SolHeader {
        fn from(value: Header) -> Self {
            SolHeader {
                signedHeader: SolSignedHeader {
                    height: value
                        .signed_header
                        .height
                        .inner()
                        .try_into()
                        .expect("value is >= 0 and <= i64::MAX; qed;"),
                    secs: value
                        .signed_header
                        .time
                        .seconds
                        .inner()
                        // TODO: Figure out a better way to represent these types other than by saturating
                        .clamp(0, i64::MAX)
                        .try_into()
                        .expect("value is >= 0 and <= i64::MAX; qed;"),
                    nanos: value
                        .signed_header
                        .time
                        .nanos
                        .inner()
                        .try_into()
                        .expect("value is >= 0 and <= i32::MAX; qed;"),
                    validatorsHash: value.signed_header.validators_hash.into(),
                    nextValidatorsHash: value.signed_header.next_validators_hash.into(),
                    appHash: value.signed_header.app_hash.into(),
                },
                trustedHeight: value.trusted_height.height(),
                zeroKnowledgeProof: value.zero_knowledge_proof.into(),
            }
        }
    }

    impl TryFrom<SolHeader> for Header {
        type Error = TryFromEthAbiBytesErrorAlloy<Error>;

        fn try_from(value: SolHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: LightHeader {
                    height: BoundedI64::new(value.signedHeader.height)
                        .map_err(Error::Height)
                        .map_err(TryFromEthAbiBytesErrorAlloy::Convert)?,
                    time: Timestamp {
                        seconds: BoundedI64::new(value.signedHeader.secs)
                            .map_err(Error::Secs)
                            .map_err(TryFromEthAbiBytesErrorAlloy::Convert)?,
                        nanos: BoundedI32::new(value.signedHeader.nanos)
                            .map_err(Error::Nanos)
                            .map_err(TryFromEthAbiBytesErrorAlloy::Convert)?,
                    },
                    validators_hash: value.signedHeader.validatorsHash.into(),
                    next_validators_hash: value.signedHeader.nextValidatorsHash.into(),
                    app_hash: value.signedHeader.appHash.into(),
                },
                trusted_height: Height::new(value.trustedHeight),
                zero_knowledge_proof: value.zeroKnowledgeProof.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid height")]
        Height(#[source] BoundedIntError<i64, u64>),
        #[error("invalid secs")]
        Secs(#[source] BoundedIntError<i64, u64>),
        #[error("invalid nanos")]
        Nanos(#[source] BoundedIntError<i32, u64>),
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bcs, Bincode, EthAbi, Json, Proto},
        google::protobuf::timestamp::Timestamp,
        primitives::H256,
        test_utils::assert_codec_iso,
    };

    use super::*;

    // pub for misbehaviour codec tests to use
    pub(crate) fn mk_header() -> Header {
        Header {
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
        }
    }

    #[test]
    fn ethabi_iso() {
        assert_codec_iso::<_, EthAbi>(&mk_header());
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_header());
    }

    #[test]
    fn bcs_iso() {
        assert_codec_iso::<_, Bcs>(&mk_header());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_header());
    }

    #[test]
    fn proto_iso() {
        assert_codec_iso::<_, Proto>(&mk_header());
    }
}
