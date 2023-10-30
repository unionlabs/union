// vim: tw=80
//! POSIX Asynchronous I/O
//!
//! The POSIX AIO interface is used for asynchronous I/O on files and disk-like
//! devices.  It supports [`read`](struct.AioCb.html#method.read),
//! [`write`](struct.AioCb.html#method.write), and
//! [`fsync`](struct.AioCb.html#method.fsync) operations.  Completion
//! notifications can optionally be delivered via
//! [signals](../signal/enum.SigevNotify.html#variant.SigevSignal), via the
//! [`aio_suspend`](fn.aio_suspend.html) function, or via polling.  Some
//! platforms support other completion
//! notifications, such as
//! [kevent](../signal/enum.SigevNotify.html#variant.SigevKevent).
//!
//! Multiple operations may be submitted in a batch with
//! [`lio_listio`](fn.lio_listio.html), though the standard does not guarantee
//! that they will be executed atomically.
//!
//! Outstanding operations may be cancelled with
//! [`cancel`](struct.AioCb.html#method.cancel) or
//! [`aio_cancel_all`](fn.aio_cancel_all.html), though the operating system may
//! not support this for all filesystems and devices.

use crate::Result;
use crate::errno::Errno;
use std::os::unix::io::RawFd;
use libc::{c_void, off_t, size_t};
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::ptr::{null, null_mut};
use crate::sys::signal::*;
use std::thread;
use crate::sys::time::TimeSpec;

libc_enum! {
    /// Mode for `AioCb::fsync`.  Controls whether only data or both data and
    /// metadata are synced.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum AioFsyncMode {
        /// do it like `fsync`
        O_SYNC,
        /// on supported operating systems only, do it like `fdatasync`
        #[cfg(any(target_os = "ios",
                  target_os = "linux",
                  target_os = "macos",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        O_DSYNC
    }
}

libc_enum! {
    /// When used with [`lio_listio`](fn.lio_listio.html), determines whether a
    /// given `aiocb` should be used for a read operation, a write operation, or
    /// ignored.  Has no effect for any other aio functions.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum LioOpcode {
        /// No operation
        LIO_NOP,
        /// Write data as if by a call to [`AioCb::write`]
        LIO_WRITE,
        /// Write data as if by a call to [`AioCb::read`]
        LIO_READ,
    }
}

libc_enum! {
    /// Mode for [`lio_listio`](fn.lio_listio.html)
    #[repr(i32)]
    pub enum LioMode {
        /// Requests that [`lio_listio`](fn.lio_listio.html) block until all
        /// requested operations have been completed
        LIO_WAIT,
        /// Requests that [`lio_listio`](fn.lio_listio.html) return immediately
        LIO_NOWAIT,
    }
}

/// Return values for [`AioCb::cancel`](struct.AioCb.html#method.cancel) and
/// [`aio_cancel_all`](fn.aio_cancel_all.html)
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AioCancelStat {
    /// All outstanding requests were canceled
    AioCanceled = libc::AIO_CANCELED,
    /// Some requests were not canceled.  Their status should be checked with
    /// `AioCb::error`
    AioNotCanceled = libc::AIO_NOTCANCELED,
    /// All of the requests have already finished
    AioAllDone = libc::AIO_ALLDONE,
}

/// Newtype that adds Send and Sync to libc::aiocb, which contains raw pointers
#[repr(transparent)]
struct LibcAiocb(libc::aiocb);

unsafe impl Send for LibcAiocb {}
unsafe impl Sync for LibcAiocb {}

/// AIO Control Block.
///
/// The basic structure used by all aio functions.  Each `AioCb` represents one
/// I/O request.
pub struct AioCb<'a> {
    aiocb: LibcAiocb,
    /// Tracks whether the buffer pointed to by `libc::aiocb.aio_buf` is mutable
    mutable: bool,
    /// Could this `AioCb` potentially have any in-kernel state?
    in_progress: bool,
    _buffer: std::marker::PhantomData<&'a [u8]>,
    _pin: std::marker::PhantomPinned
}

impl<'a> AioCb<'a> {
    /// Returns the underlying file descriptor associated with the `AioCb`
    pub fn fd(&self) -> RawFd {
        self.aiocb.0.aio_fildes
    }

