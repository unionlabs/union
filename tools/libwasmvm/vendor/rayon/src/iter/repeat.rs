use super::plumbing::*;
use super::*;
use std::iter;
use std::usize;

/// Iterator adaptor for [the `repeat()` function](fn.repeat.html).
#[derive(Debug, Clone)]
pub struct Repeat<T: Clone + Send> {
    element: T,
}

/// Creates a parallel iterator that endlessly repeats `elt` (by
/// cloning it). Note that this iterator has "infinite" length, so
/// typically you would want to use `zip` or `take` or some other
/// means to shorten it, or consider using
/// [the `repeatn()` function](fn.repeatn.html) instead.
///
/// # Examples
///
/// ```
/// use rayon::prelude::*;
/// use rayon::iter::repeat;
/// let x: Vec<(i32, i32)> = repeat(22).zip(0..3).collect();
/// assert_eq!(x, vec![(22, 0), (22, 1), (22, 2)]);
/// ```
pub fn repeat<T: Clone + Send>(elt: T) -> Repeat<T> {
    Repeat { element: elt }
}

impl<T> Repeat<T>
where
    T: Clone + Send,
{
    /// Takes only `n` repeats of the element, similar to the general
    /// [`take()`](trait.IndexedParallelIterator.html#method.take).
    ///
    /// The resulting `RepeatN` is an `IndexedParallelIterator`, allowing
    /// more functionality than `Repeat` alone.
    pub fn take(self, n: usize) -> RepeatN<T> {
        repeatn(self.element, n)
    }

    /// Iterates tuples, repeating the element with items from another
    /// iterator, similar to the general
    /// [`zip()`](trait.IndexedParallelIterator.html#method.zip).
    pub fn zip<Z>(self, zip_op: Z) -> Zip<RepeatN<T>, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Iter: IndexedParallelIterator,
    {
        let z = zip_op.into_par_iter();
        let n = z.len();
        self.take(n).zip(z)
    }
}

impl<T> ParallelIterator for Repeat<T>
where
    T: Clone + Send,
{
    type Item = T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let producer = RepeatProducer {
            element: self.element,
        };
        bridge_unindexed(producer, consumer)
    }
}

/// Unindexed producer for `Repeat`.
struct RepeatProducer<T: Clone + Send> {
    element: T,
}

impl<T: Clone + Send> UnindexedProducer for RepeatProducer<T> {
    type Item = T;

    fn split(self) -> (Self, Option<Self>) {
        (
            RepeatProducer {
                element: self.element.clone(),
            },
            Some(RepeatProducer {
                element: self.element,
            }),
        )
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<T>,
    {
        folder.consume_iter(iter::repeat(self.element))
    }
}

/// Iterator adaptor for [the `repeatn()` function](fn.repeatn.html).
#[derive(Debug, Clone)]
pub struct RepeatN<T: Clone + Send> {
    element: T,
    count: usize,
}

/// Creates a parallel iterator that produces `n` repeats of `elt`
/// (by cloning it).
///
/// # Examples
///
/// ```
/// use rayon::prelude::*;
/// use rayon::iter::repeatn;
/// let x: Vec<(i32, i32)> = repeatn(22, 3).zip(0..3).collect();
/// assert_eq!(x, vec![(22, 0), (22, 1), (22, 2)]);
/// ```
pub fn repeatn<T: Clone + Send>(elt: T, n: usize) -> RepeatN<T> {
    RepeatN {
        element: elt,
        count: n,
    }
}

impl<T> ParallelIterator for RepeatN<T>
where
    T: Clone + Send,
{
    type Item = T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.count)
    }
}

impl<T> IndexedParallelIterator for RepeatN<T>
where
    T: Clone + Send,
{
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(RepeatNProducer {
            element: self.element,
            count: self.count,
        })
    }

    fn len(&self) -> usize {
        self.count
    }
}

/// Producer for `RepeatN`.
struct RepeatNProducer<T: Clone + Send> {
    element: T,
    count: usize,
}

impl<T: Clone + Send> Producer for RepeatNProducer<T> {
    type Item = T;
    type IntoIter = Iter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            element: self.element,
            count: self.count,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        (
            RepeatNProducer {
                element: self.element.clone(),
                count: index,
            },
            RepeatNProducer {
                element: self.element,
                count: self.count - index,
            },
        )
    }
}

/// Iterator for `RepeatN`.
///
/// This is conceptually like `std::iter::Take<std::iter::Repeat<T>>`, but
/// we need `DoubleEndedIterator` and unconditional `ExactSizeIterator`.
struct Iter<T: Clone> {
    element: T,
    count: usize,
}

impl<T: Clone> Iterator for Iter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.count > 0 {
            self.count -= 1;
            Some(self.element.clone())
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<T: Clone> DoubleEndedIterator for Iter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.next()
    }
}

impl<T: Clone> ExactSizeIterator for Iter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}
