pub use crate::bellatrix::{
    beacon_block::BeaconBlock, beacon_block_body::BeaconBlockBody,
    execution_payload::ExecutionPayload, execution_payload_header::ExecutionPayloadHeader,
    light_client_bootstrap::LightClientBootstrap,
    light_client_finality_update::LightClientFinalityUpdate,
    light_client_header::LightClientHeader, light_client_update::LightClientUpdate,
    signed_beacon_block::SignedBeaconBlock,
};
#[cfg(feature = "ssz")]
pub use crate::bellatrix::{
    beacon_block::BeaconBlockSsz, beacon_block_body::BeaconBlockBodySsz,
    execution_payload::ExecutionPayloadSsz, execution_payload_header::ExecutionPayloadHeaderSsz,
    signed_beacon_block::SignedBeaconBlockSsz,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody>
pub mod beacon_block_body;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayload>
pub mod execution_payload;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayloadheader>
pub mod execution_payload_header;

/// Updated indirectly.
pub mod beacon_block;

/// Updated indirectly.
pub mod signed_beacon_block;

/// Updated indirectly.
pub mod light_client_header;

/// Updated indirectly.
pub mod light_client_update;

/// Updated indirectly.
pub mod light_client_bootstrap;

/// Updated indirectly.
pub mod light_client_finality_update;