    /// Constructs a new `AioCb` with no associated buffer.
    ///
    /// The resulting `AioCb` structure is suitable for use with `AioCb::fsync`.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`.
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                    completion.
    ///
    /// # Examples
    ///
    /// Create an `AioCb` from a raw file descriptor and use it for an
    /// [`fsync`](#method.fsync) operation.
    ///
    /// ```
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify::SigevNone;
    /// # use std::{thread, time};
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// let f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_fd( f.as_raw_fd(), 0, SigevNone);
    /// aiocb.fsync(AioFsyncMode::O_SYNC).expect("aio_fsync failed early");
    /// while (aiocb.error() == Err(Errno::EINPROGRESS)) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// aiocb.aio_return().expect("aio_fsync failed late");
    /// ```
    pub fn from_fd(fd: RawFd, prio: libc::c_int,
                    sigev_notify: SigevNotify) -> Pin<Box<AioCb<'a>>> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.0.aio_offset = 0;
        a.0.aio_nbytes = 0;
        a.0.aio_buf = null_mut();

        Box::pin(AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            _buffer: PhantomData,
            _pin: std::marker::PhantomPinned
        })
    }

    // Private helper
    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    fn from_mut_slice_unpinned(fd: RawFd, offs: off_t, buf: &'a mut [u8],
                          prio: libc::c_int, sigev_notify: SigevNotify,
                          opcode: LioOpcode) -> AioCb<'a>
    {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.0.aio_offset = offs;
        a.0.aio_nbytes = buf.len() as size_t;
        a.0.aio_buf = buf.as_ptr() as *mut c_void;
        a.0.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: true,
            in_progress: false,
            _buffer: PhantomData,
            _pin: std::marker::PhantomPinned
        }
    }

    /// Constructs a new `AioCb` from a mutable slice.
    ///
    /// The resulting `AioCb` will be suitable for both read and write
    /// operations, but only if the borrow checker can guarantee that the slice
    /// will outlive the `AioCb`.  That will usually be the case if the `AioCb`
    /// is stack-allocated.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          A memory buffer
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Examples
    ///
    /// Create an `AioCb` from a mutable slice and read into it.
    ///
    /// ```
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::io::Write;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// const INITIAL: &[u8] = b"abcdef123456";
    /// const LEN: usize = 4;
    /// let mut rbuf = vec![0; LEN];
    /// let mut f = tempfile().unwrap();
    /// f.write_all(INITIAL).unwrap();
    /// {
    ///     let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
    ///         2,   //offset
    ///         &mut rbuf,
    ///         0,   //priority
    ///         SigevNotify::SigevNone,
    ///         LioOpcode::LIO_NOP);
    ///     aiocb.read().unwrap();
    ///     while (aiocb.error() == Err(Errno::EINPROGRESS)) {
    ///         thread::sleep(time::Duration::from_millis(10));
    ///     }
    ///     assert_eq!(aiocb.aio_return().unwrap() as usize, LEN);
    /// }
    /// assert_eq!(rbuf, b"cdef");
    /// ```
    pub fn from_mut_slice(fd: RawFd, offs: off_t, buf: &'a mut [u8],
                          prio: libc::c_int, sigev_notify: SigevNotify,
                          opcode: LioOpcode) -> Pin<Box<AioCb<'a>>> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.0.aio_offset = offs;
        a.0.aio_nbytes = buf.len() as size_t;
        a.0.aio_buf = buf.as_ptr() as *mut c_void;
        a.0.aio_lio_opcode = opcode as libc::c_int;

        Box::pin(AioCb {
            aiocb: a,
            mutable: true,
            in_progress: false,
            _buffer: PhantomData,
            _pin: std::marker::PhantomPinned
        })
    }

    /// Constructs a new `AioCb` from a mutable raw pointer
    ///
    /// Unlike `from_mut_slice`, this method returns a structure suitable for
    /// placement on the heap.  It may be used for both reads and writes.  Due
    /// to its unsafety, this method is not recommended.  It is most useful when
    /// heap allocation is required.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          Pointer to the memory buffer
    /// * `len`:          Length of the buffer pointed to by `buf`
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Safety
    ///
    /// The caller must ensure that the storage pointed to by `buf` outlives the
    /// `AioCb`.  The lifetime checker can't help here.
    pub unsafe fn from_mut_ptr(fd: RawFd, offs: off_t,
                           buf: *mut c_void, len: usize,
                           prio: libc::c_int, sigev_notify: SigevNotify,
                           opcode: LioOpcode) -> Pin<Box<AioCb<'a>>> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.0.aio_offset = offs;
        a.0.aio_nbytes = len;
        a.0.aio_buf = buf;
        a.0.aio_lio_opcode = opcode as libc::c_int;

        Box::pin(AioCb {
            aiocb: a,
            mutable: true,
            in_progress: false,
            _buffer: PhantomData,
            _pin: std::marker::PhantomPinned,
        })
    }

    /// Constructs a new `AioCb` from a raw pointer.
    ///
    /// Unlike `from_slice`, this method returns a structure suitable for
    /// placement on the heap.  Due to its unsafety, this method is not
    /// recommended.  It is most useful when heap allocation is required.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          Pointer to the memory buffer
    /// * `len`:          Length of the buffer pointed to by `buf`
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Safety
    ///
    /// The caller must ensure that the storage pointed to by `buf` outlives the
    /// `AioCb`.  The lifetime checker can't help here.
    pub unsafe fn from_ptr(fd: RawFd, offs: off_t,
                           buf: *const c_void, len: usize,
                           prio: libc::c_int, sigev_notify: SigevNotify,
                           opcode: LioOpcode) -> Pin<Box<AioCb<'a>>> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.0.aio_offset = offs;
        a.0.aio_nbytes = len;
        // casting a const ptr to a mutable ptr here is ok, because we set the
        // AioCb's mutable field to false
        a.0.aio_buf = buf as *mut c_void;
        a.0.aio_lio_opcode = opcode as libc::c_int;

        Box::pin(AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            _buffer: PhantomData,
            _pin: std::marker::PhantomPinned
        })
    }

    // Private helper
    fn from_slice_unpinned(fd: RawFd, offs: off_t, buf: &'a [u8],
                           prio: libc::c_int, sigev_notify: SigevNotify,
                           opcode: LioOpcode) -> AioCb
    {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.0.aio_offset = offs;
        a.0.aio_nbytes = buf.len() as size_t;
        // casting an immutable buffer to a mutable pointer looks unsafe,
        // but technically its only unsafe to dereference it, not to create
        // it.
        a.0.aio_buf = buf.as_ptr() as *mut c_void;
        assert!(opcode != LioOpcode::LIO_READ, "Can't read into an immutable buffer");
        a.0.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            _buffer: PhantomData,
            _pin: std::marker::PhantomPinned
        }
    }

    /// Like [`AioCb::from_mut_slice`], but works on constant slices rather than
    /// mutable slices.
    ///
    /// An `AioCb` created this way cannot be used with `read`, and its
    /// `LioOpcode` cannot be set to `LIO_READ`.  This method is useful when
    /// writing a const buffer with `AioCb::write`, since `from_mut_slice` can't
    /// work with const buffers.
    ///
    /// # Examples
    ///
    /// Construct an `AioCb` from a slice and use it for writing.
    ///
    /// ```
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
    ///     2,   //offset
    ///     WBUF,
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// while (aiocb.error() == Err(Errno::EINPROGRESS)) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
    /// ```
    // Note: another solution to the problem of writing const buffers would be
    // to genericize AioCb for both &mut [u8] and &[u8] buffers.  AioCb::read
    // could take the former and AioCb::write could take the latter.  However,
    // then lio_listio wouldn't work, because that function needs a slice of
    // AioCb, and they must all be of the same type.
    pub fn from_slice(fd: RawFd, offs: off_t, buf: &'a [u8],
                      prio: libc::c_int, sigev_notify: SigevNotify,
                      opcode: LioOpcode) -> Pin<Box<AioCb>>
    {
        Box::pin(AioCb::from_slice_unpinned(fd, offs, buf, prio, sigev_notify,
                                            opcode))
    }

    fn common_init(fd: RawFd, prio: libc::c_int,
                   sigev_notify: SigevNotify) -> LibcAiocb {
        // Use mem::zeroed instead of explicitly zeroing each field, because the
        // number and name of reserved fields is OS-dependent.  On some OSes,
        // some reserved fields are used the kernel for state, and must be
        // explicitly zeroed when allocated.
        let mut a = unsafe { mem::zeroed::<libc::aiocb>()};
        a.aio_fildes = fd;
        a.aio_reqprio = prio;
        a.aio_sigevent = SigEvent::new(sigev_notify).sigevent();
        LibcAiocb(a)
    }

    /// Update the notification settings for an existing `aiocb`
    pub fn set_sigev_notify(self: &mut Pin<Box<Self>>,
                            sigev_notify: SigevNotify)
    {
        // Safe because we don't move any of the data
        let selfp = unsafe {
            self.as_mut().get_unchecked_mut()
        };
        selfp.aiocb.0.aio_sigevent = SigEvent::new(sigev_notify).sigevent();
    }

    /// Cancels an outstanding AIO request.
    ///
    /// The operating system is not required to implement cancellation for all
    /// file and device types.  Even if it does, there is no guarantee that the
    /// operation has not already completed.  So the caller must check the
    /// result and handle operations that were not canceled or that have already
    /// completed.
    ///
    /// # Examples
    ///
    /// Cancel an outstanding aio operation.  Note that we must still call
    /// `aio_return` to free resources, even though we don't care about the
    /// result.
    ///
    /// ```
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::io::Write;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// let wbuf = b"CDEF";
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
    ///     2,   //offset
    ///     &wbuf[..],
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// let cs = aiocb.cancel().unwrap();
    /// if cs == AioCancelStat::AioNotCanceled {
    ///     while (aiocb.error() == Err(Errno::EINPROGRESS)) {
    ///         thread::sleep(time::Duration::from_millis(10));
    ///     }
    /// }
    /// // Must call `aio_return`, but ignore the result
    /// let _ = aiocb.aio_return();
    /// ```
    ///
    /// # References
    ///
    /// [aio_cancel](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_cancel.html)
    pub fn cancel(self: &mut Pin<Box<Self>>) -> Result<AioCancelStat> {
        let r = unsafe {
            let selfp = self.as_mut().get_unchecked_mut();
            libc::aio_cancel(selfp.aiocb.0.aio_fildes, &mut selfp.aiocb.0)
        };
        match r {
            libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
            libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
            libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
            -1 => Err(Errno::last()),
            _ => panic!("unknown aio_cancel return value")
        }
    }

    fn error_unpinned(&mut self) -> Result<()> {
        let r = unsafe {
            libc::aio_error(&mut self.aiocb.0 as *mut libc::aiocb)
        };
        match r {
            0 => Ok(()),
            num if num > 0 => Err(Errno::from_i32(num)),
            -1 => Err(Errno::last()),
            num => panic!("unknown aio_error return value {:?}", num)
        }
    }

    /// Retrieve error status of an asynchronous operation.
    ///
    /// If the request has not yet completed, returns `EINPROGRESS`.  Otherwise,
    /// returns `Ok` or any other error.
    ///
    /// # Examples
    ///
    /// Issue an aio operation and use `error` to poll for completion.  Polling
    /// is an alternative to `aio_suspend`, used by most of the other examples.
    ///
    /// ```
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
    ///     2,   //offset
    ///     WBUF,
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// while (aiocb.error() == Err(Errno::EINPROGRESS)) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
    /// ```
    ///
    /// # References
    ///
    /// [aio_error](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_error.html)
    pub fn error(self: &mut Pin<Box<Self>>) -> Result<()> {
        // Safe because error_unpinned doesn't move the data
        let selfp = unsafe {
            self.as_mut().get_unchecked_mut()
        };
        selfp.error_unpinned()
    }

    /// An asynchronous version of `fsync(2)`.
    ///
    /// # References
    ///
    /// [aio_fsync](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_fsync.html)
    pub fn fsync(self: &mut Pin<Box<Self>>, mode: AioFsyncMode) -> Result<()> {
        // Safe because we don't move the libc::aiocb
        unsafe {
            let selfp = self.as_mut().get_unchecked_mut();
            Errno::result({
                let p: *mut libc::aiocb = &mut selfp.aiocb.0;
                libc::aio_fsync(mode as libc::c_int, p)
            }).map(|_| {
                selfp.in_progress = true;
            })
        }
    }

    /// Returns the `aiocb`'s `LioOpcode` field
    ///
    /// If the value cannot be represented as an `LioOpcode`, returns `None`
    /// instead.
    pub fn lio_opcode(&self) -> Option<LioOpcode> {
        match self.aiocb.0.aio_lio_opcode {
            libc::LIO_READ => Some(LioOpcode::LIO_READ),
            libc::LIO_WRITE => Some(LioOpcode::LIO_WRITE),
            libc::LIO_NOP => Some(LioOpcode::LIO_NOP),
            _ => None
        }
    }

    /// Returns the requested length of the aio operation in bytes
    ///
    /// This method returns the *requested* length of the operation.  To get the
    /// number of bytes actually read or written by a completed operation, use
    /// `aio_return` instead.
    pub fn nbytes(&self) -> usize {
        self.aiocb.0.aio_nbytes
    }

    /// Returns the file offset stored in the `AioCb`
    pub fn offset(&self) -> off_t {
        self.aiocb.0.aio_offset
    }

    /// Returns the priority of the `AioCb`
    pub fn priority(&self) -> libc::c_int {
        self.aiocb.0.aio_reqprio
    }

    /// Asynchronously reads from a file descriptor into a buffer
    ///
    /// # References
    ///
    /// [aio_read](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_read.html)
    pub fn read(self: &mut Pin<Box<Self>>) -> Result<()> {
        assert!(self.mutable, "Can't read into an immutable buffer");
        // Safe because we don't move anything
        let selfp = unsafe {
            self.as_mut().get_unchecked_mut()
        };
        Errno::result({
            let p: *mut libc::aiocb = &mut selfp.aiocb.0;
            unsafe { libc::aio_read(p) }
        }).map(|_| {
            selfp.in_progress = true;
        })
    }

    /// Returns the `SigEvent` stored in the `AioCb`
    pub fn sigevent(&self) -> SigEvent {
        SigEvent::from(&self.aiocb.0.aio_sigevent)
    }

    fn aio_return_unpinned(&mut self) -> Result<isize> {
        unsafe {
            let p: *mut libc::aiocb = &mut self.aiocb.0;
            self.in_progress = false;
            Errno::result(libc::aio_return(p))
        }
    }

    /// Retrieve return status of an asynchronous operation.
    ///
    /// Should only be called once for each `AioCb`, after `AioCb::error`
    /// indicates that it has completed.  The result is the same as for the
    /// synchronous `read(2)`, `write(2)`, of `fsync(2)` functions.
    ///
    /// # References
    ///
    /// [aio_return](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_return.html)
    // Note: this should be just `return`, but that's a reserved word
    pub fn aio_return(self: &mut Pin<Box<Self>>) -> Result<isize> {
        // Safe because aio_return_unpinned does not move the data
        let selfp = unsafe {
            self.as_mut().get_unchecked_mut()
        };
        selfp.aio_return_unpinned()
    }

    /// Asynchronously writes from a buffer to a file descriptor
    ///
    /// # References
    ///
    /// [aio_write](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_write.html)
    pub fn write(self: &mut Pin<Box<Self>>) -> Result<()> {
        // Safe because we don't move anything
        let selfp = unsafe {
            self.as_mut().get_unchecked_mut()
        };
        Errno::result({
            let p: *mut libc::aiocb = &mut selfp.aiocb.0;
            unsafe{ libc::aio_write(p) }
        }).map(|_| {
            selfp.in_progress = true;
        })
    }
}

