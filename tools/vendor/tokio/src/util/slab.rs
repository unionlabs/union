#![cfg_attr(not(feature = "rt"), allow(dead_code))]

use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::{AtomicBool, AtomicUsize};
use crate::loom::sync::{Arc, Mutex};
use crate::util::bit;
use std::fmt;
use std::mem;
use std::ops;
use std::ptr;
use std::sync::atomic::Ordering::Relaxed;

/// Amortized allocation for homogeneous data types.
///
/// The slab pre-allocates chunks of memory to store values. It uses a similar
/// growing strategy as `Vec`. When new capacity is needed, the slab grows by
/// 2x.
///
/// # Pages
///
/// Unlike `Vec`, growing does not require moving existing elements. Instead of
/// being a continuous chunk of memory for all elements, `Slab` is an array of
/// arrays. The top-level array is an array of pages. Each page is 2x bigger
/// than the previous one. When the slab grows, a new page is allocated.
///
/// Pages are lazily initialized.
///
/// # Allocating
///
/// When allocating an object, first previously used slots are reused. If no
/// previously used slot is available, a new slot is initialized in an existing
/// page. If all pages are full, then a new page is allocated.
///
/// When an allocated object is released, it is pushed into it's page's free
/// list. Allocating scans all pages for a free slot.
///
/// # Indexing
///
/// The slab is able to index values using an address. Even when the indexed
/// object has been released, it is still safe to index. This is a key ability
/// for using the slab with the I/O driver. Addresses are registered with the
/// OS's selector and I/O resources can be released without synchronizing with
/// the OS.
///
/// # Compaction
///
/// `Slab::compact` will release pages that have been allocated but are no
/// longer used. This is done by scanning the pages and finding pages with no
/// allocated objects. These pages are then freed.
///
/// # Synchronization
///
/// The `Slab` structure is able to provide (mostly) unsynchronized reads to
/// values stored in the slab. Insertions and removals are synchronized. Reading
/// objects via `Ref` is fully unsynchronized. Indexing objects uses amortized
/// synchronization.
///
pub(crate) struct Slab<T> {
    /// Array of pages. Each page is synchronized.
    pages: [Arc<Page<T>>; NUM_PAGES],

    /// Caches the array pointer & number of initialized slots.
    cached: [CachedPage<T>; NUM_PAGES],
}

/// Allocate values in the associated slab.
pub(crate) struct Allocator<T> {
    /// Pages in the slab. The first page has a capacity of 16 elements. Each
    /// following page has double the capacity of the previous page.
    ///
    /// Each returned `Ref` holds a reference count to this `Arc`.
    pages: [Arc<Page<T>>; NUM_PAGES],
}

/// References a slot in the slab. Indexing a slot using an `Address` is memory
/// safe even if the slot has been released or the page has been deallocated.
/// However, it is not guaranteed that the slot has not been reused and is now
/// represents a different value.
///
/// The I/O driver uses a counter to track the slot's generation. Once accessing
/// the slot, the generations are compared. If they match, the value matches the
/// address.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Address(usize);

/// An entry in the slab.
pub(crate) trait Entry: Default {
    /// Resets the entry's value and track the generation.
    fn reset(&self);
}

/// A reference to a value stored in the slab.
pub(crate) struct Ref<T> {
    value: *const Value<T>,
}

/// Maximum number of pages a slab can contain.
const NUM_PAGES: usize = 19;

/// Minimum number of slots a page can contain.
const PAGE_INITIAL_SIZE: usize = 32;
const PAGE_INDEX_SHIFT: u32 = PAGE_INITIAL_SIZE.trailing_zeros() + 1;

/// A page in the slab.
struct Page<T> {
    /// Slots.
    slots: Mutex<Slots<T>>,

    // Number of slots currently being used. This is not guaranteed to be up to
    // date and should only be used as a hint.
    used: AtomicUsize,

    // Set to `true` when the page has been allocated.
    allocated: AtomicBool,

    // The number of slots the page can hold.
    len: usize,

    // Length of all previous pages combined.
    prev_len: usize,
}

struct CachedPage<T> {
    /// Pointer to the page's slots.
    slots: *const Slot<T>,

    /// Number of initialized slots.
    init: usize,
}

