/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
pub mod attestation;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
pub mod attestation_data;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attesterslashing>
pub mod attester_slashing;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock>
pub mod beacon_block;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody>
pub mod beacon_block_body;
pub mod beacon_block_header;
pub mod bls_to_execution_change;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
pub mod checkpoint;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
pub mod deposit;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
pub mod deposit_data;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
pub mod eth1_data;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayload>
pub mod execution_payload;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayloadheader>
pub mod execution_payload_header;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#fork>
pub mod fork;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#forkdata>
pub mod fork_data;
pub mod fork_parameters;
pub mod genesis_data;
pub mod indexed_attestation;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap>
pub mod light_client_bootstrap;
pub mod light_client_finality_update;
pub mod light_client_header;
/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#lightclientupdate>
pub mod light_client_update;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
pub mod proposer_slashing;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblock>
pub mod signed_beacon_block;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblockheader>
pub mod signed_beacon_block_header;
pub mod signed_bls_to_execution_change;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
pub mod signed_voluntary_exit;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signingdata>
pub mod signing_data;
pub mod sync_aggregate;
pub mod sync_committee;
/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#voluntaryexit>
pub mod voluntary_exit;
pub mod withdrawal;

use core::{
    fmt::{self, Debug},
    str::FromStr,
};
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use hex_literal::hex;
use typenum::{NonZero, Unsigned};
use unionlabs::primitives::FixedBytes;

pub use crate::{
    attestation::Attestation, attestation_data::AttestationData,
    attester_slashing::AttesterSlashing, beacon_block::BeaconBlock,
    beacon_block_body::BeaconBlockBody, beacon_block_header::BeaconBlockHeader,
    bls_to_execution_change::BlsToExecutionChange, checkpoint::Checkpoint, deposit::Deposit,
    deposit_data::DepositData, eth1_data::Eth1Data, execution_payload::ExecutionPayload,
    fork::Fork, fork_data::ForkData, fork_parameters::ForkParameters, genesis_data::GenesisData,
    indexed_attestation::IndexedAttestation, light_client_bootstrap::LightClientBootstrap,
    light_client_finality_update::LightClientFinalityUpdate,
    light_client_header::LightClientHeader, light_client_update::LightClientUpdate,
    proposer_slashing::ProposerSlashing, signed_beacon_block::SignedBeaconBlock,
    signed_beacon_block_header::SignedBeaconBlockHeader,
    signed_bls_to_execution_change::SignedBlsToExecutionChange,
    signed_voluntary_exit::SignedVoluntaryExit, signing_data::SigningData,
    sync_aggregate::SyncAggregate, sync_committee::SyncCommittee, voluntary_exit::VoluntaryExit,
    withdrawal::Withdrawal,
};
#[cfg(feature = "ssz")]
pub use crate::{
    attestation::AttestationSsz, attester_slashing::AttesterSlashingSsz,
    beacon_block::BeaconBlockSsz, beacon_block_body::BeaconBlockBodySsz, deposit::DepositSsz,
    execution_payload::ExecutionPayloadSsz, execution_payload_header::ExecutionPayloadHeaderSsz,
    indexed_attestation::IndexedAttestationSsz, light_client_header::LightClientHeaderSsz,
    signed_beacon_block::SignedBeaconBlockSsz, sync_aggregate::SyncAggregateSsz,
    sync_committee::SyncCommitteeSsz,
};

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#custom-types>
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Version(pub FixedBytes<4>);

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#custom-types>
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct DomainType(pub FixedBytes<4>);

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#custom-types>
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ForkDigest(pub FixedBytes<4>);

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#custom-types>
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Domain(pub FixedBytes<32>);

/// <https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/phase0/beacon-chain.md#domain-types>
#[rustfmt::skip]
impl DomainType {
    pub const BEACON_PROPOSER: Self                = Self(FixedBytes::new(hex!("00000000")));
    pub const BEACON_ATTESTER: Self                = Self(FixedBytes::new(hex!("01000000")));
    pub const RANDAO: Self                         = Self(FixedBytes::new(hex!("02000000")));
    pub const DEPOSIT: Self                        = Self(FixedBytes::new(hex!("03000000")));
    pub const VOLUNTARY_EXIT: Self                 = Self(FixedBytes::new(hex!("04000000")));
    pub const SELECTION_PROOF: Self                = Self(FixedBytes::new(hex!("05000000")));
    pub const AGGREGATE_AND_PROOF: Self            = Self(FixedBytes::new(hex!("06000000")));
    pub const SYNC_COMMITTEE: Self                 = Self(FixedBytes::new(hex!("07000000")));
    pub const SYNC_COMMITTEE_SELECTION_PROOF: Self = Self(FixedBytes::new(hex!("08000000")));
    pub const CONTRIBUTION_AND_PROOF: Self         = Self(FixedBytes::new(hex!("09000000")));
    pub const BLS_TO_EXECUTION_CHANGE: Self        = Self(FixedBytes::new(hex!("0A000000")));
    pub const APPLICATION_MASK: Self               = Self(FixedBytes::new(hex!("00000001")));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz), ssz(transparent))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Slot(u64);

