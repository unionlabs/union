//! A hashmap with "external hashing": nodes are hashed or compared for
//! equality only with some external context provided on lookup/insert.
//! This allows very memory-efficient data structures where
//! node-internal data references some other storage (e.g., offsets into
//! an array or pool of shared data).

use super::unionfind::UnionFind;
use hashbrown::raw::{Bucket, RawTable};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Trait that allows for equality comparison given some external
/// context.
///
/// Note that this trait is implemented by the *context*, rather than
/// the item type, for somewhat complex lifetime reasons (lack of GATs
/// to allow `for<'ctx> Ctx<'ctx>`-like associated types in traits on
/// the value type).
///
/// Furthermore, the `ctx_eq` method includes a `UnionFind` parameter,
/// because in practice we require this and a borrow to it cannot be
/// included in the context type without GATs (similarly to above).
pub trait CtxEq<V1: ?Sized, V2: ?Sized> {
    /// Determine whether `a` and `b` are equal, given the context in
    /// `self` and the union-find data structure `uf`.
    fn ctx_eq(&self, a: &V1, b: &V2, uf: &mut UnionFind) -> bool;
}

/// Trait that allows for hashing given some external context.
pub trait CtxHash<Value: ?Sized>: CtxEq<Value, Value> {
    /// Compute the hash of `value`, given the context in `self` and
    /// the union-find data structure `uf`.
    fn ctx_hash(&self, value: &Value, uf: &mut UnionFind) -> u64;
}

/// A null-comparator context type for underlying value types that
/// already have `Eq` and `Hash`.
#[derive(Default)]
pub struct NullCtx;

impl<V: Eq + Hash> CtxEq<V, V> for NullCtx {
    fn ctx_eq(&self, a: &V, b: &V, _: &mut UnionFind) -> bool {
        a.eq(b)
    }
}
impl<V: Eq + Hash> CtxHash<V> for NullCtx {
    fn ctx_hash(&self, value: &V, _: &mut UnionFind) -> u64 {
        let mut state = fxhash::FxHasher::default();
        value.hash(&mut state);
        state.finish()
    }
}

/// A bucket in the hash table.
///
/// Some performance-related design notes: we cache the hashcode for
/// speed, as this often buys a few percent speed in
/// interning-table-heavy workloads. We only keep the low 32 bits of
/// the hashcode, for memory efficiency: in common use, `K` and `V`
/// are often 32 bits also, and a 12-byte bucket is measurably better
/// than a 16-byte bucket.
struct BucketData<K, V> {
    hash: u32,
    k: K,
    v: V,
}

/// A HashMap that takes external context for all operations.
pub struct CtxHashMap<K, V> {
    raw: RawTable<BucketData<K, V>>,
}

impl<K, V> CtxHashMap<K, V> {
    /// Create an empty hashmap.
    pub fn new() -> Self {
        Self {
            raw: RawTable::new(),
        }
    }

    /// Create an empty hashmap with pre-allocated space for the given
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: RawTable::with_capacity(capacity),
        }
    }
}

impl<K, V> CtxHashMap<K, V> {
    /// Insert a new key-value pair, returning the old value associated
    /// with this key (if any).
    pub fn insert<Ctx: CtxEq<K, K> + CtxHash<K>>(
        &mut self,
        k: K,
        v: V,
        ctx: &Ctx,
        uf: &mut UnionFind,
    ) -> Option<V> {
        let hash = ctx.ctx_hash(&k, uf) as u32;
        match self.raw.find(hash as u64, |bucket| {
            hash == bucket.hash && ctx.ctx_eq(&bucket.k, &k, uf)
        }) {
            Some(bucket) => {
                let data = unsafe { bucket.as_mut() };
                Some(std::mem::replace(&mut data.v, v))
            }
            None => {
                let data = BucketData { hash, k, v };
                self.raw
                    .insert_entry(hash as u64, data, |bucket| bucket.hash as u64);
                None
            }
        }
    }