/// Page state.
struct Slots<T> {
    /// Slots.
    slots: Vec<Slot<T>>,

    head: usize,

    /// Number of slots currently in use.
    used: usize,
}

unsafe impl<T: Sync> Sync for Page<T> {}
unsafe impl<T: Sync> Send for Page<T> {}
unsafe impl<T: Sync> Sync for CachedPage<T> {}
unsafe impl<T: Sync> Send for CachedPage<T> {}
unsafe impl<T: Sync> Sync for Ref<T> {}
unsafe impl<T: Sync> Send for Ref<T> {}

/// A slot in the slab. Contains slot-specific metadata.
///
/// `#[repr(C)]` guarantees that the struct starts w/ `value`. We use pointer
/// math to map a value pointer to an index in the page.
#[repr(C)]
struct Slot<T> {
    /// Pointed to by `Ref`.
    value: UnsafeCell<Value<T>>,

    /// Next entry in the free list.
    next: u32,

    /// Makes miri happy by making mutable references not take exclusive access.
    ///
    /// Could probably also be fixed by replacing `slots` with a raw-pointer
    /// based equivalent.
    _pin: std::marker::PhantomPinned,
}

/// Value paired with a reference to the page.
struct Value<T> {
    /// Value stored in the value.
    value: T,

    /// Pointer to the page containing the slot.
    ///
    /// A raw pointer is used as this creates a ref cycle.
    page: *const Page<T>,
}

impl<T> Slab<T> {
    /// Create a new, empty, slab.
    pub(crate) fn new() -> Slab<T> {
        // Initializing arrays is a bit annoying. Instead of manually writing
        // out an array and every single entry, `Default::default()` is used to
        // initialize the array, then the array is iterated and each value is
        // initialized.
        let mut slab = Slab {
            pages: Default::default(),
            cached: Default::default(),
        };

        let mut len = PAGE_INITIAL_SIZE;
        let mut prev_len: usize = 0;

        for page in &mut slab.pages {
            let page = Arc::get_mut(page).unwrap();
            page.len = len;
            page.prev_len = prev_len;
            len *= 2;
            prev_len += page.len;

            // Ensure we don't exceed the max address space.
            debug_assert!(
                page.len - 1 + page.prev_len < (1 << 24),
                "max = {:b}",
                page.len - 1 + page.prev_len
            );
        }

        slab
    }

    /// Returns a new `Allocator`.
    ///
    /// The `Allocator` supports concurrent allocation of objects.
    pub(crate) fn allocator(&self) -> Allocator<T> {
        Allocator {
            pages: self.pages.clone(),
        }
    }

    /// Returns a reference to the value stored at the given address.
    ///
    /// `&mut self` is used as the call may update internal cached state.
    pub(crate) fn get(&mut self, addr: Address) -> Option<&T> {
        let page_idx = addr.page();
        let slot_idx = self.pages[page_idx].slot(addr);

        // If the address references a slot that was last seen as uninitialized,
        // the `CachedPage` is updated. This requires acquiring the page lock
        // and updating the slot pointer and initialized offset.
        if self.cached[page_idx].init <= slot_idx {
            self.cached[page_idx].refresh(&self.pages[page_idx]);
        }

        // If the address **still** references an uninitialized slot, then the
        // address is invalid and `None` is returned.
        if self.cached[page_idx].init <= slot_idx {
            return None;
        }

        // Get a reference to the value. The lifetime of the returned reference
        // is bound to `&self`. The only way to invalidate the underlying memory
        // is to call `compact()`. The lifetimes prevent calling `compact()`
        // while references to values are outstanding.
        //
        // The referenced data is never mutated. Only `&self` references are
        // used and the data is `Sync`.
        Some(self.cached[page_idx].get(slot_idx))
    }

