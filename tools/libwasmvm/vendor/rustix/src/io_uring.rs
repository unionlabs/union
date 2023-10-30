//! Linux [io_uring].
//!
//! This API is very low-level. The main adaptations it makes from the raw
//! Linux io_uring API are the use of appropriately-sized `bitflags`, `enum`,
//! `Result`, `OwnedFd`, `AsFd`, `RawFd`, and `*mut c_void` in place of plain
//! integers.
//!
//! For a higher-level API built on top of this, see the [rustix-uring] crate.
//!
//! # Safety
//!
//! io_uring operates on raw pointers and raw file descriptors. Rustix does not
//! attempt to provide a safe API for these, because the abstraction level is
//! too low for this to be practical. Safety should be introduced in
//! higher-level abstraction layers.
//!
//! # References
//!  - [Linux]
//!  - [io_uring header]
//!
//! [Linux]: https://man.archlinux.org/man/io_uring.7.en
//! [io_uring]: https://en.wikipedia.org/wiki/Io_uring
//! [io_uring header]: https://github.com/torvalds/linux/blob/master/include/uapi/linux/io_uring.h
//! [rustix-uring]: https://crates.io/crates/rustix-uring
#![allow(unsafe_code)]

use crate::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};
use crate::{backend, io};
use core::ffi::c_void;
use core::mem::{zeroed, MaybeUninit};
use core::ptr::{null_mut, write_bytes};
use linux_raw_sys::general as sys;

/// `io_uring_setup(entries, params)`—Setup a context for performing
/// asynchronous I/O.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man.archlinux.org/man/io_uring_setup.2.en
#[inline]
pub fn io_uring_setup(entries: u32, params: &mut io_uring_params) -> io::Result<OwnedFd> {
    backend::io_uring::syscalls::io_uring_setup(entries, params)
}

/// `io_uring_register(fd, opcode, arg, nr_args)`—Register files or user
/// buffers for asynchronous I/O.
///
/// # Safety
///
/// io_uring operates on raw pointers and raw file descriptors. Users are
/// responsible for ensuring that memory and resources are only accessed in
/// valid ways.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man.archlinux.org/man/io_uring_register.2.en
#[inline]
pub unsafe fn io_uring_register<Fd: AsFd>(
    fd: Fd,
    opcode: IoringRegisterOp,
    arg: *const c_void,
    nr_args: u32,
) -> io::Result<u32> {
    backend::io_uring::syscalls::io_uring_register(fd.as_fd(), opcode, arg, nr_args)
}

/// `io_uring_enter(fd, to_submit, min_complete, flags, arg, size)`—Initiate
/// and/or complete asynchronous I/O.
///
/// # Safety
///
/// io_uring operates on raw pointers and raw file descriptors. Users are
/// responsible for ensuring that memory and resources are only accessed in
/// valid ways.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man.archlinux.org/man/io_uring_enter.2.en
#[inline]
pub unsafe fn io_uring_enter<Fd: AsFd>(
    fd: Fd,
    to_submit: u32,
    min_complete: u32,
    flags: IoringEnterFlags,
    arg: *const c_void,
    size: usize,
) -> io::Result<u32> {
    backend::io_uring::syscalls::io_uring_enter(
        fd.as_fd(),
        to_submit,
        min_complete,
        flags,
        arg,
        size,
    )
}

bitflags::bitflags! {
    /// `IORING_ENTER_*` flags for use with [`io_uring_enter`].
    #[derive(Default)]
    pub struct IoringEnterFlags: u32 {
        /// `IORING_ENTER_GETEVENTS`
        const GETEVENTS = sys::IORING_ENTER_GETEVENTS;

        /// `IORING_ENTER_SQ_WAKEUP`
        const SQ_WAKEUP = sys::IORING_ENTER_SQ_WAKEUP;

        /// `IORING_ENTER_SQ_WAIT`
        const SQ_WAIT = sys::IORING_ENTER_SQ_WAIT;

        /// `IORING_ENTER_EXT_ARG`
        const EXT_ARG = sys::IORING_ENTER_EXT_ARG;
    }
}

