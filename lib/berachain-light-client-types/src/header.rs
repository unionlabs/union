use beacon_api_types::deneb::ExecutionPayloadHeader;
use ethereum_light_client_types::AccountProof;
use unionlabs::ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub tm_header: tendermint_light_client_types::header::Header,

    pub l1_height: Height,
    pub execution_header: ExecutionPayloadHeader,
    pub execution_header_proof: MerkleProof,
    pub account_proof: AccountProof,
}
