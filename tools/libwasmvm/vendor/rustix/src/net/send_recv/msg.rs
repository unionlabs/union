//! [`recvmsg`], [`sendmsg_noaddr`], and related functions.

#![allow(unsafe_code)]

use crate::backend::{self, c};
use crate::fd::{AsFd, BorrowedFd, OwnedFd};
use crate::io::{self, IoSlice, IoSliceMut};

use core::convert::{TryFrom, TryInto};
use core::iter::{FromIterator, FusedIterator};
use core::marker::PhantomData;
use core::mem::{size_of, size_of_val, take};
use core::{ptr, slice};

use super::{RecvFlags, SendFlags, SocketAddrAny, SocketAddrV4, SocketAddrV6};

/// Macro for defining the amount of space used by CMSGs.
#[macro_export]
macro_rules! cmsg_space {
    // Base Rules
    (ScmRights($len:expr)) => {
        $crate::net::__cmsg_space(
            $len * ::core::mem::size_of::<$crate::fd::BorrowedFd<'static>>(),
        )
    };

    // Combo Rules
    (($($($x:tt)*),+)) => {
        $(
            cmsg_space!($($x)*) +
        )+
        0
    };
}

#[doc(hidden)]
pub fn __cmsg_space(len: usize) -> usize {
    unsafe { c::CMSG_SPACE(len.try_into().expect("CMSG_SPACE size overflow")) as usize }
}

/// Ancillary message for [`sendmsg_noaddr`], [`sendmsg_v4`], [`sendmsg_v6`],
/// [`sendmsg_unix`], and [`sendmsg_any`].
#[non_exhaustive]
pub enum SendAncillaryMessage<'slice, 'fd> {
    /// Send file descriptors.
    ScmRights(&'slice [BorrowedFd<'fd>]),
}

impl SendAncillaryMessage<'_, '_> {
    /// Get the maximum size of an ancillary message.
    ///
    /// This can be helpful in determining the size of the buffer you allocate.
    pub fn size(&self) -> usize {
        let total_bytes = match self {
            Self::ScmRights(slice) => size_of_val(*slice),
        };

        unsafe {
            c::CMSG_SPACE(
                total_bytes
                    .try_into()
                    .expect("size too large for CMSG_SPACE"),
            ) as usize
        }
    }
}

/// Ancillary message for [`recvmsg`].
#[non_exhaustive]
pub enum RecvAncillaryMessage<'a> {
    /// Received file descriptors.
    ScmRights(AncillaryIter<'a, OwnedFd>),
}

/// Buffer for sending ancillary messages.
pub struct SendAncillaryBuffer<'buf, 'slice, 'fd> {
    /// Raw byte buffer for messages.
    buffer: &'buf mut [u8],

    /// The amount of the buffer that is used.
    length: usize,

    /// Phantom data for lifetime of `&'slice [BorrowedFd<'fd>]`.
    _phantom: PhantomData<&'slice [BorrowedFd<'fd>]>,
}

impl<'buf> From<&'buf mut [u8]> for SendAncillaryBuffer<'buf, '_, '_> {
    fn from(buffer: &'buf mut [u8]) -> Self {
        Self::new(buffer)
    }
}

impl Default for SendAncillaryBuffer<'_, '_, '_> {
    fn default() -> Self {
        Self::new(&mut [])
    }
}

impl<'buf, 'slice, 'fd> SendAncillaryBuffer<'buf, 'slice, 'fd> {
    /// Create a new, empty `SendAncillaryBuffer` from a raw byte buffer.
    pub fn new(buffer: &'buf mut [u8]) -> Self {
        Self {
            buffer,
            length: 0,
            _phantom: PhantomData,
        }
    }

    /// Returns a pointer to the message data.
    pub(crate) fn as_control_ptr(&mut self) -> *mut u8 {
        if self.length > 0 {
            self.buffer.as_mut_ptr()
        } else {
            ptr::null_mut()
        }
    }