    /// Calls the given function with a reference to each slot in the slab. The
    /// slot may not be in-use.
    ///
    /// This is used by the I/O driver during the shutdown process to notify
    /// each pending task.
    pub(crate) fn for_each(&mut self, mut f: impl FnMut(&T)) {
        for page_idx in 0..self.pages.len() {
            // It is required to avoid holding the lock when calling the
            // provided function. The function may attempt to acquire the lock
            // itself. If we hold the lock here while calling `f`, a deadlock
            // situation is possible.
            //
            // Instead of iterating the slots directly in `page`, which would
            // require holding the lock, the cache is updated and the slots are
            // iterated from the cache.
            self.cached[page_idx].refresh(&self.pages[page_idx]);

            for slot_idx in 0..self.cached[page_idx].init {
                f(self.cached[page_idx].get(slot_idx));
            }
        }
    }

    // Release memory back to the allocator.
    //
    // If pages are empty, the underlying memory is released back to the
    // allocator.
    pub(crate) fn compact(&mut self) {
        // Iterate each page except the very first one. The very first page is
        // never freed.
        for (idx, page) in self.pages.iter().enumerate().skip(1) {
            if page.used.load(Relaxed) != 0 || !page.allocated.load(Relaxed) {
                // If the page has slots in use or the memory has not been
                // allocated then it cannot be compacted.
                continue;
            }

            let mut slots = match page.slots.try_lock() {
                Some(slots) => slots,
                // If the lock cannot be acquired due to being held by another
                // thread, don't try to compact the page.
                _ => continue,
            };

            if slots.used > 0 || slots.slots.capacity() == 0 {
                // The page is in use or it has not yet been allocated. Either
                // way, there is no more work to do.
                continue;
            }

            page.allocated.store(false, Relaxed);

            // Remove the slots vector from the page. This is done so that the
            // freeing process is done outside of the lock's critical section.
            let vec = mem::take(&mut slots.slots);
            slots.head = 0;

            // Drop the lock so we can drop the vector outside the lock below.
            drop(slots);

            debug_assert!(
                self.cached[idx].slots.is_null() || self.cached[idx].slots == vec.as_ptr(),
                "cached = {:?}; actual = {:?}",
                self.cached[idx].slots,
                vec.as_ptr(),
            );

            // Clear cache
            self.cached[idx].slots = ptr::null();
            self.cached[idx].init = 0;

            drop(vec);
        }
    }
}

impl<T> fmt::Debug for Slab<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(fmt, "Slab", &self.pages[..])
    }
}

impl<T: Entry> Allocator<T> {
    /// Allocate a new entry and return a handle to the entry.
    ///
    /// Scans pages from smallest to biggest, stopping when a slot is found.
    /// Pages are allocated if necessary.
    ///
    /// Returns `None` if the slab is full.
    pub(crate) fn allocate(&self) -> Option<(Address, Ref<T>)> {
        // Find the first available slot.
        for page in &self.pages[..] {
            if let Some((addr, val)) = Page::allocate(page) {
                return Some((addr, val));
            }
        }

        None
    }
}

impl<T> fmt::Debug for Allocator<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug(fmt, "slab::Allocator", &self.pages[..])
    }
}

impl<T> ops::Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: `&mut` is never handed out to the underlying value. The page
        // is not freed until all `Ref` values are dropped.
        unsafe { &(*self.value).value }
    }
}

impl<T> Drop for Ref<T> {
    fn drop(&mut self) {
        // Safety: `&mut` is never handed out to the underlying value. The page
        // is not freed until all `Ref` values are dropped.
        let _ = unsafe { (*self.value).release() };
    }
}

impl<T: fmt::Debug> fmt::Debug for Ref<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(fmt)
    }
}

