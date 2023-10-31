#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![no_std]
#![cfg_attr(
    feature = "allocator_api",
    feature(allocator_api, nonnull_slice_from_raw_parts)
)]

#[doc(hidden)]
pub extern crate alloc as core_alloc;

#[cfg(feature = "boxed")]
pub mod boxed;
#[cfg(feature = "collections")]
pub mod collections;

mod alloc;

use core::cell::Cell;
use core::fmt::Display;
use core::iter;
use core::marker::PhantomData;
use core::mem;
use core::ptr::{self, NonNull};
use core::slice;
use core::str;
use core_alloc::alloc::{alloc, dealloc, Layout};
#[cfg(feature = "allocator_api")]
use core_alloc::alloc::{AllocError, Allocator};

pub use alloc::AllocErr;

/// An error returned from [`Bump::try_alloc_try_with`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AllocOrInitError<E> {
    /// Indicates that the initial allocation failed.
    Alloc(AllocErr),
    /// Indicates that the initializer failed with the contained error after
    /// allocation.
    ///
    /// It is possible but not guaranteed that the allocated memory has been
    /// released back to the allocator at this point.
    Init(E),
}
impl<E> From<AllocErr> for AllocOrInitError<E> {
    fn from(e: AllocErr) -> Self {
        Self::Alloc(e)
    }
}
impl<E: Display> Display for AllocOrInitError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AllocOrInitError::Alloc(err) => err.fmt(f),
            AllocOrInitError::Init(err) => write!(f, "initialization failed: {}", err),
        }
    }
}

/// An arena to bump allocate into.
///
/// ## No `Drop`s
///
/// Objects that are bump-allocated will never have their [`Drop`] implementation
/// called &mdash; unless you do it manually yourself. This makes it relatively
/// easy to leak memory or other resources.
///
/// If you have a type which internally manages
///
/// * an allocation from the global heap (e.g. [`Vec<T>`]),
/// * open file descriptors (e.g. [`std::fs::File`]), or
/// * any other resource that must be cleaned up (e.g. an `mmap`)
///
/// and relies on its `Drop` implementation to clean up the internal resource,
/// then if you allocate that type with a `Bump`, you need to find a new way to
/// clean up after it yourself.
///
/// Potential solutions are:
///
/// * Using [`bumpalo::boxed::Box::new_in`] instead of [`Bump::alloc`], that
///   will drop wrapped values similarly to [`std::boxed::Box`]. Note that this
///   requires enabling the `"boxed"` Cargo feature for this crate. **This is
///   often the easiest solution.**
///
/// * Calling [`drop_in_place`][drop_in_place] or using
///   [`std::mem::ManuallyDrop`][manuallydrop] to manually drop these types.
///
/// * Using [`bumpalo::collections::Vec`] instead of [`std::vec::Vec`].
///
/// * Avoiding allocating these problematic types within a `Bump`.
///
/// Note that not calling `Drop` is memory safe! Destructors are never
/// guaranteed to run in Rust, you can't rely on them for enforcing memory
/// safety.
///
/// [`Drop`]: https://doc.rust-lang.org/std/ops/trait.Drop.html
/// [`Vec<T>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
/// [drop_in_place]: https://doc.rust-lang.org/std/ptr/fn.drop_in_place.html
/// [manuallydrop]: https://doc.rust-lang.org/std/mem/struct.ManuallyDrop.html
/// [`bumpalo::collections::Vec`]: collections/vec/struct.Vec.html
/// [`std::vec::Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`bumpalo::boxed::Box::new_in`]: boxed/struct.Box.html#method.new_in
/// [`std::boxed::Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
///
/// ## Example
///
/// ```
/// use bumpalo::Bump;
///
/// // Create a new bump arena.
/// let bump = Bump::new();
///
/// // Allocate values into the arena.
/// let forty_two = bump.alloc(42);
/// assert_eq!(*forty_two, 42);
///
/// // Mutable references are returned from allocation.
/// let mut s = bump.alloc("bumpalo");
/// *s = "the bump allocator; and also is a buffalo";
/// ```
///
/// ## Allocation Methods Come in Many Flavors
///
/// There are various allocation methods on `Bump`, the simplest being
/// [`alloc`][Bump::alloc]. The others exist to satisfy some combination of
/// fallible allocation and initialization. The allocation methods are
/// summarized in the following table:
///
/// <table>
///   <thead>
///     <tr>
///       <th></th>
///       <th>Infallible Allocation</th>
///       <th>Fallible Allocation</th>
///     </tr>
///   </thead>
///     <tr>
///       <th>By Value</th>
///       <td><a href="#method.alloc"><code>alloc</code></a></td>
///       <td><a href="#method.try_alloc"><code>try_alloc</code></a></td>
///     </tr>
///     <tr>
///       <th>Infallible Initializer Function</th>
///       <td><a href="#method.alloc_with"><code>alloc_with</code></a></td>
///       <td><a href="#method.try_alloc_with"><code>try_alloc_with</code></a></td>
///     </tr>
///     <tr>
///       <th>Fallible Initializer Function</th>
///       <td><a href="#method.alloc_try_with"><code>alloc_try_with</code></a></td>
///       <td><a href="#method.try_alloc_try_with"><code>try_alloc_try_with</code></a></td>
///     </tr>
///   <tbody>
///   </tbody>
/// </table>
///
/// ### Fallible Allocation: The `try_alloc_` Method Prefix
///
/// These allocation methods let you recover from out-of-memory (OOM)
/// scenarioes, rather than raising a panic on OOM.
///
/// ```
/// use bumpalo::Bump;
///
/// let bump = Bump::new();
///
/// match bump.try_alloc(MyStruct {
///     // ...
/// }) {
///     Ok(my_struct) => {
///         // Allocation succeeded.
///     }
///     Err(e) => {
///         // Out of memory.
///     }
/// }
///
/// struct MyStruct {
///     // ...
/// }
/// ```
///
/// ### Initializer Functions: The `_with` Method Suffix
///
/// Calling one of the generic `…alloc(x)` methods is essentially equivalent to
/// the matching [`…alloc_with(|| x)`](?search=alloc_with). However if you use
/// `…alloc_with`, then the closure will not be invoked until after allocating
/// space for storing `x` on the heap.
///
/// This can be useful in certain edge-cases related to compiler optimizations.
/// When evaluating for example `bump.alloc(x)`, semantically `x` is first put
/// on the stack and then moved onto the heap. In some cases, the compiler is
/// able to optimize this into constructing `x` directly on the heap, however
/// in many cases it does not.
///
/// The `…alloc_with` functions try to help the compiler be smarter. In most
/// cases doing for example `bump.try_alloc_with(|| x)` on release mode will be
/// enough to help the compiler realize that this optimization is valid and
/// to construct `x` directly onto the heap.
///
/// #### Warning
///
/// These functions critically depend on compiler optimizations to achieve their
/// desired effect. This means that it is not an effective tool when compiling
/// without optimizations on.
///
/// Even when optimizations are on, these functions do not **guarantee** that
/// the value is constructed on the heap. To the best of our knowledge no such
/// guarantee can be made in stable Rust as of 1.54.
///
/// ### Fallible Initialization: The `_try_with` Method Suffix
///
/// The generic [`…alloc_try_with(|| x)`](?search=_try_with) methods behave
/// like the purely `_with` suffixed methods explained above. However, they
/// allow for fallible initialization by accepting a closure that returns a
/// [`Result`] and will attempt to undo the initial allocation if this closure
/// returns [`Err`].
///
/// #### Warning
///
/// If the inner closure returns [`Ok`], space for the entire [`Result`] remains
/// allocated inside `self`. This can be a problem especially if the [`Err`]
/// variant is larger, but even otherwise there may be overhead for the
/// [`Result`]'s discriminant.
///
/// <p><details><summary>Undoing the allocation in the <code>Err</code> case
/// always fails if <code>f</code> successfully made any additional allocations
/// in <code>self</code>.</summary>
///
/// For example, the following will always leak also space for the [`Result`]
/// into this `Bump`, even though the inner reference isn't kept and the [`Err`]
/// payload is returned semantically by value:
///
/// ```rust
/// let bump = bumpalo::Bump::new();
///
/// let r: Result<&mut [u8; 1000], ()> = bump.alloc_try_with(|| {
///     let _ = bump.alloc(0_u8);
///     Err(())
/// });
///
/// assert!(r.is_err());
/// ```
///
///</details></p>
///
/// Since [`Err`] payloads are first placed on the heap and then moved to the
/// stack, `bump.…alloc_try_with(|| x)?` is likely to execute more slowly than
/// the matching `bump.…alloc(x?)` in case of initialization failure. If this
/// happens frequently, using the plain un-suffixed method may perform better.
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
/// [`Ok`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
/// [`Err`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
#[derive(Debug)]
pub struct Bump {
    // The current chunk we are bump allocating within.
    current_chunk_footer: Cell<NonNull<ChunkFooter>>,
}

