#[cfg(feature = "core")]
use core::hash::Hash;
use indexmap::IndexMap;
use rkyv::{Archive, Deserialize, Serialize};
#[cfg(feature = "std")]
use std::hash::Hash;

#[derive(Serialize, Deserialize, Archive)]
/// Rkyv Archivable IndexMap
pub struct ArchivableIndexMap<K: Hash + Ord + Archive, V: Archive> {
    entries: Vec<(K, V)>,
}

impl<K: Hash + Ord + Archive + Clone, V: Archive> From<IndexMap<K, V>>
    for ArchivableIndexMap<K, V>
{
    fn from(it: IndexMap<K, V>) -> Self {
        let mut r = Self {
            entries: Vec::new(),
        };
        for (k, v) in it.into_iter() {
            r.entries.push((k, v));
        }
        r
    }
}

impl<K: Hash + Ord + Archive + Clone, V: Archive> Into<IndexMap<K, V>>
    for ArchivableIndexMap<K, V>
{
    fn into(self) -> IndexMap<K, V> {
        let mut r = IndexMap::new();
        for (k, v) in self.entries.into_iter() {
            r.insert(k, v);
        }
        r
    }
}