    /// Returns the length of the message data.
    pub(crate) fn control_len(&self) -> usize {
        self.length
    }

    /// Delete all messages from the buffer.
    pub fn clear(&mut self) {
        self.length = 0;
    }

    /// Add an ancillary message to the buffer.
    ///
    /// Returns `true` if the message was added successfully.
    pub fn push(&mut self, msg: SendAncillaryMessage<'slice, 'fd>) -> bool {
        match msg {
            SendAncillaryMessage::ScmRights(fds) => {
                let fds_bytes =
                    unsafe { slice::from_raw_parts(fds.as_ptr().cast::<u8>(), size_of_val(fds)) };
                self.push_ancillary(fds_bytes, c::SOL_SOCKET as _, c::SCM_RIGHTS as _)
            }
        }
    }

    /// Pushes an ancillary message to the buffer.
    fn push_ancillary(&mut self, source: &[u8], cmsg_level: c::c_int, cmsg_type: c::c_int) -> bool {
        macro_rules! leap {
            ($e:expr) => {{
                match ($e) {
                    Some(x) => x,
                    None => return false,
                }
            }};
        }

        // Calculate the length of the message.
        let source_len = leap!(u32::try_from(source.len()).ok());

        // Calculate the new length of the buffer.
        let additional_space = unsafe { c::CMSG_SPACE(source_len) };
        let new_length = leap!(self.length.checked_add(additional_space as usize));
        let buffer = leap!(self.buffer.get_mut(..new_length));

        // Fill the new part of the buffer with zeroes.
        // TODO: In Rust 1.50 we can use `fill` here.
        unsafe {
            buffer
                .as_mut_ptr()
                .add(self.length)
                .write_bytes(0, new_length - self.length);
        }
        self.length = new_length;

        // Get the last header in the buffer.
        let last_header = leap!(messages::Messages::new(buffer).last());

        // Set the header fields.
        last_header.cmsg_len = unsafe { c::CMSG_LEN(source_len) } as _;
        last_header.cmsg_level = cmsg_level;
        last_header.cmsg_type = cmsg_type;

        // Get the pointer to the payload and copy the data.
        unsafe {
            let payload = c::CMSG_DATA(last_header);
            ptr::copy_nonoverlapping(source.as_ptr(), payload, source_len as _);
        }

        true
    }
}

impl<'slice, 'fd> Extend<SendAncillaryMessage<'slice, 'fd>>
    for SendAncillaryBuffer<'_, 'slice, 'fd>
{
    fn extend<T: IntoIterator<Item = SendAncillaryMessage<'slice, 'fd>>>(&mut self, iter: T) {
        // TODO: This could be optimized to add every message in one go.
        iter.into_iter().all(|msg| self.push(msg));
    }
}

/// Buffer for receiving ancillary messages.
pub struct RecvAncillaryBuffer<'buf> {
    /// Raw byte buffer for messages.
    buffer: &'buf mut [u8],

    /// The portion of the buffer we've read from already.
    read: usize,

    /// The amount of the buffer that is used.
    length: usize,
}

impl<'buf> From<&'buf mut [u8]> for RecvAncillaryBuffer<'buf> {
    fn from(buffer: &'buf mut [u8]) -> Self {
        Self::new(buffer)
    }
}

impl Default for RecvAncillaryBuffer<'_> {
    fn default() -> Self {
        Self::new(&mut [])
    }
}

impl<'buf> RecvAncillaryBuffer<'buf> {
    /// Create a new, empty `RecvAncillaryBuffer` from a raw byte buffer.
    pub fn new(buffer: &'buf mut [u8]) -> Self {
        Self {
            buffer,
            read: 0,
            length: 0,
        }
    }

    /// Returns a pointer to the message data.
    pub(crate) fn as_control_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    /// Returns the length of the message data.
    pub(crate) fn control_len(&self) -> usize {
        self.buffer.len()
    }

