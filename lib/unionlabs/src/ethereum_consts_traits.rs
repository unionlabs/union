use core::fmt::Debug;
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use typenum::Unsigned;

use crate::{
    ethereum::Version,
    ibc::lightclients::ethereum::{fork::Fork, fork_parameters::ForkParameters},
};

/// Minimal config.
#[derive(Debug, Clone, PartialEq)]
pub struct Minimal;

/// Mainnet config.
#[derive(Debug, Clone, PartialEq)]
pub struct Mainnet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetBaseKind {
    Mainnet,
    Minimal,
}

impl FromStr for PresetBaseKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minimal" => Ok(Self::Minimal),
            "mainnet" => Ok(Self::Mainnet),
            _ => Err(s.to_string()),
        }
    }
}

/// A way to emulate HKTs in the context of [`ChainSpec`]s.
///
/// # Example
///
/// ```rs
/// struct Foo<C: ChainSpec>(PhantomData<C>);
///
/// struct AnyFoo;
///
/// impl ChainSpecParameterizable for AnyFoo {
///     type T<C: ChainSpec> = Foo<C>;
/// }
///
/// struct Bar {
///     foo: AnyChainSpec<AnyFoo>,
/// }
/// ```
pub trait ChainSpecParameterizable {
    type T<C: ChainSpec>;
}

pub enum AnyChainSpec<T: ChainSpecParameterizable> {
    Mainnet(T::T<Mainnet>),
    Minimal(T::T<Minimal>),
}

impl<T: ChainSpecParameterizable> Debug for AnyChainSpec<T>
where
    T::T<Mainnet>: Debug,
    T::T<Minimal>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnyChainSpec::Mainnet(t) => f.debug_tuple("Mainnet").field(t).finish(),
            AnyChainSpec::Minimal(t) => f.debug_tuple("Minimal").field(t).finish(),
        }
    }
}

macro_rules! consts_traits {
    ($($CONST:ident $(,)?),+) => {
        $(
            #[allow(non_camel_case_types)]
            pub trait $CONST: 'static {
                // Extra traits are required because the builtin derives bound all generic
                // types unconditionally
                type $CONST: Unsigned + Debug + Clone + PartialEq + Send + Sync;
            }

            impl $CONST for Minimal {
                type $CONST = typenum::U<{ preset::MINIMAL.$CONST }>;
            }

            impl $CONST for Mainnet {
                type $CONST = typenum::U<{ preset::MAINNET.$CONST }>;
            }
        )+

        pub trait ChainSpec: 'static + Debug + Clone + PartialEq + Send + Sync + $($CONST+)+ {
            const PRESET: preset::Preset;
            const PRESET_BASE_KIND: PresetBaseKind;

            type PERIOD: 'static + Unsigned;
        }

        impl ChainSpec for Minimal {
            const PRESET: preset::Preset = preset::MINIMAL;
            const PRESET_BASE_KIND: PresetBaseKind = PresetBaseKind::Minimal;

            type PERIOD = typenum::Prod<
                <Self as EPOCHS_PER_SYNC_COMMITTEE_PERIOD>::EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
                <Self as SLOTS_PER_EPOCH>::SLOTS_PER_EPOCH,
            >;
        }

        impl ChainSpec for Mainnet {
            const PRESET: preset::Preset = preset::MAINNET;
            const PRESET_BASE_KIND: PresetBaseKind = PresetBaseKind::Mainnet;

            type PERIOD = typenum::Prod<
                <Self as EPOCHS_PER_SYNC_COMMITTEE_PERIOD>::EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
                <Self as SLOTS_PER_EPOCH>::SLOTS_PER_EPOCH,
            >;
        }
    };
}

consts_traits![
    // Misc
    DEPOSIT_CONTRACT_TREE_DEPTH,
    MAX_VALIDATORS_PER_COMMITTEE,
    // Time parameters
    SECONDS_PER_SLOT,
    SLOTS_PER_EPOCH,
    // Max operations per block
    MAX_PROPOSER_SLASHINGS,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    MAX_BLS_TO_EXECUTION_CHANGES,
    // Execution
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    // Sync committee
    SYNC_COMMITTEE_SIZE,
    EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
    // Sync protocol
    MIN_SYNC_COMMITTEE_PARTICIPANTS,
    UPDATE_TIMEOUT,
];

