use super::{Bucket, Entries, IndexMap, Slice};

use alloc::vec::{self, Vec};
use core::fmt;
use core::iter::FusedIterator;
use core::slice;

impl<'a, K, V, S> IntoIterator for &'a IndexMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut IndexMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V, S> IntoIterator for IndexMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.into_entries())
    }
}

/// An iterator over the entries of a `IndexMap`.
///
/// This `struct` is created by the [`iter`] method on [`IndexMap`]. See its
/// documentation for more.
///
/// [`iter`]: struct.IndexMap.html#method.iter
/// [`IndexMap`]: struct.IndexMap.html
pub struct Iter<'a, K, V> {
    iter: slice::Iter<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(super) fn new(entries: &'a [Bucket<K, V>]) -> Self {
        Self {
            iter: entries.iter(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &'a Slice<K, V> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    iterator_methods!(Bucket::refs);
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    double_ended_iterator_methods!(Bucket::refs);
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, V> Default for Iter<'_, K, V> {
    fn default() -> Self {
        Self { iter: [].iter() }
    }
}

/// A mutable iterator over the entries of a `IndexMap`.
///
/// This `struct` is created by the [`iter_mut`] method on [`IndexMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: struct.IndexMap.html#method.iter_mut
/// [`IndexMap`]: struct.IndexMap.html
pub struct IterMut<'a, K, V> {
    iter: slice::IterMut<'a, Bucket<K, V>>,
}

impl<'a, K, V> IterMut<'a, K, V> {
    pub(super) fn new(entries: &'a mut [Bucket<K, V>]) -> Self {
        Self {
            iter: entries.iter_mut(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<K, V> {
        Slice::from_slice(self.iter.as_slice())
    }

    /// Returns a mutable slice of the remaining entries in the iterator.
    ///
    /// To avoid creating `&mut` references that alias, this is forced to consume the iterator.
    pub fn into_slice(self) -> &'a mut Slice<K, V> {
        Slice::from_mut_slice(self.iter.into_slice())
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    iterator_methods!(Bucket::ref_mut);
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    double_ended_iterator_methods!(Bucket::ref_mut);
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::refs);
        f.debug_list().entries(iter).finish()
    }
}

impl<K, V> Default for IterMut<'_, K, V> {
    fn default() -> Self {
        Self {
            iter: [].iter_mut(),
        }
    }
}

/// An owning iterator over the entries of a `IndexMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`IndexMap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.IndexMap.html#method.into_iter
/// [`IndexMap`]: struct.IndexMap.html
pub struct IntoIter<K, V> {
    iter: vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> IntoIter<K, V> {
    pub(super) fn new(entries: Vec<Bucket<K, V>>) -> Self {
        Self {
            iter: entries.into_iter(),
        }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<K, V> {
        Slice::from_slice(self.iter.as_slice())
    }

    /// Returns a mutable slice of the remaining entries in the iterator.
    pub fn as_mut_slice(&mut self) -> &mut Slice<K, V> {
        Slice::from_mut_slice(self.iter.as_mut_slice())
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    iterator_methods!(Bucket::key_value);
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    double_ended_iterator_methods!(Bucket::key_value);
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::refs);
        f.debug_list().entries(iter).finish()
    }
}

impl<K, V> Default for IntoIter<K, V> {
    fn default() -> Self {
        Self {
            iter: Vec::new().into_iter(),
        }
    }
}

/// A draining iterator over the entries of a `IndexMap`.
///
/// This `struct` is created by the [`drain`] method on [`IndexMap`]. See its
/// documentation for more.
///
/// [`drain`]: struct.IndexMap.html#method.drain
/// [`IndexMap`]: struct.IndexMap.html
pub struct Drain<'a, K, V> {
    iter: vec::Drain<'a, Bucket<K, V>>,
}

impl<'a, K, V> Drain<'a, K, V> {
    pub(super) fn new(iter: vec::Drain<'a, Bucket<K, V>>) -> Self {
        Self { iter }
    }

    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &Slice<K, V> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<K, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    iterator_methods!(Bucket::key_value);
}

impl<K, V> DoubleEndedIterator for Drain<'_, K, V> {
    double_ended_iterator_methods!(Bucket::key_value);
}

impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Drain<'_, K, V> {}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Drain<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::refs);
        f.debug_list().entries(iter).finish()
    }
}

/// An iterator over the keys of a `IndexMap`.
///
/// This `struct` is created by the [`keys`] method on [`IndexMap`]. See its
/// documentation for more.
///
/// [`keys`]: struct.IndexMap.html#method.keys
/// [`IndexMap`]: struct.IndexMap.html
pub struct Keys<'a, K, V> {
    iter: slice::Iter<'a, Bucket<K, V>>,
}