/// `IORING_REGISTER_*` and `IORING_UNREGISTER_*` constants for use with
/// [`io_uring_register`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum IoringRegisterOp {
    /// `IORING_REGISTER_BUFFERS`
    RegisterBuffers = sys::IORING_REGISTER_BUFFERS as _,

    /// `IORING_UNREGISTER_BUFFERS`
    UnregisterBuffers = sys::IORING_UNREGISTER_BUFFERS as _,

    /// `IORING_REGISTER_FILES`
    RegisterFiles = sys::IORING_REGISTER_FILES as _,

    /// `IORING_UNREGISTER_FILES`
    UnregisterFiles = sys::IORING_UNREGISTER_FILES as _,

    /// `IORING_REGISTER_EVENTFD`
    RegisterEventfd = sys::IORING_REGISTER_EVENTFD as _,

    /// `IORING_UNREGISTER_EVENTFD`
    UnregisterEventfd = sys::IORING_UNREGISTER_EVENTFD as _,

    /// `IORING_REGISTER_FILES_UPDATE`
    RegisterFilesUpdate = sys::IORING_REGISTER_FILES_UPDATE as _,

    /// `IORING_REGISTER_EVENTFD_ASYNC`
    RegisterEventfdAsync = sys::IORING_REGISTER_EVENTFD_ASYNC as _,

    /// `IORING_REGISTER_PROBE`
    RegisterProbe = sys::IORING_REGISTER_PROBE as _,

    /// `IORING_REGISTER_PERSONALITY`
    RegisterPersonality = sys::IORING_REGISTER_PERSONALITY as _,

    /// `IORING_UNREGISTER_PERSONALITY`
    UnregisterPersonality = sys::IORING_UNREGISTER_PERSONALITY as _,

    /// `IORING_REGISTER_RESTRICTIONS`
    RegisterRestrictions = sys::IORING_REGISTER_RESTRICTIONS as _,

    /// `IORING_REGISTER_ENABLE_RINGS`
    RegisterEnableRings = sys::IORING_REGISTER_ENABLE_RINGS as _,

    /// `IORING_REGISTER_BUFFERS2`
    RegisterBuffers2 = sys::IORING_REGISTER_BUFFERS2 as _,

    /// `IORING_REGISTER_BUFFERS_UPDATE`
    RegisterBuffersUpdate = sys::IORING_REGISTER_BUFFERS_UPDATE as _,

    /// `IORING_REGISTER_FILES2`
    RegisterFiles2 = sys::IORING_REGISTER_FILES2 as _,

    /// `IORING_REGISTER_FILES_SKIP`
    RegisterFilesSkip = sys::IORING_REGISTER_FILES_SKIP as _,

    /// `IORING_REGISTER_FILES_UPDATE2`
    RegisterFilesUpdate2 = sys::IORING_REGISTER_FILES_UPDATE2 as _,

    /// `IORING_REGISTER_IOWQ_AFF`
    RegisterIowqAff = sys::IORING_REGISTER_IOWQ_AFF as _,

    /// `IORING_UNREGISTER_IOWQ_AFF`
    UnregisterIowqAff = sys::IORING_UNREGISTER_IOWQ_AFF as _,

    /// `IORING_REGISTER_IOWQ_MAX_WORKERS`
    RegisterIowqMaxWorkers = sys::IORING_REGISTER_IOWQ_MAX_WORKERS as _,

    /// `IORING_REGISTER_RING_FDS`
    RegisterRingFds = sys::IORING_REGISTER_RING_FDS as _,

    /// `IORING_UNREGISTER_RING_FDS`
    UnregisterRingFds = sys::IORING_UNREGISTER_RING_FDS as _,

    /// `IORING_REGISTER_PBUF_RING`
    RegisterPbufRing = sys::IORING_REGISTER_PBUF_RING as _,

    /// `IORING_UNREGISTER_PBUF_RING`
    UnregisterPbufRing = sys::IORING_UNREGISTER_PBUF_RING as _,

    /// `IORING_REGISTER_SYNC_CANCEL`
    RegisterSyncCancel = sys::IORING_REGISTER_SYNC_CANCEL as _,

    /// `IORING_REGISTER_FILE_ALLOC_RANGE`
    RegisterFileAllocRange = sys::IORING_REGISTER_FILE_ALLOC_RANGE as _,
}

/// `IORING_OP_*` constants for use with [`io_uring_sqe`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum IoringOp {
    /// `IORING_OP_NOP`
    Nop = sys::io_uring_op::IORING_OP_NOP as _,

    /// `IORING_OP_ACCEPT`
    Accept = sys::io_uring_op::IORING_OP_ACCEPT as _,

    /// `IORING_OP_ASYNC_CANCEL`
    AsyncCancel = sys::io_uring_op::IORING_OP_ASYNC_CANCEL as _,

    /// `IORING_OP_CLOSE`
    Close = sys::io_uring_op::IORING_OP_CLOSE as _,

    /// `IORING_OP_CONNECT`
    Connect = sys::io_uring_op::IORING_OP_CONNECT as _,

    /// `IORING_OP_EPOLL_CTL`
    EpollCtl = sys::io_uring_op::IORING_OP_EPOLL_CTL as _,

    /// `IORING_OP_FADVISE`
    Fadvise = sys::io_uring_op::IORING_OP_FADVISE as _,

    /// `IORING_OP_FALLOCATE`
    Fallocate = sys::io_uring_op::IORING_OP_FALLOCATE as _,

    /// `IORING_OP_FILES_UPDATE`
    FilesUpdate = sys::io_uring_op::IORING_OP_FILES_UPDATE as _,

    /// `IORING_OP_FSYNC`
    Fsync = sys::io_uring_op::IORING_OP_FSYNC as _,

    /// `IORING_OP_LINKAT`
    Linkat = sys::io_uring_op::IORING_OP_LINKAT as _,

    /// `IORING_OP_LINK_TIMEOUT`
    LinkTimeout = sys::io_uring_op::IORING_OP_LINK_TIMEOUT as _,

    /// `IORING_OP_MADVISE`
    Madvise = sys::io_uring_op::IORING_OP_MADVISE as _,

    /// `IORING_OP_MKDIRAT`
    Mkdirat = sys::io_uring_op::IORING_OP_MKDIRAT as _,

    /// `IORING_OP_OPENAT`
    Openat = sys::io_uring_op::IORING_OP_OPENAT as _,

    /// `IORING_OP_OPENAT2`
    Openat2 = sys::io_uring_op::IORING_OP_OPENAT2 as _,

    /// `IORING_OP_POLL_ADD`
    PollAdd = sys::io_uring_op::IORING_OP_POLL_ADD as _,

    /// `IORING_OP_POLL_REMOVE`
    PollRemove = sys::io_uring_op::IORING_OP_POLL_REMOVE as _,

    /// `IORING_OP_PROVIDE_BUFFERS`
    ProvideBuffers = sys::io_uring_op::IORING_OP_PROVIDE_BUFFERS as _,

    /// `IORING_OP_READ`
    Read = sys::io_uring_op::IORING_OP_READ as _,

    /// `IORING_OP_READV`
    Readv = sys::io_uring_op::IORING_OP_READV as _,

    /// `IORING_OP_READ_FIXED`
    ReadFixed = sys::io_uring_op::IORING_OP_READ_FIXED as _,

    /// `IORING_OP_RECV`
    Recv = sys::io_uring_op::IORING_OP_RECV as _,

    /// `IORING_OP_RECVMSG`
    Recvmsg = sys::io_uring_op::IORING_OP_RECVMSG as _,

    /// `IORING_OP_REMOVE_BUFFERS`
    RemoveBuffers = sys::io_uring_op::IORING_OP_REMOVE_BUFFERS as _,

    /// `IORING_OP_RENAMEAT`
    Renameat = sys::io_uring_op::IORING_OP_RENAMEAT as _,

    /// `IORING_OP_SEND`
    Send = sys::io_uring_op::IORING_OP_SEND as _,

    /// `IORING_OP_SENDMSG`
    Sendmsg = sys::io_uring_op::IORING_OP_SENDMSG as _,

    /// `IORING_OP_SHUTDOWN`
    Shutdown = sys::io_uring_op::IORING_OP_SHUTDOWN as _,

    /// `IORING_OP_SPLICE`
    Splice = sys::io_uring_op::IORING_OP_SPLICE as _,

    /// `IORING_OP_STATX`
    Statx = sys::io_uring_op::IORING_OP_STATX as _,

    /// `IORING_OP_SYMLINKAT`
    Symlinkat = sys::io_uring_op::IORING_OP_SYMLINKAT as _,

    /// `IORING_OP_SYNC_FILE_RANGE`
    SyncFileRange = sys::io_uring_op::IORING_OP_SYNC_FILE_RANGE as _,

    /// `IORING_OP_TEE`
    Tee = sys::io_uring_op::IORING_OP_TEE as _,

    /// `IORING_OP_TIMEOUT`
    Timeout = sys::io_uring_op::IORING_OP_TIMEOUT as _,

    /// `IORING_OP_TIMEOUT_REMOVE`
    TimeoutRemove = sys::io_uring_op::IORING_OP_TIMEOUT_REMOVE as _,

    /// `IORING_OP_UNLINKAT`
    Unlinkat = sys::io_uring_op::IORING_OP_UNLINKAT as _,

    /// `IORING_OP_WRITE`
    Write = sys::io_uring_op::IORING_OP_WRITE as _,

    /// `IORING_OP_WRITEV`
    Writev = sys::io_uring_op::IORING_OP_WRITEV as _,

    /// `IORING_OP_WRITE_FIXED`
    WriteFixed = sys::io_uring_op::IORING_OP_WRITE_FIXED as _,

    /// `IORING_OP_MSG_RING`
    MsgRing = sys::io_uring_op::IORING_OP_MSG_RING as _,

    /// `IORING_OP_FSETXATTR`
    Fsetxattr = sys::io_uring_op::IORING_OP_FSETXATTR as _,

    /// `IORING_OP_SETXATTR`
    Setxattr = sys::io_uring_op::IORING_OP_SETXATTR as _,

    /// `IORING_OP_FGETXATTR`
    Fgetxattr = sys::io_uring_op::IORING_OP_FGETXATTR as _,

    /// `IORING_OP_GETXATTR`
    Getxattr = sys::io_uring_op::IORING_OP_GETXATTR as _,

    /// `IORING_OP_SOCKET`
    Socket = sys::io_uring_op::IORING_OP_SOCKET as _,

    /// `IORING_OP_URING_CMD`
    UringCmd = sys::io_uring_op::IORING_OP_URING_CMD as _,

    /// `IORING_OP_SEND_ZC`
    SendZc = sys::io_uring_op::IORING_OP_SEND_ZC as _,

    /// `IORING_OP_SENDMSG_ZC`
    SendmsgZc = sys::io_uring_op::IORING_OP_SENDMSG_ZC as _,
}

