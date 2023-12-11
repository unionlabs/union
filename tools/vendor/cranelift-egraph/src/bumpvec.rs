//! Vectors allocated in arenas, with small per-vector overhead.

use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Range;

/// A vector of `T` stored within a `BumpArena`.
///
/// This is something like a normal `Vec`, except that all accesses
/// and updates require a separate borrow of the `BumpArena`. This, in
/// turn, makes the Vec itself very compact: only three `u32`s (12
/// bytes). The `BumpSlice` variant is only two `u32`s (8 bytes) and
/// is sufficient to reconstruct a slice, but not grow the vector.
///
/// The `BumpVec` does *not* implement `Clone` or `Copy`; it
/// represents unique ownership of a range of indices in the arena. If
/// dropped, those indices will be unavailable until the arena is
/// freed. This is "fine" (it is normally how arena allocation
/// works). To explicitly free and make available for some
/// allocations, a very rudimentary reuse mechanism exists via
/// `BumpVec::free(arena)`. (The allocation path opportunistically
/// checks the first range on the freelist, and can carve off a piece
/// of it if larger than needed, but it does not attempt to traverse
/// the entire freelist; this is a compromise between bump-allocation
/// speed and memory efficiency, which also influences speed through
/// cached-memory reuse.)
///
/// The type `T` should not have a `Drop` implementation. This
/// typically means that it does not own any boxed memory,
/// sub-collections, or other resources. This is important for the
/// efficiency of the data structure (otherwise, to call `Drop` impls,
/// the arena needs to track which indices are live or dead; the
/// BumpVec itself cannot do the drop because it does not retain a
/// reference to the arena). Note that placing a `T` with a `Drop`
/// impl in the arena is still *safe*, because leaking (that is, never
/// calling `Drop::drop()`) is safe. It is merely less efficient, and
/// so should be avoided if possible.
#[derive(Debug)]
pub struct BumpVec<T> {
    base: u32,
    len: u32,
    cap: u32,
    _phantom: PhantomData<T>,
}

/// A slice in an arena: like a `BumpVec`, but has a fixed size that
/// cannot grow. The size of this struct is one 32-bit word smaller
/// than `BumpVec`. It is copyable/cloneable because it will never be
/// freed.
#[derive(Debug, Clone, Copy)]
pub struct BumpSlice<T> {
    base: u32,
    len: u32,
    _phantom: PhantomData<T>,
}

#[derive(Default)]
pub struct BumpArena<T> {
    vec: Vec<MaybeUninit<T>>,
    freelist: Vec<Range<u32>>,
}

impl<T> BumpArena<T> {
    /// Create a new arena into which one can allocate `BumpVec`s.
    pub fn new() -> Self {
        Self {
            vec: vec![],
            freelist: vec![],
        }
    }

    /// Create a new arena, pre-allocating space for `cap` total `T`
    /// elements.
    pub fn arena_with_capacity(cap: usize) -> Self {
        Self {
            vec: Vec::with_capacity(cap),
            freelist: Vec::with_capacity(cap / 16),
        }
    }

    /// Create a new `BumpVec` with the given pre-allocated capacity
    /// and zero length.
    pub fn vec_with_capacity(&mut self, cap: usize) -> BumpVec<T> {
        let cap = u32::try_from(cap).unwrap();
        if let Some(range) = self.maybe_freelist_alloc(cap) {
            BumpVec {
                base: range.start,
                len: 0,
                cap,
                _phantom: PhantomData,
            }
        } else {
            let base = self.vec.len() as u32;
            for _ in 0..cap {
                self.vec.push(MaybeUninit::uninit());
            }
            BumpVec {
                base,
                len: 0,
                cap,
                _phantom: PhantomData,
            }
        }
    }

    /// Create a new `BumpVec` with a single element. The capacity is
    /// also only one element; growing the vector further will require
    /// a reallocation.
    pub fn single(&mut self, t: T) -> BumpVec<T> {
        let mut vec = self.vec_with_capacity(1);
        unsafe {
            self.write_into_index(vec.base, t);
        }
        vec.len = 1;
        vec
    }

    /// Create a new `BumpVec` with the sequence from an iterator.
    pub fn from_iter<I: Iterator<Item = T>>(&mut self, i: I) -> BumpVec<T> {
        let base = self.vec.len() as u32;
        self.vec.extend(i.map(|item| MaybeUninit::new(item)));
        let len = self.vec.len() as u32 - base;
        BumpVec {
            base,
            len,
            cap: len,
            _phantom: PhantomData,
        }
    }