#[repr(C)]
#[derive(Debug)]
struct ChunkFooter {
    // Pointer to the start of this chunk allocation. This footer is always at
    // the end of the chunk.
    data: NonNull<u8>,

    // The layout of this chunk's allocation.
    layout: Layout,

    // Link to the previous chunk.
    //
    // Note that the last node in the `prev` linked list is the canonical empty
    // chunk, whose `prev` link points to itself.
    prev: Cell<NonNull<ChunkFooter>>,

    // Bump allocation finger that is always in the range `self.data..=self`.
    ptr: Cell<NonNull<u8>>,
}

/// A wrapper type for the canonical, statically allocated empty chunk.
///
/// For the canonical empty chunk to be `static`, its type must be `Sync`, which
/// is the purpose of this wrapper type. This is safe because the empty chunk is
/// immutable and never actually modified.
#[repr(transparent)]
struct EmptyChunkFooter(ChunkFooter);

unsafe impl Sync for EmptyChunkFooter {}

static EMPTY_CHUNK: EmptyChunkFooter = EmptyChunkFooter(ChunkFooter {
    // This chunk is empty (except the foot itself).
    layout: Layout::new::<ChunkFooter>(),

    // The start of the (empty) allocatable region for this chunk is itself.
    data: unsafe { NonNull::new_unchecked(&EMPTY_CHUNK as *const EmptyChunkFooter as *mut u8) },

    // The end of the (empty) allocatable region for this chunk is also itself.
    ptr: Cell::new(unsafe {
        NonNull::new_unchecked(&EMPTY_CHUNK as *const EmptyChunkFooter as *mut u8)
    }),

    // Invariant: the last chunk footer in all `ChunkFooter::prev` linked lists
    // is the empty chunk footer, whose `prev` points to itself.
    prev: Cell::new(unsafe {
        NonNull::new_unchecked(&EMPTY_CHUNK as *const EmptyChunkFooter as *mut ChunkFooter)
    }),
});

impl EmptyChunkFooter {
    fn get(&'static self) -> NonNull<ChunkFooter> {
        unsafe { NonNull::new_unchecked(&self.0 as *const ChunkFooter as *mut ChunkFooter) }
    }
}

impl ChunkFooter {
    // Returns the start and length of the currently allocated region of this
    // chunk.
    fn as_raw_parts(&self) -> (*const u8, usize) {
        let data = self.data.as_ptr() as *const u8;
        let ptr = self.ptr.get().as_ptr() as *const u8;
        debug_assert!(data <= ptr);
        debug_assert!(ptr <= self as *const ChunkFooter as *const u8);
        let len = unsafe { (self as *const ChunkFooter as *const u8).offset_from(ptr) as usize };
        (ptr, len)
    }

    /// Is this chunk the last empty chunk?
    fn is_empty(&self) -> bool {
        ptr::eq(self, EMPTY_CHUNK.get().as_ptr())
    }
}

impl Default for Bump {
    fn default() -> Bump {
        Bump::new()
    }
}

impl Drop for Bump {
    fn drop(&mut self) {
        unsafe {
            dealloc_chunk_list(self.current_chunk_footer.get());
        }
    }
}

#[inline]
unsafe fn dealloc_chunk_list(mut footer: NonNull<ChunkFooter>) {
    while !footer.as_ref().is_empty() {
        let f = footer;
        footer = f.as_ref().prev.get();
        dealloc(f.as_ref().data.as_ptr(), f.as_ref().layout);
    }
}

// `Bump`s are safe to send between threads because nothing aliases its owned
// chunks until you start allocating from it. But by the time you allocate from
// it, the returned references to allocations borrow the `Bump` and therefore
// prevent sending the `Bump` across threads until the borrows end.
unsafe impl Send for Bump {}

#[inline]
pub(crate) fn round_up_to(n: usize, divisor: usize) -> Option<usize> {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    Some(n.checked_add(divisor - 1)? & !(divisor - 1))
}

#[inline]
pub(crate) fn round_down_to(n: usize, divisor: usize) -> usize {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    n & !(divisor - 1)
}

// After this point, we try to hit page boundaries instead of powers of 2
const PAGE_STRATEGY_CUTOFF: usize = 0x1000;

// We only support alignments of up to 16 bytes for iter_allocated_chunks.
const SUPPORTED_ITER_ALIGNMENT: usize = 16;
const CHUNK_ALIGN: usize = SUPPORTED_ITER_ALIGNMENT;
const FOOTER_SIZE: usize = mem::size_of::<ChunkFooter>();

// Assert that ChunkFooter is at most the supported alignment. This will give a compile time error if it is not the case
const _FOOTER_ALIGN_ASSERTION: bool = mem::align_of::<ChunkFooter>() <= CHUNK_ALIGN;
const _: [(); _FOOTER_ALIGN_ASSERTION as usize] = [()];

// Maximum typical overhead per allocation imposed by allocators.
const MALLOC_OVERHEAD: usize = 16;

// This is the overhead from malloc, footer and alignment. For instance, if
// we want to request a chunk of memory that has at least X bytes usable for
// allocations (where X is aligned to CHUNK_ALIGN), then we expect that the
// after adding a footer, malloc overhead and alignment, the chunk of memory
// the allocator actually sets aside for us is X+OVERHEAD rounded up to the
// nearest suitable size boundary.
const OVERHEAD: usize = (MALLOC_OVERHEAD + FOOTER_SIZE + (CHUNK_ALIGN - 1)) & !(CHUNK_ALIGN - 1);

