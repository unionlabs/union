use unionlabs::{
    primitives::{encoding::HexUnprefixed, H256},
    tuple::AsTuple,
};

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// TODO: Make this a versioned enum (should be trivial, I don't think this is used anywhere yet)
pub struct Extra {
    pub table_handle: H256<HexUnprefixed>,
}

#[cfg(feature = "ethabi")]
mod ethabi {
    use alloy::dyn_abi::DynSolValue;
    use state_lens_light_client_types::client_state::ethabi::{DecodeExtra, ExtraToTokens};

    use super::*;

    impl DecodeExtra for Extra {
        fn decode_extra(
            decoder: &mut alloy::dyn_abi::Decoder,
        ) -> Result<Self, alloy::dyn_abi::Error> {
            Ok(Self {
                table_handle: (*decoder.take_word()?).into(),
            })
        }
    }

    impl ExtraToTokens for Extra {
        fn encode_extra_to_dyn_value(self) -> Vec<alloy::dyn_abi::DynSolValue> {
            vec![DynSolValue::FixedBytes(self.table_handle.into(), 32)]
        }
    }
}
