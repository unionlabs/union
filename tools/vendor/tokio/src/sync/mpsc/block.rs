use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::{AtomicPtr, AtomicUsize};

use std::mem::MaybeUninit;
use std::ops;
use std::ptr::{self, NonNull};
use std::sync::atomic::Ordering::{self, AcqRel, Acquire, Release};

/// A block in a linked list.
///
/// Each block in the list can hold up to `BLOCK_CAP` messages.
pub(crate) struct Block<T> {
    /// The start index of this block.
    ///
    /// Slots in this block have indices in `start_index .. start_index + BLOCK_CAP`.
    start_index: usize,

    /// The next block in the linked list.
    next: AtomicPtr<Block<T>>,

    /// Bitfield tracking slots that are ready to have their values consumed.
    ready_slots: AtomicUsize,

    /// The observed `tail_position` value *after* the block has been passed by
    /// `block_tail`.
    observed_tail_position: UnsafeCell<usize>,

    /// Array containing values pushed into the block. Values are stored in a
    /// continuous array in order to improve cache line behavior when reading.
    /// The values must be manually dropped.
    values: Values<T>,
}

pub(crate) enum Read<T> {
    Value(T),
    Closed,
}

struct Values<T>([UnsafeCell<MaybeUninit<T>>; BLOCK_CAP]);

use super::BLOCK_CAP;

/// Masks an index to get the block identifier.
const BLOCK_MASK: usize = !(BLOCK_CAP - 1);

/// Masks an index to get the value offset in a block.
const SLOT_MASK: usize = BLOCK_CAP - 1;

/// Flag tracking that a block has gone through the sender's release routine.
///
/// When this is set, the receiver may consider freeing the block.
const RELEASED: usize = 1 << BLOCK_CAP;

/// Flag tracking all senders dropped.
///
/// When this flag is set, the send half of the channel has closed.
const TX_CLOSED: usize = RELEASED << 1;

/// Mask covering all bits used to track slot readiness.
const READY_MASK: usize = RELEASED - 1;

/// Returns the index of the first slot in the block referenced by `slot_index`.
#[inline(always)]
pub(crate) fn start_index(slot_index: usize) -> usize {
    BLOCK_MASK & slot_index
}

/// Returns the offset into the block referenced by `slot_index`.
#[inline(always)]
pub(crate) fn offset(slot_index: usize) -> usize {
    SLOT_MASK & slot_index
}

impl<T> Block<T> {
    pub(crate) fn new(start_index: usize) -> Block<T> {
        Block {
            // The absolute index in the channel of the first slot in the block.
            start_index,

            // Pointer to the next block in the linked list.
            next: AtomicPtr::new(ptr::null_mut()),

            ready_slots: AtomicUsize::new(0),

            observed_tail_position: UnsafeCell::new(0),

            // Value storage
            values: unsafe { Values::uninitialized() },
        }
    }

    /// Returns `true` if the block matches the given index.
    pub(crate) fn is_at_index(&self, index: usize) -> bool {
        debug_assert!(offset(index) == 0);
        self.start_index == index
    }

    /// Returns the number of blocks between `self` and the block at the
    /// specified index.
    ///
    /// `start_index` must represent a block *after* `self`.
    pub(crate) fn distance(&self, other_index: usize) -> usize {
        debug_assert!(offset(other_index) == 0);
        other_index.wrapping_sub(self.start_index) / BLOCK_CAP
    }

    /// Reads the value at the given offset.
    ///
    /// Returns `None` if the slot is empty.
    ///
    /// # Safety
    ///
    /// To maintain safety, the caller must ensure:
    ///
    /// * No concurrent access to the slot.
    pub(crate) unsafe fn read(&self, slot_index: usize) -> Option<Read<T>> {
        let offset = offset(slot_index);

        let ready_bits = self.ready_slots.load(Acquire);

        if !is_ready(ready_bits, offset) {
            if is_tx_closed(ready_bits) {
                return Some(Read::Closed);
            }

            return None;
        }

        // Get the value
        let value = self.values[offset].with(|ptr| ptr::read(ptr));

        Some(Read::Value(value.assume_init()))
    }