/// Values that are constant across all configurations.
pub mod consts {
    /// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#get_subtree_index>
    #[must_use]
    pub const fn get_subtree_index(idx: u64) -> u64 {
        idx % 2_u64.pow(idx.ilog2())
    }

    /// Convenience function safely to call [`u64::ilog2`] and convert the result into a usize.
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    #[must_use]
    pub const fn floorlog2(n: u64) -> usize {
        // conversion is safe since usize is either 32 or 64 bits as per cfg above
        n.ilog2() as usize
    }

    // https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#constants
    // REVIEW: Is it possible to implement get_generalized_index in const rust?

    // https://github.com/ethereum/consensus-specs/blob/dev/ssz/merkle-proofs.md
    /// `get_generalized_index(BeaconState, "finalized_checkpoint", "root")`
    pub const FINALIZED_ROOT_INDEX: u64 = 105;
    /// `get_generalized_index(BeaconState, "current_sync_committee")`
    pub const CURRENT_SYNC_COMMITTEE_INDEX: u64 = 54;
    /// `get_generalized_index(BeaconState, "next_sync_committee")`
    pub const NEXT_SYNC_COMMITTEE_INDEX: u64 = 55;
    /// `get_generalized_index(BeaconBlockBody, "execution_payload")`
    pub const EXECUTION_PAYLOAD_INDEX: u64 = 25;
}

pub mod preset {
    #[allow(non_snake_case)]
    #[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Preset {
        /// Misc
        /// ---------------------------------------------------------------
        pub DEPOSIT_CONTRACT_TREE_DEPTH: usize,
        pub MAX_VALIDATORS_PER_COMMITTEE: usize,

        /// Time parameters
        /// ---------------------------------------------------------------
        pub SECONDS_PER_SLOT: usize,
        pub SLOTS_PER_EPOCH: usize,

        /// Max operations per block
        /// ---------------------------------------------------------------
        pub MAX_PROPOSER_SLASHINGS: usize,
        pub MAX_ATTESTER_SLASHINGS: usize,
        pub MAX_ATTESTATIONS: usize,
        pub MAX_DEPOSITS: usize,
        pub MAX_VOLUNTARY_EXITS: usize,
        pub MAX_BLS_TO_EXECUTION_CHANGES: usize,

        /// Execution
        /// ---------------------------------------------------------------
        pub MAX_BYTES_PER_TRANSACTION: usize,
        pub MAX_TRANSACTIONS_PER_PAYLOAD: usize,
        pub BYTES_PER_LOGS_BLOOM: usize,
        pub MAX_EXTRA_DATA_BYTES: usize,
        pub MAX_WITHDRAWALS_PER_PAYLOAD: usize,

        /// Sync committee
        /// ---------------------------------------------------------------
        pub SYNC_COMMITTEE_SIZE: usize,
        pub EPOCHS_PER_SYNC_COMMITTEE_PERIOD: usize,

