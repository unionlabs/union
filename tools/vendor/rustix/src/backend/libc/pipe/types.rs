#[cfg(linux_kernel)]
use core::marker::PhantomData;
#[cfg(not(any(apple, target_os = "wasi")))]
use {crate::backend::c, bitflags::bitflags};

#[cfg(not(any(apple, target_os = "wasi")))]
bitflags! {
    /// `O_*` constants for use with [`pipe_with`].
    ///
    /// [`pipe_with`]: crate::io::pipe_with
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct PipeFlags: u32 {
        /// `O_CLOEXEC`
        const CLOEXEC = bitcast!(c::O_CLOEXEC);
        /// `O_DIRECT`
        #[cfg(not(any(
            solarish,
            target_os = "haiku",
            target_os = "openbsd",
            target_os = "redox",
        )))]
        const DIRECT = bitcast!(c::O_DIRECT);
        /// `O_NONBLOCK`
        const NONBLOCK = bitcast!(c::O_NONBLOCK);
    }
}

#[cfg(linux_kernel)]
bitflags! {
    /// `SPLICE_F_*` constants for use with [`splice`], [`vmsplice`],
    /// and [`tee`].
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct SpliceFlags: c::c_uint {
        /// `SPLICE_F_MOVE`
        const MOVE = c::SPLICE_F_MOVE;
        /// `SPLICE_F_NONBLOCK`
        const NONBLOCK = c::SPLICE_F_NONBLOCK;
        /// `SPLICE_F_MORE`
        const MORE = c::SPLICE_F_MORE;
        /// `SPLICE_F_GIFT`
        const GIFT = c::SPLICE_F_GIFT;
    }
}

/// A buffer type used with `vmsplice`.
///
/// It is guaranteed to be ABI compatible with the iovec type on Unix platforms
/// and `WSABUF` on Windows. Unlike `IoSlice` and `IoSliceMut` it is
/// semantically like a raw pointer, and therefore can be shared or mutated as
/// needed.
#[cfg(linux_kernel)]
#[repr(transparent)]
pub struct IoSliceRaw<'a> {
    _buf: c::iovec,
    _lifetime: PhantomData<&'a ()>,
}

#[cfg(linux_kernel)]
impl<'a> IoSliceRaw<'a> {
    /// Creates a new `IoSlice` wrapping a byte slice.
    pub fn from_slice(buf: &'a [u8]) -> Self {
        IoSliceRaw {
            _buf: c::iovec {
                iov_base: buf.as_ptr() as *mut u8 as *mut c::c_void,
                iov_len: buf.len() as _,
            },
            _lifetime: PhantomData,
        }
    }

    /// Creates a new `IoSlice` wrapping a mutable byte slice.
    pub fn from_slice_mut(buf: &'a mut [u8]) -> Self {
        IoSliceRaw {
            _buf: c::iovec {
                iov_base: buf.as_mut_ptr() as *mut c::c_void,
                iov_len: buf.len() as _,
            },
            _lifetime: PhantomData,
        }
    }
}

#[cfg(not(any(apple, target_os = "wasi")))]
#[test]
fn test_types() {
    use core::mem::size_of;
    assert_eq!(size_of::<PipeFlags>(), size_of::<c::c_int>());

    #[cfg(linux_kernel)]
    assert_eq!(size_of::<SpliceFlags>(), size_of::<c::c_int>());
}