    /// Append two `BumpVec`s, returning a new one. Consumes both
    /// vectors. This will use the capacity at the end of `a` if
    /// possible to move `b`'s elements into place; otherwise it will
    /// need to allocate new space.
    pub fn append(&mut self, a: BumpVec<T>, b: BumpVec<T>) -> BumpVec<T> {
        if (a.cap - a.len) >= b.len {
            self.append_into_cap(a, b)
        } else {
            self.append_into_new(a, b)
        }
    }

    /// Helper: read the `T` out of a given arena index. After
    /// reading, that index becomes uninitialized.
    unsafe fn read_out_of_index(&self, index: u32) -> T {
        // Note that we don't actually *track* uninitialized status
        // (and this is fine because we will never `Drop` and we never
        // allow a `BumpVec` to refer to an uninitialized index, so
        // the bits are effectively dead). We simply read the bits out
        // and return them.
        self.vec[index as usize].as_ptr().read()
    }

    /// Helper: write a `T` into the given arena index. Index must
    /// have been uninitialized previously.
    unsafe fn write_into_index(&mut self, index: u32, t: T) {
        self.vec[index as usize].as_mut_ptr().write(t);
    }

    /// Helper: move a `T` from one index to another. Old index
    /// becomes uninitialized and new index must have previously been
    /// uninitialized.
    unsafe fn move_item(&mut self, from: u32, to: u32) {
        let item = self.read_out_of_index(from);
        self.write_into_index(to, item);
    }

    /// Helper: push a `T` onto the end of the arena, growing its
    /// storage. The `T` to push is read out of another index, and
    /// that index subsequently becomes uninitialized.
    unsafe fn push_item(&mut self, from: u32) -> u32 {
        let index = self.vec.len() as u32;
        let item = self.read_out_of_index(from);
        self.vec.push(MaybeUninit::new(item));
        index
    }

    /// Helper: append `b` into the capacity at the end of `a`.
    fn append_into_cap(&mut self, mut a: BumpVec<T>, b: BumpVec<T>) -> BumpVec<T> {
        debug_assert!(a.cap - a.len >= b.len);
        for i in 0..b.len {
            // Safety: initially, the indices in `b` are initialized;
            // the indices in `a`'s cap, beyond its length, are
            // uninitialized. We move the initialized contents from
            // `b` to the tail beyond `a`, and we consume `b` (so it
            // no longer exists), and we update `a`'s length to cover
            // the initialized contents in their new location.
            unsafe {
                self.move_item(b.base + i, a.base + a.len + i);
            }
        }
        a.len += b.len;
        b.free(self);
        a
    }

    /// Helper: return a range of indices that are available
    /// (uninitialized) according to the freelist for `len` elements,
    /// if possible.
    fn maybe_freelist_alloc(&mut self, len: u32) -> Option<Range<u32>> {
        if let Some(entry) = self.freelist.last_mut() {
            if entry.len() >= len as usize {
                let base = entry.start;
                entry.start += len;
                if entry.start == entry.end {
                    self.freelist.pop();
                }
                return Some(base..(base + len));
            }
        }
        None
    }

    /// Helper: append `a` and `b` into a completely new allocation.
    fn append_into_new(&mut self, a: BumpVec<T>, b: BumpVec<T>) -> BumpVec<T> {
        // New capacity: round up to a power of two.
        let len = a.len + b.len;
        let cap = round_up_power_of_two(len);

        if let Some(range) = self.maybe_freelist_alloc(cap) {
            for i in 0..a.len {
                // Safety: the indices in `a` must be initialized. We read
                // out the item and copy it to a new index; the old index
                // is no longer covered by a BumpVec, because we consume
                // `a`.
                unsafe {
                    self.move_item(a.base + i, range.start + i);
                }
            }
            for i in 0..b.len {
                // Safety: the indices in `b` must be initialized. We read
                // out the item and copy it to a new index; the old index
                // is no longer covered by a BumpVec, because we consume
                // `b`.
                unsafe {
                    self.move_item(b.base + i, range.start + a.len + i);
                }
            }

            a.free(self);
            b.free(self);

            BumpVec {
                base: range.start,
                len,
                cap,
                _phantom: PhantomData,
            }
        } else {
            self.vec.reserve(cap as usize);
            let base = self.vec.len() as u32;
            for i in 0..a.len {
                // Safety: the indices in `a` must be initialized. We read
                // out the item and copy it to a new index; the old index
                // is no longer covered by a BumpVec, because we consume
                // `a`.
                unsafe {
                    self.push_item(a.base + i);
                }
            }
            for i in 0..b.len {
                // Safety: the indices in `b` must be initialized. We read
                // out the item and copy it to a new index; the old index
                // is no longer covered by a BumpVec, because we consume
                // `b`.
                unsafe {
                    self.push_item(b.base + i);
                }
            }
            let len = self.vec.len() as u32 - base;

            for _ in len..cap {
                self.vec.push(MaybeUninit::uninit());
            }

            a.free(self);
            b.free(self);

            BumpVec {
                base,
                len,
                cap,
                _phantom: PhantomData,
            }
        }
    }