impl Default for IoringOp {
    #[inline]
    fn default() -> Self {
        Self::Nop
    }
}

/// `IORING_RESTRICTION_*` constants for use with [`io_uring_restriction`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum IoringRestrictionOp {
    /// `IORING_RESTRICTION_REGISTER_OP`
    RegisterOp = sys::IORING_RESTRICTION_REGISTER_OP as _,

    /// `IORING_RESTRICTION_SQE_FLAGS_ALLOWED`
    SqeFlagsAllowed = sys::IORING_RESTRICTION_SQE_FLAGS_ALLOWED as _,

    /// `IORING_RESTRICTION_SQE_FLAGS_REQUIRED`
    SqeFlagsRequired = sys::IORING_RESTRICTION_SQE_FLAGS_REQUIRED as _,

    /// `IORING_RESTRICTION_SQE_OP`
    SqeOp = sys::IORING_RESTRICTION_SQE_OP as _,
}

impl Default for IoringRestrictionOp {
    #[inline]
    fn default() -> Self {
        Self::RegisterOp
    }
}

/// `IORING_MSG_*` constants which represent commands for use with
/// [`IoringOp::MsgRing`], (`seq.addr`)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u64)]
#[non_exhaustive]
pub enum IoringMsgringCmds {
    /// `IORING_MSG_DATA`
    Data = sys::IORING_MSG_DATA as _,

    /// `IORING_MSG_SEND_FD`
    SendFd = sys::IORING_MSG_SEND_FD as _,
}

bitflags::bitflags! {
    /// `IORING_SETUP_*` flags for use with [`io_uring_params`].
    #[derive(Default)]
    pub struct IoringSetupFlags: u32 {
        /// `IORING_SETUP_ATTACH_WQ`
        const ATTACH_WQ = sys::IORING_SETUP_ATTACH_WQ;

        /// `IORING_SETUP_CLAMP`
        const CLAMP = sys::IORING_SETUP_CLAMP;

        /// `IORING_SETUP_CQSIZE`
        const CQSIZE = sys::IORING_SETUP_CQSIZE;

        /// `IORING_SETUP_IOPOLL`
        const IOPOLL = sys::IORING_SETUP_IOPOLL;

        /// `IORING_SETUP_R_DISABLED`
        const R_DISABLED = sys::IORING_SETUP_R_DISABLED;

        /// `IORING_SETUP_SQPOLL`
        const SQPOLL = sys::IORING_SETUP_SQPOLL;

        /// `IORING_SETUP_SQ_AFF`
        const SQ_AFF = sys::IORING_SETUP_SQ_AFF;

        /// `IORING_SETUP_SQE128`
        const SQE128 = sys::IORING_SETUP_SQE128;

        /// `IORING_SETUP_CQE32`
        const CQE32 = sys::IORING_SETUP_CQE32;

        /// `IORING_SETUP_SUBMIT_ALL`
        const SUBMIT_ALL = sys::IORING_SETUP_SUBMIT_ALL;

        /// `IORING_SETUP_COOP_TRASKRUN`
        const COOP_TASKRUN = sys::IORING_SETUP_COOP_TASKRUN;

        /// `IORING_SETUP_TASKRUN_FLAG`
        const TASKRUN_FLAG = sys::IORING_SETUP_TASKRUN_FLAG;

        /// `IORING_SETUP_SINGLE_ISSUER`
        const SINGLE_ISSUER = sys::IORING_SETUP_SINGLE_ISSUER;

        /// `IORING_SETUP_DEFER_TASKRUN`
        const DEFER_TASKRUN = sys::IORING_SETUP_DEFER_TASKRUN;
    }
}

