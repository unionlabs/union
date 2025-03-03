use beacon_api_types::{altair::SyncCommittee, chain_spec::PresetBaseKind};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, H256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "version", content = "data", rename_all = "snake_case") // mirror the beacon api
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: u64,
    pub chain_spec: PresetBaseKind,
    pub genesis_validators_root: H256,
    pub genesis_time: u64,
    pub latest_height: u64,
    pub frozen_height: Height,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub initial_sync_committee: Option<InitialSyncCommittee>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct InitialSyncCommittee {
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bincode, Json},
        primitives::H256,
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_client_state() -> ClientState {
        ClientState::V1(ClientStateV1 {
            chain_id: 1,
            chain_spec: PresetBaseKind::Minimal,
            genesis_validators_root: H256::new([0xAA; 32]),
            genesis_time: 123,
            latest_height: 987,
            frozen_height: Height::new(1),
            ibc_contract_address: H160::new([0xAA; 20]),
            initial_sync_committee: None,
        })
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_client_state());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_client_state());
    }
}
