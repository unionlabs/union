use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    /// l2 chain id
    pub l2_chain_id: String,
    /// l1 client id used to check the l2 inclusion proof against
    pub l1_client_id: u32,
    /// l2 client id
    pub l2_client_id: u32,
    /// l2 latest height
    pub l2_latest_height: u64,
    /// ibc contract that is running on l2
    pub contract_address: H256,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use core::str;
    use std::string::FromUtf8Error;

    use alloy::sol_types::SolValue;
    use unionlabs::{
        encoding::{Decode, Encode, EthAbi},
        TryFromEthAbiBytesErrorAlloy,
    };

    use crate::ClientState;

    alloy::sol! {
        struct SolClientState {
            string l2ChainId;
            uint32 l1ClientId;
            uint32 l2ClientId;
            uint64 l2LatestHeight;
            bytes32 contractAddress;
        }
    }

    impl Encode<EthAbi> for ClientState {
        fn encode(self) -> Vec<u8> {
            SolClientState {
                l2ChainId: self.l2_chain_id,
                l1ClientId: self.l1_client_id,
                l2ClientId: self.l2_client_id,
                l2LatestHeight: self.l2_latest_height,
                contractAddress: self.contract_address.into(),
            }
            .abi_encode_params()
        }
    }

    impl Decode<EthAbi> for ClientState {
        type Error = TryFromEthAbiBytesErrorAlloy<Error>;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            let client_state = SolClientState::abi_decode_params(bytes, true)?;

            Ok(Self {
                l2_chain_id: String::from_utf8(client_state.l2ChainId.into_bytes())
                    .map_err(|err| TryFromEthAbiBytesErrorAlloy::Convert(Error::ChainId(err)))?,
                l1_client_id: client_state.l1ClientId,
                l2_client_id: client_state.l2ClientId,
                l2_latest_height: client_state.l2LatestHeight,
                contract_address: client_state.contractAddress.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid chain_id")]
        ChainId(#[from] FromUtf8Error),
    }

    #[cfg(test)]
    mod test {
        fn test_decode() {
            // TODO(aeryz): impl
        }

        fn test_encode() {
            // TODO(aeryz): impl
        }
    }
}
