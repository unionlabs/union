//! This module contains the parallel iterator types for heaps
//! (`BinaryHeap<T>`). You will rarely need to interact with it directly
//! unless you have need to name one of the iterator types.

use std::collections::BinaryHeap;

use crate::iter::plumbing::*;
use crate::iter::*;

use crate::vec;

/// Parallel iterator over a binary heap
#[derive(Debug, Clone)]
pub struct IntoIter<T: Ord + Send> {
    inner: vec::IntoIter<T>,
}

impl<T: Ord + Send> IntoParallelIterator for BinaryHeap<T> {
    type Item = T;
    type Iter = IntoIter<T>;

    fn into_par_iter(self) -> Self::Iter {
        IntoIter {
            inner: Vec::from(self).into_par_iter(),
        }
    }
}

delegate_indexed_iterator! {
    IntoIter<T> => T,
    impl<T: Ord + Send>
}

/// Parallel iterator over an immutable reference to a binary heap
#[derive(Debug)]
pub struct Iter<'a, T: Ord + Sync> {
    inner: vec::IntoIter<&'a T>,
}

impl<'a, T: Ord + Sync> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter {
            inner: self.inner.clone(),
        }
    }
}

into_par_vec! {
    &'a BinaryHeap<T> => Iter<'a, T>,
    impl<'a, T: Ord + Sync>
}

delegate_indexed_iterator! {
    Iter<'a, T> => &'a T,
    impl<'a, T: Ord + Sync + 'a>
}

// `BinaryHeap` doesn't have a mutable `Iterator`

/// Draining parallel iterator that moves out of a binary heap,
/// but keeps the total capacity.
#[derive(Debug)]
pub struct Drain<'a, T: Ord + Send> {
    heap: &'a mut BinaryHeap<T>,
}

impl<'a, T: Ord + Send> ParallelDrainFull for &'a mut BinaryHeap<T> {
    type Iter = Drain<'a, T>;
    type Item = T;

    fn par_drain(self) -> Self::Iter {
        Drain { heap: self }
    }
}

impl<'a, T: Ord + Send> ParallelIterator for Drain<'a, T> {
    type Item = T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'a, T: Ord + Send> IndexedParallelIterator for Drain<'a, T> {
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.heap.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        super::DrainGuard::new(self.heap)
            .par_drain(..)
            .with_producer(callback)
    }
}

impl<'a, T: Ord + Send> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        if !self.heap.is_empty() {
            // We must not have produced, so just call a normal drain to remove the items.
            self.heap.drain();
        }
    }
}