    /// Writes a value to the block at the given offset.
    ///
    /// # Safety
    ///
    /// To maintain safety, the caller must ensure:
    ///
    /// * The slot is empty.
    /// * No concurrent access to the slot.
    pub(crate) unsafe fn write(&self, slot_index: usize, value: T) {
        // Get the offset into the block
        let slot_offset = offset(slot_index);

        self.values[slot_offset].with_mut(|ptr| {
            ptr::write(ptr, MaybeUninit::new(value));
        });

        // Release the value. After this point, the slot ref may no longer
        // be used. It is possible for the receiver to free the memory at
        // any point.
        self.set_ready(slot_offset);
    }

    /// Signal to the receiver that the sender half of the list is closed.
    pub(crate) unsafe fn tx_close(&self) {
        self.ready_slots.fetch_or(TX_CLOSED, Release);
    }

    /// Resets the block to a blank state. This enables reusing blocks in the
    /// channel.
    ///
    /// # Safety
    ///
    /// To maintain safety, the caller must ensure:
    ///
    /// * All slots are empty.
    /// * The caller holds a unique pointer to the block.
    pub(crate) unsafe fn reclaim(&mut self) {
        self.start_index = 0;
        self.next = AtomicPtr::new(ptr::null_mut());
        self.ready_slots = AtomicUsize::new(0);
    }

    /// Releases the block to the rx half for freeing.
    ///
    /// This function is called by the tx half once it can be guaranteed that no
    /// more senders will attempt to access the block.
    ///
    /// # Safety
    ///
    /// To maintain safety, the caller must ensure:
    ///
    /// * The block will no longer be accessed by any sender.
    pub(crate) unsafe fn tx_release(&self, tail_position: usize) {
        // Track the observed tail_position. Any sender targeting a greater
        // tail_position is guaranteed to not access this block.
        self.observed_tail_position
            .with_mut(|ptr| *ptr = tail_position);

        // Set the released bit, signalling to the receiver that it is safe to
        // free the block's memory as soon as all slots **prior** to
        // `observed_tail_position` have been filled.
        self.ready_slots.fetch_or(RELEASED, Release);
    }

    /// Mark a slot as ready
    fn set_ready(&self, slot: usize) {
        let mask = 1 << slot;
        self.ready_slots.fetch_or(mask, Release);
    }

    /// Returns `true` when all slots have their `ready` bits set.
    ///
    /// This indicates that the block is in its final state and will no longer
    /// be mutated.
    ///
    /// # Implementation
    ///
    /// The implementation walks each slot checking the `ready` flag. It might
    /// be that it would make more sense to coalesce ready flags as bits in a
    /// single atomic cell. However, this could have negative impact on cache
    /// behavior as there would be many more mutations to a single slot.
    pub(crate) fn is_final(&self) -> bool {
        self.ready_slots.load(Acquire) & READY_MASK == READY_MASK
    }

    /// Returns the `observed_tail_position` value, if set
    pub(crate) fn observed_tail_position(&self) -> Option<usize> {
        if 0 == RELEASED & self.ready_slots.load(Acquire) {
            None
        } else {
            Some(self.observed_tail_position.with(|ptr| unsafe { *ptr }))
        }
    }

    /// Loads the next block
    pub(crate) fn load_next(&self, ordering: Ordering) -> Option<NonNull<Block<T>>> {
        let ret = NonNull::new(self.next.load(ordering));

        debug_assert!(unsafe {
            ret.map(|block| block.as_ref().start_index == self.start_index.wrapping_add(BLOCK_CAP))
                .unwrap_or(true)
        });

        ret
    }