    /// Returns the size of the backing `Vec`.
    pub fn size(&self) -> usize {
        self.vec.len()
    }
}

fn round_up_power_of_two(x: u32) -> u32 {
    debug_assert!(x > 0);
    debug_assert!(x < 0x8000_0000);
    let log2 = 32 - (x - 1).leading_zeros();
    1 << log2
}

impl<T> BumpVec<T> {
    /// Returns a slice view of this `BumpVec`, given a borrow of the
    /// arena.
    pub fn as_slice<'a>(&'a self, arena: &'a BumpArena<T>) -> &'a [T] {
        let maybe_uninit_slice =
            &arena.vec[(self.base as usize)..((self.base + self.len) as usize)];
        // Safety: the index range we represent must be initialized.
        unsafe { std::mem::transmute(maybe_uninit_slice) }
    }

    /// Returns a mutable slice view of this `BumpVec`, given a
    /// mutable borrow of the arena.
    pub fn as_mut_slice<'a>(&'a mut self, arena: &'a mut BumpArena<T>) -> &'a mut [T] {
        let maybe_uninit_slice =
            &mut arena.vec[(self.base as usize)..((self.base + self.len) as usize)];
        // Safety: the index range we represent must be initialized.
        unsafe { std::mem::transmute(maybe_uninit_slice) }
    }

    /// Returns the length of this vector. Does not require access to
    /// the arena.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the capacity of this vector. Does not require access
    /// to the arena.
    pub fn cap(&self) -> usize {
        self.cap as usize
    }

    /// Reserve `extra_len` capacity at the end of the vector,
    /// reallocating if necessary.
    pub fn reserve(&mut self, extra_len: usize, arena: &mut BumpArena<T>) {
        let extra_len = u32::try_from(extra_len).unwrap();
        if self.cap - self.len < extra_len {
            if self.base + self.cap == arena.vec.len() as u32 {
                for _ in 0..extra_len {
                    arena.vec.push(MaybeUninit::uninit());
                }
                self.cap += extra_len;
            } else {
                let new_cap = self.cap + extra_len;
                let new = arena.vec_with_capacity(new_cap as usize);
                unsafe {
                    for i in 0..self.len {
                        arena.move_item(self.base + i, new.base + i);
                    }
                }
                self.base = new.base;
                self.cap = new.cap;
            }
        }
    }

    /// Push an item, growing the capacity if needed.
    pub fn push(&mut self, t: T, arena: &mut BumpArena<T>) {
        if self.cap > self.len {
            unsafe {
                arena.write_into_index(self.base + self.len, t);
            }
            self.len += 1;
        } else if (self.base + self.cap) as usize == arena.vec.len() {
            arena.vec.push(MaybeUninit::new(t));
            self.cap += 1;
            self.len += 1;
        } else {
            let new_cap = round_up_power_of_two(self.cap + 1);
            let extra = new_cap - self.cap;
            self.reserve(extra as usize, arena);
            unsafe {
                arena.write_into_index(self.base + self.len, t);
            }
            self.len += 1;
        }
    }

    /// Clone, if `T` is cloneable.
    pub fn clone(&self, arena: &mut BumpArena<T>) -> BumpVec<T>
    where
        T: Clone,
    {
        let mut new = arena.vec_with_capacity(self.len as usize);
        for i in 0..self.len {
            let item = self.as_slice(arena)[i as usize].clone();
            new.push(item, arena);
        }
        new
    }

    /// Truncate the length to a smaller-or-equal length.
    pub fn truncate(&mut self, len: usize) {
        let len = len as u32;
        assert!(len <= self.len);
        self.len = len;
    }

