use cosmwasm_std::{Deps, DepsMut};
use unionlabs::ibc::core::client::height::Height;

pub const CONSENSUS_STATE_ITER_KEY_PREFIX: &str = "iter_cons";
const METADATA_SIZE: usize = std::mem::size_of::<ConsensusStateMetadata>();

#[derive(Debug, PartialEq)]
pub enum StorageError {
    InvalidConsensusStateMetadata,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ConsensusStateMetadata {
    pub timestamp: u64,
}

impl ConsensusStateMetadata {
    pub fn new(timestamp: u64) -> Self {
        ConsensusStateMetadata { timestamp }
    }

    pub fn decode_packed(data: &[u8]) -> Result<Self, StorageError> {
        if data.len() != METADATA_SIZE {
            return Err(StorageError::InvalidConsensusStateMetadata);
        }

        Ok(Self {
            timestamp: u64::from_le_bytes(data.try_into().unwrap()),
        })
    }

    pub fn encode_packed(self) -> [u8; METADATA_SIZE] {
        let mut key: [u8; METADATA_SIZE] = [0; METADATA_SIZE];
        key[0..8].copy_from_slice(&self.timestamp.to_le_bytes());
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
pub fn get_current_or_prev_consensus_state_meta(
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
pub fn get_current_or_next_consensus_state_meta(
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
    Ok(Height::new_with_revision(revision_number, revision_height))
}

/// Save the consensus state metadata at `height`.
pub fn save_consensus_state_metadata(deps: DepsMut, timestamp: u64, height: Height) {
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
        .chain(height.revision().to_be_bytes())
        .chain(height.height().to_be_bytes())
        .collect()
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Storage};

    use super::*;

    #[test]
    fn consensus_state_metadata_encode_decode_works() {
        let timestamps = [0, 123123, 132137, 99, 374891223, u64::MAX];

        timestamps.into_iter().for_each(|t| {
            let encoded = ConsensusStateMetadata::new(t).encode_packed();
            assert_eq!(encoded, t.to_le_bytes());
            assert_eq!(
                t,
                ConsensusStateMetadata::decode_packed(&encoded)
                    .unwrap()
                    .timestamp
            );
        });

        // len is < METADATA_SIZE
        assert_eq!(
            ConsensusStateMetadata::decode_packed(&[0; METADATA_SIZE - 1]),
            Err(StorageError::InvalidConsensusStateMetadata)
        );

        // len is > METADATA_SIZE
        assert_eq!(
            ConsensusStateMetadata::decode_packed(&[0; METADATA_SIZE + 1]),
            Err(StorageError::InvalidConsensusStateMetadata)
        );
    }

    #[test]
    fn consensus_state_parse_height_from_key_works() {
        let heights = [
            (2_u64, 22_u64),
            (1, 5115),
            (0, 23451),
            (3, 1234),
            (1, 2123),
            (1, 1000),
            (0, 132),
            (0, u64::MAX),
        ];

        heights
            .into_iter()
            .for_each(|(revision_number, revision_height)| {
                let height = Height::new_with_revision(revision_number, revision_height);
                let key = consensus_state_iterator_key(height);

                assert_eq!(parse_height_from_key(&key), Ok(height));
            });

        assert_eq!(
            parse_height_from_key(&[0; CONSENSUS_STATE_ITER_KEY_PREFIX.len() + 17 + 1]),
            Err(StorageError::InvalidConsensusStateMetadata)
        );
    }

    #[test]
    fn next_prev_consensus_state_key_works() {
        let ordered_heights = [
            (0, 132, 1231238123),
            (0, 23451, 12323),
            (0, u64::MAX - 1, 23713),
            (1, 1000, 9),
            (1, 2123, 3781273413),
            (1, 5115, 2377281),
            (2_u64, 22_u64, 2391232_u64),
            (3, 1234, 238918392),
        ];

        let mut deps = mock_dependencies();

        ordered_heights.iter().for_each(|(rn, rh, timestamp)| {
            save_consensus_state_metadata(
                deps.as_mut(),
                *timestamp,
                Height::new_with_revision(*rn, *rh),
            );
        });

        let prev_height = Height::new_with_revision(ordered_heights[2].0, ordered_heights[2].1);
        let next_height = Height::new_with_revision(ordered_heights[3].0, ordered_heights[3].1);

        let next = get_current_or_next_consensus_state_meta(
            deps.as_ref(),
            Height::new_with_revision(
                prev_height.revision(),
                // +1 because the api says that if the input exists, it returns the input
                prev_height.height() + 1,
            ),
        )
        .unwrap()
        .unwrap();

        assert_eq!(next.0, next_height);
        assert_eq!(next.1.timestamp, ordered_heights[3].2);

        let prev = get_current_or_prev_consensus_state_meta(
            deps.as_ref(),
            Height::new_with_revision(
                next_height.revision(),
                // -1 because the api says that if the input exists, it returns the input
                next_height.height() - 1,
            ),
        )
        .unwrap()
        .unwrap();

        assert_eq!(prev.0, prev_height);
        assert_eq!(prev.1.timestamp, ordered_heights[2].2);

        // next of a height of the largest item is None
        assert_eq!(
            get_current_or_next_consensus_state_meta(
                deps.as_ref(),
                Height::new_with_revision(
                    ordered_heights.last().unwrap().0,
                    ordered_heights.last().unwrap().1 + 1
                )
            )
            .unwrap(),
            None
        );

        // prev of a height of the smallest item is None
        assert_eq!(
            get_current_or_prev_consensus_state_meta(
                deps.as_ref(),
                Height::new_with_revision(
                    ordered_heights.first().unwrap().0,
                    ordered_heights.first().unwrap().1 - 1
                )
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn consensus_state_keys_lexicographically_ordered() {
        let mut unordered_heights = [
            (2_u64, 22_u64),
            (1, 5115),
            (0, 23451),
            (3, 1234),
            (1, 2123),
            (1, 1000),
            (0, 132),
            (0, u64::MAX),
        ];

        let mut deps = mock_dependencies();

        unordered_heights.iter().for_each(|(rn, rh)| {
            deps.storage.set(
                &consensus_state_iterator_key(Height::new_with_revision(*rn, *rh)),
                &[1],
            );
        });

        unordered_heights.sort();

        deps.storage
            .range(None, None, cosmwasm_std::Order::Ascending)
            .enumerate()
            .for_each(|(i, (k, _))| {
                assert_eq!(
                    k,
                    consensus_state_iterator_key(Height::new_with_revision(
                        unordered_heights[i].0,
                        unordered_heights[i].1,
                    ))
                );
            })
    }
}
