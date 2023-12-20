//! Densely numbered entity references as set keys.

use crate::keys::Keys;
use crate::EntityRef;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// A set of `K` for densely indexed entity references.
///
/// The `EntitySet` data structure uses the dense index space to implement a set with a bitvector.
/// Like `SecondaryMap`, an `EntitySet` is used to associate secondary information with entities.
#[derive(Debug, Clone)]
pub struct EntitySet<K>
where
    K: EntityRef,
{
    elems: Vec<u8>,
    len: usize,
    unused: PhantomData<K>,
}

impl<K: EntityRef> Default for EntitySet<K> {
    fn default() -> Self {
        Self {
            elems: Vec::new(),
            len: 0,
            unused: PhantomData,
        }
    }
}

/// Shared `EntitySet` implementation for all value types.
impl<K> EntitySet<K>
where
    K: EntityRef,
{
    /// Create a new empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new empty set with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elems: Vec::with_capacity((capacity + 7) / 8),
            ..Self::new()
        }
    }

    /// Get the element at `k` if it exists.
    pub fn contains(&self, k: K) -> bool {
        let index = k.index();
        if index < self.len {
            (self.elems[index / 8] & (1 << (index % 8))) != 0
        } else {
            false
        }
    }

    /// Is this set completely empty?
    pub fn is_empty(&self) -> bool {
        if self.len != 0 {
            false
        } else {
            self.elems.iter().all(|&e| e == 0)
        }
    }

    /// Returns the cardinality of the set.  More precisely, it returns the number of calls to
    /// `insert` with different key values, that have happened since the the set was most recently
    /// `clear`ed or created with `new`.
    pub fn cardinality(&self) -> usize {
        let mut n: usize = 0;
        for byte_ix in 0..self.len / 8 {
            n += self.elems[byte_ix].count_ones() as usize;
        }
        for bit_ix in (self.len / 8) * 8..self.len {
            if (self.elems[bit_ix / 8] & (1 << (bit_ix % 8))) != 0 {
                n += 1;
            }
        }
        n
    }

    /// Remove all entries from this set.
    pub fn clear(&mut self) {
        self.len = 0;
        self.elems.clear()
    }

    /// Iterate over all the keys in this set.
    pub fn keys(&self) -> Keys<K> {
        Keys::with_len(self.len)
    }

    /// Resize the set to have `n` entries by adding default entries as needed.
    pub fn resize(&mut self, n: usize) {
        self.elems.resize((n + 7) / 8, 0);
        self.len = n
    }

    /// Insert the element at `k`.
    pub fn insert(&mut self, k: K) -> bool {
        let index = k.index();
        if index >= self.len {
            self.resize(index + 1)
        }
        let result = !self.contains(k);
        self.elems[index / 8] |= 1 << (index % 8);
        result
    }

    /// Removes and returns the entity from the set if it exists.
    pub fn pop(&mut self) -> Option<K> {
        if self.len == 0 {
            return None;
        }

        // Clear the last known entity in the list.
        let last_index = self.len - 1;
        self.elems[last_index / 8] &= !(1 << (last_index % 8));

        // Set the length to the next last stored entity or zero if we pop'ed
        // the last entity.
        self.len = self
            .elems
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &byte)| byte != 0)
            // Map `i` from byte index to bit level index.
            // `(i + 1) * 8` = Last bit in byte.
            // `last - byte.leading_zeros()` = last set bit in byte.
            // `as usize` won't ever truncate as the potential range is `0..=8`.
            .map_or(0, |(i, byte)| ((i + 1) * 8) - byte.leading_zeros() as usize);

        Some(K::new(last_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::u32;

    // `EntityRef` impl for testing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct E(u32);

    impl EntityRef for E {
        fn new(i: usize) -> Self {
            E(i as u32)
        }
        fn index(self) -> usize {
            self.0 as usize
        }
    }

    #[test]
    fn basic() {
        let r0 = E(0);
        let r1 = E(1);
        let r2 = E(2);
        let mut m = EntitySet::new();

        let v: Vec<E> = m.keys().collect();
        assert_eq!(v, []);
        assert!(m.is_empty());

        m.insert(r2);
        m.insert(r1);

        assert!(!m.contains(r0));
        assert!(m.contains(r1));
        assert!(m.contains(r2));
        assert!(!m.contains(E(3)));
        assert!(!m.is_empty());

        let v: Vec<E> = m.keys().collect();
        assert_eq!(v, [r0, r1, r2]);

        m.resize(20);
        assert!(!m.contains(E(3)));
        assert!(!m.contains(E(4)));
        assert!(!m.contains(E(8)));
        assert!(!m.contains(E(15)));
        assert!(!m.contains(E(19)));

        m.insert(E(8));
        m.insert(E(15));
        assert!(!m.contains(E(3)));
        assert!(!m.contains(E(4)));
        assert!(m.contains(E(8)));
        assert!(!m.contains(E(9)));
        assert!(!m.contains(E(14)));
        assert!(m.contains(E(15)));
        assert!(!m.contains(E(16)));
        assert!(!m.contains(E(19)));
        assert!(!m.contains(E(20)));
        assert!(!m.contains(E(u32::MAX)));

        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn pop_ordered() {
        let r0 = E(0);
        let r1 = E(1);
        let r2 = E(2);
        let mut m = EntitySet::new();
        m.insert(r0);
        m.insert(r1);
        m.insert(r2);

        assert_eq!(r2, m.pop().unwrap());
        assert_eq!(r1, m.pop().unwrap());
        assert_eq!(r0, m.pop().unwrap());
        assert!(m.pop().is_none());
        assert!(m.pop().is_none());
    }

    #[test]
    fn pop_unordered() {
        let mut blocks = [
            E(0),
            E(1),
            E(6),
            E(7),
            E(5),
            E(9),
            E(10),
            E(2),
            E(3),
            E(11),
            E(12),
        ];

        let mut m = EntitySet::new();
        for &block in &blocks {
            m.insert(block);
        }
        assert_eq!(m.len, 13);
        blocks.sort();

        for &block in blocks.iter().rev() {
            assert_eq!(block, m.pop().unwrap());
        }

        assert!(m.is_empty());
    }
}