bitflags::bitflags! {
    /// `IOSQE_*` flags for use with [`io_uring_sqe`].
    #[derive(Default)]
    pub struct IoringSqeFlags: u8 {
        /// `1 << IOSQE_ASYNC_BIT`
        const ASYNC = 1 << sys::IOSQE_ASYNC_BIT as u8;

        /// `1 << IOSQE_BUFFER_SELECT_BIT`
        const BUFFER_SELECT = 1 << sys::IOSQE_BUFFER_SELECT_BIT as u8;

        /// `1 << IOSQE_FIXED_FILE_BIT`
        const FIXED_FILE = 1 << sys::IOSQE_FIXED_FILE_BIT as u8;

        /// 1 << `IOSQE_IO_DRAIN_BIT`
        const IO_DRAIN = 1 << sys::IOSQE_IO_DRAIN_BIT as u8;

        /// `1 << IOSQE_IO_HARDLINK_BIT`
        const IO_HARDLINK = 1 << sys::IOSQE_IO_HARDLINK_BIT as u8;

        /// `1 << IOSQE_IO_LINK_BIT`
        const IO_LINK = 1 << sys::IOSQE_IO_LINK_BIT as u8;

        /// `1 << IOSQE_CQE_SKIP_SUCCESS_BIT`
        const CQE_SKIP_SUCCESS = 1 << sys::IOSQE_CQE_SKIP_SUCCESS_BIT as u8;
    }
}

bitflags::bitflags! {
    /// `IORING_CQE_F_*` flags for use with [`io_uring_cqe`].
    #[derive(Default)]
    pub struct IoringCqeFlags: u32 {
        /// `IORING_CQE_F_BUFFER`
        const BUFFER = sys::IORING_CQE_F_BUFFER as _;

        /// `IORING_CQE_F_MORE`
        const MORE = sys::IORING_CQE_F_MORE as _;

        /// `IORING_CQE_F_SOCK_NONEMPTY`
        const SOCK_NONEMPTY = sys::IORING_CQE_F_SOCK_NONEMPTY as _;

        /// `IORING_CQE_F_NOTIF`
        const NOTIF = sys::IORING_CQE_F_NOTIF as _;
    }
}

bitflags::bitflags! {
    /// `IORING_FSYNC_*` flags for use with [`io_uring_sqe`].
    #[derive(Default)]
    pub struct IoringFsyncFlags: u32 {
        /// `IORING_FSYNC_DATASYNC`
        const DATASYNC = sys::IORING_FSYNC_DATASYNC;
    }
}

bitflags::bitflags! {
    /// `IORING_TIMEOUT_*` and `IORING_LINK_TIMEOUT_UPDATE` flags for use with
    /// [`io_uring_sqe`].
    #[derive(Default)]
    pub struct IoringTimeoutFlags: u32 {
        /// `IORING_TIMEOUT_ABS`
        const ABS = sys::IORING_TIMEOUT_ABS;

        /// `IORING_TIMEOUT_UPDATE`
        const UPDATE = sys::IORING_TIMEOUT_UPDATE;

        /// `IORING_TIMEOUT_BOOTTIME`
        const BOOTTIME = sys::IORING_TIMEOUT_BOOTTIME;

        /// `IORING_TIMEOUT_ETIME_SUCCESS`
        const ETIME_SUCCESS = sys::IORING_TIMEOUT_ETIME_SUCCESS;

        /// `IORING_TIMEOUT_REALTIME`
        const REALTIME = sys::IORING_TIMEOUT_REALTIME;

        /// `IORING_TIMEOUT_CLOCK_MASK`
        const CLOCK_MASK = sys::IORING_TIMEOUT_CLOCK_MASK;

        /// `IORING_TIMEOUT_UPDATE_MASK`
        const UPDATE_MASK = sys::IORING_TIMEOUT_UPDATE_MASK;

        /// `IORING_LINK_TIMEOUT_UPDATE`
        const LINK_TIMEOUT_UPDATE = sys::IORING_LINK_TIMEOUT_UPDATE;
    }
}

bitflags::bitflags! {
    /// `SPLICE_F_*` flags for use with [`io_uring_sqe`].
    #[derive(Default)]
    pub struct SpliceFlags: u32 {
        /// `SPLICE_F_FD_IN_FIXED`
        const FD_IN_FIXED = sys::SPLICE_F_FD_IN_FIXED;
    }
}

bitflags::bitflags! {
    /// `IORING_MSG_RING_*` flags for use with [`io_uring_sqe`].
    #[derive(Default)]
    pub struct IoringMsgringFlags: u32 {
        /// `IORING_MSG_RING_CQE_SKIP`
        const CQE_SKIP = sys::IORING_MSG_RING_CQE_SKIP;
    }
}

bitflags::bitflags! {
    /// `IORING_ASYNC_CANCEL_*` flags for use with [`io_uring_sqe`].
    #[derive(Default)]
    pub struct IoringAsyncCancelFlags: u32 {
        /// `IORING_ASYNC_CANCEL_ALL`
        const ALL = sys::IORING_ASYNC_CANCEL_ALL;

        /// `IORING_ASYNC_CANCEL_FD`
        const FD = sys::IORING_ASYNC_CANCEL_FD;

        /// `IORING_ASYNC_CANCEL_FD`
        const ANY = sys::IORING_ASYNC_CANCEL_ANY;

        /// `IORING_ASYNC_CANCEL_FD`
        const FD_FIXED = sys::IORING_ASYNC_CANCEL_FD_FIXED;
    }
}