impl<T: Entry> Page<T> {
    // Allocates an object, returns the ref and address.
    //
    // `self: &Arc<Page<T>>` is avoided here as this would not work with the
    // loom `Arc`.
    fn allocate(me: &Arc<Page<T>>) -> Option<(Address, Ref<T>)> {
        // Before acquiring the lock, use the `used` hint.
        if me.used.load(Relaxed) == me.len {
            return None;
        }

        // Allocating objects requires synchronization
        let mut locked = me.slots.lock();

        if locked.head < locked.slots.len() {
            // Re-use an already initialized slot.
            //
            // Help out the borrow checker
            let locked = &mut *locked;

            // Get the index of the slot at the head of the free stack. This is
            // the slot that will be reused.
            let idx = locked.head;
            let slot = &locked.slots[idx];

            // Update the free stack head to point to the next slot.
            locked.head = slot.next as usize;

            // Increment the number of used slots
            locked.used += 1;
            me.used.store(locked.used, Relaxed);

            // Reset the slot
            slot.value.with(|ptr| unsafe { (*ptr).value.reset() });

            // Return a reference to the slot
            Some((me.addr(idx), locked.gen_ref(idx, me)))
        } else if me.len == locked.slots.len() {
            // The page is full
            None
        } else {
            // No initialized slots are available, but the page has more
            // capacity. Initialize a new slot.
            let idx = locked.slots.len();

            if idx == 0 {
                // The page has not yet been allocated. Allocate the storage for
                // all page slots.
                locked.slots.reserve_exact(me.len);
            }

            // Initialize a new slot
            locked.slots.push(Slot {
                value: UnsafeCell::new(Value {
                    value: Default::default(),
                    page: Arc::as_ptr(me),
                }),
                next: 0,
                _pin: std::marker::PhantomPinned,
            });

            // Increment the head to indicate the free stack is empty
            locked.head += 1;

            // Increment the number of used slots
            locked.used += 1;
            me.used.store(locked.used, Relaxed);
            me.allocated.store(true, Relaxed);

            debug_assert_eq!(locked.slots.len(), locked.head);

            Some((me.addr(idx), locked.gen_ref(idx, me)))
        }
    }
}

impl<T> Page<T> {
    /// Returns the slot index within the current page referenced by the given
    /// address.
    fn slot(&self, addr: Address) -> usize {
        addr.0 - self.prev_len
    }

    /// Returns the address for the given slot.
    fn addr(&self, slot: usize) -> Address {
        Address(slot + self.prev_len)
    }
}

impl<T> Default for Page<T> {
    fn default() -> Page<T> {
        Page {
            used: AtomicUsize::new(0),
            allocated: AtomicBool::new(false),
            slots: Mutex::new(Slots {
                slots: Vec::new(),
                head: 0,
                used: 0,
            }),
            len: 0,
            prev_len: 0,
        }
    }
}

impl<T> Page<T> {
    /// Release a slot into the page's free list.
    fn release(&self, value: *const Value<T>) {
        let mut locked = self.slots.lock();

        let idx = locked.index_for(value);
        locked.slots[idx].next = locked.head as u32;
        locked.head = idx;
        locked.used -= 1;

        self.used.store(locked.used, Relaxed);
    }
}

impl<T> CachedPage<T> {
    /// Refreshes the cache.
    fn refresh(&mut self, page: &Page<T>) {
        let slots = page.slots.lock();

        if !slots.slots.is_empty() {
            self.slots = slots.slots.as_ptr();
            self.init = slots.slots.len();
        }
    }

    /// Gets a value by index.
    fn get(&self, idx: usize) -> &T {
        assert!(idx < self.init);

        // Safety: Pages are allocated concurrently, but are only ever
        // **deallocated** by `Slab`. `Slab` will always have a more
        // conservative view on the state of the slot array. Once `CachedPage`
        // sees a slot pointer and initialized offset, it will remain valid
        // until `compact()` is called. The `compact()` function also updates
        // `CachedPage`.
        unsafe {
            let slot = self.slots.add(idx);
            let value = slot as *const Value<T>;

            &(*value).value
        }
    }
}

impl<T> Default for CachedPage<T> {
    fn default() -> CachedPage<T> {
        CachedPage {
            slots: ptr::null(),
            init: 0,
        }
    }
}

impl<T> Slots<T> {
    /// Maps a slot pointer to an offset within the current page.
    ///
    /// The pointer math removes the `usize` index from the `Ref` struct,
    /// shrinking the struct to a single pointer size. The contents of the
    /// function is safe, the resulting `usize` is bounds checked before being
    /// used.
    ///
    /// # Panics
    ///
    /// panics if the provided slot pointer is not contained by the page.
    fn index_for(&self, slot: *const Value<T>) -> usize {
        use std::mem;

        assert_ne!(self.slots.capacity(), 0, "page is unallocated");

        let base = self.slots.as_ptr() as usize;
        let slot = slot as usize;
        let width = mem::size_of::<Slot<T>>();

        assert!(slot >= base, "unexpected pointer");

        let idx = (slot - base) / width;
        assert!(idx < self.slots.len() as usize);

        idx
    }

