use cosmwasm_std::{Deps, DepsMut};
use unionlabs::ibc::core::client::height::Height;

pub const CONSENSUS_STATE_ITER_KEY_PREFIX: &str = "iter_cons";
const METADATA_SIZE: usize = std::mem::size_of::<ConsensusStateMetadata>();

#[derive(Debug)]
pub enum StorageError {
    InvalidConsensusStateMetadata,
}

#[repr(C)]
pub struct ConsensusStateMetadata {
    pub timestamp: u64,
}

impl ConsensusStateMetadata {
    fn new(timestamp: u64) -> Self {
        ConsensusStateMetadata { timestamp }
    }

    fn decode_packed(data: Vec<u8>) -> Result<Self, StorageError> {
        if data.len() != METADATA_SIZE {
            return Err(StorageError::InvalidConsensusStateMetadata);
        }

        Ok(Self {
            timestamp: u64::from_le_bytes(data.as_slice().try_into().unwrap()),
        })
    }

    fn encode_packed(self) -> [u8; METADATA_SIZE] {
        let mut key: [u8; METADATA_SIZE] = [0; METADATA_SIZE];
        key[0..8].copy_from_slice(&self.timestamp.to_le_bytes());
        key
    }
}

pub fn prev_consensus_state_meta(
    deps: Deps,
    height: Height,
) -> Result<Option<(Height, ConsensusStateMetadata)>, StorageError> {
    let current_key = consensus_state_iterator_key(height);
    if let Some((key, metadata)) = deps
        .storage
        .range(None, Some(&current_key), cosmwasm_std::Order::Descending)
        .next()
    {
        Ok(Some((
            parse_height_from_key(key)?,
            ConsensusStateMetadata::decode_packed(metadata)?,
        )))
    } else {
        Ok(None)
    }
}

pub fn next_consensus_state_meta(
    deps: Deps,
    height: Height,
) -> Result<Option<(Height, ConsensusStateMetadata)>, StorageError> {
    let current_key = consensus_state_iterator_key(height);
    if let Some((key, metadata)) = deps
        .storage
        .range(Some(&current_key), None, cosmwasm_std::Order::Ascending)
        .next()
    {
        Ok(Some((
            parse_height_from_key(key)?,
            ConsensusStateMetadata::decode_packed(metadata)?,
        )))
    } else {
        Ok(None)
    }
}

fn parse_height_from_key(key: Vec<u8>) -> Result<Height, StorageError> {
    // 17: '/' + height
    if key.len() != CONSENSUS_STATE_ITER_KEY_PREFIX.len() + 17 {
        return Err(StorageError::InvalidConsensusStateMetadata);
    }
    let revision_height = u64::from_be_bytes(key[key.len() - 8..key.len()].try_into().unwrap());
    let revision_number =
        u64::from_be_bytes(key[key.len() - 16..key.len() - 8].try_into().unwrap());
    Ok(Height {
        revision_number,
        revision_height,
    })
}

pub fn save_consensus_state_metadata(deps: DepsMut, timestamp: u64, height: Height) {
    let iterator_key = consensus_state_iterator_key(height);

    deps.storage.set(
        &iterator_key,
        &ConsensusStateMetadata::new(timestamp).encode_packed(),
    );
}

pub fn consensus_state_iterator_key(height: Height) -> Vec<u8> {
    CONSENSUS_STATE_ITER_KEY_PREFIX
        .bytes()
        .chain(*b"/")
        .chain(height.revision_number.to_be_bytes())
        .chain(height.revision_height.to_be_bytes())
        .collect()
}