// Choose a relatively small default initial chunk size, since we double chunk
// sizes as we grow bump arenas to amortize costs of hitting the global
// allocator.
const FIRST_ALLOCATION_GOAL: usize = 1 << 9;

// The actual size of the first allocation is going to be a bit smaller
// than the goal. We need to make room for the footer, and we also need
// take the alignment into account.
const DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER: usize = FIRST_ALLOCATION_GOAL - OVERHEAD;

/// Wrapper around `Layout::from_size_align` that adds debug assertions.
#[inline]
unsafe fn layout_from_size_align(size: usize, align: usize) -> Layout {
    if cfg!(debug_assertions) {
        Layout::from_size_align(size, align).unwrap()
    } else {
        Layout::from_size_align_unchecked(size, align)
    }
}

#[inline(never)]
fn allocation_size_overflow<T>() -> T {
    panic!("requested allocation size overflowed")
}

impl Bump {
    /// Construct a new arena to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// # let _ = bump;
    /// ```
    pub fn new() -> Bump {
        Self::with_capacity(0)
    }

    /// Attempt to construct a new arena to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::try_new();
    /// # let _ = bump.unwrap();
    /// ```
    pub fn try_new() -> Result<Bump, AllocErr> {
        Bump::try_with_capacity(0)
    }

    /// Construct a new arena with the specified byte capacity to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::with_capacity(100);
    /// # let _ = bump;
    /// ```
    pub fn with_capacity(capacity: usize) -> Bump {
        Bump::try_with_capacity(capacity).unwrap_or_else(|_| oom())
    }

    /// Attempt to construct a new arena with the specified byte capacity to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::try_with_capacity(100);
    /// # let _ = bump.unwrap();
    /// ```
    pub fn try_with_capacity(capacity: usize) -> Result<Self, AllocErr> {
        if capacity == 0 {
            return Ok(Bump {
                current_chunk_footer: Cell::new(EMPTY_CHUNK.get()),
            });
        }

        let chunk_footer = Self::new_chunk(
            None,
            unsafe { layout_from_size_align(capacity, 1) },
            EMPTY_CHUNK.get(),
        )
        .ok_or(AllocErr)?;

        Ok(Bump {
            current_chunk_footer: Cell::new(chunk_footer),
        })
    }

    /// Allocate a new chunk and return its initialized footer.
    ///
    /// If given, `layouts` is a tuple of the current chunk size and the
    /// layout of the allocation request that triggered us to fall back to
    /// allocating a new chunk of memory.
    fn new_chunk(
        new_size_without_footer: Option<usize>,
        requested_layout: Layout,
        prev: NonNull<ChunkFooter>,
    ) -> Option<NonNull<ChunkFooter>> {
        unsafe {
            let mut new_size_without_footer =
                new_size_without_footer.unwrap_or(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);

            // We want to have CHUNK_ALIGN or better alignment
            let mut align = CHUNK_ALIGN;

            // If we already know we need to fulfill some request,
            // make sure we allocate at least enough to satisfy it
            align = align.max(requested_layout.align());
            let requested_size = round_up_to(requested_layout.size(), align)
                .unwrap_or_else(allocation_size_overflow);
            new_size_without_footer = new_size_without_footer.max(requested_size);

            // We want our allocations to play nice with the memory allocator,
            // and waste as little memory as possible.
            // For small allocations, this means that the entire allocation
            // including the chunk footer and mallocs internal overhead is
            // as close to a power of two as we can go without going over.
            // For larger allocations, we only need to get close to a page
            // boundary without going over.
            if new_size_without_footer < PAGE_STRATEGY_CUTOFF {
                new_size_without_footer =
                    (new_size_without_footer + OVERHEAD).next_power_of_two() - OVERHEAD;
            } else {
                new_size_without_footer =
                    round_up_to(new_size_without_footer + OVERHEAD, 0x1000)? - OVERHEAD;
            }

            debug_assert_eq!(align % CHUNK_ALIGN, 0);
            debug_assert_eq!(new_size_without_footer % CHUNK_ALIGN, 0);
            let size = new_size_without_footer
                .checked_add(FOOTER_SIZE)
                .unwrap_or_else(allocation_size_overflow);
            let layout = layout_from_size_align(size, align);

            debug_assert!(size >= requested_layout.size());

            let data = alloc(layout);
            let data = NonNull::new(data)?;

            // The `ChunkFooter` is at the end of the chunk.
            let footer_ptr = data.as_ptr().add(new_size_without_footer);
            debug_assert_eq!((data.as_ptr() as usize) % align, 0);
            debug_assert_eq!(footer_ptr as usize % CHUNK_ALIGN, 0);
            let footer_ptr = footer_ptr as *mut ChunkFooter;

            // The bump pointer is initialized to the end of the range we will
            // bump out of.
            let ptr = Cell::new(NonNull::new_unchecked(footer_ptr as *mut u8));

            ptr::write(
                footer_ptr,
                ChunkFooter {
                    data,
                    layout,
                    prev: Cell::new(prev),
                    ptr,
                },
            );

            Some(NonNull::new_unchecked(footer_ptr))
        }
    }

    /// Reset this bump allocator.
    ///
    /// Performs mass deallocation on everything allocated in this arena by
    /// resetting the pointer into the underlying chunk of memory to the start
    /// of the chunk. Does not run any `Drop` implementations on deallocated
    /// objects; see [the top-level documentation](struct.Bump.html) for details.
    ///
    /// If this arena has allocated multiple chunks to bump allocate into, then
    /// the excess chunks are returned to the global allocator.
    ///
    /// ## Example
    ///
    /// ```
    /// let mut bump = bumpalo::Bump::new();
    ///
    /// // Allocate a bunch of things.
    /// {
    ///     for i in 0..100 {
    ///         bump.alloc(i);
    ///     }
    /// }
    ///
    /// // Reset the arena.
    /// bump.reset();
    ///
    /// // Allocate some new things in the space previously occupied by the
    /// // original things.
    /// for j in 200..400 {
    ///     bump.alloc(j);
    /// }
    ///```
    pub fn reset(&mut self) {
        // Takes `&mut self` so `self` must be unique and there can't be any
        // borrows active that would get invalidated by resetting.
        unsafe {
            if self.current_chunk_footer.get().as_ref().is_empty() {
                return;
            }

            let cur_chunk = self.current_chunk_footer.get();

            // Deallocate all chunks except the current one
            let prev_chunk = cur_chunk.as_ref().prev.replace(EMPTY_CHUNK.get());
            dealloc_chunk_list(prev_chunk);

            // Reset the bump finger to the end of the chunk.
            cur_chunk.as_ref().ptr.set(cur_chunk.cast());

            debug_assert!(
                self.current_chunk_footer
                    .get()
                    .as_ref()
                    .prev
                    .get()
                    .as_ref()
                    .is_empty(),
                "We should only have a single chunk"
            );
            debug_assert_eq!(
                self.current_chunk_footer.get().as_ref().ptr.get(),
                self.current_chunk_footer.get().cast(),
                "Our chunk's bump finger should be reset to the start of its allocation"
            );
        }
    }

