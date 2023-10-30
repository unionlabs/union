// only used on Linux right now, so allow dead code elsewhere
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use super::Mmap;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

/// A simple arena allocator for byte buffers.
pub struct Stash {
    buffers: UnsafeCell<Vec<Vec<u8>>>,
    mmap_aux: UnsafeCell<Option<Mmap>>,
}

impl Stash {
    pub fn new() -> Stash {
        Stash {
            buffers: UnsafeCell::new(Vec::new()),
            mmap_aux: UnsafeCell::new(None),
        }
    }

    /// Allocates a buffer of the specified size and returns a mutable reference
    /// to it.
    pub fn allocate(&self, size: usize) -> &mut [u8] {
        // SAFETY: this is the only function that ever constructs a mutable
        // reference to `self.buffers`.
        let buffers = unsafe { &mut *self.buffers.get() };
        let i = buffers.len();
        buffers.push(vec![0; size]);
        // SAFETY: we never remove elements from `self.buffers`, so a reference
        // to the data inside any buffer will live as long as `self` does.
        &mut buffers[i]
    }

    /// Stores a `Mmap` for the lifetime of this `Stash`, returning a pointer
    /// which is scoped to just this lifetime.
    pub fn set_mmap_aux(&self, map: Mmap) -> &[u8] {
        // SAFETY: this is the only location for a mutable pointer to
        // `mmap_aux`, and this structure isn't threadsafe to shared across
        // threads either. This also is careful to store at most one `mmap_aux`
        // since overwriting a previous one would invalidate the previous
        // pointer. Given that though we can safely return a pointer to our
        // interior-owned contents.
        unsafe {
            let mmap_aux = &mut *self.mmap_aux.get();
            assert!(mmap_aux.is_none());
            *mmap_aux = Some(map);
            mmap_aux.as_ref().unwrap()
        }
    }
}
