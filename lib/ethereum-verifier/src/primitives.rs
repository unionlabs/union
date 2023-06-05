use crate::byte_vector::ByteVector;
use crate::error::Error;
use ssz_rs::prelude::*;

pub use ssz_rs::prelude::U256;

pub type Root = Node;
pub type Slot = u64;
pub type Epoch = u64;

pub type CommitteeIndex = usize;
pub type ValidatorIndex = usize;
pub type WithdrawalIndex = usize;
pub type Gwei = u64;
pub type Hash32 = Bytes32;

pub type Version = [u8; 4];
pub type ForkDigest = [u8; 4];
pub type Domain = [u8; 32];

pub type ExecutionAddress = ByteVector<20>;

pub type ChainId = usize;
pub type NetworkId = usize;

pub type Bytes32 = ByteVector<32>;

pub type ParticipationFlags = u8;

// Coordinate refers to a unique location in the block tree
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Coordinate {
    #[serde(with = "crate::serde::as_string")]
    slot: Slot,
    root: Root,
}

pub const GENESIS_SLOT: Slot = 0;
pub const GENESIS_EPOCH: Epoch = 0;
pub const FAR_FUTURE_EPOCH: Epoch = Epoch::MAX;

pub const BLS_WITHDRAWAL_PREFIX: u8 = 0x00;
pub const ETH1_ADDRESS_WITHDRAWAL_PREFIX: u8 = 0x01;

#[derive(Clone, Copy)]
pub enum DomainType {
    BeaconProposer,
    BeaconAttester,
    Randao,
    Deposit,
    VoluntaryExit,
    SelectionProof,
    AggregateAndProof,
    SyncCommittee,
    SyncCommitteeSelectionProof,
    ContributionAndProof,
    BlsToExecutionChange,
    ApplicationMask,
    ApplicationBuilder,
}

impl DomainType {
    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            Self::ApplicationMask => [0, 0, 0, 1],
            Self::ApplicationBuilder => [0, 0, 0, 1],
            _ => {
                let data = *self as u32;
                data.to_le_bytes()
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Account {
    pub nonce: u64,
    pub balance: Vec<u8>,
    pub storage_root: Hash32,
    pub code_hash: Hash32,
}

impl Account {
    pub fn from_rlp_bytes(bz: &[u8]) -> Result<Account, Error> {
        let r = rlp::Rlp::new(bz);
        Ok(Account {
            nonce: r.val_at::<u64>(0).unwrap(),
            balance: r.val_at::<Vec<u8>>(1).unwrap(),
            storage_root: Hash32::try_from(r.val_at::<Vec<u8>>(2).unwrap().as_slice()).unwrap(),
            code_hash: Hash32::try_from(r.val_at::<Vec<u8>>(3).unwrap().as_slice()).unwrap(),
        })
    }
}
