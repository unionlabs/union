//! Here's an overview of what the stack looks like on Windows:
//!
//! ```text
//! +------------------+  <- TEB.StackBase
//! |                  |
//! ~ Used stack space ~  [MEM_COMMIT + PAGE_READWRITE]
//! |                  |
//! +------------------+  <- TEB.StackLimit
//! | Soft guard page  |  [MEM_COMMIT + PAGE_READWRITE + PAGE_GUARD]
//! +------------------+
//! |                  |
//! ~ Stack guarantee  ~  [MEM_COMMIT + PAGE_READWRITE + PAGE_GUARD]
//! |                  |
//! +------------------+
//! |                  |
//! ~ Reserved stack   ~  [MEM_RESERVE]
//! |                  |
//! +------------------+
//! | Hard guard page  |  [MEM_RESERVE]
//! +------------------+  <- TEB.DeallocationStack
//! ```
//!
//! To understand what is going on here, we need to cover several concepts
//! first.
//!
//! ## Reserving and comitting memory
//!
//! Linux allows `mmap`ing arbitrary amounts of virtual memory regardless of how
//! much physical memory and swap space is available. This is know as
//! "overcommit". Actual allocation is lazily deferred until the pages are
//! accessed, at which point it is possible there isn't enough memory available
//! to allocate a page of physical memory. If this happens then the OS will
//! kill an arbitrary processing through the OOM killer to reclaim memory.
//!
//! Windows on the other hand will explicitly keep track of how much memory is
//! available and will refuse to "commit" more memory than the total of physical
//! memory and the page file. You can still reserve arbitrary amounts of virtual
//! memory, but reserved memory is not usable: you need to actually commit a
//! page to it to be able to read/write data at that memory address. This means
//! that out-of-memory errors only occur when trying to commit memory and there
//! is no need to go kill an arbitrary process when the system is low on memory.
//!
//! ## Stack growth
//!
//! Thread stack are usually pretty large (1MiB by default) so it is desirable
//! to not commit the entire stack for each thread. Instead, only a small region
//! is initially committed while most of the stack is left as reserved memory.
//!
//! A soft guard page (committed with `PAGE_GUARD`) is placed just below the
//! stack limit. On 64-bit targets, 2 pages are used instead of 1. Attempting to
//! access it will trigger a trap, but this is handled by the kernel
//! transparently to grow the stack: the guard page is converted to a normal
//! `PAGE_READWRITE` page and a new guard page is committed below the previous
//! one. Effectively, this "moves" the guard page down by one page every time
//! the stack grows.
//!
//! ## Stack overflows
//!
//! If the kernel can't commit a new guard page or there is only one page of
//! reserved memory left on the stack (the "hard" guard page) then the kernel
//! throws a stack overflow exception. The exception handler runs on the same
//! stack, which is a problem.
//!
//! To ensure that the stack overflow handler actually has some stack space to
//! work with, a "stack guarantee" can be reserved ahead of time. This works by
//! committing a few extra pages (the amount is controlled by the
//! `SetThreadStackGuarantee` API) just below the guard page which are shifted
//! with it as the stack grows.
//!
//! Now, the kernel triggers a stack overflow exception when there is no more
//! reserved stack left to shift the guard page and guaranteed pages down (or
//! there is no more memory to commit a new guard page). At this point, the
//! guard page and guaranteed pages are all converted from guard pages to normal
//! pages, leaving only the hard guard page to catch stack overflows from the
//! exception handler.
//!
//! As a result of this, the guard pages must be manually restored after a
//! stack overflow exception has occurred so that future stack overflows are
//! still caught.
//!
//! Honestly this whole thing could've been much simpler if Windows used
//! an alternate signal stack, but that's what we have to work with.

extern crate std;

use std::io::{Error, Result};
use std::ptr;

use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_GUARD, PAGE_READWRITE,
};
use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows_sys::Win32::System::Threading::SetThreadStackGuarantee;

use super::{Stack, StackPointer, StackTebFields, MIN_STACK_SIZE};

fn page_size() -> usize {
    unsafe {
        let mut sysinfo: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut sysinfo);
        assert!(sysinfo.dwPageSize.is_power_of_two());
        sysinfo.dwPageSize as usize
    }
}

fn page_round_up(val: usize, page_size: usize) -> usize {
    (val + page_size - 1) & !(page_size - 1)
}