/// Cancels outstanding AIO requests for a given file descriptor.
///
/// # Examples
///
/// Issue an aio operation, then cancel all outstanding operations on that file
/// descriptor.
///
/// ```
/// # use nix::errno::Errno;
/// # use nix::Error;
/// # use nix::sys::aio::*;
/// # use nix::sys::signal::SigevNotify;
/// # use std::{thread, time};
/// # use std::io::Write;
/// # use std::os::unix::io::AsRawFd;
/// # use tempfile::tempfile;
/// let wbuf = b"CDEF";
/// let mut f = tempfile().unwrap();
/// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
///     2,   //offset
///     &wbuf[..],
///     0,   //priority
///     SigevNotify::SigevNone,
///     LioOpcode::LIO_NOP);
/// aiocb.write().unwrap();
/// let cs = aio_cancel_all(f.as_raw_fd()).unwrap();
/// if cs == AioCancelStat::AioNotCanceled {
///     while (aiocb.error() == Err(Errno::EINPROGRESS)) {
///         thread::sleep(time::Duration::from_millis(10));
///     }
/// }
/// // Must call `aio_return`, but ignore the result
/// let _ = aiocb.aio_return();
/// ```
///
/// # References
///
/// [`aio_cancel`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_cancel.html)
pub fn aio_cancel_all(fd: RawFd) -> Result<AioCancelStat> {
    match unsafe { libc::aio_cancel(fd, null_mut()) } {
        libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
        libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
        libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
        -1 => Err(Errno::last()),
        _ => panic!("unknown aio_cancel return value")
    }
}

