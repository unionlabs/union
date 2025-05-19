use unionlabs::{primitives::Bytes, tuple::AsTuple};

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Extra {
    V1(ExtraV1),
}

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct ExtraV1 {
    pub store_key: Bytes,
    pub key_prefix_storage: Bytes,
}

#[cfg(feature = "ethabi")]
mod ethabi {
    use alloy::{dyn_abi::abi::token::PackedSeqToken, primitives::U256, sol_types::SolValue};
    use state_lens_light_client_types::client_state::ethabi::{DecodeExtra, ExtraToTokens};

    use super::*;
    use crate::client_state::ExtraV1;

    const V1_VERSION_TAG: U256 = U256::ONE;

    impl DecodeExtra for Extra {
        fn decode_extra(
            decoder: &mut alloy::dyn_abi::Decoder,
        ) -> Result<Self, alloy::dyn_abi::Error> {
            let version = U256::from_be_bytes((*decoder.take_word()?).into());
            let state = decoder.decode::<PackedSeqToken>()?;
            let (store_key, key_prefix_storage) =
                <(alloy::primitives::Bytes, alloy::primitives::Bytes)>::abi_decode_params(
                    state.as_slice(),
                    true,
                )?;

            match version {
                V1_VERSION_TAG => Ok(Extra::V1(ExtraV1 {
                    store_key: store_key.into(),
                    key_prefix_storage: key_prefix_storage.into(),
                })),
                _ => Err(alloy::dyn_abi::Error::custom(format!(
                    "invalid version: {version}"
                ))),
            }
        }
    }

    impl ExtraToTokens for Extra {
        fn encode_extra_to_dyn_value(self) -> Vec<alloy::dyn_abi::DynSolValue> {
            match self {
                Extra::V1(versioned_extra_v1) => vec![
                    V1_VERSION_TAG.into(),
                    (
                        versioned_extra_v1.store_key,
                        versioned_extra_v1.key_prefix_storage,
                    )
                        .abi_encode_params()
                        .into(),
                ],
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
    fn v1_ethabi() {
        // voyager rpc client-state 11155111 5 --height 8363356
        let bz = hex!(
            "00000000000000000000000000000000000000000000000000000000000000c0" // l2_chain_id offset (192, 6 slots)
            "0000000000000000000000000000000000000000000000000000000000000001" // l1_client_id
            "0000000000000000000000000000000000000000000000000000000000000006" // l2_client_id
            "00000000000000000000000000000000000000000000000000000000000fa2a1" // l2_latest_height
            "0000000000000000000000000000000000000000000000000000000000000001" // version 1
            "0000000000000000000000000000000000000000000000000000000000000100" // encoded state offset (256, 8 slots)
            "000000000000000000000000000000000000000000000000000000000000000a" // l2_chain_id length (10)
            "62626e2d746573742d3500000000000000000000000000000000000000000000" // l2_chain_id data
                                                                               // encoded state
            "00000000000000000000000000000000000000000000000000000000000000e0" // encoded state byte length (224, 7 slots)
                                                                               // state
            "0000000000000000000000000000000000000000000000000000000000000040" // store_key offset (64, 2 slots)
            "0000000000000000000000000000000000000000000000000000000000000080" // key_prefix_storage offset (64, 2 slots)
            "0000000000000000000000000000000000000000000000000000000000000004" // store_key length (4)
            "7761736d00000000000000000000000000000000000000000000000000000000" // store_key data
            "0000000000000000000000000000000000000000000000000000000000000022" // key_prefix_storage length (34, note that there is a trailing zero byte)
            "03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3b" // key_prefix_storage data
            "a400000000000000000000000000000000000000000000000000000000000000" // '''
        );

        let value = state_lens_light_client_types::ClientState::<Extra> {
            l2_chain_id: "bbn-test-5".to_string(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(6),
            l2_latest_height: 1024673,
            extra: Extra::V1(ExtraV1 {
                store_key: b"wasm".into(),
                key_prefix_storage: hex!(
                    "03"
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                    "00"
                )
                .into(),
            }),
        };

        assert_codec_iso_bytes::<_, EthAbi>(&value, &bz);
    }

    #[test]
    fn v1_json() {
        // voyager rpc client-state 11155111 5 --height 8363356
        let json = r#"{"l2_chain_id":"bbn-test-5","l1_client_id":1,"l2_client_id":6,"l2_latest_height":1024673,"v1":{"store_key":"0x7761736d","key_prefix_storage":"0x03bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba400"}}"#;

        let value = state_lens_light_client_types::ClientState::<Extra> {
            l2_chain_id: "bbn-test-5".to_string(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(6),
            l2_latest_height: 1024673,
            extra: Extra::V1(ExtraV1 {
                store_key: b"wasm".into(),
                key_prefix_storage: hex!(
                    "03"
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                    "00"
                )
                .into(),
            }),
        };

        assert_codec_iso_bytes::<_, Json>(&value, json.as_bytes());
    }
}
