use core::hash::{BuildHasher, Hash};

use super::{Bucket, Entries, Equivalent, IndexMap};

/// Opt-in mutable access to keys.
///
/// These methods expose `&mut K`, mutable references to the key as it is stored
/// in the map.
/// You are allowed to modify the keys in the hashmap **if the modification
/// does not change the key’s hash and equality**.
///
/// If keys are modified erroneously, you can no longer look them up.
/// This is sound (memory safe) but a logical error hazard (just like
/// implementing PartialEq, Eq, or Hash incorrectly would be).
///
/// `use` this trait to enable its methods for `IndexMap`.
///
/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait MutableKeys: private::Sealed {
    type Key;
    type Value;

    /// Return item index, mutable reference to key and value
    ///
    /// Computes in **O(1)** time (average).
    fn get_full_mut2<Q: ?Sized>(
        &mut self,
        key: &Q,
    ) -> Option<(usize, &mut Self::Key, &mut Self::Value)>
    where
        Q: Hash + Equivalent<Self::Key>;

    /// Return mutable reference to key and value at an index.
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    fn get_index_mut2(&mut self, index: usize) -> Option<(&mut Self::Key, &mut Self::Value)>;

    /// Scan through each key-value pair in the map and keep those where the
    /// closure `keep` returns `true`.
    ///
    /// The elements are visited in order, and remaining elements keep their
    /// order.
    ///
    /// Computes in **O(n)** time (average).
    fn retain2<F>(&mut self, keep: F)
    where
        F: FnMut(&mut Self::Key, &mut Self::Value) -> bool;
}

/// Opt-in mutable access to keys.
///
/// See [`MutableKeys`](trait.MutableKeys.html) for more information.
impl<K, V, S> MutableKeys for IndexMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    type Key = K;
    type Value = V;

    fn get_full_mut2<Q: ?Sized>(&mut self, key: &Q) -> Option<(usize, &mut K, &mut V)>
    where
        Q: Hash + Equivalent<K>,
    {
        if let Some(i) = self.get_index_of(key) {
            let entry = &mut self.as_entries_mut()[i];
            Some((i, &mut entry.key, &mut entry.value))
        } else {
            None
        }
    }

    fn get_index_mut2(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
        self.as_entries_mut().get_mut(index).map(Bucket::muts)
    }

    fn retain2<F>(&mut self, keep: F)
    where
        F: FnMut(&mut K, &mut V) -> bool,
    {
        self.retain_mut(keep)
    }
}

mod private {
    pub trait Sealed {}

    impl<K, V, S> Sealed for super::IndexMap<K, V, S> {}
}
