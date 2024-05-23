use cosmwasm_std::{CustomQuery, Deps, DepsMut};
use unionlabs::{google::protobuf::timestamp::Timestamp, ibc::core::client::height::Height};

pub const CONSENSUS_STATE_ITER_KEY_PREFIX: &str = "iter_cons";
const METADATA_SIZE: usize = std::mem::size_of::<ConsensusStateMetadata>();

#[derive(Debug, PartialEq)]
pub enum StorageError {
    InvalidConsensusStateMetadata,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ConsensusStateMetadata {
    pub timestamp: Timestamp,
}

impl ConsensusStateMetadata {
    pub fn new(timestamp: Timestamp) -> Self {
        ConsensusStateMetadata { timestamp }
    }

    pub fn decode_packed(data: &[u8]) -> Result<Self, StorageError> {
        if data.len() != METADATA_SIZE {
            return Err(StorageError::InvalidConsensusStateMetadata);
        }

        Ok(Self {
            timestamp: Timestamp {
                // unwrap is valid here since i64 and i32 is defined for all the possible values of 0..8 and 0..4
                seconds: i64::from_le_bytes((&data[0..8]).try_into().unwrap())
                    .try_into()
                    .unwrap(),
                nanos: i32::from_le_bytes((&data[8..12]).try_into().unwrap())
                    .try_into()
                    .unwrap(),
            },
        })
    }

    pub fn encode_packed(self) -> [u8; METADATA_SIZE] {
        let mut key: [u8; METADATA_SIZE] = [0; METADATA_SIZE];
        key[0..8].copy_from_slice(&self.timestamp.seconds.inner().to_le_bytes());
        key[8..12].copy_from_slice(&self.timestamp.nanos.inner().to_le_bytes());
        key
    }
}

/// Return the item at `height` if it exists or the previous item in terms of lexicographical order.
///
/// **Note**: This is not meant to be used for iteration. It wouldn't work because if the data exists
/// at `height`, it returns the it. And it will be an endless loop.
/// **Note**: The caller should note that if there is no other (lexicographically) previous consensus
/// state metadata in the storage, this could try to parse a random item and return an error. Hence,
/// on such occasion, there is no guarantee of whether the caller will get `Ok(None)` or `Err`.
pub fn get_current_or_prev_consensus_state_meta<C: CustomQuery>(
    deps: Deps<C>,
    height: Height,
) -> Result<Option<(Height, ConsensusStateMetadata)>, StorageError> {
    let current_key = consensus_state_iterator_key(height);
    if let Some((key, metadata)) = deps
        .storage
        .range(None, Some(&current_key), cosmwasm_std::Order::Descending)
        .next()
    {
        Ok(Some((
            parse_height_from_key(&key)?,
            ConsensusStateMetadata::decode_packed(&metadata)?,
        )))
    } else {
        Ok(None)
    }
}

/// Return the item at `height` if it exists or the next item in terms of lexicographical order.
///
/// **Note**: This is not meant to be used for iteration. It wouldn't work because if the data exists
/// at `height`, it returns the it. And it will be an endless loop.
/// **Note**: The caller should note that if there is no other (lexicographically) next consensus
/// state metadata in the storage, this could try to parse a random item and return an error. Hence,
/// on such occasion, there is no guarantee of whether the caller will get `Ok(None)` or `Err`.
pub fn get_current_or_next_consensus_state_meta<C: CustomQuery>(
    deps: Deps<C>,
    height: Height,
) -> Result<Option<(Height, ConsensusStateMetadata)>, StorageError> {
    let current_key = consensus_state_iterator_key(height);
    if let Some((key, metadata)) = deps
        .storage
        .range(Some(&current_key), None, cosmwasm_std::Order::Ascending)
        .next()
    {
        Ok(Some((
            parse_height_from_key(&key)?,
            ConsensusStateMetadata::decode_packed(&metadata)?,
        )))
    } else {
        Ok(None)
    }
}

/// Parse the height from the given iteration `key`.
pub fn parse_height_from_key(key: &[u8]) -> Result<Height, StorageError> {
    // 17: '/' + height
    if key.len() != CONSENSUS_STATE_ITER_KEY_PREFIX.len() + 17 {
        return Err(StorageError::InvalidConsensusStateMetadata);
    }
    // `unwrap`'s are safe here because we get a slice with length 8, which will always
    // succeed when we try to convert it to [u8; 8].
    let revision_height = u64::from_be_bytes(key[key.len() - 8..key.len()].try_into().unwrap());
    let revision_number =
        u64::from_be_bytes(key[key.len() - 16..key.len() - 8].try_into().unwrap());
    Ok(Height {
        revision_number,
        revision_height,
    })
}

/// Save the consensus state metadata at `height`.
pub fn save_consensus_state_metadata<C: CustomQuery>(
    deps: DepsMut<'_, C>,
    timestamp: Timestamp,
    height: Height,
) {
    let iterator_key = consensus_state_iterator_key(height);

    deps.storage.set(
        &iterator_key,
        &ConsensusStateMetadata::new(timestamp).encode_packed(),
    );
}

/// Get the consensus state iterator key with the `height`.
pub fn consensus_state_iterator_key(height: Height) -> Vec<u8> {
    CONSENSUS_STATE_ITER_KEY_PREFIX
        .bytes()
        .chain(*b"/")
        .chain(height.revision_number.to_be_bytes())
        .chain(height.revision_height.to_be_bytes())
        .collect()
}