    /// Allocate an object in this `Bump` and return an exclusive reference to
    /// it.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for `T` fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc("hello");
    /// assert_eq!(*x, "hello");
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.alloc_with(|| val)
    }

    /// Try to allocate an object in this `Bump` and return an exclusive
    /// reference to it.
    ///
    /// ## Errors
    ///
    /// Errors if reserving space for `T` fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.try_alloc("hello");
    /// assert_eq!(x, Ok(&mut "hello"));
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn try_alloc<T>(&self, val: T) -> Result<&mut T, AllocErr> {
        self.try_alloc_with(|| val)
    }

    /// Pre-allocate space for an object in this `Bump`, initializes it using
    /// the closure, then returns an exclusive reference to it.
    ///
    /// See [The `_with` Method Suffix](#initializer-functions-the-_with-method-suffix) for a
    /// discussion on the differences between the `_with` suffixed methods and
    /// those methods without it, their performance characteristics, and when
    /// you might or might not choose a `_with` suffixed method.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for `T` fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc_with(|| "hello");
    /// assert_eq!(*x, "hello");
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_with<F, T>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        unsafe fn inner_writer<T, F>(ptr: *mut T, f: F)
        where
            F: FnOnce() -> T,
        {
            // This function is translated as:
            // - allocate space for a T on the stack
            // - call f() with the return value being put onto this stack space
            // - memcpy from the stack to the heap
            //
            // Ideally we want LLVM to always realize that doing a stack
            // allocation is unnecessary and optimize the code so it writes
            // directly into the heap instead. It seems we get it to realize
            // this most consistently if we put this critical line into it's
            // own function instead of inlining it into the surrounding code.
            ptr::write(ptr, f())
        }

        let layout = Layout::new::<T>();

        unsafe {
            let p = self.alloc_layout(layout);
            let p = p.as_ptr() as *mut T;
            inner_writer(p, f);
            &mut *p
        }
    }

    /// Tries to pre-allocate space for an object in this `Bump`, initializes
    /// it using the closure, then returns an exclusive reference to it.
    ///
    /// See [The `_with` Method Suffix](#initializer-functions-the-_with-method-suffix) for a
    /// discussion on the differences between the `_with` suffixed methods and
    /// those methods without it, their performance characteristics, and when
    /// you might or might not choose a `_with` suffixed method.
    ///
    /// ## Errors
    ///
    /// Errors if reserving space for `T` fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.try_alloc_with(|| "hello");
    /// assert_eq!(x, Ok(&mut "hello"));
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn try_alloc_with<F, T>(&self, f: F) -> Result<&mut T, AllocErr>
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        unsafe fn inner_writer<T, F>(ptr: *mut T, f: F)
        where
            F: FnOnce() -> T,
        {
            // This function is translated as:
            // - allocate space for a T on the stack
            // - call f() with the return value being put onto this stack space
            // - memcpy from the stack to the heap
            //
            // Ideally we want LLVM to always realize that doing a stack
            // allocation is unnecessary and optimize the code so it writes
            // directly into the heap instead. It seems we get it to realize
            // this most consistently if we put this critical line into it's
            // own function instead of inlining it into the surrounding code.
            ptr::write(ptr, f())
        }

        //SAFETY: Self-contained:
        // `p` is allocated for `T` and then a `T` is written.
        let layout = Layout::new::<T>();
        let p = self.try_alloc_layout(layout)?;
        let p = p.as_ptr() as *mut T;

        unsafe {
            inner_writer(p, f);
            Ok(&mut *p)
        }
    }

    /// Pre-allocates space for a [`Result`] in this `Bump`, initializes it using
    /// the closure, then returns an exclusive reference to its `T` if [`Ok`].
    ///
    /// Iff the allocation fails, the closure is not run.
    ///
    /// Iff [`Err`], an allocator rewind is *attempted* and the `E` instance is
    /// moved out of the allocator to be consumed or dropped as normal.
    ///
    /// See [The `_with` Method Suffix](#initializer-functions-the-_with-method-suffix) for a
    /// discussion on the differences between the `_with` suffixed methods and
    /// those methods without it, their performance characteristics, and when
    /// you might or might not choose a `_with` suffixed method.
    ///
    /// For caveats specific to fallible initialization, see
    /// [The `_try_with` Method Suffix](#fallible-initialization-the-_try_with-method-suffix).
    ///
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`Ok`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    ///
    /// ## Errors
    ///
    /// Iff the allocation succeeds but `f` fails, that error is forwarded by value.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for `Result<T, E>` fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc_try_with(|| Ok("hello"))?;
    /// assert_eq!(*x, "hello");
    /// # Result::<_, ()>::Ok(())
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_try_with<F, T, E>(&self, f: F) -> Result<&mut T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let rewind_footer = self.current_chunk_footer.get();
        let rewind_ptr = unsafe { rewind_footer.as_ref() }.ptr.get();
        let mut inner_result_ptr = NonNull::from(self.alloc_with(f));
        let inner_result_address = inner_result_ptr.as_ptr() as usize;
        match unsafe { inner_result_ptr.as_mut() } {
            Ok(t) => Ok(unsafe {
                //SAFETY:
                // The `&mut Result<T, E>` returned by `alloc_with` may be
                // lifetime-limited by `E`, but the derived `&mut T` still has
                // the same validity as in `alloc_with` since the error variant
                // is already ruled out here.

                // We could conditionally truncate the allocation here, but
                // since it grows backwards, it seems unlikely that we'd get
                // any more than the `Result`'s discriminant this way, if
                // anything at all.
                &mut *(t as *mut _)
            }),
            Err(e) => unsafe {
                // If this result was the last allocation in this arena, we can
                // reclaim its space. In fact, sometimes we can do even better
                // than simply calling `dealloc` on the result pointer: we can
                // reclaim any alignment padding we might have added (which
                // `dealloc` cannot do) if we didn't allocate a new chunk for
                // this result.
                if self.is_last_allocation(NonNull::new_unchecked(inner_result_address as *mut _)) {
                    let current_footer_p = self.current_chunk_footer.get();
                    let current_ptr = &current_footer_p.as_ref().ptr;
                    if current_footer_p == rewind_footer {
                        // It's still the same chunk, so reset the bump pointer
                        // to its original value upon entry to this method
                        // (reclaiming any alignment padding we may have
                        // added).
                        current_ptr.set(rewind_ptr);
                    } else {
                        // We allocated a new chunk for this result.
                        //
                        // We know the result is the only allocation in this
                        // chunk: Any additional allocations since the start of
                        // this method could only have happened when running
                        // the initializer function, which is called *after*
                        // reserving space for this result. Therefore, since we
                        // already determined via the check above that this
                        // result was the last allocation, there must not have
                        // been any other allocations, and this result is the
                        // only allocation in this chunk.
                        //
                        // Because this is the only allocation in this chunk,
                        // we can reset the chunk's bump finger to the start of
                        // the chunk.
                        current_ptr.set(current_footer_p.as_ref().data);
                    }
                }
                //SAFETY:
                // As we received `E` semantically by value from `f`, we can
                // just copy that value here as long as we avoid a double-drop
                // (which can't happen as any specific references to the `E`'s
                // data in `self` are destroyed when this function returns).
                //
                // The order between this and the deallocation doesn't matter
                // because `Self: !Sync`.
                Err(ptr::read(e as *const _))
            },
        }
    }

    /// Tries to pre-allocates space for a [`Result`] in this `Bump`,
    /// initializes it using the closure, then returns an exclusive reference
    /// to its `T` if all [`Ok`].
    ///
    /// Iff the allocation fails, the closure is not run.
    ///
    /// Iff the closure returns [`Err`], an allocator rewind is *attempted* and
    /// the `E` instance is moved out of the allocator to be consumed or dropped
    /// as normal.
    ///
    /// See [The `_with` Method Suffix](#initializer-functions-the-_with-method-suffix) for a
    /// discussion on the differences between the `_with` suffixed methods and
    /// those methods without it, their performance characteristics, and when
    /// you might or might not choose a `_with` suffixed method.
    ///
    /// For caveats specific to fallible initialization, see
    /// [The `_try_with` Method Suffix](#fallible-initialization-the-_try_with-method-suffix).
    ///
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    /// [`Ok`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    /// [`Err`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    ///
    /// ## Errors
    ///
    /// Errors with the [`Alloc`](`AllocOrInitError::Alloc`) variant iff
    /// reserving space for `Result<T, E>` fails.
    ///
    /// Iff the allocation succeeds but `f` fails, that error is forwarded by
    /// value inside the [`Init`](`AllocOrInitError::Init`) variant.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.try_alloc_try_with(|| Ok("hello"))?;
    /// assert_eq!(*x, "hello");
    /// # Result::<_, bumpalo::AllocOrInitError<()>>::Ok(())
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn try_alloc_try_with<F, T, E>(&self, f: F) -> Result<&mut T, AllocOrInitError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let rewind_footer = self.current_chunk_footer.get();
        let rewind_ptr = unsafe { rewind_footer.as_ref() }.ptr.get();
        let mut inner_result_ptr = NonNull::from(self.try_alloc_with(f)?);
        let inner_result_address = inner_result_ptr.as_ptr() as usize;
        match unsafe { inner_result_ptr.as_mut() } {
            Ok(t) => Ok(unsafe {
                //SAFETY:
                // The `&mut Result<T, E>` returned by `alloc_with` may be
                // lifetime-limited by `E`, but the derived `&mut T` still has
                // the same validity as in `alloc_with` since the error variant
                // is already ruled out here.

                // We could conditionally truncate the allocation here, but
                // since it grows backwards, it seems unlikely that we'd get
                // any more than the `Result`'s discriminant this way, if
                // anything at all.
                &mut *(t as *mut _)
            }),
            Err(e) => unsafe {
                // If this result was the last allocation in this arena, we can
                // reclaim its space. In fact, sometimes we can do even better
                // than simply calling `dealloc` on the result pointer: we can
                // reclaim any alignment padding we might have added (which
                // `dealloc` cannot do) if we didn't allocate a new chunk for
                // this result.
                if self.is_last_allocation(NonNull::new_unchecked(inner_result_address as *mut _)) {
                    let current_footer_p = self.current_chunk_footer.get();
                    let current_ptr = &current_footer_p.as_ref().ptr;
                    if current_footer_p == rewind_footer {
                        // It's still the same chunk, so reset the bump pointer
                        // to its original value upon entry to this method
                        // (reclaiming any alignment padding we may have
                        // added).
                        current_ptr.set(rewind_ptr);
                    } else {
                        // We allocated a new chunk for this result.
                        //
                        // We know the result is the only allocation in this
                        // chunk: Any additional allocations since the start of
                        // this method could only have happened when running
                        // the initializer function, which is called *after*
                        // reserving space for this result. Therefore, since we
                        // already determined via the check above that this
                        // result was the last allocation, there must not have
                        // been any other allocations, and this result is the
                        // only allocation in this chunk.
                        //
                        // Because this is the only allocation in this chunk,
                        // we can reset the chunk's bump finger to the start of
                        // the chunk.
                        current_ptr.set(current_footer_p.as_ref().data);
                    }
                }
                //SAFETY:
                // As we received `E` semantically by value from `f`, we can
                // just copy that value here as long as we avoid a double-drop
                // (which can't happen as any specific references to the `E`'s
                // data in `self` are destroyed when this function returns).
                //
                // The order between this and the deallocation doesn't matter
                // because `Self: !Sync`.
                Err(AllocOrInitError::Init(ptr::read(e as *const _)))
            },
        }
    }

    /// `Copy` a slice into this `Bump` and return an exclusive reference to
    /// the copy.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc_slice_copy(&[1, 2, 3]);
    /// assert_eq!(x, &[1, 2, 3]);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).cast::<T>();

        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), src.len());
            slice::from_raw_parts_mut(dst.as_ptr(), src.len())
        }
    }

    /// `Clone` a slice into this `Bump` and return an exclusive reference to
    /// the clone. Prefer [`alloc_slice_copy`](#method.alloc_slice_copy) if `T` is `Copy`.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    ///
    /// ```
    /// #[derive(Clone, Debug, Eq, PartialEq)]
    /// struct Sheep {
    ///     name: String,
    /// }
    ///
    /// let originals = [
    ///     Sheep { name: "Alice".into() },
    ///     Sheep { name: "Bob".into() },
    ///     Sheep { name: "Cathy".into() },
    /// ];
    ///
    /// let bump = bumpalo::Bump::new();
    /// let clones = bump.alloc_slice_clone(&originals);
    /// assert_eq!(originals, clones);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).cast::<T>();

        unsafe {
            for (i, val) in src.iter().cloned().enumerate() {
                ptr::write(dst.as_ptr().add(i), val);
            }

            slice::from_raw_parts_mut(dst.as_ptr(), src.len())
        }
    }

    /// `Copy` a string slice into this `Bump` and return an exclusive reference to it.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the string fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let hello = bump.alloc_str("hello world");
    /// assert_eq!("hello world", hello);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_str(&self, src: &str) -> &mut str {
        let buffer = self.alloc_slice_copy(src.as_bytes());
        unsafe {
            // This is OK, because it already came in as str, so it is guaranteed to be utf8
            str::from_utf8_unchecked_mut(buffer)
        }
    }

    /// Allocates a new slice of size `len` into this `Bump` and returns an
    /// exclusive reference to the copy.
    ///
    /// The elements of the slice are initialized using the supplied closure.
    /// The closure argument is the position in the slice.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc_slice_fill_with(5, |i| 5 * (i + 1));
    /// assert_eq!(x, &[5, 10, 15, 20, 25]);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_with<T, F>(&self, len: usize, mut f: F) -> &mut [T]
    where
        F: FnMut(usize) -> T,
    {
        let layout = Layout::array::<T>(len).unwrap_or_else(|_| oom());
        let dst = self.alloc_layout(layout).cast::<T>();

        unsafe {
            for i in 0..len {
                ptr::write(dst.as_ptr().add(i), f(i));
            }

            let result = slice::from_raw_parts_mut(dst.as_ptr(), len);
            debug_assert_eq!(Layout::for_value(result), layout);
            result
        }
    }

    /// Allocates a new slice of size `len` into this `Bump` and returns an
    /// exclusive reference to the copy.
    ///
    /// All elements of the slice are initialized to `value`.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc_slice_fill_copy(5, 42);
    /// assert_eq!(x, &[42, 42, 42, 42, 42]);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_copy<T: Copy>(&self, len: usize, value: T) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| value)
    }

    /// Allocates a new slice of size `len` slice into this `Bump` and return an
    /// exclusive reference to the copy.
    ///
    /// All elements of the slice are initialized to `value.clone()`.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let s: String = "Hello Bump!".to_string();
    /// let x: &[String] = bump.alloc_slice_fill_clone(2, &s);
    /// assert_eq!(x.len(), 2);
    /// assert_eq!(&x[0], &s);
    /// assert_eq!(&x[1], &s);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_clone<T: Clone>(&self, len: usize, value: &T) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| value.clone())
    }

    /// Allocates a new slice of size `len` slice into this `Bump` and return an
    /// exclusive reference to the copy.
    ///
    /// The elements are initialized using the supplied iterator.
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails, or if the supplied
    /// iterator returns fewer elements than it promised.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x: &[i32] = bump.alloc_slice_fill_iter([2, 3, 5].iter().cloned().map(|i| i * i));
    /// assert_eq!(x, [4, 9, 25]);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();
        self.alloc_slice_fill_with(iter.len(), |_| {
            iter.next().expect("Iterator supplied too few elements")
        })
    }

    /// Allocates a new slice of size `len` slice into this `Bump` and return an
    /// exclusive reference to the copy.
    ///
    /// All elements of the slice are initialized to [`T::default()`].
    ///
    /// [`T::default()`]: https://doc.rust-lang.org/std/default/trait.Default.html#tymethod.default
    ///
    /// ## Panics
    ///
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let x = bump.alloc_slice_fill_default::<u32>(5);
    /// assert_eq!(x, &[0, 0, 0, 0, 0]);
    /// ```
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_fill_default<T: Default>(&self, len: usize) -> &mut [T] {
        self.alloc_slice_fill_with(len, |_| T::default())
    }

    /// Allocate space for an object with the given `Layout`.
    ///
    /// The returned pointer points at uninitialized memory, and should be
    /// initialized with
    /// [`std::ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html).
    ///
    /// # Panics
    ///
    /// Panics if reserving space matching `layout` fails.
    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        self.try_alloc_layout(layout).unwrap_or_else(|_| oom())
    }

    /// Attempts to allocate space for an object with the given `Layout` or else returns
    /// an `Err`.
    ///
    /// The returned pointer points at uninitialized memory, and should be
    /// initialized with
    /// [`std::ptr::write`](https://doc.rust-lang.org/std/ptr/fn.write.html).
    ///
    /// # Errors
    ///
    /// Errors if reserving space matching `layout` fails.
    #[inline(always)]
    pub fn try_alloc_layout(&self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        if let Some(p) = self.try_alloc_layout_fast(layout) {
            Ok(p)
        } else {
            self.alloc_layout_slow(layout).ok_or(AllocErr)
        }
    }

    #[inline(always)]
    fn try_alloc_layout_fast(&self, layout: Layout) -> Option<NonNull<u8>> {
        // We don't need to check for ZSTs here since they will automatically
        // be handled properly: the pointer will be bumped by zero bytes,
        // modulo alignment. This keeps the fast path optimized for non-ZSTs,
        // which are much more common.
        unsafe {
            let footer = self.current_chunk_footer.get();
            let footer = footer.as_ref();
            let ptr = footer.ptr.get().as_ptr();
            let start = footer.data.as_ptr();
            debug_assert!(start <= ptr);
            debug_assert!(ptr as *const u8 <= footer as *const _ as *const u8);

            if (ptr as usize) < layout.size() {
                return None;
            }

            let ptr = ptr.wrapping_sub(layout.size());
            let rem = ptr as usize % layout.align();
            let aligned_ptr = ptr.wrapping_sub(rem);

            if aligned_ptr >= start {
                let aligned_ptr = NonNull::new_unchecked(aligned_ptr as *mut u8);
                footer.ptr.set(aligned_ptr);
                Some(aligned_ptr)
            } else {
                None
            }
        }
    }

    /// Gets the remaining capacity in the current chunk (in bytes).
    ///
    /// ## Example
    ///
    /// ```
    /// use bumpalo::Bump;
    ///
    /// let bump = Bump::with_capacity(100);
    ///
    /// let capacity = bump.chunk_capacity();
    /// assert!(capacity >= 100);
    /// ```
    pub fn chunk_capacity(&self) -> usize {
        let current_footer = self.current_chunk_footer.get();
        let current_footer = unsafe { current_footer.as_ref() };

        current_footer as *const _ as usize - current_footer.data.as_ptr() as usize
    }

    /// Slow path allocation for when we need to allocate a new chunk from the
    /// parent bump set because there isn't enough room in our current chunk.
    #[inline(never)]
    fn alloc_layout_slow(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            let size = layout.size();

            // Get a new chunk from the global allocator.
            let current_footer = self.current_chunk_footer.get();
            let current_layout = current_footer.as_ref().layout;

            // By default, we want our new chunk to be about twice as big
            // as the previous chunk. If the global allocator refuses it,
            // we try to divide it by half until it works or the requested
            // size is smaller than the default footer size.
            let min_new_chunk_size = layout.size().max(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
            let mut base_size = (current_layout.size() - FOOTER_SIZE)
                .checked_mul(2)?
                .max(min_new_chunk_size);
            let sizes = iter::from_fn(|| {
                if base_size >= min_new_chunk_size {
                    let size = base_size;
                    base_size = base_size / 2;
                    Some(size)
                } else {
                    None
                }
            });

            let new_footer = sizes
                .filter_map(|size| Bump::new_chunk(Some(size), layout, current_footer))
                .next()?;

            debug_assert_eq!(
                new_footer.as_ref().data.as_ptr() as usize % layout.align(),
                0
            );

            // Set the new chunk as our new current chunk.
            self.current_chunk_footer.set(new_footer);

            let new_footer = new_footer.as_ref();

            // Move the bump ptr finger down to allocate room for `val`. We know
            // this can't overflow because we successfully allocated a chunk of
            // at least the requested size.
            let mut ptr = new_footer.ptr.get().as_ptr().sub(size);
            // Round the pointer down to the requested alignment.
            ptr = ptr.sub(ptr as usize % layout.align());
            debug_assert!(
                ptr as *const _ <= new_footer,
                "{:p} <= {:p}",
                ptr,
                new_footer
            );
            let ptr = NonNull::new_unchecked(ptr as *mut u8);
            new_footer.ptr.set(ptr);

            // Return a pointer to the freshly allocated region in this chunk.
            Some(ptr)
        }
    }

    /// Returns an iterator over each chunk of allocated memory that
    /// this arena has bump allocated into.
    ///
    /// The chunks are returned ordered by allocation time, with the most
    /// recently allocated chunk being returned first, and the least recently
    /// allocated chunk being returned last.
    ///
    /// The values inside each chunk are also ordered by allocation time, with
    /// the most recent allocation being earlier in the slice, and the least
    /// recent allocation being towards the end of the slice.
    ///
    /// ## Safety
    ///
    /// Because this method takes `&mut self`, we know that the bump arena
    /// reference is unique and therefore there aren't any active references to
    /// any of the objects we've allocated in it either. This potential aliasing
    /// of exclusive references is one common footgun for unsafe code that we
    /// don't need to worry about here.
    ///
    /// However, there could be regions of uninitialized memory used as padding
    /// between allocations, which is why this iterator has items of type
    /// `[MaybeUninit<u8>]`, instead of simply `[u8]`.
    ///
    /// The only way to guarantee that there is no padding between allocations
    /// or within allocated objects is if all of these properties hold:
    ///
    /// 1. Every object allocated in this arena has the same alignment,
    ///    and that alignment is at most 16.
    /// 2. Every object's size is a multiple of its alignment.
    /// 3. None of the objects allocated in this arena contain any internal
    ///    padding.
    ///
    /// If you want to use this `iter_allocated_chunks` method, it is *your*
    /// responsibility to ensure that these properties hold before calling
    /// `MaybeUninit::assume_init` or otherwise reading the returned values.
    ///
    /// Finally, you must also ensure that any values allocated into the bump
    /// arena have not had their `Drop` implementations called on them,
    /// e.g. after dropping a [`bumpalo::boxed::Box<T>`][crate::boxed::Box].
    ///
    /// ## Example
    ///
    /// ```
    /// let mut bump = bumpalo::Bump::new();
    ///
    /// // Allocate a bunch of `i32`s in this bump arena, potentially causing
    /// // additional memory chunks to be reserved.
    /// for i in 0..10000 {
    ///     bump.alloc(i);
    /// }
    ///
    /// // Iterate over each chunk we've bump allocated into. This is safe
    /// // because we have only allocated `i32`s in this arena, which fulfills
    /// // the above requirements.
    /// for ch in bump.iter_allocated_chunks() {
    ///     println!("Used a chunk that is {} bytes long", ch.len());
    ///     println!("The first byte is {:?}", unsafe {
    ///         ch[0].assume_init()
    ///     });
    /// }
    ///
    /// // Within a chunk, allocations are ordered from most recent to least
    /// // recent. If we allocated 'a', then 'b', then 'c', when we iterate
    /// // through the chunk's data, we get them in the order 'c', then 'b',
    /// // then 'a'.
    ///
    /// bump.reset();
    /// bump.alloc(b'a');
    /// bump.alloc(b'b');
    /// bump.alloc(b'c');
    ///
    /// assert_eq!(bump.iter_allocated_chunks().count(), 1);
    /// let chunk = bump.iter_allocated_chunks().nth(0).unwrap();
    /// assert_eq!(chunk.len(), 3);
    ///
    /// // Safe because we've only allocated `u8`s in this arena, which
    /// // fulfills the above requirements.
    /// unsafe {
    ///     assert_eq!(chunk[0].assume_init(), b'c');
    ///     assert_eq!(chunk[1].assume_init(), b'b');
    ///     assert_eq!(chunk[2].assume_init(), b'a');
    /// }
    /// ```
    pub fn iter_allocated_chunks(&mut self) -> ChunkIter<'_> {
        // SAFE: Ensured by mutable borrow of `self`.
        let raw = unsafe { self.iter_allocated_chunks_raw() };
        ChunkIter {
            raw,
            bump: PhantomData,
        }
    }

    /// Returns an iterator over raw pointers to chunks of allocated memory that
    /// this arena has bump allocated into.
    ///
    /// This is an unsafe version of [`iter_allocated_chunks()`](Bump::iter_allocated_chunks),
    /// with the caller responsible for safe usage of the returned pointers as
    /// well as ensuring that the iterator is not invalidated by new
    /// allocations.
    ///
    /// ## Safety
    ///
    /// Allocations from this arena must not be performed while the returned
    /// iterator is alive. If reading the chunk data (or casting to a reference)
    /// the caller must ensure that there exist no mutable references to
    /// previously allocated data.
    ///
    /// In addition, all of the caveats when reading the chunk data from
    /// [`iter_allocated_chunks()`](Bump::iter_allocated_chunks) still apply.
    pub unsafe fn iter_allocated_chunks_raw(&self) -> ChunkRawIter<'_> {
        ChunkRawIter {
            footer: self.current_chunk_footer.get(),
            bump: PhantomData,
        }
    }

    /// Calculates the number of bytes currently allocated across all chunks in
    /// this bump arena.
    ///
    /// If you allocate types of different alignments or types with
    /// larger-than-typical alignment in the same arena, some padding
    /// bytes might get allocated in the bump arena. Note that those padding
    /// bytes will add to this method's resulting sum, so you cannot rely
    /// on it only counting the sum of the sizes of the things
    /// you've allocated in the arena.
    ///
    /// ## Example
    ///
    /// ```
    /// let bump = bumpalo::Bump::new();
    /// let _x = bump.alloc_slice_fill_default::<u32>(5);
    /// let bytes = bump.allocated_bytes();
    /// assert!(bytes >= core::mem::size_of::<u32>() * 5);
    /// ```
    pub fn allocated_bytes(&self) -> usize {
        let mut footer = self.current_chunk_footer.get();

        let mut bytes = 0;

        unsafe {
            while !footer.as_ref().is_empty() {
                let foot = footer.as_ref();

                let ptr = foot.ptr.get().as_ptr() as usize;
                debug_assert!(ptr <= foot as *const _ as usize);

                bytes += foot as *const _ as usize - ptr;

                footer = foot.prev.get();
            }
        }

        bytes
    }

    #[inline]
    unsafe fn is_last_allocation(&self, ptr: NonNull<u8>) -> bool {
        let footer = self.current_chunk_footer.get();
        let footer = footer.as_ref();
        footer.ptr.get() == ptr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        // If the pointer is the last allocation we made, we can reuse the bytes,
        // otherwise they are simply leaked -- at least until somebody calls reset().
        if self.is_last_allocation(ptr) {
            let ptr = NonNull::new_unchecked(ptr.as_ptr().add(layout.size()));
            self.current_chunk_footer.get().as_ref().ptr.set(ptr);
        }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<u8>, AllocErr> {
        let old_size = old_layout.size();
        let new_size = new_layout.size();
        let align_is_compatible = old_layout.align() >= new_layout.align();

        if !align_is_compatible {
            return Err(AllocErr);
        }

        // This is how much space we would *actually* reclaim while satisfying
        // the requested alignment.
        let delta = round_down_to(old_size - new_size, new_layout.align());

        if self.is_last_allocation(ptr)
                // Only reclaim the excess space (which requires a copy) if it
                // is worth it: we are actually going to recover "enough" space
                // and we can do a non-overlapping copy.
                && delta >= old_size / 2
        {
            let footer = self.current_chunk_footer.get();
            let footer = footer.as_ref();

            // NB: new_ptr is aligned, because ptr *has to* be aligned, and we
            // made sure delta is aligned.
            let new_ptr = NonNull::new_unchecked(footer.ptr.get().as_ptr().add(delta));
            footer.ptr.set(new_ptr);

            // NB: we know it is non-overlapping because of the size check
            // in the `if` condition.
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_size);

            return Ok(new_ptr);
        } else {
            return Ok(ptr);
        }
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<u8>, AllocErr> {
        let old_size = old_layout.size();
        let new_size = new_layout.size();
        let align_is_compatible = old_layout.align() >= new_layout.align();

        if align_is_compatible && self.is_last_allocation(ptr) {
            // Try to allocate the delta size within this same block so we can
            // reuse the currently allocated space.
            let delta = new_size - old_size;
            if let Some(p) =
                self.try_alloc_layout_fast(layout_from_size_align(delta, old_layout.align()))
            {
                ptr::copy(ptr.as_ptr(), p.as_ptr(), old_size);
                return Ok(p);
            }
        }

        // Fallback: do a fresh allocation and copy the existing data into it.
        let new_ptr = self.try_alloc_layout(new_layout)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size);
        Ok(new_ptr)
    }
}