/// Suspends the calling process until at least one of the specified `AioCb`s
/// has completed, a signal is delivered, or the timeout has passed.
///
/// If `timeout` is `None`, `aio_suspend` will block indefinitely.
///
/// # Examples
///
/// Use `aio_suspend` to block until an aio operation completes.
///
/// ```
/// # use nix::sys::aio::*;
/// # use nix::sys::signal::SigevNotify;
/// # use std::os::unix::io::AsRawFd;
/// # use tempfile::tempfile;
/// const WBUF: &[u8] = b"abcdef123456";
/// let mut f = tempfile().unwrap();
/// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
///     2,   //offset
///     WBUF,
///     0,   //priority
///     SigevNotify::SigevNone,
///     LioOpcode::LIO_NOP);
/// aiocb.write().unwrap();
/// aio_suspend(&[aiocb.as_ref()], None).expect("aio_suspend failed");
/// assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
/// ```
/// # References
///
/// [`aio_suspend`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_suspend.html)
pub fn aio_suspend(list: &[Pin<&AioCb>], timeout: Option<TimeSpec>) -> Result<()> {
    let plist = list as *const [Pin<&AioCb>] as *const [*const libc::aiocb];
    let p = plist as *const *const libc::aiocb;
    let timep = match timeout {
        None    => null::<libc::timespec>(),
        Some(x) => x.as_ref() as *const libc::timespec
    };
    Errno::result(unsafe {
        libc::aio_suspend(p, list.len() as i32, timep)
    }).map(drop)
}

