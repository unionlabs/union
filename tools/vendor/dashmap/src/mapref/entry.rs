use super::one::RefMut;
use crate::lock::RwLockWriteGuard;
use crate::util;
use crate::util::SharedValue;
use crate::HashMap;
use core::hash::{BuildHasher, Hash};
use core::mem;
use core::ptr;
use std::collections::hash_map::RandomState;

pub enum Entry<'a, K, V, S = RandomState> {
    Occupied(OccupiedEntry<'a, K, V, S>),
    Vacant(VacantEntry<'a, K, V, S>),
}

impl<'a, K: Eq + Hash, V, S: BuildHasher> Entry<'a, K, V, S> {
    /// Apply a function to the stored value if it exists.
    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());

                Entry::Occupied(entry)
            }

            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    /// Get the key of the entry.
    pub fn key(&self) -> &K {
        match *self {
            Entry::Occupied(ref entry) => entry.key(),
            Entry::Vacant(ref entry) => entry.key(),
        }
    }

    /// Into the key of the entry.
    pub fn into_key(self) -> K {
        match self {
            Entry::Occupied(entry) => entry.into_key(),
            Entry::Vacant(entry) => entry.into_key(),
        }
    }

    /// Return a mutable reference to the element if it exists,
    /// otherwise insert the default and return a mutable reference to that.
    pub fn or_default(self) -> RefMut<'a, K, V, S>
    where
        V: Default,
    {
        match self {
            Entry::Occupied(entry) => entry.into_ref(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }

    /// Return a mutable reference to the element if it exists,
    /// otherwise a provided value and return a mutable reference to that.
    pub fn or_insert(self, value: V) -> RefMut<'a, K, V, S> {
        match self {
            Entry::Occupied(entry) => entry.into_ref(),
            Entry::Vacant(entry) => entry.insert(value),
        }
    }

    /// Return a mutable reference to the element if it exists,
    /// otherwise insert the result of a provided function and return a mutable reference to that.
    pub fn or_insert_with(self, value: impl FnOnce() -> V) -> RefMut<'a, K, V, S> {
        match self {
            Entry::Occupied(entry) => entry.into_ref(),
            Entry::Vacant(entry) => entry.insert(value()),
        }
    }

    pub fn or_try_insert_with<E>(
        self,
        value: impl FnOnce() -> Result<V, E>,
    ) -> Result<RefMut<'a, K, V, S>, E> {
        match self {
            Entry::Occupied(entry) => Ok(entry.into_ref()),
            Entry::Vacant(entry) => Ok(entry.insert(value()?)),
        }
    }

    /// Sets the value of the entry, and returns a reference to the inserted value.
    pub fn insert(self, value: V) -> RefMut<'a, K, V, S> {
        match self {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry.into_ref()
            }
            Entry::Vacant(entry) => entry.insert(value),
        }
    }

    /// Sets the value of the entry, and returns an OccupiedEntry.
    ///
    /// If you are not interested in the occupied entry,
    /// consider [`insert`] as it doesn't need to clone the key.
    ///
    /// [`insert`]: Entry::insert
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, S>
    where
        K: Clone,
    {
        match self {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Entry::Vacant(entry) => entry.insert_entry(value),
        }
    }
}

pub struct VacantEntry<'a, K, V, S = RandomState> {
    shard: RwLockWriteGuard<'a, HashMap<K, V, S>>,
    key: K,
}

unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for VacantEntry<'a, K, V, S> {}
unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for VacantEntry<'a, K, V, S> {}

impl<'a, K: Eq + Hash, V, S: BuildHasher> VacantEntry<'a, K, V, S> {
    pub(crate) unsafe fn new(shard: RwLockWriteGuard<'a, HashMap<K, V, S>>, key: K) -> Self {
        Self { shard, key }
    }

    pub fn insert(mut self, value: V) -> RefMut<'a, K, V, S> {
        unsafe {
            let c: K = ptr::read(&self.key);

            self.shard.insert(self.key, SharedValue::new(value));

            let (k, v) = self.shard.get_key_value(&c).unwrap();

            let k = util::change_lifetime_const(k);

            let v = &mut *v.as_ptr();

            let r = RefMut::new(self.shard, k, v);

            mem::forget(c);

            r
        }
    }

    /// Sets the value of the entry with the VacantEntry’s key, and returns an OccupiedEntry.
    pub fn insert_entry(mut self, value: V) -> OccupiedEntry<'a, K, V, S>
    where
        K: Clone,
    {
        unsafe {
            self.shard.insert(self.key.clone(), SharedValue::new(value));

            let (k, v) = self.shard.get_key_value(&self.key).unwrap();

            let kptr: *const K = k;
            let vptr: *mut V = v.as_ptr();
            OccupiedEntry::new(self.shard, self.key, (kptr, vptr))
        }
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

pub struct OccupiedEntry<'a, K, V, S = RandomState> {
    shard: RwLockWriteGuard<'a, HashMap<K, V, S>>,
    elem: (*const K, *mut V),
    key: K,
}

unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for OccupiedEntry<'a, K, V, S> {}
unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for OccupiedEntry<'a, K, V, S> {}

impl<'a, K: Eq + Hash, V, S: BuildHasher> OccupiedEntry<'a, K, V, S> {
    pub(crate) unsafe fn new(
        shard: RwLockWriteGuard<'a, HashMap<K, V, S>>,
        key: K,
        elem: (*const K, *mut V),
    ) -> Self {
        Self { shard, elem, key }
    }

    pub fn get(&self) -> &V {
        unsafe { &*self.elem.1 }
    }

    pub fn get_mut(&mut self) -> &mut V {
        unsafe { &mut *self.elem.1 }
    }

    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    pub fn into_ref(self) -> RefMut<'a, K, V, S> {
        unsafe { RefMut::new(self.shard, self.elem.0, self.elem.1) }
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn key(&self) -> &K {
        unsafe { &*self.elem.0 }
    }

    pub fn remove(mut self) -> V {
        let key = unsafe { &*self.elem.0 };
        self.shard.remove(key).unwrap().into_inner()
    }

    pub fn remove_entry(mut self) -> (K, V) {
        let key = unsafe { &*self.elem.0 };
        let (k, v) = self.shard.remove_entry(key).unwrap();
        (k, v.into_inner())
    }

    pub fn replace_entry(mut self, value: V) -> (K, V) {
        let nk = self.key;
        let key = unsafe { &*self.elem.0 };
        let (k, v) = self.shard.remove_entry(key).unwrap();
        self.shard.insert(nk, SharedValue::new(value));
        (k, v.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use crate::DashMap;

    use super::*;

    #[test]
    fn test_insert_entry_into_vacant() {
        let map: DashMap<u32, u32> = DashMap::new();

        let entry = map.entry(1);

        assert!(matches!(entry, Entry::Vacant(_)));

        let entry = entry.insert_entry(2);

        assert_eq!(*entry.get(), 2);

        drop(entry);

        assert_eq!(*map.get(&1).unwrap(), 2);
    }

    #[test]
    fn test_insert_entry_into_occupied() {
        let map: DashMap<u32, u32> = DashMap::new();

        map.insert(1, 1000);

        let entry = map.entry(1);

        assert!(matches!(&entry, Entry::Occupied(entry) if *entry.get() == 1000));

        let entry = entry.insert_entry(2);

        assert_eq!(*entry.get(), 2);

        drop(entry);

        assert_eq!(*map.get(&1).unwrap(), 2);
    }
}