bitflags::bitflags! {
    /// `IORING_FEAT_*` flags for use with [`io_uring_params`].
    #[derive(Default)]
    pub struct IoringFeatureFlags: u32 {
        /// `IORING_FEAT_CQE_SKIP`
        const CQE_SKIP = sys::IORING_FEAT_CQE_SKIP;

        /// `IORING_FEAT_CUR_PERSONALITY`
        const CUR_PERSONALITY = sys::IORING_FEAT_CUR_PERSONALITY;

        /// `IORING_FEAT_EXT_ARG`
        const EXT_ARG = sys::IORING_FEAT_EXT_ARG;

        /// `IORING_FEAT_FAST_POLL`
        const FAST_POLL = sys::IORING_FEAT_FAST_POLL;

        /// `IORING_FEAT_NATIVE_WORKERS`
        const NATIVE_WORKERS = sys::IORING_FEAT_NATIVE_WORKERS;

        /// `IORING_FEAT_NODROP`
        const NODROP = sys::IORING_FEAT_NODROP;

        /// `IORING_FEAT_POLL_32BITS`
        const POLL_32BITS = sys::IORING_FEAT_POLL_32BITS;

        /// `IORING_FEAT_RSRC_TAGS`
        const RSRC_TAGS = sys::IORING_FEAT_RSRC_TAGS;

        /// `IORING_FEAT_RW_CUR_POS`
        const RW_CUR_POS = sys::IORING_FEAT_RW_CUR_POS;

        /// `IORING_FEAT_SINGLE_MMAP`
        const SINGLE_MMAP = sys::IORING_FEAT_SINGLE_MMAP;

        /// `IORING_FEAT_SQPOLL_NONFIXED`
        const SQPOLL_NONFIXED = sys::IORING_FEAT_SQPOLL_NONFIXED;

        /// `IORING_FEAT_SUBMIT_STABLE`
        const SUBMIT_STABLE = sys::IORING_FEAT_SUBMIT_STABLE;

        /// `IORING_FEAT_LINKED_FILE`
        const LINKED_FILE = sys::IORING_FEAT_LINKED_FILE;
    }
}

bitflags::bitflags! {
    /// `IO_URING_OP_*` flags for use with [`io_uring_probe_op`].
    #[derive(Default)]
    pub struct IoringOpFlags: u16 {
        /// `IO_URING_OP_SUPPORTED`
        const SUPPORTED = sys::IO_URING_OP_SUPPORTED as _;
    }
}

bitflags::bitflags! {
    /// `IORING_RSRC_*` flags for use with [`io_uring_rsrc_register`].
    #[derive(Default)]
    pub struct IoringRsrcFlags: u32 {
        /// `IORING_RSRC_REGISTER_SPARSE`
        const REGISTER_SPARSE = sys::IORING_RSRC_REGISTER_SPARSE as _;
    }
}

bitflags::bitflags! {
    /// `IORING_SQ_*` flags.
    #[derive(Default)]
    pub struct IoringSqFlags: u32 {
        /// `IORING_SQ_NEED_WAKEUP`
        const NEED_WAKEUP = sys::IORING_SQ_NEED_WAKEUP;

        /// `IORING_SQ_CQ_OVERFLOW`
        const CQ_OVERFLOW = sys::IORING_SQ_CQ_OVERFLOW;

        /// `IORING_SQ_TASKRUN`
        const TASKRUN = sys::IORING_SQ_TASKRUN;
    }
}

bitflags::bitflags! {
    /// `IORING_CQ_*` flags.
    #[derive(Default)]
    pub struct IoringCqFlags: u32 {
        /// `IORING_CQ_EVENTFD_DISABLED`
        const EVENTFD_DISABLED = sys::IORING_CQ_EVENTFD_DISABLED;
    }
}

bitflags::bitflags! {
    /// `IORING_POLL_*` flags.
    #[derive(Default)]
    pub struct IoringPollFlags: u32 {
        /// `IORING_POLL_ADD_MULTI`
        const ADD_MULTI = sys::IORING_POLL_ADD_MULTI;

        /// `IORING_POLL_UPDATE_EVENTS`
        const UPDATE_EVENTS = sys::IORING_POLL_UPDATE_EVENTS;

        /// `IORING_POLL_UPDATE_USER_DATA`
        const UPDATE_USER_DATA = sys::IORING_POLL_UPDATE_USER_DATA;

        /// `IORING_POLL_ADD_LEVEL`
        const ADD_LEVEL = sys::IORING_POLL_ADD_LEVEL;
    }
}

bitflags::bitflags! {
    /// send/sendmsg flags (`sqe.ioprio`)
    #[derive(Default)]
    pub struct IoringSendFlags: u16 {
        /// `IORING_RECVSEND_POLL_FIRST`.
        ///
        /// See also [`IoringRecvFlags::POLL_FIRST`].
        const POLL_FIRST = sys::IORING_RECVSEND_POLL_FIRST as _;

        /// `IORING_RECVSEND_FIXED_BUF`
        ///
        /// See also [`IoringRecvFlags::FIXED_BUF`].
        const FIXED_BUF = sys::IORING_RECVSEND_FIXED_BUF as _;

        /// `IORING_SEND_ZC_REPORT_USAGE` (since Linux 6.2)
        const ZC_REPORT_USAGE = sys::IORING_SEND_ZC_REPORT_USAGE as _;
    }
}

bitflags::bitflags! {
    /// recv/recvmsg flags (`sqe.ioprio`)
    #[derive(Default)]
    pub struct IoringRecvFlags: u16 {
        /// `IORING_RECVSEND_POLL_FIRST`
        ///
        /// See also [`IoringSendFlags::POLL_FIRST`].
        const POLL_FIRST = sys::IORING_RECVSEND_POLL_FIRST as _;

        /// `IORING_RECV_MULTISHOT`
        const MULTISHOT = sys::IORING_RECV_MULTISHOT as _;

        /// `IORING_RECVSEND_FIXED_BUF`
        ///
        /// See also [`IoringSendFlags::FIXED_BUF`].
        const FIXED_BUF = sys::IORING_RECVSEND_FIXED_BUF as _;
    }
}

bitflags::bitflags! {
    /// accept flags (`sqe.ioprio`)
    #[derive(Default)]
    pub struct IoringAcceptFlags: u16 {
        /// `IORING_ACCEPT_MULTISHOT`
        const MULTISHOT = sys::IORING_ACCEPT_MULTISHOT as _;
    }
}

