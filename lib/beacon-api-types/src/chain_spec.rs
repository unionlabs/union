use core::str::FromStr;
use std::{fmt, fmt::Debug};

use typenum::{NonZero, Unsigned};

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
            const PRESET: crate::preset::Preset;
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
                            const PRESET: crate::preset::Preset = $d preset;
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

mk_chain_spec!(Minimal is crate::preset::MINIMAL);
mk_chain_spec!(Mainnet is crate::preset::MAINNET);
