use crate::lock::RwLock;
use crate::mapref::multiple::{RefMulti, RefMutMulti};
use crate::util;
use crate::{DashMap, HashMap};
use core::hash::{BuildHasher, Hash};
use rayon::iter::plumbing::UnindexedConsumer;
use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelExtend, ParallelIterator};
use std::collections::hash_map::RandomState;
use std::sync::Arc;

impl<K, V, S> ParallelExtend<(K, V)> for DashMap<K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        (&*self).par_extend(par_iter);
    }
}

// Since we don't actually need mutability, we can implement this on a
// reference, similar to `io::Write for &File`.
impl<K, V, S> ParallelExtend<(K, V)> for &'_ DashMap<K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        let &mut map = self;
        par_iter.into_par_iter().for_each(move |(key, value)| {
            map.insert(key, value);
        });
    }
}

impl<K, V, S> FromParallelIterator<(K, V)> for DashMap<K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + Default + BuildHasher,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        let map = Self::default();
        (&map).par_extend(par_iter);
        map
    }
}

// Implementation note: while the shards will iterate in parallel, we flatten
// sequentially within each shard (`flat_map_iter`), because the standard
// `HashMap` only implements `ParallelIterator` by collecting to a `Vec` first.
// There is real parallel support in the `hashbrown/rayon` feature, but we don't
// always use that map.

impl<K, V, S> IntoParallelIterator for DashMap<K, V, S>
where
    K: Send + Eq + Hash,
    V: Send,
    S: Send + Clone + BuildHasher,
{
    type Iter = OwningIter<K, V, S>;
    type Item = (K, V);

    fn into_par_iter(self) -> Self::Iter {
        OwningIter {
            shards: self.shards,
        }
    }
}

pub struct OwningIter<K, V, S = RandomState> {
    pub(super) shards: Box<[RwLock<HashMap<K, V, S>>]>,
}

impl<K, V, S> ParallelIterator for OwningIter<K, V, S>
where
    K: Send + Eq + Hash,
    V: Send,
    S: Send + Clone + BuildHasher,
{
    type Item = (K, V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        Vec::from(self.shards)
            .into_par_iter()
            .flat_map_iter(|shard| {
                shard
                    .into_inner()
                    .into_iter()
                    .map(|(k, v)| (k, v.into_inner()))
            })
            .drive_unindexed(consumer)
    }
}

// This impl also enables `IntoParallelRefIterator::par_iter`
impl<'a, K, V, S> IntoParallelIterator for &'a DashMap<K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    type Iter = Iter<'a, K, V, S>;
    type Item = RefMulti<'a, K, V, S>;

    fn into_par_iter(self) -> Self::Iter {
        Iter {
            shards: &self.shards,
        }
    }
}

pub struct Iter<'a, K, V, S = RandomState> {
    pub(super) shards: &'a [RwLock<HashMap<K, V, S>>],
}

impl<'a, K, V, S> ParallelIterator for Iter<'a, K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    type Item = RefMulti<'a, K, V, S>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.shards
            .into_par_iter()
            .flat_map_iter(|shard| {
                let guard = shard.read();
                let sref: &'a HashMap<K, V, S> = unsafe { util::change_lifetime_const(&*guard) };

                let guard = Arc::new(guard);
                sref.iter().map(move |(k, v)| {
                    let guard = Arc::clone(&guard);
                    unsafe { RefMulti::new(guard, k, v.get()) }
                })
            })
            .drive_unindexed(consumer)
    }
}

// This impl also enables `IntoParallelRefMutIterator::par_iter_mut`
impl<'a, K, V, S> IntoParallelIterator for &'a mut DashMap<K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    type Iter = IterMut<'a, K, V, S>;
    type Item = RefMutMulti<'a, K, V, S>;

    fn into_par_iter(self) -> Self::Iter {
        IterMut {
            shards: &self.shards,
        }
    }
}

impl<K, V, S> DashMap<K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    // Unlike `IntoParallelRefMutIterator::par_iter_mut`, we only _need_ `&self`.
    pub fn par_iter_mut(&self) -> IterMut<'_, K, V, S> {
        IterMut {
            shards: &self.shards,
        }
    }
}

pub struct IterMut<'a, K, V, S = RandomState> {
    shards: &'a [RwLock<HashMap<K, V, S>>],
}

impl<'a, K, V, S> ParallelIterator for IterMut<'a, K, V, S>
where
    K: Send + Sync + Eq + Hash,
    V: Send + Sync,
    S: Send + Sync + Clone + BuildHasher,
{
    type Item = RefMutMulti<'a, K, V, S>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.shards
            .into_par_iter()
            .flat_map_iter(|shard| {
                let mut guard = shard.write();
                let sref: &'a mut HashMap<K, V, S> =
                    unsafe { util::change_lifetime_mut(&mut *guard) };

                let guard = Arc::new(guard);
                sref.iter_mut().map(move |(k, v)| {
                    let guard = Arc::clone(&guard);
                    unsafe { RefMutMulti::new(guard, k, v.get_mut()) }
                })
            })
            .drive_unindexed(consumer)
    }
}
