/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0>
pub mod phase0;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair>
pub mod altair;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix>
pub mod bellatrix;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/capella>
pub mod capella;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb>
pub mod deneb;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/electra>
pub mod electra;

/// Newtype for beacon slots.
pub mod slot;

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#custom-types>
pub mod custom_types;

pub mod chain_spec;

/// Values that are constant across all configurations.
pub mod consts;

pub mod preset;
