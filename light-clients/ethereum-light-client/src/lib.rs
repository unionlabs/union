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
    // #[cfg(not(feature = "eth-minimal"))]
    // pub use ethereum_consensus::preset::mainnet::*;
    // #[cfg(feature = "eth-minimal")]
    pub use ethereum_consensus::capella::minimal::*;

    pub type LightClientHeader = ethereum_consensus::capella::minimal::LightClientHeader;
    pub type LightClientUpdate = ethereum_consensus::capella::minimal::LightClientUpdate;
}