bitflags::bitflags! {
    /// recvmsg out flags
    #[derive(Default)]
    pub struct RecvmsgOutFlags: u32 {
        /// `MSG_EOR`
        const EOR = sys::MSG_EOR;

        /// `MSG_TRUNC`
        const TRUNC = sys::MSG_TRUNC;

        /// `MSG_CTRUNC`
        const CTRUNC = sys::MSG_CTRUNC;

        /// `MSG_OOB`
        const OOB = sys::MSG_OOB;

        /// `MSG_ERRQUEUE`
        const ERRQUEUE = sys::MSG_ERRQUEUE;
    }
}

#[allow(missing_docs)]
pub const IORING_CQE_BUFFER_SHIFT: u32 = sys::IORING_CQE_BUFFER_SHIFT as _;
#[allow(missing_docs)]
pub const IORING_FILE_INDEX_ALLOC: i32 = sys::IORING_FILE_INDEX_ALLOC as _;

// Re-export these as `u64`, which is the `offset` type in `rustix::io::mmap`.
#[allow(missing_docs)]
pub const IORING_OFF_SQ_RING: u64 = sys::IORING_OFF_SQ_RING as _;
#[allow(missing_docs)]
pub const IORING_OFF_CQ_RING: u64 = sys::IORING_OFF_CQ_RING as _;
#[allow(missing_docs)]
pub const IORING_OFF_SQES: u64 = sys::IORING_OFF_SQES as _;

/// `IORING_REGISTER_FILES_SKIP`
#[inline]
#[doc(alias = "IORING_REGISTER_FILES_SKIP")]
pub const fn io_uring_register_files_skip() -> BorrowedFd<'static> {
    let files_skip = sys::IORING_REGISTER_FILES_SKIP as RawFd;

    // SAFETY: `IORING_REGISTER_FILES_SKIP` is a reserved value that is never
    // dynamically allocated, so it'll remain valid for the duration of
    // `'static`.
    unsafe { BorrowedFd::<'static>::borrow_raw(files_skip) }
}

/// `IORING_NOTIF_USAGE_ZC_COPIED` (since Linux 6.2)
pub const IORING_NOTIF_USAGE_ZC_COPIED: i32 = sys::IORING_NOTIF_USAGE_ZC_COPIED as _;

/// A pointer in the io_uring API.
///
/// `io_uring`'s native API represents pointers as `u64` values. In order to
/// preserve strict-provenance, use a `*mut c_void`. On platforms where
/// pointers are narrower than 64 bits, this requires additional padding.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct io_uring_ptr {
    #[cfg(all(target_pointer_width = "32", target_endian = "big"))]
    #[doc(hidden)]
    pub __pad32: u32,
    #[cfg(all(target_pointer_width = "16", target_endian = "big"))]
    #[doc(hidden)]
    pub __pad16: u16,

    /// The pointer value.
    pub ptr: *mut c_void,

    #[cfg(all(target_pointer_width = "16", target_endian = "little"))]
    #[doc(hidden)]
    pub __pad16: u16,
    #[cfg(all(target_pointer_width = "32", target_endian = "little"))]
    #[doc(hidden)]
    pub __pad32: u32,
}

impl From<*mut c_void> for io_uring_ptr {
    #[inline]
    fn from(ptr: *mut c_void) -> Self {
        Self {
            ptr,

            #[cfg(target_pointer_width = "16")]
            __pad16: Default::default(),
            #[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
            __pad32: Default::default(),
        }
    }
}

impl Default for io_uring_ptr {
    #[inline]
    fn default() -> Self {
        Self::from(null_mut())
    }
}

/// User data in the io_uring API.
///
/// `io_uring`'s native API represents `user_data` fields as `u64` values. In
/// order to preserve strict-provenance, use a union which allows users to
/// optionally store pointers.
#[repr(C)]
#[derive(Copy, Clone)]
pub union io_uring_user_data {
    /// An arbitrary `u64`.
    pub u64_: u64,

    /// A pointer.
    pub ptr: io_uring_ptr,
}

impl io_uring_user_data {
    /// Return the `u64` value.
    #[inline]
    pub fn u64_(self) -> u64 {
        // SAFETY: All the fields have the same underlying representation.
        unsafe { self.u64_ }
    }

    /// Create a `Self` from a `u64` value.
    #[inline]
    pub fn from_u64(u64_: u64) -> Self {
        Self { u64_ }
    }

    /// Return the `ptr` pointer value.
    #[inline]
    pub fn ptr(self) -> *mut c_void {
        // SAFETY: All the fields have the same underlying representation.
        unsafe { self.ptr }.ptr
    }

    /// Create a `Self` from a pointer value.
    #[inline]
    pub fn from_ptr(ptr: *mut c_void) -> Self {
        Self {
            ptr: io_uring_ptr::from(ptr),
        }
    }
}