impl Slot {
    pub const fn new(slot: u64) -> Self {
        Self(slot)
    }

    pub const fn get(&self) -> u64 {
        self.0
    }
}

impl Add<Self> for Slot {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}

impl Sub<Self> for Slot {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}

impl Mul<Self> for Slot {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.0 * rhs.0)
    }
}

impl Div<Self> for Slot {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.0 / rhs.0)
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Slot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(&self.0)
        } else {
            self.0.serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Slot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer).and_then(|s| {
                s.parse()
                    // TODO fix error situation
                    // FromStr::Err has no bounds
                    .map_err(|_| serde::de::Error::custom("failure to parse string data"))
                    .map(Self)
            })
        } else {
            u64::deserialize(deserializer).map(Self)
        }
    }
}

/// Minimal config.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Minimal;

/// Mainnet config.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mainnet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum PresetBaseKind {
    Minimal,
    Mainnet,
}

impl fmt::Display for PresetBaseKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PresetBaseKind::Minimal => "minimal",
            PresetBaseKind::Mainnet => "mainnet",
        })
    }
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

// https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963
macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

macro_rules! consts_traits {
    ($($CONST:ident $(,)?),+) => {
        $(
            #[allow(non_camel_case_types)]
            pub trait $CONST: Send + Sync + Unpin + 'static {
                // Extra traits are required because the builtin derives bound all generic
                // types unconditionally
                type $CONST: Unsigned + NonZero + Debug + Clone + PartialEq + Eq + Send + Sync + Unpin;
            }
        )+

        pub trait ChainSpec: 'static + Debug + Clone + PartialEq + Eq + Default + Send + Sync + Unpin + $($CONST+)+ {
            const PRESET: preset::Preset;
            // const PRESET_BASE_KIND: PresetBaseKind;

            type PERIOD: 'static + Unsigned;
        }

        with_dollar_sign! {
            ($d:tt) => {
                // TODO: Keep an eye on this issue https://github.com/rust-lang/rust/issues/98291, as it might resolve an issue with macro_export-ing this macro (currently it is only available in this crate)
                // #[macro_export]
                macro_rules! mk_chain_spec {
                    ($d T:ident is $d preset:path) => {
                        $(
                            impl $CONST for $d T {
                                #[allow(non_camel_case_types)]
                                type $CONST = typenum::U<{ $d preset.$CONST }>;
                           }
                        )*

                        impl ChainSpec for $d T {
                            const PRESET: preset::Preset = $d preset;
                            // const PRESET_BASE_KIND: PresetBaseKind = PresetBaseKind::Mainnet;

                            type PERIOD = typenum::Prod<
                                <Self as EPOCHS_PER_SYNC_COMMITTEE_PERIOD>::EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
                                <Self as SLOTS_PER_EPOCH>::SLOTS_PER_EPOCH,
                            >;
                        }
                    };
                }
            }
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
    MAX_BLOB_COMMITMENTS_PER_BLOCK,
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

mk_chain_spec!(Minimal is preset::MINIMAL);
mk_chain_spec!(Mainnet is preset::MAINNET);

/// Values that are constant across all configurations.
pub mod consts {
    use hex_literal::hex;
    use unionlabs::primitives::FixedBytes;

    use crate::{Fork, Version};

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

    pub const fn default_epoch() -> u64 {
        u64::MAX
    }

    pub const fn default_fork() -> Fork {
        Fork {
            version: Version(FixedBytes::new(hex!("00000000"))),
            epoch: default_epoch(),
        }
    }
}

pub mod preset {
    #[allow(non_snake_case)]
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        pub MAX_BLOB_COMMITMENTS_PER_BLOCK: usize,

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
        MAX_BLOB_COMMITMENTS_PER_BLOCK: 4096,
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
        MAX_BLOB_COMMITMENTS_PER_BLOCK: 16,

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
