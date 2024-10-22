use macros::model;

#[cfg(feature = "ethabi")]
use crate::cometbft::types::{commit::TryFromEthAbiCommitError, header::TryFromEthAbiHeaderError};
use crate::{
    cometbft::types::{
        commit::{Commit, TryFromCommitError},
        header::{Header, TryFromHeaderError},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cometbft::types::v1::SignedHeader), into, from))]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}

impl From<SignedHeader> for protos::cometbft::types::v1::SignedHeader {
    fn from(value: SignedHeader) -> Self {
        Self {
            header: Some(value.header.into()),
            commit: Some(value.commit.into()),
        }
    }
}

impl From<SignedHeader> for protos::tendermint::types::SignedHeader {
    fn from(value: SignedHeader) -> Self {
        Self {
            header: Some(value.header.into()),
            commit: Some(value.commit.into()),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq)]
pub enum TryFromEthAbiSignedHeaderError {
    Header(TryFromEthAbiHeaderError),
    Commit(TryFromEthAbiCommitError),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesSignedHeaderData> for SignedHeader {
    type Error = TryFromEthAbiSignedHeaderError;

    fn try_from(
        value: contracts::glue::TendermintTypesSignedHeaderData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            header: value
                .header
                .try_into()
                .map_err(TryFromEthAbiSignedHeaderError::Header)?,
            commit: value
                .commit
                .try_into()
                .map_err(TryFromEthAbiSignedHeaderError::Commit)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromSignedHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid header")]
    Header(#[source] TryFromHeaderError),
    #[error("invalid commit")]
    Commit(#[source] TryFromCommitError),
}

impl TryFrom<protos::cometbft::types::v1::SignedHeader> for SignedHeader {
    type Error = TryFromSignedHeaderError;

    fn try_from(value: protos::cometbft::types::v1::SignedHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            header: required!(value.header)?
                .try_into()
                .map_err(TryFromSignedHeaderError::Header)?,
            commit: required!(value.commit)?
                .try_into()
                .map_err(TryFromSignedHeaderError::Commit)?,
        })
    }
}

impl TryFrom<protos::tendermint::types::SignedHeader> for SignedHeader {
    type Error = TryFromSignedHeaderError;

    fn try_from(value: protos::tendermint::types::SignedHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            header: required!(value.header)?
                .try_into()
                .map_err(TryFromSignedHeaderError::Header)?,
            commit: required!(value.commit)?
                .try_into()
                .map_err(TryFromSignedHeaderError::Commit)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<SignedHeader> for contracts::glue::TendermintTypesSignedHeaderData {
    fn from(value: SignedHeader) -> Self {
        Self {
            header: value.header.into(),
            commit: value.commit.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::*;
    use crate::test_utils::assert_proto_roundtrip;

    #[test]
    fn proto() {
        let bz = hex::decode("0a262f6962632e6c69676874636c69656e74732e74656e6465726d696e742e76312e48656164657212cb0d0a93070a98030a02080b12117374617267617a652d6465766e65742d3118e60a220c089cc9dbb80610ebedb8a9012a480a2001d6af5ef3745e4177f3fae39a5133864806940048547751043031d4be00e7ca1224080112206d4f2e4720fe74e406cfd67db6bc2a5cec58dbc76d187c3d8f466c6abc05229232207cbc92d5c9ecf44233353e9f476052032e5ea83cf5e45e27effed164e876a0383a20e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8554220e9438d0911cb20220fb44b90055c7f51f9e0752d9d4b79bfdd5a8d8c7b788d704a20e9438d0911cb20220fb44b90055c7f51f9e0752d9d4b79bfdd5a8d8c7b788d705220048091bc7ddc283f77bfbf91d73c44da58c3df8a9cbc867405d8b7f3daada22f5a20b5e0bfc976bc8d4f19a4f1e86325bbae7a27fef2a58bb10a621425b867fb168e6220452f2d0c43d3b922e2a34e8a0c4dc50182fd6ec5c148206d0574dd95ecd103db6a20e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8557214778d299a43e0a1c310712dbbe9a4539e59b4885112f50308e60a1a480a2009b1fa5b179a910a03e2a49faf8f5db91f85f049154660e2c8e30e9f309def2712240801122062d6127264d462530a1524516d65e25e1ae144dbd92fedd02ba4f5def05a2a442268080212146ad09a2b7581d91567553aa930b3be55716fdad21a0c08a1c9dbb80610e68ff79e032240f559d3b7153d0aeb40491362f89a0e4fa05f9e750dd68bdf263da830377f9907f9ce93cba3ec92c6afa87e9edaead869b533ae6a7931ac080ac22d7643f18f08226808021214778d299a43e0a1c310712dbbe9a4539e59b488511a0c08a1c9dbb80610c5f2c5cf032240a312ce33a949dd14b18224f8756d5c7c2c0b2e49fd7a8ea41564bbc9be949ab55c71b9cd2b5253b420fc7ce510d998143fea8ca93fa2a0332b32e50af8a99406226808021214866c00935e86ca8a54457c49d7b27ba5edd43a331a0c08a1c9dbb8061096e59ef0022240855776ed3f7748d61e8926752e98b580177e18d4d748c09a938014ef30e88a1ba1c52f903c1c29d6dccaec85e0a53fce5d3100e35e45f19bd11428ab675e020e226808021214fde94503e3b44960eeaa2dbc0a2e8366878533da1a0c08a1c9dbb80610d6e5ebf00222406e752f057bb07ba96ed348ad0e44c175b13ca3e73b0333928b81c37543dfbc4d154a6b8810ae3bea4e7294921a9bf1ec62622cf1a0e54ae71954ecb0dc0097091295030a4e0a146ad09a2b7581d91567553aa930b3be55716fdad212220a20dd2eb4784d175670fa2b08923bd3778419c61e1871279d821cb1d5bee37d9c691880809aa6eaafe301208080ccb3aba0b9fcff010a4e0a14778d299a43e0a1c310712dbbe9a4539e59b4885112220a209decdf08a308e409d51c48509bd6fe43417a029bf30869a5f0c8a9a3ca690ff91880809aa6eaafe301208080ccb3aba0b9fcff010a4c0a14866c00935e86ca8a54457c49d7b27ba5edd43a3312220a2039a091ff4d8e84ca6c18d3eecc552694a5ee41b18940b2fbd53a6ea9bfd514d61880809aa6eaafe301208080b4ccd4dfc6030a4c0a14fde94503e3b44960eeaa2dbc0a2e8366878533da12220a2010edba28fac09112f697f354c78680bbfb9fd28a5c843a34ff49fffafa9e8aff1880809aa6eaafe301208080b4ccd4dfc603124e0a14778d299a43e0a1c310712dbbe9a4539e59b4885112220a209decdf08a308e409d51c48509bd6fe43417a029bf30869a5f0c8a9a3ca690ff91880809aa6eaafe301208080ccb3aba0b9fcff01188080e898a9bf8d071a05080110cd082293030a4e0a146ad09a2b7581d91567553aa930b3be55716fdad212220a20dd2eb4784d175670fa2b08923bd3778419c61e1871279d821cb1d5bee37d9c691880809aa6eaafe301208080b28dc1f0d5faff010a4c0a14778d299a43e0a1c310712dbbe9a4539e59b4885112220a209decdf08a308e409d51c48509bd6fe43417a029bf30869a5f0c8a9a3ca690ff91880809aa6eaafe3012080809aa6eaafe3010a4c0a14866c00935e86ca8a54457c49d7b27ba5edd43a3312220a2039a091ff4d8e84ca6c18d3eecc552694a5ee41b18940b2fbd53a6ea9bfd514d61880809aa6eaafe3012080809aa6eaafe3010a4c0a14fde94503e3b44960eeaa2dbc0a2e8366878533da12220a2010edba28fac09112f697f354c78680bbfb9fd28a5c843a34ff49fffafa9e8aff1880809aa6eaafe3012080809aa6eaafe301124e0a146ad09a2b7581d91567553aa930b3be55716fdad212220a20dd2eb4784d175670fa2b08923bd3778419c61e1871279d821cb1d5bee37d9c691880809aa6eaafe301208080b28dc1f0d5faff01188080e898a9bf8d07").unwrap();

        let json = r#"
        {"version":{"block":"11"},"chain_id":"stargaze-devnet-1","height":"2811","time":"2024-10-22T12:14:09.844039795Z","last_block_id":{"hash":"E9F1ED2BC8565C94BD999E7D9A24F7B8D40AB20F65D5B0E3FFA59D643EDFD554","parts":{"total":1,"hash":"AE3F913F9E581891ACF43E81F09FC57059C229A68198727251A54FD59BADEB71"}},"last_commit_hash":"B9EFECDA542BABDBD4551FA18CD235F753B02AAACB875B99F8F0D9F6A10CDCD7","data_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","validators_hash":"E9438D0911CB20220FB44B90055C7F51F9E0752D9D4B79BFDD5A8D8C7B788D70","next_validators_hash":"E9438D0911CB20220FB44B90055C7F51F9E0752D9D4B79BFDD5A8D8C7B788D70","consensus_hash":"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F","app_hash":"AAB90A23ACD1C96F9008D185B10554E78ADBC13DA746B9C10354863BE8E84CFD","last_results_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","evidence_hash":"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855","proposer_address":"866C00935E86CA8A54457C49D7B27BA5EDD43A33"}
        "#;

        {
            // let header = protos::google::protobuf::Any {
            //     type_url: String::new(),
            //     value: protos::ibc::lightclients::tendermint::v1::Header {
            //         signed_header: Some(protos::tendermint::types::SignedHeader {
            //             header: Some(protos::tendermint::types::Header {
            //                 version: Some(protos::tendermint::version::Consensus {
            //                     block: 11,
            //                     app: 0,
            //                 }),
            //                 chain_id: "stargaze-devnet-1".into(),
            //                 height: 2811,
            //                 time: None,
            //                 last_block_id: None,
            //                 last_commit_hash: vec![],
            //                 data_hash: vec![],
            //                 validators_hash: vec![],
            //                 next_validators_hash: vec![],
            //                 consensus_hash: vec![],
            //                 app_hash: vec![],
            //                 last_results_hash: vec![],
            //                 evidence_hash: vec![],
            //                 proposer_address: vec![],
            //             }),
            //             commit: None,
            //         }),
            //         validator_set: None,
            //         trusted_height: None,
            //         trusted_validators: None,
            //     }
            //     .encode_to_vec()
            //     .into(),
            // };

            // let header = serde_json::from_str::<protos::cometbft::types::v1::Header>(json).unwrap();

            // dbg!(&header);

            // let bz = header.encode_to_vec();

            // let tm = protos::tendermint::types::SignedHeader::decode(&*bz).unwrap();

            let cmt = protos::google::protobuf::Any::decode(&*bz).unwrap();

            let t = protos::ibc::lightclients::tendermint::v1::Header::decode(&*cmt.value).unwrap();

            dbg!(&t);
        }

        // {
        //     let version = protos::cometbft::version::v1::Consensus { block: 1, app: 0 };

        //     dbg!(&version);

        //     let bz = version.encode_to_vec();

        //     protos::cometbft::version::v1::Consensus::decode(&*bz).unwrap();
        // }
    }
}