    /// Generates a `Ref` for the slot at the given index. This involves bumping the page's ref count.
    fn gen_ref(&self, idx: usize, page: &Arc<Page<T>>) -> Ref<T> {
        assert!(idx < self.slots.len());
        mem::forget(page.clone());

        let vec_ptr = self.slots.as_ptr();
        let slot: *const Slot<T> = unsafe { vec_ptr.add(idx) };
        let value: *const Value<T> = slot as *const Value<T>;

        Ref { value }
    }
}

impl<T> Value<T> {
    /// Releases the slot, returning the `Arc<Page<T>>` logically owned by the ref.
    fn release(&self) -> Arc<Page<T>> {
        // Safety: called by `Ref`, which owns an `Arc<Page<T>>` instance.
        let page = unsafe { Arc::from_raw(self.page) };
        page.release(self as *const _);
        page
    }
}

impl Address {
    fn page(self) -> usize {
        // Since every page is twice as large as the previous page, and all page
        // sizes are powers of two, we can determine the page index that
        // contains a given address by shifting the address down by the smallest
        // page size and looking at how many twos places necessary to represent
        // that number, telling us what power of two page size it fits inside
        // of. We can determine the number of twos places by counting the number
        // of leading zeros (unused twos places) in the number's binary
        // representation, and subtracting that count from the total number of
        // bits in a word.
        let slot_shifted = (self.0 + PAGE_INITIAL_SIZE) >> PAGE_INDEX_SHIFT;
        (bit::pointer_width() - slot_shifted.leading_zeros()) as usize
    }

    pub(crate) const fn as_usize(self) -> usize {
        self.0
    }

    pub(crate) fn from_usize(src: usize) -> Address {
        Address(src)
    }
}

fn debug<T>(fmt: &mut fmt::Formatter<'_>, name: &str, pages: &[Arc<Page<T>>]) -> fmt::Result {
    let mut capacity = 0;
    let mut len = 0;

    for page in pages {
        if page.allocated.load(Relaxed) {
            capacity += page.len;
            len += page.used.load(Relaxed);
        }
    }

    fmt.debug_struct(name)
        .field("len", &len)
        .field("capacity", &capacity)
        .finish()
}

#[cfg(all(test, not(loom)))]
mod test {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::SeqCst;

    struct Foo {
        cnt: AtomicUsize,
        id: AtomicUsize,
    }

    impl Default for Foo {
        fn default() -> Foo {
            Foo {
                cnt: AtomicUsize::new(0),
                id: AtomicUsize::new(0),
            }
        }
    }

    impl Entry for Foo {
        fn reset(&self) {
            self.cnt.fetch_add(1, SeqCst);
        }
    }

    #[test]
    fn insert_remove() {
        let mut slab = Slab::<Foo>::new();
        let alloc = slab.allocator();

        let (addr1, foo1) = alloc.allocate().unwrap();
        foo1.id.store(1, SeqCst);
        assert_eq!(0, foo1.cnt.load(SeqCst));

        let (addr2, foo2) = alloc.allocate().unwrap();
        foo2.id.store(2, SeqCst);
        assert_eq!(0, foo2.cnt.load(SeqCst));

        assert_eq!(1, slab.get(addr1).unwrap().id.load(SeqCst));
        assert_eq!(2, slab.get(addr2).unwrap().id.load(SeqCst));

        drop(foo1);

        assert_eq!(1, slab.get(addr1).unwrap().id.load(SeqCst));

        let (addr3, foo3) = alloc.allocate().unwrap();
        assert_eq!(addr3, addr1);
        assert_eq!(1, foo3.cnt.load(SeqCst));
        foo3.id.store(3, SeqCst);
        assert_eq!(3, slab.get(addr3).unwrap().id.load(SeqCst));

        drop(foo2);
        drop(foo3);

        slab.compact();

        // The first page is never released
        assert!(slab.get(addr1).is_some());
        assert!(slab.get(addr2).is_some());
        assert!(slab.get(addr3).is_some());
    }