impl<'a, K, V> Keys<'a, K, V> {
    pub(super) fn new(entries: &'a [Bucket<K, V>]) -> Self {
        Self {
            iter: entries.iter(),
        }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    iterator_methods!(Bucket::key_ref);
}

impl<K, V> DoubleEndedIterator for Keys<'_, K, V> {
    double_ended_iterator_methods!(Bucket::key_ref);
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<K, V> Clone for Keys<'_, K, V> {
    fn clone(&self) -> Self {
        Keys {
            iter: self.iter.clone(),
        }
    }
}

impl<K: fmt::Debug, V> fmt::Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, V> Default for Keys<'_, K, V> {
    fn default() -> Self {
        Self { iter: [].iter() }
    }
}

/// An owning iterator over the keys of a `IndexMap`.
///
/// This `struct` is created by the [`into_keys`] method on [`IndexMap`].
/// See its documentation for more.
///
/// [`IndexMap`]: struct.IndexMap.html
/// [`into_keys`]: struct.IndexMap.html#method.into_keys
pub struct IntoKeys<K, V> {
    iter: vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> IntoKeys<K, V> {
    pub(super) fn new(entries: Vec<Bucket<K, V>>) -> Self {
        Self {
            iter: entries.into_iter(),
        }
    }
}

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    iterator_methods!(Bucket::key);
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    double_ended_iterator_methods!(Bucket::key);
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

impl<K: fmt::Debug, V> fmt::Debug for IntoKeys<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
        f.debug_list().entries(iter).finish()
    }
}

impl<K, V> Default for IntoKeys<K, V> {
    fn default() -> Self {
        Self {
            iter: Vec::new().into_iter(),
        }
    }
}

/// An iterator over the values of a `IndexMap`.
///
/// This `struct` is created by the [`values`] method on [`IndexMap`]. See its
/// documentation for more.
///
/// [`values`]: struct.IndexMap.html#method.values
/// [`IndexMap`]: struct.IndexMap.html
pub struct Values<'a, K, V> {
    iter: slice::Iter<'a, Bucket<K, V>>,
}

impl<'a, K, V> Values<'a, K, V> {
    pub(super) fn new(entries: &'a [Bucket<K, V>]) -> Self {
        Self {
            iter: entries.iter(),
        }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    iterator_methods!(Bucket::value_ref);
}

impl<K, V> DoubleEndedIterator for Values<'_, K, V> {
    double_ended_iterator_methods!(Bucket::value_ref);
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<K, V> Clone for Values<'_, K, V> {
    fn clone(&self) -> Self {
        Values {
            iter: self.iter.clone(),
        }
    }
}

impl<K, V: fmt::Debug> fmt::Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<K, V> Default for Values<'_, K, V> {
    fn default() -> Self {
        Self { iter: [].iter() }
    }
}

/// A mutable iterator over the values of a `IndexMap`.
///
/// This `struct` is created by the [`values_mut`] method on [`IndexMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: struct.IndexMap.html#method.values_mut
/// [`IndexMap`]: struct.IndexMap.html
pub struct ValuesMut<'a, K, V> {
    iter: slice::IterMut<'a, Bucket<K, V>>,
}

impl<'a, K, V> ValuesMut<'a, K, V> {
    pub(super) fn new(entries: &'a mut [Bucket<K, V>]) -> Self {
        Self {
            iter: entries.iter_mut(),
        }
    }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    iterator_methods!(Bucket::value_mut);
}

impl<K, V> DoubleEndedIterator for ValuesMut<'_, K, V> {
    double_ended_iterator_methods!(Bucket::value_mut);
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

impl<K, V: fmt::Debug> fmt::Debug for ValuesMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::value_ref);
        f.debug_list().entries(iter).finish()
    }
}

impl<K, V> Default for ValuesMut<'_, K, V> {
    fn default() -> Self {
        Self {
            iter: [].iter_mut(),
        }
    }
}

/// An owning iterator over the values of a `IndexMap`.
///
/// This `struct` is created by the [`into_values`] method on [`IndexMap`].
/// See its documentation for more.
///
/// [`IndexMap`]: struct.IndexMap.html
/// [`into_values`]: struct.IndexMap.html#method.into_values
pub struct IntoValues<K, V> {
    iter: vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> IntoValues<K, V> {
    pub(super) fn new(entries: Vec<Bucket<K, V>>) -> Self {
        Self {
            iter: entries.into_iter(),
        }
    }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    iterator_methods!(Bucket::value);
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    double_ended_iterator_methods!(Bucket::value);
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

impl<K, V: fmt::Debug> fmt::Debug for IntoValues<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.iter.as_slice().iter().map(Bucket::value_ref);
        f.debug_list().entries(iter).finish()
    }
}

impl<K, V> Default for IntoValues<K, V> {
    fn default() -> Self {
        Self {
            iter: Vec::new().into_iter(),
        }
    }
}