fn get_thread_stack_guarantee(page_size: usize) -> usize {
    // Passing a value of 0 will just query the existing value.
    let mut stack_guarantee = 0;
    unsafe {
        SetThreadStackGuarantee(&mut stack_guarantee);
    }

    // At a bare minimum we need to reserve 1 page for the stack overflow
    // handler. Also round the guarantee up to a page boundary.
    page_round_up((stack_guarantee as usize).max(page_size), page_size)
}

fn guard_page_size(page_size: usize) -> usize {
    if cfg!(target_pointer_width = "64") {
        2 * page_size
    } else {
        page_size
    }
}

/// Default stack implementation which uses `VirtualAlloc`.
pub struct DefaultStack {
    base: StackPointer,
    limit: usize,
    deallocation_stack: StackPointer,
    stack_guarantee: usize,
}

impl DefaultStack {
    /// Creates a new stack which has at least the given capacity.
    pub fn new(size: usize) -> Result<Self> {
        // Apply minimum stack size.
        let size = size.max(MIN_STACK_SIZE);

        // Calculate how many extra pages we need to add for the various guard
        // pages:
        // - 1 or 2 guard pages to catch the fault (which may be 4095/8191 bytes
        //   into the guard page).
        // - N pages for the thread stack guarantee.
        // - 1 hard guard page at the end of the stack.
        let page_size = page_size();
        let guard_size = guard_page_size(page_size);
        let stack_guarantee = get_thread_stack_guarantee(page_size);
        let extra_pages = guard_size + stack_guarantee + page_size;

        // Add the extra pages to the requested size and round the size up to
        // a page boundary.
        let alloc_len = size
            .checked_add(extra_pages + page_size - 1)
            .expect("integer overflow while calculating stack size")
            & !(page_size - 1);

        unsafe {
            // Reserve virtual memory for the stack.
            let alloc_base = VirtualAlloc(ptr::null(), alloc_len, MEM_RESERVE, PAGE_READWRITE);
            if alloc_base.is_null() {
                return Err(Error::last_os_error());
            }

            // Create the result here. If the later VirtualAlloc calls fail then
            // this will be dropped and the memory will be unmapped.
            let alloc_top = alloc_base as usize + alloc_len;
            let limit = alloc_top - page_round_up(MIN_STACK_SIZE, page_size);
            let out = Self {
                base: StackPointer::new(alloc_top).unwrap(),
                limit,
                deallocation_stack: StackPointer::new(alloc_base as usize).unwrap(),
                stack_guarantee,
            };

            // Commit the first MIN_STACK_SIZE pages of the stack.
            if VirtualAlloc(
                limit as *mut _,
                alloc_top - limit,
                MEM_COMMIT,
                PAGE_READWRITE,
            )
            .is_null()
            {
                return Err(Error::last_os_error());
            }

            // Commit the guard pages.
            let stack_guard_size = guard_size + stack_guarantee;
            if VirtualAlloc(
                (limit - stack_guard_size) as *mut _,
                stack_guard_size,
                MEM_COMMIT,
                PAGE_READWRITE | PAGE_GUARD,
            )
            .is_null()
            {
                return Err(Error::last_os_error());
            }

            Ok(out)
        }
    }
}

impl Default for DefaultStack {
    fn default() -> Self {
        Self::new(1024 * 1024).expect("failed to allocate stack")
    }
}

impl Drop for DefaultStack {
    fn drop(&mut self) {
        unsafe {
            let alloc_base = self.deallocation_stack.get() as *mut _;
            let ret = VirtualFree(alloc_base, 0, MEM_RELEASE);
            debug_assert!(ret != 0);
        }
    }
}

unsafe impl Stack for DefaultStack {
    #[inline]
    fn base(&self) -> StackPointer {
        self.base
    }

    #[inline]
    fn limit(&self) -> StackPointer {
        self.deallocation_stack
    }

    #[inline]
    fn teb_fields(&self) -> StackTebFields {
        StackTebFields {
            StackBase: self.base.get(),
            StackLimit: self.limit,
            DeallocationStack: self.deallocation_stack.get(),
            GuaranteedStackBytes: self.stack_guarantee,
        }
    }

    #[inline]
    fn update_teb_fields(&mut self, stack_limit: usize, guaranteed_stack_bytes: usize) {
        self.limit = stack_limit;
        self.stack_guarantee = guaranteed_stack_bytes;
    }
}
