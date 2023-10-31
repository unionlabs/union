use super::{Bucket, Entries, IndexSet, Slice};

use alloc::vec::{self, Vec};
use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::iter::{Chain, FusedIterator};
use core::slice::Iter as SliceIter;

impl<'a, T, S> IntoIterator for &'a IndexSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S> IntoIterator for IndexSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.into_entries())
    }
}

/// An iterator over the items of a `IndexSet`.
///
/// This `struct` is created by the [`iter`] method on [`IndexSet`].
/// See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`iter`]: struct.IndexSet.html#method.iter
pub struct Iter<'a, T> {
    iter: SliceIter<'a, Bucket<T>>,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(entries: &'a [Bucket<T>]) -> Self {
        Self {
            iter: entries.iter(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &'a Slice<T> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    iterator_methods!(Bucket::key_ref);
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    double_ended_iterator_methods!(Bucket::key_ref);
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        Self { iter: [].iter() }
    }
}

/// An owning iterator over the items of a `IndexSet`.
///
/// This `struct` is created by the [`into_iter`] method on [`IndexSet`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`into_iter`]: struct.IndexSet.html#method.into_iter
pub struct IntoIter<T> {
    iter: vec::IntoIter<Bucket<T>>,
}

impl<T> IntoIter<T> {
    pub(super) fn new(entries: Vec<Bucket<T>>) -> Self {
        Self {
            iter: entries.into_iter(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<T> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    iterator_methods!(Bucket::key);
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    double_ended_iterator_methods!(Bucket::key);
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
        f.debug_list().entries(iter).finish()
    }
}

impl<T> Default for IntoIter<T> {
    fn default() -> Self {
        Self {
            iter: Vec::new().into_iter(),
        }
    }
}

/// A draining iterator over the items of a `IndexSet`.
///
/// This `struct` is created by the [`drain`] method on [`IndexSet`].
/// See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`drain`]: struct.IndexSet.html#method.drain
pub struct Drain<'a, T> {
    iter: vec::Drain<'a, Bucket<T>>,
}

impl<'a, T> Drain<'a, T> {
    pub(super) fn new(iter: vec::Drain<'a, Bucket<T>>) -> Self {
        Self { iter }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<T> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    iterator_methods!(Bucket::key);
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
    double_ended_iterator_methods!(Bucket::key);
}

impl<T> ExactSizeIterator for Drain<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for Drain<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for Drain<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
        f.debug_list().entries(iter).finish()
    }
}

/// A lazy iterator producing elements in the difference of `IndexSet`s.
///
/// This `struct` is created by the [`difference`] method on [`IndexSet`].
/// See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`difference`]: struct.IndexSet.html#method.difference
pub struct Difference<'a, T, S> {
    iter: Iter<'a, T>,
    other: &'a IndexSet<T, S>,
}

impl<'a, T, S> Difference<'a, T, S> {
    pub(super) fn new<S1>(set: &'a IndexSet<T, S1>, other: &'a IndexSet<T, S>) -> Self {
        Self {
            iter: set.iter(),
            other,
        }
    }
}

impl<'a, T, S> Iterator for Difference<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            if !self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<T, S> DoubleEndedIterator for Difference<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next_back() {
            if !self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }
}

impl<T, S> FusedIterator for Difference<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> Clone for Difference<'_, T, S> {
    fn clone(&self) -> Self {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<T, S> fmt::Debug for Difference<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the intersection of `IndexSet`s.
///
/// This `struct` is created by the [`intersection`] method on [`IndexSet`].
/// See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`intersection`]: struct.IndexSet.html#method.intersection
pub struct Intersection<'a, T, S> {
    iter: Iter<'a, T>,
    other: &'a IndexSet<T, S>,
}

impl<'a, T, S> Intersection<'a, T, S> {
    pub(super) fn new<S1>(set: &'a IndexSet<T, S1>, other: &'a IndexSet<T, S>) -> Self {
        Self {
            iter: set.iter(),
            other,
        }
    }
}

impl<'a, T, S> Iterator for Intersection<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            if self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }
}

impl<T, S> DoubleEndedIterator for Intersection<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next_back() {
            if self.other.contains(item) {
                return Some(item);
            }
        }
        None
    }
}

impl<T, S> FusedIterator for Intersection<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> Clone for Intersection<'_, T, S> {
    fn clone(&self) -> Self {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<T, S> fmt::Debug for Intersection<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the symmetric difference of `IndexSet`s.
///
/// This `struct` is created by the [`symmetric_difference`] method on
/// [`IndexSet`]. See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`symmetric_difference`]: struct.IndexSet.html#method.symmetric_difference
pub struct SymmetricDifference<'a, T, S1, S2> {
    iter: Chain<Difference<'a, T, S2>, Difference<'a, T, S1>>,
}

impl<'a, T, S1, S2> SymmetricDifference<'a, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    pub(super) fn new(set1: &'a IndexSet<T, S1>, set2: &'a IndexSet<T, S2>) -> Self {
        let diff1 = set1.difference(set2);
        let diff2 = set2.difference(set1);
        Self {
            iter: diff1.chain(diff2),
        }
    }
}

impl<'a, T, S1, S2> Iterator for SymmetricDifference<'a, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl<T, S1, S2> DoubleEndedIterator for SymmetricDifference<'_, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<T, S1, S2> FusedIterator for SymmetricDifference<'_, T, S1, S2>
where
    T: Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
}

impl<T, S1, S2> Clone for SymmetricDifference<'_, T, S1, S2> {
    fn clone(&self) -> Self {
        SymmetricDifference {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S1, S2> fmt::Debug for SymmetricDifference<'_, T, S1, S2>
where
    T: fmt::Debug + Eq + Hash,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// A lazy iterator producing elements in the union of `IndexSet`s.
///
/// This `struct` is created by the [`union`] method on [`IndexSet`].
/// See its documentation for more.
///
/// [`IndexSet`]: struct.IndexSet.html
/// [`union`]: struct.IndexSet.html#method.union
pub struct Union<'a, T, S> {
    iter: Chain<Iter<'a, T>, Difference<'a, T, S>>,
}

impl<'a, T, S> Union<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    pub(super) fn new<S2>(set1: &'a IndexSet<T, S>, set2: &'a IndexSet<T, S2>) -> Self
    where
        S2: BuildHasher,
    {
        Self {
            iter: set1.iter().chain(set2.difference(set1)),
        }
    }
}

impl<'a, T, S> Iterator for Union<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, f)
    }
}

impl<T, S> DoubleEndedIterator for Union<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn rfold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, f)
    }
}

impl<T, S> FusedIterator for Union<'_, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> Clone for Union<'_, T, S> {
    fn clone(&self) -> Self {
        Union {
            iter: self.iter.clone(),
        }
    }
}

impl<T, S> fmt::Debug for Union<'_, T, S>
where
    T: fmt::Debug + Eq + Hash,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