    /// Pushes `block` as the next block in the link.
    ///
    /// Returns Ok if successful, otherwise, a pointer to the next block in
    /// the list is returned.
    ///
    /// This requires that the next pointer is null.
    ///
    /// # Ordering
    ///
    /// This performs a compare-and-swap on `next` using AcqRel ordering.
    ///
    /// # Safety
    ///
    /// To maintain safety, the caller must ensure:
    ///
    /// * `block` is not freed until it has been removed from the list.
    pub(crate) unsafe fn try_push(
        &self,
        block: &mut NonNull<Block<T>>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<(), NonNull<Block<T>>> {
        block.as_mut().start_index = self.start_index.wrapping_add(BLOCK_CAP);

        let next_ptr = self
            .next
            .compare_exchange(ptr::null_mut(), block.as_ptr(), success, failure)
            .unwrap_or_else(|x| x);

        match NonNull::new(next_ptr) {
            Some(next_ptr) => Err(next_ptr),
            None => Ok(()),
        }
    }

    /// Grows the `Block` linked list by allocating and appending a new block.
    ///
    /// The next block in the linked list is returned. This may or may not be
    /// the one allocated by the function call.
    ///
    /// # Implementation
    ///
    /// It is assumed that `self.next` is null. A new block is allocated with
    /// `start_index` set to be the next block. A compare-and-swap is performed
    /// with AcqRel memory ordering. If the compare-and-swap is successful, the
    /// newly allocated block is released to other threads walking the block
    /// linked list. If the compare-and-swap fails, the current thread acquires
    /// the next block in the linked list, allowing the current thread to access
    /// the slots.
    pub(crate) fn grow(&self) -> NonNull<Block<T>> {
        // Create the new block. It is assumed that the block will become the
        // next one after `&self`. If this turns out to not be the case,
        // `start_index` is updated accordingly.
        let new_block = Box::new(Block::new(self.start_index + BLOCK_CAP));

        let mut new_block = unsafe { NonNull::new_unchecked(Box::into_raw(new_block)) };

        // Attempt to store the block. The first compare-and-swap attempt is
        // "unrolled" due to minor differences in logic
        //
        // `AcqRel` is used as the ordering **only** when attempting the
        // compare-and-swap on self.next.
        //
        // If the compare-and-swap fails, then the actual value of the cell is
        // returned from this function and accessed by the caller. Given this,
        // the memory must be acquired.
        //
        // `Release` ensures that the newly allocated block is available to
        // other threads acquiring the next pointer.
        let next = NonNull::new(
            self.next
                .compare_exchange(ptr::null_mut(), new_block.as_ptr(), AcqRel, Acquire)
                .unwrap_or_else(|x| x),
        );

        let next = match next {
            Some(next) => next,
            None => {
                // The compare-and-swap succeeded and the newly allocated block
                // is successfully pushed.
                return new_block;
            }
        };

        // There already is a next block in the linked list. The newly allocated
        // block could be dropped and the discovered next block returned;
        // however, that would be wasteful. Instead, the linked list is walked
        // by repeatedly attempting to compare-and-swap the pointer into the
        // `next` register until the compare-and-swap succeed.
        //
        // Care is taken to update new_block's start_index field as appropriate.

        let mut curr = next;

        // TODO: Should this iteration be capped?
        loop {
            let actual = unsafe { curr.as_ref().try_push(&mut new_block, AcqRel, Acquire) };

            curr = match actual {
                Ok(_) => {
                    return next;
                }
                Err(curr) => curr,
            };

            crate::loom::thread::yield_now();
        }
    }
}

/// Returns `true` if the specified slot has a value ready to be consumed.
fn is_ready(bits: usize, slot: usize) -> bool {
    let mask = 1 << slot;
    mask == mask & bits
}

/// Returns `true` if the closed flag has been set.
fn is_tx_closed(bits: usize) -> bool {
    TX_CLOSED == bits & TX_CLOSED
}

impl<T> Values<T> {
    unsafe fn uninitialized() -> Values<T> {
        let mut vals = MaybeUninit::uninit();

        // When fuzzing, `UnsafeCell` needs to be initialized.
        if_loom! {
            let p = vals.as_mut_ptr() as *mut UnsafeCell<MaybeUninit<T>>;
            for i in 0..BLOCK_CAP {
                p.add(i)
                    .write(UnsafeCell::new(MaybeUninit::uninit()));
            }
        }

        Values(vals.assume_init())
    }
}

impl<T> ops::Index<usize> for Values<T> {
    type Output = UnsafeCell<MaybeUninit<T>>;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