impl<'a> Debug for AioCb<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("AioCb")
            .field("aiocb", &self.aiocb.0)
            .field("mutable", &self.mutable)
            .field("in_progress", &self.in_progress)
            .finish()
    }
}

impl<'a> Drop for AioCb<'a> {
    /// If the `AioCb` has no remaining state in the kernel, just drop it.
    /// Otherwise, dropping constitutes a resource leak, which is an error
    fn drop(&mut self) {
        assert!(thread::panicking() || !self.in_progress,
                "Dropped an in-progress AioCb");
    }
}

/// LIO Control Block.
///
/// The basic structure used to issue multiple AIO operations simultaneously.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub struct LioCb<'a> {
    /// A collection of [`AioCb`]s.  All of these will be issued simultaneously
    /// by the [`listio`] method.
    ///
    /// [`AioCb`]: struct.AioCb.html
    /// [`listio`]: #method.listio
    // Their locations in memory must be fixed once they are passed to the
    // kernel.  So this field must be non-public so the user can't swap.
    aiocbs: Box<[AioCb<'a>]>,

    /// The actual list passed to `libc::lio_listio`.
    ///
    /// It must live for as long as any of the operations are still being
    /// processesed, because the aio subsystem uses its address as a unique
    /// identifier.
    list: Vec<*mut libc::aiocb>,

    /// A partial set of results.  This field will get populated by
    /// `listio_resubmit` when an `LioCb` is resubmitted after an error
    results: Vec<Option<Result<isize>>>
}