    #[test]
    fn insert_many() {
        const MANY: usize = normal_or_miri(10_000, 50);

        let mut slab = Slab::<Foo>::new();
        let alloc = slab.allocator();
        let mut entries = vec![];

        for i in 0..MANY {
            let (addr, val) = alloc.allocate().unwrap();
            val.id.store(i, SeqCst);
            entries.push((addr, val));
        }

        for (i, (addr, v)) in entries.iter().enumerate() {
            assert_eq!(i, v.id.load(SeqCst));
            assert_eq!(i, slab.get(*addr).unwrap().id.load(SeqCst));
        }

        entries.clear();

        for i in 0..MANY {
            let (addr, val) = alloc.allocate().unwrap();
            val.id.store(MANY - i, SeqCst);
            entries.push((addr, val));
        }

        for (i, (addr, v)) in entries.iter().enumerate() {
            assert_eq!(MANY - i, v.id.load(SeqCst));
            assert_eq!(MANY - i, slab.get(*addr).unwrap().id.load(SeqCst));
        }
    }

    #[test]
    fn insert_drop_reverse() {
        let mut slab = Slab::<Foo>::new();
        let alloc = slab.allocator();
        let mut entries = vec![];

        for i in 0..normal_or_miri(10_000, 100) {
            let (addr, val) = alloc.allocate().unwrap();
            val.id.store(i, SeqCst);
            entries.push((addr, val));
        }

        for _ in 0..10 {
            // Drop 1000 in reverse
            for _ in 0..normal_or_miri(1_000, 10) {
                entries.pop();
            }

            // Check remaining
            for (i, (addr, v)) in entries.iter().enumerate() {
                assert_eq!(i, v.id.load(SeqCst));
                assert_eq!(i, slab.get(*addr).unwrap().id.load(SeqCst));
            }
        }
    }

    #[test]
    fn no_compaction_if_page_still_in_use() {
        let mut slab = Slab::<Foo>::new();
        let alloc = slab.allocator();
        let mut entries1 = vec![];
        let mut entries2 = vec![];

        for i in 0..normal_or_miri(10_000, 100) {
            let (addr, val) = alloc.allocate().unwrap();
            val.id.store(i, SeqCst);

            if i % 2 == 0 {
                entries1.push((addr, val, i));
            } else {
                entries2.push(val);
            }
        }

        drop(entries2);

        for (addr, _, i) in &entries1 {
            assert_eq!(*i, slab.get(*addr).unwrap().id.load(SeqCst));
        }
    }

    const fn normal_or_miri(normal: usize, miri: usize) -> usize {
        if cfg!(miri) {
            miri
        } else {
            normal
        }
    }

    #[test]
    fn compact_all() {
        let mut slab = Slab::<Foo>::new();
        let alloc = slab.allocator();
        let mut entries = vec![];

        for _ in 0..2 {
            entries.clear();

            for i in 0..normal_or_miri(10_000, 100) {
                let (addr, val) = alloc.allocate().unwrap();
                val.id.store(i, SeqCst);

                entries.push((addr, val));
            }

            let mut addrs = vec![];

            for (addr, _) in entries.drain(..) {
                addrs.push(addr);
            }

            slab.compact();

            // The first page is never freed
            for addr in &addrs[PAGE_INITIAL_SIZE..] {
                assert!(slab.get(*addr).is_none());
            }
        }
    }

    #[test]
    fn issue_3014() {
        let mut slab = Slab::<Foo>::new();
        let alloc = slab.allocator();
        let mut entries = vec![];

        for _ in 0..normal_or_miri(5, 2) {
            entries.clear();

            // Allocate a few pages + 1
            for i in 0..(32 + 64 + 128 + 1) {
                let (addr, val) = alloc.allocate().unwrap();
                val.id.store(i, SeqCst);

                entries.push((addr, val, i));
            }

            for (addr, val, i) in &entries {
                assert_eq!(*i, val.id.load(SeqCst));
                assert_eq!(*i, slab.get(*addr).unwrap().id.load(SeqCst));
            }

            // Release the last entry
            entries.pop();

            // Compact
            slab.compact();

            // Check all the addresses

            for (addr, val, i) in &entries {
                assert_eq!(*i, val.id.load(SeqCst));
                assert_eq!(*i, slab.get(*addr).unwrap().id.load(SeqCst));
            }
        }
    }
}
