use unionlabs::{primitives::H256, tuple::AsTuple};

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Extra {
    /// ibc contract that is running on l2
    pub contract_address: H256,
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

        type Token<'a> = <<(H256,) as SolValue>::SolType as SolType>::Token<'a>;

        const SOL_NAME: &'static str = "Extra";

        const ENCODED_SIZE: Option<usize> = None;

        const PACKED_ENCODED_SIZE: Option<usize> = None;

        fn valid_token(_token: &Self::Token<'_>) -> bool {
            true
        }

        fn detokenize((contract_address,): Self::Token<'_>) -> Self::RustType {
            Self {
                contract_address: <<H256 as SolValue>::SolType as SolType>::detokenize(
                    contract_address,
                ),
            }
        }
    }

    impl SolValue for Extra {
        type SolType = Self;
    }

    impl SolTypeValue<Self> for Extra {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (<<H256 as SolValue>::SolType as SolType>::tokenize(
                &self.contract_address,
            ),)
        }

        fn stv_abi_encode_packed_to(&self, _out: &mut Vec<u8>) {
            todo!()
        }

        fn stv_eip712_data_word(&self) -> alloy::sol_types::Word {
            todo!()
        }
    }
}