        /// Sync protocol
        /// ---------------------------------------------------------------
        pub MIN_SYNC_COMMITTEE_PARTICIPANTS: usize,
        pub UPDATE_TIMEOUT: usize,
    }

    /// <https://github.com/ethereum/consensus-specs/blob/dev/presets/mainnet>
    pub const MAINNET: Preset = Preset {
        DEPOSIT_CONTRACT_TREE_DEPTH: 32,
        MAX_VALIDATORS_PER_COMMITTEE: 2048,

        SECONDS_PER_SLOT: 12,
        SLOTS_PER_EPOCH: 32,

        MAX_PROPOSER_SLASHINGS: 16,
        MAX_ATTESTER_SLASHINGS: 2,
        MAX_ATTESTATIONS: 128,
        MAX_DEPOSITS: 16,
        MAX_VOLUNTARY_EXITS: 16,
        MAX_BLS_TO_EXECUTION_CHANGES: 16,
        SYNC_COMMITTEE_SIZE: 512,
        EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
        MIN_SYNC_COMMITTEE_PARTICIPANTS: 1,
        UPDATE_TIMEOUT: 8192,

        MAX_BYTES_PER_TRANSACTION: 1_073_741_824,
        MAX_TRANSACTIONS_PER_PAYLOAD: 1_048_576,
        BYTES_PER_LOGS_BLOOM: 256,
        MAX_EXTRA_DATA_BYTES: 32,
        MAX_WITHDRAWALS_PER_PAYLOAD: 16,
    };

    /// <https://github.com/ethereum/consensus-specs/blob/dev/presets/minimal>
    pub const MINIMAL: Preset = Preset {
        DEPOSIT_CONTRACT_TREE_DEPTH: 32,
        MAX_VALIDATORS_PER_COMMITTEE: 2048,

        SECONDS_PER_SLOT: 6,
        SLOTS_PER_EPOCH: 8,

        MAX_PROPOSER_SLASHINGS: 16,
        MAX_ATTESTER_SLASHINGS: 2,
        MAX_ATTESTATIONS: 128,
        MAX_DEPOSITS: 16,
        MAX_VOLUNTARY_EXITS: 16,
        MAX_BLS_TO_EXECUTION_CHANGES: 16,
        SYNC_COMMITTEE_SIZE: 32,
        EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 8,
        MIN_SYNC_COMMITTEE_PARTICIPANTS: 1,
        UPDATE_TIMEOUT: 64,

        MAX_BYTES_PER_TRANSACTION: 1_073_741_824,
        MAX_TRANSACTIONS_PER_PAYLOAD: 1_048_576,
        BYTES_PER_LOGS_BLOOM: 256,
        MAX_EXTRA_DATA_BYTES: 32,
        MAX_WITHDRAWALS_PER_PAYLOAD: 4,
    };
}

pub const GOERLI: Config = Config {
    preset: preset::MAINNET,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([0, 0, 16, 32]),
        genesis_slot: (0),
        altair: Fork {
            version: Version([1, 0, 16, 32]),
            epoch: (36660),
        },
        bellatrix: Fork {
            version: Version([2, 0, 16, 32]),
            epoch: 112_260,
        },
        capella: Fork {
            version: Version([3, 0, 16, 32]),
            epoch: 162_304,
        },
        // NOTE: dummy data
        eip4844: Fork {
            version: Version([4, 0, 0, 0]),
            epoch: u64::MAX,
        },
    },
    min_genesis_time: 1_614_588_812,
};

pub const MAINNET: Config = Config {
    preset: preset::MAINNET,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([0, 0, 0, 0]),
        genesis_slot: 0,

        altair: Fork {
            version: Version([1, 0, 0, 0]),
            epoch: 74_240,
        },
        bellatrix: Fork {
            version: Version([2, 0, 0, 0]),
            epoch: 144_896,
        },
        capella: Fork {
            version: Version([3, 0, 0, 0]),
            epoch: 194_048,
        },
        eip4844: Fork {
            // NOTE: dummy data
            version: Version([4, 0, 0, 0]),
            epoch: u64::MAX,
        },
    },
    min_genesis_time: 1_606_824_000,
};

pub const MINIMAL: Config = Config {
    preset: preset::MINIMAL,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([0, 0, 0, 1]),
        genesis_slot: 0,

        altair: Fork {
            version: Version([1, 0, 0, 1]),
            epoch: 0,
        },

        bellatrix: Fork {
            version: Version([2, 0, 0, 1]),
            epoch: 0,
        },

        capella: Fork {
            version: Version([3, 0, 0, 1]),
            epoch: 0,
        },

        // NOTE: dummy data
        eip4844: Fork {
            version: Version([4, 0, 0, 1]),
            epoch: u64::MAX,
        },
    },
    min_genesis_time: 1_578_009_600,
};

pub const SEPOLIA: Config = Config {
    preset: preset::MAINNET,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([144, 0, 0, 105]),
        genesis_slot: 0,

        altair: Fork {
            version: Version([144, 0, 0, 112]),
            epoch: 50,
        },

        bellatrix: Fork {
            version: Version([144, 0, 0, 113]),
            epoch: 100,
        },

        capella: Fork {
            version: Version([144, 0, 0, 114]),
            epoch: 56_832,
        },

        // NOTE: dummy data
        eip4844: Fork {
            version: Version([4, 0, 0, 0]),
            epoch: (u64::MAX),
        },
    },
    min_genesis_time: 1_655_647_200,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub preset: preset::Preset,
    pub fork_parameters: ForkParameters,
    pub min_genesis_time: u64,
}
