use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::{PopError, PushError};

const LOCKED: usize = 1 << 0;
const PUSHED: usize = 1 << 1;
const CLOSED: usize = 1 << 2;

/// A single-element queue.
pub struct Single<T> {
    state: AtomicUsize,
    slot: UnsafeCell<MaybeUninit<T>>,
}

impl<T> Single<T> {
    /// Creates a new single-element queue.
    pub fn new() -> Single<T> {
        Single {
            state: AtomicUsize::new(0),
            slot: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Attempts to push an item into the queue.
    pub fn push(&self, value: T) -> Result<(), PushError<T>> {
        // Lock and fill the slot.
        let state = self
            .state
            .compare_and_swap(0, LOCKED | PUSHED, Ordering::SeqCst);

        if state == 0 {
            // Write the value and unlock.
            unsafe { self.slot.get().write(MaybeUninit::new(value)) }
            self.state.fetch_and(!LOCKED, Ordering::Release);
            Ok(())
        } else if state & CLOSED != 0 {
            Err(PushError::Closed(value))
        } else {
            Err(PushError::Full(value))
        }
    }

    /// Attempts to pop an item from the queue.
    pub fn pop(&self) -> Result<T, PopError> {
        let mut state = PUSHED;
        loop {
            // Lock and empty the slot.
            let prev =
                self.state
                    .compare_and_swap(state, (state | LOCKED) & !PUSHED, Ordering::SeqCst);

            if prev == state {
                // Read the value and unlock.
                let value = unsafe { self.slot.get().read().assume_init() };
                self.state.fetch_and(!LOCKED, Ordering::Release);
                return Ok(value);
            }

            if prev & PUSHED == 0 {
                if prev & CLOSED == 0 {
                    return Err(PopError::Empty);
                } else {
                    return Err(PopError::Closed);
                }
            }

            if prev & LOCKED == 0 {
                state = prev;
            } else {
                thread::yield_now();
                state = prev & !LOCKED;
            }
        }
    }

    /// Returns the number of items in the queue.
    pub fn len(&self) -> usize {
        if self.state.load(Ordering::SeqCst) & PUSHED == 0 {
            0
        } else {
            1
        }
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the queue is full.
    pub fn is_full(&self) -> bool {
        self.len() == 1
    }

    /// Closes the queue.
    ///
    /// Returns `true` if this call closed the queue.
    pub fn close(&self) -> bool {
        let state = self.state.fetch_or(CLOSED, Ordering::SeqCst);
        state & CLOSED == 0
    }

    /// Returns `true` if the queue is closed.
    pub fn is_closed(&self) -> bool {
        self.state.load(Ordering::SeqCst) & CLOSED != 0
    }
}

impl<T> Drop for Single<T> {
    fn drop(&mut self) {
        // Drop the value in the slot.
        if *self.state.get_mut() & PUSHED != 0 {
            let value = unsafe { self.slot.get().read().assume_init() };
            drop(value);
        }
    }
}
