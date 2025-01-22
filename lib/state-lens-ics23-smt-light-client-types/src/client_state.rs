use unionlabs::tuple::AsTuple;

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Extra {
    /// the offset at which we extract the u64 timestamp from the l2 consensus state
    /// timestamp = consensus_state[timestamp_offset:timestamp_offset+8]
    pub timestamp_offset: u16,
    /// the offset at which we extract the bytes32 storage root (of the ibc contract on the l2) from the l2 consensus state
    /// state_root = consensus_state[state_root_offset:state_root_offset+32]
    pub state_root_offset: u16,
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

        fn detokenize((timestamp_offset, state_root_offset): Self::Token<'_>) -> Self::RustType {
            Self {
                timestamp_offset: <<u16 as SolValue>::SolType as SolType>::detokenize(
                    timestamp_offset,
                ),
                state_root_offset: <<u16 as SolValue>::SolType as SolType>::detokenize(
                    state_root_offset,
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
