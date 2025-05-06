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
    use alloy::sol_types::SolValue;
    use state_lens_light_client_types::client_state::ethabi::{DecodeExtra, ExtraToTokens};

    use super::*;

    impl DecodeExtra for Extra {
        fn decode_extra(
            decoder: &mut alloy::dyn_abi::Decoder,
        ) -> Result<Self, alloy::dyn_abi::Error> {
            Ok(Self {
                timestamp_offset: u16::detokenize(decoder.take_word()?.into()),
                state_root_offset: u16::detokenize(decoder.take_word()?.into()),
                storage_root_offset: u16::detokenize(decoder.take_word()?.into()),
            })
        }
    }

    impl ExtraToTokens for Extra {
        fn encode_extra_to_dyn_value(self) -> Vec<alloy::dyn_abi::DynSolValue> {
            vec![
                self.timestamp_offset.into(),
                self.state_root_offset.into(),
                self.storage_root_offset.into(),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use ibc_union_spec::ClientId;
    use unionlabs::{
        encoding::{Bincode, EthAbi},
        test_utils::assert_codec_iso_bytes,
    };

    use super::*;

    #[test]
    fn bincode() {
        // voyager rpc -r voy.run client-state 17000 3
        let bz = hex!(
            "0800000000000000" // l2_chain_id length (u64, 8)
            "3231303030303031" // l2_chain_id ("21000001")
            "01000000"         // l1_client_id (u32, 1)
            "07000000"         // l2_client_id (u32, 7)
            "489a030000000000" // l2_latest_height (u32, 1)
            "5800"             // timestamp_offset (u16, 88)
            "0000"             // state_root_offset (u16, 0)
            "2000"             // storage_root_offset (u16, 32)
        );

        let value = ClientState {
            l2_chain_id: "21000001".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(7),
            l2_latest_height: 236104,
            extra: Extra {
                timestamp_offset: 88,
                state_root_offset: 0,
                storage_root_offset: 32,
            },
        };

        assert_codec_iso_bytes::<_, Bincode>(&value, &bz);
    }

    #[test]
    fn ethabi() {
        // voyager rpc -r voy.run client-state 17000 3
        let bz = hex!(
            "00000000000000000000000000000000000000000000000000000000000000e0" // l2_chain_id offset (224, 7 slots)
            "0000000000000000000000000000000000000000000000000000000000000001" // l1_client_id
            "0000000000000000000000000000000000000000000000000000000000000001" // l2_client_id
            "00000000000000000000000000000000000000000000000000000000007e36fd" // l2_latest_height
            "0000000000000000000000000000000000000000000000000000000000000078" // timestamp_offset
            "0000000000000000000000000000000000000000000000000000000000000020" // state_root_offset
            "0000000000000000000000000000000000000000000000000000000000000040" // storage_root_offset
            "0000000000000000000000000000000000000000000000000000000000000008" // l2_chain_id length (8)
            "3131313535313131000000000000000000000000000000000000000000000000" // l2_chain_id data
        );

        let value = ClientState {
            l2_chain_id: "11155111".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(1),
            l2_latest_height: 8271613,
            extra: Extra {
                timestamp_offset: 120,
                state_root_offset: 32,
                storage_root_offset: 64,
            },
        };

        assert_codec_iso_bytes::<_, EthAbi>(&value, &bz);
    }
}
