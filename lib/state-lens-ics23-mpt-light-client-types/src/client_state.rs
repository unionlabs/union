use unionlabs::tuple::AsTuple;

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Extra {
    /// the offset at which we extract the u64 timestamp from the l2 consensus state
    /// timestamp = consensus_state[timestamp_offset:timestamp_offset+8]
    pub timestamp_offset: u16,
    /// the offset at which we extract the bytes32 state root from the l2 consensus state
    /// state_root = consensus_state[state_root_offset:state_root_offset+32]
    pub state_root_offset: u16,
    /// the offset at which we extract the bytes32 storage root (of the ibc contract on the l2) from the l2 consensus state
    /// storage_root = consensus_state[storage_root_offset:storage_root_offset+32]
    pub storage_root_offset: u16,
}

#[cfg(feature = "ethabi")]
mod ethabi {
    use alloy::{
        dyn_abi::SolType,
        sol_types::{private::SolTypeValue, SolValue},
    };

    use super::*;

    impl SolType for Extra {
        type RustType = Self;

        type Token<'a> = <<<Self as AsTuple>::Tuple as SolValue>::SolType as SolType>::Token<'a>;

        const SOL_NAME: &'static str = "Extra";

        const ENCODED_SIZE: Option<usize> = None;

        const PACKED_ENCODED_SIZE: Option<usize> = None;

        fn valid_token(_token: &Self::Token<'_>) -> bool {
            true
        }

        fn detokenize(
            (timestamp_offset, state_root_offset, storage_root_offset): Self::Token<'_>,
        ) -> Self::RustType {
            Self {
                timestamp_offset: <<u16 as SolValue>::SolType as SolType>::detokenize(
                    timestamp_offset,
                ),
                state_root_offset: <<u16 as SolValue>::SolType as SolType>::detokenize(
                    state_root_offset,
                ),
                storage_root_offset: <<u16 as SolValue>::SolType as SolType>::detokenize(
                    storage_root_offset,
                ),
            }
        }
    }

    impl SolValue for Extra {
        type SolType = Self;
    }

    impl SolTypeValue<Self> for Extra {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (
                <<u16 as SolValue>::SolType as SolType>::tokenize(&self.timestamp_offset),
                <<u16 as SolValue>::SolType as SolType>::tokenize(&self.state_root_offset),
                <<u16 as SolValue>::SolType as SolType>::tokenize(&self.storage_root_offset),
            )
        }

        fn stv_abi_encode_packed_to(&self, _out: &mut Vec<u8>) {
            todo!()
        }

        fn stv_eip712_data_word(&self) -> alloy::sol_types::Word {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::{dyn_abi::SolType, hex};
    use unionlabs::{
        encoding::{Bincode, EthAbi},
        test_utils::assert_codec_iso_bytes,
    };

    use super::*;

    #[test]
    fn ethabi() {
        alloy::sol! {
            struct SolClientState {
                string l2ChainId;
                uint32 l1ClientId;
                uint32 l2ClientId;
                uint64 l2LatestHeight;
                uint16 timestampOffset;
                uint16 stateRootOffset;
                uint16 storageRootOffset;
            }
        }

        let bz = alloy::hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000072734500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000083131313535313131000000000000000000000000000000000000000000000000").unwrap();

        SolClientState::abi_decode(&bz, true).unwrap();
        assert!(SolClientState::abi_decode_params(&bz, true).is_err());
    }

    #[test]
    fn ethabi_encoding() {
        let bz = hex!("0x00000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000007d22b100000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000083131313535313131000000000000000000000000000000000000000000000000");

        assert_codec_iso_bytes::<_, EthAbi>(
            &ClientState {
                l2_chain_id: "11155111".to_owned(),
                l1_client_id: 1_u32.try_into().unwrap(),
                l2_client_id: 1_u32.try_into().unwrap(),
                l2_latest_height: 8200881,
                extra: Extra {
                    timestamp_offset: 120,
                    state_root_offset: 32,
                    storage_root_offset: 64,
                },
            },
            &bz,
        );
    }

    #[test]
    fn bincode_encoding() {
        let bz =
            hex!("0x0800000000000000313131353531313101000000010000000c217d0000000000780020004000");

        assert_codec_iso_bytes::<_, Bincode>(
            &ClientState {
                l2_chain_id: "11155111".to_owned(),
                l1_client_id: 1_u32.try_into().unwrap(),
                l2_client_id: 1_u32.try_into().unwrap(),
                l2_latest_height: 8200460,
                extra: Extra {
                    timestamp_offset: 120,
                    state_root_offset: 32,
                    storage_root_offset: 64,
                },
            },
            &bz,
        );
    }
}
