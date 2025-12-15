use ibc_union_spec::ClientId;
use starknet_types::Felt;
use unionlabs_primitives::H160;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "version", content = "data", rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: Felt,
    pub l1_client_id: ClientId,
    pub latest_height: u64,
    pub ibc_contract_address: Felt,
    /// <https://docs.starknet.io/learn/cheatsheets/chain-info#important-addresses>
    ///
    /// Mainnet: `0xc662c410C0ECf747543f5bA90660f6ABeBD9C8c4`
    /// Sepolia: `0xE2Bb56ee936fd6433DC0F6e7e3b8365C906AA057`
    pub l1_contract_address: H160,
}