    /// Set the length of the message data.
    ///
    /// # Safety
    ///
    /// The buffer must be filled with valid message data.
    pub(crate) unsafe fn set_control_len(&mut self, len: usize) {
        self.length = len;
        self.read = 0;
    }

    /// Delete all messages from the buffer.
    pub(crate) fn clear(&mut self) {
        self.drain().for_each(drop);
    }

    /// Drain all messages from the buffer.
    pub fn drain(&mut self) -> AncillaryDrain<'_> {
        AncillaryDrain {
            messages: messages::Messages::new(&mut self.buffer[self.read..][..self.length]),
            read: &mut self.read,
            length: &mut self.length,
        }
    }
}

impl Drop for RecvAncillaryBuffer<'_> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// An iterator that drains messages from a `RecvAncillaryBuffer`.
pub struct AncillaryDrain<'buf> {
    /// Inner iterator over messages.
    messages: messages::Messages<'buf>,

    /// Increment the number of messages we've read.
    read: &'buf mut usize,

    /// Decrement the total length.
    length: &'buf mut usize,
}

impl<'buf> AncillaryDrain<'buf> {
    /// A closure that converts a message into a `RecvAncillaryMessage`.
    fn cvt_msg(
        read: &mut usize,
        length: &mut usize,
        msg: &c::cmsghdr,
    ) -> Option<RecvAncillaryMessage<'buf>> {
        unsafe {
            // Advance the "read" pointer.
            let msg_len = msg.cmsg_len as usize;
            *read += msg_len;
            *length -= msg_len;

            // Get a pointer to the payload.
            let payload = c::CMSG_DATA(msg);
            let payload_len = msg.cmsg_len as usize - c::CMSG_LEN(0) as usize;

            // Get a mutable slice of the payload.
            let payload: &'buf mut [u8] = slice::from_raw_parts_mut(payload, payload_len);

            // Determine what type it is.
            let (level, msg_type) = (msg.cmsg_level, msg.cmsg_type);
            match (level as _, msg_type as _) {
                (c::SOL_SOCKET, c::SCM_RIGHTS) => {
                    // Create an iterator that reads out the file descriptors.
                    let fds = AncillaryIter::new(payload);

                    Some(RecvAncillaryMessage::ScmRights(fds))
                }
                _ => None,
            }
        }
    }
}

impl<'buf> Iterator for AncillaryDrain<'buf> {
    type Item = RecvAncillaryMessage<'buf>;

    fn next(&mut self) -> Option<Self::Item> {
        let read = &mut self.read;
        let length = &mut self.length;
        self.messages.find_map(|ev| Self::cvt_msg(read, length, ev))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, max) = self.messages.size_hint();
        (0, max)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let read = self.read;
        let length = self.length;
        self.messages
            .filter_map(|ev| Self::cvt_msg(read, length, ev))
            .fold(init, f)
    }

    fn count(self) -> usize {
        let read = self.read;
        let length = self.length;
        self.messages
            .filter_map(|ev| Self::cvt_msg(read, length, ev))
            .count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let read = self.read;
        let length = self.length;
        self.messages
            .filter_map(|ev| Self::cvt_msg(read, length, ev))
            .last()
    }

    fn collect<B: FromIterator<Self::Item>>(self) -> B
    where
        Self: Sized,
    {
        let read = self.read;
        let length = self.length;
        self.messages
            .filter_map(|ev| Self::cvt_msg(read, length, ev))
            .collect()
    }
}

impl FusedIterator for AncillaryDrain<'_> {}

/// `sendmsg(msghdr)`—Sends a message on a socket.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendmsg.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendmsg.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendmsg&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendmsg.2
/// [OpenBSD]: https://man.openbsd.org/sendmsg.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendmsg&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendmsg
#[inline]
pub fn sendmsg_noaddr(
    socket: impl AsFd,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    flags: SendFlags,
) -> io::Result<usize> {
    backend::net::syscalls::sendmsg_noaddr(socket.as_fd(), iov, control, flags)
}