    /// Look up a key, returning a borrow of the value if present.
    pub fn get<'a, Q, Ctx: CtxEq<K, Q> + CtxHash<Q> + CtxHash<K>>(
        &'a self,
        k: &Q,
        ctx: &Ctx,
        uf: &mut UnionFind,
    ) -> Option<&'a V> {
        let hash = ctx.ctx_hash(k, uf) as u32;
        self.raw
            .find(hash as u64, |bucket| {
                hash == bucket.hash && ctx.ctx_eq(&bucket.k, k, uf)
            })
            .map(|bucket| {
                let data = unsafe { bucket.as_ref() };
                &data.v
            })
    }

    /// Return an Entry cursor on a given bucket for a key, allowing
    /// for fetching the current value or inserting a new one.
    #[inline(always)]
    pub fn entry<'a, Ctx: CtxEq<K, K> + CtxHash<K>>(
        &'a mut self,
        k: K,
        ctx: &'a Ctx,
        uf: &mut UnionFind,
    ) -> Entry<'a, K, V> {
        let hash = ctx.ctx_hash(&k, uf) as u32;
        match self.raw.find(hash as u64, |bucket| {
            hash == bucket.hash && ctx.ctx_eq(&bucket.k, &k, uf)
        }) {
            Some(bucket) => Entry::Occupied(OccupiedEntry {
                bucket,
                _phantom: PhantomData,
            }),
            None => Entry::Vacant(VacantEntry {
                raw: &mut self.raw,
                hash,
                key: k,
            }),
        }
    }
}

/// An entry in the hashmap.
pub enum Entry<'a, K: 'a, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

/// An occupied entry.
pub struct OccupiedEntry<'a, K, V> {
    bucket: Bucket<BucketData<K, V>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, K: 'a, V> OccupiedEntry<'a, K, V> {
    /// Get the value.
    pub fn get(&self) -> &'a V {
        let bucket = unsafe { self.bucket.as_ref() };
        &bucket.v
    }
}

/// A vacant entry.
pub struct VacantEntry<'a, K, V> {
    raw: &'a mut RawTable<BucketData<K, V>>,
    hash: u32,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    /// Insert a value.
    pub fn insert(self, v: V) -> &'a V {
        let bucket = self.raw.insert(
            self.hash as u64,
            BucketData {
                hash: self.hash,
                k: self.key,
                v,
            },
            |bucket| bucket.hash as u64,
        );
        let data = unsafe { bucket.as_ref() };
        &data.v
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::hash::Hash;

    #[derive(Clone, Copy, Debug)]
    struct Key {
        index: u32,
    }
    struct Ctx {
        vals: &'static [&'static str],
    }
    impl CtxEq<Key, Key> for Ctx {
        fn ctx_eq(&self, a: &Key, b: &Key, _: &mut UnionFind) -> bool {
            self.vals[a.index as usize].eq(self.vals[b.index as usize])
        }
    }
    impl CtxHash<Key> for Ctx {
        fn ctx_hash(&self, value: &Key, _: &mut UnionFind) -> u64 {
            let mut state = fxhash::FxHasher::default();
            self.vals[value.index as usize].hash(&mut state);
            state.finish()
        }
    }

    #[test]
    fn test_basic() {
        let ctx = Ctx {
            vals: &["a", "b", "a"],
        };
        let mut uf = UnionFind::new();

        let k0 = Key { index: 0 };
        let k1 = Key { index: 1 };
        let k2 = Key { index: 2 };

        assert!(ctx.ctx_eq(&k0, &k2, &mut uf));
        assert!(!ctx.ctx_eq(&k0, &k1, &mut uf));
        assert!(!ctx.ctx_eq(&k2, &k1, &mut uf));

        let mut map: CtxHashMap<Key, u64> = CtxHashMap::new();
        assert_eq!(map.insert(k0, 42, &ctx, &mut uf), None);
        assert_eq!(map.insert(k2, 84, &ctx, &mut uf), Some(42));
        assert_eq!(map.get(&k1, &ctx, &mut uf), None);
        assert_eq!(*map.get(&k0, &ctx, &mut uf).unwrap(), 84);
    }

    #[test]
    fn test_entry() {
        let mut ctx = Ctx {
            vals: &["a", "b", "a"],
        };
        let mut uf = UnionFind::new();

        let k0 = Key { index: 0 };
        let k1 = Key { index: 1 };
        let k2 = Key { index: 2 };

        let mut map: CtxHashMap<Key, u64> = CtxHashMap::new();
        match map.entry(k0, &mut ctx, &mut uf) {
            Entry::Vacant(v) => {
                v.insert(1);
            }
            _ => panic!(),
        }
        match map.entry(k1, &mut ctx, &mut uf) {
            Entry::Vacant(_) => {}
            Entry::Occupied(_) => panic!(),
        }
        match map.entry(k2, &mut ctx, &mut uf) {
            Entry::Occupied(o) => {
                assert_eq!(*o.get(), 1);
            }
            _ => panic!(),
        }
    }
}
