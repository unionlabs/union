pub mod client_state;
pub mod commitment;
pub mod consensus_state;
pub mod contract;
pub mod errors;
pub mod eth_encoding;
pub mod header;
pub mod misbehaviour;
pub mod msg;
pub mod state;
pub mod types;
pub mod update;

pub mod eth_types {
    #[cfg(not(feature = "eth-minimal"))]
    pub use ethereum_consensus::preset::mainnet::*;
    #[cfg(feature = "eth-minimal")]
    pub use ethereum_consensus::preset::minimal::*;

    pub const SYNC_COMMITTEE_SIZE: usize = PRESET.SYNC_COMMITTEE_SIZE;

    pub type LightClientUpdate = ethereum_consensus::capella::LightClientUpdate<
        SYNC_COMMITTEE_SIZE,
        { PRESET.BYTES_PER_LOGS_BLOOM },
        { PRESET.MAX_EXTRA_DATA_BYTES },
    >;

    pub type ConsensusUpdateInfo =
        ethereum_light_client_verifier::updates::capella::ConsensusUpdateInfo<
            SYNC_COMMITTEE_SIZE,
            { PRESET.BYTES_PER_LOGS_BLOOM },
            { PRESET.MAX_EXTRA_DATA_BYTES },
        >;

    pub type ExecutionUpdateInfo =
        ethereum_light_client_verifier::updates::capella::ExecutionUpdateInfo;

    pub type LightClientHeader = ethereum_consensus::capella::LightClientHeader<
        { PRESET.BYTES_PER_LOGS_BLOOM },
        { PRESET.MAX_EXTRA_DATA_BYTES },
    >;
}
