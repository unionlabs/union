use core::fmt::Display;

use generic_array::GenericArray;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_types::{BitList, FixedVector, VariableList};
use tree_hash::TreeHash;
use typenum::U;

use crate::{
    bls::{BlsPublicKey, BlsSignature},
    ethereum::beacon::BeaconBlock,
    ethereum_consts_traits::{
        BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS,
        MAX_ATTESTER_SLASHINGS, MAX_BLS_TO_EXECUTION_CHANGES, MAX_BYTES_PER_TRANSACTION,
        MAX_DEPOSITS, MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS, MAX_TRANSACTIONS_PER_PAYLOAD,
        MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS, MAX_WITHDRAWALS_PER_PAYLOAD,
        SYNC_COMMITTEE_SIZE,
    },
    ibc::lightclients::ethereum::beacon_block_header::BeaconBlockHeader,
};

pub mod beacon;

// REVIEW: Is this needed? Currently unused
pub const BLOCK_BODY_EXECUTION_PAYLOAD_INDEX: usize = 9;

macro_rules! hex_string_array_wrapper {
    (
        $(
            pub struct $Struct:ident(pub [u8; $N:literal]);
        )+
    ) => {
        $(
            #[derive(Clone, PartialEq, Eq, Default, Encode, Decode, Serialize, Deserialize)]
            #[ssz(struct_behaviour = "transparent")]
            pub struct $Struct(#[serde(with = "::serde_utils::hex_string")] pub [u8; $N]);

            impl std::str::FromStr for $Struct {
                type Err = serde_utils::FromHexStringError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    serde_utils::parse_hex(s)
                }
            }

            impl TryFrom<Vec<u8>> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                    value
                        .try_into()
                        .map(Self)
                        .map_err(|invalid| crate::errors::InvalidLength {
                            expected: $N,
                            found: invalid.len(),
                        })
                }
            }

            impl TryFrom<&[u8]> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                    value
                        .try_into()
                        .map(Self)
                        .map_err(|_| crate::errors::InvalidLength {
                            expected: $N,
                            found: value.len(),
                        })
                }
            }

            impl From<$Struct> for Vec<u8> {
                fn from(value: $Struct) -> Self {
                    value.0.into()
                }
            }

            impl From<$Struct> for [u8; $N] {
                fn from(value: $Struct) -> Self {
                    value.0
                }
            }

            impl From<[u8; $N]> for $Struct {
                fn from(value: [u8; $N]) -> Self {
                    Self(value)
                }
            }

            impl std::fmt::Debug for $Struct {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "H256({self})")
                }
            }

            impl Display for $Struct {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "0x{}", hex::encode(self.0).as_str())
                }
            }

            // arrays and `FixedVector`s are effectively the exact same type, implement
            // the former in terms of the latter
            impl TreeHash for $Struct {
                fn tree_hash_type() -> tree_hash::TreeHashType {
                    FixedVector::<u8, U<$N>>::tree_hash_type()
                }

                fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
                    FixedVector::<u8, U<$N>>::tree_hash_packed_encoding(&self.0.into())
                }

                fn tree_hash_packing_factor() -> usize {
                    FixedVector::<u8, U<$N>>::tree_hash_packing_factor()
                }

                fn tree_hash_root(&self) -> tree_hash::Hash256 {
                    FixedVector::<u8, U<$N>>::tree_hash_root(&self.0.into())
                }
            }

            impl From<GenericArray<u8, U<$N>>> for $Struct {
                fn from(arr: GenericArray<u8, U<$N>>) -> Self {
                    Self(arr.to_vec().try_into().expect("GenericArray has the correct length; qed;"))
                }
            }
        )+
    };
}

hex_string_array_wrapper! {
    pub struct Version(pub [u8; 4]);
    pub struct DomainType(pub [u8; 4]);
    pub struct ForkDigest(pub [u8; 4]);
    pub struct Domain(pub [u8; 32]);

    // TODO: These aren't used for only ethereum, they should be moved out of this module
    pub struct Address(pub [u8; 20]);
    pub struct H256(pub [u8; 32]);
}