/// LioCb can't automatically impl Send and Sync just because of the raw
/// pointers in list.  But that's stupid.  There's no reason that raw pointers
/// should automatically be non-Send
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
unsafe impl<'a> Send for LioCb<'a> {}
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
unsafe impl<'a> Sync for LioCb<'a> {}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
impl<'a> LioCb<'a> {
    /// Are no [`AioCb`]s contained?
    pub fn is_empty(&self) -> bool {
        self.aiocbs.is_empty()
    }

    /// Return the number of individual [`AioCb`]s contained.
    pub fn len(&self) -> usize {
        self.aiocbs.len()
    }

    /// Submits multiple asynchronous I/O requests with a single system call.
    ///
    /// They are not guaranteed to complete atomically, and the order in which
    /// the requests are carried out is not specified.  Reads, writes, and
    /// fsyncs may be freely mixed.
    ///
    /// This function is useful for reducing the context-switch overhead of
    /// submitting many AIO operations.  It can also be used with
    /// `LioMode::LIO_WAIT` to block on the result of several independent
    /// operations.  Used that way, it is often useful in programs that
    /// otherwise make little use of AIO.
    ///
    /// # Examples
    ///
    /// Use `listio` to submit an aio operation and wait for its completion.  In
    /// this case, there is no need to use [`aio_suspend`] to wait or
    /// [`AioCb::error`] to poll.
    ///
    /// ```
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut liocb = LioCbBuilder::with_capacity(1)
    ///     .emplace_slice(
    ///         f.as_raw_fd(),
    ///         2,   //offset
    ///         WBUF,
    ///         0,   //priority
    ///         SigevNotify::SigevNone,
    ///         LioOpcode::LIO_WRITE
    ///     ).finish();
    /// liocb.listio(LioMode::LIO_WAIT,
    ///              SigevNotify::SigevNone).unwrap();
    /// assert_eq!(liocb.aio_return(0).unwrap() as usize, WBUF.len());
    /// ```
    ///
    /// # References
    ///
    /// [`lio_listio`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/lio_listio.html)
    ///
    /// [`aio_suspend`]: fn.aio_suspend.html
    /// [`AioCb::error`]: struct.AioCb.html#method.error
    pub fn listio(&mut self, mode: LioMode,
                  sigev_notify: SigevNotify) -> Result<()> {
        let sigev = SigEvent::new(sigev_notify);
        let sigevp = &mut sigev.sigevent() as *mut libc::sigevent;
        self.list.clear();
        for a in &mut self.aiocbs.iter_mut() {
            a.in_progress = true;
            self.list.push(a as *mut AioCb<'a>
                             as *mut libc::aiocb);
        }
        let p = self.list.as_ptr();
        Errno::result(unsafe {
            libc::lio_listio(mode as i32, p, self.list.len() as i32, sigevp)
        }).map(drop)
    }