/// `sendmsg(msghdr)`—Sends a message on a socket to a specific IPv4 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendmsg.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendmsg.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendmsg&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendmsg.2
/// [OpenBSD]: https://man.openbsd.org/sendmsg.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendmsg&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendmsg
#[inline]
pub fn sendmsg_v4(
    socket: impl AsFd,
    addr: &SocketAddrV4,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    flags: SendFlags,
) -> io::Result<usize> {
    backend::net::syscalls::sendmsg_v4(socket.as_fd(), addr, iov, control, flags)
}

/// `sendmsg(msghdr)`—Sends a message on a socket to a specific IPv6 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendmsg.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendmsg.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendmsg&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendmsg.2
/// [OpenBSD]: https://man.openbsd.org/sendmsg.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendmsg&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendmsg
#[inline]
pub fn sendmsg_v6(
    socket: impl AsFd,
    addr: &SocketAddrV6,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    flags: SendFlags,
) -> io::Result<usize> {
    backend::net::syscalls::sendmsg_v6(socket.as_fd(), addr, iov, control, flags)
}

/// `sendmsg(msghdr)`—Sends a message on a socket to a specific Unix-domain
/// address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendmsg.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendmsg.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendmsg&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendmsg.2
/// [OpenBSD]: https://man.openbsd.org/sendmsg.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendmsg&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendmsg
#[inline]
#[cfg(unix)]
pub fn sendmsg_unix(
    socket: impl AsFd,
    addr: &super::SocketAddrUnix,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    flags: SendFlags,
) -> io::Result<usize> {
    backend::net::syscalls::sendmsg_unix(socket.as_fd(), addr, iov, control, flags)
}

/// `sendmsg(msghdr)`—Sends a message on a socket to a specific address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendmsg.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendmsg.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendmsg&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendmsg.2
/// [OpenBSD]: https://man.openbsd.org/sendmsg.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendmsg&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendmsg
#[inline]
pub fn sendmsg_any(
    socket: impl AsFd,
    addr: Option<&SocketAddrAny>,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    flags: SendFlags,
) -> io::Result<usize> {
    match addr {
        None => backend::net::syscalls::sendmsg_noaddr(socket.as_fd(), iov, control, flags),
        Some(SocketAddrAny::V4(addr)) => {
            backend::net::syscalls::sendmsg_v4(socket.as_fd(), addr, iov, control, flags)
        }
        Some(SocketAddrAny::V6(addr)) => {
            backend::net::syscalls::sendmsg_v6(socket.as_fd(), addr, iov, control, flags)
        }
        #[cfg(unix)]
        Some(SocketAddrAny::Unix(addr)) => {
            backend::net::syscalls::sendmsg_unix(socket.as_fd(), addr, iov, control, flags)
        }
    }
}

/// `recvmsg(msghdr)`—Receives a message from a socket.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvmsg.html
/// [Linux]: https://man7.org/linux/man-pages/man2/recvmsg.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/recvmsg.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=recvmsg&sektion=2
/// [NetBSD]: https://man.netbsd.org/recvmsg.2
/// [OpenBSD]: https://man.openbsd.org/recvmsg.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=recvmsg&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/recvmsg
#[inline]
pub fn recvmsg(
    socket: impl AsFd,
    iov: &mut [IoSliceMut<'_>],
    control: &mut RecvAncillaryBuffer<'_>,
    flags: RecvFlags,
) -> io::Result<RecvMsgReturn> {
    backend::net::syscalls::recvmsg(socket.as_fd(), iov, control, flags)
}

/// The result of a successful [`recvmsg`] call.
pub struct RecvMsgReturn {
    /// The number of bytes received.
    pub bytes: usize,

    /// The flags received.
    pub flags: RecvFlags,

    /// The address of the socket we received from, if any.
    pub address: Option<SocketAddrAny>,
}

/// An iterator over data in an ancillary buffer.
pub struct AncillaryIter<'data, T> {
    /// The data we're iterating over.
    data: &'data mut [u8],

    /// The raw data we're removing.
    _marker: PhantomData<T>,
}