/// An iterator over each chunk of allocated memory that
/// an arena has bump allocated into.
///
/// The chunks are returned ordered by allocation time, with the most recently
/// allocated chunk being returned first.
///
/// The values inside each chunk are also ordered by allocation time, with the most
/// recent allocation being earlier in the slice.
///
/// This struct is created by the [`iter_allocated_chunks`] method on
/// [`Bump`]. See that function for a safety description regarding reading from the returned items.
///
/// [`Bump`]: struct.Bump.html
/// [`iter_allocated_chunks`]: struct.Bump.html#method.iter_allocated_chunks
#[derive(Debug)]
pub struct ChunkIter<'a> {
    raw: ChunkRawIter<'a>,
    bump: PhantomData<&'a mut Bump>,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = &'a [mem::MaybeUninit<u8>];
    fn next(&mut self) -> Option<&'a [mem::MaybeUninit<u8>]> {
        unsafe {
            let (ptr, len) = self.raw.next()?;
            let slice = slice::from_raw_parts(ptr as *const mem::MaybeUninit<u8>, len);
            Some(slice)
        }
    }
}

impl<'a> iter::FusedIterator for ChunkIter<'a> {}

/// An iterator over raw pointers to chunks of allocated memory that this
/// arena has bump allocated into.
///
/// See [`ChunkIter`] for details regarding the returned chunks.
///
/// This struct is created by the [`iter_allocated_chunks_raw`] method on
/// [`Bump`]. See that function for a safety description regarding reading from
/// the returned items.
///
/// [`Bump`]: struct.Bump.html
/// [`iter_allocated_chunks_raw`]: struct.Bump.html#method.iter_allocated_chunks_raw
#[derive(Debug)]
pub struct ChunkRawIter<'a> {
    footer: NonNull<ChunkFooter>,
    bump: PhantomData<&'a Bump>,
}