    /// Resubmits any incomplete operations with [`lio_listio`].
    ///
    /// Sometimes, due to system resource limitations, an `lio_listio` call will
    /// return `EIO`, or `EAGAIN`.  Or, if a signal is received, it may return
    /// `EINTR`.  In any of these cases, only a subset of its constituent
    /// operations will actually have been initiated.  `listio_resubmit` will
    /// resubmit any operations that are still uninitiated.
    ///
    /// After calling `listio_resubmit`, results should be collected by
    /// [`LioCb::aio_return`].
    ///
    /// # Examples
    /// ```no_run
    /// # use nix::Error;
    /// # use nix::errno::Errno;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::os::unix::io::AsRawFd;
    /// # use std::{thread, time};
    /// # use tempfile::tempfile;
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut liocb = LioCbBuilder::with_capacity(1)
    ///     .emplace_slice(
    ///         f.as_raw_fd(),
    ///         2,   //offset
    ///         WBUF,
    ///         0,   //priority
    ///         SigevNotify::SigevNone,
    ///         LioOpcode::LIO_WRITE
    ///     ).finish();
    /// let mut err = liocb.listio(LioMode::LIO_WAIT, SigevNotify::SigevNone);
    /// while err == Err(Errno::EIO) ||
    ///       err == Err(Errno::EAGAIN) {
    ///     thread::sleep(time::Duration::from_millis(10));
    ///     err = liocb.listio_resubmit(LioMode::LIO_WAIT, SigevNotify::SigevNone);
    /// }
    /// assert_eq!(liocb.aio_return(0).unwrap() as usize, WBUF.len());
    /// ```
    ///
    /// # References
    ///
    /// [`lio_listio`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/lio_listio.html)
    ///
    /// [`lio_listio`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/lio_listio.html
    /// [`LioCb::aio_return`]: struct.LioCb.html#method.aio_return
    // Note: the addresses of any EINPROGRESS or EOK aiocbs _must_ not be
    // changed by this method, because the kernel relies on their addresses
    // being stable.
    // Note: aiocbs that are Ok(()) must be finalized by aio_return, or else the
    // sigev_notify will immediately refire.
    pub fn listio_resubmit(&mut self, mode:LioMode,
                           sigev_notify: SigevNotify) -> Result<()> {
        let sigev = SigEvent::new(sigev_notify);
        let sigevp = &mut sigev.sigevent() as *mut libc::sigevent;
        self.list.clear();

        while self.results.len() < self.aiocbs.len() {
            self.results.push(None);
        }

        for (i, a) in self.aiocbs.iter_mut().enumerate() {
            if self.results[i].is_some() {
                // Already collected final status for this operation
                continue;
            }
            match a.error_unpinned() {
                Ok(()) => {
                    // aiocb is complete; collect its status and don't resubmit
                    self.results[i] = Some(a.aio_return_unpinned());
                },
                Err(Errno::EAGAIN) => {
                    self.list.push(a as *mut AioCb<'a> as *mut libc::aiocb);
                },
                Err(Errno::EINPROGRESS) => {
                    // aiocb is was successfully queued; no need to do anything
                },
                Err(Errno::EINVAL) => panic!(
                    "AioCb was never submitted, or already finalized"),
                _ => unreachable!()
            }
        }
        let p = self.list.as_ptr();
        Errno::result(unsafe {
            libc::lio_listio(mode as i32, p, self.list.len() as i32, sigevp)
        }).map(drop)
    }

    /// Collect final status for an individual `AioCb` submitted as part of an
    /// `LioCb`.
    ///
    /// This is just like [`AioCb::aio_return`], except it takes into account
    /// operations that were restarted by [`LioCb::listio_resubmit`]
    ///
    /// [`AioCb::aio_return`]: struct.AioCb.html#method.aio_return
    /// [`LioCb::listio_resubmit`]: #method.listio_resubmit
    pub fn aio_return(&mut self, i: usize) -> Result<isize> {
        if i >= self.results.len() || self.results[i].is_none() {
            self.aiocbs[i].aio_return_unpinned()
        } else {
            self.results[i].unwrap()
        }
    }

    /// Retrieve error status of an individual `AioCb` submitted as part of an
    /// `LioCb`.
    ///
    /// This is just like [`AioCb::error`], except it takes into account
    /// operations that were restarted by [`LioCb::listio_resubmit`]
    ///
    /// [`AioCb::error`]: struct.AioCb.html#method.error
    /// [`LioCb::listio_resubmit`]: #method.listio_resubmit
    pub fn error(&mut self, i: usize) -> Result<()> {
        if i >= self.results.len() || self.results[i].is_none() {
            self.aiocbs[i].error_unpinned()
        } else {
            Ok(())
        }
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
impl<'a> Debug for LioCb<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LioCb")
            .field("aiocbs", &self.aiocbs)
            .finish()
    }
}

/// Used to construct `LioCb`
// This must be a separate class from LioCb due to pinning constraints.  LioCb
// must use a boxed slice of AioCbs so they will have stable storage, but
// LioCbBuilder must use a Vec to make construction possible when the final size
// is unknown.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[derive(Debug)]
pub struct LioCbBuilder<'a> {
    /// A collection of [`AioCb`]s.
    ///
    /// [`AioCb`]: struct.AioCb.html
    pub aiocbs: Vec<AioCb<'a>>,
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
impl<'a> LioCbBuilder<'a> {
    /// Initialize an empty `LioCb`
    pub fn with_capacity(capacity: usize) -> LioCbBuilder<'a> {
        LioCbBuilder {
            aiocbs: Vec::with_capacity(capacity),
        }
    }

