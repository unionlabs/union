use unionlabs::{
    primitives::{Bytes, H256},
    tuple::AsTuple,
};

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum Extra {
    Legacy(LegacyExtra),
    Versioned(VersionedExtra),
}

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct LegacyExtra {
    /// ibc contract that is running on l2
    pub contract_address: H256,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum VersionedExtra {
    V1(VersionedExtraV1),
}

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct VersionedExtraV1 {
    pub store_key: Bytes,
    pub key_prefix_storage: Bytes,
}

#[cfg(feature = "ethabi")]
mod ethabi {
    use alloy::{
        dyn_abi::{abi::token::PackedSeqToken, DynSolValue, Word},
        primitives::U256,
        sol_types::SolValue,
    };
    use state_lens_light_client_types::client_state::ethabi::{DecodeExtra, ExtraToTokens};

    use super::*;
    use crate::client_state::VersionedExtraV1;

    const V1_VERSION_TAG: U256 = U256::ONE;

    impl DecodeExtra for Extra {
        fn decode_extra(
            decoder: &mut alloy::dyn_abi::Decoder,
        ) -> Result<Self, alloy::dyn_abi::Error> {
            let extra = U256::from_be_bytes((*decoder.take_word()?).into());

            let extra = if extra.is_zero() {
                let version = U256::from_be_bytes((*decoder.take_word()?).into());
                let state = decoder.decode::<PackedSeqToken>()?;
                let (store_key, key_prefix_storage) =
                    <(alloy::primitives::Bytes, alloy::primitives::Bytes)>::abi_decode_params(
                        state.as_slice(),
                        true,
                    )?;

                match version {
                    V1_VERSION_TAG => Extra::Versioned(VersionedExtra::V1(VersionedExtraV1 {
                        store_key: store_key.into(),
                        key_prefix_storage: key_prefix_storage.into(),
                    })),
                    _ => panic!(),
                }
            } else {
                Extra::Legacy(LegacyExtra {
                    contract_address: extra.to_be_bytes().into(),
                })
            };

            Ok(extra)
        }
    }

    impl ExtraToTokens for Extra {
        fn encode_extra_to_dyn_value(self) -> Vec<alloy::dyn_abi::DynSolValue> {
            match self {
                Extra::Legacy(legacy_extra) => {
                    vec![DynSolValue::FixedBytes(
                        legacy_extra.contract_address.into(),
                        32,
                    )]
                }
                Extra::Versioned(versioned_extra) => match versioned_extra {
                    VersionedExtra::V1(versioned_extra_v1) => vec![
                        // empty legacy contract address
                        DynSolValue::FixedBytes(Word::default(), 32),
                        V1_VERSION_TAG.into(),
                        (
                            versioned_extra_v1.store_key,
                            versioned_extra_v1.key_prefix_storage,
                        )
                            .abi_encode_params()
                            .into(),
                    ],
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex;
    use ibc_union_spec::ClientId;
    use unionlabs::{
        encoding::{EthAbi, Json},
        test_utils::assert_codec_iso_bytes,
    };

    use super::*;

    #[test]
    fn legacy_ethabi() {
        // voyager rpc -r voy.run client-state 17000 4 --height 3794906
        let bz = hex!(
            "00000000000000000000000000000000000000000000000000000000000000a0" // l2_chain_id offset (160, 5 slots)
            "0000000000000000000000000000000000000000000000000000000000000001" // l1_client_id
            "0000000000000000000000000000000000000000000000000000000000000006" // l2_client_id
            "00000000000000000000000000000000000000000000000000000000000e10cf" // l2_latest_height
            "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4" // extra `(bytes32,)`
            "000000000000000000000000000000000000000000000000000000000000000a" // l2_chain_id length (10)
            "62626e2d746573742d3500000000000000000000000000000000000000000000" // l2_chain_id data
        );

        let value = state_lens_light_client_types::ClientState::<Extra> {
            l2_chain_id: "bbn-test-5".to_string(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(6),
            l2_latest_height: 921807,
            extra: Extra::Legacy(LegacyExtra {
                contract_address: hex!(
                    "0xbcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            }),
        };

        assert_codec_iso_bytes::<_, EthAbi>(&value, &bz);
    }

    #[test]
    fn v1_ethabi() {
        let bz = hex!(
            "00000000000000000000000000000000000000000000000000000000000000e0" // l2_chain_id offset (224, 7 slots)
            "0000000000000000000000000000000000000000000000000000000000000001" // l1_client_id
            "0000000000000000000000000000000000000000000000000000000000000006" // l2_client_id
            "00000000000000000000000000000000000000000000000000000000000e10cf" // l2_latest_height
            "0000000000000000000000000000000000000000000000000000000000000000" // empty legacy contract address
            "0000000000000000000000000000000000000000000000000000000000000001" // version 1
            "0000000000000000000000000000000000000000000000000000000000000120" // encoded state offset (288, 9 slots)
            "000000000000000000000000000000000000000000000000000000000000000a" // l2_chain_id length (10)
            "62626e2d746573742d3500000000000000000000000000000000000000000000" // l2_chain_id data
                                                                               // encoded state
            "00000000000000000000000000000000000000000000000000000000000000e0" // encoded state byte length (224, 7 slots)
                                                                               // state
            "0000000000000000000000000000000000000000000000000000000000000040" // store_key offset (64, 2 slots)
            "0000000000000000000000000000000000000000000000000000000000000080" // key_prefix_storage offset (64, 2 slots)
            "0000000000000000000000000000000000000000000000000000000000000003" // store_key length (3)
            "65766d0000000000000000000000000000000000000000000000000000000000" // store_key data
            "0000000000000000000000000000000000000000000000000000000000000021" // key_prefix_storage length (33)
            "03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3b" // key_prefix_storage data
            "a400000000000000000000000000000000000000000000000000000000000000" // '''
        );

        let value = state_lens_light_client_types::ClientState::<Extra> {
            l2_chain_id: "bbn-test-5".to_string(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(6),
            l2_latest_height: 921807,
            extra: Extra::Versioned(VersionedExtra::V1(VersionedExtraV1 {
                store_key: b"evm".into(),
                key_prefix_storage: hex!(
                    "03"
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            })),
        };

        assert_codec_iso_bytes::<_, EthAbi>(&value, &bz);
    }

    #[test]
    fn legacy_json() {
        // voyager rpc -r voy.run client-state 17000 4 --height 3794906 --decode
        let json = r#"{"l2_chain_id":"bbn-test-5","l1_client_id":1,"l2_client_id":6,"l2_latest_height":923715,"contract_address":"0xbcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"}"#;

        let value = state_lens_light_client_types::ClientState::<Extra> {
            l2_chain_id: "bbn-test-5".to_string(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(6),
            l2_latest_height: 923715,
            extra: Extra::Legacy(LegacyExtra {
                contract_address: hex!(
                    "0xbcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            }),
        };

        assert_codec_iso_bytes::<_, Json>(&value, json.as_bytes());
    }

    #[test]
    fn v1_json() {
        let json = r#"{"l2_chain_id":"bbn-test-5","l1_client_id":1,"l2_client_id":6,"l2_latest_height":921807,"v1":{"store_key":"0x65766d","key_prefix_storage":"0x03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"}}"#;

        let value = state_lens_light_client_types::ClientState::<Extra> {
            l2_chain_id: "bbn-test-5".to_string(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(6),
            l2_latest_height: 921807,
            extra: Extra::Versioned(VersionedExtra::V1(VersionedExtraV1 {
                store_key: b"evm".into(),
                key_prefix_storage: hex!(
                    "03"
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            })),
        };

        assert_codec_iso_bytes::<_, Json>(&value, json.as_bytes());
    }
}