impl Iterator for ChunkRawIter<'_> {
    type Item = (*mut u8, usize);
    fn next(&mut self) -> Option<(*mut u8, usize)> {
        unsafe {
            let foot = self.footer.as_ref();
            if foot.is_empty() {
                return None;
            }
            let (ptr, len) = foot.as_raw_parts();
            self.footer = foot.prev.get();
            Some((ptr as *mut u8, len))
        }
    }
}

impl iter::FusedIterator for ChunkRawIter<'_> {}

#[inline(never)]
#[cold]
fn oom() -> ! {
    panic!("out of memory")
}

unsafe impl<'a> alloc::Alloc for &'a Bump {
    #[inline(always)]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        self.try_alloc_layout(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        Bump::dealloc(self, ptr, layout)
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>, AllocErr> {
        let old_size = layout.size();

        if old_size == 0 {
            return self.try_alloc_layout(layout);
        }

        let new_layout = layout_from_size_align(new_size, layout.align());
        if new_size <= old_size {
            self.shrink(ptr, layout, new_layout)
        } else {
            self.grow(ptr, layout, new_layout)
        }
    }
}

#[cfg(feature = "allocator_api")]
unsafe impl<'a> Allocator for &'a Bump {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.try_alloc_layout(layout)
            .map(|p| NonNull::slice_from_raw_parts(p, layout.size()))
            .map_err(|_| AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        Bump::dealloc(self, ptr, layout)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        Bump::shrink(self, ptr, old_layout, new_layout)
            .map(|p| NonNull::slice_from_raw_parts(p, new_layout.size()))
            .map_err(|_| AllocError)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        Bump::grow(self, ptr, old_layout, new_layout)
            .map(|p| NonNull::slice_from_raw_parts(p, new_layout.size()))
            .map_err(|_| AllocError)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let mut ptr = self.grow(ptr, old_layout, new_layout)?;
        ptr.as_mut()[old_layout.size()..].fill(0);
        Ok(ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_footer_is_five_words() {
        assert_eq!(mem::size_of::<ChunkFooter>(), mem::size_of::<usize>() * 5);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_realloc() {
        use crate::alloc::Alloc;

        unsafe {
            const CAPACITY: usize = 1024 - OVERHEAD;
            let mut b = Bump::with_capacity(CAPACITY);

            // `realloc` doesn't shrink allocations that aren't "worth it".
            let layout = Layout::from_size_align(100, 1).unwrap();
            let p = b.alloc_layout(layout);
            let q = (&b).realloc(p, layout, 51).unwrap();
            assert_eq!(p, q);
            b.reset();

            // `realloc` will shrink allocations that are "worth it".
            let layout = Layout::from_size_align(100, 1).unwrap();
            let p = b.alloc_layout(layout);
            let q = (&b).realloc(p, layout, 50).unwrap();
            assert!(p != q);
            b.reset();

            // `realloc` will reuse the last allocation when growing.
            let layout = Layout::from_size_align(10, 1).unwrap();
            let p = b.alloc_layout(layout);
            let q = (&b).realloc(p, layout, 11).unwrap();
            assert_eq!(q.as_ptr() as usize, p.as_ptr() as usize - 1);
            b.reset();

            // `realloc` will allocate a new chunk when growing the last
            // allocation, if need be.
            let layout = Layout::from_size_align(1, 1).unwrap();
            let p = b.alloc_layout(layout);
            let q = (&b).realloc(p, layout, CAPACITY + 1).unwrap();
            assert!(q.as_ptr() as usize != p.as_ptr() as usize - CAPACITY);
            b = Bump::with_capacity(CAPACITY);

            // `realloc` will allocate and copy when reallocating anything that
            // wasn't the last allocation.
            let layout = Layout::from_size_align(1, 1).unwrap();
            let p = b.alloc_layout(layout);
            let _ = b.alloc_layout(layout);
            let q = (&b).realloc(p, layout, 2).unwrap();
            assert!(q.as_ptr() as usize != p.as_ptr() as usize - 1);
            b.reset();
        }
    }

    #[test]
    fn invalid_read() {
        use alloc::Alloc;

        let mut b = &Bump::new();

        unsafe {
            let l1 = Layout::from_size_align(12000, 4).unwrap();
            let p1 = Alloc::alloc(&mut b, l1).unwrap();

            let l2 = Layout::from_size_align(1000, 4).unwrap();
            Alloc::alloc(&mut b, l2).unwrap();

            let p1 = b.realloc(p1, l1, 24000).unwrap();
            let l3 = Layout::from_size_align(24000, 4).unwrap();
            b.realloc(p1, l3, 48000).unwrap();
        }
    }
}