    /// Add a new operation on an immutable slice to the [`LioCb`] under
    /// construction.
    ///
    /// Arguments are the same as for [`AioCb::from_slice`]
    ///
    /// [`LioCb`]: struct.LioCb.html
    /// [`AioCb::from_slice`]: struct.AioCb.html#method.from_slice
    pub fn emplace_slice(mut self, fd: RawFd, offs: off_t, buf: &'a [u8],
                         prio: libc::c_int, sigev_notify: SigevNotify,
                         opcode: LioOpcode) -> Self
    {
        self.aiocbs.push(AioCb::from_slice_unpinned(fd, offs, buf, prio,
                                                    sigev_notify, opcode));
        self
    }

    /// Add a new operation on a mutable slice to the [`LioCb`] under
    /// construction.
    ///
    /// Arguments are the same as for [`AioCb::from_mut_slice`]
    ///
    /// [`LioCb`]: struct.LioCb.html
    /// [`AioCb::from_mut_slice`]: struct.AioCb.html#method.from_mut_slice
    pub fn emplace_mut_slice(mut self, fd: RawFd, offs: off_t,
                             buf: &'a mut [u8], prio: libc::c_int,
                             sigev_notify: SigevNotify, opcode: LioOpcode)
        -> Self
    {
        self.aiocbs.push(AioCb::from_mut_slice_unpinned(fd, offs, buf, prio,
                                                        sigev_notify, opcode));
        self
    }

    /// Finalize this [`LioCb`].
    ///
    /// Afterwards it will be possible to issue the operations with
    /// [`LioCb::listio`].  Conversely, it will no longer be possible to add new
    /// operations with [`LioCbBuilder::emplace_slice`] or
    /// [`LioCbBuilder::emplace_mut_slice`].
    ///
    /// [`LioCb::listio`]: struct.LioCb.html#method.listio
    /// [`LioCb::from_mut_slice`]: struct.LioCb.html#method.from_mut_slice
    /// [`LioCb::from_slice`]: struct.LioCb.html#method.from_slice
    pub fn finish(self) -> LioCb<'a> {
        let len = self.aiocbs.len();
        LioCb {
            aiocbs: self.aiocbs.into(),
            list: Vec::with_capacity(len),
            results: Vec::with_capacity(len)
        }
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[cfg(test)]
mod t {
    use super::*;

    // It's important that `LioCb` be `UnPin`.  The tokio-file crate relies on
    // it.
    #[test]
    fn liocb_is_unpin() {
        use assert_impl::assert_impl;

        assert_impl!(Unpin: LioCb);
    }
}