    /// Consume the BumpVec and return its indices to a free pool in
    /// the arena.
    pub fn free(self, arena: &mut BumpArena<T>) {
        arena.freelist.push(self.base..(self.base + self.cap));
    }

    /// Freeze the capacity of this BumpVec, turning it into a slice,
    /// for a smaller struct (8 bytes rather than 12). Once this
    /// exists, it is copyable, because the slice will never be freed.
    pub fn freeze(self, arena: &mut BumpArena<T>) -> BumpSlice<T> {
        if self.cap > self.len {
            arena
                .freelist
                .push((self.base + self.len)..(self.base + self.cap));
        }
        BumpSlice {
            base: self.base,
            len: self.len,
            _phantom: PhantomData,
        }
    }
}

impl<T> BumpSlice<T> {
    /// Returns a slice view of the `BumpSlice`, given a borrow of the
    /// arena.
    pub fn as_slice<'a>(&'a self, arena: &'a BumpArena<T>) -> &'a [T] {
        let maybe_uninit_slice =
            &arena.vec[(self.base as usize)..((self.base + self.len) as usize)];
        // Safety: the index range we represent must be initialized.
        unsafe { std::mem::transmute(maybe_uninit_slice) }
    }

    /// Returns a mutable slice view of the `BumpSlice`, given a
    /// mutable borrow of the arena.
    pub fn as_mut_slice<'a>(&'a mut self, arena: &'a mut BumpArena<T>) -> &'a mut [T] {
        let maybe_uninit_slice =
            &mut arena.vec[(self.base as usize)..((self.base + self.len) as usize)];
        // Safety: the index range we represent must be initialized.
        unsafe { std::mem::transmute(maybe_uninit_slice) }
    }

    /// Returns the length of the `BumpSlice`.
    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl<T> std::default::Default for BumpVec<T> {
    fn default() -> Self {
        BumpVec {
            base: 0,
            len: 0,
            cap: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T> std::default::Default for BumpSlice<T> {
    fn default() -> Self {
        BumpSlice {
            base: 0,
            len: 0,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_up() {
        assert_eq!(1, round_up_power_of_two(1));
        assert_eq!(2, round_up_power_of_two(2));
        assert_eq!(4, round_up_power_of_two(3));
        assert_eq!(4, round_up_power_of_two(4));
        assert_eq!(32, round_up_power_of_two(24));
        assert_eq!(0x8000_0000, round_up_power_of_two(0x7fff_ffff));
    }

    #[test]
    fn test_basic() {
        let mut arena: BumpArena<u32> = BumpArena::new();

        let a = arena.single(1);
        let b = arena.single(2);
        let c = arena.single(3);
        let ab = arena.append(a, b);
        assert_eq!(ab.as_slice(&arena), &[1, 2]);
        assert_eq!(ab.cap(), 2);
        let abc = arena.append(ab, c);
        assert_eq!(abc.len(), 3);
        assert_eq!(abc.cap(), 4);
        assert_eq!(abc.as_slice(&arena), &[1, 2, 3]);
        assert_eq!(arena.size(), 9);
        let mut d = arena.single(4);
        // Should have reused the freelist.
        assert_eq!(arena.size(), 9);
        assert_eq!(d.len(), 1);
        assert_eq!(d.cap(), 1);
        assert_eq!(d.as_slice(&arena), &[4]);
        d.as_mut_slice(&mut arena)[0] = 5;
        assert_eq!(d.as_slice(&arena), &[5]);
        abc.free(&mut arena);
        let d2 = d.clone(&mut arena);
        let dd = arena.append(d, d2);
        // Should have reused the freelist.
        assert_eq!(arena.size(), 9);
        assert_eq!(dd.as_slice(&arena), &[5, 5]);
        let mut e = arena.from_iter([10, 11, 12].into_iter());
        e.push(13, &mut arena);
        assert_eq!(arena.size(), 13);
        e.reserve(4, &mut arena);
        assert_eq!(arena.size(), 17);
        let _f = arena.from_iter([1, 2, 3, 4, 5, 6, 7, 8].into_iter());
        assert_eq!(arena.size(), 25);
        e.reserve(8, &mut arena);
        assert_eq!(e.cap(), 16);
        assert_eq!(e.as_slice(&arena), &[10, 11, 12, 13]);
        // `e` must have been copied now that `f` is at the end of the
        // arena.
        assert_eq!(arena.size(), 41);
    }
}
