pub use crate::altair::{
    light_client_bootstrap::LightClientBootstrap,
    light_client_finality_update::LightClientFinalityUpdate,
    light_client_header::LightClientHeader, light_client_update::LightClientUpdate,
    sync_aggregate::SyncAggregate, sync_committee::SyncCommittee,
};
#[cfg(feature = "ssz")]
pub use crate::altair::{sync_aggregate::SyncAggregateSsz, sync_committee::SyncCommitteeSsz};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap>
pub mod light_client_bootstrap;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientfinalityupdate>
pub mod light_client_finality_update;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientheader>
pub mod light_client_header;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientupdate>
pub mod light_client_update;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#syncaggregate>
pub mod sync_aggregate;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#synccommittee>
pub mod sync_committee;
