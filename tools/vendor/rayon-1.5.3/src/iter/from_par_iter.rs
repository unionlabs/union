use super::noop::NoopConsumer;
use super::{FromParallelIterator, IntoParallelIterator, ParallelExtend, ParallelIterator};

use std::borrow::Cow;
use std::collections::LinkedList;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::collections::{BinaryHeap, VecDeque};
use std::hash::{BuildHasher, Hash};

/// Creates an empty default collection and extends it.
fn collect_extended<C, I>(par_iter: I) -> C
where
    I: IntoParallelIterator,
    C: ParallelExtend<I::Item> + Default,
{
    let mut collection = C::default();
    collection.par_extend(par_iter);
    collection
}

/// Collects items from a parallel iterator into a vector.
impl<T> FromParallelIterator<T> for Vec<T>
where
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        collect_extended(par_iter)
    }
}

/// Collects items from a parallel iterator into a vecdeque.
impl<T> FromParallelIterator<T> for VecDeque<T>
where
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        Vec::from_par_iter(par_iter).into()
    }
}

/// Collects items from a parallel iterator into a binaryheap.
/// The heap-ordering is calculated serially after all items are collected.
impl<T> FromParallelIterator<T> for BinaryHeap<T>
where
    T: Ord + Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        Vec::from_par_iter(par_iter).into()
    }
}

/// Collects items from a parallel iterator into a freshly allocated
/// linked list.
impl<T> FromParallelIterator<T> for LinkedList<T>
where
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        collect_extended(par_iter)
    }
}

/// Collects (key, value) pairs from a parallel iterator into a
/// hashmap. If multiple pairs correspond to the same key, then the
/// ones produced earlier in the parallel iterator will be
/// overwritten, just as with a sequential iterator.
impl<K, V, S> FromParallelIterator<(K, V)> for HashMap<K, V, S>
where
    K: Eq + Hash + Send,
    V: Send,
    S: BuildHasher + Default + Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        collect_extended(par_iter)
    }
}

/// Collects (key, value) pairs from a parallel iterator into a
/// btreemap. If multiple pairs correspond to the same key, then the
/// ones produced earlier in the parallel iterator will be
/// overwritten, just as with a sequential iterator.
impl<K, V> FromParallelIterator<(K, V)> for BTreeMap<K, V>
where
    K: Ord + Send,
    V: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (K, V)>,
    {
        collect_extended(par_iter)
    }
}

/// Collects values from a parallel iterator into a hashset.
impl<V, S> FromParallelIterator<V> for HashSet<V, S>
where
    V: Eq + Hash + Send,
    S: BuildHasher + Default + Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = V>,
    {
        collect_extended(par_iter)
    }
}

/// Collects values from a parallel iterator into a btreeset.
impl<V> FromParallelIterator<V> for BTreeSet<V>
where
    V: Send + Ord,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = V>,
    {
        collect_extended(par_iter)
    }
}

/// Collects characters from a parallel iterator into a string.
impl FromParallelIterator<char> for String {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = char>,
    {
        collect_extended(par_iter)
    }
}

/// Collects characters from a parallel iterator into a string.
impl<'a> FromParallelIterator<&'a char> for String {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = &'a char>,
    {
        collect_extended(par_iter)
    }
}

/// Collects string slices from a parallel iterator into a string.
impl<'a> FromParallelIterator<&'a str> for String {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = &'a str>,
    {
        collect_extended(par_iter)
    }
}

/// Collects strings from a parallel iterator into one large string.
impl FromParallelIterator<String> for String {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = String>,
    {
        collect_extended(par_iter)
    }
}

/// Collects string slices from a parallel iterator into a string.
impl<'a> FromParallelIterator<Cow<'a, str>> for String {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = Cow<'a, str>>,
    {
        collect_extended(par_iter)
    }
}

/// Collects an arbitrary `Cow` collection.
///
/// Note, the standard library only has `FromIterator` for `Cow<'a, str>` and
/// `Cow<'a, [T]>`, because no one thought to add a blanket implementation
/// before it was stabilized.
impl<'a, C: ?Sized, T> FromParallelIterator<T> for Cow<'a, C>
where
    C: ToOwned,
    C::Owned: FromParallelIterator<T>,
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        Cow::Owned(C::Owned::from_par_iter(par_iter))
    }
}

/// Collapses all unit items from a parallel iterator into one.
///
/// This is more useful when combined with higher-level abstractions, like
/// collecting to a `Result<(), E>` where you only care about errors:
///
/// ```
/// use std::io::*;
/// use rayon::prelude::*;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let res: Result<()> = data.par_iter()
///     .map(|x| writeln!(stdout(), "{}", x))
///     .collect();
/// assert!(res.is_ok());
/// ```
impl FromParallelIterator<()> for () {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = ()>,
    {
        par_iter.into_par_iter().drive_unindexed(NoopConsumer)
    }
}