impl Default for io_uring_user_data {
    #[inline]
    fn default() -> Self {
        let mut s = MaybeUninit::<Self>::uninit();
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe {
            write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

impl core::fmt::Debug for io_uring_user_data {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // SAFETY: Just format as a `u64`, since formatting doesn't preserve
        // provenance, and we don't have a discriminant.
        unsafe { self.u64_.fmt(fmt) }
    }
}

/// An io_uring Submission Queue Entry.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct io_uring_sqe {
    pub opcode: IoringOp,
    pub flags: IoringSqeFlags,
    pub ioprio: ioprio_union,
    pub fd: RawFd,
    pub off_or_addr2: off_or_addr2_union,
    pub addr_or_splice_off_in: addr_or_splice_off_in_union,
    pub len: len_union,
    pub op_flags: op_flags_union,
    pub user_data: io_uring_user_data,
    pub buf: buf_union,
    pub personality: u16,
    pub splice_fd_in_or_file_index: splice_fd_in_or_file_index_union,
    pub addr3_or_cmd: addr3_or_cmd_union,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union ioprio_union {
    pub recv_flags: IoringRecvFlags,
    pub send_flags: IoringSendFlags,
    pub accept_flags: IoringAcceptFlags,
    pub ioprio: u16,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union len_union {
    pub poll_flags: IoringPollFlags,
    pub len: u32,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union addr3_or_cmd_union {
    pub addr3: addr3_struct,
    pub cmd: [u8; 0],
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct addr3_struct {
    pub addr3: u64,
    pub __pad2: [u64; 1],
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union off_or_addr2_union {
    pub off: u64,
    pub addr2: io_uring_ptr,
    pub cmd_op: cmd_op_struct,
    pub user_data: io_uring_user_data,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cmd_op_struct {
    pub cmd_op: u32,
    pub __pad1: u32,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union addr_or_splice_off_in_union {
    pub addr: io_uring_ptr,
    pub splice_off_in: u64,
    pub msgring_cmd: IoringMsgringCmds,
    pub user_data: io_uring_user_data,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union op_flags_union {
    pub rw_flags: crate::io::ReadWriteFlags,
    pub fsync_flags: IoringFsyncFlags,
    pub poll_events: u16,
    pub poll32_events: u32,
    pub sync_range_flags: u32,
    /// `msg_flags` is split into `send_flags` and `recv_flags`.
    #[doc(alias = "msg_flags")]
    pub send_flags: crate::net::SendFlags,
    /// `msg_flags` is split into `send_flags` and `recv_flags`.
    #[doc(alias = "msg_flags")]
    pub recv_flags: crate::net::RecvFlags,
    pub timeout_flags: IoringTimeoutFlags,
    pub accept_flags: crate::net::SocketFlags,
    pub cancel_flags: IoringAsyncCancelFlags,
    pub open_flags: crate::fs::OFlags,
    pub statx_flags: crate::fs::AtFlags,
    pub fadvise_advice: crate::fs::Advice,
    pub splice_flags: SpliceFlags,
    pub rename_flags: crate::fs::RenameFlags,
    pub unlink_flags: crate::fs::AtFlags,
    pub hardlink_flags: crate::fs::AtFlags,
    pub msg_ring_flags: IoringMsgringFlags,
}

#[allow(missing_docs)]
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub union buf_union {
    pub buf_index: u16,
    pub buf_group: u16,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union splice_fd_in_or_file_index_union {
    pub splice_fd_in: i32,
    pub file_index: u32,
}

/// An io_uring Completion Queue Entry.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_cqe {
    pub user_data: io_uring_user_data,
    pub res: i32,
    pub flags: IoringCqeFlags,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct io_uring_restriction {
    pub opcode: IoringRestrictionOp,
    pub register_or_sqe_op_or_sqe_flags: register_or_sqe_op_or_sqe_flags_union,
    pub resv: u8,
    pub resv2: [u32; 3],
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union register_or_sqe_op_or_sqe_flags_union {
    pub register_op: IoringRegisterOp,
    pub sqe_op: IoringOp,
    pub sqe_flags: IoringSqeFlags,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_params {
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub flags: IoringSetupFlags,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    pub features: IoringFeatureFlags,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_off: io_sqring_offsets,
    pub cq_off: io_cqring_offsets,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_sqring_offsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub flags: u32,
    pub dropped: u32,
    pub array: u32,
    pub resv1: u32,
    pub resv2: u64,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_cqring_offsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub overflow: u32,
    pub cqes: u32,
    pub flags: u32,
    pub resv1: u32,
    pub resv2: u64,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
pub struct io_uring_probe {
    pub last_op: IoringOp,
    pub ops_len: u8,
    pub resv: u16,
    pub resv2: [u32; 3],
    pub ops: sys::__IncompleteArrayField<io_uring_probe_op>,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_probe_op {
    pub op: IoringOp,
    pub resv: u8,
    pub flags: IoringOpFlags,
    pub resv2: u32,
}

#[allow(missing_docs)]
#[repr(C, align(8))]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_files_update {
    pub offset: u32,
    pub resv: u32,
    pub fds: u64,
}

#[allow(missing_docs)]
#[repr(C, align(8))]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_rsrc_register {
    pub nr: u32,
    pub flags: IoringRsrcFlags,
    pub resv2: u64,
    pub data: u64,
    pub tags: u64,
}

#[allow(missing_docs)]
#[repr(C, align(8))]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_rsrc_update {
    pub offset: u32,
    pub resv: u32,
    pub data: u64,
}

#[allow(missing_docs)]
#[repr(C, align(8))]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_rsrc_update2 {
    pub offset: u32,
    pub resv: u32,
    pub data: u64,
    pub tags: u64,
    pub nr: u32,
    pub resv2: u32,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_getevents_arg {
    pub sigmask: u64,
    pub sigmask_sz: u32,
    pub pad: u32,
    pub ts: u64,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct io_uring_recvmsg_out {
    pub namelen: u32,
    pub controllen: u32,
    pub payloadlen: u32,
    pub flags: RecvmsgOutFlags,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct iovec {
    pub iov_base: *mut c_void,
    pub iov_len: usize,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct open_how {
    /// An [`OFlags`] value represented as a `u64`.
    ///
    /// [`OFlags`]: crate::fs::OFlags
    pub flags: u64,

    /// A [`Mode`] value represented as a `u64`.
    ///
    /// [`Mode`]: crate::fs::Mode
    pub mode: u64,

    pub resolve: crate::fs::ResolveFlags,
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_buf_reg {
    pub ring_addr: u64,
    pub ring_entries: u32,
    pub bgid: u16,
    pub pad: u16,
    pub resv: [u64; 3_usize],
}

#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct io_uring_buf {
    pub addr: u64,
    pub len: u32,
    pub bid: u16,
    pub resv: u16,
}

impl Default for ioprio_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for len_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for off_or_addr2_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for addr_or_splice_off_in_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for addr3_or_cmd_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for op_flags_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for buf_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for splice_fd_in_or_file_index_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

impl Default for register_or_sqe_op_or_sqe_flags_union {
    #[inline]
    fn default() -> Self {
        // SAFETY: All of Linux's io_uring structs may be zero-initialized.
        unsafe { zeroed::<Self>() }
    }
}

/// Check that our custom structs and unions have the same layout as the
/// kernel's versions.
#[test]
fn io_uring_layouts() {
    use core::mem::{align_of, size_of};
    use memoffset::{offset_of, span_of};

    // Check that the size and alignment of a type match the `sys` bindings.
    macro_rules! check_type {
        ($struct:ident) => {
            assert_eq!(
                (size_of::<$struct>(), align_of::<$struct>()),
                (size_of::<sys::$struct>(), align_of::<sys::$struct>())
            );
        };
    }

    // The same as `check_type`, but for unions and anonymous structs we've
    // renamed to avoid having types like "bindgen_ty_1" in the API.
    macro_rules! check_renamed_type {
        ($to:ident, $from:ident) => {
            assert_eq!(
                (size_of::<$to>(), align_of::<$to>()),
                (size_of::<sys::$from>(), align_of::<sys::$from>())
            );
        };
    }

    // Check that the field of a struct has the same offset as the
    // corresponding field in the `sys` bindings.
    macro_rules! check_struct_field {
        ($struct:ident, $field:ident) => {
            assert_eq!(
                offset_of!($struct, $field),
                offset_of!(sys::$struct, $field)
            );
            assert_eq!(span_of!($struct, $field), span_of!(sys::$struct, $field));
        };
    }

    // The same as `check_struct_field`, but for unions and anonymous structs
    // we've renamed to avoid having types like "bindgen_ty_1" in the API.
    macro_rules! check_struct_renamed_field {
        ($struct:ident, $to:ident, $from:ident) => {
            assert_eq!(offset_of!($struct, $to), offset_of!(sys::$struct, $from));
            assert_eq!(span_of!($struct, $to), span_of!(sys::$struct, $from));
        };
    }

    // For the common case of no renaming, check all fields of a struct.
    macro_rules! check_struct {
        ($name:ident, $($field:ident),*) => {
            // Check the size and alignment.
            check_type!($name);

            // Check that we have all the fields.
            let _test = $name {
                // SAFETY: All of io_uring's types can be zero-initialized.
                $($field: unsafe { core::mem::zeroed() }),*
            };

            // Check that the fields have the right sizes and offsets.
            $(check_struct_field!($name, $field));*
        };
    }

    check_renamed_type!(off_or_addr2_union, io_uring_sqe__bindgen_ty_1);
    check_renamed_type!(addr_or_splice_off_in_union, io_uring_sqe__bindgen_ty_2);
    check_renamed_type!(addr3_or_cmd_union, io_uring_sqe__bindgen_ty_6);
    check_renamed_type!(op_flags_union, io_uring_sqe__bindgen_ty_3);
    check_renamed_type!(buf_union, io_uring_sqe__bindgen_ty_4);
    check_renamed_type!(splice_fd_in_or_file_index_union, io_uring_sqe__bindgen_ty_5);
    check_renamed_type!(
        register_or_sqe_op_or_sqe_flags_union,
        io_uring_restriction__bindgen_ty_1
    );

    check_renamed_type!(addr3_struct, io_uring_sqe__bindgen_ty_6__bindgen_ty_1);
    check_renamed_type!(cmd_op_struct, io_uring_sqe__bindgen_ty_1__bindgen_ty_1);

    check_type!(io_uring_sqe);
    check_struct_field!(io_uring_sqe, opcode);
    check_struct_field!(io_uring_sqe, flags);
    check_struct_field!(io_uring_sqe, ioprio);
    check_struct_field!(io_uring_sqe, fd);
    check_struct_renamed_field!(io_uring_sqe, off_or_addr2, __bindgen_anon_1);
    check_struct_renamed_field!(io_uring_sqe, addr_or_splice_off_in, __bindgen_anon_2);
    check_struct_field!(io_uring_sqe, len);
    check_struct_renamed_field!(io_uring_sqe, op_flags, __bindgen_anon_3);
    check_struct_field!(io_uring_sqe, user_data);
    check_struct_renamed_field!(io_uring_sqe, buf, __bindgen_anon_4);
    check_struct_field!(io_uring_sqe, personality);
    check_struct_renamed_field!(io_uring_sqe, splice_fd_in_or_file_index, __bindgen_anon_5);
    check_struct_renamed_field!(io_uring_sqe, addr3_or_cmd, __bindgen_anon_6);

    check_type!(io_uring_restriction);
    check_struct_field!(io_uring_restriction, opcode);
    check_struct_renamed_field!(
        io_uring_restriction,
        register_or_sqe_op_or_sqe_flags,
        __bindgen_anon_1
    );
    check_struct_field!(io_uring_restriction, resv);
    check_struct_field!(io_uring_restriction, resv2);

    check_struct!(io_uring_cqe, user_data, res, flags);
    check_struct!(
        io_uring_params,
        sq_entries,
        cq_entries,
        flags,
        sq_thread_cpu,
        sq_thread_idle,
        features,
        wq_fd,
        resv,
        sq_off,
        cq_off
    );
    check_struct!(
        io_sqring_offsets,
        head,
        tail,
        ring_mask,
        ring_entries,
        flags,
        dropped,
        array,
        resv1,
        resv2
    );
    check_struct!(
        io_cqring_offsets,
        head,
        tail,
        ring_mask,
        ring_entries,
        overflow,
        cqes,
        flags,
        resv1,
        resv2
    );
    check_struct!(io_uring_recvmsg_out, namelen, controllen, payloadlen, flags);
    check_struct!(io_uring_probe, last_op, ops_len, resv, resv2, ops);
    check_struct!(io_uring_probe_op, op, resv, flags, resv2);
    check_struct!(io_uring_files_update, offset, resv, fds);
    check_struct!(io_uring_rsrc_register, nr, flags, resv2, data, tags);
    check_struct!(io_uring_rsrc_update, offset, resv, data);
    check_struct!(io_uring_rsrc_update2, offset, resv, data, tags, nr, resv2);
    check_struct!(io_uring_getevents_arg, sigmask, sigmask_sz, pad, ts);
    check_struct!(iovec, iov_base, iov_len);
    check_struct!(open_how, flags, mode, resolve);
    check_struct!(io_uring_buf_reg, ring_addr, ring_entries, bgid, pad, resv);
    check_struct!(io_uring_buf, addr, len, bid, resv);
}
