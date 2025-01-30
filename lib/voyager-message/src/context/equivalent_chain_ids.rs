use std::{collections::HashSet, sync::OnceLock};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core::ChainId;

/// [`ChainId`] to consider equivalent.
///
/// Some chains expose multiple chain IDs due to requirements of certain components used in the
/// chain (for example, different execution environments running in the same chain may have
/// different chain ID specifications for transaction signing). In cases such as this, multiple
/// chain IDs can refer to the same "abstract machine" of a chain.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "Vec<HashSet<ChainId>>", into = "Vec<HashSet<ChainId>>")]
// TODO: Implement JsonSchema manually to properly encode the constraints on the value
#[schemars(transparent)]
pub struct EquivalentChainIds {
    // NOTE: All chain IDs in this list are required to be unique
    // TODO: Ensure that all of the inner HashSets contain >= 2 items
    pub(crate) inner: Vec<HashSet<ChainId>>,
}

impl EquivalentChainIds {
    /// Get all chain IDs that are equivalent to `chain_id`.
    ///
    /// Note that this does ***NOT*** include `chain_id`.
    pub fn equivalents<'a, 'b>(
        &'b self,
        chain_id: &'a ChainId,
    ) -> impl Iterator<Item = &'b ChainId> + use<'a, 'b> {
        static EMPTY_ITER: OnceLock<HashSet<ChainId>> = OnceLock::new();

        self.inner
            .iter()
            .find(|v| v.contains(chain_id))
            .unwrap_or(EMPTY_ITER.get_or_init(HashSet::new))
            .iter()
            .filter(move |id| id != chain_id)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[allow(
    clippy::implicit_hasher,
    reason = "EquivalentChainIds.inner is not generic over the hasher"
)]
impl From<EquivalentChainIds> for Vec<HashSet<ChainId>> {
    fn from(value: EquivalentChainIds) -> Self {
        value.inner
    }
}

impl TryFrom<Vec<HashSet<ChainId>>> for EquivalentChainIds {
    type Error = EquivalentChainIdsError;

    fn try_from(value: Vec<HashSet<ChainId>>) -> Result<Self, Self::Error> {
        let mut all_ids = value.iter().flatten().collect::<Vec<_>>();

        all_ids.sort();

        let (_, dups) = all_ids.partition_dedup();

        if dups.is_empty() {
            Ok(Self { inner: value })
        } else {
            Err(EquivalentChainIdsError {
                duplicates: dups.iter().map(|&c| c.clone()).collect(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error(
    "duplicated chain IDs in chain ID equivalence lists: {}",
    duplicates.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "),
)]
pub struct EquivalentChainIdsError {
    pub duplicates: HashSet<ChainId>,
}