impl<'data, T> AncillaryIter<'data, T> {
    /// Create a new iterator over data in an ancillary buffer.
    ///
    /// # Safety
    ///
    /// The buffer must contain valid ancillary data.
    unsafe fn new(data: &'data mut [u8]) -> Self {
        assert_eq!(data.len() % size_of::<T>(), 0);

        Self {
            data,
            _marker: PhantomData,
        }
    }
}

impl<'data, T> Drop for AncillaryIter<'data, T> {
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<T> Iterator for AncillaryIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // See if there is a next item.
        if self.data.len() < size_of::<T>() {
            return None;
        }

        // Get the next item.
        let item = unsafe { self.data.as_ptr().cast::<T>().read_unaligned() };

        // Move forward.
        let data = take(&mut self.data);
        self.data = &mut data[size_of::<T>()..];

        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<T> FusedIterator for AncillaryIter<'_, T> {}

impl<T> ExactSizeIterator for AncillaryIter<'_, T> {
    fn len(&self) -> usize {
        self.data.len() / size_of::<T>()
    }
}

impl<T> DoubleEndedIterator for AncillaryIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // See if there is a next item.
        if self.data.len() < size_of::<T>() {
            return None;
        }

        // Get the next item.
        let item = unsafe {
            let ptr = self.data.as_ptr().add(self.data.len() - size_of::<T>());
            ptr.cast::<T>().read_unaligned()
        };

        // Move forward.
        let len = self.data.len();
        let data = take(&mut self.data);
        self.data = &mut data[..len - size_of::<T>()];

        Some(item)
    }
}

mod messages {
    use crate::backend::c;
    use core::convert::TryInto;
    use core::iter::FusedIterator;
    use core::marker::PhantomData;
    use core::mem::zeroed;
    use core::ptr::NonNull;

    /// An iterator over the messages in an ancillary buffer.
    pub(super) struct Messages<'buf> {
        /// The message header we're using to iterator over the messages.
        msghdr: c::msghdr,

        /// The current pointer to the next message header to return.
        ///
        /// This has a lifetime of `'buf`.
        header: Option<NonNull<c::cmsghdr>>,

        /// Capture the original lifetime of the buffer.
        _buffer: PhantomData<&'buf mut [u8]>,
    }

    impl<'buf> Messages<'buf> {
        /// Create a new iterator over messages from a byte buffer.
        pub(super) fn new(buf: &'buf mut [u8]) -> Self {
            let msghdr = {
                let mut h: c::msghdr = unsafe { zeroed() };
                h.msg_control = buf.as_mut_ptr().cast();
                h.msg_controllen = buf.len().try_into().expect("buffer too large for msghdr");
                h
            };

            // Get the first header.
            let header = NonNull::new(unsafe { c::CMSG_FIRSTHDR(&msghdr) });

            Self {
                msghdr,
                header,
                _buffer: PhantomData,
            }
        }
    }

    impl<'a> Iterator for Messages<'a> {
        type Item = &'a mut c::cmsghdr;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            // Get the current header.
            let header = self.header?;

            // Get the next header.
            self.header = NonNull::new(unsafe { c::CMSG_NXTHDR(&self.msghdr, header.as_ptr()) });

            // If the headers are equal, we're done.
            if Some(header) == self.header {
                self.header = None;
            }

            // SAFETY: The lifetime of `header` is tied to this.
            Some(unsafe { &mut *header.as_ptr() })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            if self.header.is_some() {
                // The remaining buffer *could* be filled with zero-length
                // messages.
                let max_size = unsafe { c::CMSG_LEN(0) } as usize;
                let remaining_count = self.msghdr.msg_controllen as usize / max_size;
                (1, Some(remaining_count))
            } else {
                (0, Some(0))
            }
        }
    }

    impl FusedIterator for Messages<'_> {}
}