#[rustfmt::skip]
impl DomainType {
    pub const BEACON_PROPOSER: Self                = Self(hex!("00000000"));
    pub const BEACON_ATTESTER: Self                = Self(hex!("01000000"));
    pub const RANDAO: Self                         = Self(hex!("02000000"));
    pub const DEPOSIT: Self                        = Self(hex!("03000000"));
    pub const VOLUNTARY_EXIT: Self                 = Self(hex!("04000000"));
    pub const SELECTION_PROOF: Self                = Self(hex!("05000000"));
    pub const AGGREGATE_AND_PROOF: Self            = Self(hex!("06000000"));
    pub const SYNC_COMMITTEE: Self                 = Self(hex!("07000000"));
    pub const SYNC_COMMITTEE_SELECTION_PROOF: Self = Self(hex!("08000000"));
    pub const CONTRIBUTION_AND_PROOF: Self         = Self(hex!("09000000"));
    pub const BLS_TO_EXECUTION_CHANGE: Self        = Self(hex!("0A000000"));
    pub const APPLICATION_MASK: Self               = Self(hex!("00000001"));
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct ForkData {
    pub current_version: Version,
    pub genesis_validators_root: H256,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signingdata>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct SigningData {
    pub object_root: H256,
    pub domain: Domain,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblockheader>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblock>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct SignedBeaconBlock<
    C: MAX_PROPOSER_SLASHINGS
        + MAX_VALIDATORS_PER_COMMITTEE
        + MAX_ATTESTER_SLASHINGS
        + MAX_ATTESTATIONS
        + DEPOSIT_CONTRACT_TREE_DEPTH
        + MAX_DEPOSITS
        + MAX_VOLUNTARY_EXITS
        + BYTES_PER_LOGS_BLOOM
        + MAX_EXTRA_DATA_BYTES
        + MAX_BYTES_PER_TRANSACTION
        + MAX_TRANSACTIONS_PER_PAYLOAD
        + MAX_WITHDRAWALS_PER_PAYLOAD
        + MAX_BLS_TO_EXECUTION_CHANGES
        + SYNC_COMMITTEE_SIZE,
> {
    pub message: BeaconBlock<C>,
    pub signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct Eth1Data {
    pub deposit_root: H256,
    #[serde(with = "::serde_utils::string")]
    pub deposit_count: u64,
    pub block_hash: H256,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attesterslashing>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AttesterSlashing<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attestation_1: IndexedAttestation<C>,
    pub attestation_2: IndexedAttestation<C>,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#indexedattestation>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IndexedAttestation<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attesting_indices: VariableList<u64, C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct AttestationData {
    #[serde(with = "::serde_utils::string")]
    pub slot: u64,
    #[serde(with = "::serde_utils::string")]
    pub index: u64,
    /// LMD GHOST vote
    pub beacon_block_root: H256,
    /// FFG vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct Checkpoint {
    #[serde(with = "::serde_utils::string")]
    pub epoch: u64,
    pub root: H256,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct Attestation<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub aggregation_bits: BitList<C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct Deposit<C: DEPOSIT_CONTRACT_TREE_DEPTH> {
    /// Merkle path to deposit root
    pub proof: FixedVector<[u8; 32], C::DEPOSIT_CONTRACT_TREE_DEPTH>,
    pub data: DepositData,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct DepositData {
    pub pubkey: BlsPublicKey,
    pub withdrawal_credentials: [u8; 32],
    #[serde(with = "::serde_utils::string")]
    pub amount: u64,
    /// Signing over DepositMessage
    pub signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#voluntaryexit>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct VoluntaryExit {
    /// Earliest epoch when voluntary exit can be processed
    #[serde(with = "::serde_utils::string")]
    pub epoch: u64,
    #[serde(with = "::serde_utils::string")]
    pub validator_index: u64,
}

impl H256 {
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        // use this if we ever swap out the inner value for primitive_types::H256
        // self.0.into_iter().flat_map(|n| n.to_le_bytes()).collect()
        self.0.to_vec()
    }
}

impl From<H256> for primitive_types::H256 {
    fn from(value: H256) -> Self {
        Self(value.0)
    }
}

impl From<primitive_types::H256> for H256 {
    fn from(value: primitive_types::H256) -> Self {
        Self(value.0)
    }
}

impl From<Address> for primitive_types::H160 {
    fn from(value: Address) -> Self {
        Self(value.0)
    }
}

impl From<primitive_types::H160> for Address {
    fn from(value: primitive_types::H160) -> Self {
        Self(value.0)
    }
}

// #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
// pub struct LightClientBootstrap<const SYNC_COMMITTEE_SIZE: usize> {
//     pub beacon_header: BeaconBlockHeader,
//     /// Current sync committee corresponding to `beacon_header.state_root`
//     pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
//     pub current_sync_committee_branch: [H256; CURRENT_SYNC_COMMITTEE_DEPTH],
// }
