use crossbeam_deque::{Steal, Stealer, Worker};

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, TryLockError};
use std::thread::yield_now;

use crate::current_num_threads;
use crate::iter::plumbing::{bridge_unindexed, Folder, UnindexedConsumer, UnindexedProducer};
use crate::iter::ParallelIterator;

/// Conversion trait to convert an `Iterator` to a `ParallelIterator`.
///
/// This creates a "bridge" from a sequential iterator to a parallel one, by distributing its items
/// across the Rayon thread pool. This has the advantage of being able to parallelize just about
/// anything, but the resulting `ParallelIterator` can be less efficient than if you started with
/// `par_iter` instead. However, it can still be useful for iterators that are difficult to
/// parallelize by other means, like channels or file or network I/O.
///
/// The resulting iterator is not guaranteed to keep the order of the original iterator.
///
/// # Examples
///
/// To use this trait, take an existing `Iterator` and call `par_bridge` on it. After that, you can
/// use any of the `ParallelIterator` methods:
///
/// ```
/// use rayon::iter::ParallelBridge;
/// use rayon::prelude::ParallelIterator;
/// use std::sync::mpsc::channel;
///
/// let rx = {
///     let (tx, rx) = channel();
///
///     tx.send("one!");
///     tx.send("two!");
///     tx.send("three!");
///
///     rx
/// };
///
/// let mut output: Vec<&'static str> = rx.into_iter().par_bridge().collect();
/// output.sort_unstable();
///
/// assert_eq!(&*output, &["one!", "three!", "two!"]);
/// ```
pub trait ParallelBridge: Sized {
    /// Creates a bridge from this type to a `ParallelIterator`.
    fn par_bridge(self) -> IterBridge<Self>;
}

impl<T: Iterator + Send> ParallelBridge for T
where
    T::Item: Send,
{
    fn par_bridge(self) -> IterBridge<Self> {
        IterBridge { iter: self }
    }
}

/// `IterBridge` is a parallel iterator that wraps a sequential iterator.
///
/// This type is created when using the `par_bridge` method on `ParallelBridge`. See the
/// [`ParallelBridge`] documentation for details.
///
/// [`ParallelBridge`]: trait.ParallelBridge.html
#[derive(Debug, Clone)]
pub struct IterBridge<Iter> {
    iter: Iter,
}

impl<Iter: Iterator + Send> ParallelIterator for IterBridge<Iter>
where
    Iter::Item: Send,
{
    type Item = Iter::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let split_count = AtomicUsize::new(current_num_threads());
        let worker = Worker::new_fifo();
        let stealer = worker.stealer();
        let done = AtomicBool::new(false);
        let iter = Mutex::new((self.iter, worker));

        bridge_unindexed(
            IterParallelProducer {
                split_count: &split_count,
                done: &done,
                iter: &iter,
                items: stealer,
            },
            consumer,
        )
    }
}

struct IterParallelProducer<'a, Iter: Iterator> {
    split_count: &'a AtomicUsize,
    done: &'a AtomicBool,
    iter: &'a Mutex<(Iter, Worker<Iter::Item>)>,
    items: Stealer<Iter::Item>,
}

// manual clone because T doesn't need to be Clone, but the derive assumes it should be
impl<'a, Iter: Iterator + 'a> Clone for IterParallelProducer<'a, Iter> {
    fn clone(&self) -> Self {
        IterParallelProducer {
            split_count: self.split_count,
            done: self.done,
            iter: self.iter,
            items: self.items.clone(),
        }
    }
}

impl<'a, Iter: Iterator + Send + 'a> UnindexedProducer for IterParallelProducer<'a, Iter>
where
    Iter::Item: Send,
{
    type Item = Iter::Item;

    fn split(self) -> (Self, Option<Self>) {
        let mut count = self.split_count.load(Ordering::SeqCst);

        loop {
            // Check if the iterator is exhausted *and* we've consumed every item from it.
            let done = self.done.load(Ordering::SeqCst) && self.items.is_empty();

            match count.checked_sub(1) {
                Some(new_count) if !done => {
                    match self.split_count.compare_exchange_weak(
                        count,
                        new_count,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(_) => return (self.clone(), Some(self)),
                        Err(last_count) => count = last_count,
                    }
                }
                _ => {
                    return (self, None);
                }
            }
        }
    }

    fn fold_with<F>(self, mut folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        loop {
            match self.items.steal() {
                Steal::Success(it) => {
                    folder = folder.consume(it);
                    if folder.full() {
                        return folder;
                    }
                }
                Steal::Empty => {
                    // Don't storm the mutex if we're already done.
                    if self.done.load(Ordering::SeqCst) {
                        // Someone might have pushed more between our `steal()` and `done.load()`
                        if self.items.is_empty() {
                            // The iterator is out of items, no use in continuing
                            return folder;
                        }
                    } else {
                        // our cache is out of items, time to load more from the iterator
                        match self.iter.try_lock() {
                            Ok(mut guard) => {
                                // Check `done` again in case we raced with the previous lock
                                // holder on its way out.
                                if self.done.load(Ordering::SeqCst) {
                                    if self.items.is_empty() {
                                        return folder;
                                    }
                                    continue;
                                }

                                let count = current_num_threads();
                                let count = (count * count) * 2;

                                let (ref mut iter, ref worker) = *guard;

                                // while worker.len() < count {
                                // FIXME the new deque doesn't let us count items.  We can just
                                // push a number of items, but that doesn't consider active
                                // stealers elsewhere.
                                for _ in 0..count {
                                    if let Some(it) = iter.next() {
                                        worker.push(it);
                                    } else {
                                        self.done.store(true, Ordering::SeqCst);
                                        break;
                                    }
                                }
                            }
                            Err(TryLockError::WouldBlock) => {
                                // someone else has the mutex, just sit tight until it's ready
                                yield_now(); //TODO: use a thread-pool-aware yield? (#548)
                            }
                            Err(TryLockError::Poisoned(_)) => {
                                // any panics from other threads will have been caught by the pool,
                                // and will be re-thrown when joined - just exit
                                return folder;
                            }
                        }
                    }
                }
                Steal::Retry => (),
            }
        }
    }
}
