use beacon_api_types::execution_payload_header::ExecutionPayloadHeader;
use ethereum_light_client_types::AccountProof;
use unionlabs::ibc::core::commitment::merkle_proof::MerkleProof;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub cometbft_header: tendermint_light_client_types::Header,
    pub execution_header: ExecutionPayloadHeader,
    pub execution_header_proof: MerkleProof,
    pub account_proof: AccountProof,
}
