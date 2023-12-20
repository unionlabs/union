use crate::setref::multiple::RefMulti;
use crate::DashSet;
use core::hash::{BuildHasher, Hash};
use rayon::iter::plumbing::UnindexedConsumer;
use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelExtend, ParallelIterator};
use std::collections::hash_map::RandomState;

impl<K, S> ParallelExtend<K> for DashSet<K, S>
where
    K: Send + Sync + Eq + Hash,
    S: Send + Sync + Clone + BuildHasher,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = K>,
    {
        (&*self).par_extend(par_iter);
    }
}

// Since we don't actually need mutability, we can implement this on a
// reference, similar to `io::Write for &File`.
impl<K, S> ParallelExtend<K> for &'_ DashSet<K, S>
where
    K: Send + Sync + Eq + Hash,
    S: Send + Sync + Clone + BuildHasher,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = K>,
    {
        let &mut set = self;
        par_iter.into_par_iter().for_each(move |key| {
            set.insert(key);
        });
    }
}

impl<K, S> FromParallelIterator<K> for DashSet<K, S>
where
    K: Send + Sync + Eq + Hash,
    S: Send + Sync + Clone + Default + BuildHasher,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = K>,
    {
        let set = Self::default();
        (&set).par_extend(par_iter);
        set
    }
}

impl<K, S> IntoParallelIterator for DashSet<K, S>
where
    K: Send + Eq + Hash,
    S: Send + Clone + BuildHasher,
{
    type Iter = OwningIter<K, S>;
    type Item = K;

    fn into_par_iter(self) -> Self::Iter {
        OwningIter {
            inner: self.inner.into_par_iter(),
        }
    }
}

pub struct OwningIter<K, S = RandomState> {
    inner: super::map::OwningIter<K, (), S>,
}

impl<K, S> ParallelIterator for OwningIter<K, S>
where
    K: Send + Eq + Hash,
    S: Send + Clone + BuildHasher,
{
    type Item = K;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.inner.map(|(k, _)| k).drive_unindexed(consumer)
    }
}

// This impl also enables `IntoParallelRefIterator::par_iter`
impl<'a, K, S> IntoParallelIterator for &'a DashSet<K, S>
where
    K: Send + Sync + Eq + Hash,
    S: Send + Sync + Clone + BuildHasher,
{
    type Iter = Iter<'a, K, S>;
    type Item = RefMulti<'a, K, S>;

    fn into_par_iter(self) -> Self::Iter {
        Iter {
            inner: (&self.inner).into_par_iter(),
        }
    }
}

pub struct Iter<'a, K, S = RandomState> {
    inner: super::map::Iter<'a, K, (), S>,
}

impl<'a, K, S> ParallelIterator for Iter<'a, K, S>
where
    K: Send + Sync + Eq + Hash,
    S: Send + Sync + Clone + BuildHasher,
{
    type Item = RefMulti<'a, K, S>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.inner.map(RefMulti::new).drive_unindexed(consumer)
    }
}
